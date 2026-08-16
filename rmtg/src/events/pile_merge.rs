use crate::CARD_WIDTH;
use crate::drag::TargetPosition;
use crate::events::repaint::Repaint;
use crate::pile::{FlippedState, PendingCards, Pile, TapState};
use avian3d::prelude::CollisionStart;
use bevy::math::{Vec3, Vec3Swizzles as _};
use bevy::prelude::{Event, Transform};
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::On;
use bevy_ecs::query::Without;
use bevy_ecs::system::{Commands, Query};
use bevy_query_fn_macro::query_fn;
use std::mem;
#[derive(Event)]
pub struct PileMerge {
    pub from: Entity,
    pub to: Entity,
}
pub fn on_pile_merge(event: On<PileMerge>, mut commands: Commands, mut piles: Query<&mut Pile>) {
    let [mut pile1, mut pile2] = piles.get_many_mut([event.from, event.to]).unwrap();
    pile2.extend(mem::take(&mut pile1));
    commands.trigger(Repaint::new(event.to));
    commands.entity(event.from).despawn();
}
#[query_fn]
pub fn trigger_pile_merge(
    collision: On<CollisionStart>,
    mut commands: Commands,
    piles: Query<(Entity, &mut Pile, &Transform), (Without<PendingCards>, Without<TargetPosition>)>,
) {
    let Ok(pile1) = piles.get(collision.collider1) else {
        return;
    };
    let Ok(pile2) = piles.get(collision.collider2) else {
        return;
    };
    if TapState::from(pile1.transform.rotation) != TapState::from(pile2.transform.rotation)
        || FlippedState::from(pile1.transform.rotation)
            != FlippedState::from(pile2.transform.rotation)
    {
        return;
    }
    if pile1.transform.translation.y > pile2.transform.translation.y {
        return;
    }
    if pile1
        .transform
        .with_translation(Vec3::splat(0.0))
        .mul_transform(
            pile2
                .transform
                .with_translation(pile2.transform.translation - pile1.transform.translation),
        )
        .translation
        .xz()
        .length()
        > CARD_WIDTH / 4.0
    {
        return;
    }
    commands.trigger(PileMerge {
        from: pile2.entity,
        to: pile1.entity,
    });
}
