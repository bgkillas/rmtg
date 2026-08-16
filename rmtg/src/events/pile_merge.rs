use crate::drag::TargetPosition;
use crate::events::repaint::Repaint;
use crate::pile::{FlippedState, PendingCards, Pile, TapState};
use avian3d::prelude::CollisionStart;
use bevy::prelude::{Event, Transform};
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::On;
use bevy_ecs::query::Without;
use bevy_ecs::system::{Commands, Query};
use bevy_query_fn_macro::query_fn;
#[derive(Event)]
pub struct PileMerge {
    pub from: Entity,
    pub to: Entity,
}
pub fn on_pile_merge(event: On<PileMerge>, mut commands: Commands) {
    commands.entity(event.from).despawn();
    commands.trigger(Repaint::new(event.to));
}
#[query_fn]
pub fn trigger_pile_merge(
    collision: On<CollisionStart>,
    mut commands: Commands,
    piles: Query<(Entity, &mut Pile, &Transform), (Without<PendingCards>, Without<TargetPosition>)>,
) {
    let Ok(mut pile1) = piles.get(collision.collider1) else {
        return;
    };
    let Ok(mut pile2) = piles.get(collision.collider2) else {
        return;
    };
    if TapState::from(pile1.transform.rotation) != TapState::from(pile2.transform.rotation)
        || FlippedState::from(pile1.transform.rotation)
            != FlippedState::from(pile2.transform.rotation)
    {
        return;
    }
    if pile1.transform.translation.y > pile2.transform.translation.y {
        (pile1, pile2) = (pile2, pile1);
    }
    commands.trigger(PileMerge {
        from: pile2.entity,
        to: pile1.entity,
    });
}
