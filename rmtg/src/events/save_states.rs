use crate::pile::Pile;
use crate::shapes::Shape;
use bevy::prelude::Transform;
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Local, Query, ResMut};
use bevy_p2p::bitcode::{self, Decode, Encode};
use bevy_query_fn_macro::query_fn;
use circular_buffer::FixedCircularBuffer;
use importer::card::CardAttributes;
use importer::coder::DataCoder;
use importer::uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
#[derive(Resource, Default)]
pub struct SaveStates {
    pub states: FixedCircularBuffer<SaveState, 256>,
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
    pub attributes: CardAttributes,
}
pub const SAVE_PER_SECOND: f64 = 1.0;
#[query_fn]
pub fn update_save_states(
    mut states: ResMut<SaveStates>,
    dice_query: Query<(&Shape, &Transform)>,
    pile_query: Query<(&Pile, &Transform)>,
    mut last: Local<f64>,
) {
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let mult = (time * SAVE_PER_SECOND).floor();
    if mult <= *last {
        return;
    }
    *last = mult;
    let mut dices = Vec::new();
    for dice in dice_query {
        let state = ShapeState {
            shape: *dice.shape,
            transform: *dice.transform,
        };
        dices.push(state);
    }
    let mut piles = Vec::new();
    for pile in pile_query {
        let cards = pile
            .pile
            .iter()
            .map(|card| CardState {
                id: card.data.id,
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
}
