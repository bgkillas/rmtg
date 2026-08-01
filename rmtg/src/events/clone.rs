use bevy::math::Vec3;
use bevy::prelude::{Commands, Entity, EntityEvent, On, Query, Transform};
#[derive(EntityEvent)]
pub struct Clone {
    pub entity: Entity,
    pub pos: Vec3,
}
pub fn on_clone(clone: On<Clone>, transforms: Query<&Transform>, mut commands: Commands) {
    //TODO
}
