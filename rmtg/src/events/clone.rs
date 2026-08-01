use crate::assets::Asset;
use crate::events::move_up::MoveUp;
use crate::shapes::{OUTLINE_COLOR, Shape};
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::prelude::{Commands, Entity, EntityEvent, On, Query, Transform};
#[derive(EntityEvent)]
pub struct Clone {
    pub entity: Entity,
    pub pos: Vec3,
}
pub fn on_clone(
    clone: On<Clone>,
    query: Query<(&Transform, Option<&Shape>)>,
    mut commands: Commands,
    mut asset: Asset,
) {
    let (&(mut transform), is_shape) = query.get(clone.entity).unwrap();
    transform.translation = clone.pos;
    let ent = commands.spawn(transform);
    if let Some(shape) = is_shape {
        let shape_ent = shape.insert_dice(Color::WHITE, OUTLINE_COLOR, &mut asset, ent);
        let id = shape_ent.id();
        commands.trigger(MoveUp::new(id));
    }
}
