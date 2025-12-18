use crate::counters::Value;
use crate::download::{
    Exact, get_alts, get_deck, get_deck_export, spawn_scryfall_list, spawn_singleton,
    spawn_singleton_id,
};
use crate::misc::{
    Equipment, adjust_meshes, default_cam_pos, is_reversed, move_up, new_pile, new_pile_at,
    repaint_face, spawn_equip, vec2_to_ground,
};
use crate::setup::{EscMenu, FontRes, MAT_WIDTH, Player, SideMenu, T, W, Wall};
use crate::sync::{CameraInd, CursorInd, InOtherHand, Net, SyncObjectMe, Trans};
use crate::*;
use avian3d::math::Vector;
use bevy::input::mouse::{
    AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit, MouseWheel,
};
use bevy::input_focus::InputFocus;
use bevy::picking::hover::HoverMap;
use bevy::window::PrimaryWindow;
use bevy_rich_text3d::Text3d;
use bevy_tangled::PeerId;
use bevy_ui_text_input::{TextInputBuffer, TextInputContents, TextInputMode, TextInputNode};
use cosmic_text::Edit;
#[cfg(feature = "calc")]
use kalc_lib::complex::NumStr;
#[cfg(feature = "calc")]
use kalc_lib::units::{Number, Options, Variable};
use rand::Rng;
use std::f32::consts::PI;
use std::mem;
#[derive(Component)]
pub struct HandIgnore;
pub fn gather_hand(
    mut hand: Single<(&Transform, &mut Hand, Entity, Option<&Children>)>,
    mut cards: Query<
        (
            Entity,
            &mut GravityScale,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &Pile,
            Option<&SyncObject>,
            Option<&HandIgnore>,
            &mut Transform,
            Option<&FollowMouse>,
        ),
        (
            With<Pile>,
            Without<Hand>,
            Without<InHand>,
            Without<FollowOtherMouse>,
        ),
    >,
    mut child: Query<&mut InHand>,
    spatial: SpatialQuery,
    mut commands: Commands,
    mut net: Net,
) {
    let intersections = spatial.shape_intersections(
        &Collider::cuboid(MAT_WIDTH, CARD_HEIGHT, CARD_HEIGHT),
        hand.0.translation,
        hand.0.rotation,
        &SpatialQueryFilter::DEFAULT,
    );
    for ent in intersections {
        if let Ok((entity, mut grav, mut linvel, mut angvel, pile, obj, ign, mut trans, fm)) =
            cards.get_mut(ent)
            && pile.len() == 1
        {
            if ign.is_some() {
                commands.entity(entity).remove::<HandIgnore>();
            } else {
                if let Some(n) = obj {
                    net.take(entity, *n);
                }
                linvel.0 = default();
                angvel.0 = default();
                grav.0 = 0.0;
                let entry = place_pos(&mut hand, trans.translation.x, &mut child);
                commands
                    .entity(entity)
                    .insert(InHand(entry))
                    .insert(RigidBodyDisabled);
                hand.1.count += 1;
                if fm.is_none() {
                    commands.entity(hand.2).add_child(entity);
                }
                trans.rotation = Quat::default();
            }
        }
    }
}
fn place_pos(
    hand: &mut Single<(&Transform, &mut Hand, Entity, Option<&Children>)>,
    x: f32,
    child: &mut Query<&mut InHand>,
) -> usize {
    let entry =
        ((2.0 * (x - hand.0.translation.x) / CARD_WIDTH + (hand.1.count as f32 - 1.0) / 2.0).ceil()
            as usize)
            .min(hand.1.count);
    if entry != hand.1.count
        && let Some(children) = hand.3
    {
        for c in children {
            if let Ok(mut e) = child.get_mut(*c)
                && entry <= e.0
            {
                e.0 += 1;
            }
        }
    }
    entry
}
fn swap_pos(
    hand: &mut Single<(&Transform, &mut Hand, Entity, Option<&Children>)>,
    x: f32,
    child: &mut Query<&mut InHand>,
    cur: usize,
) -> usize {
    let entry =
        ((2.0 * (x - hand.0.translation.x) / CARD_WIDTH + (hand.1.count as f32 - 2.0) / 2.0).ceil()
            as usize)
            .min(hand.1.count - 1);
    if cur != entry
        && let Some(children) = hand.3
    {
        for c in children {
            if let Ok(mut e) = child.get_mut(*c)
                && e.0 == entry
            {
                e.0 = cur;
                break;
            }
        }
    }
    entry
}
pub fn update_hand(
    mut hand: Single<(&Transform, &mut Hand, Option<&Children>)>,
    mut card: Query<
        (&mut InHand, &mut Transform, &Pile),
        (With<InHand>, Without<Hand>, Without<FollowMouse>),
    >,
    input: Res<ButtonInput<KeyCode>>,
) {
    if let Some(children) = hand.2 {
        for child in children.iter() {
            let Ok((mut entry, _, _)) = card.get_mut(child) else {
                continue;
            };
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
        }
        if input.just_pressed(KeyCode::KeyS)
            && input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
            && !input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
        {
            let mut order = children
                .iter()
                .filter_map(|c| card.get(c).ok())
                .map(|(e, _, c)| (c.get(0).unwrap().data.face.mana_cost.total, e.0))
                .collect::<Vec<(u8, usize)>>();
            order.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
            for child in children.iter() {
                let Ok((mut entry, _, _)) = card.get_mut(child) else {
                    continue;
                };
                let pos = order.iter().position(|(_, a)| *a == entry.0).unwrap();
                entry.0 = pos;
            }
        }
        for child in children.iter() {
            let Ok((entry, mut transform, _)) = card.get_mut(child) else {
                continue;
            };
            let idx = entry.0 as f32 - hand.1.count as f32 / 2.0;
            transform.translation = Vec3::new(
                (idx + 0.5) * CARD_WIDTH / 2.0,
                idx * CARD_THICKNESS / 2.0,
                0.0,
            );
        }
    }
    hand.1.removed.clear();
}
pub fn follow_mouse(
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window, With<PrimaryWindow>>,
    cards: Query<(&Collider, &Transform), (Without<FollowMouse>, Without<Hand>)>,
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
            &GlobalTransform,
            Option<&ChildOf>,
        ),
        (With<FollowMouse>, Without<Hand>),
    >,
    menu: Res<Menu>,
    mut child: Query<&mut InHand>,
    mut hand: Single<(&Transform, &mut Hand, Entity, Option<&Children>)>,
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
            card.1.translation.y = max + CARD_THICKNESS * 4.0;
        }
        if let Some(time) = ray.intersect_plane(
            if card.6.is_some() {
                card.5.translation()
            } else {
                card.1.translation
            },
            InfinitePlane3d { normal: Dir3::Y },
        ) {
            let mut point = ray.get_point(time);
            point.x = point.x.clamp(
                -W + (aabb.min.x - card.1.translation.x).abs(),
                W - (aabb.max.x - card.1.translation.x).abs(),
            );
            point.z = point.z.clamp(
                -W + (aabb.min.z - card.1.translation.z).abs(),
                W - (aabb.max.z - card.1.translation.z).abs(),
            );
            if child.contains(card.0) {
                if Collider::cuboid(
                    MAT_WIDTH + CARD_THICKNESS,
                    CARD_HEIGHT + CARD_THICKNESS,
                    CARD_HEIGHT + CARD_THICKNESS,
                )
                .aabb(hand.0.translation, hand.0.rotation)
                .intersects(&card.4.aabb(card.5.translation(), card.1.rotation))
                {
                    let cur = child.get(card.0).unwrap().0;
                    let n = swap_pos(&mut hand, point.x, &mut child, cur);
                    child.get_mut(card.0).unwrap().0 = n;
                    if card.6.is_some() {
                        commands.entity(card.0).remove_parent_in_place();
                    }
                    commands.entity(card.0).insert(RigidBodyDisabled);
                    point.y = CARD_HEIGHT * 3.0 / 4.0;
                } else {
                    hand.1.count -= 1;
                    hand.1.removed.push(child.get(card.0).unwrap().0);
                    commands
                        .entity(card.0)
                        .remove_parent_in_place()
                        .remove::<RigidBodyDisabled>()
                        .remove::<InHand>();
                }
            }
            card.1.translation = point;
        }
    } else if let Some(time) =
        ray.intersect_plane(card.5.translation(), InfinitePlane3d { normal: Dir3::Y })
        && Collider::cuboid(
            MAT_WIDTH + CARD_THICKNESS,
            CARD_HEIGHT + CARD_THICKNESS,
            CARD_HEIGHT + CARD_THICKNESS,
        )
        .aabb(hand.0.translation, hand.0.rotation)
        .intersects(&card.4.aabb(ray.get_point(time), card.1.rotation))
    {
        commands.entity(card.0).remove::<FollowMouse>();
        commands.entity(hand.2).add_child(card.0);
    } else {
        if let Some(time) =
            ray.intersect_plane(card.5.translation(), InfinitePlane3d { normal: Dir3::Y })
        {
            let mut point = ray.get_point(time);
            point.x = point.x.clamp(-W, W);
            point.z = point.z.clamp(-W, W);
            card.3.0 = (point - card.5.translation()) / time_since.delta_secs();
        }
        commands
            .entity(card.0)
            .remove::<FollowMouse>()
            .remove::<RigidBodyDisabled>()
            .remove::<SleepingDisabled>();
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
    mut cards: Query<(
        &mut Pile,
        &Children,
        Option<&ChildOf>,
        Option<&InHand>,
        Option<&InOtherHand>,
    )>,
    mut mats: Query<&mut MeshMaterial3d<StandardMaterial>>,
    mut hand: Single<(&mut Hand, Entity)>,
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
        Option<Single<(Entity, &mut ZoomHold, &mut ImageNode, &mut UiTransform)>>,
        ResMut<Download>,
        Res<AssetServer>,
        ResMut<GameClipboard>,
        Query<&SyncObjectMe>,
        Query<&SyncObject>,
        Query<
            (&mut Mesh3d, &mut Transform),
            (
                Without<Children>,
                With<ChildOf>,
                Without<InHand>,
                Without<Shape>,
                Without<Pile>,
            ),
        >,
        Option<Single<Entity, With<FollowMouse>>>,
        Query<(&mut Shape, Entity)>,
        ResMut<Menu>,
        ResMut<InputFocus>,
        Option<Single<Entity, With<SideMenu>>>,
        Option<Single<(Entity, &SearchDeck)>>,
    ),
    (
        text,
        font,
        mut text3d,
        children,
        mut transforms,
        hover_map,
        equipment,
        mut net,
        mut turn,
        peers,
    ): (
        Option<Single<&TextInputContents>>,
        Res<FontRes>,
        Query<&mut Text3d>,
        Query<&Children, Without<Pile>>,
        Query<&mut Transform, Or<(With<Pile>, With<Shape>)>>,
        Res<HoverMap>,
        Query<(), With<Equipment>>,
        Net,
        ResMut<Turn>,
        Res<Peers>,
    ),
) {
    if matches!(*menu, Menu::Esc)
        || (matches!(*menu, Menu::Side | Menu::Counter) && active_input.get().is_some())
    {
        if let Some(single) = zoom {
            commands.entity(single.0).despawn();
        }
        return;
    }
    let Some(cursor_position) = window.cursor_position() else {
        if let Some(single) = zoom {
            commands.entity(single.0).despawn();
        }
        return;
    };
    let (camera, camera_transform) = camera.into_inner();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        if let Some(single) = zoom {
            commands.entity(single.0).despawn();
        }
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
        let Ok(mut transform) = transforms.get_mut(entity) else {
            if let Some(single) = zoom {
                commands.entity(single.0).despawn();
            }
            return;
        };
        if let Ok((mut pile, children, parent, inhand, inother)) = cards.get_mut(entity) {
            if input.just_pressed(KeyCode::KeyF) {
                if let Ok(id) = others_ids.get(entity) {
                    net.take(entity, *id);
                }
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
                        &side,
                        &mut menu,
                    );
                }
            } else if input.just_pressed(KeyCode::KeyR) {
                if pile.len() > 1 {
                    pile.shuffle(&mut net.rand);
                    let card = pile.last();
                    repaint_face(&mut mats, &mut materials, card, children);
                    if let Ok(id) = ids.get(entity) {
                        net.reorder_me(*id, pile.iter().map(|a| a.data.id).collect());
                    } else if let Ok(id) = others_ids.get(entity) {
                        net.reorder(*id, pile.iter().map(|a| a.data.id).collect());
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
                            &side,
                            &mut menu,
                        );
                    }
                }
            } else if ((input.just_pressed(KeyCode::Backspace)
                || (input.pressed(KeyCode::Backspace)
                    && input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])))
                && input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
                && input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]))
                || input.just_pressed(KeyCode::Delete)
            {
                if let Ok(id) = ids.get(entity) {
                    net.killed_me(*id)
                } else if let Ok(id) = others_ids.get(entity) {
                    net.killed(*id);
                }
                if let Some(inhand) = inhand {
                    hand.0.removed.push(inhand.0);
                    hand.0.count -= 1;
                }
                commands.entity(entity).despawn();
                if search_deck.is_some_and(|s| s.1.0 == entity) {
                    *menu = Menu::World;
                    commands.entity(**side.as_ref().unwrap()).despawn();
                }
            } else if input.just_pressed(KeyCode::KeyC)
                && input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
            {
                if input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
                    *game_clipboard = GameClipboard::Pile(pile.clone());
                } else if !is_reversed(&transform) {
                    let card = pile.get_card(&transform);
                    let text = format!("https://scryfall.com/card/{}", card.data.id);
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
                if matches!(*menu, Menu::Side | Menu::Counter)
                    && hover_map
                        .values()
                        .all(|a| a.keys().all(|e| e.to_bits() != u32::MAX as u64))
                {
                    return;
                }
                if inother.is_some() {
                    let mut ent = commands.entity(entity);
                    ent.remove::<InOtherHand>();
                    ent.remove::<SleepingDisabled>();
                    repaint_face(&mut mats, &mut materials, pile.first(), children);
                    colliders.get_mut(entity).unwrap().1.0 = GRAVITY;
                }
                if inhand.is_some() {
                    transform.translation.y += 128.0 * CARD_THICKNESS;
                } else {
                    transform.translation.y += 8.0 * CARD_THICKNESS;
                }
                if input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
                    && pile.len() > 1
                {
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
                            &equipment,
                            &mut commands,
                        );
                    }
                    let mut transform = *transform;
                    transform.translation.y += len + CARD_THICKNESS * 4.0;
                    if let Some(e) = follow {
                        commands.entity(*e).remove::<FollowMouse>();
                    }
                    let id = net.new_id();
                    new_pile_at(
                        Pile::Single(new.into()),
                        card_base.clone(),
                        &mut materials,
                        &mut commands,
                        &mut meshes,
                        transform,
                        true,
                        None,
                        None,
                        Some(id),
                    );
                    if let Ok(lid) = ids.get(entity) {
                        net.draw_me(
                            *lid,
                            vec![(id, Trans::from_transform(&transform))],
                            draw_len,
                        );
                    } else if let Ok(oid) = others_ids.get(entity) {
                        net.draw(
                            *oid,
                            vec![(id, Trans::from_transform(&transform))],
                            draw_len,
                        );
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
                            &side,
                            &mut menu,
                        );
                    }
                } else {
                    if let Some(e) = follow {
                        commands.entity(*e).remove::<FollowMouse>();
                    }
                    if let Ok(id) = others_ids.get(entity) {
                        net.take(entity, *id);
                    }
                    colliders.get_mut(entity).unwrap().1.0 = 0.0;
                    commands
                        .entity(entity)
                        .insert(FollowMouse)
                        .insert(SleepingDisabled)
                        .remove::<InOtherHand>()
                        .remove::<FollowOtherMouse>()
                        .remove::<RigidBodyDisabled>()
                        .remove_parent_in_place();
                }
            } else if input.just_pressed(KeyCode::KeyE) {
                if input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
                    if !is_reversed(&transform) {
                        let b = pile.equip();
                        if let Ok(id) = ids.get(entity) {
                            net.equip_me(*id)
                        } else if let Ok(id) = others_ids.get(entity) {
                            net.equip(*id);
                        }
                        repaint_face(&mut mats, &mut materials, pile.last(), children);
                        adjust_meshes(
                            &pile,
                            children,
                            &mut meshes,
                            &mut query_meshes,
                            &mut transform,
                            &mut colliders.get_mut(entity).unwrap().0,
                            &equipment,
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
                } else if zoom.is_none() {
                    if let Ok(id) = others_ids.get(entity) {
                        net.take(entity, *id);
                    }
                    rotate_right(&mut transform);
                }
            } else if input.just_pressed(KeyCode::KeyS)
                && input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
                && input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
                && pile.len() > 1
            {
                let mut start = *transform;
                start.translation.y -= pile.len() as f32 * CARD_THICKNESS;
                let mut transform = start;
                let mut vec = Vec::with_capacity(pile.len());
                for c in pile.drain(..) {
                    let id = net.new_id();
                    new_pile_at(
                        Pile::Single(c.into()),
                        card_base.clone(),
                        &mut materials,
                        &mut commands,
                        &mut meshes,
                        transform,
                        false,
                        None,
                        None,
                        Some(id),
                    );
                    transform.translation.x += CARD_WIDTH + CARD_THICKNESS * 2.0;
                    if transform.translation.x >= W - T - CARD_WIDTH - CARD_THICKNESS * 2.0 {
                        transform.translation.x = start.translation.x;
                        transform.translation.z += CARD_HEIGHT + CARD_THICKNESS * 2.0;
                    }
                    vec.push((id, Trans::from_transform(&transform)));
                }
                if let Ok(lid) = ids.get(entity) {
                    let len = vec.len();
                    net.draw_me(*lid, vec, len);
                } else if let Ok(id) = others_ids.get(entity) {
                    let len = vec.len();
                    net.draw(*id, vec, len);
                }
                if let Ok(id) = ids.get(entity) {
                    net.killed_me(*id)
                } else if let Ok(id) = others_ids.get(entity) {
                    net.killed(*id);
                }
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
                        &side,
                        &mut menu,
                    );
                }
            } else if input.just_pressed(KeyCode::KeyQ) && zoom.is_none() {
                if let Ok(id) = others_ids.get(entity) {
                    net.take(entity, *id);
                }
                rotate_left(&mut transform);
            } else if input.just_pressed(KeyCode::KeyO)
                && input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
                && input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
                && !is_reversed(&transform)
            {
                let top = pile.get_card(&transform);
                let v = Vec2::new(
                    transform.translation.x,
                    transform.translation.z - CARD_HEIGHT - CARD_THICKNESS,
                );
                let client = down.client.0.clone();
                let get_deck = down.get_deck.clone();
                let asset_server = asset_server.clone();
                let id = top.data.id;
                info!("{}: {} has requested printings", top.face().name, id);
                #[cfg(not(feature = "wasm"))]
                down.runtime.0.spawn(async move {
                    let sid = id.to_string();
                    get_alts(&sid, client, asset_server, get_deck, v).await
                });
                #[cfg(feature = "wasm")]
                wasm_bindgen_futures::spawn_local(async move {
                    let sid = id.to_string();
                    get_alts(&sid, client, asset_server, get_deck, v).await;
                })
            } else if input.just_pressed(KeyCode::KeyT)
                && input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
                && input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
                && !is_reversed(&transform)
            {
                let top = pile.get_card(&transform);
                let v = Vec2::new(
                    transform.translation.x,
                    transform.translation.z - CARD_HEIGHT - CARD_THICKNESS,
                );
                let client = down.client.0.clone();
                let get_deck = down.get_deck.clone();
                let asset_server = asset_server.clone();
                let ids = top.data.tokens.clone();
                info!(
                    "{}: {} has requested tokens {ids:?}",
                    top.face().name,
                    top.data.id
                );
                #[cfg(not(feature = "wasm"))]
                down.runtime.0.spawn(async move {
                    spawn_scryfall_list(ids, client, asset_server, get_deck, v).await
                });
                #[cfg(feature = "wasm")]
                wasm_bindgen_futures::spawn_local(async move {
                    spawn_scryfall_list(ids, client, asset_server, get_deck, v).await;
                })
            } else if input.just_pressed(KeyCode::KeyO)
                && !is_reversed(&transform)
                && zoom
                    .as_ref()
                    .map(|single| single.1.0 != entity.to_bits())
                    .unwrap_or(true)
            {
                let card = pile.get_mut_card(&transform);
                if card.data.back.is_some() {
                    card.flipped = !card.flipped;
                    repaint_face(&mut mats, &mut materials, card, children);
                }
                let idx = if is_reversed(&transform) {
                    0
                } else {
                    pile.len() - 1
                };
                if let Ok(id) = ids.get(entity) {
                    net.flip_me(*id, idx);
                } else if let Ok(id) = others_ids.get(entity) {
                    net.flip(*id, idx);
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
                if parent.is_none() && inother.is_none() {
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
                    let mut vec = Vec::new();
                    let len = if is_reversed(&transform) {
                        n
                    } else {
                        pile.len()
                    };
                    for _ in 0..n {
                        let new = pile.take_card(&transform);
                        let id = net.new_id();
                        let mut ent = new_pile_at(
                            Pile::Single(new.into()),
                            card_base.clone(),
                            &mut materials,
                            &mut commands,
                            &mut meshes,
                            Transform::default(),
                            false,
                            Some(hand.1),
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
                            &side,
                            &mut menu,
                        );
                    }
                    if let Ok(lid) = ids.get(entity) {
                        if !is_reversed(&transform) {
                            vec.reverse();
                        }
                        net.draw_me(*lid, vec, len);
                    } else if let Ok(id) = others_ids.get(entity) {
                        if !is_reversed(&transform) {
                            vec.reverse();
                        }
                        net.draw(*id, vec, len);
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
                            &equipment,
                            &mut commands,
                        );
                    } else {
                        if let Ok(id) = ids.get(entity) {
                            net.killed_me(*id);
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
            if input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) && inother.is_none() {
                let mut spawn = || {
                    let card = pile.get_card(&transform);
                    let mut transform = UiTransform::default();
                    if matches!(card.data.layout, Layout::Room) {
                        ui_rotate_right(&mut transform)
                    }
                    commands
                        .spawn((
                            Node {
                                width: Val::Px(IMAGE_WIDTH),
                                height: Val::Px(IMAGE_HEIGHT),
                                ..default()
                            },
                            transform,
                            ZoomHold(entity.to_bits(), false),
                            card.image_node(),
                        ))
                        .with_children(|parent| {
                            if pile.is_equiped() {
                                for (i, c) in pile.iter_equipment().rev().enumerate() {
                                    let top = i.is_multiple_of(2);
                                    parent.spawn((
                                        Node {
                                            width: Val::Px(IMAGE_WIDTH * EQUIP_SCALE),
                                            height: Val::Px(IMAGE_HEIGHT * EQUIP_SCALE),
                                            position_type: PositionType::Absolute,
                                            left: Val::Px(
                                                (EQUIP_SCALE * ((i & !1) + 1) as f32 + 1.5)
                                                    * IMAGE_WIDTH
                                                    / 2.0,
                                            ),
                                            top: Val::Px(if top {
                                                0.0
                                            } else {
                                                IMAGE_HEIGHT * EQUIP_SCALE
                                            }),
                                            ..default()
                                        },
                                        c.image_node(),
                                    ));
                                }
                            }
                        });
                };
                if let Some(mut single) = zoom {
                    if single.1.0 != entity.to_bits() {
                        if !is_reversed(&transform) {
                            spawn();
                        }
                        commands.entity(single.0).despawn();
                    } else if input.just_pressed(KeyCode::KeyO) {
                        let card = pile.get_mut_card(&transform);
                        if card.back().is_some() {
                            single.1.1 = !single.1.1;
                            if single.1.1 {
                                card.flipped = !card.flipped;
                                *single.2 = card.image_node();
                                card.flipped = !card.flipped;
                            } else {
                                *single.2 = card.image_node();
                            }
                        }
                    } else if input.just_pressed(KeyCode::KeyE) {
                        ui_rotate_right(&mut single.3);
                    } else if input.just_pressed(KeyCode::KeyQ) {
                        ui_rotate_left(&mut single.3);
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
                && let Ok((s, _)) = shape.get_mut(entity)
                && let Shape::Counter(v, _) = s.into_inner()
            {
                v.0 -= 1;
                for ent in children.get(entity).unwrap() {
                    let mut text = text3d.get_mut(*ent).unwrap();
                    *text.get_single_mut().unwrap() = v.0.to_string();
                }
                if let Ok(id) = ids.get(entity) {
                    net.counter_me(*id, v.clone());
                } else if let Ok(id) = others_ids.get(entity) {
                    net.counter(*id, v.clone());
                }
            } else if mouse_input.just_pressed(MouseButton::Left) {
                if !input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
                    && let Ok((s, _)) = shape.get_mut(entity)
                    && let Shape::Counter(v, _) = s.into_inner()
                {
                    v.0 += 1;
                    for ent in children.get(entity).unwrap() {
                        let mut text = text3d.get_mut(*ent).unwrap();
                        *text.get_single_mut().unwrap() = v.0.to_string();
                    }
                    if let Ok(id) = ids.get(entity) {
                        net.counter_me(*id, v.clone());
                    } else if let Ok(id) = others_ids.get(entity) {
                        net.counter(*id, v.clone());
                    }
                } else {
                    if let Some(e) = follow {
                        commands.entity(*e).remove::<FollowMouse>();
                    }
                    if let Ok(id) = others_ids.get(entity) {
                        net.take(entity, *id);
                    }
                    phys.0 = 0.0;
                    commands
                        .entity(entity)
                        .insert(FollowMouse)
                        .insert(SleepingDisabled)
                        .remove::<FollowOtherMouse>();
                }
            } else if input.just_pressed(KeyCode::KeyC)
                && input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
                && input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
                && let Ok((shape, _)) = shape.get(entity)
            {
                *game_clipboard = GameClipboard::Shape(shape.clone());
            } else if input.just_pressed(KeyCode::KeyR)
                && input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
                && let Ok((s, _)) = shape.get(entity)
                && let Shape::Counter(v, _) = s
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
            } else if (input.just_pressed(KeyCode::Backspace)
                || (input.pressed(KeyCode::Backspace)
                    && input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])))
                && input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
                && input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
            {
                if let Ok(id) = ids.get(entity) {
                    net.killed_me(*id)
                } else if let Ok(id) = others_ids.get(entity) {
                    net.killed(*id);
                }
                commands.entity(entity).despawn();
            } else if input.just_pressed(KeyCode::KeyF) {
                if let Ok(id) = others_ids.get(entity) {
                    net.take(entity, *id);
                }
                let s = shape.get(entity);
                match s.map(|(a, _)| a.clone()) {
                    Ok(Shape::Turn(n)) => {
                        let mut flip = true;
                        let mut up = false;
                        let peers = peers.map();
                        if peers.len() <= 1
                            || shape.iter().all(|(s, e)| {
                                if let Shape::Counter(_, _) = s {
                                    is_reversed(transforms.get(e).unwrap())
                                } else {
                                    true
                                }
                            })
                        {
                            flip = false
                        } else if n == turn.0 {
                            next_turn(
                                others_ids,
                                &mut shape,
                                &mut transforms,
                                &mut net,
                                &mut turn,
                                entity,
                                &mut flip,
                                &peers,
                            );
                        } else if peers.iter().any(|(_, b)| *b == n)
                            && shape
                                .iter()
                                .find_map(|(s, e)| {
                                    if let Shape::Counter(_, v) = s
                                        && *v == n
                                    {
                                        Some(e)
                                    } else {
                                        None
                                    }
                                })
                                .map(|ent| !is_reversed(transforms.get(ent).unwrap()))
                                .unwrap_or(false)
                        {
                            to_turn(
                                others_ids,
                                &mut shape,
                                &mut transforms,
                                &mut net,
                                &mut turn,
                                n,
                            );
                            up = true
                        } else {
                            flip = false;
                        }
                        if flip {
                            let mut transform = transforms.get_mut(entity).unwrap();
                            transform.rotation = Quat::default();
                            *transform = transform
                                .looking_to(Dir3::Z, if up { Dir3::Y } else { Dir3::NEG_Y });
                        }
                    }
                    Ok(Shape::Counter(_, n)) if n == turn.0 && !is_reversed(&transform) => {
                        let mut flip = true;
                        next_turn(
                            others_ids,
                            &mut shape,
                            &mut transforms,
                            &mut net,
                            &mut turn,
                            entity,
                            &mut flip,
                            &peers.map(),
                        );
                        let mut transform = transforms.get_mut(entity).unwrap();
                        transform.rotate_local_z(PI);
                        if flip
                            && let Some(entity) = shape.iter().find_map(|(s, e)| {
                                if let Shape::Turn(v) = s
                                    && *v == n
                                {
                                    Some(e)
                                } else {
                                    None
                                }
                            })
                        {
                            let mut transform = transforms.get_mut(entity).unwrap();
                            transform.rotation = Quat::default();
                            *transform = transform.looking_to(Dir3::Z, Dir3::NEG_Y);
                        }
                    }
                    _ => {
                        let mut transform = transforms.get_mut(entity).unwrap();
                        transform.rotate_local_z(PI);
                    }
                }
            } else if (input.just_pressed(KeyCode::KeyR)
                || (input.pressed(KeyCode::KeyR)
                    && input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])))
                && let Ok((mut lv, mut av)) = vels.get_mut(entity)
            {
                commands.entity(entity).insert(TempDisable);
                if layers.filters & 0b01 == 0b01 {
                    layers.filters = (layers.filters.0 - 0b01).into();
                }
                if let Ok(id) = others_ids.get(entity) {
                    net.take(entity, *id);
                }
                lv.y = MAT_WIDTH;
                av.x = if net.rand.random() { 1.0 } else { -1.0 }
                    * (net.rand.random_range(32.0..64.0) + av.x.abs());
                av.y = if net.rand.random() { 1.0 } else { -1.0 }
                    * (net.rand.random_range(32.0..64.0) + av.y.abs());
                av.z = if net.rand.random() { 1.0 } else { -1.0 }
                    * (net.rand.random_range(32.0..64.0) + av.z.abs());
            } else if input.just_pressed(KeyCode::KeyE) {
                if let Ok(id) = others_ids.get(entity) {
                    net.take(entity, *id);
                }
                rotate_right(&mut transform)
            } else if input.just_pressed(KeyCode::KeyQ) {
                if let Ok(id) = others_ids.get(entity) {
                    net.take(entity, *id);
                }
                rotate_left(&mut transform)
            }
        } else if let Some(single) = zoom {
            commands.entity(single.0).despawn();
        }
    } else if let Some(single) = zoom {
        commands.entity(single.0).despawn();
    }
}
pub fn turn_keybinds(
    others_ids: Query<&SyncObject>,
    mut shape: Query<(&mut Shape, Entity)>,
    mut transforms: Query<&mut Transform, Or<(With<Pile>, With<Shape>)>>,
    mut net: Net,
    mut turn: ResMut<Turn>,
    peers: Res<Peers>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::KeyX) {
        let mut flip = true;
        let mut up = false;
        let map = peers.map();
        if map.len() <= 1
            || shape.iter().all(|(s, e)| {
                if let Shape::Counter(_, _) = s {
                    is_reversed(transforms.get(e).unwrap())
                } else {
                    true
                }
            })
        {
            return;
        }
        let Some(me) = peers.me else { return };
        let Some(entity) = shape.iter().find_map(|(s, e)| {
            if let Shape::Turn(v) = s
                && *v == me
            {
                Some(e)
            } else {
                None
            }
        }) else {
            return;
        };
        if input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            if me == turn.0 {
                return;
            }
            to_turn(
                others_ids,
                &mut shape,
                &mut transforms,
                &mut net,
                &mut turn,
                me,
            );
            up = true
        } else {
            if me != turn.0
                || shape
                    .iter()
                    .find_map(|(s, e)| {
                        if let Shape::Counter(_, v) = s
                            && *v == me
                        {
                            Some(e)
                        } else {
                            None
                        }
                    })
                    .map(|ent| is_reversed(transforms.get(ent).unwrap()))
                    .unwrap_or(true)
            {
                return;
            }
            next_turn(
                others_ids,
                &mut shape,
                &mut transforms,
                &mut net,
                &mut turn,
                entity,
                &mut flip,
                &map,
            );
        }
        if flip {
            if let Ok(id) = others_ids.get(entity) {
                net.take(entity, *id);
            }
            let mut transform = transforms.get_mut(entity).unwrap();
            transform.rotation = Quat::default();
            *transform = transform.looking_to(Dir3::Z, if up { Dir3::Y } else { Dir3::NEG_Y });
        }
    }
}
fn to_turn(
    others_ids: Query<&SyncObject>,
    shape: &mut Query<(&mut Shape, Entity)>,
    transforms: &mut Query<&mut Transform, Or<(With<Pile>, With<Shape>)>>,
    net: &mut Net,
    turn: &mut ResMut<Turn>,
    n: usize,
) {
    let last = shape
        .iter()
        .find_map(|(s, e)| {
            if Shape::Turn(turn.0) == *s {
                Some(e)
            } else {
                None
            }
        })
        .unwrap();
    if let Ok(id) = others_ids.get(last) {
        net.take(last, *id);
    }
    let mut transform = transforms.get_mut(last).unwrap();
    transform.rotation = Quat::default();
    *transform = transform.looking_to(Dir3::Z, Dir3::NEG_Y);
    turn.0 = n;
    net.turn(turn.0);
}
fn next_turn(
    others_ids: Query<&SyncObject>,
    shape: &mut Query<(&mut Shape, Entity)>,
    transforms: &mut Query<&mut Transform, Or<(With<Pile>, With<Shape>)>>,
    net: &mut Net,
    turn: &mut ResMut<Turn>,
    entity: Entity,
    flip: &mut bool,
    peers: &MutexGuard<HashMap<PeerId, usize>>,
) {
    let next = |n: usize| -> usize {
        match n {
            0 => 2,
            2 => 3,
            3 => 1,
            1 => 0,
            _ => unreachable!(),
        }
    };
    turn.0 = next(turn.0);
    while peers.iter().all(|(_, b)| *b != turn.0)
        || shape
            .iter()
            .find_map(|(s, e)| {
                if let Shape::Counter(_, v) = s
                    && *v == turn.0
                {
                    Some(e)
                } else {
                    None
                }
            })
            .map(|ent| is_reversed(transforms.get(ent).unwrap()))
            .unwrap_or(true)
    {
        turn.0 = next(turn.0);
    }
    let last = shape
        .iter()
        .find_map(|(s, e)| {
            if Shape::Turn(turn.0) == *s {
                Some(e)
            } else {
                None
            }
        })
        .unwrap();
    if last == entity {
        *flip = false
    } else {
        if let Ok(id) = others_ids.get(last) {
            net.take(last, *id);
        }
        let mut transform = transforms.get_mut(last).unwrap();
        transform.rotation = Quat::default();
        net.turn(turn.0);
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
fn ui_rotate_right(transform: &mut UiTransform) {
    transform.rotation = match transform.rotation.sin_cos() {
        (0.0, 1.0) => Rot2::from_sin_cos(1.0, 0.0),
        (1.0, 0.0) => Rot2::from_sin_cos(0.0, -1.0),
        (0.0, -1.0) => Rot2::from_sin_cos(-1.0, 0.0),
        (-1.0, 0.0) => Rot2::from_sin_cos(0.0, 1.0),
        _ => unreachable!(),
    };
    transform.translation = if matches!(transform.rotation.sin_cos(), (1.0, 0.0) | (-1.0, 0.0)) {
        Val2::px(
            (IMAGE_HEIGHT - IMAGE_WIDTH) / 2.0,
            (IMAGE_WIDTH - IMAGE_HEIGHT) / 2.0,
        )
    } else {
        Val2::px(0.0, 0.0)
    };
}
fn ui_rotate_left(transform: &mut UiTransform) {
    transform.rotation = match transform.rotation.sin_cos() {
        (0.0, 1.0) => Rot2::from_sin_cos(-1.0, 0.0),
        (-1.0, 0.0) => Rot2::from_sin_cos(0.0, -1.0),
        (0.0, -1.0) => Rot2::from_sin_cos(1.0, 0.0),
        (1.0, 0.0) => Rot2::from_sin_cos(0.0, 1.0),
        _ => unreachable!(),
    };
    transform.translation = if matches!(transform.rotation.sin_cos(), (1.0, 0.0) | (-1.0, 0.0)) {
        Val2::px(
            (IMAGE_HEIGHT - IMAGE_WIDTH) / 2.0,
            (IMAGE_WIDTH - IMAGE_HEIGHT) / 2.0,
        )
    } else {
        Val2::px(0.0, 0.0)
    };
}
#[derive(Component)]
pub struct CounterMenu(Entity, Value);
#[derive(Component)]
pub struct TempDisable;
#[derive(Debug)]
pub enum SpotType {
    CommanderMain,
    CommanderAlt,
    Exile,
    Main,
    Graveyard,
}
#[derive(Component, Debug)]
pub struct CardSpot {
    spot_type: SpotType,
    ent: Option<Entity>,
}
impl CardSpot {
    pub fn new(spot_type: SpotType) -> Self {
        Self {
            spot_type,
            ent: None,
        }
    }
}
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
    side: &Option<Single<Entity, With<SideMenu>>>,
    menu: &mut Menu,
) {
    if pile.is_empty() {
        *menu = Menu::World;
        commands.entity(**side.as_ref().unwrap()).despawn();
        return;
    }
    let mut search = commands.entity(search);
    search.clear_children();
    search.with_children(|parent| {
        let node = |(i, c): (usize, &SubCard)| {
            parent.spawn((
                TargetCard(i),
                c.image_node(),
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
    mut query: Query<(&TargetCard, &mut ImageNode)>,
    search_deck: Single<(Entity, &SearchDeck)>,
    mut decks: Query<(&mut Pile, &mut Transform, &Children)>,
    mut menu: ResMut<Menu>,
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
            Without<InHand>,
            Without<Shape>,
            Without<Pile>,
        ),
    >,
    (mut colliders, follow, ids, others_ids, text, equipment, input, active_input, side, mut net): (
        Query<&mut Collider>,
        Option<Single<Entity, With<FollowMouse>>>,
        Query<&SyncObjectMe>,
        Query<&SyncObject>,
        Single<&TextInputContents>,
        Query<(), With<Equipment>>,
        Res<ButtonInput<KeyCode>>,
        Res<InputFocus>,
        Option<Single<Entity, With<SideMenu>>>,
        Net,
    ),
) {
    let left = mouse_input.just_pressed(MouseButton::Left);
    let swap = input.just_pressed(KeyCode::KeyO) && active_input.get().is_none();
    if !matches!(*menu, Menu::Side)
        || !(left || swap || (follow.is_some() && !mouse_input.pressed(MouseButton::Left)))
    {
        return;
    }
    for pointer_event in hover_map.values() {
        for entity in pointer_event.keys().copied() {
            if decks.contains(search_deck.1.0) {
                let mut drop = None;
                if let Ok((card, mut image)) = query.get_mut(entity)
                    && let Ok((mut pile, mut trans, children)) = decks.get_mut(search_deck.1.0)
                {
                    if left {
                        if pile.is_equiped() {
                            return;
                        }
                        commands.entity(entity).despawn();
                        let entity = search_deck.1.0;
                        if let Ok(id) = others_ids.get(entity) {
                            net.take(entity, *id);
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
                                &equipment,
                                &mut commands,
                            );
                        } else {
                            commands.entity(search_deck.1.0).despawn();
                        }
                        let mut transform = *trans;
                        transform.translation.y += len + CARD_THICKNESS * 4.0;
                        if let Some(e) = &follow {
                            commands.entity(**e).remove::<FollowMouse>();
                        }
                        let id = net.new_id();
                        new_pile_at(
                            Pile::Single(new.into()),
                            card_base.clone(),
                            &mut materials,
                            &mut commands,
                            &mut meshes,
                            transform,
                            true,
                            None,
                            None,
                            Some(id),
                        );
                        if let Ok(lid) = ids.get(entity) {
                            net.draw_me(
                                *lid,
                                vec![(id, Trans::from_transform(&transform))],
                                card.0 + 1,
                            );
                        } else if let Ok(oid) = others_ids.get(entity) {
                            net.draw(
                                *oid,
                                vec![(id, Trans::from_transform(&transform))],
                                card.0 + 1,
                            );
                        }
                        update_search(
                            &mut commands,
                            search_deck.0,
                            &pile,
                            &trans,
                            text.get(),
                            &side,
                            &mut menu,
                        );
                        return;
                    } else if swap {
                        let last = pile.len() - 1 == card.0;
                        let inner_card = pile.get_mut(card.0).unwrap();
                        if inner_card.data.back.is_some() {
                            inner_card.flipped = !inner_card.flipped;
                            if last {
                                repaint_face(&mut mats, &mut materials, inner_card, children);
                            }
                        }
                        if let Ok(id) = ids.get(entity) {
                            net.flip_me(*id, card.0);
                        } else if let Ok(id) = others_ids.get(entity) {
                            net.flip(*id, card.0);
                        }
                        *image = inner_card.image_node();
                        return;
                    }
                    drop = Some(card.0);
                }
                if let Some(e) = &follow
                    && **e != search_deck.1.0
                    && (drop.is_some() || entity == search_deck.0)
                {
                    let p = mem::take(decks.get_mut(**e).unwrap().0.into_inner());
                    let Ok((mut pile, trans, _)) = decks.get_mut(search_deck.1.0) else {
                        unreachable!()
                    };
                    if let Some(i) = drop {
                        pile.splice_at(i, p);
                    } else {
                        pile.extend(p)
                    }
                    commands.entity(**e).despawn();
                    update_search(
                        &mut commands,
                        search_deck.0,
                        &pile,
                        &trans,
                        text.get(),
                        &side,
                        &mut menu,
                    );
                }
            }
        }
    }
}
#[derive(Component)]
pub struct TargetCard(pub usize);
#[derive(Component)]
pub struct SearchDeck(pub Entity);
pub fn update_search_deck(
    mut commands: Commands,
    text: Single<&TextInputContents, Changed<TextInputContents>>,
    single: Option<Single<(Entity, &SearchDeck)>>,
    query: Query<(&Pile, &Transform)>,
    mut menu: ResMut<Menu>,
    counter: Option<Single<&CounterMenu>>,
    mut text3d: Query<&mut Text3d>,
    mut children: Query<(&Children, &mut Shape)>,
    ids: Query<&SyncObjectMe>,
    other_ids: Query<&SyncObject>,
    side: Option<Single<Entity, With<SideMenu>>>,
    net: Net,
) {
    match *menu {
        Menu::Side => {
            if let Some(single) = single {
                let (pile, transform) = query.get(single.1.0).unwrap();
                update_search(
                    &mut commands,
                    single.0,
                    pile,
                    transform,
                    text.get(),
                    &side,
                    &mut menu,
                )
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
                && let Shape::Counter(v, _) = v.into_inner()
            {
                v.0 = n.number.real().to_f64().round() as i128;
                for ent in children {
                    let mut text = text3d.get_mut(*ent).unwrap();
                    *text.get_single_mut().unwrap() = v.0.to_string();
                }
                if let Ok(id) = ids.get(counter.0) {
                    net.counter_me(*id, v.clone());
                } else if let Ok(id) = other_ids.get(counter.0) {
                    net.counter(*id, v.clone());
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
    peers: Res<Peers>,
    hover_map: Res<HoverMap>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if matches!(*menu, Menu::Esc)
        || (matches!(*menu, Menu::Side | Menu::Counter) && active_input.get().is_some())
    {
        return;
    }
    let scale = if input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        CARD_THICKNESS * 64.0
    } else {
        CARD_THICKNESS * 16.0
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
    if mouse_motion.delta.y != 0.0
        && (!matches!(*menu, Menu::Side | Menu::Counter)
            || hover_map
                .values()
                .any(|a| a.keys().all(|e| e.to_bits() == u32::MAX as u64)))
    {
        let translate = cam.forward().as_vec3() * scale * mouse_motion.delta.y * 16.0;
        if cam.translation.y + translate.y <= 0.0 {
            let (camera, camera_transform) = camera.into_inner();
            let Ok(ray) = camera.viewport_to_world(camera_transform, window.size() / 2.0) else {
                return;
            };
            if let Some(time) =
                ray.intersect_plane(Vec3::default(), InfinitePlane3d { normal: Dir3::Y })
            {
                cam.translation += ray.direction * (time / 2.0);
            }
        } else {
            cam.translation += translate;
        }
    }
    let epsilon = Vec3::splat(CARD_THICKNESS);
    cam.translation = cam.translation.clamp(
        Vec3::new(-W, 0.0, -W) + epsilon,
        Vec3::new(W, 2.0 * W, W) - epsilon,
    );
    if input.pressed(KeyCode::Space) {
        *cam.into_inner() = default_cam_pos(peers.me.unwrap_or_default());
    }
}
pub fn cam_rotation(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut cam: Single<&mut Transform, With<Camera3d>>,
    menu: Res<Menu>,
    hover_map: Res<HoverMap>,
) {
    if matches!(*menu, Menu::Esc)
        || (matches!(*menu, Menu::Side | Menu::Counter)
            && hover_map
                .values()
                .all(|a| a.keys().all(|e| e.to_bits() != u32::MAX as u64)))
    {
        return;
    }
    if mouse_button.pressed(MouseButton::Right) && mouse_motion.delta != Vec2::ZERO {
        let delta_yaw = -mouse_motion.delta.x * 0.001;
        let delta_pitch = -mouse_motion.delta.y * 0.001;
        let (yaw, pitch, roll) = cam.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;
        let pitch = (pitch + delta_pitch)
            .max((-PI / 2.0).next_up())
            .min((PI / 2.0).next_down());
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
    mut to_move: ResMut<ToMoveUp>,
    menu: Res<Menu>,
    active_input: Res<InputFocus>,
    mut net: Net,
) {
    if matches!(*menu, Menu::Esc)
        || (matches!(*menu, Menu::Side | Menu::Counter) && active_input.get().is_some())
    {
        return;
    }
    if input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && (input.just_pressed(KeyCode::KeyV)
            || (input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
                && input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
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
        if !input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
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
                    get_deck(url, client, asset_server, decks).await;
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
                card_base.clone(),
                &mut materials,
                &mut commands,
                &mut meshes,
                v,
                None,
                Some(net.new_id()),
                false,
            ),
            GameClipboard::Shape(shape) => Some(
                shape
                    .create(
                        Transform::from_xyz(v.x, 4.0 * MAT_BAR, v.y),
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        bevy::color::Color::WHITE,
                    )
                    .insert(net.new_id())
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
    mut sent: ResMut<Sent>,
    mut to_move: ResMut<ToMoveUp>,
    mut spots: Query<(&GlobalTransform, &mut CardSpot, &Player)>,
    peers: Res<Peers>,
    mut trans: Query<Entity, With<Pile>>,
    (ids, others_ids, search_deck, mut net): (
        Query<&SyncObjectMe>,
        Query<&SyncObject>,
        Option<Single<(Entity, &SearchDeck)>>,
        Net,
    ),
) {
    let mut decks = decks.get_deck.0.lock().unwrap();
    for (mut deck, deck_type) in decks.drain(..) {
        if deck.is_empty() {
            continue;
        }
        let (id, my_id) = if let DeckType::Other(_, id) = deck_type {
            (Some(id), None)
        } else {
            (None, Some(net.new_id()))
        };
        if id.is_none() {
            info!("deck found of size {} of type {:?}", deck.len(), deck_type);
        }
        let mut rev = false;
        let mut rem = |mut spot: (&GlobalTransform, Mut<CardSpot>, &Player)| -> Vec3 {
            if let Some(ent) = spot.1.ent
                && let Ok(entity) = trans.get_mut(ent)
            {
                spot.1.ent = None;
                if let Ok(id) = ids.get(entity) {
                    net.killed_me(*id)
                } else if let Ok(id) = others_ids.get(entity) {
                    net.killed(*id);
                }
                commands.entity(entity).despawn();
                if let Some(entity) = search_deck
                    .as_ref()
                    .and_then(|s| if s.1.0 == entity { Some(s.0) } else { None })
                {
                    commands.entity(entity).despawn()
                }
            }
            spot.0.translation()
        };
        let v = match deck_type {
            DeckType::Other(v, _) => v,
            DeckType::Single(v) => vec2_to_ground(&deck, v, rev),
            DeckType::Token => continue,
            DeckType::Deck => {
                let spot = spots
                    .iter_mut()
                    .filter(|(_, _, p)| p.0 == peers.me.unwrap_or(0))
                    .find(|(_, s, _)| matches!(s.spot_type, SpotType::Main))
                    .unwrap();
                rev = true;
                let trans = rem(spot);
                let spot = spots
                    .iter_mut()
                    .filter(|(_, _, p)| p.0 == peers.me.unwrap_or(0))
                    .find(|(_, s, _)| matches!(s.spot_type, SpotType::Exile))
                    .unwrap();
                rem(spot);
                let spot = spots
                    .iter_mut()
                    .filter(|(_, _, p)| p.0 == peers.me.unwrap_or(0))
                    .find(|(_, s, _)| matches!(s.spot_type, SpotType::Graveyard))
                    .unwrap();
                rem(spot);
                deck.shuffle(&mut net.rand);
                let v = Vec2::new(trans.x, trans.z);
                vec2_to_ground(&deck, v, rev)
            }
            DeckType::Commander => {
                let spot = spots
                    .iter_mut()
                    .filter(|(_, _, p)| p.0 == peers.me.unwrap_or(0))
                    .find(|(_, s, _)| matches!(s.spot_type, SpotType::CommanderMain))
                    .unwrap();
                let trans = rem(spot);
                let v = {
                    let spot = spots
                        .iter_mut()
                        .filter(|(_, _, p)| p.0 == peers.me.unwrap_or(0))
                        .find(|(_, s, _)| matches!(s.spot_type, SpotType::CommanderAlt))
                        .unwrap();
                    let trans = rem(spot);
                    Vec2::new(trans.x, trans.z)
                };
                if deck.len() == 2 {
                    let top = Pile::Single(deck.pop().into());
                    if let Some(ent) = new_pile(
                        top,
                        card_base.clone(),
                        &mut materials,
                        &mut commands,
                        &mut meshes,
                        v,
                        id,
                        Some(net.new_id()),
                        rev,
                    ) {
                        to_move.0.push(ent);
                    }
                }
                let v = Vec2::new(trans.x, trans.z);
                vec2_to_ground(&deck, v, rev)
            }
            DeckType::SideBoard => {
                let spot = spots
                    .iter_mut()
                    .filter(|(_, _, p)| p.0 == peers.me.unwrap_or(0))
                    .find(|(_, s, _)| matches!(s.spot_type, SpotType::CommanderMain))
                    .unwrap();
                let trans = spot.0.translation();
                let v = Vec2::new(trans.x, trans.z);
                vec2_to_ground(&deck, v, rev)
            }
        };
        if let Some(ent) = new_pile_at(
            deck,
            card_base.clone(),
            &mut materials,
            &mut commands,
            &mut meshes,
            v,
            false,
            None,
            id,
            my_id,
        ) {
            to_move.0.push(ent.id());
        }
        if let Some(id) = id {
            sent.del(id);
            net.received(id.user, id.id);
        }
    }
}
pub fn to_move_up(
    mut to_do: ResMut<ToMoveUp>,
    mut ents: Query<(&Collider, &mut Transform), Without<Wall>>,
    mut pset: ParamSet<(Query<&mut Position>, SpatialQuery)>,
) {
    for ent in to_do.0.drain(..) {
        move_up(ent, &mut ents, &mut pset);
    }
}
#[derive(Resource, Default)]
pub struct ToMoveUp(pub Vec<Entity>);
pub fn give_ents(to_do: Res<GiveEnts>, ents: Query<(&SyncObject, Entity)>, mut net: Net) {
    for peer in to_do.0.lock().unwrap().drain(..) {
        for (id, ent) in ents {
            if id.user == peer {
                net.take(ent, *id);
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
    update_search(
        commands,
        search.unwrap(),
        pile,
        transform,
        "",
        &None,
        &mut Menu::World,
    );
}
pub fn pile_merge(
    collision: On<CollisionStart>,
    mut piles: Query<
        (
            Entity,
            &mut Pile,
            &mut Transform,
            &Children,
            &mut Collider,
            Option<&SyncObject>,
            Option<&SyncObjectMe>,
        ),
        (Without<InHand>, Without<FollowMouse>),
    >,
    mut mats: Query<&mut MeshMaterial3d<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query_meshes: Query<
        (&mut Mesh3d, &mut Transform),
        (
            Without<Children>,
            With<ChildOf>,
            Without<InHand>,
            Without<Shape>,
            Without<Pile>,
        ),
    >,
    mut commands: Commands,
    search_deck: Option<Single<(Entity, &SearchDeck)>>,
    text: Option<Single<&TextInputContents>>,
    equipment: Query<(), With<Equipment>>,
    (mut menu, side, net): (ResMut<Menu>, Option<Single<Entity, With<SideMenu>>>, Net),
) {
    if let Ok((e1, _, t1, _, _, s1, m1)) = piles.get(collision.collider1)
        && let Ok((e2, _, t2, _, _, s2, m2)) = piles.get(collision.collider2)
        && e1 < e2
        && (t1.translation.x - t2.translation.x).abs() < CARD_WIDTH / 2.0
        && (t1.translation.z - t2.translation.z).abs() < CARD_HEIGHT / 2.0
        && is_reversed(t1) == is_reversed(t2)
    {
        let (
            (ent, mut bottom_pile, mut bottom_transform, children, mut collider, _, sync_me),
            top_pile,
            top_ent,
            top_sync,
            top_sync_me,
        ) = if t1.translation.y < t2.translation.y {
            if s1.is_some() {
                return;
            }
            let s2 = s2.cloned();
            let m2 = m2.cloned();
            let p2 = mem::replace(
                piles.get_mut(collision.collider2).unwrap().1.into_inner(),
                Pile::Empty,
            );
            (piles.get_mut(collision.collider1).unwrap(), p2, e2, s2, m2)
        } else {
            if s2.is_some() {
                return;
            }
            let s1 = s1.cloned();
            let m1 = m1.cloned();
            let p1 = mem::replace(
                piles.get_mut(collision.collider1).unwrap().1.into_inner(),
                Pile::Empty,
            );
            (piles.get_mut(collision.collider2).unwrap(), p1, e1, s1, m1)
        };
        if let Some(mid) = sync_me {
            let at = if is_reversed(&bottom_transform) {
                0
            } else {
                bottom_pile.len()
            };
            if let Some(id) = top_sync {
                net.merge(*mid, id, at)
            } else if let Some(id) = top_sync_me {
                net.merge_me(*mid, id, at)
            }
        }
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
            &equipment,
            &mut commands,
        );
        if let Some(search_deck) = search_deck {
            if search_deck.1.0 == ent {
                update_search(
                    &mut commands,
                    search_deck.0,
                    &bottom_pile,
                    &bottom_transform,
                    text.as_ref().unwrap().get(),
                    &None,
                    &mut Menu::World,
                );
            } else if search_deck.1.0 == top_ent {
                *menu = Menu::World;
                commands.entity(**side.as_ref().unwrap()).despawn();
            }
        }
        commands.entity(top_ent).despawn();
    }
}
pub fn set_card_spot(
    spatial: SpatialQuery,
    query: Query<(&GlobalTransform, &mut CardSpot)>,
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
    for (transform, mut spot) in query {
        let transform = transform.compute_transform();
        let intersects = spatial.shape_intersections(
            &Collider::cuboid(CARD_WIDTH / 2.0, CARD_THICKNESS / 2.0, CARD_HEIGHT / 2.0),
            transform.translation,
            transform.rotation,
            &SpatialQueryFilter::DEFAULT,
        );
        if let Some(ent) = spot.ent {
            if intersects.contains(&ent) {
                continue;
            } else {
                spot.ent = None;
            }
        }
        for ent in intersects {
            if let Ok((mut t, mut lv, mut av, pile)) = transforms.get_mut(ent) {
                let mut transform = transform;
                transform.translation.y = pile.len() as f32 * CARD_THICKNESS / 2.0;
                if transform.translation.distance(t.translation) > CARD_THICKNESS {
                    lv.0 = Vector::default();
                    av.0 = Vector::default();
                    let rev = is_reversed(&t);
                    *t = transform;
                    if rev {
                        t.rotate_local_z(PI);
                    }
                }
                spot.ent = Some(ent);
                continue;
            }
        }
    }
}
pub fn rem_peers(
    rem_peers: Res<RemPeers>,
    cams: Query<(Entity, &CameraInd)>,
    curs: Query<(Entity, &CursorInd)>,
    mut commands: Commands,
) {
    for peer in rem_peers.0.lock().unwrap().drain(..) {
        if let Some(e) = cams
            .iter()
            .find_map(|(e, a)| if a.0 == peer { Some(e) } else { None })
        {
            commands.entity(e).despawn()
        }
        if let Some(e) = curs
            .iter()
            .find_map(|(e, a)| if a.0 == peer { Some(e) } else { None })
        {
            commands.entity(e).despawn()
        }
    }
}
