use avian3d::prelude::ColliderAabb;
use avian3d::spatial_query::SpatialQuery;
use bevy::prelude::{Entity, EntityEvent, On, Query, Transform};
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
    aabbs: Query<&ColliderAabb>,
    mut transforms: Query<&mut Transform>,
    spatial: SpatialQuery,
) {
    let mut transform = transforms.get_mut(entity.entity).unwrap();
    let mut collider = *aabbs.get(entity.entity).unwrap();
    let mut some = true;
    while some {
        some = false;
        spatial.aabb_intersections_with_aabb_callback(collider, |ent| {
            if ent != entity.entity {
                let aabb = aabbs.get(ent).unwrap();
                let delta = aabb.max.y - collider.min.y;
                if delta > 0.0 {
                    some = true;
                    collider.min.y += delta;
                    collider.max.y += delta;
                    transform.translation.y += delta;
                }
            }
            true
        });
    }
}
