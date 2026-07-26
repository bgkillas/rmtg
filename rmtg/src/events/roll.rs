use crate::MAT_WIDTH;
use crate::deck::Pile;
use crate::events::repaint::Repaint;
use avian3d::prelude::{AngularVelocity, LinearVelocity};
use bevy::math::Quat;
use bevy::prelude::{Children, Commands, Entity, EntityEvent, On, Query, Transform};
use rand::prelude::StdRng;
use rand::{RngExt as _, make_rng};
use std::f32::consts::TAU;
#[derive(EntityEvent)]
pub struct Roll {
    pub entity: Entity,
}
impl Roll {
    #[must_use]
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}
pub fn on_roll(
    on: On<Repaint>,
    mut decks: Query<&mut Pile>,
    mut query: Query<(
        &mut Transform,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &Children,
    )>,
    faces: Query<&Transform>,
    mut commands: Commands,
) {
    if let Ok(mut deck) = decks.get_mut(on.entity) {
        deck.shuffle();
        commands.trigger(Repaint::new(on.entity));
    } else if let Ok((mut transform, mut vel, mut ang, children)) = query.get_mut(on.entity) {
        let mut rng = make_rng::<StdRng>();
        let i1 = rng.random_range(0..children.len());
        let i2 = rng.random_range(0..children.len());
        let t1 = faces.get(children[i1]).unwrap();
        let t2 = faces.get(children[i2]).unwrap();
        transform.rotation *= Quat::from_rotation_arc(t1.translation, t2.translation);
        vel.y += MAT_WIDTH;
        ang.x = TAU * rng.random_range(10.0..20.0) + ang.x.abs();
        ang.y = TAU * rng.random_range(10.0..20.0) + ang.y.abs();
        ang.z = TAU * rng.random_range(10.0..20.0) + ang.z.abs();
        if rng.random() {
            ang.x = -ang.x;
        }
        if rng.random() {
            ang.y = -ang.y;
        }
        if rng.random() {
            ang.z = -ang.z;
        }
    }
}
