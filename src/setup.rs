use crate::counters::Value;
use crate::download::get_from_img;
use crate::misc::default_cam_pos;
#[cfg(feature = "steam")]
use crate::sync::COMPRESSION;
#[cfg(feature = "steam")]
use crate::sync::Packet;
#[cfg(feature = "steam")]
use crate::sync::SendSleeping;
use crate::sync::{SyncObjectMe, spawn_hand};
use crate::update::{CardSpot, GiveEnts, SpotType};
use crate::*;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy_framepace::{FramepaceSettings, Limiter};
use bevy_rand::global::GlobalRng;
#[cfg(feature = "steam")]
use bevy_tangled::{Client, ClientTrait, Reliability};
use bytes::Bytes;
#[cfg(feature = "steam")]
use std::collections::HashMap;
#[cfg(feature = "steam")]
use std::collections::hash_map::Entry::Vacant;
#[cfg(feature = "steam")]
use std::env::args;
use std::f32::consts::PI;
use std::fs;
pub const MAT_WIDTH: f32 = 8.0;
pub const MAT_HEIGHT: f32 = MAT_WIDTH * 9.0 / 16.0;
pub const MAT_BAR: f32 = MAT_HEIGHT / 64.0;
pub const T: f32 = W / 2.0;
pub const W: f32 = MAT_WIDTH * 2.0;
pub const WALL_COLOR: bevy::prelude::Color = bevy::prelude::Color::srgb_u8(103, 73, 40);
pub const FLOOR_COLOR: bevy::prelude::Color = bevy::prelude::Color::srgb_u8(103, 73, 40);
pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut framepace: ResMut<FramepaceSettings>,
    #[cfg(feature = "steam")] mut client: ResMut<Client>,
    mut rand: Single<&mut WyRand, With<GlobalRng>>,
    mut count: ResMut<SyncCount>,
    mut light: ResMut<AmbientLight>,
    #[cfg(feature = "steam")] send_sleep: Res<SendSleeping>,
    #[cfg(feature = "steam")] give: Res<GiveEnts>,
    #[cfg(feature = "steam")] peers: Res<Peers>,
    #[cfg(feature = "steam")] rempeers: Res<RemPeers>,
) {
    light.brightness = 100.0;
    let mut no_obj = false;
    #[cfg(feature = "steam")]
    {
        let who = Arc::new(Mutex::new(HashMap::new()));
        let who2 = who.clone();
        let send = send_sleep.0.clone();
        let give = give.0.clone();
        let rempeers = rempeers.0.clone();
        let peers1 = peers.map.clone();
        let peers2 = peers1.clone();
        let _ = client.init_steam(
            Some(Box::new(move |client, peer| {
                info!("user {peer} has joined");
                if client.is_host() {
                    let mut k = 1;
                    {
                        let mut who = who.lock().unwrap();
                        loop {
                            if let Vacant(e) = who.entry(k) {
                                e.insert(peer);
                                break;
                            }
                            k += 1;
                        }
                    }
                    peers1.lock().unwrap().insert(peer, k);
                    client
                        .broadcast(
                            &Packet::SetUser(peer, k),
                            Reliability::Reliable,
                            COMPRESSION,
                        )
                        .unwrap();
                    client
                        .send(
                            peer,
                            &Packet::SetUser(client.my_id(), 0),
                            Reliability::Reliable,
                            COMPRESSION,
                        )
                        .unwrap();
                }
                send.store(true, std::sync::atomic::Ordering::Relaxed);
            })),
            Some(Box::new(move |client, peer| {
                info!("user {peer} has left");
                peers2.lock().unwrap().remove(&peer);
                rempeers.lock().unwrap().push(peer);
                if client.is_host() {
                    give.lock().unwrap().push(peer);
                    let mut who = who2.lock().unwrap();
                    who.retain(|_, p| *p != peer)
                }
            })),
        );
        let mut next = false;
        let mut lobby = None;
        for arg in args().skip(1) {
            if arg == "+connect_lobby" {
                next = true;
            } else if next {
                lobby = Some(arg.parse::<u64>().unwrap());
            }
        }
        if let Some(lobby) = lobby {
            no_obj = true;
            client.join_steam(lobby);
        }
    }
    let font = include_bytes!("../assets/noto.ttf");
    let font = asset_server.add(Font::try_from_bytes(font.to_vec()).unwrap());
    commands.insert_resource(FontRes(font.clone()));
    let _ = fs::create_dir("./cache");
    framepace.limiter = Limiter::from_framerate(60.0);
    let card_stock = meshes.add(Rectangle::new(CARD_WIDTH, CARD_HEIGHT));
    let bytes = include_bytes!("../assets/back.jpg");
    let card_back = get_from_img(Bytes::from(bytes.as_slice()), &asset_server).unwrap();
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(card_back),
        alpha_mode: AlphaMode::Opaque,
        unlit: true,
        ..default()
    });
    let card_side = materials.add(StandardMaterial {
        base_color: bevy::prelude::Color::srgb_u8(0x11, 0x0F, 0x02),
        unlit: true,
        ..default()
    });
    commands.insert_resource(CardBase {
        stock: card_stock,
        back: material_handle,
        side: card_side,
    });
    let transform = Transform::from_xyz(MAT_WIDTH / 2.0, 0.0, MAT_HEIGHT / 2.0);
    make_mat(
        &mut materials,
        &mut meshes,
        &mut commands,
        transform,
        true,
        PLAYER0,
        Player(0),
    );
    let mut transform = Transform::from_xyz(MAT_WIDTH / 2.0, 0.0, -MAT_HEIGHT / 2.0);
    transform.rotate_y(PI);
    make_mat(
        &mut materials,
        &mut meshes,
        &mut commands,
        transform,
        false,
        PLAYER1,
        Player(1),
    );
    let transform = Transform::from_xyz(-MAT_WIDTH / 2.0, 0.0, MAT_HEIGHT / 2.0);
    make_mat(
        &mut materials,
        &mut meshes,
        &mut commands,
        transform,
        false,
        PLAYER2,
        Player(2),
    );
    let mut transform = Transform::from_xyz(-MAT_WIDTH / 2.0, 0.0, -MAT_HEIGHT / 2.0);
    transform.rotate_y(PI);
    make_mat(
        &mut materials,
        &mut meshes,
        &mut commands,
        transform,
        true,
        PLAYER3,
        Player(3),
    );
    spawn_hand(0, &mut commands);
    commands.spawn((
        Transform::from_xyz(0.0, -T / 2.0, 0.0),
        CollisionLayers::new(0b01, LayerMask::ALL),
        Collider::cuboid(2.0 * W + T, T, 2.0 * W + T),
        RigidBody::Static,
        Floor,
        Mesh3d(meshes.add(Cuboid::new(2.0 * W, T - CARD_THICKNESS, 2.0 * W))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: FLOOR_COLOR,
            unlit: true,
            ..default()
        })),
    ));
    commands.spawn((
        Transform::from_xyz(0.0, 2.0 * W + T / 2.0, 0.0),
        CollisionLayers::new(0b11, LayerMask::ALL),
        Collider::cuboid(2.0 * W + T, T, 2.0 * W + T),
        RigidBody::Static,
        Ceiling,
        Mesh3d(meshes.add(Cuboid::new(2.0 * W, T, 2.0 * W))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: WALL_COLOR,
            unlit: true,
            ..default()
        })),
    ));
    commands.spawn((
        Transform::from_xyz(W + T / 2.0, W, 0.0),
        CollisionLayers::new(0b11, LayerMask::ALL),
        Collider::cuboid(T, 2.0 * W + T, 2.0 * W + T),
        RigidBody::Static,
        Wall,
        Mesh3d(meshes.add(Cuboid::new(T, 2.0 * W, 2.0 * W))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: WALL_COLOR,
            unlit: true,
            ..default()
        })),
    ));
    commands.spawn((
        Transform::from_xyz(-(W + T / 2.0), W, 0.0),
        CollisionLayers::new(0b11, LayerMask::ALL),
        Collider::cuboid(T, 2.0 * W + T, 2.0 * W + T),
        RigidBody::Static,
        Wall,
        Mesh3d(meshes.add(Cuboid::new(T, 2.0 * W, 2.0 * W))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: WALL_COLOR,
            unlit: true,
            ..default()
        })),
    ));
    commands.spawn((
        Transform::from_xyz(0.0, W, W + T / 2.0),
        CollisionLayers::new(0b11, LayerMask::ALL),
        Collider::cuboid(2.0 * W + T, 2.0 * W + T, T),
        RigidBody::Static,
        Wall,
        Mesh3d(meshes.add(Cuboid::new(2.0 * W, 2.0 * W, T))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: WALL_COLOR,
            unlit: true,
            ..default()
        })),
    ));
    commands.spawn((
        Transform::from_xyz(0.0, W, -(W + T / 2.0)),
        CollisionLayers::new(0b11, LayerMask::ALL),
        Collider::cuboid(2.0 * W + T, 2.0 * W + T, T),
        RigidBody::Static,
        Wall,
        Mesh3d(meshes.add(Cuboid::new(2.0 * W, 2.0 * W, T))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: WALL_COLOR,
            unlit: true,
            ..default()
        })),
    ));
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: PI / 3.0,
            near: CARD_THICKNESS / 32.0,
            far: W * 2.0,
            ..default()
        }),
        Msaa::Sample8,
        default_cam_pos(0),
        Tonemapping::None,
    ));
    if !no_obj {
        let mut cube = Shape::Cube.create(
            Transform::from_xyz(MAT_WIDTH + 2.0 * CARD_WIDTH, MAT_BAR * 4.0, -CARD_WIDTH),
            &mut commands,
            &mut meshes,
            &mut materials,
            bevy::color::Color::WHITE,
        );
        cube.insert(SyncObjectMe::new(&mut rand, &mut count));
        let mut tetra = Shape::Tetrahedron.create(
            Transform::from_xyz(MAT_WIDTH + 2.0 * CARD_WIDTH, MAT_BAR * 4.0, CARD_WIDTH),
            &mut commands,
            &mut meshes,
            &mut materials,
            bevy::color::Color::WHITE,
        );
        tetra.insert(SyncObjectMe::new(&mut rand, &mut count));
        let mut ico = Shape::Icosahedron.create(
            Transform::from_xyz(MAT_WIDTH + 3.0 * CARD_WIDTH, MAT_BAR * 4.0, -CARD_WIDTH),
            &mut commands,
            &mut meshes,
            &mut materials,
            bevy::color::Color::WHITE,
        );
        ico.insert(SyncObjectMe::new(&mut rand, &mut count));
        let mut oct = Shape::Octohedron.create(
            Transform::from_xyz(MAT_WIDTH + 3.0 * CARD_WIDTH, MAT_BAR * 4.0, CARD_WIDTH),
            &mut commands,
            &mut meshes,
            &mut materials,
            bevy::color::Color::WHITE,
        );
        oct.insert(SyncObjectMe::new(&mut rand, &mut count));
        let mut dodec = Shape::Dodecahedron.create(
            Transform::from_xyz(MAT_WIDTH + 4.0 * CARD_WIDTH, MAT_BAR * 4.0, -CARD_WIDTH),
            &mut commands,
            &mut meshes,
            &mut materials,
            bevy::color::Color::WHITE,
        );
        dodec.insert(SyncObjectMe::new(&mut rand, &mut count));
        let mut coin = Shape::Disc.create(
            Transform::from_xyz(MAT_WIDTH + 4.0 * CARD_WIDTH, MAT_BAR * 4.0, CARD_WIDTH),
            &mut commands,
            &mut meshes,
            &mut materials,
            bevy::color::Color::WHITE,
        );
        coin.insert(SyncObjectMe::new(&mut rand, &mut count));
        let mut counter = Shape::Counter(Value(40)).create(
            Transform::from_xyz(MAT_BAR * 3.0, MAT_BAR * 4.0, MAT_BAR * 3.0),
            &mut commands,
            &mut meshes,
            &mut materials,
            bevy::color::Color::WHITE,
        );
        counter.insert(SyncObjectMe::new(&mut rand, &mut count));
        let mut counter = Shape::Counter(Value(40)).create(
            Transform::from_xyz(MAT_BAR * 3.0, MAT_BAR * 4.0, -MAT_BAR * 3.0)
                .looking_to(Dir3::Z, Dir3::Y),
            &mut commands,
            &mut meshes,
            &mut materials,
            bevy::color::Color::WHITE,
        );
        counter.insert(SyncObjectMe::new(&mut rand, &mut count));
        let mut counter = Shape::Counter(Value(40)).create(
            Transform::from_xyz(-MAT_BAR * 3.0, MAT_BAR * 4.0, MAT_BAR * 3.0),
            &mut commands,
            &mut meshes,
            &mut materials,
            bevy::color::Color::WHITE,
        );
        counter.insert(SyncObjectMe::new(&mut rand, &mut count));
        let mut counter = Shape::Counter(Value(40)).create(
            Transform::from_xyz(-MAT_BAR * 3.0, MAT_BAR * 4.0, -MAT_BAR * 3.0)
                .looking_to(Dir3::Z, Dir3::Y),
            &mut commands,
            &mut meshes,
            &mut materials,
            bevy::color::Color::WHITE,
        );
        counter.insert(SyncObjectMe::new(&mut rand, &mut count));
    }
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        EscMenu,
        Visibility::Hidden,
        BackgroundColor(bevy::color::Color::srgba_u8(0, 0, 0, 127)),
    ));
    #[cfg(feature = "steam")]
    commands.spawn((
        Node {
            width: Val::Px(0.0),
            height: Val::Px(0.0),
            ..default()
        },
        Text(String::new()),
        SteamInfo,
        EscMenu,
        Visibility::Hidden,
        TextFont {
            font,
            font_size: FONT_SIZE,
            ..default()
        },
    ));
}
#[derive(Resource)]
pub struct FontRes(pub Handle<Font>);
#[derive(Component)]
pub struct EscMenu;
#[derive(Component)]
pub struct SideMenu;
#[cfg(feature = "steam")]
#[derive(Component)]
pub struct SteamInfo;
#[derive(Component)]
pub struct Wall;
#[derive(Component)]
pub struct Floor;
#[derive(Component)]
pub struct Ceiling;
#[derive(Component, Copy, Clone, Debug)]
pub struct Player(pub usize);
pub fn make_mat(
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
    commands: &mut Commands,
    transform: Transform,
    right: bool,
    color: bevy::color::Color,
    player: Player,
) {
    let mat = materials.add(StandardMaterial {
        alpha_mode: AlphaMode::Opaque,
        unlit: true,
        base_color: color,
        ..default()
    });
    let trans = |x: f32, y: f32, z: f32| -> Transform {
        Transform::from_xyz(if right { x } else { -x }, y, z)
    };
    commands
        .spawn((transform, InheritedVisibility::default()))
        .with_children(|p| {
            p.spawn((
                Mesh3d(meshes.add(Rectangle::new(MAT_WIDTH, MAT_BAR))),
                MeshMaterial3d(mat.clone()),
                trans(0.0, 0.0, MAT_HEIGHT / 2.0 - MAT_BAR / 2.0)
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
            p.spawn((
                Mesh3d(meshes.add(Rectangle::new(MAT_WIDTH, MAT_BAR))),
                MeshMaterial3d(mat.clone()),
                trans(0.0, 0.0, MAT_BAR / 2.0 - MAT_HEIGHT / 2.0)
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
            p.spawn((
                Mesh3d(meshes.add(Rectangle::new(MAT_BAR, MAT_HEIGHT))),
                MeshMaterial3d(mat.clone()),
                trans(MAT_WIDTH / 2.0 - MAT_BAR / 2.0, 0.0, 0.0)
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
            p.spawn((
                Mesh3d(meshes.add(Rectangle::new(MAT_BAR, MAT_HEIGHT))),
                MeshMaterial3d(mat.clone()),
                trans(MAT_BAR / 2.0 - MAT_WIDTH / 2.0, 0.0, 0.0)
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
            for i in 1..5 {
                p.spawn((
                    Mesh3d(meshes.add(Rectangle::new(CARD_WIDTH, MAT_BAR))),
                    MeshMaterial3d(mat.clone()),
                    trans(
                        MAT_WIDTH / 2.0 - CARD_WIDTH / 2.0 - MAT_BAR,
                        0.0,
                        i as f32 * (CARD_HEIGHT + MAT_BAR) - MAT_HEIGHT / 2.0 + MAT_BAR / 2.0,
                    )
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
                ));
            }
            for i in 0..5 {
                p.spawn((
                    trans(
                        MAT_WIDTH / 2.0 - MAT_BAR - CARD_WIDTH / 2.0,
                        CARD_THICKNESS / 2.0,
                        MAT_HEIGHT / 2.0
                            - MAT_BAR
                            - CARD_HEIGHT / 2.0
                            - i as f32 * (CARD_HEIGHT + MAT_BAR),
                    ),
                    match i {
                        4 => CardSpot::new(SpotType::CommanderMain),
                        3 => CardSpot::new(SpotType::CommanderAlt),
                        2 => CardSpot::new(SpotType::Exile),
                        1 => CardSpot::new(SpotType::Main),
                        0 => CardSpot::new(SpotType::Graveyard),
                        _ => unreachable!(),
                    },
                    player,
                ));
            }
            p.spawn((
                Mesh3d(meshes.add(Rectangle::new(MAT_BAR, MAT_HEIGHT))),
                MeshMaterial3d(mat.clone()),
                trans(MAT_WIDTH / 2.0 - MAT_BAR * 1.5 - CARD_WIDTH, 0.0, 0.0)
                    .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
            p.spawn((
                Mesh3d(meshes.add(Rectangle::new(
                    MAT_WIDTH - CARD_WIDTH - 2.0 * MAT_BAR,
                    MAT_BAR,
                ))),
                MeshMaterial3d(mat.clone()),
                trans(
                    -CARD_WIDTH / 2.0 - MAT_BAR,
                    0.0,
                    MAT_HEIGHT / 2.0 - MAT_BAR * 1.5 - CARD_HEIGHT * 1.5,
                )
                .looking_to(Vec3::NEG_Y, Vec3::NEG_Z),
            ));
        });
}
