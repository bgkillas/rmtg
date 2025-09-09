use crate::download::{get_deck, get_from_img};
use crate::misc::new_pile;
use crate::*;
use bevy::prelude::*;
use bevy_framepace::{FramepaceSettings, Limiter};
use bevy_prng::WyRand;
use bevy_rand::global::GlobalEntropy;
use bevy_rapier3d::prelude::*;
use bytes::Bytes;
use std::fs;
pub fn setup(
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
        base_color: bevy::prelude::Color::srgb_u8(0x11, 0x0F, 0x02),
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
    commands.spawn((
        Transform::from_xyz(0.0, 64.0, START_Z / 3.0),
        Hand::default(),
        Collider::cuboid(128.0, 128.0, 16.0),
        Owned,
    ));
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
            angular_damping: 0.0,
        },
        AdditionalMassProperties::Mass(4.0),
        SyncObject::new(&mut rand),
        Mesh3d(meshes.add(RegularPolygon::new(32.0, 4))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::WHITE,
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
