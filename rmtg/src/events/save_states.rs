use crate::assets::AssetManager;
use crate::pile::Pile;
use crate::shapes::Shape;
use bevy::prelude::Transform;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::observer::On;
use bevy_ecs::query::{Or, With};
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Commands, Local, Query, Res, ResMut};
use bevy_ecs::world::World;
use bevy_p2p::bitcode::{self, Decode, Encode};
use bevy_query_fn_macro::query_fn;
use circular_buffer::FixedCircularBuffer;
use importer::card::{CardAttributes, SubCard};
use importer::coder::DataCoder;
use importer::scryfall::{CACHE, Quality};
use importer::uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
#[derive(Resource, Default)]
pub struct SaveStates {
    pub states: FixedCircularBuffer<SaveState, 256>,
    pub pause: bool,
}
#[derive(Encode, Decode)]
pub struct SaveState {
    pub dices: Box<[ShapeState]>,
    pub piles: Box<[PileState]>,
}
#[derive(Encode, Decode)]
pub struct ShapeState {
    pub shape: Shape,
    #[bitcode(with = "DataCoder<Transform>")]
    pub transform: Transform,
}
#[derive(Encode, Decode)]
pub struct PileState {
    pub equiped: bool,
    pub cards: Box<[CardState]>,
    #[bitcode(with = "DataCoder<Transform>")]
    pub transform: Transform,
}
#[derive(Encode, Decode)]
pub struct CardState {
    #[bitcode(with = "DataCoder<Uuid>")]
    pub id: Uuid,
    pub quality: Quality,
    pub attributes: CardAttributes,
}
pub const SAVE_PER_SECOND: f64 = 1.0;
#[query_fn]
pub fn update_save_states(
    mut states: ResMut<SaveStates>,
    dice_query: Query<(&Shape, &Transform)>,
    pile_query: Query<(&Pile, &Transform)>,
    mut last: Local<f64>,
    mut commands: Commands,
) {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let mult = (time * SAVE_PER_SECOND).floor();
    if *last == 0.0 {
        *last = mult;
        return;
    }
    if mult <= *last {
        return;
    }
    *last = mult;
    if states.pause {
        return;
    }
    let mut dices = Vec::with_capacity(dice_query.iter().len());
    for dice in dice_query {
        let state = ShapeState {
            shape: *dice.shape,
            transform: *dice.transform,
        };
        dices.push(state);
    }
    let mut piles = Vec::with_capacity(pile_query.iter().len());
    for pile in pile_query {
        let cards = pile
            .pile
            .iter()
            .map(|card| CardState {
                id: card.data.id,
                quality: card.quality,
                attributes: card.attributes.clone(),
            })
            .collect();
        let state = PileState {
            equiped: pile.pile.is_equiped(),
            cards,
            transform: *pile.transform,
        };
        piles.push(state);
    }
    let state = SaveState {
        dices: Box::from(dices),
        piles: Box::from(piles),
    };
    states.states.push_front(state);
    commands.trigger(NewSaveState);
}
#[derive(Event)]
pub struct NewSaveState;
#[derive(Event)]
pub struct ApplySaveState {
    pub from_front: usize,
}
impl ApplySaveState {
    pub fn new(from_front: usize) -> Self {
        Self { from_front }
    }
}
pub fn apply_save_state(
    apply: On<ApplySaveState>,
    states: Res<SaveStates>,
    mut commands: Commands,
    to_remove: Query<Entity, Or<(With<Shape>, With<Pile>)>>,
    asset: AssetManager,
) {
    for entity in to_remove {
        commands.entity(entity).despawn();
    }
    let from_front = apply.from_front;
    let Some(state) = states.states.nth_front(from_front) else {
        return;
    };
    let cache = CACHE.blocking_lock();
    for pile_state in &state.piles {
        let mut pile = Pile::new(
            pile_state
                .cards
                .iter()
                .map(|c| {
                    let mut card = SubCard::from_cache(&cache, c.id, c.quality);
                    card.attributes = c.attributes.clone();
                    card
                })
                .collect(),
        );
        if pile_state.equiped {
            pile.equip();
        }
        commands.spawn((pile.bundle(), pile_state.transform));
    }
    for dice_state in &state.dices {
        let mut ent = commands.spawn(dice_state.transform);
        dice_state.shape.insert(&asset, &mut ent);
    }
    commands.queue(move |world: &mut World| {
        let mut states = world.resource_mut::<SaveStates>();
        let len = states.states.len();
        states.states.truncate_front(len - from_front);
    });
}
