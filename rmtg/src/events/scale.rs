use crate::events::hover::HoveredObject;
use crate::events::move_up::MoveUp;
use crate::keybinds::Keybind;
use bevy::input::ButtonInput;
use bevy::prelude::{EntityEvent, Transform};
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::On;
use bevy_ecs::query::With;
use bevy_ecs::system::{Commands, Query, Res};
#[derive(EntityEvent)]
pub struct Scale {
    pub entity: Entity,
    pub up: bool,
}
pub fn on_scale(event: On<Scale>, mut transforms: Query<&mut Transform>, mut commands: Commands) {
    const SCALE: f32 = 1.25;
    let mut transform = transforms.get_mut(event.entity).unwrap();
    transform.scale *= if event.up { SCALE } else { 1.0 / SCALE };
    commands.trigger(MoveUp::new(event.entity));
}
pub fn update_scale(
    mut commands: Commands,
    keybinds: Res<ButtonInput<Keybind>>,
    query: Query<Entity, With<HoveredObject>>,
) {
    let up = keybinds.just_pressed(Keybind::ScaleUp);
    let down = keybinds.just_pressed(Keybind::ScaleDown);
    if up || down {
        for entity in query {
            commands.trigger(Scale { entity, up });
        }
    }
}
