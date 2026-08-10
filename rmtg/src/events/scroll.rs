use bevy::math::Vec2;
use bevy::prelude::EntityEvent;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::On;
#[derive(EntityEvent)]
pub struct ScrollToBottom {
    pub entity: Entity,
}
impl ScrollToBottom {
    #[must_use]
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }
}
pub fn scroll_to_bottom(event: On<ScrollToBottom>) {
    _ = event;
}
#[derive(EntityEvent)]
pub struct Scroll {
    pub entity: Entity,
    pub delta: Vec2,
}
impl Scroll {
    #[must_use]
    pub fn new(entity: Entity, delta: Vec2) -> Self {
        Self { entity, delta }
    }
}
pub fn scroll(event: On<Scroll>) {
    _ = event;
}
