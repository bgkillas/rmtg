use crate::spatial::Spatial;
use bevy::prelude::{Commands, Component, Entity, Single, With};
#[derive(Component)]
pub struct Hovered;
pub fn update_hover(
    old: Option<Single<Entity, With<Hovered>>>,
    spatial: Spatial,
    mut commands: Commands,
) {
    if let Some((hit, _)) = spatial.ray()
        && old.as_ref().is_none_or(|e| **e != hit.entity)
    {
        if let Some(ent) = old {
            commands.entity(*ent).remove::<Hovered>();
        }
        commands.entity(hit.entity).insert(Hovered);
    }
}
