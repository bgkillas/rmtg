use crate::pile::{FlippedState, Pile, TapState};
use avian3d::prelude::Sleeping;
use bevy::prelude::{Event, Transform};
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::On;
use bevy_ecs::query::With;
use bevy_ecs::system::{Commands, Query};
use bevy_query_fn_macro::query_fn;
#[derive(Event)]
pub struct PileMerge {
    pub from: Pile,
    pub to: Entity,
}
pub fn on_pile_merge(event: On<PileMerge>) {
    _ = event;
}
#[query_fn]
pub fn trigger_pile_merge(
    mut commands: Commands,
    piles: Query<(&Pile, &Transform), With<Sleeping>>,
) {
    _ = &mut commands;
    for pile in piles {
        let tap_state = TapState::from(&pile.transform.rotation);
        let flipped_state = FlippedState::from(&pile.transform.rotation);
        _ = tap_state;
        _ = flipped_state;
    }
}
