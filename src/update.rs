use crate::counters::Value;
use crate::download::{
    Exact, get_alts, get_deck, get_deck_export, spawn_singleton, spawn_singleton_id,
};
use crate::misc::{adjust_meshes, is_reversed, move_up, new_pile, new_pile_at, repaint_face};
use crate::setup::{EscMenu, FontRes, SideMenu, T, W, Wall};
use crate::sync::{Packet, SyncObjectMe, Trans};
use crate::*;
use avian3d::math::Vector;
use bevy::input::mouse::{
    AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit, MouseWheel,
};
use bevy::input_focus::InputFocus;
use bevy::picking::hover::HoverMap;
use bevy::window::PrimaryWindow;
use bevy_prng::WyRand;
use bevy_rand::global::GlobalRng;
use bevy_rich_text3d::Text3d;
use bevy_tangled::{ClientTrait, Compression, PeerId, Reliability};
use bevy_ui_text_input::{TextInputBuffer, TextInputContents, TextInputMode, TextInputNode};
use cosmic_text::Edit;
#[cfg(feature = "calc")]
use kalc_lib::complex::NumStr;
#[cfg(feature = "calc")]
use kalc_lib::units::{Number, Options, Variable};
use rand::Rng;
use std::f32::consts::PI;
use std::mem;
pub fn gather_hand(
    mut hand: Single<(&Transform, &mut Hand, Entity), With<Owned>>,
    mut cards: Query<
        (
            Entity,
            &mut GravityScale,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &Pile,
        ),
        (
            With<Pile>,
            Without<Hand>,
            Without<InHand>,
            Without<FollowMouse>,
        ),
    >,
    spatial: SpatialQuery,
    mut commands: Commands,
) {
    let intersections = spatial.shape_intersections(
        &Collider::cuboid(4096.0, 256.0, 32.0),
        hand.0.translation,
        hand.0.rotation,
        &SpatialQueryFilter::DEFAULT,
    );
    for ent in intersections {
        if let Ok((entity, mut grav, mut linvel, mut angvel, pile)) = cards.get_mut(ent)
            && pile.len() == 1
        {
            linvel.0 = default();
            angvel.0 = default();
            grav.0 = 0.0;
            commands.entity(entity).insert(InHand(hand.1.count));
            commands.entity(entity).insert(RigidBodyDisabled);
            hand.1.count += 1;
            commands.entity(hand.2).add_child(entity);
        }
    }
}
pub fn update_hand(
    mut hand: Single<(&Transform, &mut Hand, Option<&Children>), With<Owned>>,
    mut card: Query<(&mut InHand, &mut Transform), (With<InHand>, Without<Hand>)>,
) {
    if let Some(children) = hand.2 {
        for child in children.into_iter() {
            let (mut entry, mut transform) = card.get_mut(*child).unwrap();
            if let Some((i, n)) = hand
                .1
                .removed
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.cmp(b.1))
                .map(|(a, b)| (a, *b))
                && entry.0 > n
            {
                hand.1.removed.remove(i);
                hand.1.removed.push(entry.0);
                entry.0 = n;
            }
            let idx = entry.0 as f32 - hand.1.count as f32 / 2.0;
            transform.translation = Vec3::new((idx + 0.5) * CARD_WIDTH / 2.0, idx * 2.0, 0.0);
        }
    }
    hand.1.removed.clear();
}
pub fn follow_mouse(
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window, With<PrimaryWindow>>,
    cards: Query<(&Collider, &Transform), Without<FollowMouse>>,
    mut commands: Commands,
    time_since: Res<Time>,
    spatial: SpatialQuery,
    walls: Query<(), With<Wall>>,
    mut card: Single<
        (
            Entity,
            &mut Transform,
            &mut GravityScale,
            &mut LinearVelocity,
            &Collider,
        ),
        With<FollowMouse>,
    >,
    menu: Res<Menu>,
) {
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let (camera, camera_transform) = camera.into_inner();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };
    if matches!(*menu, Menu::World | Menu::Side | Menu::Counter)
        && mouse_input.pressed(MouseButton::Left)
    {
        card.3.y = 0.0;
        let aabb = card.4.aabb(card.1.translation, card.1.rotation);
        if let Some(max) = spatial
            .shape_intersections(
                card.4,
                card.1.translation,
                card.1.rotation,
                &SpatialQueryFilter::from_mask(u32::MAX - 0b100),
            )
            .into_iter()
            .filter_map(|a| {
                if !walls.contains(a)
                    && let Ok((collider, transform)) = cards.get(a)
                {
                    let y = collider
                        .aabb(transform.translation, transform.rotation)
                        .max
                        .y;
                    Some(y)
                } else {
                    None
                }
            })
            .reduce(f32::max)
        {
            let max = max.max(aabb.max.y);
            card.1.translation.y = max + 8.0;
        }
        if let Some(time) =
            ray.intersect_plane(card.1.translation, InfinitePlane3d { normal: Dir3::Y })
        {
            let mut point = ray.get_point(time);
            point.x = point.x.clamp(
                T - W + (aabb.min.x - card.1.translation.x).abs(),
                W - T - (aabb.max.x - card.1.translation.x).abs(),
            );
            point.z = point.z.clamp(
                T - W + (aabb.min.z - card.1.translation.z).abs(),
                W - T - (aabb.max.z - card.1.translation.z).abs(),
            );
            card.1.translation = point;
        }
    } else {
        if let Some(time) =
            ray.intersect_plane(card.1.translation, InfinitePlane3d { normal: Dir3::Y })
        {
            let point = ray.get_point(time);
            card.3.0 = (point - card.1.translation) / time_since.delta_secs();
        }
        commands.entity(card.0).remove::<FollowMouse>();
        commands.entity(card.0).remove::<SleepingDisabled>();
        card.2.0 = GRAVITY
    }
}
pub fn listen_for_mouse(
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut pset: ParamSet<(
        SpatialQuery,
        Query<
            (&mut Collider, &mut GravityScale, &mut CollisionLayers),
            Or<(With<Pile>, With<Shape>)>,
        >,
    )>,
    mut cards: Query<(&mut Pile, &Children, Option<&ChildOf>, Option<&InHand>)>,
    mut mats: Query<&mut MeshMaterial3d<StandardMaterial>>,
    mut hands: Query<(&mut Hand, Option<&Owned>, Entity)>,
    mut vels: Query<(&mut LinearVelocity, &mut AngularVelocity)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    card_base: Res<CardBase>,
    input: Res<ButtonInput<KeyCode>>,
    #[cfg(not(feature = "wasm"))] mut clipboard: ResMut<Clipboard>,
    #[cfg(feature = "wasm")] clipboard: Res<Clipboard>,
    (
        zoom,
        down,
        asset_server,
        mut game_clipboard,
        mut count,
        mut sync_actions,
        ids,
        others_ids,
        mut query_meshes,
        follow,
        mut shape,
        mut menu,
        mut active_input,
        side,
        search_deck,
    ): (
        Option<Single<(Entity, &mut ZoomHold, &mut ImageNode)>>,
        ResMut<Download>,
        Res<AssetServer>,
        ResMut<GameClipboard>,
        ResMut<SyncCount>,
        ResMut<SyncActions>,
        Query<&SyncObjectMe>,
        Query<&SyncObject>,
        Query<
            (&mut Mesh3d, &mut Transform),
            (
                Without<Children>,
                With<ChildOf>,
                Without<Shape>,
                Without<Pile>,
            ),
        >,
        Option<Single<Entity, With<FollowMouse>>>,
        Query<&mut Shape>,
        ResMut<Menu>,
        ResMut<InputFocus>,
        Option<Single<Entity, With<SideMenu>>>,
        Option<Single<(Entity, &SearchDeck)>>,
    ),
    (mut rand, text, font, mut text3d, children, mut transform): (
        Single<&mut WyRand, With<GlobalRng>>,
        Option<Single<&TextInputContents>>,
        Res<FontRes>,
        Query<&mut Text3d>,
        Query<&Children, Without<Pile>>,
        Query<&mut Transform, Or<(With<Pile>, With<Shape>)>>,
    ),
) {
    if matches!(*menu, Menu::Esc)
        || (matches!(*menu, Menu::Side | Menu::Counter) && active_input.get().is_some())
    {
        return;
    }
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let (camera, camera_transform) = camera.into_inner();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };
    let hit = pset.p0().cast_ray(
        ray.origin,
        ray.direction,
        f32::MAX,
        true,
        &SpatialQueryFilter::DEFAULT,
    );
    let mut colliders = pset.p1();
    if let Some(RayHitData { entity, .. }) = hit {
        let Ok(mut transform) = transform.get_mut(entity) else {
            return;
        };
        if let Ok((mut pile, children, parent, inhand)) = cards.get_mut(entity) {
            if input.just_pressed(KeyCode::KeyF) {
                transform.rotate_local_z(PI);
                if let Some(entity) =
                    search_deck.and_then(|s| if s.1.0 == entity { Some(s.0) } else { None })
                {
                    update_search(
                        &mut commands,
                        entity,
                        &pile,
                        &transform,
                        text.as_ref().unwrap().get(),
                    );
                }
            } else if input.just_pressed(KeyCode::KeyR) {
                if pile.len() > 1 {
                    if let Ok(id) = others_ids.get(entity) {
                        let myid = SyncObjectMe::new(&mut rand, &mut count);
                        sync_actions.take_owner.push((*id, myid));
                    }
                    pile.shuffle(&mut rand);
                    let card = pile.last();
                    repaint_face(&mut mats, &mut materials, card, children);
                    if let Ok(id) = ids.get(entity) {
                        sync_actions
                            .reorder
                            .push((*id, pile.iter().map(|a| a.id.clone()).collect()));
                    }
                    if let Some(entity) =
                        search_deck.and_then(|s| if s.1.0 == entity { Some(s.0) } else { None })
                    {
                        update_search(
                            &mut commands,
                            entity,
                            &pile,
                            &transform,
                            text.as_ref().unwrap().get(),
                        );
                    }
                }
            } else if input.just_pressed(KeyCode::Backspace)
                && input.all_pressed([KeyCode::ControlLeft, KeyCode::AltLeft])
            {
                if ids.contains(entity) {
                    count.rem(1);
                }
                sync_actions.killed.push(*ids.get(entity).unwrap());
                commands.entity(entity).despawn();
                if let Some(entity) =
                    search_deck.and_then(|s| if s.1.0 == entity { Some(s.0) } else { None })
                {
                    commands.entity(entity).despawn()
                }
            } else if input.just_pressed(KeyCode::KeyC) && input.pressed(KeyCode::ControlLeft) {
                if input.pressed(KeyCode::ShiftLeft) {
                    *game_clipboard = GameClipboard::Pile(pile.clone());
                } else if !is_reversed(&transform) {
                    let card = pile.get_card(&transform);
                    let text = format!("https://scryfall.com/card/{}", card.id);
                    #[cfg(feature = "wasm")]
                    let clipboard = *clipboard;
                    #[cfg(feature = "wasm")]
                    wasm_bindgen_futures::spawn_local(async move {
                        clipboard.set_text(&text).await;
                    });
                    #[cfg(not(feature = "wasm"))]
                    clipboard.set_text(&text);
                }
            } else if mouse_input.just_pressed(MouseButton::Left) {
                if let Some(parent) = parent
                    && let Some(inhand) = inhand
                {
                    let mut hand = hands.get_mut(parent.0).unwrap().0;
                    hand.count -= 1;
                    hand.removed.push(inhand.0);
                    transform.translation.y += 128.0;
                } else {
                    transform.translation.y += 8.0;
                }
                if input.pressed(KeyCode::ControlLeft) && pile.len() > 1 {
                    let len = pile.len() as f32 * CARD_THICKNESS;
                    let draw_len = if is_reversed(&transform) {
                        1
                    } else {
                        pile.len()
                    };
                    let new = pile.take_card(&transform);
                    if !pile.is_empty() {
                        let card = pile.last();
                        repaint_face(&mut mats, &mut materials, card, children);
                        adjust_meshes(
                            &pile,
                            children,
                            &mut meshes,
                            &mut query_meshes,
                            &mut transform,
                            &mut colliders.get_mut(entity).unwrap().0,
                        );
                    }
                    let mut transform = *transform;
                    transform.translation.y += len + 8.0;
                    if let Some(e) = follow {
                        commands.entity(*e).remove::<FollowMouse>();
                    }
                    let id = SyncObjectMe::new(&mut rand, &mut count);
                    new_pile_at(
                        Pile::Single(new.into()),
                        card_base.stock.clone(),
                        &mut materials,
                        &mut commands,
                        &mut meshes,
                        card_base.back.clone(),
                        card_base.side.clone(),
                        transform,
                        true,
                        None,
                        None,
                        Some(id),
                    );
                    if let Ok(lid) = ids.get(entity) {
                        sync_actions.draw.push((
                            *lid,
                            vec![(id, Trans::from_transform(&transform))],
                            draw_len,
                        ));
                    }
                    if let Some(entity) =
                        search_deck.and_then(|s| if s.1.0 == entity { Some(s.0) } else { None })
                    {
                        update_search(
                            &mut commands,
                            entity,
                            &pile,
                            &transform,
                            text.as_ref().unwrap().get(),
                        );
                    }
                } else {
                    if let Some(e) = follow {
                        commands.entity(*e).remove::<FollowMouse>();
                    }
                    if let Ok(id) = others_ids.get(entity) {
                        let myid = SyncObjectMe::new(&mut rand, &mut count);
                        sync_actions.take_owner.push((*id, myid));
                    }
                    colliders.get_mut(entity).unwrap().1.0 = 0.0;
                    commands
                        .entity(entity)
                        .insert(FollowMouse)
                        .remove::<InHand>()
                        .remove::<RigidBodyDisabled>()
                        .remove_parent_in_place();
                }
            } else if input.just_pressed(KeyCode::KeyE) {
                rotate_right(&mut transform);
            } else if input.just_pressed(KeyCode::KeyS)
                && input.pressed(KeyCode::ControlLeft)
                && pile.len() > 1
            {
                let mut start = *transform;
                start.translation.y -= pile.len() as f32 * CARD_THICKNESS;
                let mut transform = start;
                let mut vec = Vec::with_capacity(pile.len());
                for c in pile.drain(..) {
                    let id = SyncObjectMe::new(&mut rand, &mut count);
                    new_pile_at(
                        Pile::Single(c.into()),
                        card_base.stock.clone(),
                        &mut materials,
                        &mut commands,
                        &mut meshes,
                        card_base.back.clone(),
                        card_base.side.clone(),
                        transform,
                        false,
                        None,
                        None,
                        Some(id),
                    );
                    transform.translation.x += CARD_WIDTH + 4.0;
                    if transform.translation.x >= W - T - CARD_WIDTH - 4.0 {
                        transform.translation.x = start.translation.x;
                        transform.translation.z += CARD_HEIGHT + 4.0;
                    }
                    vec.push((id, Trans::from_transform(&transform)));
                }
                if let Ok(lid) = ids.get(entity) {
                    let len = vec.len();
                    sync_actions.draw.push((*lid, vec, len));
                }
                if ids.contains(entity) {
                    count.rem(1);
                }
                sync_actions.killed.push(*ids.get(entity).unwrap());
                commands.entity(entity).despawn();
                if let Some(entity) =
                    search_deck.and_then(|s| if s.1.0 == entity { Some(s.0) } else { None })
                {
                    update_search(
                        &mut commands,
                        entity,
                        &pile,
                        &transform,
                        text.as_ref().unwrap().get(),
                    );
                }
            } else if input.just_pressed(KeyCode::KeyQ) {
                rotate_left(&mut transform);
            } else if input.just_pressed(KeyCode::KeyO)
                && input.all_pressed([KeyCode::ControlLeft, KeyCode::ShiftLeft])
                && !is_reversed(&transform)
            {
                let top = pile.get_card(&transform);
                let v = Vec2::new(
                    transform.translation.x,
                    transform.translation.z - CARD_HEIGHT - 1.0,
                );
                let client = down.client.0.clone();
                let get_deck = down.get_deck.clone();
                let asset_server = asset_server.clone();
                let id = top.id.clone();
                info!("{}: {id} has requested printings", top.normal.name);
                #[cfg(not(feature = "wasm"))]
                down.runtime
                    .0
                    .spawn(async move { get_alts(&id, client, asset_server, get_deck, v).await });
                #[cfg(feature = "wasm")]
                wasm_bindgen_futures::spawn_local(async move {
                    get_alts(&id, client, asset_server, get_deck, v).await;
                })
            } else if input.just_pressed(KeyCode::KeyO)
                && !is_reversed(&transform)
                && zoom
                    .as_ref()
                    .map(|single| single.1.0 != entity.to_bits())
                    .unwrap_or(true)
            {
                let card = pile.get_mut_card(&transform);
                if let Some(alt) = &mut card.alt {
                    mem::swap(&mut card.normal, alt);
                    repaint_face(&mut mats, &mut materials, card, children);
                    card.is_alt = !card.is_alt;
                }
                if let Ok(id) = ids.get(entity) {
                    sync_actions.flip.push(*id);
                }
            } else if input.any_just_pressed([
                KeyCode::Digit1,
                KeyCode::Digit2,
                KeyCode::Digit3,
                KeyCode::Digit4,
                KeyCode::Digit5,
                KeyCode::Digit6,
                KeyCode::Digit7,
                KeyCode::Digit8,
                KeyCode::Digit9,
            ]) {
                if parent.is_none() {
                    let mut n = 0;
                    macro_rules! get {
                        ($(($a:expr, $b:expr)),*) => {
                            $(
                                if input.just_pressed($a){
                                    n = $b
                                }
                            )*
                        };
                    }
                    get!(
                        (KeyCode::Digit1, 1),
                        (KeyCode::Digit2, 2),
                        (KeyCode::Digit3, 3),
                        (KeyCode::Digit4, 4),
                        (KeyCode::Digit5, 5),
                        (KeyCode::Digit6, 6),
                        (KeyCode::Digit7, 7),
                        (KeyCode::Digit8, 8),
                        (KeyCode::Digit9, 9)
                    );
                    let n = n.min(pile.len());
                    let mut hand = hands.iter_mut().find(|e| e.1.is_some()).unwrap();
                    let mut vec = Vec::new();
                    let len = if is_reversed(&transform) {
                        n
                    } else {
                        pile.len()
                    };
                    for _ in 0..n {
                        let new = pile.take_card(&transform);
                        let id = SyncObjectMe::new(&mut rand, &mut count);
                        let mut ent = new_pile_at(
                            Pile::Single(new.into()),
                            card_base.stock.clone(),
                            &mut materials,
                            &mut commands,
                            &mut meshes,
                            card_base.back.clone(),
                            card_base.side.clone(),
                            Transform::default(),
                            false,
                            Some(hand.2),
                            None,
                            Some(id),
                        )
                        .unwrap();
                        ent.insert(InHand(hand.0.count));
                        ent.insert(RigidBodyDisabled);
                        vec.push((id, Trans::from_transform(&Transform::default())));
                        hand.0.count += 1;
                    }
                    if let Some(entity) =
                        search_deck.and_then(|s| if s.1.0 == entity { Some(s.0) } else { None })
                    {
                        update_search(
                            &mut commands,
                            entity,
                            &pile,
                            &transform,
                            text.as_ref().unwrap().get(),
                        );
                    }
                    if let Ok(lid) = ids.get(entity) {
                        if !is_reversed(&transform) {
                            vec.reverse();
                        }
                        sync_actions.draw.push((*lid, vec, len));
                    }
                    if !pile.is_empty() {
                        let card = pile.last();
                        repaint_face(&mut mats, &mut materials, card, children);
                        adjust_meshes(
                            &pile,
                            children,
                            &mut meshes,
                            &mut query_meshes,
                            &mut transform,
                            &mut colliders.get_mut(entity).unwrap().0,
                        );
                    } else {
                        if let Ok(id) = ids.get(entity) {
                            sync_actions.killed.push(*id);
                            count.rem(1);
                        }
                        commands.entity(entity).despawn();
                    }
                }
            } else if input.just_pressed(KeyCode::KeyZ) {
                search(
                    entity,
                    &pile,
                    &transform,
                    &side,
                    &mut commands,
                    &mut active_input,
                    font.0.clone(),
                );
                *menu = Menu::Side;
            }
            if input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) {
                let mut spawn = || {
                    let card = pile.get_card(&transform);
                    commands.spawn((
                        Node {
                            width: Val::Px(CARD_WIDTH),
                            height: Val::Px(CARD_HEIGHT),
                            ..default()
                        },
                        ImageNode::new(card.normal.image().clone()),
                        ZoomHold(entity.to_bits(), false),
                    ));
                };
                if let Some(mut single) = zoom {
                    if single.1.0 != entity.to_bits() {
                        if !is_reversed(&transform) {
                            spawn();
                        }
                        commands.entity(single.0).despawn();
                    } else if input.just_pressed(KeyCode::KeyO)
                        && let Some(alt) = &pile.get_card(&transform).alt
                    {
                        let card = pile.get_card(&transform);
                        single.2.image =
                            if single.1.1 { &card.normal } else { alt }.image().clone();
                        single.1.1 = !single.1.1;
                    }
                } else if !is_reversed(&transform) {
                    spawn()
                }
            } else if let Some(single) = zoom {
                commands.entity(single.0).despawn();
            }
        } else if let Ok((_, mut phys, mut layers)) = colliders.get_mut(entity) {
            if let Some(single) = zoom {
                commands.entity(single.0).despawn();
            }
            if mouse_input.just_pressed(MouseButton::Right)
                && let Ok(s) = shape.get_mut(entity)
                && let Shape::Counter(v) = s.into_inner()
            {
                v.0 -= 1;
                let ent = children.get(entity).unwrap()[0];
                let mut text = text3d.get_mut(ent).unwrap();
                *text.get_single_mut().unwrap() = v.0.to_string();
                if let Ok(id) = ids.get(entity) {
                    sync_actions.counter_me.push((*id, v.clone()));
                } else if let Ok(id) = others_ids.get(entity) {
                    sync_actions.counter.push((*id, v.clone()));
                }
            } else if mouse_input.just_pressed(MouseButton::Left) {
                if !input.pressed(KeyCode::ControlLeft)
                    && let Ok(s) = shape.get_mut(entity)
                    && let Shape::Counter(v) = s.into_inner()
                {
                    v.0 += 1;
                    let ent = children.get(entity).unwrap()[0];
                    let mut text = text3d.get_mut(ent).unwrap();
                    *text.get_single_mut().unwrap() = v.0.to_string();
                    if let Ok(id) = ids.get(entity) {
                        sync_actions.counter_me.push((*id, v.clone()));
                    } else if let Ok(id) = others_ids.get(entity) {
                        sync_actions.counter.push((*id, v.clone()));
                    }
                } else {
                    if let Some(e) = follow {
                        commands.entity(*e).remove::<FollowMouse>();
                    }
                    if let Ok(id) = others_ids.get(entity) {
                        let myid = SyncObjectMe::new(&mut rand, &mut count);
                        sync_actions.take_owner.push((*id, myid));
                    }
                    phys.0 = 0.0;
                    commands.entity(entity).insert(FollowMouse);
                    commands.entity(entity).insert(SleepingDisabled);
                }
            } else if input.just_pressed(KeyCode::KeyC)
                && input.all_pressed([KeyCode::ControlLeft, KeyCode::ShiftLeft])
                && let Ok(shape) = shape.get(entity)
            {
                *game_clipboard = GameClipboard::Shape(shape.clone());
            } else if input.just_pressed(KeyCode::KeyR)
                && input.pressed(KeyCode::ControlLeft)
                && let Ok(s) = shape.get(entity)
                && let Shape::Counter(v) = s
            {
                #[cfg(feature = "calc")]
                {
                    *menu = Menu::Counter;
                    let mut input_buffer = TextInputBuffer::default();
                    let editor = &mut input_buffer.editor;
                    editor.insert_string("n", None);
                    let ent = commands
                        .spawn((
                            CounterMenu(entity, v.clone()),
                            Node {
                                width: Val::Percent(20.0),
                                height: Val::Px(FONT_HEIGHT * 2.0 * 1.5),
                                ..default()
                            },
                            BackgroundColor(bevy::color::Color::srgba_u8(0, 0, 0, 127)),
                            TextInputNode {
                                mode: TextInputMode::SingleLine,
                                clear_on_submit: false,
                                unfocus_on_submit: false,
                                ..default()
                            },
                            TextFont {
                                font: font.0.clone(),
                                font_size: FONT_SIZE * 2.0,
                                ..default()
                            },
                            TextInputContents::default(),
                            input_buffer,
                        ))
                        .id();
                    active_input.set(ent);
                }
            } else if (input.just_pressed(KeyCode::KeyR)
                || input.all_pressed([KeyCode::KeyR, KeyCode::AltLeft]))
                && let Ok((mut lv, mut av)) = vels.get_mut(entity)
            {
                commands.entity(entity).insert(TempDisable);
                if layers.filters & 0b01 == 0b01 {
                    layers.filters = (layers.filters.0 - 0b01).into();
                }
                if let Ok(id) = others_ids.get(entity) {
                    let myid = SyncObjectMe::new(&mut rand, &mut count);
                    sync_actions.take_owner.push((*id, myid));
                }
                lv.y = 4096.0;
                av.x = if rand.random() { 1.0 } else { -1.0 }
                    * (rand.random_range(32.0..64.0) + av.x.abs());
                av.y = if rand.random() { 1.0 } else { -1.0 }
                    * (rand.random_range(32.0..64.0) + av.y.abs());
                av.z = if rand.random() { 1.0 } else { -1.0 }
                    * (rand.random_range(32.0..64.0) + av.z.abs());
            } else if input.just_pressed(KeyCode::KeyE) {
                rotate_right(&mut transform)
            } else if input.just_pressed(KeyCode::KeyQ) {
                rotate_left(&mut transform)
            }
        } else if let Some(single) = zoom {
            commands.entity(single.0).despawn();
        }
    } else if let Some(single) = zoom {
        commands.entity(single.0).despawn();
    }
}
fn rotate_left(transform: &mut Mut<Transform>) {
    let (_, rot, _) = transform.rotation.to_euler(EulerRot::XYZ);
    let n = (2.0 * rot / PI).round() as isize;
    transform.rotate_y(
        match n {
            0 => PI / 2.0,
            1 => PI,
            2 | -2 => -PI / 2.0,
            -1 => 0.0,
            _ => unreachable!(),
        } - rot,
    );
}
fn rotate_right(transform: &mut Mut<Transform>) {
    let (_, rot, _) = transform.rotation.to_euler(EulerRot::XYZ);
    let n = (2.0 * rot / PI).round() as isize;
    transform.rotate_y(
        match n {
            0 => -PI / 2.0,
            1 => 0.0,
            2 | -2 => PI / 2.0,
            -1 => PI,
            _ => unreachable!(),
        } - rot,
    );
}
#[derive(Component)]
pub struct CounterMenu(Entity, Value);
#[derive(Component)]
pub struct TempDisable;
#[derive(Component)]
pub struct CardSpot;
pub fn reset_layers(
    mut phys: Query<(Entity, &LinearVelocity, &mut CollisionLayers), With<TempDisable>>,
    mut commands: Commands,
) {
    for (e, v, mut c) in phys.iter_mut() {
        if v.y <= 0.0 {
            c.filters.0 += 0b01;
            commands.entity(e).remove::<TempDisable>();
        }
    }
}
pub fn esc_menu(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    ents: Query<&mut Visibility, With<EscMenu>>,
    mut menu: ResMut<Menu>,
    side: Option<Single<Entity, With<SideMenu>>>,
    counter: Option<Single<Entity, With<CounterMenu>>>,
    text: Query<Entity, With<TextInputContents>>,
    hover_map: Res<HoverMap>,
    mut active_input: ResMut<InputFocus>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    if input.just_pressed(KeyCode::Escape)
        || (input.just_pressed(KeyCode::Enter) && matches!(*menu, Menu::Counter))
    {
        if let Some(e) = side {
            commands.entity(*e).despawn()
        }
        if let Some(e) = counter {
            commands.entity(*e).despawn()
        }
        let new = if matches!(*menu, Menu::Esc | Menu::Side | Menu::Counter) {
            *menu = Menu::World;
            Visibility::Hidden
        } else {
            *menu = Menu::Esc;
            Visibility::Visible
        };
        for mut visibility in ents {
            *visibility = new;
        }
    }
    if mouse_input.just_pressed(MouseButton::Left) {
        for pointer_event in hover_map.values() {
            for entity in pointer_event.keys().copied() {
                for text in text.iter() {
                    if text == entity {
                        active_input.set(entity);
                    } else {
                        active_input.clear();
                    }
                }
            }
        }
    }
}
pub fn update_search(
    commands: &mut Commands,
    search: Entity,
    pile: &Pile,
    transform: &Transform,
    text: &str,
) {
    let mut search = commands.get_entity(search).unwrap();
    search.clear_children();
    search.with_children(|parent| {
        let node = |(i, c): (usize, &SubCard)| {
            parent.spawn((
                TargetCard(i),
                ImageNode::new(c.normal.image.clone_handle()),
                Node {
                    aspect_ratio: Some(CARD_WIDTH / CARD_HEIGHT),
                    ..default()
                },
            ));
        };
        if is_reversed(transform) {
            pile.iter()
                .enumerate()
                .filter(|(_, c)| c.filter(text))
                .for_each(node);
        } else {
            pile.iter()
                .enumerate()
                .filter(|(_, c)| c.filter(text))
                .rev()
                .for_each(node);
        }
    });
}
pub fn pick_from_list(
    hover_map: Res<HoverMap>,
    query: Query<&TargetCard>,
    search_deck: Single<(Entity, &SearchDeck)>,
    mut decks: Query<(&mut Pile, &mut Transform, &Children)>,
    menu: Res<Menu>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
    card_base: Res<CardBase>,
    mut mats: Query<&mut MeshMaterial3d<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query_meshes: Query<
        (&mut Mesh3d, &mut Transform),
        (
            Without<Children>,
            With<ChildOf>,
            Without<Shape>,
            Without<Pile>,
        ),
    >,
    (mut colliders, follow, mut rand, mut count, ids, mut sync_actions, others_ids, text): (
        Query<&mut Collider>,
        Option<Single<Entity, With<FollowMouse>>>,
        Single<&mut WyRand, With<GlobalRng>>,
        ResMut<SyncCount>,
        Query<&SyncObjectMe>,
        ResMut<SyncActions>,
        Query<&SyncObject>,
        Single<&TextInputContents>,
    ),
) {
    if !matches!(*menu, Menu::Side) || !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }
    for pointer_event in hover_map.values() {
        for entity in pointer_event.keys().copied() {
            if let Ok(card) = query.get(entity)
                && let Ok((mut pile, mut trans, children)) = decks.get_mut(search_deck.1.0)
            {
                commands.entity(entity).despawn();
                let entity = search_deck.1.0;
                if let Ok(id) = others_ids.get(entity) {
                    let myid = SyncObjectMe::new(&mut rand, &mut count);
                    sync_actions.take_owner.push((*id, myid));
                }
                let len = pile.len() as f32 * CARD_THICKNESS;
                let new = pile.remove(card.0);
                if !pile.is_empty() {
                    let card = pile.last();
                    repaint_face(&mut mats, &mut materials, card, children);
                    adjust_meshes(
                        &pile,
                        children,
                        &mut meshes,
                        &mut query_meshes,
                        &mut trans,
                        &mut colliders.get_mut(entity).unwrap(),
                    );
                }
                let mut transform = *trans;
                transform.translation.y += len + 8.0;
                if let Some(e) = &follow {
                    commands.entity(**e).remove::<FollowMouse>();
                }
                let id = SyncObjectMe::new(&mut rand, &mut count);
                new_pile_at(
                    Pile::Single(new.into()),
                    card_base.stock.clone(),
                    &mut materials,
                    &mut commands,
                    &mut meshes,
                    card_base.back.clone(),
                    card_base.side.clone(),
                    transform,
                    true,
                    None,
                    None,
                    Some(id),
                );
                if let Ok(lid) = ids.get(entity) {
                    sync_actions.draw.push((
                        *lid,
                        vec![(id, Trans::from_transform(&transform))],
                        card.0 + 1,
                    ));
                }
                update_search(&mut commands, search_deck.0, &pile, &trans, text.get());
            }
        }
    }
}
#[allow(dead_code)]
#[derive(Component)]
pub struct TargetCard(pub usize);
#[derive(Component)]
pub struct SearchDeck(pub Entity);
pub fn update_search_deck(
    mut commands: Commands,
    text: Single<&TextInputContents, Changed<TextInputContents>>,
    single: Option<Single<(Entity, &SearchDeck)>>,
    query: Query<(&Pile, &Transform)>,
    menu: Res<Menu>,
    counter: Option<Single<&CounterMenu>>,
    mut text3d: Query<&mut Text3d>,
    mut children: Query<(&Children, &mut Shape)>,
    mut sync_actions: ResMut<SyncActions>,
    ids: Query<&SyncObjectMe>,
    other_ids: Query<&SyncObject>,
) {
    match *menu {
        Menu::Side => {
            if let Some(single) = single {
                let (pile, transform) = query.get(single.1.0).unwrap();
                update_search(&mut commands, single.0, pile, transform, text.get())
            }
        }
        #[cfg(feature = "calc")]
        Menu::Counter => {
            if let Some(counter) = counter
                && let Ok(parsed) = kalc_lib::parse::input_var(
                    text.get(),
                    &[Variable {
                        name: vec!['n'],
                        parsed: vec![NumStr::Num(
                            Number::from_f64(counter.1.0 as f64, &Options::default()).into(),
                        )],
                        unparsed: counter.1.0.to_string(),
                        funcvars: vec![],
                    }],
                    &mut Vec::new(),
                    &mut 0,
                    Options::default(),
                    false,
                    0,
                    Vec::new(),
                    false,
                    &mut Vec::new(),
                    None,
                    None,
                )
                && let Ok(value) = kalc_lib::math::do_math(parsed.0, Options::default(), parsed.1)
                && let NumStr::Num(n) = value
                && let Ok((children, v)) = children.get_mut(counter.0)
                && let Shape::Counter(v) = v.into_inner()
                && let Ok(mut text) = text3d.get_mut(children[0])
            {
                v.0 = n.number.real().to_f64().round() as i128;
                *text.get_single_mut().unwrap() = v.0.to_string();
                if let Ok(id) = ids.get(counter.0) {
                    sync_actions.counter_me.push((*id, v.clone()));
                } else if let Ok(id) = other_ids.get(counter.0) {
                    sync_actions.counter.push((*id, v.clone()));
                }
            }
        }
        _ => {}
    }
}
pub fn cam_translation(
    input: Res<ButtonInput<KeyCode>>,
    mouse_motion: Res<AccumulatedMouseScroll>,
    mut cam: Single<&mut Transform, With<Camera3d>>,
    menu: Res<Menu>,
    active_input: Res<InputFocus>,
) {
    if matches!(*menu, Menu::Esc)
        || (matches!(*menu, Menu::Side | Menu::Counter) && active_input.get().is_some())
    {
        return;
    }
    let scale = if input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        128.0
    } else {
        32.0
    };
    if !input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        let apply = |translate: Vec3, cam: &mut Transform| {
            let mut norm = translate.normalize();
            norm.y = 0.0;
            let abs = norm.length();
            if abs != 0.0 {
                let translate = norm * translate.length() / abs;
                cam.translation += translate;
            }
        };
        if input.pressed(KeyCode::KeyW) {
            let translate = cam.forward().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        if input.pressed(KeyCode::KeyA) {
            let translate = cam.left().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        if input.pressed(KeyCode::KeyD) {
            let translate = cam.right().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        if input.pressed(KeyCode::KeyS) {
            let translate = cam.back().as_vec3() * scale;
            apply(translate, &mut cam)
        }
    }
    if mouse_motion.delta.y != 0.0 {
        let translate = cam.forward().as_vec3() * scale * mouse_motion.delta.y * 16.0;
        if cam.translation.y < -translate.y {
            cam.translation.y /= 2.0;
        } else {
            cam.translation += translate;
        }
    }
    cam.translation = cam.translation.clamp(
        Vec3::new(T - W, 1.0, T - W),
        Vec3::new(W - T, 2.0 * (W - 2.0 * T), W - T),
    );
    if input.pressed(KeyCode::Space) {
        *cam.into_inner() =
            Transform::from_xyz(0.0, START_Y, START_Z).looking_at(Vec3::ZERO, Vec3::Y);
    }
}
pub fn cam_rotation(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut cam: Single<&mut Transform, With<Camera3d>>,
    menu: Res<Menu>,
    active_input: Res<InputFocus>,
) {
    if matches!(*menu, Menu::Esc)
        || (matches!(*menu, Menu::Side | Menu::Counter) && active_input.get().is_some())
    {
        return;
    }
    if mouse_button.pressed(MouseButton::Right) && mouse_motion.delta != Vec2::ZERO {
        let delta_yaw = -mouse_motion.delta.x * 0.001;
        let delta_pitch = -mouse_motion.delta.y * 0.001;
        let (yaw, pitch, roll) = cam.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;
        let pitch = pitch + delta_pitch;
        cam.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}
pub fn listen_for_deck(
    input: Res<ButtonInput<KeyCode>>,
    #[cfg(not(feature = "wasm"))] mut clipboard: ResMut<Clipboard>,
    #[cfg(feature = "wasm")] clipboard: Res<Clipboard>,
    down: ResMut<Download>,
    asset_server: Res<AssetServer>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window, With<PrimaryWindow>>,
    game_clipboard: Res<GameClipboard>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    card_base: Res<CardBase>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut rand: Single<&mut WyRand, With<GlobalRng>>,
    mut count: ResMut<SyncCount>,
    mut to_move: ResMut<ToMoveUp>,
    menu: Res<Menu>,
    active_input: Res<InputFocus>,
) {
    if matches!(*menu, Menu::Esc)
        || (matches!(*menu, Menu::Side | Menu::Counter) && active_input.get().is_some())
    {
        return;
    }
    if input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && (input.just_pressed(KeyCode::KeyV)
            || (input.pressed(KeyCode::ShiftLeft)
                && input.pressed(KeyCode::AltLeft)
                && input.pressed(KeyCode::KeyV)))
    {
        let Some(cursor_position) = window.cursor_position() else {
            return;
        };
        let (camera, camera_transform) = camera.into_inner();
        let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
            return;
        };
        let mut v = Vec2::default();
        if let Some(time) =
            ray.intersect_plane(Vec3::default(), InfinitePlane3d { normal: Dir3::Y })
        {
            let point = ray.get_point(time);
            v.x = point.x;
            v.y = point.z;
        }
        if !input.pressed(KeyCode::ShiftLeft) {
            let client = down.client.0.clone();
            let decks = down.get_deck.clone();
            let asset_server = asset_server.clone();
            #[cfg(not(feature = "wasm"))]
            let paste = clipboard.get_text();
            #[cfg(feature = "wasm")]
            let clipboard = *clipboard;
            let f = async move {
                #[cfg(feature = "wasm")]
                let paste = clipboard.get_text().await;
                let paste = paste.trim();
                let paste = paste.trim_end_matches('/');
                if paste.starts_with("https://moxfield.com/decks/")
                    || paste.starts_with("https://www.moxfield.com/decks/")
                    || paste.len() == 22
                {
                    let id = paste.rsplit_once('/').map(|(_, b)| b).unwrap();
                    info!("{id} request received");
                    let url = format!("https://api2.moxfield.com/v3/decks/all/{id}");
                    get_deck(url, client, asset_server, decks, v).await;
                } else if paste.starts_with("https://scryfall.com/card/") {
                    if paste.chars().filter(|c| *c == '/').count() == 4 {
                        let id = paste.rsplit_once('/').map(|(_, b)| b).unwrap();
                        info!("{id} request received");
                        spawn_singleton_id(client, asset_server, decks, v, id).await;
                    } else {
                        let mut split = paste.rsplitn(4, '/');
                        let Some(cn) = split.nth(1) else { return };
                        let Some(set) = split.next() else { return };
                        let set = set.to_string();
                        let cn = cn.to_string();
                        info!("{set} {cn} request received");
                        spawn_singleton(client, asset_server, decks, v, set, cn).await;
                    }
                } else {
                    let mut list = Vec::new();
                    for l in paste.lines() {
                        if !l.starts_with(['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']) {
                            return;
                        }
                        let mut split = l.split(' ');
                        if let Some(num) = split.next()
                            && let Some(cn) = split.next_back()
                            && let Some(set) = split.next_back()
                            && let Ok(count) = num.parse()
                        {
                            list.push(Exact {
                                count,
                                cn: cn.to_string(),
                                set: set[1..set.len() - 1].to_string(),
                            });
                        } else {
                            return;
                        }
                    }
                    get_deck_export(list, client, asset_server, decks, v).await;
                }
            };
            #[cfg(not(feature = "wasm"))]
            down.runtime.0.spawn(f);
            #[cfg(feature = "wasm")]
            wasm_bindgen_futures::spawn_local(f);
        } else if let Some(ent) = match game_clipboard.clone() {
            GameClipboard::Pile(pile) => new_pile(
                pile,
                card_base.stock.clone(),
                &mut materials,
                &mut commands,
                &mut meshes,
                card_base.back.clone(),
                card_base.side.clone(),
                &mut rand,
                v,
                &mut count,
                None,
            ),
            GameClipboard::Shape(shape) => Some(
                shape
                    .create(
                        Transform::from_xyz(v.x, 256.0, v.y),
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                    )
                    .insert(SyncObjectMe::new(&mut rand, &mut count))
                    .id(),
            ),
            GameClipboard::None => None,
        } {
            to_move.0.push(ent)
        }
    }
}
pub fn register_deck(
    mut commands: Commands,
    decks: ResMut<Download>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    card_base: Res<CardBase>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut rand: Single<&mut WyRand, With<GlobalRng>>,
    mut count: ResMut<SyncCount>,
    client: Res<Client>,
    mut sent: ResMut<Sent>,
    mut to_move: ResMut<ToMoveUp>,
) {
    let mut decks = decks.get_deck.0.lock().unwrap();
    for (deck, v, id) in decks.drain(..) {
        info!("deck found of size {} at {} {}", deck.len(), v.x, v.y);
        if let Some(ent) = new_pile(
            deck,
            card_base.stock.clone(),
            &mut materials,
            &mut commands,
            &mut meshes,
            card_base.back.clone(),
            card_base.side.clone(),
            &mut rand,
            v,
            &mut count,
            id,
        ) {
            to_move.0.push(ent);
        }
        if let Some(id) = id {
            sent.del(id);
            client
                .send(
                    id.user,
                    &Packet::Received(id.id),
                    Reliability::Reliable,
                    Compression::Compressed,
                )
                .unwrap();
        }
    }
}
pub fn to_move_up(
    mut to_do: ResMut<ToMoveUp>,
    ents: Query<(&Collider, &mut Transform), Without<Wall>>,
    mut pset: ParamSet<(Query<&mut Position>, SpatialQuery)>,
) {
    for ent in to_do.0.drain(..) {
        move_up(ent, &ents, &mut pset);
    }
}
#[derive(Resource, Default)]
pub struct ToMoveUp(pub Vec<Entity>);
pub fn give_ents(
    to_do: Res<GiveEnts>,
    ents: Query<&SyncObject>,
    mut sync_actions: ResMut<SyncActions>,
    mut rand: Single<&mut WyRand, With<GlobalRng>>,
    mut count: ResMut<SyncCount>,
) {
    for peer in to_do.0.lock().unwrap().drain(..) {
        for id in ents {
            if id.user == peer {
                sync_actions
                    .take_owner
                    .push((*id, SyncObjectMe::new(&mut rand, &mut count)));
            }
        }
    }
}
#[derive(Resource, Default)]
pub struct GiveEnts(pub Arc<Mutex<Vec<PeerId>>>);
pub fn on_scroll_handler(
    mut scroll: On<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };
    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();
    let delta = &mut scroll.delta;
    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0. {
        let max = if delta.x > 0. {
            scroll_position.x >= max_offset.x
        } else {
            scroll_position.x <= 0.
        };
        if !max {
            scroll_position.x += delta.x;
            delta.x = 0.;
        }
    }
    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0. {
        let max = if delta.y > 0. {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.
        };
        if !max {
            scroll_position.y += delta.y;
            delta.y = 0.;
        }
    }
    if *delta == Vec2::ZERO {
        scroll.propagate(false);
    }
}
pub fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);
        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= 128.0;
        }
        if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            mem::swap(&mut delta.x, &mut delta.y);
        }
        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(Scroll { entity, delta });
            }
        }
    }
}
#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct Scroll {
    entity: Entity,
    delta: Vec2,
}
pub fn search(
    entity: Entity,
    pile: &Pile,
    transform: &Transform,
    side: &Option<Single<Entity, With<SideMenu>>>,
    commands: &mut Commands,
    active_input: &mut InputFocus,
    font: Handle<Font>,
) {
    let mut search = None;
    if let Some(e) = &side {
        commands.entity(**e).despawn()
    }
    let mut ent = commands.spawn((
        Node {
            width: Val::Percent(40.0),
            height: Val::Percent(100.0),
            left: Val::Percent(60.0),
            ..default()
        },
        SideMenu,
        Visibility::Visible,
        BackgroundColor(bevy::color::Color::srgba_u8(0, 0, 0, 127)),
    ));
    ent.with_children(|p| {
        let id = p
            .spawn((
                TextInputNode {
                    mode: TextInputMode::SingleLine,
                    clear_on_submit: false,
                    unfocus_on_submit: false,
                    ..default()
                },
                TextFont {
                    font,
                    font_size: FONT_SIZE,
                    ..default()
                },
                TextInputContents::default(),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Px(FONT_HEIGHT * 1.5),
                    ..default()
                },
            ))
            .id();
        active_input.set(id);
        search = Some(
            p.spawn((
                SearchDeck(entity),
                Node {
                    display: Display::Grid,
                    position_type: PositionType::Absolute,
                    top: Val::Px(FONT_HEIGHT * 1.5),
                    left: Val::Px(0.0),
                    height: Val::Percent(100.0),
                    width: Val::Percent(100.0),
                    grid_template_columns: vec![RepeatedGridTrack::percent(3, 100.0 / 3.0)],
                    align_content: AlignContent::Start,
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
            ))
            .id(),
        );
    });
    update_search(commands, search.unwrap(), pile, transform, "");
}
pub fn pile_merge(
    collision: On<CollisionStart>,
    mut piles: Query<(Entity, &mut Pile, &mut Transform, &Children, &mut Collider)>,
    mut mats: Query<&mut MeshMaterial3d<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query_meshes: Query<
        (&mut Mesh3d, &mut Transform),
        (
            Without<Children>,
            With<ChildOf>,
            Without<Shape>,
            Without<Pile>,
        ),
    >,
    mut commands: Commands,
    search_deck: Option<Single<(Entity, &SearchDeck)>>,
    text: Option<Single<&TextInputContents>>,
) {
    if let Ok((e1, p1, t1, _, _)) = piles.get(collision.collider1)
        && let Ok((e2, p2, t2, _, _)) = piles.get(collision.collider2)
        && e1 < e2
        && (t1.translation.x - t2.translation.x).abs() < CARD_WIDTH / 2.0
        && (t1.translation.z - t2.translation.z).abs() < CARD_HEIGHT / 2.0
        && is_reversed(t1) == is_reversed(t2)
    {
        let (
            (ent, mut bottom_pile, mut bottom_transform, children, mut collider),
            top_pile,
            top_ent,
        ) = if t1.translation.y < t2.translation.y {
            let p2 = p2.clone();
            (piles.get_mut(collision.collider1).unwrap(), p2, e2)
        } else {
            let p1 = p1.clone();
            (piles.get_mut(collision.collider2).unwrap(), p1, e1)
        };
        if is_reversed(&bottom_transform) {
            bottom_pile.extend_start(top_pile);
        } else {
            bottom_pile.extend(top_pile);
        }
        let card = bottom_pile.last();
        repaint_face(&mut mats, &mut materials, card, children);
        adjust_meshes(
            &bottom_pile,
            children,
            &mut meshes,
            &mut query_meshes,
            &mut bottom_transform,
            &mut collider,
        );
        if let Some(search_deck) = search_deck {
            if search_deck.1.0 == ent {
                update_search(
                    &mut commands,
                    search_deck.0,
                    &bottom_pile,
                    &bottom_transform,
                    text.as_ref().unwrap().get(),
                );
            } else if search_deck.1.0 == top_ent {
                commands.entity(search_deck.0).despawn();
            }
        }
        commands.entity(top_ent).despawn();
    }
}
pub fn set_card_spot(
    spatial: SpatialQuery,
    query: Query<&GlobalTransform, With<CardSpot>>,
    mut transforms: Query<
        (
            &mut Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &Pile,
        ),
        (With<SyncObjectMe>, Without<FollowMouse>),
    >,
) {
    for transform in query {
        let transform = transform.compute_transform();
        for ent in spatial.shape_intersections(
            &Collider::cuboid(CARD_WIDTH / 2.0, CARD_HEIGHT / 2.0, CARD_THICKNESS / 2.0),
            transform.translation,
            transform.rotation,
            &SpatialQueryFilter::DEFAULT,
        ) {
            if let Ok((mut t, mut lv, mut av, pile)) = transforms.get_mut(ent) {
                let mut transform = transform;
                transform.translation.y = pile.len() as f32 * CARD_THICKNESS / 2.0;
                if transform.translation.distance(t.translation) > CARD_THICKNESS {
                    lv.0 = Vector::default();
                    av.0 = Vector::default();
                    *t = transform;
                }
                continue;
            }
        }
    }
}
