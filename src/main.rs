use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use image::{GenericImageView, ImageReader};
use json::JsonValue;
use std::io::Cursor;
use std::sync::LazyLock;
use tokio::task::JoinHandle;

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
#[tokio::main(flavor = "multi_thread")]
async fn main() {
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
        //.add_systems(Startup, ...)
        .add_systems(Update, (pos, listen_for_deck, register_deck))
        .run();
}
fn pos(time: Res<Time>, mut query: Query<(&Pile, &mut Transform)>) {
    for (cards, mut pos) in &mut query {
        pos.translation.x += time.delta_secs();
        println!("{}", cards.0.len());
        if !cards.0.is_empty() {
            println!("{}", cards.0[0].image.size());
        }
    }
}
fn listen_for_deck(
    input: Res<ButtonInput<KeyCode>>,
    mut clipboard: ResMut<Clipboard>,
    client: Res<Client>,
    runtime: Res<Runtime>,
    mut commands: Commands,
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
        let task = runtime.0.spawn(async move { get_deck(url, client).await });
        commands.spawn(GetDeck(task));
    }
}
async fn parse(value: &mut JsonValue, client: reqwest::Client) -> Option<Card> {
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
        RenderAssetUsages::MAIN_WORLD,
    );
    Some(Card { image })
}
#[test]
fn test_parse() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let tmr = std::time::Instant::now();
    let img = runtime
        .block_on(runtime.spawn(async move {
            let mut json = json::object!(id: "OxoZm");
            parse(
                &mut json,
                reqwest::Client::builder()
                    .user_agent(USER_AGENT)
                    .build()
                    .unwrap(),
            )
            .await
        }))
        .unwrap();
    assert!(img.is_some());
    let img = img.unwrap();
    println!("{}", tmr.elapsed().as_millis(),);
    let mut wimg = image::RgbaImage::new(img.image.width(), img.image.height());
    use bevy::render::render_resource::encase::internal::BufferMut;
    wimg.write_slice(0, &img.image.data.unwrap());
    wimg.save("/home/png.png").unwrap();
}
async fn get_deck(url: String, client: reqwest::Client) -> Option<Deck> {
    if let Ok(res) = client.get(url).send().await
        && let Ok(text) = res.text().await
        && let Ok(mut json) = json::parse(&text)
    {
        macro_rules! get {
            ($b:tt, $s:tt) => {
                $b[$s]
                    .members_mut()
                    .map(|p| parse(p, client.clone()))
                    .collect::<FuturesUnordered<_>>()
                    .filter_map(async |a| a)
                    .collect::<Vec<Card>>()
                    .await
            };
        }
        let tokens = get!(json, "tokens");
        let board = &mut json["boards"];
        let main = get!(board, "mainboard");
        let side = get!(board, "sideboard");
        let commanders = get!(board, "commanders");
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
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let tmr = std::time::Instant::now();
    let deck = runtime
        .block_on(runtime.spawn(async move {
            get_deck(
                "https://api2.moxfield.com/v3/decks/all/_HGo1kgcB0i-4Iq0vR-LZA".to_string(),
                reqwest::Client::builder()
                    .user_agent(USER_AGENT)
                    .build()
                    .unwrap(),
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
fn register_deck(
    mut commands: Commands,
    mut query: Query<(Entity, &mut GetDeck)>,
    runtime: Res<Runtime>,
) {
    for (entity, mut deck) in query.iter_mut() {
        if deck.0.is_finished() {
            let handle = std::mem::replace(&mut deck.0, runtime.0.spawn(async { None }));
            commands.entity(entity).despawn();
            if let Some(result) = runtime.0.block_on(handle).ok().flatten() {
                commands.spawn((Pile(result.commanders), Transform::default()));
                commands.spawn((Pile(result.main), Transform::default()));
                commands.spawn((Pile(result.side), Transform::default()));
                commands.spawn((Pile(result.tokens), Transform::default()));
            }
        }
    }
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
    image: Image,
}
struct Clipboard(LazyLock<arboard::Clipboard>);
impl Resource for Clipboard {}
struct Client(reqwest::Client);
impl Resource for Client {}
struct Runtime(tokio::runtime::Runtime);
impl Resource for Runtime {}
