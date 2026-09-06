use crate::events::hover::HoveredObject;
use crate::keybinds::Keybind;
use crate::pile::{FlippedState, Pile, TapState};
use crate::ui::alt_menu::{AltMenu, RotateUi};
use bevy::input::ButtonInput;
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
        commands.trigger(Tapped {
            entity: obj.entity,
            state,
        });
    }
}
