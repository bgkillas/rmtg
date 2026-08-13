use crate::events::hover::HoveredObject;
use crate::keybinds::Keybind;
use bevy::input::ButtonInput;
use bevy::prelude::EntityEvent;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::On;
use bevy_ecs::prelude::{Commands, Query, With};
use bevy_ecs::system::Res;
#[derive(EntityEvent)]
pub struct Delete {
    pub entity: Entity,
}
impl Delete {
    #[must_use]
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}
pub fn do_delete(
    hovered: Query<Entity, With<HoveredObject>>,
    mut commands: Commands,
    keybinds: Res<ButtonInput<Keybind>>,
) {
    if keybinds.just_pressed(Keybind::Remove) {
        for ent in hovered {
            commands.trigger(Delete::new(ent));
        }
    }
}
pub fn on_delete(event: On<Delete>, mut commands: Commands) {
    commands.entity(event.entity).despawn();
}
