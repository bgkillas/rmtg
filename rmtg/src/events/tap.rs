use crate::events::hover::HoveredObject;
use crate::keybinds::Keybind;
use crate::pile::{FlippedState, Pile};
use bevy::input::ButtonInput;
use bevy::prelude::Transform;
use bevy_ecs::change_detection::Res;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::prelude::{Query, With};
use bevy_query_fn_macro::query_fn;
use std::f32::consts::PI;
#[derive(Event)]
pub struct Tapped {
    pub entity: Entity,
    pub state: bool,
}
#[query_fn]
pub fn trigger_tap(
    hovered: Query<&mut Transform, (With<HoveredObject>, With<Pile>)>,
    keybinds: Res<ButtonInput<Keybind>>,
) {
    for mut obj in hovered {
        let state = FlippedState::from(obj.rotation).flipped();
        if keybinds.just_pressed(Keybind::TapLeft) {
            obj.rotate_local_y(if state { -PI / 2.0 } else { PI / 2.0 });
        }
        if keybinds.just_pressed(Keybind::TapRight) {
            obj.rotate_local_y(if state { PI / 2.0 } else { -PI / 2.0 });
        }
    }
}
