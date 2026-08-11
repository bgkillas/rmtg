use crate::assets::Asset;
use crate::events::hover::HoveredObject;
use crate::events::move_up::MoveUp;
use crate::keybinds::{Keybind, Keybinds};
use crate::pile::Pile;
use crate::shapes::{OUTLINE_COLOR, Shape};
use crate::spatial::Spatial;
use bevy::color::Color;
use bevy::prelude::{Commands, Event, Local, On, Query, Transform, With};
use bevy_query_macro::query_fn;
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
#[query_fn]
pub fn update_clone(
    keybinds: Keybinds,
    mut commands: Commands,
    hovered_entities: Query<(&Transform, Option<&Shape>, Option<&Pile>), With<HoveredObject>>,
    spatial: Spatial,
    mut objects: Local<Vec<Clone>>,
) {
    let Some((_, pos, _)) = spatial.ray() else {
        return;
    };
    if keybinds.just_pressed(Keybind::CopyObject) {
        objects.clear();
        for hovered in hovered_entities {
            let ty = match (hovered.shape, hovered.pile) {
                (Some(&shape), None) => CloneType::Shape(shape),
                (None, Some(pile)) => CloneType::Pile(pile.clone()),
                _ => unreachable!(),
            };
            let mut trans = *hovered.transform;
            trans.translation -= pos;
            objects.push(Clone::new(ty, trans));
        }
    }
    if keybinds.just_pressed(Keybind::PasteObject) {
        for mut clone in objects.iter().cloned() {
            clone.transform.translation += pos;
            commands.trigger(clone);
        }
    }
}
