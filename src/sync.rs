use crate::download::add_images;
use crate::misc::{adjust_meshes, get_mut_card, new_pile_at, repaint_face};
#[cfg(feature = "steam")]
use crate::setup::SteamInfo;
use crate::setup::{MAT_HEIGHT, MAT_WIDTH};
use crate::shapes::Shape;
use crate::*;
use bevy::diagnostic::FrameCount;
use bevy_rand::global::GlobalRng;
use bevy_tangled::{ClientTrait, Compression, Reliability};
use bitcode::{Decode, Encode};
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use std::mem;
use std::sync::atomic::AtomicBool;
#[derive(Resource, Default)]
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
    mut sync_actions: ResMut<SyncActions>,
    mut sent: ResMut<Sent>,
    query_take: Query<(Entity, &SyncObject)>,
    mut commands: Commands,
    client: Res<Client>,
    send_sleep: Res<SendSleeping>,
    frame: Res<FrameCount>,
) {
    let send_sleep = send_sleep
        .0
        .swap(false, std::sync::atomic::Ordering::Relaxed);
    let mut v = 0;
    let mut vec = count.take();
    for (id, transform, vel, ang, in_hand, is_sleep, follow) in query {
        if send_sleep || (is_sleep.is_none() && frame.0.is_multiple_of(8)) || follow.is_some() {
            vec.push((
                *id,
                Trans::from(transform),
                Phys::from(vel, ang),
                in_hand.is_some(),
            ))
        }
    }
    for dead in sync_actions.killed.drain(..) {
        client
            .broadcast(
                &Packet::Dead(dead),
                Reliability::Reliable,
                Compression::Compressed,
            )
            .unwrap();
    }
    for flip in sync_actions.flip.drain(..) {
        client
            .broadcast(
                &Packet::Flip(flip),
                Reliability::Reliable,
                Compression::Compressed,
            )
            .unwrap();
    }
    for (id, order) in sync_actions.reorder.drain(..) {
        client
            .broadcast(
                &Packet::Reorder(id, order),
                Reliability::Reliable,
                Compression::Compressed,
            )
            .unwrap();
    }
    for (id, to, start) in sync_actions.draw.drain(..) {
        client
            .broadcast(
                &Packet::Draw(id, to, start),
                Reliability::Reliable,
                Compression::Compressed,
            )
            .unwrap();
    }
    for (from, to) in sync_actions.take_owner.drain(..) {
        if let Some((entity, _)) = query_take.iter().find(|(_, b)| **b == from) {
            commands.entity(entity).remove::<SyncObject>().insert(to);
            v += 1;
        }
        sent.add(from);
        client
            .broadcast(
                &Packet::Take(from, to),
                Reliability::Reliable,
                Compression::Compressed,
            )
            .unwrap();
    }
    let packet = Packet::Pos(vec);
    client
        .broadcast(&packet, Reliability::Reliable, Compression::Compressed)
        .unwrap();
    let Packet::Pos(mut vec) = packet else {
        unreachable!()
    };
    vec.clear();
    count.give(vec);
    count.add(v);
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
    let Some(info) = client.info() else { return };
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
            Option<&mut Position>,
            Option<&mut Rotation>,
            &mut LinearVelocity,
            &mut AngularVelocity,
            Entity,
            Option<&InOtherHand>,
            Option<&Children>,
            Option<&mut Pile>,
            &mut GravityScale,
        ),
        With<Children>,
    >,
    mut queryme: Query<
        (
            &SyncObjectMe,
            &GlobalTransform,
            Option<&Pile>,
            Option<&Shape>,
            Entity,
        ),
        Without<SyncObject>,
    >,
    mut sent: ResMut<Sent>,
    asset_server: Res<AssetServer>,
    down: Res<Download>,
    mut commands: Commands,
    hand: Single<Entity, (With<Owned>, With<Hand>)>,
    mut mats: Query<&mut MeshMaterial3d<StandardMaterial>, Without<ZoomHold>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    card_base: Res<CardBase>,
    mut count: ResMut<SyncCount>,
    mut client: ResMut<Client>,
    mut colliders: Query<&mut Collider>,
    mut query_meshes: Query<(&mut Mesh3d, &mut Transform), Without<Children>>,
) {
    let mut ignore = HashSet::new();
    client.recv(Compression::Compressed, |client, packet| {
        let sender = packet.src;
        let data = packet.data;
        match data {
            Packet::Pos(data) => {
                let user = sender.raw();
                for (lid, trans, phys, in_hand) in data {
                    let id = SyncObject { user, id: lid.0 };
                    if sent.has(&id) || ignore.contains(&id) {
                        continue;
                    }
                    if let Some((
                        mut t,
                        p,
                        r,
                        hand,
                        children,
                        entity,
                        pile,
                        mut gravity,
                        mut lv,
                        mut av,
                    )) = query
                        .iter_mut()
                        .find_map(|(a, b, z, r, lv, av, e, h, c, p, g)| {
                            if *a == id {
                                Some((b, z, r, h, c, e, p, g, lv, av))
                            } else {
                                None
                            }
                        })
                    {
                        let (lvt, avt) = phys.to();
                        *lv = lvt;
                        *av = avt;
                        *t = trans.into();
                        if let Some(mut p) = p {
                            *p = t.translation.into();
                        }
                        if let Some(mut r) = r {
                            *r = t.rotation.into();
                        }
                        if let Some(pile) = pile
                            && in_hand != hand.is_some()
                            && let Some(children) = children
                        {
                            if in_hand {
                                commands.entity(entity).insert(InOtherHand);
                                mats.get_mut(*children.first().unwrap()).unwrap().0 =
                                    mats.get_mut(*children.get(1).unwrap()).unwrap().0.clone();
                                gravity.0 = 0.0
                            } else {
                                commands.entity(entity).remove::<InOtherHand>();
                                repaint_face(&mut mats, &mut materials, &pile.0[0], children);
                                gravity.0 = GRAVITY;
                            }
                        }
                    } else if sent.add(id) {
                        client
                            .send(
                                sender,
                                &Packet::Request(lid),
                                Reliability::Reliable,
                                Compression::Compressed,
                            )
                            .unwrap();
                    }
                }
            }
            Packet::Take(from, to) => {
                let new = SyncObject {
                    user: sender.raw(),
                    id: to.0,
                };
                ignore.insert(new);
                if from.user == client.my_id().raw() {
                    client
                        .broadcast(
                            &Packet::Received(SyncObjectMe(from.id)),
                            Reliability::Reliable,
                            Compression::Compressed,
                        )
                        .unwrap();
                    if let Some((_, _, _, _, e)) =
                        queryme.iter().find(|(id, _, _, _, _)| id.0 == from.id)
                    {
                        count.rem(1);
                        commands
                            .entity(e)
                            .remove::<SyncObjectMe>()
                            .remove::<FollowMouse>()
                            .insert(new);
                    }
                } else if let Some((mut id, _, _, _, _, _, _, _, _, _, _)) = query
                    .iter_mut()
                    .find(|(id, _, _, _, _, _, _, _, _, _, _)| *id.as_ref() == from)
                {
                    sent.add(*id);
                    *id = new
                }
            }
            Packet::Request(lid) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                if sent.add(id) {
                    if let Some((b, c, s)) = queryme.iter_mut().find_map(|(a, b, c, s, _)| {
                        if a.0 == lid.0 { Some((b, c, s)) } else { None }
                    }) {
                        if let Some(c) = c {
                            client
                                .send(
                                    sender,
                                    &Packet::New(lid, c.clone_no_image(), Trans::from(b)),
                                    Reliability::Reliable,
                                    Compression::Compressed,
                                )
                                .unwrap();
                        } else if let Some(s) = s {
                            client
                                .send(
                                    sender,
                                    &Packet::NewShape(lid, *s, Trans::from(b)),
                                    Reliability::Reliable,
                                    Compression::Compressed,
                                )
                                .unwrap();
                        }
                    } else {
                        sent.rem(&id);
                    }
                }
            }
            Packet::Received(lid) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                sent.del(id);
            }
            Packet::New(lid, pile, trans) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                let deck = down.get_deck.clone();
                let client = down.client.0.clone();
                let asset_server = asset_server.clone();
                let f = async move {
                    add_images(pile, trans.into(), id, deck, client, asset_server).await;
                };
                #[cfg(feature = "wasm")]
                wasm_bindgen_futures::spawn_local(f);
                #[cfg(not(feature = "wasm"))]
                down.runtime.0.spawn(f);
            }
            Packet::NewShape(lid, shape, trans) => {
                client
                    .send(
                        sender,
                        &Packet::Received(lid),
                        Reliability::Reliable,
                        Compression::Compressed,
                    )
                    .unwrap();
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                sent.del(id);
                ignore.insert(id);
                shape
                    .create(trans.into(), &mut commands, &mut materials, &mut meshes)
                    .insert(id);
            }
            Packet::SetUser(id) => {
                info!("joined as number {} user", id);
                commands.entity(*hand).despawn();
                spawn_hand(id, &mut commands);
            }
            Packet::Dead(lid) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                if let Some(e) = query.iter_mut().find_map(
                    |(a, _, _, _, _, _, b, _, _, _, _)| if *a == id { Some(b) } else { None },
                ) {
                    commands.entity(e).despawn();
                }
            }
            Packet::Flip(lid) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                if let Some((transform, children, mut pile)) = query.iter_mut().find_map(
                    |(a, b, _, _, _, _, _, _, c, d, _)| {
                        if *a == id { Some((b, c, d)) } else { None }
                    },
                ) && let Some(pile) = &mut pile
                {
                    let card = get_mut_card(pile, &transform);
                    if let Some(alt) = &mut card.alt
                        && let Some(children) = children
                    {
                        mem::swap(&mut card.normal, alt);
                        repaint_face(&mut mats, &mut materials, card, children);
                        card.is_alt = !card.is_alt;
                    }
                }
            }
            Packet::Reorder(lid, order) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                if let Some((mut pile, children)) = query.iter_mut().find_map(
                    |(a, _, _, _, _, _, _, _, c, d, _)| {
                        if *a == id { Some((d, c)) } else { None }
                    },
                ) && let Some(pile) = &mut pile
                    && let Some(children) = children
                {
                    for (i, id) in order.into_iter().enumerate() {
                        let n = pile.0[i..].iter().position(|c| c.id == id).unwrap() + i;
                        pile.0.swap(i, n);
                    }
                    let card = pile.0.last().unwrap();
                    repaint_face(&mut mats, &mut materials, card, children);
                }
            }
            Packet::Draw(lid, to, start) => {
                let user = sender.raw();
                let id = SyncObject { user, id: lid.0 };
                if let Some((mut pile, children, mut transform, entity)) =
                    query.iter_mut().find_map(
                        |(a, t, _, _, _, _, e, _, c, d, _)| {
                            if *a == id { Some((d, c, t, e)) } else { None }
                        },
                    )
                    && let Some(pile) = &mut pile
                    && let Some(children) = children
                {
                    let len = to.len();
                    for ((id, trans), card) in to.into_iter().zip(pile.0.drain(start - len..start))
                    {
                        let syncobject = SyncObject { user, id: id.0 };
                        new_pile_at(
                            Pile(vec![card]),
                            card_base.stock.clone(),
                            &mut materials,
                            &mut commands,
                            &mut meshes,
                            card_base.back.clone(),
                            card_base.side.clone(),
                            trans.into(),
                            false,
                            None,
                            Some(syncobject),
                            None,
                        );
                        ignore.insert(syncobject);
                    }
                    let card = pile.0.last().unwrap();
                    repaint_face(&mut mats, &mut materials, card, children);
                    adjust_meshes(
                        pile,
                        children,
                        &mut meshes,
                        &mut query_meshes,
                        &mut transform,
                        &mut colliders.get_mut(entity).unwrap(),
                    );
                }
            }
        }
    });
    #[cfg(feature = "steam")]
    client.flush();
}
pub fn spawn_hand(me: usize, commands: &mut Commands) {
    let mut transform = match me {
        0 => Transform::from_xyz(MAT_WIDTH / 2.0, 64.0, MAT_HEIGHT + CARD_HEIGHT / 2.0),
        1 => Transform::from_xyz(MAT_WIDTH / 2.0, 64.0, -MAT_HEIGHT - CARD_HEIGHT / 2.0),
        2 => Transform::from_xyz(-MAT_WIDTH / 2.0, 64.0, MAT_HEIGHT + CARD_HEIGHT / 2.0),
        3 => Transform::from_xyz(-MAT_WIDTH / 2.0, 64.0, -MAT_HEIGHT - CARD_HEIGHT / 2.0),
        _ => Transform::from_xyz(0.0, 64.0, 0.0),
    };
    if me == 1 || me == 3 {
        transform.rotate_y(PI);
    }
    commands.spawn((transform, Hand::default(), Owned));
}
#[cfg(all(feature = "steam", feature = "ip"))]
pub fn new_lobby(
    input: Res<ButtonInput<KeyCode>>,
    mut client: ResMut<Client>,
    down: Res<Download>,
    #[cfg(feature = "ip")] send_sleep: Res<SendSleeping>,
) {
    if input.all_pressed([KeyCode::ShiftLeft, KeyCode::AltLeft, KeyCode::ControlLeft]) {
        if input.just_pressed(KeyCode::KeyN) {
            info!("hosting steam");
            #[cfg(feature = "steam")]
            client.host_steam().unwrap();
        } else if input.just_pressed(KeyCode::KeyM) {
            info!("hosting ip");
            #[cfg(feature = "ip")]
            let send = send_sleep.0.clone();
            #[cfg(feature = "ip")]
            client
                .host_ip_runtime(
                    Some(Box::new(move |client, peer| {
                        client
                            .send(
                                peer,
                                &Packet::SetUser(peer.0 as usize),
                                Reliability::Reliable,
                                Compression::Compressed,
                            )
                            .unwrap();
                        send.store(true, std::sync::atomic::Ordering::Relaxed);
                    })),
                    None,
                    &down.runtime.0,
                )
                .unwrap();
        } else if input.just_pressed(KeyCode::KeyK) {
            info!("joining ip");
            #[cfg(feature = "ip")]
            let send = send_sleep.0.clone();
            #[cfg(feature = "ip")]
            client
                .join_ip_runtime(
                    "127.0.0.1".parse().unwrap(),
                    Some(Box::new(move |_, _| {
                        send.store(true, std::sync::atomic::Ordering::Relaxed);
                    })),
                    None,
                    &down.runtime.0,
                )
                .unwrap();
        }
    }
}
#[derive(Resource, Default)]
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
#[derive(Resource, Default)]
pub struct SyncActions {
    pub killed: Vec<SyncObjectMe>,
    pub take_owner: Vec<(SyncObject, SyncObjectMe)>,
    pub reorder: Vec<(SyncObjectMe, Vec<String>)>,
    pub draw: Vec<(SyncObjectMe, Vec<(SyncObjectMe, Trans)>, usize)>,
    pub flip: Vec<SyncObjectMe>,
}
#[derive(Encode, Decode, Debug)]
pub enum Packet {
    Pos(Vec<(SyncObjectMe, Trans, Phys, bool)>),
    Request(SyncObjectMe),
    Received(SyncObjectMe),
    Dead(SyncObjectMe),
    Take(SyncObject, SyncObjectMe),
    New(SyncObjectMe, Pile, Trans),
    NewShape(SyncObjectMe, Shape, Trans),
    Flip(SyncObjectMe),
    Reorder(SyncObjectMe, Vec<String>),
    Draw(SyncObjectMe, Vec<(SyncObjectMe, Trans)>, usize),
    SetUser(usize),
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
pub struct Trans {
    pub translation: (u32, u32, u32),
    pub rotation: u128,
}
impl Trans {
    pub fn from(value: &GlobalTransform) -> Self {
        Self {
            translation: unsafe { mem::transmute::<Vec3, (u32, u32, u32)>(value.translation()) },
            rotation: unsafe { mem::transmute::<Quat, u128>(value.rotation()) },
        }
    }
    pub fn from_transform(value: &Transform) -> Self {
        Self {
            translation: unsafe { mem::transmute::<Vec3, (u32, u32, u32)>(value.translation) },
            rotation: unsafe { mem::transmute::<Quat, u128>(value.rotation) },
        }
    }
}
impl From<Trans> for Transform {
    fn from(value: Trans) -> Self {
        Self {
            translation: unsafe { mem::transmute::<(u32, u32, u32), Vec3>(value.translation) },
            rotation: unsafe { mem::transmute::<u128, Quat>(value.rotation) },
            scale: Vec3::splat(1.0),
        }
    }
}
#[derive(Component, Default, Debug, Encode, Decode, Eq, PartialEq, Hash, Copy, Clone)]
pub struct SyncObject {
    pub user: u64,
    pub id: u64,
}
#[derive(Component, Default, Debug, Encode, Decode, Eq, PartialEq, Copy, Clone)]
pub struct SyncObjectMe(pub u64);
impl SyncObjectMe {
    pub fn new(rand: &mut Single<&mut WyRand, With<GlobalRng>>, count: &mut SyncCount) -> Self {
        count.add(1);
        Self(rand.next_u64())
    }
}
#[derive(Resource, Default)]
pub struct SyncCount {
    vec: Vec<(SyncObjectMe, Trans, Phys, bool)>,
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
    pub fn take(&mut self) -> Vec<(SyncObjectMe, Trans, Phys, bool)> {
        mem::take(&mut self.vec)
    }
    pub fn give(&mut self, vec: Vec<(SyncObjectMe, Trans, Phys, bool)>) {
        self.vec = vec;
    }
}
#[derive(Component)]
pub struct InOtherHand;
