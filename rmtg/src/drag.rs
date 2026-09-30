use crate::events::gravity::NewGravity;
use crate::events::hover::{BoxSelect, HoveredObject};
use crate::keybinds::Keybind;
use crate::physics::{GRAVITY, LIN_DAMPING, WorldLayer};
use crate::spatial::Spatial;
use crate::startup::wall_aabb;
use crate::{CARD_THICKNESS, CARD_WIDTH};
use avian3d::prelude::{
    CollisionLayers, LayerMask, LinearDamping, LinearVelocity, SleepingDisabled,
};
use bevy::input::ButtonInput;
use bevy::math::{Dir3, Vec3};
use bevy::prelude::{
    Commands, Component, Entity, InfinitePlane3d, Local, Query, Res, Transform, With,
};
use bevy::time::Time;
use bevy_ecs::event::Event;
use bevy_ecs::observer::On;
use bevy_ecs::query::Without;
use bevy_ecs::system::Single;
use bevy_query_fn_macro::query_fn;
#[derive(Component, Clone, Debug)]
pub struct Dragging {
    pub pos: Vec3,
}
#[derive(Event)]
pub struct DragEntity {
    pub entity: Entity,
}
#[query_fn]
pub fn on_drag(event: On<DragEntity>) {
    _ = event;
}
#[derive(Event)]
pub struct StopDragEntity {
    pub entity: Entity,
}
#[query_fn]
pub fn stop_drag(event: On<DragEntity>) {
    _ = event;
}
#[query_fn]
pub fn drag(
    box_select: Option<Single<(), With<BoxSelect>>>,
    mut hovered_entities: Query<
        (
            Entity,
            &Transform,
            &mut LinearVelocity,
            Option<&mut Dragging>,
        ),
        With<HoveredObject>,
    >,
    mut velocity: Query<&mut LinearVelocity, Without<HoveredObject>>,
    mut commands: Commands,
    keybinds: Res<ButtonInput<Keybind>>,
    spatial: Spatial,
    mut last: Local<Vec3>,
    last_ents: Query<Entity, With<Dragging>>,
    time: Res<Time>,
) {
    if box_select.is_some() {
        return;
    }
    if hovered_entities.is_empty() {
        for ent in last_ents {
            if let Ok(mut query) = velocity.get_mut(ent) {
                query.y = 0.0;
                commands.trigger(NewGravity::new(ent, GRAVITY));
                commands
                    .entity(ent)
                    .remove::<(Dragging, SleepingDisabled)>()
                    .insert((
                        CollisionLayers::new(WorldLayer::Default, LayerMask::ALL),
                        LinearDamping(LIN_DAMPING),
                    ));
            }
        }
        return;
    }
    if keybinds.just_pressed(Keybind::Select) {
        let Some((_, pos, _)) = spatial.ray() else {
            return;
        };
        *last = pos;
        for ent in last_ents {
            commands.trigger(NewGravity::new(ent, GRAVITY));
            commands
                .entity(ent)
                .remove::<(Dragging, SleepingDisabled)>()
                .insert((
                    CollisionLayers::new(WorldLayer::Default, LayerMask::ALL),
                    LinearDamping(LIN_DAMPING),
                ));
        }
        return;
    }
    if keybinds.pressed(Keybind::Select) || keybinds.pressed(Keybind::Draw) {
        let Some(ray) = spatial.cam_ray() else {
            return;
        };
        let Some(delta) = ray.intersect_plane(*last, InfinitePlane3d::new(Dir3::Y)) else {
            return;
        };
        let pos = ray.origin + ray.direction * delta;
        let delta = pos - *last;
        for mut hovered in hovered_entities {
            let target = if let Some(mut target) = hovered.dragging {
                target.pos += delta;
                target.pos
            } else if keybinds.just_pressed(Keybind::Select) {
                commands.trigger(NewGravity::new(hovered.entity, 0.0));
                let mut pos = hovered.transform.translation + delta;
                pos.y += CARD_WIDTH;
                commands.entity(hovered.entity).insert((
                    Dragging { pos },
                    LinearDamping(0.0),
                    CollisionLayers::NONE,
                    SleepingDisabled,
                ));
                pos
            } else {
                return;
            };
            let delta =
                Vec3::from(wall_aabb().closest_point(target)) - hovered.transform.translation;
            if delta.y <= CARD_THICKNESS / 8.0 {
                commands
                    .entity(hovered.entity)
                    .insert(CollisionLayers::new(WorldLayer::Default, LayerMask::ALL));
            }
            hovered.linear_velocity.0 = delta / (time.delta_secs() * 4.0);
        }
        *last = pos;
    } else {
        for ent in last_ents {
            if let Ok(mut query) = hovered_entities.get_mut(ent) {
                query.linear_velocity.y = 0.0;
            }
            if let Ok(mut query) = velocity.get_mut(ent) {
                query.y = 0.0;
            }
            commands.trigger(NewGravity::new(ent, GRAVITY));
            commands
                .entity(ent)
                .remove::<(Dragging, SleepingDisabled)>()
                .insert((
                    LinearDamping(LIN_DAMPING),
                    CollisionLayers::new(WorldLayer::Default, LayerMask::ALL),
                ));
        }
    }
}
