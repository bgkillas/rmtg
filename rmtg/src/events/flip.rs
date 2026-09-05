use crate::events::hover::HoveredObject;
use crate::keybinds::Keybind;
use crate::pile::Pile;
use bevy::input::ButtonInput;
use bevy::prelude::Transform;
use bevy_ecs::change_detection::Res;
use bevy_ecs::prelude::{Query, With};
use bevy_query_fn_macro::query_fn;
use std::f32::consts::PI;
#[query_fn]
pub fn trigger_flip(
    hovered: Query<&mut Transform, (With<HoveredObject>, With<Pile>)>,
    keybinds: Res<ButtonInput<Keybind>>,
) {
    if keybinds.just_pressed(Keybind::Flip) {
        for mut obj in hovered {
            obj.rotate_local_z(PI);
        }
    }
}
