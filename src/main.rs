mod download;
mod misc;
mod setup;
mod update;
use crate::setup::setup;
use crate::update::{
    cam_rotation, cam_translation, follow_mouse, listen_for_deck, listen_for_mouse, register_deck,
};
use bevy::prelude::*;
use bevy_framepace::FramepacePlugin;
use bevy_prng::WyRand;
use bevy_rand::global::GlobalEntropy;
use bevy_rand::prelude::EntropyPlugin;
use bevy_rapier3d::prelude::*;
use rand::RngCore;
use std::sync::LazyLock;
use tokio::task::JoinHandle;
static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
static CARD_WIDTH: f32 = 488.0;
static CARD_HEIGHT: f32 = 680.0;
static START_Y: f32 = 8192.0;
static START_Z: f32 = 4096.0;
static GRAVITY: f32 = 256.0;
static DAMPING: f32 = 4.0;
fn main() {
    let runtime = Runtime(tokio::runtime::Runtime::new().unwrap());
    let client = Client(
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .unwrap(),
    );
    let clipboard = Clipboard(LazyLock::new(|| arboard::Clipboard::new().unwrap()));
    let app_window = Some(Window {
        title: "rmtg".into(),
        ..default()
    });
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: app_window,
                ..default()
            }),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
            FramepacePlugin,
            EntropyPlugin::<WyRand>::default(),
        ))
        .insert_resource(clipboard)
        .insert_resource(runtime)
        .insert_resource(client)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                listen_for_deck,
                register_deck,
                cam_translation,
                cam_rotation,
                (listen_for_mouse, follow_mouse).chain(),
            ),
        )
        .run();
}
#[test]
fn test_parse() {
    let mut app = App::new();
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    fn test(asset_server: Res<AssetServer>) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let tmr = std::time::Instant::now();
        let asset_server = asset_server.clone();
        let img = runtime
            .block_on(runtime.spawn(async move {
                let mut json = json::object!(id: "OxoZm", name: "kilo");
                download::parse(
                    &mut json,
                    reqwest::Client::builder()
                        .user_agent(USER_AGENT)
                        .build()
                        .unwrap(),
                    asset_server,
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
fn test_get_deck() {
    let mut app = App::new();
    app.add_plugins(AssetPlugin::default());
    app.init_asset::<Image>();
    fn test(asset_server: Res<AssetServer>) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let tmr = std::time::Instant::now();
        let asset_server = asset_server.clone();
        let deck = runtime
            .block_on(runtime.spawn(async move {
                download::get_deck(
                    "https://api2.moxfield.com/v3/decks/all/_HGo1kgcB0i-4Iq0vR-LZA".to_string(),
                    reqwest::Client::builder()
                        .user_agent(USER_AGENT)
                        .build()
                        .unwrap(),
                    asset_server,
                )
                .await
            }))
            .unwrap();
        assert!(deck.is_some());
        let deck = deck.unwrap();
        println!(
            "{} {} {} {} {}",
            tmr.elapsed().as_millis(),
            deck.commanders.len(),
            deck.main.len(),
            deck.tokens.len(),
            deck.side.len()
        );
    }
    app.add_systems(Update, test);
    app.update();
}
#[derive(Component)]
struct Pile(Vec<Card>);
#[derive(Component)]
struct GetDeck(JoinHandle<Option<Deck>>);
#[derive(Debug)]
struct Deck {
    commanders: Vec<Card>,
    main: Vec<Card>,
    tokens: Vec<Card>,
    side: Vec<Card>,
}
#[derive(Debug, Default)]
#[allow(dead_code)]
struct CardInfo {
    name: String,
    mana_cost: Cost,
    card_type: String, //todo parsable
    text: String,
    color: Color,
    power: u16,
    toughness: u16,
    image: Handle<Image>,
}
#[derive(Debug, Default)]
struct Color {
    red: bool,
    blue: bool,
    green: bool,
    black: bool,
    white: bool,
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
#[derive(Debug, Default)]
struct Cost {
    red: u8,
    blue: u8,
    green: u8,
    black: u8,
    white: u8,
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
                    "P" => cost.pay += 1,
                    "X" => cost.var += 1,
                    _ => cost.any += c.parse::<u8>().unwrap(),
                }
            }
        }
        cost
    }
}
#[derive(Debug, Default)]
struct Card {
    normal: CardInfo,
    alt: Option<CardInfo>,
    is_alt: bool,
}
#[derive(Component)]
#[allow(dead_code)]
struct SyncObject(u64);
impl SyncObject {
    pub fn new(rand: &mut GlobalEntropy<WyRand>) -> Self {
        Self(rand.next_u64())
    }
}
#[derive(Component)]
struct FollowMouse;
#[derive(Component)]
struct ZoomHold((u64, bool));
#[derive(Component)]
struct Reversed;
struct Clipboard(LazyLock<arboard::Clipboard>);
impl Resource for Clipboard {}
struct Client(reqwest::Client);
impl Resource for Client {}
struct Runtime(tokio::runtime::Runtime);
impl Resource for Runtime {}
struct CardStock(Handle<Mesh>);
impl Resource for CardStock {}
struct CardBack(Handle<StandardMaterial>);
impl Resource for CardBack {}
struct CardSide(Handle<StandardMaterial>);
impl Resource for CardSide {}
