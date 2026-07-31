use crate::events::hover::Hovered;
use crate::events::move_up::MoveUp;
use crate::keybinds::{Keybind, Keybinds};
use crate::spatial::Spatial;
use bevy::math::{Dir3, Vec3};
use bevy::prelude::{Commands, Entity, InfinitePlane3d, Local, Query, Transform, With};
pub fn drag(
    hovered: Query<(Entity, &mut Transform), With<Hovered>>,
    mut commands: Commands,
    keybinds: Keybinds,
    spatial: Spatial,
    mut last: Local<Vec3>,
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
        }
        *last = pos;
    }
}
