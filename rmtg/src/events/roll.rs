use crate::events::hover::HoveredObject;
use crate::events::repaint::Repaint;
use crate::keybinds::{Keybind, Keybinds};
use crate::physics::WorldLayer;
use crate::pile::Pile;
use crate::shapes::{FaceNumber, Shape};
use crate::{CARD_THICKNESS, MAT_HEIGHT};
use avian3d::prelude::{AngularVelocity, CollisionLayers, LayerMask, LinearVelocity, Sleeping};
use bevy::prelude::{
    Children, Commands, Component, Entity, EntityEvent, On, Query, Transform, With, Without,
};
use bevy_ecs::system::In;
use bevy_query_fn_macro::query_fn;
use rand::prelude::StdRng;
use rand::{RngExt as _, make_rng};
use std::f32::consts::TAU;
#[derive(EntityEvent)]
pub struct Roll {
    pub entity: Entity,
}
#[derive(Component, Clone)]
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
        transform.rotation *= t2.rotation * t1.rotation.inverse();
        vel.y = MAT_HEIGHT * rng.random_range(1.0..=1.5);
        let start = 2.0;
        let end = 4.0;
        ang.x = TAU * rng.random_range(start..=end) + ang.x.abs();
        ang.y = TAU * rng.random_range(start..=end) / 4.0 + ang.y.abs();
        ang.z = TAU * rng.random_range(start..=end) + ang.z.abs();
        if rng.random() {
            ang.x = -ang.x;
        }
        if rng.random() {
            ang.y = -ang.y;
        }
        if rng.random() {
            ang.z = -ang.z;
        }
        commands.entity(on.entity).insert((
            Rolling,
            CollisionLayers::new(WorldLayer::Default, LayerMask::NONE),
        ));
    }
}
#[query_fn]
pub fn update_rolling(
    query: Query<(Entity, &LinearVelocity, &CollisionLayers, Option<&Sleeping>), With<Rolling>>,
    mut commands: Commands,
) {
    for rolling in query {
        if rolling.linear_velocity.y <= CARD_THICKNESS {
            if rolling.sleeping.is_some() {
                commands.entity(rolling.entity).remove::<Rolling>();
                commands.run_system_cached_with(stopped_roll, rolling.entity);
            }
            if !rolling.collision_layers.filters.has_all(LayerMask::ALL) {
                commands
                    .entity(rolling.entity)
                    .insert(CollisionLayers::new(WorldLayer::Default, LayerMask::ALL));
            }
        }
    }
}
#[derive(EntityEvent)]
pub struct StoppedRoll {
    pub entity: Entity,
    pub val: usize,
}
#[query_fn]
fn stopped_roll(
    In(entity): In<Entity>,
    query: Query<(&Transform, &Children, &Shape), Without<FaceNumber>>,
    faces: Query<&Transform, With<FaceNumber>>,
    mut commands: Commands,
) {
    let shape = query.get(entity).unwrap();
    for (i, &face) in shape.children[1..].iter().enumerate() {
        let trans = faces.get(face).unwrap();
        let global = shape.transform.mul_transform(*trans);
        let forward = global.forward();
        let val = forward.x.hypot(forward.z);
        if val < 1.0 / 256.0
            && (matches!(shape.shape, Shape::Tetrahedron) || forward.y.is_sign_negative())
        {
            commands.trigger(StoppedRoll { entity, val: i });
            return;
        }
    }
}
pub fn do_roll(
    hovered: Query<Entity, With<HoveredObject>>,
    mut commands: Commands,
    keybinds: Keybinds,
) {
    if keybinds.just_pressed(Keybind::Shuffle) {
        for ent in hovered {
            commands.trigger(Roll::new(ent));
        }
    }
}
