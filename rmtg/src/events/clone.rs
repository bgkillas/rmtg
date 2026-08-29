use crate::assets::AssetManager;
use crate::events::hover::HoveredObject;
use crate::events::move_up::MoveUp;
use crate::keybinds::Keybind;
use crate::pile::{PendingCards, Pile};
use crate::shapes::Shape;
use crate::spatial::Spatial;
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::prelude::{Commands, Event, On, Query, Resource, Transform, With};
use bevy_ecs::entity::Entity;
use bevy_ecs::query::Without;
use bevy_ecs::system::{Res, ResMut};
use bevy_query_fn_macro::query_fn;
#[derive(Event, Clone, Debug)]
pub struct CloneObj {
    pub clone_type: CloneType,
    pub transform: Transform,
}
#[derive(Resource, Default, Debug)]
pub struct CloneObjs {
    pub objects: Vec<CloneObj>,
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
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum CloneType {
    Pile(Pile),
    Shape(Shape),
}
impl Clone for CloneType {
    fn clone(&self) -> Self {
        match self {
            Self::Pile(pile) => Self::Pile(pile.clone()),
            Self::Shape(shape) => Self::Shape(*shape),
        }
    }
}
pub fn on_clone(clone: On<CloneObj>, mut commands: Commands, asset: AssetManager) {
    let mut ent = commands.spawn(clone.transform);
    let id = match &clone.clone_type {
        CloneType::Pile(deck) => {
            ent.insert(deck.clone().bundle());
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
    hovered_entities: Query<Entity, (With<HoveredObject>, Without<PendingCards>)>,
    spatial: Spatial,
) {
    let Some((_, pos, _)) = spatial.ray() else {
        return;
    };
    if keybinds.just_pressed(Keybind::CopyObject) {
        commands.trigger(CloneObjects {
            objects: hovered_entities.into_iter().collect(),
            pos,
        });
    }
    if keybinds.just_pressed(Keybind::PasteObject) {
        commands.trigger(PasteObjects { pos });
    }
}
#[derive(Event)]
pub struct CloneObjects {
    pub objects: Box<[Entity]>,
    pub pos: Vec3,
}
#[derive(Event)]
pub struct PasteObjects {
    pub pos: Vec3,
}
#[query_fn]
pub fn on_clone_objects(
    on: On<CloneObjects>,
    hovered_entities: Query<(&Transform, Option<&Shape>, Option<&Pile>), Without<PendingCards>>,
    mut objects: ResMut<CloneObjs>,
) {
    objects.objects.clear();
    for &ent in &on.objects {
        let hovered = hovered_entities.get(ent).unwrap();
        let ty = match (hovered.shape, hovered.pile) {
            (Some(&shape), None) => CloneType::Shape(shape),
            (None, Some(pile)) => CloneType::Pile(pile.clone()),
            _ => unreachable!(),
        };
        let mut trans = *hovered.transform;
        trans.translation -= on.pos;
        objects.objects.push(CloneObj::new(ty, trans));
    }
}
#[query_fn]
pub fn on_paste_objects(on: On<PasteObjects>, mut commands: Commands, objects: Res<CloneObjs>) {
    for mut clone in objects.objects.iter().cloned() {
        clone.transform.translation += on.pos;
        commands.trigger(clone);
    }
}
