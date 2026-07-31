use crate::events::hover::Hovered;
use crate::events::move_up::MoveUp;
use crate::keybinds::{Keybind, Keybinds};
use crate::physics::GRAVITY;
use crate::spatial::Spatial;
use avian3d::prelude::{GravityScale, RigidBody};
use bevy::math::{Dir3, Vec3};
use bevy::prelude::{Commands, Component, Entity, InfinitePlane3d, Local, Query, Transform, With};
use rustc_hash::FxHashSet;
#[derive(Component)]
pub struct Dragged;
#[allow(clippy::implicit_hasher)]
pub fn drag(
    hovered: Query<(Entity, &mut Transform), With<Hovered>>,
    mut commands: Commands,
    keybinds: Keybinds,
    spatial: Spatial,
    mut gravity: Query<&mut GravityScale>,
    mut last: Local<Vec3>,
    mut last_ents: Local<FxHashSet<Entity>>,
) {
    if hovered.is_empty() {
        return;
    }
    if keybinds.just_pressed(Keybind::Select) && !keybinds.just_pressed(Keybind::HoldSelect) {
        let Some((_, pos)) = spatial.ray() else {
            return;
        };
        *last = pos;
    } else if keybinds.pressed(Keybind::Select) && !keybinds.pressed(Keybind::HoldSelect) {
        let Some(ray) = spatial.cam_ray() else {
            return;
        };
        let Some(delta) = ray.intersect_plane(*last, InfinitePlane3d::new(Dir3::Y)) else {
            return;
        };
        let pos = ray.origin + ray.direction * delta;
        let delta = pos - *last;
        for (ent, mut t) in hovered {
            t.translation += delta;
            commands.trigger(MoveUp::new(ent));
            if let Ok(mut grav) = gravity.get_mut(ent) {
                commands.entity(ent).insert((RigidBody::Static, Dragged));
                grav.0 = 0.0;
            }
            last_ents.insert(ent);
        }
        *last = pos;
    } else {
        for ent in last_ents.drain() {
            if let Ok(mut grav) = gravity.get_mut(ent) {
                commands
                    .entity(ent)
                    .insert(RigidBody::Dynamic)
                    .remove::<Dragged>();
                grav.0 = GRAVITY;
            }
        }
    }
}
