use crate::counters::{Counter, Value, modify, modify_view};
use crate::download::{
    Exact, get_alts, get_deck, get_deck_export, spawn_scryfall_list, spawn_singleton,
    spawn_singleton_id,
};
use crate::misc::{
    Equipment, adjust_meshes, default_cam_pos, is_reversed, new_pile, new_pile_at, remove_follow,
    repaint_face, rotate_left, rotate_right, spawn_equip, ui_rotate_left, ui_rotate_right,
    vec2_to_ground,
};
use crate::setup::{EscMenu, MAT_WIDTH, Player, SideMenu, TextChat, TextInput, TextMenu, W, Wall};
use crate::shapes::Side;
use crate::sync::{CameraInd, CursorInd, InOtherHand, Net, SyncObjectMe, Trans};
use crate::*;
use avian3d::math::Vector;
#[cfg(feature = "steam")]
use bevy::diagnostic::FrameCount;
use bevy::input::mouse::{
    AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit, MouseWheel,
};
use bevy::input_focus::InputFocus;
use bevy::math::bounding::{Aabb3d, BoundingVolume};
use bevy::picking::hover::HoverMap;
use bevy::window::PrimaryWindow;
use bevy_rich_text3d::Text3d;
#[cfg(feature = "steam")]
use bevy_tangled::ClientTrait;
use bevy_tangled::PeerId;
#[cfg(feature = "calc")]
use bevy_ui_text_input::TextInputBuffer;
use bevy_ui_text_input::actions::{TextInputAction, TextInputEdit};
use bevy_ui_text_input::{
    SubmitText, TextInputContents, TextInputMode, TextInputNode, TextInputQueue,
};
#[cfg(feature = "calc")]
use kalc_lib::complex::NumStr;
#[cfg(feature = "calc")]
use kalc_lib::types::{Complex, Float};
#[cfg(feature = "calc")]
use kalc_lib::units::{Number, Options, Variable};
use rand::Rng;
use std::collections::hash_map::Entry;
use std::f32::consts::PI;
use std::mem;
#[derive(Component)]
pub struct HandIgnore;
pub const HAND_WIDTH: f32 = MAT_WIDTH - CARD_HEIGHT;
pub const LIFT_SPACE: f32 = 2.0 * CARD_THICKNESS;
pub const HAND_LIFT_SPACE: f32 = CARD_HEIGHT * 3.0 / 4.0;
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
    peers: Res<Peers>,
) {
    spatial.shape_intersections_callback(
        &Collider::cuboid(HAND_WIDTH, CARD_HEIGHT, CARD_HEIGHT),
        hand.0.translation,
        hand.0.rotation,
        &SpatialQueryFilter::from_mask(u32::MAX - 0b100),
        |ent| {
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
                    *linvel = default();
                    *angvel = default();
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
                    if peers.me.is_some_and(|i| i == 1 || i == 3) {
                        rotate_right(&mut trans);
                        rotate_right(&mut trans);
                    }
                    return false;
                }
            }
            true
        },
    );
}
#[cfg(feature = "steam")]
pub fn update_rich(client: Res<Client>, peers: Res<Peers>, frame: Res<FrameCount>) {
    if !frame.0.is_multiple_of(600) {
        return;
    }
    if peers.me.is_some() {
        fn get_time(time: u32) -> String {
            format!(
                "{:02}h:{:02}m",
                time / (60 * 60 * 60),
                (time / (60 * 60)) % 60
            )
        }
        client.set_rich_presence("players", Some(&peers.map().len().to_string()));
        client.set_rich_presence("max_players", Some("4"));
        client.set_rich_presence("time", Some(&get_time(frame.0 / 60)));
        client.set_rich_presence("steam_display", Some("#InLobby"));
        client.set_rich_presence("steam_player_group", Some(&client.host_id().to_string()));
    } else {
        client.clear_rich_presence();
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
    keybinds: Keybinds,
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
        if keybinds.just_pressed(Keybind::SortHand) {
            let mut order = children
                .iter()
                .filter_map(|c| card.get(c).ok())
                .map(|(e, _, c)| (c.first().data.face.mana_cost.total(), e.0))
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
    keybinds: Keybinds,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window, With<PrimaryWindow>>,
    cards: Query<&ColliderAabb, (Without<FollowMouse>, Without<Hand>)>,
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
            &ColliderAabb,
            &GlobalTransform,
            Option<&ChildOf>,
            Option<&Pile>,
            &Collider,
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
        && keybinds.pressed(Keybind::Select)
    {
        card.3.y = 0.0;
        let aabb = card.4;
        let mut some = false;
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
            card.1.translation = point;
        }
        let mut max = f32::NEG_INFINITY;
        spatial.shape_intersections_callback(
            card.8,
            card.1.translation,
            card.1.rotation,
            &SpatialQueryFilter::from_mask(u32::MAX - 0b100),
            |a| {
                if !walls.contains(a)
                    && a != card.0
                    && let Ok(aabb) = cards.get(a)
                    && aabb.max.y > max
                {
                    max = aabb.max.y;
                }
                true
            },
        );
        if max != f32::INFINITY {
            some = true;
            let max = max + (aabb.max.y - aabb.min.y) / 2.0 + CARD_THICKNESS;
            let max = max.max(card.1.translation.y);
            card.1.translation.y = max;
        }
        if child.contains(card.0) {
            if Collider::cuboid(
                HAND_WIDTH + CARD_THICKNESS,
                CARD_HEIGHT + CARD_THICKNESS,
                CARD_HEIGHT + CARD_THICKNESS,
            )
            .aabb(hand.0.translation, hand.0.rotation)
            .intersects(card.4)
            {
                let cur = child.get(card.0).unwrap().0;
                let n = swap_pos(&mut hand, card.1.translation.x, &mut child, cur);
                child.get_mut(card.0).unwrap().0 = n;
                if card.6.is_some() {
                    commands.entity(card.0).remove_parent_in_place();
                }
                commands.entity(card.0).insert(RigidBodyDisabled);
                card.1.translation.y = HAND_LIFT_SPACE;
                some = true;
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
        if some
            && let Some(time) = ray.intersect_plane(
                if card.6.is_some() {
                    card.5.translation()
                } else {
                    card.1.translation
                },
                InfinitePlane3d { normal: Dir3::Y },
            )
        {
            let mut point = ray.get_point(time);
            point.x = point.x.clamp(
                -W + (aabb.min.x - card.1.translation.x).abs(),
                W - (aabb.max.x - card.1.translation.x).abs(),
            );
            point.z = point.z.clamp(
                -W + (aabb.min.z - card.1.translation.z).abs(),
                W - (aabb.max.z - card.1.translation.z).abs(),
            );
            card.1.translation = point;
        }
    } else if card.7.is_some_and(|s| s.len() == 1)
        && let Some(time) =
            ray.intersect_plane(card.5.translation(), InfinitePlane3d { normal: Dir3::Y })
        && ColliderAabb::new(
            hand.0.translation,
            Vec3::new(
                HAND_WIDTH + CARD_THICKNESS,
                CARD_HEIGHT + CARD_THICKNESS,
                CARD_HEIGHT + CARD_THICKNESS,
            ) / 2.0,
        )
        .intersects(&card.8.aabb(ray.get_point(time), card.1.rotation))
    {
        remove_follow(&mut commands, card.0);
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
        remove_follow(&mut commands, card.0);
        commands
            .entity(card.0)
            .remove::<RigidBodyDisabled>()
            .remove::<SleepingDisabled>();
        card.2.0 = GRAVITY
    }
}
#[derive(Component, Deref, DerefMut)]
pub struct PingDrag(pub Vec3);
pub fn ping_drag(
    keybinds: Keybinds,
    focus: Focus,
    drag: Option<Single<(Entity, &PingDrag, &mut Mesh3d, &mut Transform), Without<PeerId>>>,
    mut commands: Commands,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window, With<PrimaryWindow>>,
    spatial: SpatialQuery,
    peer: Res<Peers>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if focus.key_lock() || !keybinds.pressed(Keybind::Ping) {
        if let Some(drag) = drag {
            commands.entity(drag.0).despawn();
        }
        return;
    }
    let Some(cursor_position) = window.cursor_position() else {
        if let Some(drag) = drag {
            commands.entity(drag.0).despawn();
        }
        return;
    };
    let (camera, camera_transform) = camera.into_inner();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        if let Some(drag) = drag {
            commands.entity(drag.0).despawn();
        }
        return;
    };
    let Some(v) = spatial.cast_ray(
        ray.origin,
        ray.direction,
        f32::MAX,
        true,
        &SpatialQueryFilter::default(),
    ) else {
        if let Some(drag) = drag {
            commands.entity(drag.0).despawn();
        }
        return;
    };
    let v = ray.origin + ray.direction * v.distance;
    if let Some((_, orig, mut mesh, mut transform)) = drag.map(|e| e.into_inner()) {
        let dir = (v - orig.0).normalize();
        let d = (v - orig.0).length();
        let m = (v + orig.0) / 2.0;
        transform.translation = m;
        mesh.0 = meshes.add(Cylinder::new(CARD_THICKNESS * 8.0, d));
        transform.rotation = Quat::from_rotation_arc(Vec3::Y, dir);
    } else {
        commands.spawn((
            PingDrag(v),
            Mesh3d(meshes.add(Cylinder::new(CARD_THICKNESS * 8.0, 0.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                alpha_mode: AlphaMode::Opaque,
                unlit: true,
                base_color: PLAYER[peer.me.unwrap_or(0) % PLAYER.len()],
                ..default()
            })),
            Transform::from_xyz(v.x, v.y, v.z),
        ));
    }
}
pub fn untap_keybinds(
    spatial: SpatialQuery,
    mut cards: Query<
        (&Pile, &mut Transform, Option<&SyncObject>),
        (Without<FollowOtherMouse>, Without<FollowMouse>),
    >,
    mut net: Net,
    peers: Res<Peers>,
    mut commands: Commands,
    mut focus: Focus,
    keybinds: Keybinds,
    search_deck: Option<Single<(Entity, &SearchDeck)>>,
    text: Option<Single<&TextInputContents, With<SearchText>>>,
    side: Option<Single<Entity, With<SideMenu>>>,
) {
    if focus.key_lock() || !keybinds.just_pressed(Keybind::Untap) {
        return;
    }
    let (x, y, z) = (
        MAT_WIDTH / 2.0 - CARD_WIDTH / 2.0 - MAT_BAR,
        MAT_HEIGHT / 2.0,
        CARD_THICKNESS / 2.0,
    );
    let aabb = match peers.me.unwrap_or(0) {
        1 => (x, z, -y),
        2 => (-x, z, y),
        3 => (-x, z, -y),
        _ => (x, z, y),
    };
    let aabb = Aabb3d::new(aabb, (x, z, y));
    spatial.shape_intersections_callback(
        &Collider::cuboid(2.0 * x, CARD_THICKNESS, 2.0 * y),
        aabb.center().into(),
        Quat::default(),
        &SpatialQueryFilter::from_mask(u32::MAX - 0b100),
        |ent| {
            let Ok((pile, mut transform, id)) = cards.get_mut(ent) else {
                return true;
            };
            if aabb.closest_point(transform.translation) != transform.translation.into() {
                return true;
            }
            if let Some(id) = id {
                net.take(ent, *id);
            }
            let rev = is_reversed(&transform);
            transform.rotation = Quat::default();
            if rev {
                transform.rotate_local_z(PI);
                if let Some(entity) = search_deck
                    .as_ref()
                    .and_then(|s| if s.1.0 == ent { Some(s.0) } else { None })
                {
                    update_search(
                        &mut commands,
                        entity,
                        pile,
                        &transform,
                        text.as_ref().unwrap().get(),
                        &side,
                        &mut focus.menu,
                    );
                }
            }
            if matches!(peers.me.unwrap_or(0), 1 | 3) {
                rotate_right(&mut transform);
                rotate_right(&mut transform);
            }
            if matches!(pile.get_card(&transform).data.layout, Layout::Room) {
                rotate_right(&mut transform);
            }
            true
        },
    );
}
pub fn listen_for_mouse(
    keybinds: Keybinds,
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
        side,
        search_deck,
        mut shape_hold,
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
                Without<Side>,
            ),
        >,
        Option<Single<Entity, With<FollowMouse>>>,
        Query<(&mut Shape, Entity)>,
        Option<Single<Entity, With<SideMenu>>>,
        Option<Single<(Entity, &SearchDeck)>>,
        Query<(&mut ShapeHold, &Children), Without<Pile>>,
    ),
    (
        text,
        mut text3d,
        children,
        mut transforms,
        equipment,
        counters,
        mut net,
        mut turn,
        peers,
        mut focus,
        sides,
    ): (
        Option<Single<&TextInputContents, With<SearchText>>>,
        Query<&mut Text3d>,
        Query<&Children, (Without<Pile>, Without<ShapeHold>)>,
        Query<&mut Transform, (Or<(With<Pile>, With<Shape>)>, Without<Side>)>,
        Query<(), With<Equipment>>,
        Query<&Counter>,
        Net,
        ResMut<Turn>,
        Res<Peers>,
        Focus,
        Query<(&Side, &Transform)>,
    ),
) {
    if focus.key_lock() {
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
        if keybinds.just_pressed(Keybind::ScaleUp) {
            transform.scale *= 1.25;
            if let Ok(id) = ids.get(entity) {
                net.scale_me(*id, transform.scale.x);
            } else if let Ok(id) = others_ids.get(entity) {
                net.scale(*id, transform.scale.x);
            }
        } else if keybinds.just_pressed(Keybind::ScaleDown) {
            transform.scale /= 1.25;
            if let Ok(id) = ids.get(entity) {
                net.scale_me(*id, transform.scale.x);
            } else if let Ok(id) = others_ids.get(entity) {
                net.scale(*id, transform.scale.x);
            }
        } else if let Ok((mut pile, children, parent, inhand, inother)) = cards.get_mut(entity) {
            if keybinds.just_pressed(Keybind::Flip) && zoom.is_none() {
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
                        &mut focus.menu,
                    );
                }
            } else if keybinds.just_pressed(Keybind::Shuffle) {
                if pile.len() > 1 {
                    pile.shuffle(&mut net.rand);
                    let card = pile.last();
                    repaint_face(&mut mats, &mut materials, card, children);
                    if let Ok(id) = ids.get(entity) {
                        net.reorder_me(*id, pile.iter().map(|a| a.id).collect());
                    } else if let Ok(id) = others_ids.get(entity) {
                        net.reorder(*id, pile.iter().map(|a| a.id).collect());
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
                            &mut focus.menu,
                        );
                    }
                }
            } else if keybinds.just_pressed(Keybind::Remove) {
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
                    *focus.menu = Menu::World;
                    commands.entity(**side.as_ref().unwrap()).despawn();
                }
            } else if keybinds.just_pressed(Keybind::MiscCounter)
                && let Pile::Single(c) = &mut *pile
            {
                c.misc = if c.misc.is_none() {
                    Some(Value(0))
                } else {
                    None
                };
                if let Ok(id) = ids.get(entity) {
                    net.modify_me(*id, Counter::Misc, c.misc.clone());
                } else if let Ok(id) = others_ids.get(entity) {
                    net.modify(*id, Counter::Misc, c.misc.clone());
                }
                modify(
                    entity,
                    c,
                    children,
                    &mut commands,
                    counters,
                    &mut materials,
                    &mut meshes,
                    Counter::Misc,
                );
            } else if keybinds.just_pressed(Keybind::Counters)
                && let Pile::Single(c) = &mut *pile
            {
                c.counters = if c.counters.is_none() {
                    Some(Value(0))
                } else {
                    None
                };
                if let Ok(id) = ids.get(entity) {
                    net.modify_me(*id, Counter::Counters, c.counters.clone());
                } else if let Ok(id) = others_ids.get(entity) {
                    net.modify(*id, Counter::Counters, c.counters.clone());
                }
                modify(
                    entity,
                    c,
                    children,
                    &mut commands,
                    counters,
                    &mut materials,
                    &mut meshes,
                    Counter::Counters,
                );
            } else if keybinds.just_pressed(Keybind::Loyalty)
                && let Pile::Single(c) = &mut *pile
            {
                c.loyalty = if c.loyalty.is_none() {
                    Some(Value(c.subcard.face().loyalty.unwrap_or_default() as i128))
                } else {
                    None
                };
                if let Ok(id) = ids.get(entity) {
                    net.modify_me(*id, Counter::Loyalty, c.loyalty.clone());
                } else if let Ok(id) = others_ids.get(entity) {
                    net.modify(*id, Counter::Loyalty, c.loyalty.clone());
                }
                modify(
                    entity,
                    c,
                    children,
                    &mut commands,
                    counters,
                    &mut materials,
                    &mut meshes,
                    Counter::Loyalty,
                );
            } else if keybinds.just_pressed(Keybind::Power)
                && let Pile::Single(c) = &mut *pile
            {
                c.power = if c.power.is_none() {
                    Some(Value(c.subcard.face().power.unwrap_or_default() as i128))
                } else {
                    None
                };
                if let Ok(id) = ids.get(entity) {
                    net.modify_me(*id, Counter::Power, c.power.clone());
                } else if let Ok(id) = others_ids.get(entity) {
                    net.modify(*id, Counter::Power, c.power.clone());
                }
                modify(
                    entity,
                    c,
                    children,
                    &mut commands,
                    counters,
                    &mut materials,
                    &mut meshes,
                    Counter::Power,
                );
            } else if keybinds.just_pressed(Keybind::Toughness)
                && let Pile::Single(c) = &mut *pile
            {
                c.toughness = if c.toughness.is_none() {
                    Some(Value(c.subcard.face().toughness.unwrap_or_default() as i128))
                } else {
                    None
                };
                if let Ok(id) = ids.get(entity) {
                    net.modify_me(*id, Counter::Toughness, c.toughness.clone());
                } else if let Ok(id) = others_ids.get(entity) {
                    net.modify(*id, Counter::Toughness, c.toughness.clone());
                }
                modify(
                    entity,
                    c,
                    children,
                    &mut commands,
                    counters,
                    &mut materials,
                    &mut meshes,
                    Counter::Toughness,
                );
            } else if keybinds.just_pressed(Keybind::CopyObject) {
                *game_clipboard = GameClipboard::Pile(pile.clone());
            } else if keybinds.just_pressed(Keybind::Copy) && !is_reversed(&transform) {
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
            } else if keybinds.just_pressed(Keybind::PickCard) && pile.len() > 1 {
                if !focus.mouse_lock() {
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
                    let len = pile.len() as f32 * CARD_THICKNESS;
                    let draw_len = if is_reversed(&transform) {
                        1
                    } else {
                        pile.len()
                    };
                    let new = pile.take_card(&transform);
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
                        None,
                        &mut commands,
                    );
                    let mut transform = *transform;
                    transform.translation.y += len + LIFT_SPACE;
                    if let Some(e) = follow {
                        remove_follow(&mut commands, *e);
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
                            vec![(id, Trans::from_transform(&transform), card.id)],
                            draw_len,
                        );
                    } else if let Ok(oid) = others_ids.get(entity) {
                        net.draw(
                            *oid,
                            vec![(id, Trans::from_transform(&transform), card.id)],
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
                            &mut focus.menu,
                        );
                    }
                }
            } else if keybinds.just_pressed(Keybind::Select) {
                if !focus.mouse_lock() {
                    if inother.is_some() {
                        let mut ent = commands.entity(entity);
                        ent.remove::<InOtherHand>();
                        ent.remove::<SleepingDisabled>();
                        repaint_face(&mut mats, &mut materials, pile.first(), children);
                        colliders.get_mut(entity).unwrap().1.0 = GRAVITY;
                    }
                    if inhand.is_some() {
                        transform.translation.y = HAND_LIFT_SPACE;
                    } else {
                        transform.translation.y += LIFT_SPACE;
                    }
                    if let Some(e) = follow {
                        remove_follow(&mut commands, *e);
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
            } else if keybinds.just_pressed(Keybind::Equip) && !is_reversed(&transform) {
                //TODO instead move equipment above card
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
            } else if keybinds.just_pressed(Keybind::RotateRight) && zoom.is_none() {
                if let Ok(id) = others_ids.get(entity) {
                    net.take(entity, *id);
                }
                rotate_right(&mut transform);
            } else if keybinds.just_pressed(Keybind::Spread) && pile.len() > 1 {
                let mut start = *transform;
                start.translation.y = CARD_THICKNESS / 2.0;
                let mut transform = start;
                transform.rotation = Quat::default();
                let mut vec = Vec::with_capacity(pile.len());
                for c in pile.drain(..) {
                    let id = net.new_id();
                    let uuid = c.id;
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
                    transform.translation.x += CARD_WIDTH + CARD_THICKNESS;
                    if transform.translation.x >= W - CARD_WIDTH - CARD_THICKNESS {
                        transform.translation.x = start.translation.x;
                        transform.translation.z += CARD_HEIGHT + CARD_THICKNESS;
                    }
                    vec.push((id, Trans::from_transform(&transform), uuid));
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
                        &mut focus.menu,
                    );
                }
            } else if keybinds.just_pressed(Keybind::RotateLeft) && zoom.is_none() {
                if let Ok(id) = others_ids.get(entity) {
                    net.take(entity, *id);
                }
                rotate_left(&mut transform);
            } else if keybinds.just_pressed(Keybind::Printings) && !is_reversed(&transform) {
                let top = pile.get_card(&transform);
                let mut v = Vec2::new(transform.translation.x, transform.translation.z);
                if v.y.is_sign_positive() {
                    v.y -= CARD_HEIGHT + CARD_THICKNESS
                } else {
                    v.y += CARD_HEIGHT + CARD_THICKNESS
                }
                let client = down.client.0.clone();
                let get_deck = down.get_deck.clone();
                let asset_server = asset_server.clone();
                let id = top.id;
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
            } else if keybinds.just_pressed(Keybind::Tokens) && !is_reversed(&transform) {
                let top = pile.get_card(&transform);
                let mut v = Vec2::new(transform.translation.x, transform.translation.z);
                if v.y.is_sign_positive() {
                    v.y -= CARD_HEIGHT + CARD_THICKNESS
                } else {
                    v.y += CARD_HEIGHT + CARD_THICKNESS
                }
                let client = down.client.0.clone();
                let get_deck = down.get_deck.clone();
                let asset_server = asset_server.clone();
                let ids = top.tokens.clone();
                if !ids.is_empty() {
                    info!(
                        "{}: {} has requested tokens {ids:?}",
                        top.face().name,
                        top.id
                    );
                    #[cfg(not(feature = "wasm"))]
                    down.runtime.0.spawn(async move {
                        spawn_scryfall_list(ids, client, asset_server, get_deck, v).await
                    });
                    #[cfg(feature = "wasm")]
                    wasm_bindgen_futures::spawn_local(async move {
                        spawn_scryfall_list(ids, client, asset_server, get_deck, v).await;
                    })
                }
            } else if keybinds.just_pressed(Keybind::Transform)
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
                let flipped = card.flipped;
                let idx = if is_reversed(&transform) {
                    0
                } else {
                    pile.len() - 1
                };
                if let Ok(id) = ids.get(entity) {
                    net.flip_me(*id, idx, flipped);
                } else if let Ok(id) = others_ids.get(entity) {
                    net.flip(*id, idx, flipped);
                }
            } else if keybinds.just_pressed(Keybind::Reveal)
                && parent.is_none()
                && inother.is_none()
            {
                todo!()
                /*let n = keybinds.get_numeric();
                if n != 0 {
                    //TODO
                }*/
            } else if keybinds.just_pressed(Keybind::Exile) && parent.is_none() && inother.is_none()
            {
                let n = keybinds.get_numeric();
                if n != 0 {
                    let pile = pile.take_n_card(&transform, n);
                    if pile.is_empty() {
                        if let Ok(id) = ids.get(entity) {
                            net.killed_me(*id)
                        } else if let Ok(id) = others_ids.get(entity) {
                            net.killed(*id);
                        }
                        commands.entity(entity).despawn();
                    }
                    let pile = Pile::new(pile);
                    commands.trigger(AddToSpot {
                        pile,
                        spot: SpotType::Exile,
                        player: peers.me.unwrap_or_default(),
                        from: if let Ok(id) = ids.get(entity) {
                            Some((net.to_global(*id), is_reversed(&transform)))
                        } else if let Ok(id) = others_ids.get(entity) {
                            Some((*id, is_reversed(&transform)))
                        } else {
                            None
                        },
                    });
                }
            } else if keybinds.just_pressed(Keybind::Mill) && parent.is_none() && inother.is_none()
            {
                let n = keybinds.get_numeric();
                if n != 0 {
                    let pile = pile.take_n_card(&transform, n);
                    if pile.is_empty() {
                        if let Ok(id) = ids.get(entity) {
                            net.killed_me(*id)
                        } else if let Ok(id) = others_ids.get(entity) {
                            net.killed(*id);
                        }
                        commands.entity(entity).despawn();
                    }
                    let pile = Pile::new(pile);
                    commands.trigger(AddToSpot {
                        pile,
                        spot: SpotType::Graveyard,
                        player: peers.me.unwrap_or_default(),
                        from: if let Ok(id) = ids.get(entity) {
                            Some((net.to_global(*id), is_reversed(&transform)))
                        } else if let Ok(id) = others_ids.get(entity) {
                            Some((*id, is_reversed(&transform)))
                        } else {
                            None
                        },
                    });
                }
            } else if keybinds.just_pressed(Keybind::Draw) && parent.is_none() && inother.is_none()
            {
                let n = keybinds.get_numeric();
                if n != 0 {
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
                        let uuid = new.id;
                        let mut ent = new_pile_at(
                            Pile::Single(new.into()),
                            card_base.clone(),
                            &mut materials,
                            &mut commands,
                            &mut meshes,
                            Transform::default().looking_to(
                                if peers.me.is_some_and(|i| i == 1 || i == 3) {
                                    Dir3::Z
                                } else {
                                    Dir3::NEG_Z
                                },
                                Dir3::Y,
                            ),
                            false,
                            Some(hand.1),
                            None,
                            Some(id),
                        )
                        .unwrap();
                        ent.insert(InHand(hand.0.count));
                        ent.insert(RigidBodyDisabled);
                        vec.push((id, Trans::from_transform(&Transform::default()), uuid));
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
                            &mut focus.menu,
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
                            Some(&counters),
                            &mut commands,
                        );
                    } else {
                        if let Ok(id) = ids.get(entity) {
                            net.killed_me(*id)
                        } else if let Ok(id) = others_ids.get(entity) {
                            net.killed(*id);
                        }
                        commands.entity(entity).despawn();
                    }
                }
            } else if keybinds.just_pressed(Keybind::Search) {
                search(
                    entity,
                    &pile,
                    &transform,
                    &side,
                    &mut commands,
                    &mut focus.active_input,
                );
                *focus.menu = Menu::Side;
            }
            if keybinds.pressed(Keybind::View) && inother.is_none() && !pile.is_empty() {
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
                            if let Pile::Single(card) = &*pile
                                && pile.is_modified()
                            {
                                modify_view(card, parent);
                            }
                        });
                };
                if let Some(mut single) = zoom {
                    if single.1.0 != entity.to_bits() {
                        if !is_reversed(&transform) {
                            spawn();
                        }
                        commands.entity(single.0).despawn();
                    } else if keybinds.just_pressed(Keybind::Transform) {
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
                    } else if keybinds.just_pressed(Keybind::RotateRight) {
                        ui_rotate_right(&mut single.3);
                    } else if keybinds.just_pressed(Keybind::RotateLeft) {
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
            if keybinds.just_pressed(Keybind::Sub)
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
            } else if keybinds.just_pressed(Keybind::Add)
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
            } else if keybinds.just_pressed(Keybind::Select) {
                if let Some(e) = follow {
                    remove_follow(&mut commands, *e);
                }
                if let Ok(id) = others_ids.get(entity) {
                    net.take(entity, *id);
                }
                transform.translation.y += LIFT_SPACE;
                phys.0 = 0.0;
                let mut cmd = commands.entity(entity);
                cmd.insert((FollowMouse, SleepingDisabled))
                    .remove::<FollowOtherMouse>();
                if shape.get(entity).is_ok_and(|s| s.0.is_dice()) {
                    cmd.insert(ShapeHold(None));
                }
            } else if keybinds.just_pressed(Keybind::CopyObject)
                && let Ok((shape, _)) = shape.get(entity)
            {
                *game_clipboard = GameClipboard::Shape(shape.clone());
            } else if keybinds.just_pressed(Keybind::Calc)
                && let Ok((s, _)) = shape.get(entity)
                && let Shape::Counter(v, _) = s
            {
                spawn_calc(&mut commands, &mut focus, entity, v, None);
            } else if keybinds.just_pressed(Keybind::Remove) {
                if let Ok(id) = ids.get(entity) {
                    net.killed_me(*id)
                } else if let Ok(id) = others_ids.get(entity) {
                    net.killed(*id);
                }
                commands.entity(entity).despawn();
            } else if keybinds.just_pressed(Keybind::Flip) {
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
            } else if keybinds.just_pressed(Keybind::Shuffle)
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
            } else if keybinds.just_pressed(Keybind::RotateRight) {
                if let Ok(id) = others_ids.get(entity) {
                    net.take(entity, *id);
                }
                rotate_right(&mut transform)
            } else if keybinds.just_pressed(Keybind::RotateLeft) {
                if let Ok(id) = others_ids.get(entity) {
                    net.take(entity, *id);
                }
                rotate_left(&mut transform)
            } else if let Ok((shape, _)) = shape.get(entity)
                && let Ok((mut hold, children)) = shape_hold.get_mut(entity)
                && keybinds.just_pressed(Keybind::Draw)
            {
                let n = keybinds.get_numeric();
                **hold = if let Some(k) = &**hold {
                    format!("{k}{n}").parse().ok().and_then(|k| {
                        if shape.in_range(k) {
                            Some(k)
                        } else if shape.in_range(n) {
                            Some(n)
                        } else {
                            None
                        }
                    })
                } else if shape.in_range(n) {
                    Some(n)
                } else {
                    None
                };
                if let Some(n) = &**hold {
                    let t = children
                        .iter()
                        .find_map(|c| {
                            sides
                                .get(c)
                                .ok()
                                .and_then(|(s, t)| if **s == *n { Some(t) } else { None })
                        })
                        .unwrap();
                    transform.rotation = Quat::from_rotation_arc(*-t.forward(), Vec3::Y);
                }
            }
        } else if let Some(single) = zoom {
            commands.entity(single.0).despawn();
        }
    } else if let Some(single) = zoom {
        commands.entity(single.0).despawn();
    }
}
pub fn spawn_calc(
    commands: &mut Commands,
    focus: &mut Focus,
    entity: Entity,
    v: &Value,
    counter: Option<Counter>,
) {
    #[cfg(feature = "calc")]
    {
        *focus.menu = Menu::Counter;
        let mut queue = TextInputQueue::default();
        queue.add(TextInputAction::Edit(TextInputEdit::Insert('n', false)));
        let ent = commands
            .spawn((
                CounterMenu(entity, v.clone(), counter),
                Node {
                    width: Val::Percent(20.0),
                    height: Val::Px(FONT_HEIGHT * 1.5),
                    ..default()
                },
                BackgroundColor(Color::srgba_u8(0, 0, 0, 127)),
                TextInputNode {
                    mode: TextInputMode::SingleLine,
                    clear_on_submit: false,
                    unfocus_on_submit: false,
                    ..default()
                },
                TextFont {
                    font_size: FONT_SIZE,
                    ..default()
                },
                TextInputContents::default(),
                TextInputBuffer::default(),
                queue,
            ))
            .id();
        focus.active_input.set(ent);
    }
}
pub fn text_send(
    mut msg: MessageReader<SubmitText>,
    net: Net,
    chat: Single<Entity, With<TextChat>>,
    mut commands: Commands,
    peers: Res<Peers>,
) {
    for msg in msg.read() {
        if msg.text.is_empty() {
            return;
        }
        if let Some(name) = &peers.name {
            net.text(msg.text.clone());
            spawn_msg(*chat, name.clone(), msg.text.clone(), &mut commands);
        }
    }
}
pub fn spawn_msg(entity: Entity, name: String, msg: String, commands: &mut Commands) {
    commands
        .entity(entity)
        .with_child((
            Node {
                width: Val::Percent(100.0),
                ..default()
            },
            Text(format!("{name}: {msg}")),
            Visibility::Inherited,
            TextFont {
                font_size: FONT_SIZE,
                ..default()
            },
        ))
        .insert(ScrollToBottom);
}
#[derive(Component)]
pub struct ScrollToBottom;
pub fn scroll_to_bottom(mut commands: Commands, query: Query<Entity, With<ScrollToBottom>>) {
    for entity in &query {
        commands.trigger(Scroll {
            entity,
            delta: Vec2::new(0.0, f32::INFINITY),
        });
        commands.entity(entity).remove::<ScrollToBottom>();
    }
}
pub fn text_keybinds(
    mut active_input: ResMut<InputFocus>,
    text: Single<Entity, With<TextInput>>,
    menu: Res<Menu>,
    keybinds: Keybinds,
) {
    if !matches!(*menu, Menu::World) || !keybinds.just_pressed(Keybind::Chat) {
        return;
    }
    active_input.set(*text);
}
#[derive(Default, Debug, Component, Deref, DerefMut)]
pub struct ShapeHold(pub Option<usize>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct VoiceActive(pub bool);
#[cfg(feature = "mic")]
pub fn voice_keybinds(
    keybinds: Keybinds,
    mut active: ResMut<VoiceActive>,
    audio: Res<AudioResource>,
) {
    let new = keybinds.pressed(Keybind::Voice);
    if new != **active {
        **active = new;
        audio.stop(!new)
    }
}
#[cfg(feature = "mic")]
pub fn voice_chat(net: Net, audio: Res<AudioResource>) {
    audio.try_recv_audio(|data| net.voice(data))
}
pub fn turn_keybinds(
    others_ids: Query<&SyncObject>,
    mut shape: Query<(&mut Shape, Entity)>,
    mut transforms: Query<&mut Transform, (Or<(With<Pile>, With<Shape>)>, Without<Side>)>,
    mut net: Net,
    mut turn: ResMut<Turn>,
    peers: Res<Peers>,
    keybinds: Keybinds,
) {
    if keybinds.just_pressed(Keybind::PassTurn) || keybinds.just_pressed(Keybind::TakeTurn) {
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
        if keybinds.just_pressed(Keybind::TakeTurn) {
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
    transforms: &mut Query<&mut Transform, (Or<(With<Pile>, With<Shape>)>, Without<Side>)>,
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
    transforms: &mut Query<&mut Transform, (Or<(With<Pile>, With<Shape>)>, Without<Side>)>,
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
    while (!peers.is_empty() && peers.iter().all(|(_, b)| *b != turn.0))
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
#[derive(Component)]
pub struct CounterMenu(Entity, Value, Option<Counter>);
#[derive(Component)]
pub struct TempDisable;
#[derive(Component)]
pub struct SearchText;
#[derive(Debug, PartialEq)]
pub enum SpotType {
    CommanderMain,
    CommanderAlt,
    Exile,
    Main,
    Graveyard,
}
#[derive(Component, Debug)]
pub struct CardSpot {
    pub spot_type: SpotType,
    pub ent: Option<Entity>,
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
    keybinds: Keybinds,
    mut ents: Single<&mut Visibility, (With<EscMenu>, Without<TextMenu>)>,
    mut other_ents: Single<&mut Visibility, (With<TextMenu>, Without<EscMenu>)>,
    mut menu: ResMut<Menu>,
    side: Option<Single<Entity, With<SideMenu>>>,
    counter: Option<Single<Entity, With<CounterMenu>>>,
    text: Query<Entity, With<TextInputContents>>,
    hover_map: Res<HoverMap>,
    mut active_input: ResMut<InputFocus>,
) {
    if keybinds.just_pressed(Keybind::Menu)
        || (keybinds.just_pressed(Keybind::CalcClose) && matches!(*menu, Menu::Counter))
    {
        if let Some(e) = side {
            commands.entity(*e).despawn()
        }
        if let Some(e) = counter {
            commands.entity(*e).despawn()
        }
        let (new, old) = if matches!(*menu, Menu::Esc | Menu::Side | Menu::Counter) {
            *menu = Menu::World;
            (Visibility::Hidden, Visibility::Visible)
        } else {
            active_input.clear();
            *menu = Menu::Esc;
            (Visibility::Visible, Visibility::Hidden)
        };
        **ents = new;
        **other_ents = old;
    }
    if keybinds.just_pressed(Keybind::Select) {
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
    //TODO this could be used less
    if pile.is_empty() {
        *menu = Menu::World;
        commands.entity(**side.as_ref().unwrap()).despawn();
        return;
    }
    let mut search = commands.entity(search);
    search.despawn_children();
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
    keybinds: Keybinds,
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
            Without<Side>,
        ),
    >,
    (mut colliders, follow, ids, others_ids, text, equipment, counters, side, mut net, mut focus): (
        Query<&mut Collider>,
        Option<Single<Entity, With<FollowMouse>>>,
        Query<&SyncObjectMe>,
        Query<&SyncObject>,
        Single<&TextInputContents, With<SearchText>>,
        Query<(), With<Equipment>>,
        Query<&Counter>,
        Option<Single<Entity, With<SideMenu>>>,
        Net,
        Focus,
    ),
) {
    let left = keybinds.just_pressed(Keybind::Select);
    let swap = keybinds.just_pressed(Keybind::Transform) && !focus.key_lock();
    if !matches!(*focus.menu, Menu::Side)
        || !(left || swap || (follow.is_some() && !keybinds.pressed(Keybind::Select)))
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
                                Some(&counters),
                                &mut commands,
                            );
                        } else {
                            if let Ok(id) = ids.get(entity) {
                                net.killed_me(*id)
                            } else if let Ok(id) = others_ids.get(entity) {
                                net.killed(*id);
                            }
                            commands.entity(entity).despawn();
                        }
                        let mut transform = *trans;
                        transform.translation.y += len + LIFT_SPACE;
                        if let Some(e) = &follow {
                            remove_follow(&mut commands, **e);
                        }
                        let id = net.new_id();
                        let uuid = new.id;
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
                                vec![(id, Trans::from_transform(&transform), uuid)],
                                card.0 + 1,
                            );
                        } else if let Ok(oid) = others_ids.get(entity) {
                            net.draw(
                                *oid,
                                vec![(id, Trans::from_transform(&transform), uuid)],
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
                            &mut focus.menu,
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
                        if let Ok(id) = ids.get(search_deck.1.0) {
                            net.flip_me(*id, card.0, inner_card.flipped);
                        } else if let Ok(id) = others_ids.get(search_deck.1.0) {
                            net.flip(*id, card.0, inner_card.flipped);
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
                    if let Ok(tid) = ids.get(**e) {
                        if let Ok(id) = ids.get(search_deck.1.0) {
                            net.merge_me(*id, *tid, drop.unwrap_or(pile.len()));
                        } else if let Ok(id) = others_ids.get(search_deck.1.0) {
                            net.merge_them(*id, *tid, drop.unwrap_or(pile.len()));
                        }
                    }
                    commands.entity(**e).despawn();
                    update_search(
                        &mut commands,
                        search_deck.0,
                        &pile,
                        &trans,
                        text.get(),
                        &side,
                        &mut focus.menu,
                    );
                }
            }
        }
    }
}
#[derive(Component, Deref, DerefMut)]
pub struct TargetCard(pub usize);
#[derive(Component, Deref, DerefMut)]
pub struct SearchDeck(pub Entity);
pub fn update_search_deck(
    mut commands: Commands,
    text: Single<&TextInputContents, Changed<TextInputContents>>,
    single: Option<Single<(Entity, &SearchDeck)>>,
    query: Query<&Transform>,
    mut menu: ResMut<Menu>,
    counter: Option<Single<&CounterMenu>>,
    mut text3d: Query<&mut Text3d>,
    children: Query<&Children>,
    mut obj: Query<(Option<&mut Shape>, Option<&mut Pile>)>,
    ids: Query<&SyncObjectMe>,
    other_ids: Query<&SyncObject>,
    side: Option<Single<Entity, With<SideMenu>>>,
    net: Net,
) {
    match *menu {
        Menu::Side => {
            if let Some(single) = single {
                let transform = query.get(single.1.0).unwrap();
                let pile = obj.get(single.1.0).unwrap().1.unwrap();
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
                            Number::<I, F, C>::from_f64(counter.1.0 as f64, &Options::default())
                                .into(),
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
                && let Ok(childs) = children.get(counter.0)
                && let Ok((v, pile)) = obj.get_mut(counter.0)
            {
                let Some(v) = v
                    .and_then(|s| match s.into_inner() {
                        Shape::Counter(v, _) => Some(v),
                        _ => None,
                    })
                    .or_else(|| {
                        pile.and_then(|c| match c.into_inner() {
                            Pile::Single(c) => Some(c),
                            _ => None,
                        })
                        .and_then(|card| match counter.2 {
                            Some(Counter::Power) => card.power.as_mut(),
                            Some(Counter::Toughness) => card.toughness.as_mut(),
                            Some(Counter::Loyalty) => card.loyalty.as_mut(),
                            Some(Counter::Counters) => card.counters.as_mut(),
                            Some(Counter::Misc) => card.misc.as_mut(),
                            None => None,
                        })
                    })
                else {
                    return;
                };
                v.0 = n.number.real().to_f64().round() as i128;
                for ent in childs {
                    if let Some(c) = counter.2 {
                        if let Ok(children) = children.get(*ent) {
                            for ent in children {
                                if let Ok(mut text) = text3d.get_mut(*ent) {
                                    *text.get_single_mut().unwrap() = v.0.to_string();
                                    if let Ok(id) = ids.get(counter.0) {
                                        net.modify_me(*id, c, Some(v.clone()));
                                    } else if let Ok(id) = other_ids.get(counter.0) {
                                        net.modify(*id, c, Some(v.clone()));
                                    }
                                    return;
                                }
                            }
                        }
                    } else if let Ok(mut text) = text3d.get_mut(*ent) {
                        *text.get_single_mut().unwrap() = v.0.to_string();
                        if let Ok(id) = ids.get(counter.0) {
                            net.counter_me(*id, v.clone());
                        } else if let Ok(id) = other_ids.get(counter.0) {
                            net.counter(*id, v.clone());
                        }
                        return;
                    }
                }
            }
        }
        _ => {}
    }
}
pub fn cam_translation(
    keybinds: Keybinds,
    mouse_motion: Res<AccumulatedMouseScroll>,
    mut cam: Single<&mut Transform, With<Camera3d>>,
    peers: Res<Peers>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window, With<PrimaryWindow>>,
    focus: Focus,
) {
    if focus.key_lock() {
        return;
    }
    let scale = CARD_THICKNESS * 16.0;
    let apply = |translate: Vec3, cam: &mut Transform| {
        let mut norm = translate.normalize();
        norm.y = 0.0;
        let abs = norm.length();
        if abs != 0.0 {
            let translate = norm * translate.length() / abs;
            cam.translation += translate;
        }
    };
    if !keybinds
        .keyboard
        .any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
    {
        if keybinds.pressed(Keybind::Up) {
            let translate = cam.forward().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        if keybinds.pressed(Keybind::Left) {
            let translate = cam.left().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        if keybinds.pressed(Keybind::Right) {
            let translate = cam.right().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        if keybinds.pressed(Keybind::Down) {
            let translate = cam.back().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        let scale = scale * 4.0;
        if keybinds.pressed(Keybind::UpFast) {
            let translate = cam.forward().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        if keybinds.pressed(Keybind::LeftFast) {
            let translate = cam.left().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        if keybinds.pressed(Keybind::RightFast) {
            let translate = cam.right().as_vec3() * scale;
            apply(translate, &mut cam)
        }
        if keybinds.pressed(Keybind::DownFast) {
            let translate = cam.back().as_vec3() * scale;
            apply(translate, &mut cam)
        }
    }
    if mouse_motion.delta.y != 0.0 && !focus.mouse_lock() {
        let mut translate = cam.forward().as_vec3() * scale * mouse_motion.delta.y * 16.0;
        if mouse_motion.unit != MouseScrollUnit::Line {
            translate /= 4.0;
        }
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
    if keybinds.just_pressed(Keybind::Reset) {
        *cam.into_inner() = default_cam_pos(peers.me.unwrap_or_default());
    }
}
pub fn cam_rotation(
    keybinds: Keybinds,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut cam: Single<(&mut Transform, &Camera, &GlobalTransform), With<Camera3d>>,
    window: Single<&Window, With<PrimaryWindow>>,
    focus: Focus,
) {
    if focus.mouse_lock() {
        return;
    }
    if keybinds.pressed(Keybind::Rotate) && mouse_motion.delta != Vec2::ZERO {
        let Ok(ray) = cam.1.viewport_to_world(cam.2, window.size() / 2.0) else {
            return;
        };
        let delta_yaw = -mouse_motion.delta.x * 0.001;
        let delta_pitch = -mouse_motion.delta.y * 0.001;
        let (yaw, pitch, roll) = cam.0.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;
        let pitch = (pitch + delta_pitch)
            .max((-PI / 2.0).next_up())
            .min(-PI / 12.0);
        cam.0.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
        let Some(time) = ray.intersect_plane(Vec3::default(), InfinitePlane3d { normal: Dir3::Y })
        else {
            return;
        };
        let orig = cam.0.translation + ray.direction * time;
        cam.0.translation = orig - cam.0.rotation * Dir3::NEG_Z * time;
    }
}
pub fn listen_for_deck(
    keybinds: Keybinds,
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
    mut net: Net,
    focus: Focus,
) {
    if focus.key_lock() {
        return;
    }
    if keybinds.just_pressed(Keybind::Paste) || keybinds.just_pressed(Keybind::PasteObject) {
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
        if !keybinds.just_pressed(Keybind::PasteObject) {
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
                        Color::WHITE,
                    )
                    .insert(net.new_id())
                    .id(),
            ),
            GameClipboard::None => None,
        } {
            commands.trigger(MoveUp(ent))
        }
    }
}
pub fn register_deck(
    mut commands: Commands,
    decks: ResMut<Download>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    card_base: Res<CardBase>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut spots: Query<(&GlobalTransform, &mut CardSpot, &Player)>,
    peers: Res<Peers>,
    mut trans: Query<Entity, With<Pile>>,
    (ids, others_ids, search_deck, mut net, mut cards): (
        Query<&SyncObjectMe>,
        Query<&SyncObject>,
        Option<Single<(Entity, &SearchDeck)>>,
        Net,
        ResMut<CardList>,
    ),
) {
    let mut decks = decks.get_deck.0.lock().unwrap();
    for (mut deck, deck_type) in decks.drain(..) {
        for c in deck.iter() {
            if let Entry::Vacant(v) = cards.entry(c.id) {
                v.insert(c.data.clone());
            }
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
            DeckType::CommanderAlt => {
                let spot = spots
                    .iter_mut()
                    .filter(|(_, _, p)| p.0 == peers.me.unwrap_or(0))
                    .find(|(_, s, _)| matches!(s.spot_type, SpotType::CommanderAlt))
                    .unwrap();
                let trans = rem(spot);
                let v = Vec2::new(trans.x, trans.z);
                vec2_to_ground(&deck, v, rev)
            }
            DeckType::Companion => {
                let spot = spots
                    .iter_mut()
                    .filter(|(_, _, p)| p.0 == peers.me.unwrap_or(0))
                    .find(|(_, s, _)| matches!(s.spot_type, SpotType::CommanderAlt))
                    .unwrap();
                let trans = spot.0.translation();
                let v = Vec2::new(trans.x + CARD_WIDTH + MAT_BAR, trans.z);
                vec2_to_ground(&deck, v, rev)
            }
            DeckType::Attraction => {
                let spot = spots
                    .iter_mut()
                    .filter(|(_, _, p)| p.0 == peers.me.unwrap_or(0))
                    .find(|(_, s, _)| matches!(s.spot_type, SpotType::Exile))
                    .unwrap();
                let trans = spot.0.translation();
                let v = Vec2::new(trans.x + CARD_WIDTH + MAT_BAR, trans.z);
                vec2_to_ground(&deck, v, rev)
            }
            DeckType::Sticker => {
                let spot = spots
                    .iter_mut()
                    .filter(|(_, _, p)| p.0 == peers.me.unwrap_or(0))
                    .find(|(_, s, _)| matches!(s.spot_type, SpotType::Main))
                    .unwrap();
                let trans = spot.0.translation();
                let v = Vec2::new(trans.x + CARD_WIDTH + MAT_BAR, trans.z);
                vec2_to_ground(&deck, v, rev)
            }
            DeckType::Commander => {
                let spot = spots
                    .iter_mut()
                    .filter(|(_, _, p)| p.0 == peers.me.unwrap_or(0))
                    .find(|(_, s, _)| matches!(s.spot_type, SpotType::CommanderMain))
                    .unwrap();
                let trans = rem(spot);
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
                let v = Vec2::new(trans.x + CARD_WIDTH + MAT_BAR, trans.z);
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
            let id = ent.id();
            commands.trigger(MoveUp(id));
        }
        if let Some(id) = id {
            net.sent.del(id);
            net.received(id.user, id.id);
        }
    }
}
pub fn give_ents(to_do: Res<GiveEnts>, ents: Query<(&SyncObject, Entity)>, mut net: Net) {
    for peer in to_do.0.lock().unwrap().drain(..) {
        for (id, ent) in ents {
            if id.user == peer {
                net.take(ent, *id);
            }
        }
    }
}
#[derive(Resource, Default, Deref, DerefMut)]
pub struct GiveEnts(pub Arc<Mutex<Vec<PeerId>>>);
#[derive(Resource, Default, Deref, DerefMut)]
pub struct FlipCounter(pub Arc<Mutex<Vec<(usize, bool)>>>);
pub fn flip_ents(
    to_do: Res<FlipCounter>,
    others_ids: Query<&SyncObject>,
    mut shape: Query<(&mut Shape, Entity)>,
    mut transforms: Query<&mut Transform, (Or<(With<Pile>, With<Shape>)>, Without<Side>)>,
    mut net: Net,
    mut turn: ResMut<Turn>,
    peers: Res<Peers>,
) {
    for (id, up) in to_do.0.lock().unwrap().drain(..) {
        if let Some(ent) = shape.iter().find_map(|(s, e)| {
            if let Shape::Counter(_, v) = s
                && *v == id
            {
                Some(e)
            } else {
                None
            }
        }) {
            if let Ok(id) = others_ids.get(ent) {
                net.take(ent, *id);
            }
            let mut t = transforms.get_mut(ent).unwrap();
            if up == is_reversed(&t) {
                t.rotate_local_z(PI);
                if !up && turn.0 == id {
                    let map = peers.map();
                    if let Some(ent) = shape.iter().find_map(|(s, e)| {
                        if let Shape::Turn(v) = s
                            && *v == id
                        {
                            Some(e)
                        } else {
                            None
                        }
                    }) {
                        let mut flip = true;
                        next_turn(
                            others_ids,
                            &mut shape,
                            &mut transforms,
                            &mut net,
                            &mut turn,
                            ent,
                            &mut flip,
                            &map,
                        );
                        if flip {
                            if let Ok(id) = others_ids.get(ent) {
                                net.take(ent, *id);
                            }
                            let mut transform = transforms.get_mut(ent).unwrap();
                            transform.rotation = Quat::default();
                            *transform = transform.looking_to(Dir3::Z, Dir3::NEG_Y);
                        }
                    }
                }
            }
        }
    }
}
pub fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keybinds: Keybinds,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);
        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= 128.0;
        }
        if keybinds
            .keyboard
            .any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        {
            mem::swap(&mut delta.x, &mut delta.y);
        }
        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(Scroll { entity, delta });
            }
        }
    }
}
pub fn search(
    entity: Entity,
    pile: &Pile,
    transform: &Transform,
    side: &Option<Single<Entity, With<SideMenu>>>,
    commands: &mut Commands,
    active_input: &mut InputFocus,
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
        BackgroundColor(Color::srgba_u8(0, 0, 0, 127)),
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
                    font_size: FONT_SIZE,
                    ..default()
                },
                TextInputContents::default(),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(5.0),
                    ..default()
                },
                SearchText,
            ))
            .id();
        active_input.set(id);
        search = Some(
            p.spawn((
                SearchDeck(entity),
                Node {
                    display: Display::Grid,
                    position_type: PositionType::Absolute,
                    top: Val::Percent(5.0),
                    left: Val::Px(0.0),
                    height: Val::Percent(95.0),
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
            &SpatialQueryFilter::from_mask(u32::MAX - 0b100),
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
                    t.translation = transform.translation;
                    t.rotation = transform.rotation;
                    if rev {
                        t.rotate_local_z(PI);
                    }
                }
                spot.ent = Some(ent);
                break;
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
