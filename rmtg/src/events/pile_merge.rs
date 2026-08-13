use crate::pile::Pile;
use bevy::prelude::Event;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::On;
use bevy_ecs::system::Commands;
#[derive(Event)]
pub struct PileMerge {
    pub from: Pile,
    pub to: Entity,
}
pub fn on_pile_merge(event: On<PileMerge>) {
    _ = event;
}
pub fn trigger_pile_merge(mut commands: Commands) {
    _ = &mut commands;
}
