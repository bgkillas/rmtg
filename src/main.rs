use bevy::asset::RenderAssetUsages;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::PrimaryWindow;
use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};
use bevy_prng::WyRand;
use bevy_rand::global::GlobalEntropy;
use bevy_rand::prelude::EntropyPlugin;
use bevy_rapier3d::prelude::*;
use bytes::Bytes;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use image::{GenericImageView, ImageReader};
use json::JsonValue;
use rand::RngCore;
use rand::seq::SliceRandom;
use std::f32::consts::PI;
use std::io::Cursor;
use std::sync::LazyLock;
use std::{fs, mem};
use tokio::task::JoinHandle;
static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
static CARD_WIDTH: f32 = 488.0;
static CARD_HEIGHT: f32 = 680.0;
static START_Y: f32 = 8192.0;
static START_Z: f32 = -4096.0;
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
fn follow_mouse(
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut card: Single<
        (
            Entity,
            &mut Transform,
            &mut GravityScale,
            &mut Velocity,
            &Collider,
        ),
        With<FollowMouse>,
    >,
    cards: Query<(&Pile, &Transform), Without<FollowMouse>>,
    mut commands: Commands,
    time_since: Res<Time>,
    rapier_context: ReadRapierContext,
) {
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let (camera, camera_transform) = camera.into_inner();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };
    if mouse_input.pressed(MouseButton::Left) {
        let Ok(context) = rapier_context.single() else {
            return;
        };
        if let Some(max) =
            context.with_query_pipeline(QueryFilter::only_dynamic(), |query_pipeline| {
                query_pipeline
                    .intersect_shape(card.1.translation, card.1.rotation, card.4.raw.0.as_ref())
                    .filter_map(|a| {
                        if a != card.0
                            && let Ok((pile, transform)) = cards.get(a)
                        {
                            Some(transform.translation.y + pile.0.len() as f32)
                        } else {
                            None
                        }
                    })
                    .reduce(f32::max)
            })
        {
            card.1.translation.y = max + 4.0;
        }
        if let Some(time) =
            ray.intersect_plane(card.1.translation, InfinitePlane3d { normal: Dir3::Y })
        {
            let point = ray.get_point(time);
            card.1.translation = point;
        }
    } else {
        if let Some(time) =
            ray.intersect_plane(card.1.translation, InfinitePlane3d { normal: Dir3::Y })
        {
            let point = ray.get_point(time);
            card.3.linvel = (point - card.1.translation) / time_since.delta_secs()
        }
        commands.entity(card.0).remove::<FollowMouse>();
        card.2.0 = GRAVITY
    }
}
fn listen_for_mouse(
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera: Single<(&Camera, &GlobalTransform, Entity)>,
    window: Single<&Window, With<PrimaryWindow>>,
    rapier_context: ReadRapierContext,
    mut cards: Query<(&mut Pile, &mut Transform, &Children)>,
    reversed: Query<&Reversed>,
    mut mats: Query<&mut MeshMaterial3d<StandardMaterial>, Without<ZoomHold>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    card_back: Res<CardBack>,
    card_side: Res<CardSide>,
    card_stock: Res<CardStock>,
    input: Res<ButtonInput<KeyCode>>,
    mut rand: GlobalEntropy<WyRand>,
    zoom: Option<Single<(Entity, &mut ZoomHold, &mut MeshMaterial3d<StandardMaterial>)>>,
) {
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    let (camera, camera_transform, cament) = camera.into_inner();
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };
    let Ok(context) = rapier_context.single() else {
        return;
    };
    let hit = context.cast_ray(
        ray.origin,
        ray.direction.into(),
        f32::MAX,
        true,
        QueryFilter::only_dynamic(),
    );
    if let Some((entity, _toi)) = hit
        && let Ok((mut pile, mut transform, children)) = cards.get_mut(entity)
    {
        if input.just_pressed(KeyCode::KeyF) {
            let is_reversed;
            if reversed.contains(entity) {
                is_reversed = false;
                commands.entity(entity).remove::<Reversed>();
            } else {
                is_reversed = true;
                commands.entity(entity).insert(Reversed);
            }
            pile.0.reverse();
            transform.rotate_z(PI);
            let pile = mem::take(&mut pile.0);
            new_pile_at(
                pile,
                card_stock.0.clone_weak(),
                &mut materials,
                &mut commands,
                &mut meshes,
                card_back.0.clone_weak(),
                card_side.0.clone_weak(),
                *transform,
                &mut rand,
                false,
                is_reversed,
            );
            commands.entity(entity).despawn();
        } else if input.just_pressed(KeyCode::KeyR) {
            pile.0.shuffle(&mut rand);
            let pile = mem::take(&mut pile.0);
            let reversed = reversed.contains(entity);
            new_pile_at(
                pile,
                card_stock.0.clone_weak(),
                &mut materials,
                &mut commands,
                &mut meshes,
                card_back.0.clone_weak(),
                card_side.0.clone_weak(),
                *transform,
                &mut rand,
                false,
                reversed,
            );
            commands.entity(entity).despawn();
        } else if mouse_input.just_pressed(MouseButton::Left) {
            let reversed = reversed.contains(entity);
            let len = pile.0.len() as f32;
            let new = pile.0.pop().unwrap();
            if !pile.0.is_empty() {
                let pile = mem::take(&mut pile.0);
                new_pile_at(
                    pile,
                    card_stock.0.clone_weak(),
                    &mut materials,
                    &mut commands,
                    &mut meshes,
                    card_back.0.clone_weak(),
                    card_side.0.clone_weak(),
                    *transform,
                    &mut rand,
                    false,
                    reversed,
                );
            }
            commands.entity(entity).despawn();
            transform.translation.y += len + 4.0;
            new_pile_at(
                vec![new],
                card_stock.0.clone_weak(),
                &mut materials,
                &mut commands,
                &mut meshes,
                card_back.0.clone_weak(),
                card_side.0.clone_weak(),
                *transform,
                &mut rand,
                true,
                reversed,
            );
        } else if input.just_pressed(KeyCode::KeyE) {
            let (_, _, rot) = transform.rotation.to_euler(EulerRot::XYZ);
            let n = (2.0 * rot / PI).round() as isize;
            transform.rotation = Quat::from_rotation_z(match n {
                0 => -PI / 2.0,
                1 => 0.0,
                2 | -2 => PI / 2.0,
                -1 => PI,
                _ => unreachable!(),
            });
            transform.rotate_x(-PI / 2.0);
        } else if input.just_pressed(KeyCode::KeyQ) {
            let (_, _, rot) = transform.rotation.to_euler(EulerRot::XYZ);
            let n = (2.0 * rot / PI).round() as isize;
            transform.rotation = Quat::from_rotation_z(match n {
                0 => PI / 2.0,
                1 => PI,
                2 | -2 => -PI / 2.0,
                -1 => 0.0,
                _ => unreachable!(),
            });
            transform.rotate_x(-PI / 2.0);
        } else if input.just_pressed(KeyCode::KeyO)
            && !reversed.contains(entity)
            && zoom
                .as_ref()
                .map(|single| single.1.0.0 != entity.to_bits())
                .unwrap_or(true)
        {
            let mut card = pile.0.pop().unwrap();
            if let Some(alt) = &mut card.alt {
                mem::swap(&mut card.normal, alt);
                mats.get_mut(*children.first().unwrap()).unwrap().0 =
                    make_material(&mut materials, card.normal.image.clone_weak());
                card.is_alt = !card.is_alt;
            }
            pile.0.push(card)
        } else if input.just_pressed(KeyCode::KeyZ) {
            //TODO search
        }
        if input.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) {
            if let Some(mut single) = zoom {
                if single.1.0.0 != entity.to_bits() {
                    commands.entity(single.0).despawn();
                } else if input.just_pressed(KeyCode::KeyO)
                    && let Some(alt) = &pile.0.last().unwrap().alt
                {
                    single.2.0 = make_material(
                        &mut materials,
                        if single.1.0.1 {
                            &pile.0.last().unwrap().normal
                        } else {
                            alt
                        }
                        .image
                        .clone_weak(),
                    );
                    single.1.0.1 = !single.1.0.1;
                }
            } else if !reversed.contains(entity) {
                let card = pile.0.last().unwrap();
                commands.entity(cament).with_child((
                    Mesh3d(card_stock.0.clone_weak()),
                    MeshMaterial3d(make_material(
                        &mut materials,
                        card.normal.image.clone_weak(),
                    )),
                    Transform::from_xyz(0.0, 0.0, -1024.0),
                    ZoomHold((entity.to_bits(), false)),
                ));
            }
        } else if let Some(single) = zoom {
            commands.entity(single.0).despawn();
        }
    } else if let Some(single) = zoom {
        commands.entity(single.0).despawn();
    }
}
fn cam_translation(
    input: Res<ButtonInput<KeyCode>>,
    mouse_motion: Res<AccumulatedMouseScroll>,
    mut cam: Single<&mut Transform, With<Camera3d>>,
) {
    let scale = if input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        128.0
    } else {
        32.0
    };
    let apply = |translate: Vec3, cam: &mut Transform| {
        let mut norm = translate.normalize();
        norm.y = 0.0;
        let abs = norm.length();
        if abs != 0.0 {
            let translate = norm * translate.length() / abs;
            cam.translation += translate;
        }
    };
    if input.pressed(KeyCode::KeyW) {
        let translate = cam.forward().as_vec3() * scale;
        apply(translate, &mut cam)
    }
    if input.pressed(KeyCode::KeyA) {
        let translate = cam.left().as_vec3() * scale;
        apply(translate, &mut cam)
    }
    if input.pressed(KeyCode::KeyD) {
        let translate = cam.right().as_vec3() * scale;
        apply(translate, &mut cam)
    }
    if input.pressed(KeyCode::KeyS) {
        let translate = cam.back().as_vec3() * scale;
        apply(translate, &mut cam)
    }
    if mouse_motion.delta.y != 0.0 {
        let translate = cam.forward().as_vec3() * scale * mouse_motion.delta.y * 16.0;
        if cam.translation.y < -translate.y {
            cam.translation.y /= 2.0;
        } else {
            cam.translation += translate;
        }
    }
    if input.pressed(KeyCode::Space) {
        *cam.into_inner() =
            Transform::from_xyz(0.0, START_Y, START_Z).looking_at(Vec3::ZERO, Vec3::Y);
    }
}
fn cam_rotation(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut cam: Single<&mut Transform, With<Camera3d>>,
) {
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
    mut rand: GlobalEntropy<WyRand>,
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
    let card = Card {
        normal: CardInfo {
            image: get_from_img(
                Bytes::from(fs::read("/home/.r/rmtg/assets/png.png").unwrap()),
                &asset_server,
            )
            .unwrap(),
            ..Default::default()
        },
        ..Default::default()
    };
    let card_side = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(0x11, 0x0F, 0x02),
        unlit: true,
        ..Default::default()
    });
    new_pile(
        vec![card],
        card_stock.clone_weak(),
        &mut materials,
        &mut commands,
        &mut meshes,
        material_handle.clone_weak(),
        card_side.clone_weak(),
        &mut rand,
        0.0,
        0.0,
    );
    commands.insert_resource(CardSide(card_side));
    commands.insert_resource(CardBack(material_handle));
    commands.insert_resource(CardStock(card_stock));
    const T: f32 = 256.0;
    const W: f32 = 16384.0;
    commands.spawn((Transform::from_xyz(0.0, -T, 0.0), Collider::cuboid(W, T, W)));
    commands.spawn((
        Transform::from_xyz(W + T / 2.0, T / 2.0, 0.0),
        Collider::cuboid(T, T, W),
    ));
    commands.spawn((
        Transform::from_xyz(-(W + T / 2.0), T / 2.0, 0.0),
        Collider::cuboid(T, T, W),
    ));
    commands.spawn((
        Transform::from_xyz(0.0, T / 2.0, W + T / 2.0),
        Collider::cuboid(W, T, T),
    ));
    commands.spawn((
        Transform::from_xyz(0.0, T / 2.0, -(W + T / 2.0)),
        Collider::cuboid(W, T, T),
    ));
    commands.spawn((
        Camera3d::default(),
        Msaa::Sample8,
        Transform::from_xyz(0.0, START_Y, START_Z).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Collider::cuboid(32.0, 32.0, 32.0),
        Transform::from_xyz(0.0, 64.0, 0.0),
        RigidBody::Dynamic,
        GravityScale(GRAVITY),
        Ccd::enabled(),
        Velocity::zero(),
        Damping {
            linear_damping: DAMPING,
            angular_damping: DAMPING,
        },
        AdditionalMassProperties::Mass(4.0),
        SyncObject::new(&mut rand),
        Sleeping::disabled(),
        Mesh3d(meshes.add(RegularPolygon::new(32.0, 6))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..Default::default()
        })),
    ));
    let client = client.0.clone();
    let asset_server = asset_server.clone();
    let url = "https://api2.moxfield.com/v3/decks/all/o7Iy63M1wkWHOx4fM-ODMA".to_string();
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
    {
        let paste = paste.trim();
        if paste.starts_with("https://moxfield.com/decks/")
            || paste.starts_with("https://www.moxfield.com/decks/")
            || paste.len() == 22
        {
            let id = paste.rsplit_once('/').map(|(_, b)| b).unwrap_or(paste);
            let url = format!("https://api2.moxfield.com/v3/decks/all/{id}");
            let client = client.0.clone();
            let asset_server = asset_server.clone();
            let task = runtime
                .0
                .spawn(async move { get_deck(url, client, asset_server).await });
            commands.spawn(GetDeck(task));
        }
    }
}
fn get_from_img(bytes: Bytes, asset_server: &AssetServer) -> Option<Handle<Image>> {
    fn to_asset(bytes: Bytes, asset_server: &AssetServer) -> Option<Handle<Image>> {
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
        Some(asset_server.add(image))
    }
    to_asset(bytes, asset_server)
}
async fn parse(
    value: &JsonValue,
    client: reqwest::Client,
    asset_server: AssetServer,
) -> Option<Card> {
    async fn get_bytes(id: &str, client: &reqwest::Client, normal: bool) -> Option<Bytes> {
        let url = if normal {
            format!("https://assets.moxfield.net/cards/card-{id}-normal.webp")
        } else {
            format!("https://assets.moxfield.net/cards/card-face-{id}-normal.webp")
        };
        let res = client.get(url).send().await.ok()?;
        res.bytes().await.ok()
    }
    let normal = value["card_faces"].members().next().is_none();
    let id = value["card_faces"]
        .members()
        .next()
        .and_then(|a| a["id"].as_str())
        .unwrap_or_else(|| value["id"].as_str().unwrap());
    let bytes = get_bytes(id, &client, normal).await?;
    let alt_bytes = if let Some(id) = value["meld_result"]["id"].as_str().or(value["card_faces"]
        .members()
        .nth(1)
        .and_then(|a| a["id"].as_str()))
    {
        get_bytes(id, &client, normal).await
    } else {
        None
    };
    let alt_name = value["meld_result"]["name"]
        .as_str()
        .or(value["card_faces"]
            .members()
            .nth(1)
            .and_then(|a| a["name"].as_str()))
        .map(|a| a.to_string());
    let name = value["card_faces"]
        .members()
        .next()
        .and_then(|a| a["name"].as_str())
        .map(|a| a.to_string())
        .unwrap_or_else(|| value["name"].to_string());
    let image = get_from_img(bytes, &asset_server)?;
    let alt_image = alt_bytes.and_then(|bytes| get_from_img(bytes, &asset_server));
    fn get<T: Default, F>(value: &JsonValue, index: &str, f: F) -> (T, T)
    where
        F: Fn(&JsonValue) -> T,
    {
        (
            value["card_faces"]
                .members()
                .next()
                .map(|a| f(&a[index]))
                .or_else(|| Some(f(&value[index])))
                .unwrap_or_default(),
            value["card_faces"]
                .members()
                .nth(1)
                .map(|a| f(&a[index]))
                .unwrap_or_default(),
        )
    }
    let (mana_cost, alt_mana_cost) = get(value, "mana_cost", |a| a.as_str().unwrap().to_string());
    let (card_type, alt_card_type) = get(value, "type_line", |a| a.as_str().unwrap().to_string());
    let (text, alt_text) = get(value, "oracle_text", |a| a.as_str().unwrap().to_string());
    let (colors, alt_colors) = get(value, "colors", |a| {
        a.members()
            .map(|a| a.as_str().unwrap().to_string())
            .collect::<Vec<String>>()
            .join(":")
    });
    let (power, alt_power) = get(value, "power", |a| a.as_u16().unwrap_or_default());
    let (toughness, alt_toughness) = get(value, "toughness", |a| a.as_u16().unwrap_or_default());
    Some(Card {
        normal: CardInfo {
            name,
            mana_cost,
            card_type,
            text,
            colors,
            power,
            toughness,
            image,
        },
        alt: alt_image.map(|image| CardInfo {
            name: alt_name.unwrap(),
            mana_cost: alt_mana_cost,
            card_type: alt_card_type,
            text: alt_text,
            colors: alt_colors,
            power: alt_power,
            toughness: alt_toughness,
            image,
        }),
        is_alt: false,
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
    query: Single<(Entity, &mut GetDeck)>,
    runtime: Res<Runtime>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    card_back: Res<CardBack>,
    card_side: Res<CardSide>,
    card_stock: Res<CardStock>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut rand: GlobalEntropy<WyRand>,
) {
    let (entity, mut deck) = query.into_inner();
    if deck.0.is_finished() {
        let handle = mem::replace(&mut deck.0, runtime.0.spawn(async { None }));
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
                &mut rand,
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
                &mut rand,
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
                &mut rand,
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
                &mut rand,
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
    rand: &mut GlobalEntropy<WyRand>,
    x: f32,
    z: f32,
) {
    let size = pile.len() as f32;
    let mut transform = Transform::from_xyz(x, size, z);
    transform.rotate_x(-PI / 2.0);
    transform.rotate_y(PI);
    new_pile_at(
        pile, card_stock, materials, commands, meshes, card_back, card_side, transform, rand,
        false, false,
    );
}
fn new_pile_at(
    pile: Vec<Card>,
    card_stock: Handle<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    card_back: Handle<StandardMaterial>,
    card_side: Handle<StandardMaterial>,
    transform: Transform,
    rand: &mut GlobalEntropy<WyRand>,
    follow_mouse: bool,
    reverse: bool,
) {
    if pile.is_empty() {
        return;
    }
    let card = pile.last().unwrap();
    let top = card.normal.image.clone_weak();
    let material_handle = make_material(materials, top);
    let size = pile.len() as f32;
    let mut transform1 = Transform::from_rotation(Quat::from_rotation_y(PI));
    transform1.translation.z = -size;
    let mesh = meshes.add(Rectangle::new(2.0 * size, CARD_HEIGHT));
    let mut transform2 = Transform::from_rotation(Quat::from_rotation_y(PI / 2.0));
    transform2.translation.x = CARD_WIDTH / 2.0;
    let mut transform3 = Transform::from_rotation(Quat::from_rotation_y(-PI / 2.0));
    transform3.translation.x = -CARD_WIDTH / 2.0;
    let mesh2 = meshes.add(Rectangle::new(CARD_WIDTH, 2.0 * size));
    let mut transform4 = Transform::from_rotation(Quat::from_rotation_x(PI / 2.0));
    transform4.translation.y = -CARD_HEIGHT / 2.0;
    let mut transform5 = Transform::from_rotation(Quat::from_rotation_x(-PI / 2.0));
    transform5.translation.y = CARD_HEIGHT / 2.0;
    let mut ent = commands.spawn((
        Pile(pile),
        transform,
        Visibility::default(),
        RigidBody::Dynamic,
        Collider::cuboid(CARD_WIDTH / 2.0, CARD_HEIGHT / 2.0, size),
        GravityScale(if follow_mouse { 0.0 } else { GRAVITY }),
        Ccd::enabled(),
        Velocity::zero(),
        Damping {
            linear_damping: DAMPING,
            angular_damping: DAMPING,
        },
        AdditionalMassProperties::Mass(size),
        SyncObject::new(rand),
        Sleeping::disabled(),
        children![
            (
                Mesh3d(card_stock.clone_weak()),
                MeshMaterial3d(material_handle),
                Transform::from_xyz(0.0, 0.0, size),
            ),
            (Mesh3d(card_stock), MeshMaterial3d(card_back), transform1),
            (
                Mesh3d(mesh.clone_weak()),
                MeshMaterial3d(card_side.clone_weak()),
                transform2,
            ),
            (
                Mesh3d(mesh),
                MeshMaterial3d(card_side.clone_weak()),
                transform3,
            ),
            (
                Mesh3d(mesh2.clone_weak()),
                MeshMaterial3d(card_side.clone_weak()),
                transform4,
            ),
            (Mesh3d(mesh2), MeshMaterial3d(card_side), transform5)
        ],
    ));
    if follow_mouse {
        ent.insert(FollowMouse);
    }
    if reverse {
        ent.insert(Reversed);
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
#[derive(Debug, Default)]
#[allow(dead_code)]
struct CardInfo {
    name: String,
    mana_cost: String,
    card_type: String,
    text: String,
    colors: String,
    power: u16,
    toughness: u16,
    image: Handle<Image>,
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
