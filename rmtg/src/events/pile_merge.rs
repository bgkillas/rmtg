use crate::drag::TargetPosition;
use crate::events::repaint::Repaint;
use crate::pile::{FlippedState, PendingCards, Pile, TapState};
use crate::{CARD_THICKNESS, CARD_WIDTH};
use avian3d::prelude::CollisionStart;
use bevy::math::{Vec3, Vec3Swizzles as _};
use bevy::prelude::{Event, Transform};
use bevy_ecs::entity::Entity;
use bevy_ecs::message::{Message, MessageWriter, PopulatedMessageReader};
use bevy_ecs::observer::On;
use bevy_ecs::query::Without;
use bevy_ecs::system::{Commands, Query};
use bevy_query_fn_macro::query_fn;
use std::mem;
#[derive(Event, Clone, Copy)]
pub struct PileMerge {
    pub from: Entity,
    pub to: Entity,
}
#[query_fn]
pub fn on_pile_merge(
    event: On<PileMerge>,
    mut commands: Commands,
    mut piles: Query<(Entity, &mut Pile, &mut Transform)>,
) {
    let Ok([mut pile1, mut pile2]) = piles.get_many_mut([event.from, event.to]) else {
        return;
    };
    let l1 = pile1.pile.len();
    pile2.pile.extend(mem::take(&mut pile1.pile));
    let up = pile2.transform.up();
    pile2.transform.translation += up * l1 as f32 * CARD_THICKNESS / 2.0;
    commands.trigger(Repaint::new(pile2.entity));
    commands.entity(pile1.entity).despawn();
}
#[derive(Message, Clone, Copy)]
pub struct DelayPileMerge(pub PileMerge);
pub fn delayed_pile_merge(
    mut reader: PopulatedMessageReader<DelayPileMerge>,
    mut commands: Commands,
) {
    for &DelayPileMerge(event) in reader.read() {
        commands.trigger(event);
    }
}
#[query_fn]
pub fn trigger_pile_merge(
    collision: On<CollisionStart>,
    piles: Query<(Entity, &mut Pile, &Transform), (Without<PendingCards>, Without<TargetPosition>)>,
    mut writer: MessageWriter<DelayPileMerge>,
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
    writer.write(DelayPileMerge(PileMerge {
        from: pile2.entity,
        to: pile1.entity,
    }));
}
