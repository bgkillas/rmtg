use crate::download::{Exact, get_alts, get_deck, get_deck_export, spawn_singleton};
use crate::misc::{
    adjust_meshes, get_card, get_mut_card, is_reversed, make_material, move_up, new_pile,
    new_pile_at, repaint_face, take_card,
};
use crate::setup::{T, W, Wall};
use crate::sync::{Packet, SyncObjectMe};
use crate::*;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::window::PrimaryWindow;
use bevy_prng::WyRand;
use bevy_rand::global::GlobalEntropy;
use bitcode::encode;
use net::{PeerId, Reliability};
use rand::Rng;
use rand::seq::SliceRandom;
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
            && pile.0.len() == 1
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
            transform.rotation = Quat::from_rotation_x(-PI / 2.0);
        }
    }
    hand.1.removed.clear();
}
pub fn follow_mouse(
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform)>,
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
) {
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let (camera, camera_transform) = camera.into_inner();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };
    if mouse_input.pressed(MouseButton::Left) {
        card.3.y = 0.0;
        let aabb = card.4.aabb(card.1.translation, card.1.rotation);
        if let Some(max) = spatial
            .shape_intersections(
                card.4,
                card.1.translation,
                card.1.rotation,
                &SpatialQueryFilter::DEFAULT,
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
        card.2.0 = GRAVITY
    }
}
pub fn listen_for_mouse(
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform, Entity)>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut pset: ParamSet<(SpatialQuery, Query<&mut Collider>)>,
    mut cards: Query<(
        &mut Pile,
        &mut Transform,
        &Children,
        Option<&ChildOf>,
        Option<&InHand>,
    )>,
    mut mats: Query<&mut MeshMaterial3d<StandardMaterial>, Without<ZoomHold>>,
    mut hands: Query<(&mut Hand, Option<&Owned>, Entity)>,
    mut vels: Query<(&mut LinearVelocity, &mut AngularVelocity)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    card_base: Res<CardBase>,
    input: Res<ButtonInput<KeyCode>>,
    mut rand: GlobalEntropy<WyRand>,
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
        mut grav,
        shape,
    ): (
        Option<Single<(Entity, &mut ZoomHold, &mut MeshMaterial3d<StandardMaterial>)>>,
        ResMut<Download>,
        Res<AssetServer>,
        ResMut<GameClipboard>,
        ResMut<SyncCount>,
        ResMut<SyncActions>,
        Query<&SyncObjectMe>,
        Query<&SyncObject>,
        Query<(&mut Mesh3d, &mut Transform), Without<Children>>,
        Option<Single<Entity, With<FollowMouse>>>,
        Query<&mut GravityScale>,
        Query<&Shape>,
    ),
) {
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let (camera, camera_transform, cament) = camera.into_inner();
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
        if let Ok((mut pile, mut transform, children, parent, inhand)) = cards.get_mut(entity) {
            if input.just_pressed(KeyCode::KeyF) {
                transform.rotate_local_y(PI);
            } else if input.just_pressed(KeyCode::KeyR) {
                if pile.0.len() > 1 {
                    pile.0.shuffle(&mut rand);
                    let card = pile.0.last().unwrap();
                    repaint_face(&mut mats, &mut materials, card, children);
                }
            } else if input.just_pressed(KeyCode::Backspace)
                && input.all_pressed([KeyCode::ControlLeft, KeyCode::AltLeft])
            {
                if ids.contains(entity) {
                    count.rem(1);
                }
                sync_actions.killed.push(*ids.get(entity).unwrap());
                commands.entity(entity).despawn();
            } else if input.just_pressed(KeyCode::KeyC) && input.pressed(KeyCode::ControlLeft) {
                if input.pressed(KeyCode::ShiftLeft) {
                    *game_clipboard = GameClipboard::Pile(pile.clone());
                } else if !is_reversed(&transform) {
                    let card = get_card(&pile, &transform);
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
                if input.pressed(KeyCode::ControlLeft) && pile.0.len() > 1 {
                    let len = pile.0.len() as f32;
                    let new = take_card(&mut pile, &transform);
                    if !pile.0.is_empty() {
                        let card = pile.0.last().unwrap();
                        repaint_face(&mut mats, &mut materials, card, children);
                        adjust_meshes(
                            &pile,
                            children,
                            &mut meshes,
                            &mut query_meshes,
                            &mut transform,
                            &mut colliders.get_mut(entity).unwrap(),
                        );
                    }
                    let mut transform = *transform;
                    transform.translation.y += len + 8.0;
                    if let Some(e) = follow {
                        commands.entity(*e).remove::<FollowMouse>();
                    }
                    let id = SyncObjectMe::new(&mut rand, &mut count);
                    new_pile_at(
                        Pile(vec![new]),
                        card_base.stock.clone_weak(),
                        &mut materials,
                        &mut commands,
                        &mut meshes,
                        card_base.back.clone_weak(),
                        card_base.side.clone_weak(),
                        transform,
                        true,
                        None,
                        None,
                        Some(id),
                    );
                } else {
                    if let Some(e) = follow {
                        commands.entity(*e).remove::<FollowMouse>();
                    }
                    if let Ok(id) = others_ids.get(entity) {
                        let myid = SyncObjectMe::new(&mut rand, &mut count);
                        sync_actions.take_owner.push((*id, myid));
                    }
                    grav.get_mut(entity).unwrap().0 = 0.0;
                    commands
                        .entity(entity)
                        .insert(FollowMouse)
                        .remove::<InHand>()
                        .remove::<RigidBodyDisabled>()
                        .remove_parent_in_place();
                }
            } else if input.just_pressed(KeyCode::KeyE) {
                let (_, _, rot) = transform.rotation.to_euler(EulerRot::XYZ);
                let n = (2.0 * rot / PI).round() as isize;
                let rev = is_reversed(&transform);
                transform.rotation = Quat::from_rotation_z(match n {
                    0 => -PI / 2.0,
                    1 => 0.0,
                    2 | -2 => PI / 2.0,
                    -1 => PI,
                    _ => unreachable!(),
                });
                transform.rotate_x(-PI / 2.0);
                if rev {
                    transform.rotate_z(PI);
                }
            } else if input.just_pressed(KeyCode::KeyS)
                && input.pressed(KeyCode::ControlLeft)
                && pile.0.len() > 1
            {
                let mut start = *transform;
                start.translation.y -= pile.0.len() as f32;
                let mut transform = start;
                for c in pile.0.drain(..) {
                    let id = SyncObjectMe::new(&mut rand, &mut count);
                    new_pile_at(
                        Pile(vec![c]),
                        card_base.stock.clone_weak(),
                        &mut materials,
                        &mut commands,
                        &mut meshes,
                        card_base.back.clone_weak(),
                        card_base.side.clone_weak(),
                        transform,
                        false,
                        None,
                        None,
                        Some(id),
                    );
                    transform.translation.x += CARD_WIDTH;
                    if transform.translation.x >= W - T - CARD_WIDTH {
                        transform.translation.x = start.translation.x;
                        transform.translation.z += CARD_HEIGHT;
                    }
                }
                if ids.contains(entity) {
                    count.rem(1);
                }
                sync_actions.killed.push(*ids.get(entity).unwrap());
                commands.entity(entity).despawn();
            } else if input.just_pressed(KeyCode::KeyQ) {
                let (_, _, rot) = transform.rotation.to_euler(EulerRot::XYZ);
                let n = (2.0 * rot / PI).round() as isize;
                let rev = is_reversed(&transform);
                transform.rotation = Quat::from_rotation_z(match n {
                    0 => PI / 2.0,
                    1 => PI,
                    2 | -2 => -PI / 2.0,
                    -1 => 0.0,
                    _ => unreachable!(),
                });
                transform.rotate_x(-PI / 2.0);
                if rev {
                    transform.rotate_z(PI);
                }
            } else if input.just_pressed(KeyCode::KeyO)
                && input.all_pressed([KeyCode::ControlLeft, KeyCode::ShiftLeft])
                && !is_reversed(&transform)
            {
                let top = get_card(&pile, &transform);
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
                let card = get_mut_card(&mut pile, &transform);
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
                    let mut hand = hands.iter_mut().find(|e| e.1.is_some()).unwrap();
                    for _ in 0..n {
                        if !pile.0.is_empty() {
                            let new = take_card(&mut pile, &transform);
                            let mut ent = new_pile_at(
                                Pile(vec![new]),
                                card_base.stock.clone_weak(),
                                &mut materials,
                                &mut commands,
                                &mut meshes,
                                card_base.back.clone_weak(),
                                card_base.side.clone_weak(),
                                Transform::default(),
                                false,
                                Some(hand.2),
                                None,
                                Some(SyncObjectMe::new(&mut rand, &mut count)),
                            )
                            .unwrap();
                            ent.insert(InHand(hand.0.count));
                            ent.insert(RigidBodyDisabled);
                            hand.0.count += 1;
                        }
                    }
                    if !pile.0.is_empty() {
                        let card = pile.0.last().unwrap();
                        repaint_face(&mut mats, &mut materials, card, children);
                        adjust_meshes(
                            &pile,
                            children,
                            &mut meshes,
                            &mut query_meshes,
                            &mut transform,
                            &mut colliders.get_mut(entity).unwrap(),
                        );
                    } else {
                        if ids.contains(entity) {
                            count.rem(1);
                        }
                        sync_actions.killed.push(*ids.get(entity).unwrap());
                        commands.entity(entity).despawn();
                    }
                }
            } else if input.just_pressed(KeyCode::KeyZ) {
                //TODO search
            }
            if input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) {
                if let Some(mut single) = zoom {
                    if single.1.0 != entity.to_bits() {
                        commands.entity(single.0).despawn();
                    } else if input.just_pressed(KeyCode::KeyO)
                        && let Some(alt) = &get_card(&pile, &transform).alt
                    {
                        single.2.0 = make_material(
                            &mut materials,
                            if single.1.1 {
                                &get_card(&pile, &transform).normal
                            } else {
                                alt
                            }
                            .image()
                            .clone_weak(),
                        );
                        single.1.1 = !single.1.1;
                    }
                } else if !is_reversed(&transform) {
                    let card = get_card(&pile, &transform);
                    commands.entity(cament).with_child((
                        Mesh3d(card_base.stock.clone_weak()),
                        MeshMaterial3d(make_material(
                            &mut materials,
                            card.normal.image().clone_weak(),
                        )),
                        Transform::from_xyz(0.0, 0.0, -1024.0),
                        ZoomHold(entity.to_bits(), false),
                    ));
                }
            } else if let Some(single) = zoom {
                commands.entity(single.0).despawn();
            }
        } else if let Ok(mut grav) = grav.get_mut(entity) {
            if let Some(single) = zoom {
                commands.entity(single.0).despawn();
            }
            if mouse_input.just_pressed(MouseButton::Left) {
                if let Some(e) = follow {
                    commands.entity(*e).remove::<FollowMouse>();
                }
                if let Ok(id) = others_ids.get(entity) {
                    let myid = SyncObjectMe::new(&mut rand, &mut count);
                    sync_actions.take_owner.push((*id, myid));
                }
                grav.0 = 0.0;
                commands.entity(entity).insert(FollowMouse);
            } else if input.just_pressed(KeyCode::KeyC)
                && input.all_pressed([KeyCode::ControlLeft, KeyCode::ShiftLeft])
                && let Ok(shape) = shape.get(entity)
            {
                *game_clipboard = GameClipboard::Shape(*shape);
            } else if input.just_pressed(KeyCode::KeyR)
                && let Ok((mut lv, mut av)) = vels.get_mut(entity)
            {
                lv.y = 4096.0;
                av.x = if rand.random() { 1.0 } else { -1.0 }
                    * (rand.random_range(32.0..64.0) + av.x.abs());
                av.y = if rand.random() { 1.0 } else { -1.0 }
                    * (rand.random_range(32.0..64.0) + av.y.abs());
                av.z = if rand.random() { 1.0 } else { -1.0 }
                    * (rand.random_range(32.0..64.0) + av.z.abs());
            }
        } else if let Some(single) = zoom {
            commands.entity(single.0).despawn();
        }
    } else if let Some(single) = zoom {
        commands.entity(single.0).despawn();
    }
}
pub fn cam_translation(
    input: Res<ButtonInput<KeyCode>>,
    mouse_motion: Res<AccumulatedMouseScroll>,
    mut cam: Single<&mut Transform, With<Camera3d>>,
) {
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
) {
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
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window, With<PrimaryWindow>>,
    game_clipboard: Res<GameClipboard>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    card_base: Res<CardBase>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    mut rand: GlobalEntropy<WyRand>,
    mut count: ResMut<SyncCount>,
    mut to_move: ResMut<ToMoveUp>,
) {
    if input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && input.just_pressed(KeyCode::KeyV)
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
                if paste.starts_with("https://moxfield.com/decks/")
                    || paste.starts_with("https://www.moxfield.com/decks/")
                    || paste.len() == 22
                {
                    let id = paste.rsplit_once('/').map(|(_, b)| b).unwrap_or(paste);
                    info!("{id} request received");
                    let url = format!("https://api2.moxfield.com/v3/decks/all/{id}");
                    get_deck(url, client, asset_server, decks, v).await;
                } else if paste.starts_with("https://scryfall.com/card/") {
                    let mut split = paste.rsplitn(4, '/');
                    let Some(cn) = split.nth(1) else { return };
                    let Some(set) = split.next() else { return };
                    let set = set.to_string();
                    let cn = cn.to_string();
                    info!("{set} {cn} request received");
                    spawn_singleton(client, asset_server, decks, v, set, cn).await;
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
                card_base.stock.clone_weak(),
                &mut materials,
                &mut commands,
                &mut meshes,
                card_base.back.clone_weak(),
                card_base.side.clone_weak(),
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
                        &mut materials,
                        &mut meshes,
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
    mut rand: GlobalEntropy<WyRand>,
    mut count: ResMut<SyncCount>,
    client: Res<Client>,
    mut sent: ResMut<Sent>,
    mut to_move: ResMut<ToMoveUp>,
) {
    let mut decks = decks.get_deck.0.lock().unwrap();
    for (deck, v, id) in decks.drain(..) {
        info!("deck found of size {} at {} {}", deck.0.len(), v.x, v.y);
        if let Some(ent) = new_pile(
            deck,
            card_base.stock.clone_weak(),
            &mut materials,
            &mut commands,
            &mut meshes,
            card_base.back.clone_weak(),
            card_base.side.clone_weak(),
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
                .send_message(
                    PeerId(id.user),
                    &encode(&Packet::Received(SyncObjectMe(id.id))),
                    Reliability::Reliable,
                )
                .unwrap();
        }
    }
}
pub fn to_move_up(
    mut to_do: ResMut<ToMoveUp>,
    mut cards: Query<(&Collider, &mut Transform)>,
    walls: Query<(), With<Wall>>,
    spatial: SpatialQuery,
) {
    for ent in to_do.0.drain(..) {
        move_up(ent, &spatial, &mut cards, &walls);
    }
}
#[derive(Resource)]
pub struct ToMoveUp(pub Vec<Entity>);
