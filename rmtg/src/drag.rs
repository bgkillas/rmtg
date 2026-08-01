use crate::events::hover::Hovered;
use crate::keybinds::{Keybind, Keybinds};
use crate::physics::GRAVITY;
use crate::spatial::Spatial;
use avian3d::prelude::{GravityScale, LinearVelocity};
use bevy::math::{Dir3, Vec3};
use bevy::prelude::{Entity, InfinitePlane3d, Local, Query, Res, Transform, With};
use bevy::time::Time;
use rustc_hash::FxHashSet;
#[allow(clippy::implicit_hasher)]
pub fn drag(
    hovered: Query<(Entity, &Transform, &mut LinearVelocity), With<Hovered>>,
    keybinds: Keybinds,
    spatial: Spatial,
    mut gravity: Query<&mut GravityScale>,
    mut last: Local<Vec3>,
    mut last_ents: Local<FxHashSet<Entity>>,
    time: Res<Time>,
) {
    if hovered.is_empty() {
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
        for (ent, t, mut vel) in hovered {
            let delta = pos - t.translation;
            vel.0 = delta / time.delta_secs() * 1.0 / 32.0;
            if let Ok(mut grav) = gravity.get_mut(ent) {
                grav.0 = 0.0;
            }
            last_ents.insert(ent);
        }
        *last = pos;
    } else {
        for ent in last_ents.drain() {
            if let Ok(mut grav) = gravity.get_mut(ent) {
                grav.0 = GRAVITY;
            }
        }
    }
}
