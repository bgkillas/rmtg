use crate::events::hover::HoveredObject;
use crate::keybinds::Keybind;
use crate::pile::{FlippedState, Pile, TapState};
use crate::ui::alt_menu::{AltMenu, RotateUi};
use avian3d::parry::glamx::Vec3;
use bevy::input::ButtonInput;
use bevy::math::Quat;
use bevy::prelude::Transform;
use bevy_ecs::change_detection::Res;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::prelude::{Query, With};
use bevy_ecs::system::{Commands, Single};
use bevy_query_fn_macro::query_fn;
use std::f32::consts::PI;
#[derive(Event)]
pub struct Tapped {
    pub entity: Entity,
    pub state: TapState,
}
impl TapState {
    pub fn set_state(self, rot: &mut Quat) {
        let forward = *rot * Vec3::Z;
        let delta = if forward.x.is_sign_positive() {
            1.0 - forward.z
        } else {
            forward.z - 1.0
        }
        .round();
        *rot = Quat::from_rotation_y(delta * PI / 2.0);
    }
}
#[query_fn]
pub fn trigger_tap(
    hovered: Query<(Entity, &mut Transform), (With<HoveredObject>, With<Pile>)>,
    keybinds: Res<ButtonInput<Keybind>>,
    altmenu: Option<Single<Entity, With<AltMenu>>>,
    mut commands: Commands,
) {
    let left = keybinds.just_pressed(Keybind::TapLeft);
    let right = keybinds.just_pressed(Keybind::TapRight);
    if (left || right)
        && let Some(entity) = altmenu.as_deref().copied()
    {
        if left && right {
            return;
        }
        commands.write_message(RotateUi { entity, right });
        return;
    }
    for mut obj in hovered {
        let state = FlippedState::from(obj.transform.rotation).flipped();
        match (left, right) {
            (true, false) => {
                obj.transform
                    .rotate_local_y(if state { -PI / 2.0 } else { PI / 2.0 });
            }
            (false, true) => {
                obj.transform
                    .rotate_local_y(if state { PI / 2.0 } else { -PI / 2.0 });
            }
            _ => continue,
        }
        let state = TapState::from(obj.transform.rotation);
        state.set_state(&mut obj.transform.rotation);
        commands.trigger(Tapped {
            entity: obj.entity,
            state,
        });
    }
}
