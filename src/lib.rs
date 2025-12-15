use crate::setup::{MAT_BAR, MAT_HEIGHT, MAT_WIDTH, setup};
use crate::update::{
    GiveEnts, ToMoveUp, cam_rotation, cam_translation, esc_menu, follow_mouse, gather_hand,
    give_ents, listen_for_deck, listen_for_mouse, on_scroll_handler, pick_from_list, pile_merge,
    register_deck, rem_peers, reset_layers, send_scroll_events, set_card_spot, to_move_up,
    update_hand, update_search_deck,
};
use avian3d::prelude::*;
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy_framepace::FramepacePlugin;
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use bevy_rich_text3d::{LoadFonts, Text3dPlugin};
use bevy_tangled::{Client, PeerId};
use bevy_ui_text_input::TextInputPlugin;
use rand::RngCore;
use std::collections::HashMap;
use std::fmt::{Debug, Error, Formatter};
use std::ops::{Bound, RangeBounds};
use std::slice::{Iter, IterMut};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::{fmt, iter, mem};
const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
const CARD_WIDTH: f32 = CARD_HEIGHT * IMAGE_WIDTH / IMAGE_HEIGHT;
const CARD_HEIGHT: f32 = (MAT_HEIGHT - MAT_BAR) / 5.0 - MAT_BAR;
const IMAGE_WIDTH: f32 = 500.0;
const IMAGE_HEIGHT: f32 = 700.0;
const EQUIP_SCALE: f32 = 0.5;
const CARD_THICKNESS: f32 = CARD_WIDTH / 256.0;
const START_Y: f32 = MAT_WIDTH;
const GRAVITY: f32 = CARD_HEIGHT;
const LIN_DAMPING: f32 = CARD_THICKNESS;
const ANG_DAMPING: f32 = 0.25;
const PLAYER0: bevy::color::Color = bevy::color::Color::srgb_u8(255, 85, 85);
const PLAYER1: bevy::color::Color = bevy::color::Color::srgb_u8(85, 85, 255);
const PLAYER2: bevy::color::Color = bevy::color::Color::srgb_u8(255, 85, 255);
const PLAYER3: bevy::color::Color = bevy::color::Color::srgb_u8(85, 255, 85);
const PLAYER4: bevy::color::Color = bevy::color::Color::srgb_u8(85, 255, 255);
const PLAYER5: bevy::color::Color = bevy::color::Color::srgb_u8(255, 255, 85);
const PLAYER: [bevy::color::Color; 6] = [PLAYER0, PLAYER1, PLAYER2, PLAYER3, PLAYER4, PLAYER5];
mod counters;
mod download;
mod misc;
mod setup;
mod shapes;
mod sync;
mod update;
use crate::misc::is_reversed;
use crate::shapes::Shape;
#[cfg(feature = "steam")]
use crate::sync::display_steam_info;
#[cfg(all(feature = "steam", feature = "ip"))]
use crate::sync::new_lobby;
use crate::sync::{SendSleeping, Sent, SyncActions, SyncCount, SyncObject, apply_sync, get_sync};
use bitcode::{Decode, Encode};
#[cfg(feature = "wasm")]
use futures::channel::oneshot;
use itertools::Either;
use rand::seq::SliceRandom;
use uuid::Uuid;
#[cfg(feature = "wasm")]
use wasm_bindgen::JsCast;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "wasm")]
use wasm_bindgen_futures::JsFuture;
#[cfg(feature = "wasm")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};
#[cfg(feature = "steam")]
const APPID: u32 = 4046880;
const FONT_SIZE: f32 = 16.0;
const FONT_HEIGHT: f32 = FONT_SIZE;
const FONT_WIDTH: f32 = FONT_HEIGHT * 3.0 / 5.0;
//TODO multi select, in card counters
//TODO voice/text chat, turns, cards into search
//TODO meld, rooms
rules::generate_types!();
#[cfg_attr(feature = "wasm", wasm_bindgen(start))]
#[cfg(feature = "wasm")]
fn main() {
    start();
}
pub fn start() -> AppExit {
    #[cfg(feature = "wasm")]
    console_error_panic_hook::set_once();
    #[cfg(feature = "wasm")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    #[cfg(not(feature = "wasm"))]
    let runtime = Runtime(tokio::runtime::Runtime::new().unwrap());
    let client = ReqClient(
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .unwrap(),
    );
    #[cfg(not(feature = "wasm"))]
    let clipboard = Clipboard(arboard::Clipboard::new().unwrap());
    #[cfg(feature = "wasm")]
    let clipboard = Clipboard;
    let app_window = Some(Window {
        title: "rmtg".into(),
        resizable: true,
        fit_canvas_to_parent: true,
        ..default()
    });
    let get_deck = GetDeck::default();
    let game_clipboard = GameClipboard::None;
    let mut app = App::new();
    app.add_plugins(Client::new(
        #[cfg(feature = "steam")]
        APPID,
    ));
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: app_window,
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            }),
        PhysicsPlugins::default(),
        PhysicsDebugPlugin,
        FramepacePlugin,
        EntropyPlugin::<WyRand>::default(),
        Text3dPlugin::default(),
        TextInputPlugin,
    ))
    .insert_gizmo_config(
        PhysicsGizmos {
            axis_lengths: None,
            ..default()
        },
        GizmoConfig::default(),
    )
    .insert_resource(LoadFonts {
        font_embedded: vec![include_bytes!("../assets/noto.ttf")],
        ..default()
    })
    .insert_resource(clipboard)
    .insert_resource(ToMoveUp::default())
    .insert_resource(SyncCount::default())
    .insert_resource(Sent::default())
    .insert_resource(Peers::default())
    .insert_resource(RemPeers::default())
    .insert_resource(Menu::default())
    .insert_resource(GiveEnts::default())
    .insert_resource(SendSleeping::default())
    .insert_resource(SyncActions::default())
    .insert_resource(game_clipboard)
    .insert_resource(Download {
        client,
        #[cfg(not(feature = "wasm"))]
        runtime,
        get_deck,
    })
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (
            (
                (
                    set_card_spot,
                    pick_from_list,
                    send_scroll_events,
                    #[cfg(feature = "steam")]
                    display_steam_info,
                    listen_for_deck,
                    register_deck,
                    cam_translation,
                    cam_rotation,
                    esc_menu,
                    #[cfg(all(feature = "steam", feature = "ip"))]
                    new_lobby,
                    update_search_deck,
                    (gather_hand, listen_for_mouse, follow_mouse, update_hand).chain(),
                ),
                to_move_up,
                reset_layers,
            )
                .chain(),
            give_ents,
            rem_peers,
        ),
    )
    .add_systems(PreUpdate, (get_sync, apply_sync).chain())
    .add_observer(on_scroll_handler)
    .add_observer(pile_merge);
    app.run()
}
#[derive(Resource, Default)]
enum Menu {
    #[default]
    World,
    Counter,
    Esc,
    Side,
}
const SLEEP: SleepThreshold = SleepThreshold {
    linear: 4.0 * CARD_THICKNESS,
    angular: 0.25,
};
#[test]
#[cfg(not(feature = "wasm"))]
fn test_parse() {
    use reqwest::header::USER_AGENT;
    let mut app = App::new();
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    fn test(asset_server: Res<AssetServer>) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let tmr = std::time::Instant::now();
        let asset_server = asset_server.clone();
        let img = runtime
            .block_on(runtime.spawn(async move {
                let mut json = json::object!(scryfall_id: "64b0acfa-1a8d-4a94-8972-c9bb235e4897", name: "kilo");
                download::parse(
                    &mut json,
                    &reqwest::Client::builder()
                        .user_agent(USER_AGENT)
                        .build()
                        .unwrap(),
                    &asset_server,
                    1
                )
                .await
            }))
            .unwrap();
        assert!(img.is_some());
        println!("{}", tmr.elapsed().as_millis())
    }
    app.add_systems(Update, test);
    app.update();
}
#[test]
#[cfg(not(feature = "wasm"))]
fn test_get_deck() {
    use reqwest::header::USER_AGENT;
    let mut app = App::new();
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    fn test(asset_server: Res<AssetServer>) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let tmr = std::time::Instant::now();
        let asset_server = asset_server.clone();
        let decks = GetDeck::default();
        let deck = decks.clone();
        runtime
            .block_on(runtime.spawn(async move {
                download::get_deck(
                    "https://api2.moxfield.com/v3/decks/all/_HGo1kgcB0i-4Iq0vR-LZA".to_string(),
                    reqwest::Client::builder()
                        .user_agent(USER_AGENT)
                        .build()
                        .unwrap(),
                    asset_server,
                    deck,
                )
                .await
            }))
            .unwrap();
        let deck = decks.0.lock().unwrap();
        assert_eq!(deck.len(), 4);
        println!(
            "{} {} {} {} {}",
            tmr.elapsed().as_millis(),
            deck[0].0.len(),
            deck[1].0.len(),
            deck[2].0.len(),
            deck[3].0.len()
        );
    }
    app.add_systems(Update, test);
    app.update();
}
#[derive(Resource, Default, Debug)]
struct Peers {
    map: Arc<Mutex<HashMap<PeerId, usize>>>,
    me: Option<usize>,
}
#[derive(Resource, Default, Debug)]
struct RemPeers(Arc<Mutex<Vec<PeerId>>>);
#[derive(Component, Default, Debug)]
struct InHand(usize);
#[derive(Component, Default, Debug)]
struct Hand {
    count: usize,
    removed: Vec<usize>,
}
#[derive(Component, Default, Clone, Debug, Encode, Decode)]
enum Pile {
    Multiple(Vec<SubCard>),
    Single(Box<Card>),
    #[default]
    Empty,
}
impl Pile {
    fn new(mut v: Vec<SubCard>) -> Self {
        if v.len() == 1 {
            Self::Single(v.remove(0).into())
        } else {
            Self::Multiple(v)
        }
    }
    fn equip(&mut self) -> bool {
        match self {
            s @ Pile::Multiple(_) => {
                let subcard = s.pop();
                let Pile::Multiple(equiped) = mem::take(s) else {
                    unreachable!();
                };
                *s = Pile::Single(Box::new(Card {
                    subcard,
                    equiped,
                    power: None,
                    health: None,
                    loyalty: None,
                    misc: None,
                    is_token: false,
                }));
                true
            }
            s @ Pile::Single(_) => {
                let Pile::Single(cards) = mem::take(s) else {
                    unreachable!();
                };
                *s = Pile::Multiple(cards.flatten());
                false
            }
            Pile::Empty => {
                unreachable!()
            }
        }
    }
    fn is_equiped(&self) -> bool {
        if let Pile::Single(s) = self {
            !s.equiped.is_empty()
        } else {
            false
        }
    }
    fn clone_no_image(&self) -> Self {
        match self {
            Pile::Multiple(v) => Pile::Multiple(v.iter().map(|a| a.clone_no_image()).collect()),
            Pile::Single(s) => Pile::Single(s.clone_no_image().into()),
            Pile::Empty => Pile::Empty,
        }
    }
    fn get_card(&self, transform: &Transform) -> &SubCard {
        if is_reversed(transform) {
            self.first()
        } else {
            self.last()
        }
    }
    fn get_mut_card(&mut self, transform: &Transform) -> &mut SubCard {
        if is_reversed(transform) {
            self.first_mut()
        } else {
            self.last_mut()
        }
    }
    #[allow(dead_code)]
    fn get(&self, idx: usize) -> Option<&SubCard> {
        match self {
            Pile::Multiple(v) => v.get(idx),
            Pile::Single(s) => s.get(idx),
            Pile::Empty => unreachable!(),
        }
    }
    fn get_mut(&mut self, idx: usize) -> Option<&mut SubCard> {
        match self {
            Pile::Multiple(v) => v.get_mut(idx),
            Pile::Single(s) => s.get_mut(idx),
            Pile::Empty => unreachable!(),
        }
    }
    fn set_single(&mut self) {
        if self.len() == 1 {
            *self = Pile::Multiple(vec![self.pop()])
        }
    }
    fn take_card(&mut self, transform: &Transform) -> SubCard {
        let ret = if is_reversed(transform) {
            self.remove(0)
        } else {
            self.pop()
        };
        self.set_single();
        ret
    }
    fn len(&self) -> usize {
        match self {
            Pile::Multiple(v) => v.len(),
            Pile::Single(_) => 1,
            Pile::Empty => 0,
        }
    }
    fn is_empty(&self) -> bool {
        match self {
            Pile::Multiple(v) => v.is_empty(),
            Pile::Single(_) => false,
            Pile::Empty => true,
        }
    }
    fn last(&self) -> &SubCard {
        match self {
            Pile::Multiple(v) => v.last().unwrap(),
            Pile::Single(s) => s.into(),
            Pile::Empty => unreachable!(),
        }
    }
    fn pop(&mut self) -> SubCard {
        match self {
            Pile::Multiple(v) => {
                let ret = v.pop().unwrap();
                self.set_single();
                ret
            }
            se @ Pile::Single(_) => {
                let Pile::Single(s) = mem::take(se) else {
                    unreachable!()
                };
                s.into()
            }
            Pile::Empty => unreachable!(),
        }
    }
    fn first(&self) -> &SubCard {
        match self {
            Pile::Multiple(v) => &v[0],
            Pile::Single(s) => s.into(),
            Pile::Empty => unreachable!(),
        }
    }
    fn last_mut(&mut self) -> &mut SubCard {
        match self {
            Pile::Multiple(v) => v.last_mut().unwrap(),
            Pile::Single(s) => s.into(),
            Pile::Empty => unreachable!(),
        }
    }
    fn first_mut(&mut self) -> &mut SubCard {
        match self {
            Pile::Multiple(v) => &mut v[0],
            Pile::Single(s) => s.into(),
            Pile::Empty => unreachable!(),
        }
    }
    fn extend(&mut self, other: Self) {
        match (self, other) {
            (Pile::Multiple(a), Pile::Multiple(b)) => a.extend(b),
            (Pile::Multiple(a), Pile::Single(b)) => a.extend(b.flatten()),
            (se @ Pile::Single(_), o) => {
                let Pile::Single(s) = mem::take(se) else {
                    unreachable!()
                };
                let mut vec = s.flatten();
                match o {
                    Pile::Multiple(v) => vec.extend(v),
                    Pile::Single(s) => vec.extend(s.flatten()),
                    Pile::Empty => unreachable!(),
                }
                *se = Pile::Multiple(vec);
            }
            _ => unreachable!(),
        }
    }
    fn extend_start(&mut self, other: Self) {
        match (self, other) {
            (Pile::Multiple(a), Pile::Multiple(b)) => {
                a.splice(0..0, b);
            }
            (Pile::Multiple(a), Pile::Single(b)) => {
                a.splice(0..0, b.flatten());
            }
            (se @ Pile::Single(_), o) => {
                let Pile::Single(s) = mem::take(se) else {
                    unreachable!()
                };
                let mut vec = s.flatten();
                match o {
                    Pile::Multiple(v) => vec.splice(0..0, v),
                    Pile::Single(s) => vec.splice(0..0, s.flatten()),
                    Pile::Empty => unreachable!(),
                };
                *se = Pile::Multiple(vec);
            }
            _ => unreachable!(),
        }
    }
    fn splice_at(&mut self, at: usize, other: Self) {
        match (self, other) {
            (Pile::Multiple(a), Pile::Multiple(b)) => {
                a.splice(at..at, b);
            }
            (Pile::Multiple(a), Pile::Single(b)) => {
                a.splice(at..at, b.flatten());
            }
            (se @ Pile::Single(_), o) => {
                let Pile::Single(s) = mem::take(se) else {
                    unreachable!()
                };
                let mut vec = s.flatten();
                match o {
                    Pile::Multiple(v) => vec.splice(at..at, v),
                    Pile::Single(s) => vec.splice(at..at, s.flatten()),
                    Pile::Empty => unreachable!(),
                };
                *se = Pile::Multiple(vec);
            }
            _ => unreachable!(),
        }
    }
    fn shuffle(&mut self, rng: &mut WyRand) {
        if let Pile::Multiple(v) = self {
            v.shuffle(rng)
        }
    }
    fn remove(&mut self, n: usize) -> SubCard {
        match self {
            Pile::Multiple(v) => {
                let ret = v.remove(n);
                self.set_single();
                ret
            }
            se @ Pile::Single(_) => {
                let Pile::Single(s) = mem::take(se) else {
                    unreachable!()
                };
                s.into()
            }
            Pile::Empty => unreachable!(),
        }
    }
    fn drain<R>(
        &mut self,
        range: R,
    ) -> Either<impl Iterator<Item = SubCard>, impl Iterator<Item = SubCard>>
    where
        R: RangeBounds<usize>,
    {
        match self {
            Pile::Multiple(v) => Either::Left(v.drain(range)),
            se @ Pile::Single(_) => {
                if matches!(range.start_bound(), Bound::Included(&0) | Bound::Unbounded)
                    && matches!(
                        range.end_bound(),
                        Bound::Included(&0) | Bound::Excluded(&1) | Bound::Unbounded
                    )
                {
                    let Pile::Single(s) = mem::take(se) else {
                        unreachable!()
                    };
                    Either::Right(iter::once(s.into()))
                } else {
                    unreachable!()
                }
            }
            Pile::Empty => unreachable!(),
        }
    }
    fn iter(&self) -> Either<Iter<'_, SubCard>, CardIter<'_>> {
        match self {
            Pile::Multiple(v) => Either::Left(v.iter()),
            Pile::Single(s) => Either::Right(s.iter()),
            Pile::Empty => unreachable!(),
        }
    }
    fn iter_equipment(&self) -> Iter<'_, SubCard> {
        match self {
            Pile::Multiple(_) => unreachable!(),
            Pile::Single(s) => s.equiped.iter(),
            Pile::Empty => unreachable!(),
        }
    }
    fn iter_mut(&mut self) -> Either<IterMut<'_, SubCard>, CardIterMut<'_>> {
        match self {
            Pile::Multiple(v) => Either::Left(v.iter_mut()),
            Pile::Single(s) => Either::Right(s.iter_mut()),
            Pile::Empty => unreachable!(),
        }
    }
}
#[derive(Debug, Clone)]
enum DeckType {
    Other(Vec2, SyncObject),
    Single(Vec2),
    Token,
    Deck,
    Commander,
    SideBoard,
}
#[derive(Resource, Debug, Default, Clone)]
struct GetDeck(Arc<Mutex<Vec<(Pile, DeckType)>>>);
#[derive(Debug, Default, Clone, Encode, Decode)]
#[allow(dead_code)]
struct CardInfo {
    name: String,
    mana_cost: Cost,
    card_type: Types,
    text: String,
    color: Color,
    power: u16,
    toughness: u16,
    #[bitcode(skip)]
    image: UninitImage,
}
impl CardInfo {
    fn clone_no_image(&self) -> Self {
        Self {
            name: self.name.clone(),
            mana_cost: self.mana_cost,
            card_type: self.card_type.clone(),
            text: self.text.clone(),
            color: self.color,
            power: self.power,
            toughness: self.toughness,
            image: default(),
        }
    }
}
#[derive(Debug, Clone, Default)]
struct UninitImage(Option<Handle<Image>>);
impl From<Handle<Image>> for UninitImage {
    fn from(value: Handle<Image>) -> Self {
        Self(Some(value))
    }
}
impl UninitImage {
    fn clone_handle(&self) -> Handle<Image> {
        self.handle().clone()
    }
    fn handle(&self) -> &Handle<Image> {
        self.0.as_ref().unwrap()
    }
}
impl CardInfo {
    fn clone_image(&self) -> Handle<Image> {
        self.image.clone_handle()
    }
}
impl Type {
    #[allow(dead_code)]
    fn is_permanent(&self) -> bool {
        !matches!(self, Self::Instant | Self::Sorcery)
    }
}
#[allow(dead_code)]
#[derive(Debug, Default, Clone, Encode, Decode)]
struct Types {
    super_type: Vec<SuperType>,
    main_type: Vec<Type>,
    sub_type: Vec<SubType>,
}
impl Types {
    #[allow(dead_code)]
    fn is_permanent(&self) -> bool {
        self.main_type
            .first()
            .map(|a| a.is_permanent())
            .unwrap_or(false)
    }
}
impl FromStr for Types {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ret = Self::default();
        for word in s.split(' ') {
            if let Ok(super_type) = SuperType::from_str(word) {
                ret.super_type.push(super_type)
            } else if let Ok(ty) = Type::from_str(word) {
                ret.main_type.push(ty)
            } else if let Ok(sub_type) = SubType::from_str(word) {
                ret.sub_type.push(sub_type)
            }
        }
        Ok(ret)
    }
}
#[derive(Debug, Default, Clone, Copy, Encode, Decode)]
struct Color {
    white: bool,
    blue: bool,
    black: bool,
    red: bool,
    green: bool,
}
impl Color {
    fn parse<'a>(value: impl Iterator<Item = &'a str>) -> Self {
        let mut cost = Self::default();
        for c in value {
            match c {
                "W" => cost.white = true,
                "U" => cost.blue = true,
                "B" => cost.black = true,
                "R" => cost.red = true,
                "G" => cost.green = true,
                _ => unreachable!(),
            }
        }
        cost
    }
}
#[derive(Debug, Default, Clone, Copy, Encode, Decode)]
struct Cost {
    white: u8,
    blue: u8,
    black: u8,
    red: u8,
    green: u8,
    colorless: u8,
    any: u8,
    pay: u8,
    var: u8,
    total: u8,
}
impl From<&str> for Cost {
    fn from(value: &str) -> Self {
        let mut cost = Self::default();
        if value.is_empty() {
            return cost;
        }
        let value = &value[1..value.len() - 1];
        for c in value.split("}{") {
            cost.total += 1;
            for c in c.split('/') {
                match c {
                    "W" => cost.white += 1,
                    "U" => cost.blue += 1,
                    "B" => cost.black += 1,
                    "R" => cost.red += 1,
                    "G" => cost.green += 1,
                    "C" => cost.colorless += 1,
                    "P" => {
                        cost.total -= 1;
                        cost.pay += 1
                    }
                    "X" => {
                        cost.total -= 1;
                        cost.var += 1
                    }
                    c => {
                        cost.total -= 1;
                        cost.total += c.parse::<u8>().unwrap();
                        cost.any += c.parse::<u8>().unwrap()
                    }
                }
            }
        }
        cost
    }
}
#[derive(Debug, Default, Clone, Copy, Encode, Decode)]
enum Layout {
    #[default]
    Normal,
    Flip,
    Room,
}
#[derive(Default, PartialEq, Clone, Copy, Encode, Decode)]
struct Id(u128);
impl FromStr for Id {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(|a| Id(a.as_u128()))
    }
}
impl fmt::Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Uuid::from_u128(self.0))
    }
}
impl Debug for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
#[derive(Debug, Default, Clone, Encode, Decode)]
struct CardData {
    id: Id,
    tokens: Vec<Id>,
    face: CardInfo,
    back: Option<CardInfo>,
    layout: Layout,
}
impl CardData {
    fn clone_no_image(&self) -> Self {
        Self {
            id: self.id,
            tokens: self.tokens.clone(),
            face: self.face.clone_no_image(),
            back: self.back.as_ref().map(|a| a.clone_no_image()),
            layout: self.layout,
        }
    }
}
#[derive(Debug, Default, Clone, Encode, Decode)]
struct SubCard {
    data: CardData,
    flipped: bool,
}
impl SubCard {
    fn clone_no_image(&self) -> Self {
        Self {
            data: self.data.clone_no_image(),
            flipped: self.flipped,
        }
    }
    fn filter(&self, text: &str) -> bool {
        self.data.face.filter(text)
            || self
                .data
                .back
                .as_ref()
                .map(|a| a.filter(text))
                .unwrap_or(false)
    }
    fn face(&self) -> &CardInfo {
        if self.flipped {
            self.data.back.as_ref().unwrap()
        } else {
            &self.data.face
        }
    }
    fn back(&self) -> Option<&CardInfo> {
        if self.flipped {
            Some(&self.data.face)
        } else {
            self.data.back.as_ref()
        }
    }
    fn image_node(&self) -> ImageNode {
        if matches!(self.data.layout, Layout::Flip) && self.flipped {
            ImageNode {
                image: self.data.face.clone_image(),
                flip_x: true,
                flip_y: true,
                ..default()
            }
        } else {
            ImageNode::new(self.face().clone_image())
        }
    }
    fn material(&self) -> StandardMaterial {
        if matches!(self.data.layout, Layout::Flip) && self.flipped {
            StandardMaterial {
                base_color_texture: Some(self.data.face.clone_image()),
                unlit: true,
                uv_transform: StandardMaterial::FLIP_VERTICAL * StandardMaterial::FLIP_HORIZONTAL,
                ..default()
            }
        } else {
            StandardMaterial {
                base_color_texture: Some(self.face().clone_image()),
                unlit: true,
                ..default()
            }
        }
    }
}
#[derive(Debug, Default, Clone, Encode, Decode)]
struct Card {
    subcard: SubCard,
    equiped: Vec<SubCard>,
    #[allow(dead_code)]
    power: Option<i32>,
    #[allow(dead_code)]
    health: Option<i32>,
    #[allow(dead_code)]
    loyalty: Option<i32>,
    #[allow(dead_code)]
    misc: Option<i32>,
    #[allow(dead_code)]
    is_token: bool,
}
impl Card {
    fn clone_no_image(&self) -> Self {
        Self {
            subcard: self.subcard.clone_no_image(),
            equiped: self.equiped.iter().map(|c| c.clone_no_image()).collect(),
            power: None,
            health: None,
            loyalty: None,
            misc: None,
            is_token: false,
        }
    }
    #[allow(dead_code)]
    fn filter(&self, text: &str) -> bool {
        self.subcard.filter(text)
    }
    fn flatten(mut self) -> Vec<SubCard> {
        let mut vec = Vec::with_capacity(self.equiped.len() + 1);
        let drain = mem::take(&mut self.equiped);
        vec.extend(drain);
        vec.push(self.into());
        vec
    }
    fn iter(&self) -> CardIter<'_> {
        CardIter {
            subcard: &self.subcard,
            equiped: self.equiped.iter(),
            started: false,
        }
    }
    fn iter_mut(&mut self) -> CardIterMut<'_> {
        CardIterMut {
            subcard: &mut self.subcard,
            equiped: self.equiped.iter_mut(),
            started: false,
        }
    }
    #[allow(dead_code)]
    fn get(&self, idx: usize) -> Option<&SubCard> {
        if idx == 0 {
            Some(&self.subcard)
        } else {
            self.equiped.get(idx - 1)
        }
    }
    fn get_mut(&mut self, idx: usize) -> Option<&mut SubCard> {
        if idx == 0 {
            Some(&mut self.subcard)
        } else {
            self.equiped.get_mut(idx - 1)
        }
    }
}
struct CardIter<'a> {
    subcard: &'a SubCard,
    equiped: Iter<'a, SubCard>,
    started: bool,
}
impl<'a> Iterator for CardIter<'a> {
    type Item = &'a SubCard;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.started {
            self.started = true;
            Some(self.subcard)
        } else {
            self.equiped.next()
        }
    }
}
impl<'a> DoubleEndedIterator for CardIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let back = self.equiped.next_back();
        if back.is_some() {
            back
        } else if !self.started {
            self.started = true;
            Some(self.subcard)
        } else {
            None
        }
    }
}
impl<'a> ExactSizeIterator for CardIter<'a> {
    fn len(&self) -> usize {
        1 + self.equiped.len()
    }
}
impl<'a> ExactSizeIterator for CardIterMut<'a> {
    fn len(&self) -> usize {
        1 + self.equiped.len()
    }
}
struct CardIterMut<'a> {
    subcard: *mut SubCard,
    equiped: IterMut<'a, SubCard>,
    started: bool,
}
impl<'a> Iterator for CardIterMut<'a> {
    type Item = &'a mut SubCard;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.started {
            self.started = true;
            unsafe { self.subcard.as_mut() }
        } else {
            self.equiped.next()
        }
    }
}
impl<'a> DoubleEndedIterator for CardIterMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let back = self.equiped.next_back();
        if back.is_some() {
            back
        } else if !self.started {
            self.started = true;
            unsafe { self.subcard.as_mut() }
        } else {
            None
        }
    }
}
impl From<Card> for SubCard {
    fn from(value: Card) -> Self {
        value.subcard
    }
}
impl From<Box<Card>> for SubCard {
    fn from(value: Box<Card>) -> Self {
        value.subcard
    }
}
impl From<SubCard> for Card {
    fn from(value: SubCard) -> Self {
        Self {
            subcard: value,
            equiped: Vec::new(),
            power: None,
            health: None,
            loyalty: None,
            misc: None,
            is_token: false,
        }
    }
}
impl From<SubCard> for Box<Card> {
    fn from(value: SubCard) -> Self {
        Box::new(Card {
            subcard: value,
            equiped: Vec::new(),
            power: None,
            health: None,
            loyalty: None,
            misc: None,
            is_token: false,
        })
    }
}
impl<'a> From<&'a Card> for &'a SubCard {
    fn from(value: &'a Card) -> Self {
        &value.subcard
    }
}
impl<'a> From<&'a mut Card> for &'a mut SubCard {
    fn from(value: &'a mut Card) -> Self {
        &mut value.subcard
    }
}
impl<'a> From<&'a Box<Card>> for &'a SubCard {
    fn from(value: &'a Box<Card>) -> Self {
        &value.subcard
    }
}
impl<'a> From<&'a mut Box<Card>> for &'a mut SubCard {
    fn from(value: &'a mut Box<Card>) -> Self {
        &mut value.subcard
    }
}
impl CardInfo {
    fn filter(&self, text: &str) -> bool {
        self.name
            .to_ascii_lowercase()
            .contains(&text.to_ascii_lowercase()) //TODO
    }
}
#[derive(Resource)]
struct Download {
    client: ReqClient,
    get_deck: GetDeck,
    #[cfg(not(feature = "wasm"))]
    runtime: Runtime,
}
#[derive(Resource, Clone)]
enum GameClipboard {
    Pile(Pile),
    Shape(Shape),
    None,
}
#[derive(Component, Default, Debug)]
struct FollowMouse;
#[derive(Component, Default, Debug)]
struct FollowOtherMouse;
#[derive(Component, Default, Debug)]
struct ZoomHold(u64, bool);
#[cfg(not(feature = "wasm"))]
#[derive(Resource)]
struct Clipboard(arboard::Clipboard);
#[cfg(feature = "wasm")]
#[cfg_attr(feature = "wasm", derive(Clone, Copy))]
#[derive(Resource)]
struct Clipboard;
impl Clipboard {
    #[cfg(not(feature = "wasm"))]
    fn get_text(&mut self) -> String {
        self.0.get_text().unwrap_or_default()
    }
    #[cfg(not(feature = "wasm"))]
    fn set_text(&mut self, string: &str) {
        self.0.set_text(string).unwrap_or_default()
    }
    #[cfg(feature = "wasm")]
    async fn get_text(&self) -> String {
        let window = web_sys::window().unwrap();
        let navigator = window.navigator();
        let clipboard = navigator.clipboard();
        JsFuture::from(clipboard.read_text())
            .await
            .unwrap()
            .as_string()
            .unwrap_or_default()
    }
    #[cfg(feature = "wasm")]
    async fn set_text(&self, text: &str) {
        let window = web_sys::window().unwrap();
        let navigator = window.navigator();
        let clipboard = navigator.clipboard();
        let _ = JsFuture::from(clipboard.write_text(text)).await;
    }
}
#[derive(Resource)]
struct ReqClient(reqwest::Client);
#[derive(Resource)]
struct Runtime(tokio::runtime::Runtime);
#[derive(Resource, Clone)]
struct CardBase {
    stock: Handle<Mesh>,
    back: Handle<StandardMaterial>,
    side: Handle<StandardMaterial>,
}
#[cfg(feature = "wasm")]
async fn get_image_bytes(url: &str) -> Option<(Vec<u8>, u32, u32)> {
    let img = HtmlImageElement::new().ok()?;
    img.set_cross_origin(Some("anonymous"));
    img.set_src(url);
    let (tx, rx) = oneshot::channel::<()>();
    let onload = Closure::once(Box::new(move || tx.send(()).unwrap()) as Box<dyn FnOnce()>);
    img.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();
    rx.await.unwrap();
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas: HtmlCanvasElement = document.create_element("canvas").ok()?.dyn_into().ok()?;
    canvas.set_width(img.width());
    canvas.set_height(img.height());
    let ctx: CanvasRenderingContext2d = canvas.get_context("2d").ok()?.unwrap().dyn_into().ok()?;
    ctx.draw_image_with_html_image_element(&img, 0.0, 0.0)
        .ok()?;
    let data = ctx
        .get_image_data(0.0, 0.0, img.width() as f64, img.height() as f64)
        .ok()?
        .data();
    Some((data.0, img.width(), img.height()))
}
