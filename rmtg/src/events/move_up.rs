use crate::CARD_THICKNESS;
use crate::startup::{Ceiling, Wall};
use avian3d::prelude::{Collider, ColliderAabb, ScalableCollider as _, SpatialQueryFilter};
use avian3d::spatial_query::SpatialQuery;
use bevy::math::Vec3;
use bevy::prelude::{Entity, EntityEvent, On, Or, Query, Transform, With};
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
) {
    let mut transform = transforms.get_mut(entity.entity).unwrap();
    let mut ent_aabb = *aabbs.get(entity.entity).unwrap();
    let mut collider = colliders.get(entity.entity).unwrap().clone();
    collider.scale_by(Vec3::splat(63.0 / 64.0), 0);
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
