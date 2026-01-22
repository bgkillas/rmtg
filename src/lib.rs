use crate::events::Scroll;
use crate::events::*;
use crate::setup::{MAT_BAR, MAT_HEIGHT, MAT_WIDTH, setup, setup_net};
use crate::update::*;
use avian3d::prelude::*;
use bevy::asset::AssetMetaCheck;
//use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use bevy::ecs::system::SystemParam;
use bevy::image::{ImageFilterMode, ImageSamplerDescriptor};
use bevy::input_focus::InputFocus;
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;
use bevy::ui::UiSystems;
use bevy_framepace::FramepacePlugin;
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use bevy_rich_text3d::{LoadFonts, Text3dPlugin};
use bevy_tangled::{Client, PeerId};
use bevy_ui_text_input::TextInputPlugin;
use rand::RngCore;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Error, Formatter};
use std::hash::Hash;
use std::ops::{Bound, RangeBounds};
use std::slice::{Iter, IterMut};
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};
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
const PLAYER0: Color = Color::srgb_u8(255, 85, 85);
const PLAYER1: Color = Color::srgb_u8(85, 85, 255);
const PLAYER2: Color = Color::srgb_u8(255, 85, 255);
const PLAYER3: Color = Color::srgb_u8(85, 255, 85);
const PLAYER4: Color = Color::srgb_u8(85, 255, 255);
const PLAYER5: Color = Color::srgb_u8(255, 255, 85);
const PLAYER: [Color; 6] = [PLAYER0, PLAYER1, PLAYER2, PLAYER3, PLAYER4, PLAYER5];
pub mod counters;
pub mod download;
pub mod events;
pub mod misc;
pub mod setup;
pub mod shapes;
pub mod sync;
pub mod update;
use crate::counters::{Value, counter_hit};
use crate::misc::is_reversed;
use crate::shapes::Shape;
#[cfg(feature = "steam")]
use crate::sync::display_steam_info;
#[cfg(any(feature = "steam", feature = "ip"))]
use crate::sync::new_lobby;
use crate::sync::{SendSleeping, Sent, SyncCount, SyncObject, apply_sync, get_sync};
#[cfg(feature = "steam")]
use crate::update::update_rich;
#[cfg(feature = "mic")]
use bevy_microphone::{AudioResource, AudioSettings};
use bitcode::{Decode, Encode};
use enum_map::{Enum, EnumMap, enum_map};
use enumset::{EnumSet, EnumSetType, enum_set};
#[cfg(feature = "wasm")]
use futures::channel::oneshot;
use itertools::{Either, Itertools};
use rand::seq::SliceRandom;
#[cfg(feature = "mic")]
use rodio::{OutputStreamBuilder, Sink};
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
const FONT_SIZE: f32 = 32.0;
const FONT_HEIGHT: f32 = FONT_SIZE;
const FONT_WIDTH: f32 = FONT_HEIGHT * 3.0 / 5.0;
//TODO multi select
//TODO spawn stuff touching the floor
//TODO half card width between card spots
//TODO search does not scroll far down enough
//TODO card ban lists
//TODO commander damage
//TODO allow listing cards in side menu for tokens and printings instead of spawning in world
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
    #[cfg(feature = "mic")]
    let mut stream_handle = OutputStreamBuilder::open_default_stream().unwrap();
    #[cfg(feature = "mic")]
    stream_handle.log_on_drop(false);
    #[cfg(feature = "mic")]
    let sink = Sink::connect_new(stream_handle.mixer());
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: app_window,
                ..default()
            })
            .set(AssetPlugin {
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
            .set(ImagePlugin {
                default_sampler: ImageSamplerDescriptor {
                    mag_filter: ImageFilterMode::Linear,
                    min_filter: ImageFilterMode::Linear,
                    mipmap_filter: ImageFilterMode::Linear,
                    anisotropy_clamp: 16,
                    ..Default::default()
                },
            }),
        PhysicsPlugins::default(),
        PhysicsDebugPlugin,
        FramepacePlugin,
        EntropyPlugin::<WyRand>::default(),
        Text3dPlugin::default(),
        TextInputPlugin,
        MeshPickingPlugin,
        //FpsOverlayPlugin::default(),
    ))
    .insert_gizmo_config(
        PhysicsGizmos {
            axis_lengths: None,
            collider_color: Some(Color::srgba_u8(0, 0, 0, 127)),
            sleeping_color_multiplier: None,
            ..default()
        },
        GizmoConfig::default(),
    )
    .insert_resource(LoadFonts {
        font_embedded: vec![include_bytes!("../assets/noto.ttf")],
        ..default()
    })
    .insert_resource(clipboard)
    .insert_resource(Turn::default())
    .insert_resource(SyncCount::default())
    .insert_resource(Sent::default())
    .insert_resource(Peers::default())
    .insert_resource(RemPeers::default())
    .insert_resource(Menu::default())
    .insert_resource(GiveEnts::default())
    .insert_resource(FlipCounter::default())
    .insert_resource(SendSleeping::default())
    .insert_resource(VoiceActive::default())
    .insert_resource(KeybindsList::default())
    .insert_resource(game_clipboard)
    .insert_resource(CardList::default())
    .insert_resource(Download {
        client,
        #[cfg(not(feature = "wasm"))]
        runtime,
        get_deck,
    })
    .add_systems(Startup, (setup_net, setup).chain())
    .add_systems(
        Update,
        //TODO could be more parralized
        (
            (
                give_ents,
                flip_ents,
                (
                    #[cfg(feature = "steam")]
                    update_rich,
                    ping_drag,
                    untap_keybinds,
                    text_send,
                    #[cfg(feature = "mic")]
                    (voice_keybinds, voice_chat).chain(),
                    text_keybinds,
                    turn_keybinds,
                    set_card_spot,
                    send_scroll_events,
                    #[cfg(feature = "steam")]
                    display_steam_info,
                    listen_for_deck,
                    register_deck,
                    (cam_rotation, cam_translation).chain(),
                    esc_menu,
                    #[cfg(any(feature = "steam", feature = "ip"))]
                    new_lobby,
                    update_search_deck,
                    counter_hit,
                    (
                        pick_from_list,
                        gather_hand,
                        listen_for_mouse,
                        follow_mouse,
                        update_hand,
                    )
                        .chain(),
                ),
                reset_layers,
                (get_sync, apply_sync).chain(),
            )
                .chain(),
            rem_peers,
            scroll_to_bottom.after(UiSystems::Layout),
        ),
    )
    .add_observer(on_scroll_handler)
    .add_observer(move_to_floor)
    .add_observer(move_up)
    .add_observer(add_to_spot)
    .add_observer(pile_merge);
    #[cfg(feature = "mic")]
    let audio = AudioResource::new(&AudioSettings::default());
    #[cfg(feature = "mic")]
    audio.stop(true);
    #[cfg(feature = "mic")]
    app.insert_resource(AudioSettings::default())
        .insert_resource(audio)
        .insert_resource(AudioPlayer(sink));
    app.run()
}
#[derive(Resource, Deref, DerefMut, Default)]
pub struct CardList(HashMap<Id, CardData>);
#[cfg(feature = "mic")]
#[derive(Resource, Deref, DerefMut)]
pub struct AudioPlayer(Sink);
#[derive(Resource, Default, Debug)]
pub enum Menu {
    #[default]
    World,
    Counter,
    Esc,
    Side,
}
#[derive(SystemParam)]
pub struct Focus<'w> {
    menu: ResMut<'w, Menu>,
    active_input: ResMut<'w, InputFocus>,
    hover_map: Res<'w, HoverMap>,
}
impl<'w> Focus<'w> {
    pub fn key_lock(&self) -> bool {
        self.active_input
            .0
            .is_some_and(|e| e.to_bits() != u32::MAX as u64)
            || matches!(*self.menu, Menu::Esc)
    }
    pub fn mouse_lock(&self) -> bool {
        self.hover_map
            .values()
            .all(|a| a.keys().all(|e| e.to_bits() != u32::MAX as u64))
    }
}
const SLEEP: SleepThreshold = SleepThreshold {
    linear: 4.0 * CARD_THICKNESS,
    angular: 0.25,
};
#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct Turn(usize);
#[derive(Resource, Default, Debug)]
pub struct Peers {
    map: Arc<Mutex<HashMap<PeerId, usize>>>,
    me: Option<usize>,
    #[allow(dead_code)]
    names: HashMap<PeerId, String>,
    #[allow(dead_code)]
    name: Option<String>,
}
impl Peers {
    fn map(&self) -> MutexGuard<'_, HashMap<PeerId, usize>> {
        self.map.lock().unwrap()
    }
}
#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub struct RemPeers(Arc<Mutex<Vec<PeerId>>>);
#[derive(Component, Default, Debug, Deref, DerefMut)]
pub struct InHand(usize);
#[derive(Component, Default, Debug)]
pub struct Hand {
    count: usize,
    removed: Vec<usize>,
}
#[derive(Component, Default, Clone, Debug, Encode, Decode)]
pub enum Pile {
    Multiple(Vec<SubCard>),
    Single(Box<Card>),
    #[default]
    Empty,
}
impl Pile {
    #[allow(dead_code)]
    fn sort_by<F>(&mut self, sort: F)
    where
        F: FnMut(&SubCard, &SubCard) -> Ordering,
    {
        if let Pile::Multiple(v) = self {
            v.sort_by(sort)
        }
    }
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
                    toughness: None,
                    counters: None,
                    loyalty: None,
                    misc: None,
                    is_token: false,
                }));
                true
            }
            s @ Pile::Single(_) => {
                if let Pile::Single(c) = &s
                    && !c.equiped.is_empty()
                {
                    let Pile::Single(cards) = mem::take(s) else {
                        unreachable!();
                    };
                    *s = Pile::Multiple(cards.flatten());
                }
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
    fn is_modified(&self) -> bool {
        if let Pile::Single(s) = self {
            s.is_modified()
        } else {
            false
        }
    }
    fn has_counters(&self) -> bool {
        if let Pile::Single(s) = self {
            s.has_counters()
        } else {
            false
        }
    }
    fn merge(&mut self, to: Self) {
        let Pile::Single(mut top) = to else {
            unreachable!()
        };
        if !self.is_equiped() {
            self.equip();
        }
        let Pile::Single(s) = self else {
            unreachable!()
        };
        mem::swap(s, &mut top);
        s.equiped.splice(0..0, top.flatten());
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
    fn take_n_card(&mut self, transform: &Transform, n: usize) -> Vec<SubCard> {
        let ret = if is_reversed(transform) {
            self.drain(0..n.min(self.len())).collect()
        } else {
            self.drain(self.len().saturating_sub(n)..self.len())
                .rev()
                .collect()
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
    #[allow(dead_code)]
    fn insert(&mut self, n: usize, card: SubCard) {
        match self {
            Pile::Multiple(v) => v.insert(n, card),
            se @ Pile::Single(_) => {
                let Pile::Single(s) = mem::take(se) else {
                    unreachable!()
                };
                let mut v = s.flatten();
                if n == 0 {
                    v.insert(0, card)
                } else if n == 1 {
                    v.push(card)
                } else {
                    panic!()
                };
                *se = Pile::Multiple(v);
            }
            Pile::Empty => unreachable!(),
        }
    }
    fn drain<R>(
        &mut self,
        range: R,
    ) -> Either<impl DoubleEndedIterator<Item = SubCard>, impl DoubleEndedIterator<Item = SubCard>>
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
pub enum DeckType {
    Other(Transform, SyncObject),
    Single(Vec2),
    Deck,
    Commander,
    SideBoard,
    CommanderAlt,
    Companion,
    Sticker,
    Attraction,
}
#[derive(Resource, Debug, Default, Clone, Deref, DerefMut)]
pub struct GetDeck(Arc<Mutex<Vec<(Pile, DeckType)>>>);
#[derive(Debug, Default, Clone, Encode, Decode)]
#[allow(dead_code)]
pub struct CardInfo {
    name: String,
    mana_cost: Cost,
    card_type: Types,
    text: String,
    color: ColorIdentity,
    power: Option<u8>,
    toughness: Option<u8>,
    loyalty: Option<u8>,
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
            loyalty: self.loyalty,
            toughness: self.toughness,
            image: default(),
        }
    }
}
#[derive(Debug, Clone, Default, Deref, DerefMut)]
pub struct UninitImage(Option<Handle<Image>>);
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
#[derive(Debug, Default, Clone, Encode, Decode, Eq, PartialEq)]
pub struct Types {
    super_type: SuperTypes,
    main_type: MainTypes,
    sub_type: SubTypes,
}
impl Types {
    pub fn len(&self) -> usize {
        self.super_type.len() + self.main_type.len() + self.sub_type.len()
    }
    pub fn is_empty(&self) -> bool {
        self.super_type.is_empty() && self.main_type.is_empty() && self.sub_type.is_empty()
    }
}
//TODO also could be enum sets
#[derive(Debug, Default, Clone, Encode, Decode, Eq, PartialEq, Deref, DerefMut)]
pub struct SuperTypes(pub Vec<SuperType>);
#[derive(Debug, Default, Clone, Encode, Decode, Eq, PartialEq, Deref, DerefMut)]
pub struct MainTypes(pub Vec<Type>);
#[derive(Debug, Default, Clone, Encode, Decode, Eq, PartialEq, Deref, DerefMut)]
pub struct SubTypes(pub Vec<SubType>);
fn subset<T: PartialEq>(a: &[T], b: &[T]) -> bool {
    b.iter().all(|t| a.contains(t))
}
impl PartialOrd for Types {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let left = subset(&self.super_type, &other.super_type)
            && subset(&self.main_type, &other.main_type)
            && subset(&self.sub_type, &other.sub_type);
        let right = subset(&other.super_type, &self.super_type)
            && subset(&other.main_type, &self.main_type)
            && subset(&other.sub_type, &self.sub_type);
        if left && right {
            Some(Ordering::Equal)
        } else if left {
            Some(Ordering::Greater)
        } else if right {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}
impl PartialOrd for SuperTypes {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let left = subset(self, other);
        let right = subset(other, self);
        if left && right {
            Some(Ordering::Equal)
        } else if left {
            Some(Ordering::Greater)
        } else if right {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}
impl PartialOrd for MainTypes {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let left = subset(self, other);
        let right = subset(other, self);
        if left && right {
            Some(Ordering::Equal)
        } else if left {
            Some(Ordering::Greater)
        } else if right {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}
impl PartialOrd for SubTypes {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let left = subset(self, other);
        let right = subset(other, self);
        if left && right {
            Some(Ordering::Equal)
        } else if left {
            Some(Ordering::Greater)
        } else if right {
            Some(Ordering::Less)
        } else {
            None
        }
    }
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
impl FromStr for SuperTypes {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ret = Self::default();
        for word in s.split(' ') {
            if let Ok(super_type) = SuperType::from_str(word) {
                ret.push(super_type)
            }
        }
        Ok(ret)
    }
}
impl FromStr for MainTypes {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ret = Self::default();
        for word in s.split(' ') {
            if let Ok(main_type) = Type::from_str(word) {
                ret.push(main_type)
            }
        }
        Ok(ret)
    }
}
impl FromStr for SubTypes {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ret = Self::default();
        for word in s.split(' ') {
            if let Ok(sub_type) = SubType::from_str(word) {
                ret.push(sub_type)
            }
        }
        Ok(ret)
    }
}
//TODO enumset
#[derive(Debug, Default, Clone, Copy, Encode, Decode, PartialEq)]
pub struct ColorIdentity {
    white: bool,
    blue: bool,
    black: bool,
    red: bool,
    green: bool,
}
impl FromStr for ColorIdentity {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cost = Self::default();
        for c in s.chars() {
            match c {
                'w' => cost.white = true,
                'u' => cost.blue = true,
                'b' => cost.black = true,
                'r' => cost.red = true,
                'g' => cost.green = true,
                _ => return Err(()),
            }
        }
        Ok(cost)
    }
}
impl ColorIdentity {
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
impl PartialOrd for ColorIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        fn contains(a: bool, b: bool) -> bool {
            if b { a } else { true }
        }
        fn subset(a: &ColorIdentity, b: &ColorIdentity) -> bool {
            contains(a.white, b.white)
                && contains(a.blue, b.blue)
                && contains(a.black, b.black)
                && contains(a.red, b.red)
                && contains(a.green, b.green)
        }
        if self == other {
            Some(Ordering::Equal)
        } else if subset(self, other) {
            Some(Ordering::Greater)
        } else if subset(other, self) {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}
impl ColorIdentity {
    pub fn len(&self) -> usize {
        let mut n = 0;
        if self.white {
            n += 1
        };
        if self.blue {
            n += 1
        };
        if self.black {
            n += 1
        };
        if self.red {
            n += 1
        };
        if self.green {
            n += 1
        };
        n
    }
    pub fn is_empty(&self) -> bool {
        !self.white && !self.blue && !self.black && !self.red && !self.green
    }
}
#[derive(Debug, Default, Clone, Copy, Encode, Decode)]
pub struct Cost {
    white: u8,
    blue: u8,
    black: u8,
    red: u8,
    green: u8,
    colorless: u8,
    any: u8,
    pay: u8,
    var: u8,
}
impl From<&str> for Cost {
    fn from(value: &str) -> Self {
        let mut cost = Self::default();
        if value.is_empty() {
            return cost;
        }
        let value = &value[1..value.len() - 1];
        for c in value.split("}{") {
            for c in c.split('/') {
                match c {
                    "W" => cost.white += 1,
                    "U" => cost.blue += 1,
                    "B" => cost.black += 1,
                    "R" => cost.red += 1,
                    "G" => cost.green += 1,
                    "C" => cost.colorless += 1,
                    "P" => cost.pay += 1,
                    "X" => cost.var += 1,
                    c => cost.any += c.parse::<u8>().unwrap(),
                }
            }
        }
        cost
    }
}
impl Cost {
    pub fn total(&self) -> u8 {
        self.white + self.blue + self.black + self.red + self.green + self.colorless + self.any
    }
}
#[derive(Debug, Default, Clone, Copy, Encode, Decode)]
pub enum Layout {
    #[default]
    Normal,
    Flip,
    Room,
}
#[derive(Default, PartialEq, Clone, Copy, Encode, Decode, Eq, Hash)]
pub struct Id(u128);
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
pub struct CardData {
    face: CardInfo,
    back: Option<CardInfo>,
    layout: Layout,
}
impl CardData {
    fn clone_no_image(&self) -> Self {
        Self {
            face: self.face.clone_no_image(),
            back: self.back.as_ref().map(|a| a.clone_no_image()),
            layout: self.layout,
        }
    }
}
#[derive(Debug, Default, Clone, Encode, Decode)]
pub struct SubCard {
    id: Id,
    tokens: Vec<Id>,
    data: CardData, //this may be ommited instead getting data from the resource
    flipped: bool,
}
impl SubCard {
    fn clone_no_image(&self) -> Self {
        Self {
            id: self.id,
            tokens: self.tokens.clone(),
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
pub struct Card {
    subcard: SubCard,
    equiped: Vec<SubCard>,
    power: Option<Value>,
    toughness: Option<Value>,
    counters: Option<Value>,
    loyalty: Option<Value>,
    misc: Option<Value>,
    #[allow(dead_code)]
    is_token: bool,
}
impl Card {
    fn is_modified(&self) -> bool {
        !self.equiped.is_empty() || self.has_counters()
    }
    fn has_counters(&self) -> bool {
        self.power.is_some()
            || self.toughness.is_some()
            || self.counters.is_some()
            || self.loyalty.is_some()
            || self.misc.is_some()
    }
    fn clone_no_image(&self) -> Self {
        Self {
            subcard: self.subcard.clone_no_image(),
            equiped: self.equiped.iter().map(|c| c.clone_no_image()).collect(),
            power: None,
            toughness: None,
            counters: None,
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
pub struct CardIter<'a> {
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
pub struct CardIterMut<'a> {
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
            toughness: None,
            counters: None,
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
            loyalty: None,
            power: None,
            toughness: None,
            counters: None,
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
        let text = text.to_ascii_lowercase();
        let text = text.trim();
        let pairs = get_pairs(text);
        pairs
            .into_iter()
            .all(|(n, k, v, o)| self.filter_pair(n, k, v, o))
    }
    fn filter_pair(&self, negate: bool, key: SearchKey, value: &str, ordering: Order) -> bool {
        let res = match key {
            SearchKey::Name => self.name.to_ascii_lowercase().contains(value),
            SearchKey::Cmc => {
                if let Ok(v) = value.parse() {
                    self.mana_cost.total().cmp(&v) == ordering
                } else {
                    return false;
                }
            }
            SearchKey::Type => {
                if let Ok(count) = value.parse::<usize>() {
                    self.card_type.len() == count
                } else if let Ok(types) = value.parse()
                    && let Some(order) = self.card_type.partial_cmp(&types)
                {
                    order == ordering
                } else {
                    return false;
                }
            }
            SearchKey::SuperType => {
                if let Ok(count) = value.parse::<usize>() {
                    self.card_type.super_type.len() == count
                } else if let Ok(types) = value.parse()
                    && let Some(order) = self.card_type.super_type.partial_cmp(&types)
                {
                    order == ordering
                } else {
                    return false;
                }
            }
            SearchKey::MainType => {
                if let Ok(count) = value.parse::<usize>() {
                    self.card_type.main_type.len() == count
                } else if let Ok(types) = value.parse()
                    && let Some(order) = self.card_type.main_type.partial_cmp(&types)
                {
                    order == ordering
                } else {
                    return false;
                }
            }
            SearchKey::SubType => {
                if let Ok(count) = value.parse::<usize>() {
                    self.card_type.sub_type.len() == count
                } else if let Ok(types) = value.parse()
                    && let Some(order) = self.card_type.sub_type.partial_cmp(&types)
                {
                    order == ordering
                } else {
                    return false;
                }
            }
            SearchKey::Text => self.text.to_ascii_lowercase().contains(value),
            SearchKey::Color => {
                if let Ok(count) = value.parse::<usize>() {
                    self.color.len() == count
                } else if let Ok(types) = value.parse()
                    && let Some(order) = self.color.partial_cmp(&types)
                {
                    order == ordering
                } else {
                    return false;
                }
            }
            SearchKey::Power => {
                if let Some(power) = self.power
                    && let Ok(v) = value.parse()
                {
                    power.cmp(&v) == ordering
                } else {
                    return false;
                }
            }
            SearchKey::Loyalty => {
                if let Some(loyalty) = self.loyalty
                    && let Ok(v) = value.parse()
                {
                    loyalty.cmp(&v) == ordering
                } else {
                    return false;
                }
            }
            SearchKey::Toughness => {
                if let Some(toughness) = self.toughness
                    && let Ok(v) = value.parse()
                {
                    toughness.cmp(&v) == ordering
                } else {
                    return false;
                }
            }
        };
        if negate { !res } else { res }
    }
}
#[derive(Debug)]
pub enum Order {
    Greater,
    Less,
    Equal,
    GreaterEqual,
    LessEqual,
}
impl PartialEq<Order> for Ordering {
    fn eq(&self, other: &Order) -> bool {
        match other {
            Order::Greater => matches!(self, Ordering::Greater),
            Order::Less => matches!(self, Ordering::Less),
            Order::Equal => matches!(self, Ordering::Equal),
            Order::GreaterEqual => matches!(self, Ordering::Greater | Ordering::Equal),
            Order::LessEqual => matches!(self, Ordering::Less | Ordering::Equal),
        }
    }
}
fn get_pairs(text: &str) -> Vec<(bool, SearchKey, &str, Order)> {
    let mut quotes = false;
    let mut quoted = false;
    let mut order = None;
    let mut k = 0;
    let mut v = 0;
    let mut pairs = Vec::new();
    let mut key = None;
    let mut negate = false;
    for (i, c) in text.char_indices() {
        match c {
            '!' => negate = true,
            '\"' => {
                quoted = true;
                quotes = !quotes;
            }
            '=' if !quotes => {
                v = i + 1;
                if order.is_none() {
                    key = get_key(&text[if negate { k + 1 } else { k }..i]);
                    if key.is_some() {
                        order = Some(Order::Equal)
                    }
                } else if matches!(order, Some(Order::Greater)) {
                    order = Some(Order::GreaterEqual)
                } else if matches!(order, Some(Order::Less)) {
                    order = Some(Order::LessEqual)
                }
            }
            '<' if !quotes => {
                v = i + 1;
                if order.is_none() {
                    key = get_key(&text[if negate { k + 1 } else { k }..i]);
                    if key.is_some() {
                        order = Some(Order::Less)
                    }
                }
            }
            '>' if !quotes => {
                v = i + 1;
                if order.is_none() {
                    key = get_key(&text[if negate { k + 1 } else { k }..i]);
                    if key.is_some() {
                        order = Some(Order::Greater)
                    }
                }
            }
            ' ' if !quotes => {
                if let Some(order) = order
                    && let Some(key) = key
                {
                    pairs.push((
                        negate,
                        key,
                        if quoted {
                            &text[v + 1..i - 1]
                        } else {
                            &text[v..i]
                        },
                        order,
                    ));
                    k = i + 1;
                }
                order = None;
                quoted = false;
                negate = false;
            }
            _ => {}
        }
    }
    if let Some(order) = order
        && let Some(key) = key
    {
        pairs.push((
            negate,
            key,
            if quoted {
                &text[v + 1..text.len() - 1]
            } else {
                &text[v..]
            },
            order,
        ));
    } else {
        pairs.push((false, SearchKey::Name, &text[k..], Order::Equal));
    }
    pairs
}
fn get_key(key: &str) -> Option<SearchKey> {
    Some(match key {
        "name" | "n" => SearchKey::Name,
        "cmc" | "cost" => SearchKey::Cmc,
        "type" | "t" => SearchKey::Type,
        "super_type" | "ut" => SearchKey::SuperType,
        "main_type" | "mt" => SearchKey::MainType,
        "sub_type" | "st" => SearchKey::SubType,
        "text" | "o" => SearchKey::Text,
        "color" | "c" => SearchKey::Color,
        "power" | "p" => SearchKey::Power,
        "loyalty" | "l" => SearchKey::Loyalty,
        "toughness" | "h" => SearchKey::Toughness,
        _ => return None,
    })
}
#[derive(Debug, Clone, Copy)]
pub enum SearchKey {
    Name,
    Cmc,
    Type,
    SuperType,
    MainType,
    SubType,
    Text,
    Color,
    Power,
    Toughness,
    Loyalty,
}
#[derive(Resource)]
pub struct Download {
    client: ReqClient,
    get_deck: GetDeck,
    #[cfg(not(feature = "wasm"))]
    runtime: Runtime,
}
#[derive(Resource, Clone)]
pub enum GameClipboard {
    Pile(Pile),
    Shape(Shape),
    None,
}
#[derive(Component, Default, Debug)]
pub struct FollowMouse;
#[derive(Component, Default, Debug)]
pub struct FollowOtherMouse;
#[derive(Component, Default, Debug)]
pub struct ZoomHold(u64, bool);
#[cfg(not(feature = "wasm"))]
#[derive(Resource, Deref, DerefMut)]
pub struct Clipboard(arboard::Clipboard);
#[cfg(feature = "wasm")]
#[cfg_attr(feature = "wasm", derive(Clone, Copy))]
#[derive(Resource)]
pub struct Clipboard;
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
#[derive(Resource, Deref, DerefMut)]
pub struct ReqClient(reqwest::Client);
#[derive(Resource, Deref, DerefMut)]
pub struct Runtime(tokio::runtime::Runtime);
#[derive(Resource, Clone)]
pub struct CardBase {
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
//TODO maybe should be combined with focus
#[derive(SystemParam)]
pub struct Keybinds<'w> {
    keyboard: Res<'w, ButtonInput<KeyCode>>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    keybinds: ResMut<'w, KeybindsList>,
}
impl Keybinds<'_> {
    pub fn just_pressed(&self, keybind: Keybind) -> bool {
        self.keybinds[keybind].just_pressed(&self.keyboard, &self.mouse)
    }
    pub fn pressed(&self, keybind: Keybind) -> bool {
        self.keybinds[keybind].pressed(&self.keyboard, &self.mouse)
    }
    pub fn get_numeric(&self) -> usize {
        match DIGITS.iter().find(|n| self.keyboard.pressed(**n)) {
            Some(KeyCode::Digit0) | Some(KeyCode::Numpad0) => 0,
            Some(KeyCode::Digit1) | Some(KeyCode::Numpad1) => 1,
            Some(KeyCode::Digit2) | Some(KeyCode::Numpad2) => 2,
            Some(KeyCode::Digit3) | Some(KeyCode::Numpad3) => 3,
            Some(KeyCode::Digit4) | Some(KeyCode::Numpad4) => 4,
            Some(KeyCode::Digit5) | Some(KeyCode::Numpad5) => 5,
            Some(KeyCode::Digit6) | Some(KeyCode::Numpad6) => 6,
            Some(KeyCode::Digit7) | Some(KeyCode::Numpad7) => 7,
            Some(KeyCode::Digit8) | Some(KeyCode::Numpad8) => 8,
            Some(KeyCode::Digit9) | Some(KeyCode::Numpad9) => 9,
            _ => unreachable!(),
        }
    }
    #[allow(dead_code)]
    pub fn set(&mut self, keybind: Keybind) -> bool {
        if let Some(new) = Bind::new_from(&self.keyboard, &self.mouse) {
            self.keybinds[keybind] = new;
            true
        } else {
            false
        }
    }
}
#[derive(Enum, Debug)]
pub enum Keybind {
    Ping,
    HostSteam,
    HostIp,
    JoinIp,
    SortHand,
    Select,
    Flip,
    Shuffle,
    Remove,
    Copy,
    CopyObject,
    Paste,
    PasteObject,
    PickCard,
    Equip,
    RotateRight,
    RotateLeft,
    Spread,
    Printings,
    Tokens,
    Transform,
    Search,
    View,
    Sub,
    Add,
    Calc,
    Chat,
    Voice,
    TakeTurn,
    PassTurn,
    Menu,
    CalcClose,
    Left,
    Right,
    Up,
    Down,
    LeftFast,
    RightFast,
    UpFast,
    DownFast,
    Reset,
    Rotate,
    Untap,
    ScaleUp,
    ScaleDown,
    Mill,
    Exile,
    Reveal,
    Draw,
    Loyalty,
    Power,
    Toughness,
    MiscCounter,
    Counters,
}
#[derive(Resource, Deref, DerefMut)]
pub struct KeybindsList(EnumMap<Keybind, Bind>);
impl Default for KeybindsList {
    fn default() -> Self {
        let ctrl = Modifier::Control;
        let alt = Modifier::Alt;
        let shift = Modifier::Shift;
        Self(enum_map! {
            Keybind::Ping => Bind::new(enum_set!(), MouseButton::Middle),
            Keybind::Select => Bind::new(enum_set!(), MouseButton::Left),
            Keybind::Add => Bind::new(enum_set!(), MouseButton::Left),
            Keybind::Sub => Bind::new(enum_set!(), MouseButton::Right),
            Keybind::PickCard => Bind::new(enum_set!(ctrl), MouseButton::Left),
            Keybind::HostSteam => Bind::new(enum_set!(ctrl | alt | shift), KeyCode::KeyN),
            Keybind::HostIp => Bind::new(enum_set!(ctrl | alt | shift), KeyCode::KeyM),
            Keybind::JoinIp => Bind::new(enum_set!(ctrl | alt | shift), KeyCode::KeyK),
            Keybind::SortHand => Bind::new(enum_set!(ctrl), KeyCode::KeyS),
            Keybind::Flip => Bind::new(enum_set!(), KeyCode::KeyF),
            Keybind::Shuffle => Bind::new(enum_set!(), KeyCode::KeyR),
            Keybind::Calc => Bind::new(enum_set!(ctrl), KeyCode::KeyR),
            Keybind::Remove => Bind::new(enum_set!(), KeyCode::Delete),
            Keybind::Copy => Bind::new(enum_set!(ctrl), KeyCode::KeyC),
            Keybind::CopyObject => Bind::new(enum_set!(ctrl | shift), KeyCode::KeyC),
            Keybind::Paste => Bind::new(enum_set!(ctrl), KeyCode::KeyV),
            Keybind::PasteObject => Bind::new(enum_set!(ctrl | shift), KeyCode::KeyV),
            Keybind::Equip => Bind::new(enum_set!(ctrl), KeyCode::KeyE),
            Keybind::RotateLeft => Bind::new(enum_set!(), KeyCode::KeyQ),
            Keybind::RotateRight => Bind::new(enum_set!(), KeyCode::KeyE),
            Keybind::Spread => Bind::new(enum_set!(ctrl | alt | shift), KeyCode::KeyS),
            Keybind::Printings => Bind::new(enum_set!(ctrl | shift), KeyCode::KeyO),
            Keybind::Tokens => Bind::new(enum_set!(ctrl | shift), KeyCode::KeyT),
            Keybind::Transform => Bind::new(enum_set!(), KeyCode::KeyO),
            Keybind::Search => Bind::new(enum_set!(ctrl), KeyCode::KeyZ),
            Keybind::View => Bind::new(enum_set!(alt), Key::None),
            Keybind::Chat => Bind::new(enum_set!(), KeyCode::Enter),
            Keybind::Voice => Bind::new(enum_set!(), KeyCode::KeyB),
            Keybind::TakeTurn => Bind::new(enum_set!(ctrl), KeyCode::KeyX),
            Keybind::PassTurn => Bind::new(enum_set!(), KeyCode::KeyX),
            Keybind::Menu => Bind::new(enum_set!(), KeyCode::Escape),
            Keybind::CalcClose => Bind::new(enum_set!(), KeyCode::Enter),
            Keybind::Left => Bind::new(enum_set!(), KeyCode::KeyA),
            Keybind::Up => Bind::new(enum_set!(), KeyCode::KeyW),
            Keybind::Down => Bind::new(enum_set!(), KeyCode::KeyS),
            Keybind::Right => Bind::new(enum_set!(), KeyCode::KeyD),
            Keybind::LeftFast => Bind::new(enum_set!(shift), KeyCode::KeyA),
            Keybind::UpFast => Bind::new(enum_set!(shift), KeyCode::KeyW),
            Keybind::DownFast => Bind::new(enum_set!(shift), KeyCode::KeyS),
            Keybind::RightFast => Bind::new(enum_set!(shift), KeyCode::KeyD),
            Keybind::Reset => Bind::new(enum_set!(), KeyCode::Space),
            Keybind::Rotate => Bind::new(enum_set!(), MouseButton::Right),
            Keybind::Untap => Bind::new(enum_set!(), KeyCode::KeyU),
            Keybind::ScaleUp => Bind::new(enum_set!(), KeyCode::Equal),
            Keybind::ScaleDown => Bind::new(enum_set!(), KeyCode::Minus),
            Keybind::Mill => Bind::new(enum_set!(ctrl), Key::Numeric),
            Keybind::Exile => Bind::new(enum_set!(ctrl | shift), Key::Numeric),
            Keybind::Reveal => Bind::new(enum_set!(alt), Key::Numeric),
            Keybind::Draw => Bind::new(enum_set!(), Key::Numeric),
            Keybind::Loyalty => Bind::new(enum_set!(alt), KeyCode::KeyL),
            Keybind::Power => Bind::new(enum_set!(alt), KeyCode::KeyP),
            Keybind::Toughness => Bind::new(enum_set!(alt), KeyCode::KeyT),
            Keybind::MiscCounter => Bind::new(enum_set!(alt), KeyCode::KeyM),
            Keybind::Counters => Bind::new(enum_set!(alt), KeyCode::KeyC),
        })
    }
}
impl fmt::Display for KeybindsList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.iter()
                .map(|a| format!("{:?} => {}", a.0, a.1))
                .join("\n")
        )
    }
}
impl fmt::Display for Bind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{:?}",
            self.modifiers.iter().map(|m| format!("{m:?}+")).join(""),
            self.key
        )
    }
}
#[derive(PartialEq, Debug)]
pub enum Key {
    KeyCode(KeyCode),
    Mouse(MouseButton),
    Numeric,
    None,
}
impl From<KeyCode> for Key {
    fn from(value: KeyCode) -> Self {
        Self::KeyCode(value)
    }
}
impl From<MouseButton> for Key {
    fn from(value: MouseButton) -> Self {
        Self::Mouse(value)
    }
}
#[derive(EnumSetType, Debug)]
pub enum Modifier {
    Alt,
    Control,
    Shift,
    Super,
}
impl Modifier {
    pub fn pressed(&self, keyboard: &ButtonInput<KeyCode>) -> bool {
        keyboard.any_pressed(match self {
            Modifier::Alt => [KeyCode::AltLeft, KeyCode::AltRight],
            Modifier::Control => [KeyCode::ControlLeft, KeyCode::ControlRight],
            Modifier::Shift => [KeyCode::ShiftLeft, KeyCode::ShiftRight],
            Modifier::Super => [KeyCode::SuperLeft, KeyCode::SuperRight],
        })
    }
    #[allow(dead_code)]
    pub fn just_pressed(&self, keyboard: &ButtonInput<KeyCode>) -> bool {
        keyboard.any_just_pressed(match self {
            Modifier::Alt => [KeyCode::AltLeft, KeyCode::AltRight],
            Modifier::Control => [KeyCode::ControlLeft, KeyCode::ControlRight],
            Modifier::Shift => [KeyCode::ShiftLeft, KeyCode::ShiftRight],
            Modifier::Super => [KeyCode::SuperLeft, KeyCode::SuperRight],
        })
    }
}
impl TryFrom<&KeyCode> for Modifier {
    type Error = ();
    fn try_from(value: &KeyCode) -> Result<Self, Self::Error> {
        Ok(match value {
            KeyCode::AltLeft | KeyCode::AltRight => Modifier::Alt,
            KeyCode::ControlLeft | KeyCode::ControlRight => Modifier::Control,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => Modifier::Shift,
            KeyCode::SuperLeft | KeyCode::SuperRight => Modifier::Super,
            _ => return Err(()),
        })
    }
}
pub struct Bind {
    modifiers: EnumSet<Modifier>,
    key: Key,
}
impl From<KeyCode> for Bind {
    fn from(value: KeyCode) -> Self {
        Self {
            modifiers: default(),
            key: value.into(),
        }
    }
}
impl From<MouseButton> for Bind {
    fn from(value: MouseButton) -> Self {
        Self {
            modifiers: default(),
            key: value.into(),
        }
    }
}
impl Bind {
    #[allow(dead_code)]
    pub fn new_from(
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> Option<Self> {
        let mut modifiers = EnumSet::empty();
        for modifier in keyboard.get_pressed().flat_map(|k| k.try_into().ok()) {
            modifiers.insert(modifier);
        }
        let mut mouse_pressed = mouse.get_just_pressed();
        let mouse = mouse_pressed.next();
        let mut keyboard_pressed = keyboard
            .get_just_pressed()
            .filter(|k| Modifier::try_from(*k).is_err());
        let keyboard = keyboard_pressed.next();
        if let Some(key) = mouse.copied() {
            if mouse_pressed.next().is_some() {
                return None;
            }
            Some(Self {
                modifiers,
                key: key.into(),
            })
        } else if let Some(key) = keyboard.copied() {
            if keyboard_pressed.next().is_some() {
                return None;
            }
            Some(Self {
                modifiers,
                key: key.into(),
            })
        } else {
            None
        }
    }
    pub fn new(modifiers: EnumSet<Modifier>, key: impl Into<Key>) -> Self {
        Self {
            modifiers,
            key: key.into(),
        }
    }
    pub fn modifiers_pressed(&self, keyboard: &ButtonInput<KeyCode>) -> bool {
        self.modifiers.iter().all(|m| m.pressed(keyboard))
        /*&& keyboard.get_pressed().all(|k| {
            if let Ok(m) = k.try_into() {
                self.modifiers.contains(m)
            } else {
                true
            }
        })*/
    }
    pub fn just_pressed(
        &self,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        (match self.key {
            Key::KeyCode(key) => keyboard.just_pressed(key),
            Key::Mouse(button) => mouse.just_pressed(button),
            Key::None => self.modifiers.iter().all(|m| m.just_pressed(keyboard)),
            Key::Numeric => DIGITS.iter().any(|n| keyboard.just_pressed(*n)),
        }) && self.modifiers_pressed(keyboard)
    }
    pub fn pressed(
        &self,
        keyboard: &ButtonInput<KeyCode>,
        mouse: &ButtonInput<MouseButton>,
    ) -> bool {
        (match self.key {
            Key::KeyCode(key) => keyboard.pressed(key),
            Key::Mouse(button) => mouse.pressed(button),
            Key::None => true,
            Key::Numeric => DIGITS.iter().any(|n| keyboard.pressed(*n)),
        }) && self.modifiers_pressed(keyboard)
    }
}
const DIGITS: &[KeyCode] = &[
    KeyCode::Digit0,
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
    KeyCode::Numpad0,
    KeyCode::Numpad1,
    KeyCode::Numpad2,
    KeyCode::Numpad3,
    KeyCode::Numpad4,
    KeyCode::Numpad5,
    KeyCode::Numpad6,
    KeyCode::Numpad7,
    KeyCode::Numpad8,
    KeyCode::Numpad9,
];
