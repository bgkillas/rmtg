use crate::CARD_THICKNESS;
use crate::events::gravity::NewGravity;
use crate::events::hover::{BoxSelect, HoveredObject};
use crate::keybinds::Keybind;
use crate::physics::{GRAVITY, LIN_DAMPING};
use crate::spatial::Spatial;
use crate::startup::wall_aabb;
use avian3d::prelude::{LinearDamping, LinearVelocity};
use bevy::ecs::entity::EntityHash;
use bevy::input::ButtonInput;
use bevy::math::{Dir3, Vec3};
use bevy::prelude::{
    Commands, Component, Entity, InfinitePlane3d, Local, Query, Res, Transform, With,
};
use bevy::time::Time;
use bevy_ecs::system::Single;
use bevy_query_fn_macro::query_fn;
use std::collections::HashSet;
#[derive(Component, Clone)]
pub struct TargetPosition {
    pub pos: Vec3,
}
#[query_fn]
pub fn drag(
    box_select: Option<Single<(), With<BoxSelect>>>,
    hovered_entities: Query<
        (
            Entity,
            &Transform,
            &mut LinearVelocity,
            Option<&mut TargetPosition>,
        ),
        With<HoveredObject>,
    >,
    mut commands: Commands,
    keybinds: Res<ButtonInput<Keybind>>,
    spatial: Spatial,
    mut last: Local<Vec3>,
    mut last_ents: Local<HashSet<Entity, EntityHash>>,
    time: Res<Time>,
) {
    if box_select.is_some() {
        return;
    }
    if hovered_entities.is_empty() {
        for ent in last_ents.drain() {
            commands.trigger(NewGravity::new(ent, GRAVITY));
            commands
                .entity(ent)
                .remove::<TargetPosition>()
                .insert(LinearDamping(LIN_DAMPING));
        }
        return;
    }
    if keybinds.just_pressed(Keybind::Select) {
        let Some((_, pos, _)) = spatial.ray() else {
            return;
        };
        *last = pos;
        for ent in last_ents.drain() {
            commands.trigger(NewGravity::new(ent, GRAVITY));
            commands
                .entity(ent)
                .remove::<TargetPosition>()
                .insert(LinearDamping(LIN_DAMPING));
        }
        return;
    }
    if keybinds.pressed(Keybind::Select) {
        let Some(ray) = spatial.cam_ray() else {
            return;
        };
        let Some(delta) = ray.intersect_plane(*last, InfinitePlane3d::new(Dir3::Y)) else {
            return;
        };
        let pos = ray.origin + ray.direction * delta;
        let delta = pos - *last;
        for mut hovered in hovered_entities {
            let target = if let Some(mut target) = hovered.target_position {
                target.pos += delta;
                target.pos
            } else {
                last_ents.insert(hovered.entity);
                commands.trigger(NewGravity::new(hovered.entity, 0.0));
                let mut pos = hovered.transform.translation + delta;
                pos.y += 4.0 * CARD_THICKNESS;
                commands
                    .entity(hovered.entity)
                    .insert(TargetPosition { pos })
                    .insert(LinearDamping(0.0));
                pos
            };
            let delta =
                Vec3::from(wall_aabb().closest_point(target)) - hovered.transform.translation;
            hovered.linear_velocity.0 = delta / (time.delta_secs() * 4.0);
        }
        *last = pos;
    } else {
        for ent in last_ents.drain() {
            commands.trigger(NewGravity::new(ent, GRAVITY));
            commands
                .entity(ent)
                .remove::<TargetPosition>()
                .insert(LinearDamping(LIN_DAMPING));
        }
    }
}
