use bevy::prelude::{Entity, EntityEvent, On};
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
pub fn move_up(entity: On<MoveUp>) {
    _ = entity;
    todo!()
}
