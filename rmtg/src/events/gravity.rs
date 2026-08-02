use avian3d::prelude::GravityScale;
use bevy::prelude::{Entity, EntityEvent, On, Query};
#[derive(EntityEvent)]
pub struct NewGravity {
    pub entity: Entity,
    pub gravity: f32,
}
impl NewGravity {
    #[must_use]
    pub fn new(entity: Entity, gravity: f32) -> Self {
        Self { entity, gravity }
    }
}
pub fn on_change_gravity(event: On<NewGravity>, mut query: Query<&mut GravityScale>) {
    let mut gravity = query.get_mut(event.entity).unwrap();
    gravity.0 = event.gravity;
}
