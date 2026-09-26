use crate::assets::AssetManager;
use crate::events::life_counter::{CommanderCounter, LifeCounter, NewLifeCount};
use crate::events::ping_drag::DragObject;
use crate::events::select_drag::{SelectDrag, TempSelect};
use crate::net::Peer;
use crate::pile::Pile;
use crate::shapes::Shape;
use bevy::log::warn;
use bevy::prelude::Transform;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::observer::On;
use bevy_ecs::query::{Or, With, Without};
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Commands, Local, Query, ResMut};
use bevy_p2p::bitcode::{self, Decode, Encode};
use bevy_query_fn_macro::query_fn;
use circular_buffer::FixedCircularBuffer;
use enum_map::EnumMap;
use importer::card::{CardAttributes, Commander, Player, SubCard};
use importer::coder::DataCoder;
use importer::scryfall::{CACHE, Quality};
use importer::uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};
pub const MAX_SAVE_STATES: usize = 1024;
#[derive(Resource, Default)]
pub struct SaveStates {
    pub states: FixedCircularBuffer<SaveState, MAX_SAVE_STATES>,
    pub pause: bool,
}
#[derive(Encode, Decode, Clone)]
pub struct SaveState {
    pub dices: Box<[ShapeState]>,
    pub piles: Box<[PileState]>,
    #[bitcode(with = "crate::coder::DataCoder<EnumMap<Peer, EnumMap<CommanderCounter, i32>>>")]
    pub life_counters: EnumMap<Peer, EnumMap<CommanderCounter, i32>>,
    pub select_drags: Box<[SelectDragState]>,
}
#[derive(Encode, Decode, Clone, Copy)]
pub enum SelectDragEntity {
    Dice(u32),
    Pile(u32),
    LifeCounter(Peer),
}
#[derive(Encode, Decode, Clone)]
pub struct SelectDragState {
    pub source: SelectDragEntity,
    pub target: SelectDragEntity,
}
#[derive(Encode, Decode, Clone)]
pub struct ShapeState {
    pub shape: Shape,
    #[bitcode(with = "DataCoder<Transform>")]
    pub transform: Transform,
}
#[derive(Encode, Decode, Clone)]
pub struct PileState {
    pub equiped: bool,
    pub cards: Box<[CardState]>,
    #[bitcode(with = "DataCoder<Transform>")]
    pub transform: Transform,
}
#[derive(Encode, Decode, Clone)]
pub struct CardState {
    #[bitcode(with = "DataCoder<Uuid>")]
    pub id: Uuid,
    pub quality: Quality,
    pub commander: Commander,
    pub owner: Player,
    pub attributes: CardAttributes,
}
pub const SAVE_PER_SECOND: f64 = 1.0;
#[query_fn]
pub fn update_save_states(
    mut states: ResMut<SaveStates>,
    dice_query: Query<(&Shape, &Transform, Entity)>,
    pile_query: Query<(&Pile, &Transform, Entity)>,
    life_counters_query: Query<(&LifeCounter, &Peer, Entity)>,
    select_drags_query: Query<&SelectDrag, Without<TempSelect>>,
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
                commander: card.commander,
                owner: card.owner,
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
    let mut select_drags = Vec::with_capacity(select_drags_query.iter().len());
    for select_drag in select_drags_query {
        let get = |entity: Entity| -> SelectDragEntity {
            if dice_query.contains(entity) {
                SelectDragEntity::Dice(
                    dice_query.iter().position(|q| q.entity == entity).unwrap() as u32
                )
            } else if pile_query.contains(entity) {
                SelectDragEntity::Pile(
                    pile_query.iter().position(|q| q.entity == entity).unwrap() as u32
                )
            } else if let Ok(life_counter) = life_counters_query.get(entity) {
                SelectDragEntity::LifeCounter(*life_counter.peer)
            } else {
                unreachable!()
            }
        };
        let source = get(select_drag.source);
        let target = get(select_drag.target);
        let state = SelectDragState { source, target };
        select_drags.push(state);
    }
    let life_counters = EnumMap::from_fn(|peer| {
        EnumMap::from_fn(|c| {
            life_counters_query
                .iter()
                .find(|q| *q.peer == peer && c == q.life_counter.commander)
                .unwrap()
                .life_counter
                .life
        })
    });
    let state = SaveState {
        dices: Box::from(dices),
        piles: Box::from(piles),
        life_counters,
        select_drags: Box::from(select_drags),
    };
    states.states.push_front(state);
    commands.trigger(NewSaveState);
}
#[derive(Event)]
pub struct NewSaveState;
#[derive(Event)]
pub struct ApplySaveState {
    pub state: SaveState,
    pub local: bool,
}
impl ApplySaveState {
    pub fn local(state: SaveState) -> Self {
        Self { state, local: true }
    }
    pub fn new(state: SaveState) -> Self {
        Self {
            state,
            local: false,
        }
    }
}
#[query_fn]
pub fn apply_save_state(
    apply: On<ApplySaveState>,
    mut commands: Commands,
    to_remove: Query<Entity, Or<(With<Shape>, With<Pile>)>>,
    life_counters_query: Query<(&LifeCounter, &Peer, Entity)>,
    assets: AssetManager,
) {
    for entity in to_remove {
        commands.entity(entity).despawn();
    }
    let cache = CACHE.blocking_lock();
    let mut piles = Vec::with_capacity(apply.state.piles.len());
    for pile_state in &apply.state.piles {
        let Some(cards) = pile_state
            .cards
            .iter()
            .map(|c| {
                let mut card = SubCard::from_cache(&cache, c.id, c.quality)?;
                card.attributes = c.attributes.clone();
                card.commander = c.commander;
                card.owner = c.owner;
                Some(card)
            })
            .collect::<Option<Vec<_>>>()
        else {
            warn!("not in cache");
            return;
        };
        let mut pile = Pile::new(cards);
        if pile_state.equiped {
            pile.equip();
        }
        let ent = commands.spawn((pile.bundle(), pile_state.transform));
        piles.push(ent.id());
    }
    let mut dices = Vec::with_capacity(apply.state.piles.len());
    for dice_state in &apply.state.dices {
        let mut ent = commands.spawn(dice_state.transform);
        dice_state.shape.insert(&assets, &mut ent);
        dices.push(ent.id());
    }
    for (peer, map) in &apply.state.life_counters {
        for (commander, &life) in map {
            commands.trigger(NewLifeCount {
                peer,
                commander,
                life,
            });
        }
    }
    for state in &apply.state.select_drags {
        let get = |entity: SelectDragEntity| -> Entity {
            match entity {
                SelectDragEntity::Dice(index) => dices[index as usize],
                SelectDragEntity::Pile(index) => piles[index as usize],
                SelectDragEntity::LifeCounter(peer) => {
                    life_counters_query
                        .iter()
                        .find(|q| *q.peer == peer && q.life_counter.commander.peer().is_none())
                        .unwrap()
                        .entity
                }
            }
        };
        let source = get(state.source);
        let target = get(state.target);
        commands.spawn((
            SelectDrag {
                source,
                target,
                source_identifier: Entity::PLACEHOLDER,
                target_identifier: Entity::PLACEHOLDER,
            },
            DragObject::empty(&assets),
        ));
    }
}
