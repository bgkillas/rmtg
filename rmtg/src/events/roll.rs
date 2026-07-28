use crate::MAT_HEIGHT;
use crate::deck::Pile;
use crate::events::hover::Hovered;
use crate::events::repaint::Repaint;
use crate::keybinds::{Keybind, Keybinds};
use crate::shapes::FaceNumber;
use avian3d::prelude::{AngularVelocity, ColliderDisabled, LinearVelocity};
use bevy::prelude::{
    Children, Commands, Component, Entity, EntityEvent, On, Query, Transform, With, Without,
};
use rand::prelude::StdRng;
use rand::{RngExt as _, make_rng};
use std::f32::consts::TAU;
#[derive(EntityEvent)]
pub struct Roll {
    pub entity: Entity,
}
#[derive(Component)]
pub struct Rolling;
impl Roll {
    #[must_use]
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}
pub fn on_roll(
    on: On<Roll>,
    mut decks: Query<&mut Pile>,
    mut query: Query<
        (
            &mut Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &Children,
        ),
        Without<FaceNumber>,
    >,
    faces: Query<&Transform, With<FaceNumber>>,
    mut commands: Commands,
) {
    if let Ok(mut deck) = decks.get_mut(on.entity) {
        deck.shuffle();
        commands.trigger(Repaint::new(on.entity));
    } else if let Ok((mut transform, mut vel, mut ang, children)) = query.get_mut(on.entity) {
        let mut rng = make_rng::<StdRng>();
        let i1 = rng.random_range(1..children.len());
        let i2 = rng.random_range(1..children.len());
        let t1 = faces.get(children[i1]).unwrap();
        let t2 = faces.get(children[i2]).unwrap();
        transform.rotation = t2.rotation * t1.rotation.inverse() * transform.rotation;
        vel.y = MAT_HEIGHT;
        ang.x = TAU * rng.random_range(2.0..4.0) + ang.x.abs();
        ang.y = TAU * rng.random_range(2.0..4.0) + ang.y.abs();
        ang.z = TAU * rng.random_range(2.0..4.0) + ang.z.abs();
        if rng.random() {
            ang.x = -ang.x;
        }
        if rng.random() {
            ang.y = -ang.y;
        }
        if rng.random() {
            ang.z = -ang.z;
        }
        commands
            .entity(on.entity)
            .insert((Rolling, ColliderDisabled));
    }
}
pub fn update_rolling(
    query: Query<(Entity, &LinearVelocity), With<Rolling>>,
    mut commands: Commands,
) {
    for (ent, vel) in query {
        if vel.y < 0.0 {
            commands.entity(ent).remove::<(Rolling, ColliderDisabled)>();
        }
    }
}
pub fn do_roll(hovered: Query<Entity, With<Hovered>>, mut commands: Commands, keybinds: Keybinds) {
    for ent in hovered {
        if keybinds.just_pressed(Keybind::Shuffle) {
            commands.trigger(Roll::new(ent));
        }
    }
}
