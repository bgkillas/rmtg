use crate::CARD_THICKNESS;
use crate::startup::{Ceiling, Wall};
use avian3d::parry::shape::SharedShape;
use avian3d::prelude::{
    Collider, ColliderAabb, ScalableCollider as _, Sleeping, SpatialQueryFilter, WakeBody,
};
use avian3d::spatial_query::SpatialQuery;
use bevy::math::Vec3;
use bevy::prelude::{Entity, EntityEvent, On, Or, Query, Transform, With};
use bevy_ecs::system::Commands;
use std::sync::Arc;
#[derive(EntityEvent)]
pub struct MoveUp {
    pub entity: Entity,
}
impl MoveUp {
    #[must_use]
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}
pub fn move_up(
    entity: On<MoveUp>,
    colliders: Query<&Collider>,
    aabbs: Query<&ColliderAabb>,
    is_wall: Query<(), Or<(With<Wall>, With<Ceiling>)>>,
    mut transforms: Query<&mut Transform>,
    spatial: SpatialQuery,
    mut commands: Commands,
    is_sleeping: Query<(), With<Sleeping>>,
) {
    const SCALE: f32 = 63.0 / 64.0;
    if is_sleeping.contains(entity.entity) {
        commands.queue(WakeBody(entity.entity));
    }
    let mut transform = transforms.get_mut(entity.entity).unwrap();
    let mut ent_aabb = *aabbs.get(entity.entity).unwrap();
    let mut shape = colliders.get(entity.entity).unwrap().shape_scaled().clone();
    let mut collider = Collider::from(SharedShape(Arc::from(shape.make_mut().clone_dyn())));
    let delta = ent_aabb.max - ent_aabb.min;
    let val = if delta.min_element() - CARD_THICKNESS > delta.min_element() * SCALE {
        (delta.min_element() - CARD_THICKNESS) / delta.min_element()
    } else {
        SCALE
    };
    collider.scale_by(Vec3::splat(val), 0);
    let mut some = true;
    while some {
        some = false;
        spatial.shape_intersections_callback(
            &collider,
            transform.translation,
            transform.rotation,
            &SpatialQueryFilter::DEFAULT,
            |ent| {
                if ent != entity.entity && !is_wall.contains(ent) {
                    let aabb = aabbs.get(ent).unwrap();
                    let delta = aabb.max.y - ent_aabb.min.y;
                    if delta > 0.0 {
                        some = true;
                        let eps = delta + CARD_THICKNESS;
                        ent_aabb.min.y += eps;
                        ent_aabb.max.y += eps;
                        transform.translation.y += eps;
                    }
                }
                true
            },
        );
    }
}
