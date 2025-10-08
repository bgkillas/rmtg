use crate::setup::setup;
use crate::update::{
    ToMoveUp, cam_rotation, cam_translation, esc_menu, follow_mouse, gather_hand, listen_for_deck,
    listen_for_mouse, register_deck, to_move_up, update_hand, update_search_deck,
};
use avian3d::prelude::*;
use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;
use bevy_framepace::FramepacePlugin;
use bevy_prng::WyRand;
use bevy_rand::prelude::EntropyPlugin;
use bevy_rich_text3d::{LoadFonts, Text3dPlugin};
use bevy_ui_text_input::TextInputPlugin;
use bitcode::{Decode, Encode};
use net::Client;
use rand::RngCore;
use std::mem::MaybeUninit;
use std::sync::{Arc, Mutex};
pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
pub const CARD_WIDTH: f32 = 488.0;
pub const CARD_HEIGHT: f32 = 680.0;
pub const START_Y: f32 = 8192.0;
pub const START_Z: f32 = 4096.0;
pub const GRAVITY: f32 = 512.0;
pub const LIN_DAMPING: f32 = 0.25;
pub const ANG_DAMPING: f32 = 0.25;
mod download;
mod misc;
mod setup;
mod shapes;
pub mod sync;
mod update;
use crate::shapes::Shape;
#[cfg(feature = "steam")]
use crate::sync::display_steam_info;
#[cfg(all(feature = "steam", feature = "ip"))]
use crate::sync::new_lobby;
use crate::sync::{SendSleeping, Sent, SyncActions, SyncCount, SyncObject, apply_sync, get_sync};
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;
#[cfg(feature = "wasm")]
use wasm_bindgen_futures::JsFuture;
#[cfg(feature = "steam")]
const APPID: u32 = 480; // 4046880
const FONT_SIZE: f32 = 16.0;
const FONT_HEIGHT: f32 = FONT_SIZE;
const FONT_WIDTH: f32 = FONT_HEIGHT * 3.0 / 5.0;
#[cfg_attr(feature = "wasm", wasm_bindgen(start))]
pub fn start() {
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
    .insert_resource(LoadFonts {
        font_embedded: vec![include_bytes!("../assets/noto.ttf")],
        ..default()
    })
    .insert_resource(clipboard)
    .insert_resource(ToMoveUp(Vec::new()))
    .insert_resource(SyncCount::default())
    .insert_resource(Sent::default())
    .insert_resource(Menu::default())
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
        )
            .chain(),
    )
    .add_systems(PreUpdate, (get_sync, apply_sync).chain());
    app.run();
}
#[derive(Resource, Default)]
pub enum Menu {
    #[default]
    World,
    Esc,
    Side,
}
pub const SLEEP: SleepThreshold = SleepThreshold {
    linear: 8.0,
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
                    Vec2::default(),
                )
                .await
            }))
            .unwrap();
        let deck = decks.0.lock().unwrap();
        assert_eq!(deck.len(), 4);
        println!(
            "{} {} {} {} {}",
            tmr.elapsed().as_millis(),
            deck[0].0.0.len(),
            deck[1].0.0.len(),
            deck[2].0.0.len(),
            deck[3].0.0.len()
        );
    }
    app.add_systems(Update, test);
    app.update();
}
#[derive(Component, Default, Debug)]
pub struct Owned;
#[derive(Component, Default, Debug)]
pub struct InHand(pub usize);
#[allow(dead_code)]
#[derive(Component, Default, Debug)]
pub struct Hand {
    pub id: usize,
    pub count: usize,
    pub removed: Vec<usize>,
}
#[derive(Component, Default, Debug, Clone, Encode, Decode)]
pub struct Pile(pub Vec<Card>);
impl Pile {
    fn clone_no_image(&self) -> Self {
        Pile(self.0.iter().map(|a| a.clone_no_image()).collect())
    }
}
#[derive(Resource, Debug, Default, Clone)]
pub struct GetDeck(pub Arc<Mutex<Vec<(Pile, Vec2, Option<SyncObject>)>>>);
#[derive(Debug, Default, Clone, Encode, Decode)]
#[allow(dead_code)]
pub struct CardInfo {
    pub name: String,
    pub mana_cost: Cost,
    pub card_type: Types,
    pub text: String,
    pub color: Color,
    pub power: u16,
    pub toughness: u16,
    #[bitcode(skip)]
    image: UninitImage,
}
impl CardInfo {
    pub fn clone_no_image(&self) -> Self {
        Self {
            name: self.name.clone(),
            mana_cost: self.mana_cost,
            card_type: self.card_type,
            text: self.text.clone(),
            color: self.color,
            power: self.power,
            toughness: self.toughness,
            image: default(),
        }
    }
}
#[derive(Debug)]
struct UninitImage(MaybeUninit<Handle<Image>>);
impl From<Handle<Image>> for UninitImage {
    fn from(value: Handle<Image>) -> Self {
        Self(MaybeUninit::new(value))
    }
}
impl Clone for UninitImage {
    fn clone(&self) -> Self {
        unsafe { self.0.assume_init_ref().clone().into() }
    }
}
impl Default for UninitImage {
    fn default() -> Self {
        Self(MaybeUninit::uninit())
    }
}
impl CardInfo {
    pub fn image(&self) -> &Handle<Image> {
        unsafe { self.image.0.assume_init_ref() }
    }
}
#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy, Encode, Decode)]
pub enum SuperType {
    Basic,
    Legendary,
    Ongoing,
    Snow,
    World,
    #[default]
    None,
}
#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy, Encode, Decode)]
pub enum SubType {
    Equipment,
    Fortification,
    Vehicle,
    Wall,
    Aura,
    Background,
    Saga,
    Plains,
    Island,
    Swamp,
    Mountain,
    Forest,
    Cave,
    Desert,
    Gate,
    Lair,
    Locus,
    Mine,
    PowerPlant,
    Sphere,
    Tower,
    Urzas,
    #[default]
    None,
}
impl Type {
    #[allow(dead_code)]
    pub fn is_permanent(&self) -> bool {
        !matches!(self, Self::Instant | Self::Sorcery)
    }
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}
#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy, Encode, Decode)]
pub enum Type {
    Land,
    Creature,
    Artifact,
    Enchantment,
    PlanesWalker,
    Battle,
    Instant,
    Sorcery,
    Kindred,
    #[default]
    None,
}
#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy, Encode, Decode)]
pub struct Types {
    pub super_type: SuperType,
    pub main_type: Type,
    pub alt_type: Type,
    pub creature_type: CreatureType,
    pub creature_alt_type: CreatureType,
    pub creature_extra_alt_type: CreatureType,
    pub sub_type: SubType,
}
impl Types {
    #[allow(dead_code)]
    pub fn is_permanent(&self) -> bool {
        self.main_type.is_permanent() || self.alt_type.is_permanent()
    }
}
impl From<&str> for Types {
    fn from(value: &str) -> Self {
        let mut ret = Self::default();
        let mut push_main = |s: Type| {
            if ret.main_type.is_some() {
                ret.alt_type = s
            } else {
                ret.main_type = s
            }
        };
        for word in value.split(' ') {
            match word {
                "Land" => push_main(Type::Land),
                "Creature" => push_main(Type::Creature),
                "Artifact" => push_main(Type::Artifact),
                _ => {} //TODO
            }
        }
        ret
    }
}
#[derive(Debug, Default, Clone, Copy, Encode, Decode)]
pub struct Color {
    pub white: bool,
    pub blue: bool,
    pub black: bool,
    pub red: bool,
    pub green: bool,
}
impl Color {
    pub fn parse<'a>(value: impl Iterator<Item = &'a str>) -> Self {
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
pub struct Cost {
    pub white: u8,
    pub blue: u8,
    pub black: u8,
    pub red: u8,
    pub green: u8,
    pub colorless: u8,
    pub any: u8,
    pub pay: u8,
    pub var: u8,
    pub total: u8,
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
                    "P" => cost.pay += 1,
                    "X" => cost.var += 1,
                    _ => cost.any += c.parse::<u8>().unwrap(),
                }
            }
        }
        cost
    }
}
#[derive(Debug, Default, Clone, Encode, Decode)]
pub struct Card {
    pub id: String,
    pub normal: CardInfo,
    pub alt: Option<CardInfo>,
    pub is_alt: bool,
}
impl Card {
    pub fn clone_no_image(&self) -> Self {
        Self {
            id: self.id.clone(),
            normal: self.normal.clone_no_image(),
            alt: self.alt.as_ref().map(|a| a.clone_no_image()),
            is_alt: self.is_alt,
        }
    }
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
pub struct ZoomHold(pub u64, pub bool);
#[cfg(not(feature = "wasm"))]
#[derive(Resource)]
pub struct Clipboard(pub arboard::Clipboard);
#[cfg(feature = "wasm")]
#[cfg_attr(feature = "wasm", derive(Clone, Copy))]
#[derive(Resource)]
pub struct Clipboard;
impl Clipboard {
    #[cfg(not(feature = "wasm"))]
    pub fn get_text(&mut self) -> String {
        self.0.get_text().unwrap_or_default()
    }
    #[cfg(not(feature = "wasm"))]
    pub fn set_text(&mut self, string: &str) {
        self.0.set_text(string).unwrap_or_default()
    }
    #[cfg(feature = "wasm")]
    pub async fn get_text(&self) -> String {
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
    pub async fn set_text(&self, text: &str) {
        let window = web_sys::window().unwrap();
        let navigator = window.navigator();
        let clipboard = navigator.clipboard();
        let _ = JsFuture::from(clipboard.write_text(text)).await;
    }
}
#[derive(Resource)]
pub struct ReqClient(pub reqwest::Client);
#[derive(Resource)]
pub struct Runtime(pub tokio::runtime::Runtime);
#[derive(Resource)]
pub struct CardBase {
    pub stock: Handle<Mesh>,
    back: Handle<StandardMaterial>,
    side: Handle<StandardMaterial>,
}
#[derive(Debug, Default, Clone, Copy, Encode, Decode)]
pub enum CreatureType {
    TimeLord,
    Advisor,
    Aetherborn,
    Alien,
    Ally,
    Angel,
    Antelope,
    Ape,
    Archer,
    Archon,
    Armadillo,
    Army,
    Artificer,
    Assassin,
    AssemblyWorker,
    Astartes,
    Atog,
    Aurochs,
    Avatar,
    Azra,
    Badger,
    Balloon,
    Barbarian,
    Bard,
    Basilisk,
    Bat,
    Bear,
    Beast,
    Beaver,
    Beeble,
    Beholder,
    Berserker,
    Bird,
    Blinkmoth,
    Boar,
    Bringer,
    Brushwagg,
    Camarid,
    Camel,
    Capybara,
    Caribou,
    Carrier,
    Cat,
    Centaur,
    Child,
    Chimera,
    Citizen,
    Cleric,
    Clown,
    Cockatrice,
    Construct,
    Coward,
    Coyote,
    Crab,
    Crocodile,
    Ctan,
    Custodes,
    Cyberman,
    Cyclops,
    Dalek,
    Dauthi,
    Demigod,
    Demon,
    Deserter,
    Detective,
    Devil,
    Dinosaur,
    Djinn,
    Doctor,
    Dog,
    Dragon,
    Drake,
    Dreadnought,
    Drone,
    Druid,
    Dryad,
    Dwarf,
    Efreet,
    Egg,
    Elder,
    Eldrazi,
    Elemental,
    Elephant,
    Elf,
    Elk,
    Employee,
    Eye,
    Faerie,
    Ferret,
    Fish,
    Flagbearer,
    Fox,
    Fractal,
    Frog,
    Fungus,
    Gamer,
    Gargoyle,
    Germ,
    Giant,
    Gith,
    Glimmer,
    Gnoll,
    Gnome,
    Goat,
    Goblin,
    God,
    Golem,
    Gorgon,
    Graveborn,
    Gremlin,
    Griffin,
    Guest,
    Hag,
    Halfling,
    Hamster,
    Harpy,
    Hellion,
    Hippo,
    Hippogriff,
    Homarid,
    Homunculus,
    Horror,
    Horse,
    Human,
    Hydra,
    Hyena,
    Illusion,
    Imp,
    Incarnation,
    Inkling,
    Inquisitor,
    Insect,
    Jackal,
    Jellyfish,
    Juggernaut,
    Kavu,
    Kirin,
    Kithkin,
    Knight,
    Kobold,
    Kor,
    Kraken,
    Llama,
    Lamia,
    Lammasu,
    Leech,
    Leviathan,
    Lhurgoyf,
    Licid,
    Lizard,
    Manticore,
    Masticore,
    Mercenary,
    Merfolk,
    Metathran,
    Minion,
    Minotaur,
    Mite,
    Mole,
    Monger,
    Mongoose,
    Monk,
    Monkey,
    Moonfolk,
    Mount,
    Mouse,
    Mutant,
    Myr,
    Mystic,
    Nautilus,
    Necron,
    Nephilim,
    Nightmare,
    Nightstalker,
    Ninja,
    Noble,
    Noggle,
    Nomad,
    Nymph,
    Octopus,
    Ogre,
    Ooze,
    Orb,
    Orc,
    Orgg,
    Otter,
    Ouphe,
    Ox,
    Oyster,
    Pangolin,
    Peasant,
    Pegasus,
    Pentavite,
    Performer,
    Pest,
    Phelddagrif,
    Phoenix,
    Phyrexian,
    Pilot,
    Pincher,
    Pirate,
    Plant,
    Porcupine,
    Possum,
    Praetor,
    Primarch,
    Prism,
    Processor,
    Rabbit,
    Raccoon,
    Ranger,
    Rat,
    Rebel,
    Reflection,
    Rhino,
    Rigger,
    Robot,
    Rogue,
    Sable,
    Salamander,
    Samurai,
    Sand,
    Saproling,
    Satyr,
    Scarecrow,
    Scientist,
    Scion,
    Scorpion,
    Scout,
    Sculpture,
    Serf,
    Serpent,
    Servo,
    Shade,
    Shaman,
    Shapeshifter,
    Shark,
    Sheep,
    Siren,
    Skeleton,
    Skunk,
    Slith,
    Sliver,
    Sloth,
    Slug,
    Snail,
    Snake,
    Soldier,
    Soltari,
    Spawn,
    Specter,
    Spellshaper,
    Sphinx,
    Spider,
    Spike,
    Spirit,
    Splinter,
    Sponge,
    Squid,
    Squirrel,
    Starfish,
    Surrakar,
    Survivor,
    Synth,
    Tentacle,
    Tetravite,
    Thalakos,
    Thopter,
    Thrull,
    Tiefling,
    Toy,
    Treefolk,
    Trilobite,
    Triskelavite,
    Troll,
    Turtle,
    Tyranid,
    Unicorn,
    Vampire,
    Varmint,
    Vedalken,
    Volver,
    Wall,
    Walrus,
    Warlock,
    Warrior,
    Weasel,
    Weird,
    Werewolf,
    Whale,
    Wizard,
    Wolf,
    Wolverine,
    Wombat,
    Worm,
    Wraith,
    Wurm,
    Yeti,
    Zombie,
    Zubera,
    #[default]
    None,
    All,
}
