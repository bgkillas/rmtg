use crate::assets::AssetManager;
use crate::events::hover::HoveredObject;
use crate::events::move_up::MoveUp;
use crate::keybinds::Keybind;
use crate::pile::{PendingCards, Pile};
use crate::shapes::Shape;
use crate::spatial::Spatial;
use bevy::input::ButtonInput;
use bevy::prelude::{Commands, Event, Local, On, Query, Transform, With};
use bevy_ecs::query::Without;
use bevy_ecs::system::Res;
use bevy_query_fn_macro::query_fn;
#[derive(Event, Clone)]
pub struct CloneObj {
    pub clone_type: CloneType,
    pub transform: Transform,
}
impl CloneObj {
    #[must_use]
    pub fn new(clone_type: CloneType, transform: Transform) -> Self {
        Self {
            clone_type,
            transform,
        }
    }
}
pub enum CloneType {
    Pile(Pile),
    Shape(Shape),
}
impl Clone for CloneType {
    fn clone(&self) -> Self {
        match self {
            Self::Pile(pile) => Self::Pile(pile.try_clone().unwrap()),
            Self::Shape(shape) => Self::Shape(*shape),
        }
    }
}
pub fn on_clone(clone: On<CloneObj>, mut commands: Commands, asset: AssetManager) {
    let mut ent = commands.spawn(clone.transform);
    let id = match &clone.clone_type {
        CloneType::Pile(deck) => {
            ent.insert(deck.try_clone().unwrap().bundle());
            ent.id()
        }
        &CloneType::Shape(shape) => {
            let shape_ent = shape.insert_dice(&asset, ent);
            shape_ent.id()
        }
    };
    commands.trigger(MoveUp::new(id));
}
#[query_fn]
pub fn update_clone(
    keybinds: Res<ButtonInput<Keybind>>,
    mut commands: Commands,
    hovered_entities: Query<
        (&Transform, Option<&Shape>, Option<&Pile>),
        (With<HoveredObject>, Without<PendingCards>),
    >,
    spatial: Spatial,
    mut objects: Local<Vec<CloneObj>>,
) {
    let Some((_, pos, _)) = spatial.ray() else {
        return;
    };
    if keybinds.just_pressed(Keybind::CopyObject) {
        objects.clear();
        for hovered in hovered_entities {
            let ty = match (hovered.shape, hovered.pile) {
                (Some(&shape), None) => CloneType::Shape(shape),
                (None, Some(pile)) => CloneType::Pile(pile.try_clone().unwrap()),
                _ => unreachable!(),
            };
            let mut trans = *hovered.transform;
            trans.translation -= pos;
            objects.push(CloneObj::new(ty, trans));
        }
    }
    if keybinds.just_pressed(Keybind::PasteObject) {
        for mut clone in objects.iter().cloned() {
            clone.transform.translation += pos;
            commands.trigger(clone);
        }
    }
}
