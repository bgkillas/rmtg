use bevy::prelude::*;
use json::JsonValue;
use std::sync::LazyLock;
use tokio::task::JoinHandle;
#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let runtime = Runtime(tokio::runtime::Runtime::new().unwrap());
    static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .unwrap();
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Clipboard(LazyLock::new(|| {
            arboard::Clipboard::new().unwrap()
        })))
        .insert_resource(runtime)
        .insert_resource(Client(client))
        .add_systems(Startup, add_people)
        .add_systems(Update, (pos, get_deck, register_deck))
        .run();
}
fn add_people(mut commands: Commands) {
    commands.spawn(Transform::default());
}
fn pos(time: Res<Time>, mut query: Query<&mut Transform>) {
    for mut pos in &mut query {
        pos.translation.x += time.delta_secs()
    }
}
fn get_deck(
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
        let entity = commands.spawn_empty().id();
        let client = client.0.clone();
        let task = runtime.0.spawn(async move {
            let ret = client.get(url).send().await;
            if let Ok(res) = ret
                && { true }
                && let Ok(text) = res.text().await
            {
                json::parse(&text).ok()
            } else {
                None
            }
        });
        commands.entity(entity).insert(GetDeck(task));
    }
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
            let result = runtime.0.block_on(handle).ok().flatten();
            println!("{:#?}", result);
        }
    }
}
#[derive(Component)]
struct GetDeck(JoinHandle<Option<JsonValue>>);
struct Clipboard(LazyLock<arboard::Clipboard>);
impl Resource for Clipboard {}
struct Client(reqwest::Client);
impl Resource for Client {}
struct Runtime(tokio::runtime::Runtime);
impl Resource for Runtime {}
