use crate::counters::{Counter, Value, modify};
use crate::download::add_images;
use crate::misc::{
    Equipment, adjust_meshes, default_cam_pos, make_cam, make_cur, new_pile_at, remove_follow,
    repaint_face, spawn_equip,
};
#[cfg(feature = "steam")]
use crate::setup::SteamInfo;
use crate::setup::{MAT_HEIGHT, MAT_WIDTH, SideMenu, TextChat};
use crate::shapes::{Shape, Side};
use crate::update::*;
use crate::*;
#[cfg(feature = "steam")]
use bevy::diagnostic::FrameCount;
use bevy::ecs::system::SystemParam;
use bevy::window::PrimaryWindow;
use bevy_rand::global::GlobalRng;
use bevy_rich_text3d::Text3d;
use bevy_tangled::{ClientMode, ClientTrait, ClientTypeRef, Compression, PeerId, Reliability};
use bevy_ui_text_input::TextInputContents;
use bitcode::{Decode, Encode};
#[cfg(feature = "mic")]
use rodio::buffer::SamplesBuffer;
use std::collections::hash_map::Entry::Vacant;
use std::collections::{HashMap, HashSet};
use std::mem;
use std::sync::atomic::AtomicBool;
pub const COMPRESSION: Compression = Compression::Uncompressed;
#[derive(Resource, Default, Deref, DerefMut)]
pub struct SendSleeping(pub Arc<AtomicBool>);
pub fn get_sync(
    query: Query<(
        &SyncObjectMe,
        &GlobalTransform,
        &LinearVelocity,
        &AngularVelocity,
        Option<&InHand>,
        Option<&Sleeping>,
        Option<&FollowMouse>,
    )>,
    mut count: ResMut<SyncCount>,
    mut client: ResMut<Client>,
    send_sleep: Res<SendSleeping>,
    camera: Single<(&Camera, &GlobalTransform), (With<Camera3d>, Without<SyncObjectMe>)>,
    window: Single<&Window, With<PrimaryWindow>>,
    spatial: SpatialQuery,
    keybinds: Keybinds,
) {
    #[allow(unused_variables)]
    if let Err(e) = client.update() {
        #[cfg(feature = "steam")]
        warn!("{e}")
    }
    if !client.is_connected() {
        return;
    }
    let send_sleep = send_sleep
        .0
        .swap(false, std::sync::atomic::Ordering::Relaxed);
    let mut vec = count.take();
    for (id, transform, vel, ang, in_hand, is_sleep, follow) in query {
        if send_sleep || is_sleep.is_none() || follow.is_some() {
            vec.push((
                *id,
                Trans::from(transform),
                Phys::from(vel, ang),
                in_hand.is_some(),
                follow.is_some(),
            ))
        }
    }
    if !vec.is_empty() {
        let packet = Packet::Pos(vec);
        client
            .broadcast(&packet, Reliability::Reliable, COMPRESSION)
            .unwrap();
        let Packet::Pos(mut vec) = packet else {
            unreachable!()
        };
        vec.clear();
        count.give(vec);
    } else {
        count.give(vec);
    }
    let (camera, camera_transform) = camera.into_inner();
    fn get_dest(
        camera: &Camera,
        camera_transform: &GlobalTransform,
        window: Single<&Window, With<PrimaryWindow>>,
        spatial: SpatialQuery,
    ) -> Option<Pos> {
        let cursor_position = window.cursor_position()?;
        let ray = camera
            .viewport_to_world(camera_transform, cursor_position)
            .ok()?;
        let hit = spatial.cast_ray(
            ray.origin,
            ray.direction,
            f32::MAX,
            true,
            &SpatialQueryFilter::DEFAULT,
        );
        if let Some(hit) = hit {
            let cur = ray.origin + ray.direction * hit.distance;
            Some(cur.into())
        } else {
            None
        }
    }
    let cam = camera_transform.translation();
    client
        .broadcast(
            &Packet::Indicator(
                cam.into(),
                get_dest(camera, camera_transform, window, spatial),
                keybinds.pressed(Keybind::Ping),
            ),
            Reliability::Reliable,
            COMPRESSION,
        )
        .unwrap();
    #[cfg(feature = "steam")]
    client.flush();
}
#[cfg(feature = "steam")]
pub fn display_steam_info(
    frame: Res<FrameCount>,
    mut text: Single<(&mut Node, &mut Text), With<SteamInfo>>,
    client: Res<Client>,
    menu: Res<Menu>,
) {
    if matches!(*menu, Menu::Esc) || !frame.0.is_multiple_of(20) {
        return;
    }
    let info = client.info();
    let info = info
        .0
        .into_iter()
        .map(|(p, a)| {
            format!(
                "{p}: {} {} {}",
                a.ping(),
                a.in_bytes_per_sec(),
                a.out_bytes_per_sec(),
            )
        })
        .collect::<Vec<String>>();
    let Val::Px(width) = text.0.width else {
        unreachable!()
    };
    let Val::Px(height) = text.0.height else {
        unreachable!()
    };
    let max = info.iter().map(|a| a.len()).max().unwrap_or(0) as f32 * FONT_WIDTH;
    if max > width {
        text.0.width = Val::Px(max);
    }
    let max = info.len() as f32 * FONT_HEIGHT;
    if max > height {
        text.0.height = Val::Px(max);
    }
    text.1.0 = info.join("\n");
}
pub fn apply_sync(
    mut query: Query<
        (
            &mut SyncObject,
            &mut Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
            Entity,
            Option<&InOtherHand>,
            Option<&Children>,
            Option<&mut Pile>,
            &mut GravityScale,
            Option<&FollowOtherMouse>,
        ),
        (With<Children>, Without<SyncObjectMe>),
    >,
    mut queryme: Query<
        (
            &SyncObjectMe,
            &GlobalTransform,
            Option<&mut Pile>,
            Entity,
            Option<&Children>,
            Option<&InHand>,
            &mut Transform,
        ),
        (Without<SyncObject>, Or<(Without<ChildOf>, With<InHand>)>),
    >,
    mut sent: ResMut<Sent>,
    asset_server: Res<AssetServer>,
    down: Res<Download>,
    mut commands: Commands,
    mut hand: Single<(Entity, &mut Hand)>,
    mut mats: Query<&mut MeshMaterial3d<StandardMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    card_base: Res<CardBase>,
    mut count: ResMut<SyncCount>,
    mut client: ResMut<Client>,
    #[cfg(feature = "mic")] (sink, audio, audio_settings): (
        Res<AudioPlayer>,
        Res<AudioResource>,
        Res<AudioSettings>,
    ),
    (mut query_meshes, chat, mut drag, mut colliders, cards): (
        Query<
            (&mut Mesh3d, &mut Transform),
            (
                Without<Children>,
                With<ChildOf>,
                Without<InHand>,
                Without<Shape>,
                Without<Pile>,
                Without<Side>,
            ),
        >,
        Single<Entity, With<TextChat>>,
        Query<
            (Entity, &PingDrag, &mut Mesh3d, &mut Transform, &PeerId),
            (
                Without<ChildOf>,
                Without<Pile>,
                Without<Shape>,
                Without<SyncObject>,
                Without<SyncObjectMe>,
                Without<CursorInd>,
                Without<CameraInd>,
            ),
        >,
        Query<&mut Collider>,
        Res<CardList>,
    ),
    (
        search,
        text,
        mut shape,
        mut text3d,
        mut cams,
        mut curs,
        mut peers,
        cam,
        equipment,
        counters,
        side,
        mut menu,
        mut turn,
    ): (
        Option<Single<(Entity, &SearchDeck)>>,
        Option<Single<&TextInputContents, With<SearchText>>>,
        Query<&mut Shape>,
        Query<&mut Text3d>,
        Query<
            (&CameraInd, &mut Transform),
            (
                Without<SyncObject>,
                Without<SyncObjectMe>,
                Without<ChildOf>,
                Without<CursorInd>,
            ),
        >,
        Query<
            (&mut CursorInd, &mut Transform, Entity),
            (
                Without<SyncObject>,
                Without<SyncObjectMe>,
                Without<ChildOf>,
                Without<CameraInd>,
            ),
        >,
        ResMut<Peers>,
        Single<
            &mut Transform,
            (
                With<Camera3d>,
                Without<SyncObject>,
                Without<SyncObjectMe>,
                Without<ChildOf>,
                Without<CursorInd>,
                Without<CameraInd>,
                Without<PeerId>,
            ),
        >,
        Query<(), With<Equipment>>,
        Query<&Counter>,
        Option<Single<Entity, With<SideMenu>>>,
        ResMut<Menu>,
        ResMut<Turn>,
    ),
) {
    if !client.is_connected() {
        return;
    }
    let mut cam = cam.into_inner();
    let mut new = HashMap::new();
    let mut ind = false;
    let mut ignore = HashSet::new();
    client.recv(|client, packet| {
        let sender = packet.src;
        let data = packet.data;
        match data {
            Packet::Pos(data) => {
                let user = sender;
                for (lid, trans, phys, in_hand, follow_mouse) in data {
                    let id = SyncObject { user, id: lid };
                    if sent.has(&id) || ignore.contains(&id) {
                        continue;
                    }
                    if let Some((
                        mut t,
                        hand,
                        children,
                        entity,
                        pile,
                        mut gravity,
                        mut lv,
                        mut av,
                        followother,
                    )) = query
                        .iter_mut()
                        .find_map(|(a, b, lv, av, e, h, c, p, g, f)| {
                            if *a == id {
                                Some((b, h, c, e, p, g, lv, av, f))
                            } else {
                                None
                            }
                        })
                    {
                        let (lvt, avt) = phys.to();
                        *lv = lvt;
                        *av = avt;
                        t.translation = trans.translation.into();
                        t.rotation = trans.rotation.into();
                        if let Some(pile) = pile
                            && in_hand != hand.is_some()
                            && let Some(children) = children
                        {
                            if in_hand {
                                let mut ent = commands.entity(entity);
                                ent.insert(InOtherHand);
                                ent.insert(SleepingDisabled);
                                ent.insert(RigidBodyDisabled);
                                mats.get_mut(*children.first().unwrap()).unwrap().0 =
                                    mats.get_mut(*children.get(1).unwrap()).unwrap().0.clone();
                                gravity.0 = 0.0
                            } else {
                                let mut ent = commands.entity(entity);
                                ent.remove::<InOtherHand>();
                                ent.remove::<SleepingDisabled>();
                                ent.remove::<RigidBodyDisabled>();
                                repaint_face(&mut mats, &mut materials, pile.first(), children);
                                gravity.0 = GRAVITY;
                            }
                        }
                        if followother.is_some() != follow_mouse {
                            if follow_mouse {
                                let mut ent = commands.entity(entity);
                                ent.insert(SleepingDisabled);
                                ent.insert(FollowOtherMouse);
                                gravity.0 = 0.0
                            } else {
                                let mut ent = commands.entity(entity);
                                ent.remove::<SleepingDisabled>();
                                ent.remove::<FollowOtherMouse>();
                                gravity.0 = GRAVITY
                            }
                        }
                    } else if sent.add(id) {
                        client
                            .send(
                                sender,
                                &Packet::Request(lid),
                                Reliability::Reliable,
                                COMPRESSION,
                            )
                            .unwrap();
                    }
                }
            }
            Packet::Take(from, to) => {
                let new = SyncObject {
                    user: sender,
                    id: to,
                };
                ignore.insert(new);
                if from.user == client.my_id() {
                    client
                        .broadcast(
                            &Packet::Received(from.id),
                            Reliability::Reliable,
                            COMPRESSION,
                        )
                        .unwrap();
                    if let Some((_, _, _, e, _, inhand, _)) = queryme
                        .iter()
                        .find(|(id, _, _, _, _, _, _)| **id == from.id)
                    {
                        if let Some(inhand) = inhand {
                            hand.1.count -= 1;
                            hand.1.removed.push(inhand.0);
                        }
                        count.rem(1);
                        remove_follow(&mut commands, e);
                        commands
                            .entity(e)
                            .remove::<InHand>()
                            .remove::<RigidBodyDisabled>()
                            .remove::<SyncObjectMe>()
                            .insert(new)
                            .insert(HandIgnore)
                            .remove_parent_in_place();
                    }
                } else if let Some((mut id, _, _, _, _, _, _, _, _, _)) = query
                    .iter_mut()
                    .find(|(id, _, _, _, _, _, _, _, _, _)| *id.as_ref() == from)
                {
                    sent.add(*id);
                    *id = new
                } else if sent.add(new) {
                    client
                        .send(
                            sender,
                            &Packet::Request(new.id),
                            Reliability::Reliable,
                            COMPRESSION,
                        )
                        .unwrap();
                }
            }
            Packet::Request(lid) => {
                let user = sender;
                let id = SyncObject { user, id: lid };
                if sent.add(id) {
                    if let Some((b, c, e)) = queryme.iter_mut().find_map(|(a, b, c, e, _, _, _)| {
                        if a.0 == lid.0 { Some((b, c, e)) } else { None }
                    }) {
                        if let Some(c) = c {
                            client
                                .send(
                                    sender,
                                    &Packet::New(
                                        lid,
                                        c.clone_no_image(),
                                        Trans::from(b),
                                        b.scale().x,
                                    ),
                                    Reliability::Reliable,
                                    COMPRESSION,
                                )
                                .unwrap();
                        } else if let Ok(s) = shape.get(e) {
                            client
                                .send(
                                    sender,
                                    &Packet::NewShape(lid, s.clone(), Trans::from(b), b.scale().x),
                                    Reliability::Reliable,
                                    COMPRESSION,
                                )
                                .unwrap();
                        }
                    } else {
                        sent.rem(&id);
                    }
                }
            }
            Packet::Received(lid) => {
                let user = sender;
                let id = SyncObject { user, id: lid };
                sent.del(id);
            }
            Packet::New(lid, mut pile, trans, scale) => {
                let user = sender;
                let id = SyncObject { user, id: lid };
                let mut some = false;
                for c in pile.iter_mut() {
                    if let Some(d) = cards.get(&c.id) {
                        c.data.face.image = d.face.image.clone();
                        if let Some(c) = c.data.back.as_mut() {
                            if let Some(d) = d.back.as_ref() {
                                c.image = d.image.clone();
                            } else {
                                return;
                            }
                        }
                    } else {
                        some = true;
                    }
                }
                if some {
                    let deck = down.get_deck.clone();
                    let client = down.client.0.clone();
                    let asset_server = asset_server.clone();
                    let f = async move {
                        add_images(
                            pile,
                            trans.with_scale(scale),
                            id,
                            deck,
                            client,
                            asset_server,
                        )
                        .await;
                    };
                    #[cfg(feature = "wasm")]
                    wasm_bindgen_futures::spawn_local(f);
                    #[cfg(not(feature = "wasm"))]
                    down.runtime.0.spawn(f);
                } else {
                    //TODO
                }
            }
            Packet::NewShape(lid, shape, trans, scale) => {
                client
                    .send(
                        sender,
                        &Packet::Received(lid),
                        Reliability::Reliable,
                        COMPRESSION,
                    )
                    .unwrap();
                let user = sender;
                let id = SyncObject { user, id: lid };
                sent.del(id);
                ignore.insert(id);
                new.insert(
                    id,
                    shape
                        .create(
                            trans.with_scale(scale),
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            Color::WHITE,
                        )
                        .insert(id)
                        .id(),
                );
            }
            Packet::SetUser(peer, id) => {
                if peer == client.my_id() {
                    for (id, _, _, e, _, _, _) in queryme.iter() {
                        if shape.contains(e) {
                            client
                                .broadcast(
                                    &Packet::Dead(SyncObject {
                                        user: client.my_id(),
                                        id: *id,
                                    }),
                                    Reliability::Reliable,
                                    COMPRESSION,
                                )
                                .unwrap();
                            commands.entity(e).despawn()
                        }
                    }
                    info!("joined as number {} user", id);
                    commands.entity(hand.0).despawn();
                    spawn_hand(id, &mut commands);
                    peers.me = Some(id);
                    *cam = default_cam_pos(peers.me.unwrap_or_default());
                    if matches!(client.mode(), ClientMode::Ip) {
                        client
                            .broadcast(
                                &Packet::Name(peers.name.clone().unwrap()),
                                Reliability::Reliable,
                                COMPRESSION,
                            )
                            .unwrap();
                    }
                } else if matches!(client.mode(), ClientMode::Steam)
                    && let Some(name) = client.get_name_of(peer)
                {
                    peers.names.insert(peer, name);
                }
                peers.map().insert(peer, id);
            }
            Packet::Name(name) => {
                peers.names.insert(sender, name);
            }
            Packet::Dead(id) => {
                if id.user == client.my_id()
                    && let Some(e) = queryme
                        .iter()
                        .find_map(|(a, _, _, e, _, _, _)| if *a == id.id { Some(e) } else { None })
                {
                    count.rem(1);
                    commands.entity(e).despawn();
                    if let Some(search) = &search
                        && search.1.0 == e
                    {
                        *menu = Menu::World;
                        commands.entity(**side.as_ref().unwrap()).despawn();
                    }
                } else if let Some(e) = query.iter().find_map(
                    |(a, _, _, _, b, _, _, _, _, _)| if *a == id { Some(b) } else { None },
                ) {
                    commands.entity(e).despawn();
                    if let Some(search) = &search
                        && search.1.0 == e
                    {
                        *menu = Menu::World;
                        commands.entity(**side.as_ref().unwrap()).despawn();
                    }
                } else if let Some(ent) = new.get(&id) {
                    commands.entity(*ent).despawn();
                }
            }
            Packet::Equip(id) => {
                if id.user == client.my_id()
                    && let Some((pile, entity, children, mut transform)) =
                        queryme.iter_mut().find_map(|(a, _, p, e, c, _, t)| {
                            if *a == id.id {
                                Some((p, e, c, t))
                            } else {
                                None
                            }
                        })
                    && let Some(children) = children
                    && let Some(mut pile) = pile
                {
                    let b = pile.equip();
                    repaint_face(&mut mats, &mut materials, pile.last(), children);
                    adjust_meshes(
                        &pile,
                        children,
                        &mut meshes,
                        &mut query_meshes,
                        &mut transform,
                        &mut colliders.get_mut(entity).unwrap(),
                        &equipment,
                        Some(&counters),
                        &mut commands,
                    );
                    if b {
                        spawn_equip(
                            entity,
                            &pile,
                            &mut commands,
                            card_base.clone(),
                            &mut materials,
                            &mut meshes,
                        );
                    }
                } else if let Some((pile, entity, children, mut transform)) =
                    query.iter_mut().find_map(
                        |(a, t, _, _, e, _, c, p, _, _)| {
                            if *a == id { Some((p, e, c, t)) } else { None }
                        },
                    )
                    && let Some(children) = children
                    && let Some(mut pile) = pile
                {
                    let b = pile.equip();
                    repaint_face(&mut mats, &mut materials, pile.last(), children);
                    adjust_meshes(
                        &pile,
                        children,
                        &mut meshes,
                        &mut query_meshes,
                        &mut transform,
                        &mut colliders.get_mut(entity).unwrap(),
                        &equipment,
                        None,
                        &mut commands,
                    );
                    if b {
                        spawn_equip(
                            entity,
                            &pile,
                            &mut commands,
                            card_base.clone(),
                            &mut materials,
                            &mut meshes,
                        );
                    }
                }
            }
            Packet::Modify(id, counter, value) => {
                let (card, entity, children) = if id.user == client.my_id()
                    && let Some((pile, entity, children)) =
                        queryme.iter_mut().find_map(|(a, _, p, e, c, _, _)| {
                            if *a == id.id { Some((p, e, c)) } else { None }
                        })
                    && let Some(children) = children
                    && let Some(pile) = pile
                    && let Pile::Single(card) = pile.into_inner()
                {
                    (card, entity, children)
                } else if let Some((pile, entity, children)) = query.iter_mut().find_map(
                    |(a, _, _, _, e, _, c, p, _, _)| {
                        if *a == id { Some((p, e, c)) } else { None }
                    },
                ) && let Some(children) = children
                    && let Some(pile) = pile
                    && let Pile::Single(card) = pile.into_inner()
                {
                    (card, entity, children)
                } else if sent.add(id) {
                    client
                        .send(
                            id.user,
                            &Packet::Request(id.id),
                            Reliability::Reliable,
                            COMPRESSION,
                        )
                        .unwrap();
                    return;
                } else {
                    return;
                };
                match counter {
                    Counter::Power => card.power = value,
                    Counter::Toughness => card.toughness = value,
                    Counter::Loyalty => card.loyalty = value,
                    Counter::Counters => card.counters = value,
                    Counter::Misc => card.misc = value,
                };
                modify(
                    entity,
                    card,
                    children,
                    &mut commands,
                    counters,
                    &mut materials,
                    &mut meshes,
                    counter,
                );
            }
            Packet::Flip(id, idx, rev) => {
                if id.user == client.my_id()
                    && let Some((pile, children, entity, transform)) =
                        queryme.iter_mut().find_map(|(a, _, p, e, c, _, t)| {
                            if *a == id.id {
                                Some((p, c, e, t))
                            } else {
                                None
                            }
                        })
                    && let Some(children) = children
                    && let Some(mut pile) = pile
                {
                    let last = idx == pile.len() - 1;
                    if let Some(card) = pile.get_mut(idx)
                        && card.data.back.is_some()
                        && card.flipped != rev
                    {
                        card.flipped = !card.flipped;
                        if last {
                            repaint_face(&mut mats, &mut materials, card, children);
                        }
                        if let Some(entity) = search
                            .as_ref()
                            .and_then(|s| if s.1.0 == entity { Some(s.0) } else { None })
                        {
                            update_search(
                                &mut commands,
                                entity,
                                &pile,
                                &transform,
                                text.as_ref().unwrap().get(),
                                &side,
                                &mut menu,
                            );
                        }
                    }
                } else if let Some((children, mut pile, entity, transform)) =
                    query.iter_mut().find_map(
                        |(a, t, _, _, e, _, c, d, _, _)| {
                            if *a == id { Some((c, d, e, t)) } else { None }
                        },
                    )
                    && let Some(pile) = &mut pile
                    && let Some(children) = children
                {
                    let last = idx == pile.len() - 1;
                    if let Some(card) = pile.get_mut(idx) {
                        if card.data.back.is_some() && card.flipped != rev {
                            card.flipped = !card.flipped;
                            if last {
                                repaint_face(&mut mats, &mut materials, card, children);
                            }
                            if let Some(entity) = search
                                .as_ref()
                                .and_then(|s| if s.1.0 == entity { Some(s.0) } else { None })
                            {
                                update_search(
                                    &mut commands,
                                    entity,
                                    pile,
                                    &transform,
                                    text.as_ref().unwrap().get(),
                                    &side,
                                    &mut menu,
                                );
                            }
                        }
                    } else {
                        commands.entity(entity).despawn();
                        if sent.add(id) {
                            client
                                .send(
                                    id.user,
                                    &Packet::Request(id.id),
                                    Reliability::Reliable,
                                    COMPRESSION,
                                )
                                .unwrap();
                        }
                    }
                } else if sent.add(id) {
                    client
                        .send(
                            id.user,
                            &Packet::Request(id.id),
                            Reliability::Reliable,
                            COMPRESSION,
                        )
                        .unwrap();
                }
            }
            Packet::Reorder(id, order) => {
                let run =
                    |pile: &mut Pile, children: _, transform: &Transform, entity: _, ask: bool| {
                        let mut fail = false;
                        if let Pile::Multiple(pile) = pile {
                            for (i, id) in order.into_iter().enumerate() {
                                if let Some(k) = pile[i..].iter().position(|c| c.id == id) {
                                    pile.swap(i, k + i);
                                } else {
                                    fail = true;
                                    break;
                                }
                            }
                        } else {
                            fail = true;
                        }
                        if fail && ask {
                            if let Some(search) = &search
                                && search.1.0 == entity
                            {
                                commands.entity(search.0).despawn()
                            }
                            commands.entity(entity).despawn();
                            if sent.add(id) {
                                client
                                    .send(
                                        id.user,
                                        &Packet::Request(id.id),
                                        Reliability::Reliable,
                                        COMPRESSION,
                                    )
                                    .unwrap();
                            }
                            return;
                        }
                        if let Some(search) = &search
                            && search.1.0 == entity
                        {
                            update_search(
                                &mut commands,
                                search.0,
                                pile,
                                transform,
                                text.as_ref().unwrap().get(),
                                &side,
                                &mut menu,
                            );
                        }
                        let card = pile.last();
                        repaint_face(&mut mats, &mut materials, card, children);
                    };
                if id.user == client.my_id()
                    && let Some((mut pile, children, transform, entity)) =
                        queryme.iter_mut().find_map(|(a, _, p, e, c, _, t)| {
                            if *a == id.id {
                                Some((p, c, t, e))
                            } else {
                                None
                            }
                        })
                    && let Some(pile) = &mut pile
                    && let Some(children) = children
                {
                    run(pile, children, &transform, entity, false)
                } else if let Some((pile, children, entity, transform)) = query.iter_mut().find_map(
                    |(a, t, _, _, e, _, c, d, _, _)| {
                        if *a == id { Some((d, c, e, t)) } else { None }
                    },
                ) && let Some(mut pile) = pile
                    && let Some(children) = children
                {
                    run(&mut pile, children, &transform, entity, true)
                }
            }
            #[allow(unused_variables)]
            Packet::Move(from, to, count, from_top, to_top) => {
                //TODO
            }
            Packet::Merge(base, from, at) => {
                let (top_pile, top_ent) = if from.user == client.my_id()
                    && let Some((pile, ent)) = queryme.iter_mut().find_map(
                        |(a, _, p, e, _, _, _)| {
                            if *a == from.id { Some((p, e)) } else { None }
                        },
                    )
                    && let Some(pile) = pile
                {
                    (mem::replace(pile.into_inner(), Pile::Empty), ent)
                } else if let Some((pile, ent)) = query.iter_mut().find_map(
                    |(a, _, _, _, e, _, _, d, _, _)| {
                        if *a == from { Some((d, e)) } else { None }
                    },
                ) && let Some(pile) = pile
                {
                    (mem::replace(pile.into_inner(), Pile::Empty), ent)
                } else if base.user != client.my_id()
                    && let Some(base_ent) = query.iter_mut().find_map(
                        |(a, _, _, _, e, _, _, _, _, _)| {
                            if *a == base { Some(e) } else { None }
                        },
                    )
                {
                    commands.entity(base_ent).despawn();
                    if sent.add(base) {
                        client
                            .send(
                                base.user,
                                &Packet::Request(base.id),
                                Reliability::Reliable,
                                COMPRESSION,
                            )
                            .unwrap();
                    }
                    return;
                } else {
                    return;
                };
                commands.entity(top_ent).despawn();
                let (mut base_pile, base_children, base_ent, mut base_transform) = if base.user
                    == client.my_id()
                    && let Some((pile, children, ent, transform)) =
                        queryme.iter_mut().find_map(|(a, _, p, e, c, _, t)| {
                            if *a == base.id {
                                Some((p, c, e, t))
                            } else {
                                None
                            }
                        })
                    && let Some(pile) = pile
                    && let Some(children) = children
                {
                    count.rem(1);
                    (pile, children, ent, transform)
                } else if let Some((pile, children, ent, transform)) = query.iter_mut().find_map(
                    |(a, t, _, _, e, _, c, d, _, _)| {
                        if *a == base { Some((d, c, e, t)) } else { None }
                    },
                ) && let Some(pile) = pile
                    && let Some(children) = children
                {
                    (pile, children, ent, transform)
                } else {
                    if sent.add(base) {
                        client
                            .send(
                                base.user,
                                &Packet::Request(base.id),
                                Reliability::Reliable,
                                COMPRESSION,
                            )
                            .unwrap();
                    }
                    return;
                };
                if at > base_pile.len() && base.user != client.my_id() {
                    if sent.add(base) {
                        client
                            .send(
                                base.user,
                                &Packet::Request(base.id),
                                Reliability::Reliable,
                                COMPRESSION,
                            )
                            .unwrap();
                    }
                    return;
                }
                let mut equip = false;
                if top_pile.is_modified() {
                    base_pile.merge(top_pile);
                    equip = true;
                } else {
                    base_pile.splice_at(at, top_pile);
                }
                let card = base_pile.last();
                repaint_face(&mut mats, &mut materials, card, base_children);
                adjust_meshes(
                    &base_pile,
                    base_children,
                    &mut meshes,
                    &mut query_meshes,
                    &mut base_transform,
                    &mut colliders.get_mut(base_ent).unwrap(),
                    &equipment,
                    None,
                    &mut commands,
                );
                if equip {
                    spawn_equip(
                        base_ent,
                        &base_pile,
                        &mut commands,
                        card_base.clone(),
                        &mut materials,
                        &mut meshes,
                    );
                }
                if let Some(search) = &search
                    && search.1.0 == base_ent
                {
                    update_search(
                        &mut commands,
                        search.0,
                        &base_pile,
                        &base_transform,
                        text.as_ref().unwrap().get(),
                        &side,
                        &mut menu,
                    );
                }
            }
            Packet::Scale(id, new) => {
                if id.user == client.my_id() {
                    if let Some(mut t) =
                        queryme.iter_mut().find_map(
                            |(a, _, _, _, _, _, t)| {
                                if *a == id.id { Some(t) } else { None }
                            },
                        )
                    {
                        t.scale = Vec3::splat(new);
                    }
                } else if let Some(mut t) = query.iter_mut().find_map(
                    |(a, t, _, _, _, _, _, _, _, _)| {
                        if *a == id { Some(t) } else { None }
                    },
                ) {
                    t.scale = Vec3::splat(new);
                }
            }
            Packet::Draw(id, to, start) => {
                let user = sender;
                let run = |pile: &mut Pile,
                           children: _,
                           mut transform: Mut<Transform>,
                           entity: _,
                           resend: bool| {
                    let len = to.len();
                    if start > pile.len() || len > start {
                        if resend {
                            commands.entity(entity).despawn();
                            if sent.add(id) {
                                client
                                    .send(
                                        id.user,
                                        &Packet::Request(id.id),
                                        Reliability::Reliable,
                                        COMPRESSION,
                                    )
                                    .unwrap();
                            }
                            for (cid, _, _) in to {
                                let syncobject = SyncObject { user, id: cid };
                                if sent.add(syncobject) {
                                    client
                                        .send(
                                            id.user,
                                            &Packet::Request(cid),
                                            Reliability::Reliable,
                                            COMPRESSION,
                                        )
                                        .unwrap();
                                }
                            }
                        }
                        return;
                    }
                    let mut fail = false;
                    for ((cid, trans, uuid), card) in
                        to.into_iter().zip(pile.drain(start - len..start))
                    {
                        let syncobject = SyncObject { user, id: cid };
                        if resend && card.id != uuid {
                            fail = true;
                            if sent.add(syncobject) {
                                client
                                    .send(
                                        id.user,
                                        &Packet::Request(cid),
                                        Reliability::Reliable,
                                        COMPRESSION,
                                    )
                                    .unwrap();
                            }
                        } else {
                            new_pile_at(
                                Pile::Single(card.into()),
                                card_base.clone(),
                                &mut materials,
                                &mut commands,
                                &mut meshes,
                                trans.with_scale(transform.scale.x),
                                false,
                                None,
                                Some(syncobject),
                                None,
                            );
                            ignore.insert(syncobject);
                        }
                    }
                    if fail {
                        commands.entity(entity).despawn();
                        if sent.add(id) {
                            client
                                .send(
                                    id.user,
                                    &Packet::Request(id.id),
                                    Reliability::Reliable,
                                    COMPRESSION,
                                )
                                .unwrap();
                        }
                        return;
                    }
                    pile.set_single();
                    if !pile.is_empty() {
                        let card = pile.last();
                        repaint_face(&mut mats, &mut materials, card, children);
                        adjust_meshes(
                            pile,
                            children,
                            &mut meshes,
                            &mut query_meshes,
                            &mut transform,
                            &mut colliders.get_mut(entity).unwrap(),
                            &equipment,
                            None,
                            &mut commands,
                        );
                    }
                    if let Some(search) = &search
                        && search.1.0 == entity
                    {
                        update_search(
                            &mut commands,
                            search.0,
                            pile,
                            &transform,
                            text.as_ref().unwrap().get(),
                            &side,
                            &mut menu,
                        );
                    }
                };
                if id.user == client.my_id()
                    && let Some((mut pile, children, transform, entity)) =
                        queryme.iter_mut().find_map(|(a, _, p, e, c, _, t)| {
                            if *a == id.id {
                                Some((p, c, t, e))
                            } else {
                                None
                            }
                        })
                    && let Some(pile) = &mut pile
                    && let Some(children) = children
                {
                    run(pile, children, transform, entity, false);
                } else if let Some((mut pile, children, transform, entity)) =
                    query.iter_mut().find_map(
                        |(a, t, _, _, e, _, c, d, _, _)| {
                            if *a == id { Some((d, c, t, e)) } else { None }
                        },
                    )
                    && let Some(pile) = &mut pile
                    && let Some(children) = children
                {
                    run(pile, children, transform, entity, true);
                }
            }
            #[allow(unused_variables)]
            Packet::Repaint(id, to, tokens, flipped) => {
                todo!()
            }
            Packet::Counter(id, to) => {
                let mut run = |children: &Children, e: Entity| {
                    let c = children[0];
                    if let (Ok(counter), Ok(mut t)) = (shape.get_mut(e), text3d.get_mut(c)) {
                        let Shape::Counter(v, _) = counter.into_inner() else {
                            unreachable!()
                        };
                        *v = to.clone();
                        *t.get_single_mut().unwrap() = v.0.to_string()
                    }
                };
                if id.user == client.my_id() {
                    if let Some((Some(children), e)) = queryme.iter_mut().find_map(
                        |(a, _, _, e, c, _, _)| {
                            if *a == id.id { Some((c, e)) } else { None }
                        },
                    ) {
                        run(children, e);
                    }
                } else if let Some((Some(children), e)) = query.iter_mut().find_map(
                    |(a, _, _, _, e, _, c, _, _, _)| {
                        if *a == id { Some((c, e)) } else { None }
                    },
                ) {
                    run(children, e);
                }
            }
            Packet::Indicator(cam, cur, ping) => {
                if ind {
                    return;
                }
                if let Some(id) = peers.map().get(&sender) {
                    if let Some(mut t) = cams
                        .iter_mut()
                        .find_map(|(a, t)| if a.0 == sender { Some(t) } else { None })
                    {
                        t.translation = cam.into()
                    } else {
                        ind = true;
                        make_cam(
                            &mut commands,
                            sender,
                            *id,
                            cam.into(),
                            &mut materials,
                            &mut meshes,
                        );
                    }
                    if let Some((mut b, mut t, e)) = curs.iter_mut().find(|(a, _, _)| a.0 == sender)
                    {
                        if ping != b.1
                            && let Ok(mut mat) = mats.get_mut(e)
                        {
                            b.1 = ping;
                            if ping {
                                fn sub(a: Color, b: Color) -> Color {
                                    let mut a = a.to_linear();
                                    let b = b.to_linear();
                                    a.red -= b.red;
                                    a.green -= b.green;
                                    a.blue -= b.blue;
                                    a.into()
                                }
                                mat.0 = materials.add(StandardMaterial {
                                    alpha_mode: AlphaMode::Opaque,
                                    unlit: true,
                                    base_color: sub(Color::WHITE, PLAYER[id % PLAYER.len()]),
                                    ..default()
                                });
                            } else {
                                mat.0 = materials.add(StandardMaterial {
                                    alpha_mode: AlphaMode::Opaque,
                                    unlit: true,
                                    base_color: PLAYER[id % PLAYER.len()],
                                    ..default()
                                });
                            }
                        }
                        if let Some(v) = cur {
                            let v: Vec3 = v.into();
                            if ping {
                                if let Some((_, orig, mut mesh, mut transform, _)) =
                                    drag.iter_mut().find(|p| *p.4 == sender)
                                {
                                    let dir = (v - orig.0).normalize();
                                    let d = (v - orig.0).length();
                                    let m = (v + orig.0) / 2.0;
                                    transform.translation = m;
                                    mesh.0 = meshes.add(Cylinder::new(CARD_THICKNESS * 8.0, d));
                                    transform.rotation = Quat::from_rotation_arc(Vec3::Y, dir);
                                } else {
                                    commands.spawn((
                                        PingDrag(v),
                                        Mesh3d(
                                            meshes.add(Cylinder::new(CARD_THICKNESS * 8.0, 0.0)),
                                        ),
                                        MeshMaterial3d(materials.add(StandardMaterial {
                                            alpha_mode: AlphaMode::Opaque,
                                            unlit: true,
                                            base_color: PLAYER[id % PLAYER.len()],
                                            ..default()
                                        })),
                                        Transform::from_xyz(v.x, v.y, v.z),
                                        sender,
                                    ));
                                }
                            } else if let Some(e) = drag.iter_mut().find(|p| *p.4 == sender) {
                                commands.entity(e.0).despawn();
                            }
                            t.scale = Vec3::splat(1.0);
                            t.translation = v;
                        } else {
                            if let Some(e) = drag.iter_mut().find(|p| *p.4 == sender) {
                                commands.entity(e.0).despawn();
                            }
                            t.scale = Vec3::splat(0.0);
                        }
                    } else {
                        make_cur(
                            &mut commands,
                            sender,
                            *id,
                            cam.into(),
                            &mut materials,
                            &mut meshes,
                        );
                    }
                }
            }
            Packet::Text(msg) => {
                if let Some(name) = peers.names.get(&sender) {
                    spawn_msg(*chat, name.clone(), msg, &mut commands);
                }
                //TODO deal with no name
            }
            Packet::Voice(data) => {
                #[cfg(feature = "mic")]
                audio.decode(data, |data| {
                    let source = SamplesBuffer::new(
                        1,
                        (audio_settings.sample_rate.get_number() * 1000) as u32,
                        data,
                    );
                    sink.append(source);
                    sink.play()
                });
            }
            Packet::Turn(player) => turn.0 = player,
        }
    });
    #[cfg(feature = "steam")]
    client.flush();
}
#[derive(Component, Deref, DerefMut)]
pub struct CameraInd(pub PeerId);
#[derive(Component)]
pub struct CursorInd(pub PeerId, pub bool);
pub fn spawn_hand(me: usize, commands: &mut Commands) {
    let transform = match me {
        0 => Transform::from_xyz(MAT_WIDTH / 2.0, CARD_HEIGHT / 2.0, MAT_HEIGHT + CARD_HEIGHT),
        1 => Transform::from_xyz(
            MAT_WIDTH / 2.0,
            CARD_HEIGHT / 2.0,
            -MAT_HEIGHT - CARD_HEIGHT,
        ),
        2 => Transform::from_xyz(
            -MAT_WIDTH / 2.0,
            CARD_HEIGHT / 2.0,
            MAT_HEIGHT + CARD_HEIGHT,
        ),
        3 => Transform::from_xyz(
            -MAT_WIDTH / 2.0,
            CARD_HEIGHT / 2.0,
            -MAT_HEIGHT - CARD_HEIGHT,
        ),
        _ => Transform::default(),
    };
    commands.spawn((transform, Hand::default()));
}
pub fn on_join(
    client: ClientTypeRef,
    peer: PeerId,
    peers: &Arc<Mutex<HashMap<PeerId, usize>>>,
    flip: &Arc<Mutex<Vec<(usize, bool)>>>,
    send: &Arc<AtomicBool>,
    who: &Arc<Mutex<HashMap<usize, PeerId>>>,
) {
    info!("user {peer} has joined");
    if client.is_host() {
        let mut k = 1;
        {
            let mut who = who.lock().unwrap();
            loop {
                if let Vacant(e) = who.entry(k) {
                    e.insert(peer);
                    break;
                }
                k += 1;
            }
        }
        client
            .broadcast(
                &Packet::SetUser(peer, k),
                Reliability::Reliable,
                COMPRESSION,
            )
            .unwrap();
        for (k, v) in peers.lock().unwrap().iter() {
            client
                .send(
                    peer,
                    &Packet::SetUser(*k, *v),
                    Reliability::Reliable,
                    COMPRESSION,
                )
                .unwrap();
        }
        peers.lock().unwrap().insert(peer, k);
        flip.lock().unwrap().push((k, true));
    }
    send.store(true, std::sync::atomic::Ordering::Relaxed);
}
pub fn on_leave(
    client: ClientTypeRef,
    peer: PeerId,
    peers: &Arc<Mutex<HashMap<PeerId, usize>>>,
    flip: &Arc<Mutex<Vec<(usize, bool)>>>,
    who: &Arc<Mutex<HashMap<usize, PeerId>>>,
    rempeers: &Arc<Mutex<Vec<PeerId>>>,
    give: &Arc<Mutex<Vec<PeerId>>>,
) {
    info!("user {peer} has left");
    let k = peers.lock().unwrap().remove(&peer);
    rempeers.lock().unwrap().push(peer);
    if client.is_host() {
        if let Some(k) = k {
            flip.lock().unwrap().push((k, false));
        }
        give.lock().unwrap().push(peer);
        let mut who = who.lock().unwrap();
        who.retain(|_, p| *p != peer)
    }
}
#[cfg(any(feature = "steam", feature = "ip"))]
pub fn new_lobby(
    keybinds: Keybinds,
    mut client: ResMut<Client>,
    down: Res<Download>,
    #[cfg(feature = "ip")] send_sleep: Res<SendSleeping>,
    #[cfg(feature = "ip")] flip_counter: Res<FlipCounter>,
    #[cfg(feature = "ip")] give: Res<GiveEnts>,
    mut peers: ResMut<Peers>,
    #[cfg(feature = "ip")] rempeers: Res<RemPeers>,
    shapes: Query<Entity, (With<Shape>, With<SyncObjectMe>)>,
    mut commands: Commands,
) {
    #[cfg(feature = "steam")]
    if keybinds.just_pressed(Keybind::HostSteam) {
        info!("hosting steam");
        peers.me = Some(0);
        peers.name = client.get_name();
        peers.map().insert(client.my_id(), 0);
        client.host_steam().unwrap();
    }
    #[cfg(feature = "ip")]
    if keybinds.just_pressed(Keybind::HostIp) {
        info!("hosting ip");
        peers.name = client.get_name();
        let flip = flip_counter.0.clone();
        let flip2 = flip.clone();
        let send = send_sleep.0.clone();
        let give = give.0.clone();
        let rempeers = rempeers.0.clone();
        let peers1 = peers.map.clone();
        let peers2 = peers1.clone();
        let who = Arc::new(Mutex::new(HashMap::new()));
        let who2 = who.clone();
        client
            .host_ip_runtime(
                Some(Box::new(move |client, peer| {
                    on_join(client, peer, &peers1, &flip, &send, &who);
                })),
                Some(Box::new(move |client, peer| {
                    on_leave(client, peer, &peers2, &flip2, &who2, &rempeers, &give);
                })),
                &down.runtime.0,
            )
            .unwrap();
        peers.me = Some(0);
        peers.map().insert(client.my_id(), 0);
    }
    #[cfg(feature = "ip")]
    if keybinds.just_pressed(Keybind::JoinIp) {
        info!("joining ip");
        for e in shapes {
            commands.entity(e).despawn()
        }
        peers.name = client.get_name();
        let flip = flip_counter.0.clone();
        let flip2 = flip.clone();
        let send = send_sleep.0.clone();
        let give = give.0.clone();
        let rempeers = rempeers.0.clone();
        let peers = peers.map.clone();
        let peers2 = peers.clone();
        let who = Arc::new(Mutex::new(HashMap::new()));
        let who2 = who.clone();
        client
            .join_ip_runtime(
                "127.0.0.1".parse().unwrap(),
                Some(Box::new(move |client, peer| {
                    on_join(client, peer, &peers, &flip, &send, &who);
                })),
                Some(Box::new(move |client, peer| {
                    on_leave(client, peer, &peers2, &flip2, &who2, &rempeers, &give);
                })),
                &down.runtime.0,
            )
            .unwrap();
    }
}
#[derive(Resource, Default, Deref, DerefMut)]
pub struct Sent(pub HashMap<SyncObject, bool>);
impl Sent {
    pub fn add(&mut self, key: SyncObject) -> bool {
        if let Some(b) = self.0.insert(key, true) {
            if b {
                false
            } else {
                self.0.remove(&key);
                true
            }
        } else {
            true
        }
    }
    pub fn del(&mut self, key: SyncObject) -> bool {
        if self.0.remove(&key).is_some() {
            true
        } else {
            self.0.insert(key, false);
            false
        }
    }
    pub fn rem(&mut self, key: &SyncObject) {
        self.0.remove(key);
    }
    pub fn has(&self, key: &SyncObject) -> bool {
        self.0.get(key) == Some(&true)
    }
}
#[derive(Encode, Decode, Debug)]
pub enum Packet {
    Pos(Vec<(SyncObjectMe, Trans, Phys, bool, bool)>),
    Scale(SyncObject, f32),
    Request(SyncObjectMe),
    Received(SyncObjectMe),
    Dead(SyncObject),
    Take(SyncObject, SyncObjectMe),
    New(SyncObjectMe, Pile, Trans, f32),
    NewShape(SyncObjectMe, Shape, Trans, f32),
    Flip(SyncObject, usize, bool),
    Equip(SyncObject),
    Counter(SyncObject, Value),
    Reorder(SyncObject, Vec<Id>),
    Draw(SyncObject, Vec<(SyncObjectMe, Trans, Id)>, usize),
    Merge(SyncObject, SyncObject, usize),
    Move(SyncObject, SyncObject, usize, bool, bool),
    SetUser(PeerId, usize),
    Indicator(Pos, Option<Pos>, bool),
    Repaint(SyncObjectMe, Id, Vec<Id>, bool),
    Name(String),
    Text(String),
    Modify(SyncObject, Counter, Option<Value>),
    Voice(Vec<u8>),
    Turn(usize),
}
#[derive(Encode, Decode, Debug)]
pub struct Phys {
    pub vel: (u32, u32, u32),
    pub ang: (u32, u32, u32),
}
impl Phys {
    fn from(pos: &LinearVelocity, ang: &AngularVelocity) -> Self {
        Self {
            vel: unsafe { mem::transmute::<LinearVelocity, (u32, u32, u32)>(*pos) },
            ang: unsafe { mem::transmute::<AngularVelocity, (u32, u32, u32)>(*ang) },
        }
    }
    fn to(self) -> (LinearVelocity, AngularVelocity) {
        unsafe { mem::transmute::<Self, (LinearVelocity, AngularVelocity)>(self) }
    }
}
#[derive(Encode, Decode, Debug, Copy, Clone)]
pub struct Pos(u32, u32, u32);
impl From<Pos> for Vec3 {
    fn from(value: Pos) -> Self {
        unsafe { mem::transmute::<Pos, Vec3>(value) }
    }
}
impl From<Vec3> for Pos {
    fn from(value: Vec3) -> Self {
        unsafe { mem::transmute::<Vec3, Pos>(value) }
    }
}
#[derive(Encode, Decode, Debug, Copy, Clone)]
pub struct Rot(u128);
#[derive(Encode, Decode, Debug, Copy, Clone)]
pub struct Trans {
    pub translation: Pos,
    pub rotation: Rot,
}
impl Trans {
    pub fn from(value: &GlobalTransform) -> Self {
        Self {
            translation: value.translation().into(),
            rotation: value.rotation().into(),
        }
    }
    pub fn from_transform(value: &Transform) -> Self {
        Self {
            translation: value.translation.into(),
            rotation: value.rotation.into(),
        }
    }
    pub fn with_scale(self, scale: f32) -> Transform {
        Transform {
            translation: self.translation.into(),
            rotation: self.rotation.into(),
            scale: Vec3::splat(scale),
        }
    }
}
impl From<Rot> for Quat {
    fn from(value: Rot) -> Self {
        unsafe { mem::transmute::<Rot, Quat>(value) }
    }
}
impl From<Quat> for Rot {
    fn from(value: Quat) -> Self {
        unsafe { mem::transmute::<Quat, Rot>(value) }
    }
}
#[derive(Component, Debug, Encode, Decode, Eq, PartialEq, Hash, Copy, Clone)]
pub struct SyncObject {
    pub user: PeerId,
    pub id: SyncObjectMe,
}
#[derive(Component, Default, Debug, Encode, Decode, Eq, PartialEq, Copy, Clone, Hash)]
pub struct SyncObjectMe(pub u64);
impl SyncObjectMe {
    pub fn new(rand: &mut Single<&mut WyRand, With<GlobalRng>>, count: &mut SyncCount) -> Self {
        count.add(1);
        Self(rand.next_u64())
    }
}
#[derive(Resource, Default)]
pub struct SyncCount {
    vec: Vec<(SyncObjectMe, Trans, Phys, bool, bool)>,
    count: usize,
}
impl SyncCount {
    pub fn add(&mut self, n: usize) {
        self.count += n;
        self.vec.reserve(self.count);
    }
    pub fn rem(&mut self, n: usize) {
        self.count -= n;
    }
    pub fn take(&mut self) -> Vec<(SyncObjectMe, Trans, Phys, bool, bool)> {
        mem::take(&mut self.vec)
    }
    pub fn give(&mut self, vec: Vec<(SyncObjectMe, Trans, Phys, bool, bool)>) {
        self.vec = vec;
    }
}
#[derive(Component)]
pub struct InOtherHand;
#[derive(SystemParam)]
pub struct Net<'w, 's> {
    pub client: ResMut<'w, Client>,
    pub rand: Single<'w, 's, &'static mut WyRand, With<GlobalRng>>,
    pub count: ResMut<'w, SyncCount>,
    pub sent: ResMut<'w, Sent>,
    commands: Commands<'w, 's>,
}
impl<'w, 's> Net<'w, 's> {
    pub fn new_id(&mut self) -> SyncObjectMe {
        SyncObjectMe::new(&mut self.rand, &mut self.count)
    }
    pub fn received(&self, user: PeerId, id: SyncObjectMe) {
        self.client
            .send(
                user,
                &Packet::Received(id),
                Reliability::Reliable,
                COMPRESSION,
            )
            .unwrap();
    }
    pub fn scale(&self, id: SyncObject, scale: f32) {
        self.client
            .broadcast(
                &Packet::Scale(id, scale),
                Reliability::Reliable,
                COMPRESSION,
            )
            .unwrap();
    }
    pub fn scale_me(&self, id: SyncObjectMe, scale: f32) {
        self.scale(self.to_global(id), scale)
    }
    pub fn take(&mut self, entity: Entity, id: SyncObject) {
        self.sent.add(id);
        let nid = self.new_id();
        self.client
            .broadcast(&Packet::Take(id, nid), Reliability::Reliable, COMPRESSION)
            .unwrap();
        self.commands
            .entity(entity)
            .remove::<SyncObject>()
            .insert(nid);
    }
    pub fn to_global(&self, id: SyncObjectMe) -> SyncObject {
        SyncObject {
            user: self.client.my_id(),
            id,
        }
    }
    pub fn text(&self, msg: String) {
        self.client
            .broadcast(&Packet::Text(msg), Reliability::Reliable, COMPRESSION)
            .unwrap();
    }
    pub fn voice(&self, msg: Vec<u8>) {
        self.client
            .broadcast(&Packet::Voice(msg), Reliability::Reliable, COMPRESSION)
            .unwrap();
    }
    pub fn killed_me(&mut self, id: SyncObjectMe) {
        self.count.rem(1);
        self.killed(self.to_global(id))
    }
    pub fn killed(&self, id: SyncObject) {
        self.client
            .broadcast(&Packet::Dead(id), Reliability::Reliable, COMPRESSION)
            .unwrap();
    }
    pub fn merge_them(&self, base: SyncObject, top: SyncObjectMe, at: usize) {
        self.client
            .broadcast(
                &Packet::Merge(base, self.to_global(top), at),
                Reliability::Reliable,
                COMPRESSION,
            )
            .unwrap();
    }
    pub fn merge(&self, base: SyncObjectMe, top: SyncObject, at: usize) {
        self.client
            .broadcast(
                &Packet::Merge(self.to_global(base), top, at),
                Reliability::Reliable,
                COMPRESSION,
            )
            .unwrap();
    }
    pub fn move_to(
        &self,
        from: SyncObject,
        to: SyncObject,
        count: usize,
        from_top: bool,
        to_top: bool,
    ) {
        self.client
            .broadcast(
                &Packet::Move(from, to, count, from_top, to_top),
                Reliability::Reliable,
                COMPRESSION,
            )
            .unwrap();
    }
    pub fn merge_me(&self, base: SyncObjectMe, top: SyncObjectMe, at: usize) {
        self.merge(base, self.to_global(top), at);
    }
    pub fn reorder(&self, id: SyncObject, order: Vec<Id>) {
        self.client
            .broadcast(
                &Packet::Reorder(id, order),
                Reliability::Reliable,
                COMPRESSION,
            )
            .unwrap();
    }
    pub fn reorder_me(&self, id: SyncObjectMe, order: Vec<Id>) {
        self.reorder(self.to_global(id), order);
    }
    pub fn equip(&self, id: SyncObject) {
        self.client
            .broadcast(&Packet::Equip(id), Reliability::Reliable, COMPRESSION)
            .unwrap();
    }
    pub fn equip_me(&self, id: SyncObjectMe) {
        self.equip(self.to_global(id))
    }
    pub fn draw(&self, id: SyncObject, to: Vec<(SyncObjectMe, Trans, Id)>, start: usize) {
        self.client
            .broadcast(
                &Packet::Draw(id, to, start),
                Reliability::Reliable,
                COMPRESSION,
            )
            .unwrap();
    }
    pub fn draw_me(&self, id: SyncObjectMe, to: Vec<(SyncObjectMe, Trans, Id)>, start: usize) {
        self.draw(self.to_global(id), to, start)
    }
    pub fn flip(&self, id: SyncObject, at: usize, rev: bool) {
        self.client
            .broadcast(
                &Packet::Flip(id, at, rev),
                Reliability::Reliable,
                COMPRESSION,
            )
            .unwrap();
    }
    pub fn flip_me(&self, id: SyncObjectMe, at: usize, rev: bool) {
        self.flip(self.to_global(id), at, rev)
    }
    pub fn counter(&self, id: SyncObject, value: Value) {
        self.client
            .broadcast(
                &Packet::Counter(id, value),
                Reliability::Reliable,
                COMPRESSION,
            )
            .unwrap();
    }
    pub fn counter_me(&self, id: SyncObjectMe, value: Value) {
        self.counter(self.to_global(id), value)
    }
    pub fn modify(&self, id: SyncObject, counter: Counter, value: Option<Value>) {
        self.client
            .broadcast(
                &Packet::Modify(id, counter, value),
                Reliability::Reliable,
                COMPRESSION,
            )
            .unwrap();
    }
    pub fn modify_me(&self, id: SyncObjectMe, counter: Counter, value: Option<Value>) {
        self.modify(self.to_global(id), counter, value)
    }
    pub fn turn(&self, n: usize) {
        self.client
            .broadcast(&Packet::Turn(n), Reliability::Reliable, COMPRESSION)
            .unwrap();
    }
}
