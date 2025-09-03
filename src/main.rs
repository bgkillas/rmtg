use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use image::{GenericImageView, ImageReader};
use json::JsonValue;
use std::f32::consts::PI;
use std::io::Cursor;
use std::sync::LazyLock;
use tokio::task::JoinHandle;

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
fn main() {
    let runtime = Runtime(tokio::runtime::Runtime::new().unwrap());
    let client = Client(
        reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .unwrap(),
    );
    let clipboard = Clipboard(LazyLock::new(|| arboard::Clipboard::new().unwrap()));
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(clipboard)
        .insert_resource(runtime)
        .insert_resource(client)
        .add_systems(Startup, setup)
        .add_systems(Update, (listen_for_deck, register_deck))
        .run();
}
fn setup(
    mut commands: Commands,
    client: Res<Client>,
    asset_server: Res<AssetServer>,
    runtime: Res<Runtime>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let height = 680.0;
    let width = 480.0;
    let quad_handle = meshes.add(Rectangle::new(width, height));
    commands.insert_resource(CardStock(quad_handle));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-1.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    let client = client.0.clone();
    let asset_server = asset_server.clone();
    let url = "https://api2.moxfield.com/v3/decks/all/_HGo1kgcB0i-4Iq0vR-LZA".to_string();
    let task = runtime
        .0
        .spawn(async move { get_deck(url, client, asset_server).await });
    commands.spawn(GetDeck(task));
}
fn listen_for_deck(
    input: Res<ButtonInput<KeyCode>>,
    mut clipboard: ResMut<Clipboard>,
    client: Res<Client>,
    runtime: Res<Runtime>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        && input.just_pressed(KeyCode::KeyV)
        && let Ok(paste) = clipboard.0.get_text()
        && (paste.starts_with("https://moxfield.com/decks/")
            || paste.starts_with("https://www.moxfield.com/decks/")
            || paste.len() == 22)
    {
        let id = paste.rsplit_once('/').map(|(_, b)| b).unwrap_or(&paste);
        let url = format!("https://api2.moxfield.com/v3/decks/all/{id}");
        let client = client.0.clone();
        let asset_server = asset_server.clone();
        let task = runtime
            .0
            .spawn(async move { get_deck(url, client, asset_server).await });
        commands.spawn(GetDeck(task));
    }
}
async fn parse(
    value: &JsonValue,
    client: reqwest::Client,
    asset_server: AssetServer,
) -> Option<Card> {
    let id = &value["id"];
    let url = format!("https://assets.moxfield.net/cards/card-{id}-normal.webp");
    let res = client.get(url).send().await.ok()?;
    let bytes = res.bytes().await.ok()?;
    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    let rgba = image.to_rgba8();
    let (width, height) = image.dimensions();
    let image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    Some(Card {
        image: asset_server.add(image),
    })
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
                let mut json = json::object!(id: "OxoZm");
                parse(
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
async fn get_deck(url: String, client: reqwest::Client, asset_server: AssetServer) -> Option<Deck> {
    if let Ok(res) = client.get(url).send().await
        && let Ok(text) = res.text().await
        && let Ok(json) = json::parse(&text)
    {
        macro_rules! get {
            ($b:expr) => {
                $b.map(|p| parse(p, client.clone(), asset_server.clone()))
                    .collect::<FuturesUnordered<_>>()
                    .filter_map(async |a| a)
                    .collect::<Vec<Card>>()
                    .await
            };
        }
        let tokens = get!(json["tokens"].members());
        let board = &json["boards"];
        let main = get!(
            board["mainboard"]["cards"]
                .entries()
                .map(|(_, c)| &c["card"])
        );
        let side = get!(
            board["sideboard"]["cards"]
                .entries()
                .map(|(_, c)| &c["card"])
        );
        let commanders = get!(
            board["commanders"]["cards"]
                .entries()
                .map(|(_, c)| &c["card"])
        );
        Some(Deck {
            commanders,
            main,
            tokens,
            side,
        })
    } else {
        None
    }
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
                get_deck(
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
fn register_deck(
    mut commands: Commands,
    mut query: Query<(Entity, &mut GetDeck)>,
    runtime: Res<Runtime>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    card_stock: Res<CardStock>,
) {
    for (entity, mut deck) in query.iter_mut() {
        if deck.0.is_finished() {
            let handle = std::mem::replace(&mut deck.0, runtime.0.spawn(async { None }));
            commands.entity(entity).despawn();
            if let Some(result) = runtime.0.block_on(handle).ok().flatten() {
                commands.spawn(new_pile(
                    result.commanders,
                    card_stock.0.clone_weak(),
                    &mut materials,
                    -1000.0,
                ));
                commands.spawn(new_pile(
                    result.main,
                    card_stock.0.clone_weak(),
                    &mut materials,
                    -500.0,
                ));
                commands.spawn(new_pile(
                    result.side,
                    card_stock.0.clone_weak(),
                    &mut materials,
                    500.0,
                ));
                commands.spawn(new_pile(
                    result.tokens,
                    card_stock.0.clone_weak(),
                    &mut materials,
                    1000.0,
                ));
            }
        }
    }
}
fn new_pile(
    pile: Vec<Card>,
    card_stock: Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    z: f32,
) -> (Pile, Mesh3d, MeshMaterial3d<StandardMaterial>, Transform) {
    let top = pile[0].image.clone_weak();
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(top),
        alpha_mode: AlphaMode::Opaque,
        unlit: true,
        ..default()
    });
    let mut transform = Transform::from_xyz(2048.0, 0.0, z);
    transform.rotate_y(-PI / 2.0);
    (
        Pile(pile),
        Mesh3d(card_stock),
        MeshMaterial3d(material_handle),
        transform,
    )
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
#[derive(Debug)]
struct Card {
    // name: String,
    image: Handle<Image>,
}
struct Clipboard(LazyLock<arboard::Clipboard>);
impl Resource for Clipboard {}
struct Client(reqwest::Client);
impl Resource for Client {}
struct Runtime(tokio::runtime::Runtime);
impl Resource for Runtime {}
struct CardStock(Handle<Mesh>);
impl Resource for CardStock {}
