use crate::download::{get_alts, get_deck, spawn_singleton};
use crate::misc::{make_material, new_pile, new_pile_at};
use crate::*;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::window::PrimaryWindow;
use bevy_prng::WyRand;
use bevy_rand::global::GlobalEntropy;
use rand::Rng;
use rand::seq::SliceRandom;
use std::f32::consts::PI;
use std::mem;
pub fn gather_hand(
    mut hand: Single<(&Transform, &mut Hand, Entity), With<Owned>>,
    mut cards: Query<
        (Entity, &mut GravityScale, &mut Velocity, &Pile),
        (
            With<Pile>,
            Without<Hand>,
            Without<InHand>,
            Without<FollowMouse>,
        ),
    >,
    rapier_context: ReadRapierContext,
    mut commands: Commands,
) {
    let Ok(context) = rapier_context.single() else {
        return;
    };
    context.with_query_pipeline(QueryFilter::only_dynamic(), |query_pipeline| {
        for ent in query_pipeline.intersect_shape(
            hand.0.translation,
            hand.0.rotation,
            Collider::cuboid(1024.0, 128.0, 16.0).raw.0.as_ref(),
        ) {
            if let Ok((entity, mut grav, mut vel, pile)) = cards.get_mut(ent)
                && pile.0.len() == 1
            {
                vel.linvel = Vect::default();
                vel.angvel = Vect::default();
                grav.0 = 0.0;
                commands.entity(entity).insert(InHand(hand.1.count));
                hand.1.count += 1;
                commands.entity(hand.2).add_child(entity);
            }
        }
    });
}
pub fn update_hand(
    mut hand: Single<(&Transform, &mut Hand, &Children), With<Owned>>,
    mut card: Query<(&mut InHand, &mut Transform), (With<InHand>, Without<Hand>)>,
) {
    for child in hand.2.into_iter() {
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
            hand.1.removed.push(entry.0); //TODO quite bad
            entry.0 = n;
        }
        let idx = entry.0 as f32 - hand.1.count as f32 / 2.0;
        transform.translation = Vec3::new((idx + 0.5) * CARD_WIDTH / 2.0, idx * 2.0, 0.0);
        transform.rotation = Quat::from_rotation_x(-PI / 2.0);
    }
    hand.1.removed.clear();
}
pub fn follow_mouse(
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut card: Single<
        (
            Entity,
            &mut Transform,
            &mut GravityScale,
            &mut Velocity,
            &Collider,
        ),
        With<FollowMouse>,
    >,
    cards: Query<(&Pile, &Transform), Without<FollowMouse>>,
    mut commands: Commands,
    time_since: Res<Time>,
    rapier_context: ReadRapierContext,
) {
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let (camera, camera_transform) = camera.into_inner();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };
    if mouse_input.pressed(MouseButton::Left) {
        let Ok(context) = rapier_context.single() else {
            return;
        };
        if let Some(max) =
            context.with_query_pipeline(QueryFilter::only_dynamic(), |query_pipeline| {
                query_pipeline
                    .intersect_shape(card.1.translation, card.1.rotation, card.4.raw.0.as_ref())
                    .filter_map(|a| {
                        if a != card.0
                            && let Ok((pile, transform)) = cards.get(a)
                        {
                            Some(transform.translation.y + pile.0.len() as f32)
                        } else {
                            None
                        }
                    })
                    .reduce(f32::max)
            })
        {
            card.1.translation.y = max + 4.0;
        }
        if let Some(time) =
            ray.intersect_plane(card.1.translation, InfinitePlane3d { normal: Dir3::Y })
        {
            let point = ray.get_point(time);
            card.1.translation = point;
        }
    } else {
        if let Some(time) =
            ray.intersect_plane(card.1.translation, InfinitePlane3d { normal: Dir3::Y })
        {
            let point = ray.get_point(time);
            card.3.linvel = (point - card.1.translation) / time_since.delta_secs()
        }
        commands.entity(card.0).remove::<FollowMouse>();
        card.2.0 = GRAVITY
    }
}
pub fn listen_for_mouse(
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform, Entity)>,
    window: Single<&Window, With<PrimaryWindow>>,
    rapier_context: ReadRapierContext,
    mut cards: Query<(
        &mut Pile,
        &mut Transform,
        &Children,
        Option<&Reversed>,
        Option<&ChildOf>,
        Option<&InHand>,
    )>,
    mut mats: Query<&mut MeshMaterial3d<StandardMaterial>, Without<ZoomHold>>,
    mut hands: Query<(&mut Hand, Option<&Owned>, Entity)>,
    mut vels: Query<&mut Velocity>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    card_base: Res<CardBase>,
    input: Res<ButtonInput<KeyCode>>,
    mut rand: GlobalEntropy<WyRand>,
    zoom: Option<Single<(Entity, &mut ZoomHold, &mut MeshMaterial3d<StandardMaterial>)>>,
    (down, asset_server, mut game_clipboard, mut count): (
        ResMut<Download>,
        Res<AssetServer>,
        ResMut<GameClipboard>,
        ResMut<SyncCount>,
    ),
) {
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let (camera, camera_transform, cament) = camera.into_inner();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };
    let Ok(context) = rapier_context.single() else {
        return;
    };
    let hit = context.cast_ray(
        ray.origin,
        ray.direction.into(),
        f32::MAX,
        true,
        QueryFilter::only_dynamic(),
    );
    if let Some((entity, _)) = hit {
        if let Ok((mut pile, mut transform, children, is_rev, parent, inhand)) =
            cards.get_mut(entity)
        {
            if input.just_pressed(KeyCode::KeyF) {
                let is_reversed;
                if is_rev.is_some() {
                    is_reversed = false;
                    commands.entity(entity).remove::<Reversed>();
                } else {
                    is_reversed = true;
                    commands.entity(entity).insert(Reversed);
                }
                pile.0.reverse();
                transform.rotate_z(PI);
                let pile = mem::take(&mut pile.0);
                new_pile_at(
                    Pile(pile),
                    card_base.stock.clone_weak(),
                    &mut materials,
                    &mut commands,
                    &mut meshes,
                    card_base.back.clone_weak(),
                    card_base.side.clone_weak(),
                    *transform,
                    &mut rand,
                    false,
                    is_reversed,
                    None,
                    Some(&mut count),
                );
                commands.entity(entity).despawn();
            } else if input.just_pressed(KeyCode::KeyR) {
                pile.0.shuffle(&mut rand);
                let pile = mem::take(&mut pile.0);
                let reversed = is_rev.is_some();
                new_pile_at(
                    Pile(pile),
                    card_base.stock.clone_weak(),
                    &mut materials,
                    &mut commands,
                    &mut meshes,
                    card_base.back.clone_weak(),
                    card_base.side.clone_weak(),
                    *transform,
                    &mut rand,
                    false,
                    reversed,
                    None,
                    Some(&mut count),
                );
                commands.entity(entity).despawn();
            } else if input.just_pressed(KeyCode::Backspace)
                && input.all_pressed([KeyCode::ControlLeft, KeyCode::AltLeft])
            {
                commands.entity(entity).despawn();
            } else if input.just_pressed(KeyCode::KeyC)
                && input.all_pressed([KeyCode::ControlLeft, KeyCode::ShiftLeft])
            {
                game_clipboard.0 = Some(pile.clone());
            } else if mouse_input.just_pressed(MouseButton::Left) {
                if let Some(parent) = parent
                    && let Some(inhand) = inhand
                {
                    let mut hand = hands.get_mut(parent.0).unwrap().0;
                    hand.count -= 1;
                    hand.removed.push(inhand.0)
                }
                let reversed = is_rev.is_some();
                let len = pile.0.len() as f32;
                let new = pile.0.pop().unwrap();
                if !pile.0.is_empty() {
                    let pile = mem::take(&mut pile.0);
                    new_pile_at(
                        Pile(pile),
                        card_base.stock.clone_weak(),
                        &mut materials,
                        &mut commands,
                        &mut meshes,
                        card_base.back.clone_weak(),
                        card_base.side.clone_weak(),
                        *transform,
                        &mut rand,
                        false,
                        reversed,
                        None,
                        Some(&mut count),
                    );
                }
                commands.entity(entity).despawn();
                transform.translation.y += len + 4.0;
                new_pile_at(
                    Pile(vec![new]),
                    card_base.stock.clone_weak(),
                    &mut materials,
                    &mut commands,
                    &mut meshes,
                    card_base.back.clone_weak(),
                    card_base.side.clone_weak(),
                    *transform,
                    &mut rand,
                    true,
                    reversed,
                    None,
                    Some(&mut count),
                );
            } else if input.just_pressed(KeyCode::KeyE) {
                let (_, _, rot) = transform.rotation.to_euler(EulerRot::XYZ);
                let n = (2.0 * rot / PI).round() as isize;
                transform.rotation = Quat::from_rotation_z(match n {
                    0 => -PI / 2.0,
                    1 => 0.0,
                    2 | -2 => PI / 2.0,
                    -1 => PI,
                    _ => unreachable!(),
                });
                transform.rotate_x(-PI / 2.0);
            } else if input.just_pressed(KeyCode::KeyQ) {
                let (_, _, rot) = transform.rotation.to_euler(EulerRot::XYZ);
                let n = (2.0 * rot / PI).round() as isize;
                transform.rotation = Quat::from_rotation_z(match n {
                    0 => PI / 2.0,
                    1 => PI,
                    2 | -2 => -PI / 2.0,
                    -1 => 0.0,
                    _ => unreachable!(),
                });
                transform.rotate_x(-PI / 2.0);
            } else if input.just_pressed(KeyCode::KeyO)
                && input.all_pressed([KeyCode::ControlLeft, KeyCode::ShiftLeft])
            {
                let top = pile.0.last().unwrap();
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
                && is_rev.is_none()
                && zoom
                    .as_ref()
                    .map(|single| single.1.0 != entity.to_bits())
                    .unwrap_or(true)
            {
                let mut card = pile.0.pop().unwrap();
                if let Some(alt) = &mut card.alt {
                    mem::swap(&mut card.normal, alt);
                    mats.get_mut(*children.first().unwrap()).unwrap().0 =
                        make_material(&mut materials, card.normal.image.clone_weak());
                    card.is_alt = !card.is_alt;
                }
                pile.0.push(card)
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
                        if let Some(new) = pile.0.pop() {
                            let ent = new_pile_at(
                                Pile(vec![new]),
                                card_base.stock.clone_weak(),
                                &mut materials,
                                &mut commands,
                                &mut meshes,
                                card_base.back.clone_weak(),
                                card_base.side.clone_weak(),
                                Transform::default(),
                                &mut rand,
                                false,
                                false,
                                Some(hand.2),
                                Some(&mut count),
                            )
                            .unwrap();
                            commands.entity(ent).insert(InHand(hand.0.count));
                            hand.0.count += 1;
                        }
                    }
                    if !pile.0.is_empty() {
                        let reversed = is_rev.is_some();
                        let pile = mem::take(&mut pile.0);
                        new_pile_at(
                            Pile(pile),
                            card_base.stock.clone_weak(),
                            &mut materials,
                            &mut commands,
                            &mut meshes,
                            card_base.back.clone_weak(),
                            card_base.side.clone_weak(),
                            *transform,
                            &mut rand,
                            false,
                            reversed,
                            None,
                            Some(&mut count),
                        );
                    }
                    commands.entity(entity).despawn();
                }
            } else if input.just_pressed(KeyCode::KeyZ) {
                //TODO search
            }
            if input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) {
                if let Some(mut single) = zoom {
                    if single.1.0 != entity.to_bits() {
                        commands.entity(single.0).despawn();
                    } else if input.just_pressed(KeyCode::KeyO)
                        && let Some(alt) = &pile.0.last().unwrap().alt
                    {
                        single.2.0 = make_material(
                            &mut materials,
                            if single.1.1 {
                                &pile.0.last().unwrap().normal
                            } else {
                                alt
                            }
                            .image
                            .clone_weak(),
                        );
                        single.1.1 = !single.1.1;
                    }
                } else if is_rev.is_none() {
                    let card = pile.0.last().unwrap();
                    commands.entity(cament).with_child((
                        Mesh3d(card_base.stock.clone_weak()),
                        MeshMaterial3d(make_material(
                            &mut materials,
                            card.normal.image.clone_weak(),
                        )),
                        Transform::from_xyz(0.0, 0.0, -1024.0),
                        ZoomHold(entity.to_bits(), false),
                    ));
                }
            } else if let Some(single) = zoom {
                commands.entity(single.0).despawn();
            }
        } else {
            if let Some(single) = zoom {
                commands.entity(single.0).despawn();
            }
            if mouse_input.just_pressed(MouseButton::Left) {
                commands.entity(entity).insert(FollowMouse);
            } else if input.just_pressed(KeyCode::KeyR)
                && let Ok(mut v) = vels.get_mut(entity)
            {
                v.linvel.y = 2048.0;
                v.angvel.x = rand.random_range(-32.0..32.0);
                v.angvel.y = rand.random_range(-32.0..32.0);
                v.angvel.z = rand.random_range(-32.0..32.0);
            }
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
    if mouse_motion.delta.y != 0.0 {
        let translate = cam.forward().as_vec3() * scale * mouse_motion.delta.y * 16.0;
        if cam.translation.y < -translate.y {
            cam.translation.y /= 2.0;
        } else {
            cam.translation += translate;
        }
    }
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
    #[cfg(feature = "wasm")] clipboard: Res<Clipboard>,
    #[cfg(not(feature = "wasm"))] mut clipboard: ResMut<Clipboard>,
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
                }
            };
            #[cfg(not(feature = "wasm"))]
            down.runtime.0.spawn(f);
            #[cfg(feature = "wasm")]
            wasm_bindgen_futures::spawn_local(f);
        } else if let Some(pile) = &game_clipboard.0 {
            new_pile(
                pile.clone(),
                card_base.stock.clone_weak(),
                &mut materials,
                &mut commands,
                &mut meshes,
                card_base.back.clone_weak(),
                card_base.side.clone_weak(),
                &mut rand,
                v,
                &mut count,
            );
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
) {
    let mut decks = decks.get_deck.0.lock().unwrap();
    for (deck, v) in decks.drain(..) {
        info!("deck found of size {} at {} {}", deck.0.len(), v.x, v.y);
        new_pile(
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
        );
    }
}
