use crate::assets::Asset;
use crate::deck::Pile;
use crate::events::move_up::MoveUp;
use crate::keybinds::{Keybind, Keybinds};
use crate::shapes::{OUTLINE_COLOR, Shape};
use crate::spatial::Spatial;
use bevy::color::Color;
use bevy::prelude::{Commands, Event, Local, On, Query, Transform};
#[derive(Event, Clone)]
pub struct Clone {
    pub clone_type: CloneType,
    pub transform: Transform,
}
impl Clone {
    #[must_use]
    pub fn new(clone_type: CloneType, transform: Transform) -> Self {
        Self {
            clone_type,
            transform,
        }
    }
}
#[derive(Clone)]
pub enum CloneType {
    Pile(Pile),
    Shape(Shape),
}
pub fn on_clone(clone: On<Clone>, mut commands: Commands, mut asset: Asset) {
    let mut ent = commands.spawn(clone.transform);
    let id = match &clone.clone_type {
        CloneType::Pile(deck) => {
            ent.insert(deck.clone().bundle(&mut asset));
            ent.id()
        }
        &CloneType::Shape(shape) => {
            let shape_ent = shape.insert_dice(Color::WHITE, OUTLINE_COLOR, &mut asset, ent);
            shape_ent.id()
        }
    };
    commands.trigger(MoveUp::new(id));
}
pub fn update_clone(
    keybinds: Keybinds,
    mut commands: Commands,
    query: Query<(&Transform, Option<&Shape>, Option<&Pile>)>,
    spatial: Spatial,
    mut object: Local<Option<Clone>>,
) {
    let Some((hit, pos)) = spatial.ray() else {
        return;
    };
    if keybinds.just_pressed(Keybind::CopyObject) {
        *object = match query.get(hit.entity) {
            Ok((&transform, Some(&shape), None)) => {
                Some(Clone::new(CloneType::Shape(shape), transform))
            }
            Ok((&transform, None, Some(pile))) => {
                Some(Clone::new(CloneType::Pile(pile.clone()), transform))
            }
            Ok(_) | Err(_) => None,
        };
    }
    if keybinds.just_pressed(Keybind::PasteObject)
        && let Some(mut obj) = object.clone()
    {
        obj.transform.translation = pos;
        commands.trigger(obj);
    }
}
