use crate::CARD_THICKNESS;
use crate::events::gravity::NewGravity;
use crate::events::hover::Hovered;
use crate::keybinds::{Keybind, Keybinds};
use crate::physics::GRAVITY;
use crate::spatial::Spatial;
use avian3d::prelude::{LinearVelocity};
use bevy::math::{Dir3, Vec3};
use bevy::prelude::{
    Commands, Component, Entity, InfinitePlane3d, Local, Query, Res, Transform, With,
};
use bevy::time::Time;
use rustc_hash::FxBuildHasher;
use std::collections::HashSet;
#[derive(Component, Clone)]
pub struct TargetPosition {
    pub pos: Vec3,
}
pub fn drag(
    hovered: Query<
        (
            Entity,
            &Transform,
            &mut LinearVelocity,
            Option<&mut TargetPosition>,
        ),
        With<Hovered>,
    >,
    mut commands: Commands,
    keybinds: Keybinds,
    spatial: Spatial,
    mut last: Local<Vec3>,
    mut last_ents: Local<HashSet<Entity, FxBuildHasher>>,
    time: Res<Time>,
) {
    if hovered.is_empty() {
        for ent in last_ents.drain() {
            commands.trigger(NewGravity::new(ent, GRAVITY));
            commands.entity(ent).remove::<TargetPosition>();
        }
        return;
    }
    if keybinds.just_pressed(Keybind::Select) && !keybinds.just_pressed(Keybind::HoldSelect) {
        let Some((_, pos)) = spatial.ray() else {
            return;
        };
        *last = pos;
    }
    if keybinds.pressed(Keybind::Select) && !keybinds.pressed(Keybind::HoldSelect) {
        let Some(ray) = spatial.cam_ray() else {
            return;
        };
        let Some(delta) = ray.intersect_plane(*last, InfinitePlane3d::new(Dir3::Y)) else {
            return;
        };
        let pos = ray.origin + ray.direction * delta;
        let delta = pos - *last;
        for (ent, t, mut vel, opt_target) in hovered {
            let target = if let Some(mut target) = opt_target {
                target.pos += delta;
                target.pos
            } else {
                commands.trigger(NewGravity::new(ent, 0.0));
                let mut pos = t.translation + delta;
                pos.y += 4.0 * CARD_THICKNESS;
                commands.entity(ent).insert(TargetPosition { pos });
                pos
            };
            let delta = target - t.translation;
            vel.0 = delta / time.delta_secs() * 1.0 / 8.0;
            last_ents.insert(ent);
        }
        *last = pos;
    } else {
        for ent in last_ents.drain() {
            commands.trigger(NewGravity::new(ent, GRAVITY));
            commands.entity(ent).remove::<TargetPosition>();
        }
    }
}
