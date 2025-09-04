use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use bevy_rapier3d::prelude::*;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use image::{GenericImageView, ImageReader};
use json::JsonValue;
use std::f32::consts::PI;
use std::fs;
use std::io::Cursor;
use std::sync::LazyLock;
use tokio::task::JoinHandle;
static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
static CARD_WIDTH: f32 = 488.0;
static CARD_HEIGHT: f32 = 680.0;
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
                listen_for_mouse,
            ),
        )
        .run();
}
fn listen_for_mouse(
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window, With<PrimaryWindow>>,
    rapier_context: ReadRapierContext,
    mut cards: Query<(&mut Pile, &Transform, &Children)>,
    mut card_mats: Query<&mut MeshMaterial3d<StandardMaterial>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    card_back: Res<CardBack>,
    card_side: Res<CardSide>,
    card_stock: Res<CardStock>,
    input: Res<ButtonInput<KeyCode>>,
) {
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let (camera, camera_transform) = camera.into_inner();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };
    let Ok(context) = rapier_context.single() else {
        return;
    };
    context.with_query_pipeline(QueryFilter::only_dynamic(), |query_pipeline| {
        let hit = query_pipeline.cast_ray(ray.origin, ray.direction.into(), f32::MAX, true);
        if let Some((entity, _toi)) = hit
            && let Ok((pile, transform, children)) = cards.get_mut(entity)
        {
            if mouse_input.just_pressed(MouseButton::Left) {
                let pile = pile.into_inner();
                let new = pile.0.pop().unwrap();
                if pile.0.is_empty() {
                    commands.entity(entity).despawn()
                } else {
                    let top = pile.0.last().unwrap();
                    let mat = make_material(&mut materials, top.image.clone_weak());
                    for child in children {
                        let mut old_mat = card_mats.get_mut(*child).unwrap();
                        if old_mat.0.id() != card_back.0.id() {
                            old_mat.0 = mat;
                            break;
                        }
                    }
                }
                new_pile(
                    vec![new],
                    card_stock.0.clone_weak(),
                    &mut materials,
                    &mut commands,
                    &mut meshes,
                    card_back.0.clone_weak(),
                    card_side.0.clone_weak(),
                    transform.translation.x,
                    transform.translation.z - CARD_HEIGHT,
                );
            } else if input.just_pressed(KeyCode::KeyE) {
            } else if input.just_pressed(KeyCode::KeyQ) {
            }
        }
    });
}
fn cam_translation(input: Res<ButtonInput<KeyCode>>, cam: Single<(&mut Transform, &Camera3d)>) {
    let cam = cam.into_inner().0.into_inner();
    if input.pressed(KeyCode::KeyW) {
        cam.translation += cam.forward().as_vec3() * 32.0;
    }
    if input.pressed(KeyCode::KeyA) {
        cam.translation += cam.left().as_vec3() * 32.0;
    }
    if input.pressed(KeyCode::KeyD) {
        cam.translation += cam.right().as_vec3() * 32.0;
    }
    if input.pressed(KeyCode::KeyS) {
        cam.translation += cam.back().as_vec3() * 32.0;
    }
    if input.pressed(KeyCode::Space) {
        *cam = Transform::from_xyz(0.0, 2048.0, -2048.0).looking_at(Vec3::ZERO, Vec3::Y);
    }
}
fn cam_rotation(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    cam: Single<(&mut Transform, &Camera3d)>,
) {
    let cam = cam.into_inner().0.into_inner();
    if mouse_button.pressed(MouseButton::Right) && mouse_motion.delta != Vec2::ZERO {
        let delta_yaw = -mouse_motion.delta.x * 0.001;
        let delta_pitch = -mouse_motion.delta.y * 0.001;
        let (yaw, pitch, roll) = cam.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;
        let pitch = pitch + delta_pitch;
        cam.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}
fn setup(
    mut commands: Commands,
    client: Res<Client>,
    asset_server: Res<AssetServer>,
    runtime: Res<Runtime>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut framepace: ResMut<FramepaceSettings>,
) {
    framepace.limiter = Limiter::from_framerate(60.0);
    let card_stock = meshes.add(Rectangle::new(CARD_WIDTH, CARD_HEIGHT));
    let card_back = asset_server.load("back2.png");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(card_back),
        alpha_mode: AlphaMode::Opaque,
        unlit: true,
        ..default()
    });
    let card = get_from_img(
        Cursor::new(&fs::read("/home/.r/rmtg/assets/png.png").unwrap()),
        asset_server.clone(),
    )
    .unwrap();
    let card_side = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(0x11, 0x0F, 0x02),
        unlit: true,
        ..Default::default()
    });
    new_pile(
        vec![
            Card {
                image: card.image.clone_weak(),
            },
            card,
        ],
        card_stock.clone_weak(),
        &mut materials,
        &mut commands,
        &mut meshes,
        material_handle.clone_weak(),
        card_side.clone_weak(),
        0.0,
        0.0,
    );
    commands.insert_resource(CardSide(card_side));
    commands.insert_resource(CardBack(material_handle));
    commands.insert_resource(CardStock(card_stock));
    commands.spawn((
        Transform::from_xyz(0.0, -1.0, 0.0),
        Collider::cuboid(4096.0, 1.0, 4096.0),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2048.0, -2048.0).looking_at(Vec3::ZERO, Vec3::Z),
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
fn get_from_img(bytes: Cursor<&[u8]>, asset_server: AssetServer) -> Option<Card> {
    let image = ImageReader::new(bytes)
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
    let image = asset_server.add(image);
    Some(Card { image })
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
    get_from_img(Cursor::new(bytes.as_ref()), asset_server)
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
    query: Single<(Entity, &mut GetDeck)>,
    runtime: Res<Runtime>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    card_back: Res<CardBack>,
    card_side: Res<CardSide>,
    card_stock: Res<CardStock>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let (entity, mut deck) = query.into_inner();
    if deck.0.is_finished() {
        let handle = std::mem::replace(&mut deck.0, runtime.0.spawn(async { None }));
        commands.entity(entity).despawn();
        if let Some(result) = runtime.0.block_on(handle).ok().flatten() {
            new_pile(
                result.commanders,
                card_stock.0.clone_weak(),
                &mut materials,
                &mut commands,
                &mut meshes,
                card_back.0.clone_weak(),
                card_side.0.clone_weak(),
                -1000.0,
                0.0,
            );
            new_pile(
                result.main,
                card_stock.0.clone_weak(),
                &mut materials,
                &mut commands,
                &mut meshes,
                card_back.0.clone_weak(),
                card_side.0.clone_weak(),
                -500.0,
                0.0,
            );
            new_pile(
                result.side,
                card_stock.0.clone_weak(),
                &mut materials,
                &mut commands,
                &mut meshes,
                card_back.0.clone_weak(),
                card_side.0.clone_weak(),
                500.0,
                0.0,
            );
            new_pile(
                result.tokens,
                card_stock.0.clone_weak(),
                &mut materials,
                &mut commands,
                &mut meshes,
                card_back.0.clone_weak(),
                card_side.0.clone_weak(),
                1000.0,
                0.0,
            );
        }
    }
}
fn make_material(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    top: Handle<Image>,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color_texture: Some(top),
        unlit: true,
        ..default()
    })
}
fn new_pile(
    pile: Vec<Card>,
    card_stock: Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    card_back: Handle<StandardMaterial>,
    card_side: Handle<StandardMaterial>,
    x: f32,
    z: f32,
) {
    if pile.is_empty() {
        return;
    }
    let top = pile.last().unwrap().image.clone_weak();
    let material_handle = make_material(materials, top);
    let size = pile.len() as f32;
    let mut transform = Transform::from_xyz(x, size, z);
    transform.rotate_x(-PI / 2.0);
    transform.rotate_y(PI);
    commands
        .spawn((
            Pile(pile),
            transform,
            Visibility::default(),
            RigidBody::Dynamic,
            Collider::cuboid(CARD_WIDTH / 2.0, CARD_HEIGHT / 2.0, size),
            GravityScale(16.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(card_stock.clone_weak()),
                MeshMaterial3d(material_handle),
                Transform::from_xyz(0.0, 0.0, size),
            ));
            let mut transform = Transform::from_rotation(Quat::from_rotation_y(PI));
            transform.translation.z = -size;
            parent.spawn((Mesh3d(card_stock), MeshMaterial3d(card_back), transform));

            let mesh = meshes.add(Rectangle::new(2.0 * size, CARD_HEIGHT));
            let mut transform = Transform::from_rotation(Quat::from_rotation_y(PI / 2.0));
            transform.translation.x = CARD_WIDTH / 2.0;
            parent.spawn((
                Mesh3d(mesh.clone_weak()),
                MeshMaterial3d(card_side.clone_weak()),
                transform,
            ));

            let mut transform = Transform::from_rotation(Quat::from_rotation_y(-PI / 2.0));
            transform.translation.x = -CARD_WIDTH / 2.0;
            parent.spawn((
                Mesh3d(mesh),
                MeshMaterial3d(card_side.clone_weak()),
                transform,
            ));

            let mesh = meshes.add(Rectangle::new(CARD_WIDTH, 2.0 * size));
            let mut transform = Transform::from_rotation(Quat::from_rotation_x(PI / 2.0));
            transform.translation.y = -CARD_HEIGHT / 2.0;
            parent.spawn((
                Mesh3d(mesh.clone_weak()),
                MeshMaterial3d(card_side.clone_weak()),
                transform,
            ));

            let mut transform = Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0));
            transform.translation.y = CARD_HEIGHT / 2.0;
            parent.spawn((Mesh3d(mesh), MeshMaterial3d(card_side), transform));
        });
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
struct CardBack(Handle<StandardMaterial>);
impl Resource for CardBack {}
struct CardSide(Handle<StandardMaterial>);
impl Resource for CardSide {}
