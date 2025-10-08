use crate::download::get_from_img;
use crate::shapes::{spawn_coin, spawn_cube, spawn_dodec, spawn_ico, spawn_oct, spawn_tetra};
#[cfg(feature = "steam")]
use crate::sync::Packet;
#[cfg(feature = "steam")]
use crate::sync::SendSleeping;
use crate::sync::{SyncObjectMe, spawn_hand};
use crate::*;
use bevy_framepace::{FramepaceSettings, Limiter};
use bevy_rand::global::GlobalRng;
use bytes::Bytes;
#[cfg(feature = "steam")]
use net::{Client, ClientTrait, Reliability};
#[cfg(feature = "steam")]
use std::collections::HashMap;
#[cfg(feature = "steam")]
use std::collections::hash_map::Entry::Vacant;
#[cfg(feature = "steam")]
use std::env::args;
use std::f32::consts::PI;
use std::fs;
const MAT_SCALE: f32 = 10.0;
pub const MAT_WIDTH: f32 = 872.0 * MAT_SCALE;
pub const MAT_HEIGHT: f32 = 525.0 * MAT_SCALE;
pub const T: f32 = 256.0;
pub const W: f32 = 16384.0;
pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut framepace: ResMut<FramepaceSettings>,
    #[cfg(feature = "steam")] mut client: ResMut<Client>,
    mut rand: Single<&mut WyRand, With<GlobalRng>>,
    mut count: ResMut<SyncCount>,
    #[cfg(feature = "steam")] send_sleep: Res<SendSleeping>,
) {
    #[cfg(feature = "steam")]
    {
        let who = Arc::new(Mutex::new(HashMap::new()));
        let who2 = who.clone();
        let send = send_sleep.0.clone();
        let _ = client.init_steam(
            Some(Box::new(move |client, peer| {
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
                    client
                        .send_message(peer, &Packet::SetUser(k), Reliability::Reliable)
                        .unwrap();
                }
                send.store(true, std::sync::atomic::Ordering::Relaxed);
            })),
            Some(Box::new(move |client, peer| {
                if client.is_host() {
                    let mut who = who2.lock().unwrap();
                    who.retain(|_, p| *p != peer)
                }
            })),
        );
        let mut next = false;
        let mut lobby = None;
        let mut f = |arg: &str| {
            if arg == "+connect_lobby" {
                next = true;
            } else if next {
                lobby = Some(arg.parse::<u64>().unwrap());
            }
        };
        for arg in args().skip(1) {
            f(&arg)
        }
        for arg in client.args().split(' ') {
            f(arg)
        }
        if let Some(lobby) = lobby {
            client.join_steam(lobby).unwrap();
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
    let bytes = include_bytes!("../assets/mat.png");
    let mat = get_from_img(Bytes::from(bytes.as_slice()), &asset_server).unwrap();
    let playmat = materials.add(StandardMaterial {
        base_color_texture: Some(mat),
        alpha_mode: AlphaMode::Opaque,
        unlit: true,
        ..default()
    });
    let mat_mesh = meshes.add(Rectangle::new(MAT_WIDTH, MAT_HEIGHT));
    let mut transform = Transform::from_xyz(MAT_WIDTH / 2.0, 0.0, MAT_HEIGHT / 2.0);
    transform.rotate_x(-PI / 2.0);
    commands.spawn((
        Mesh3d(mat_mesh.clone()),
        MeshMaterial3d(playmat.clone()),
        transform,
    ));
    let mut transform = Transform::from_xyz(-MAT_WIDTH / 2.0, 0.0, MAT_HEIGHT / 2.0);
    transform.rotate_x(-PI / 2.0);
    commands.spawn((
        Mesh3d(mat_mesh.clone()),
        MeshMaterial3d(playmat.clone()),
        transform,
    ));
    let mut transform = Transform::from_xyz(MAT_WIDTH / 2.0, 0.0, -MAT_HEIGHT / 2.0);
    transform.rotate_x(-PI / 2.0);
    transform.rotate_y(PI);
    commands.spawn((
        Mesh3d(mat_mesh.clone()),
        MeshMaterial3d(playmat.clone()),
        transform,
    ));
    let mut transform = Transform::from_xyz(-MAT_WIDTH / 2.0, 0.0, -MAT_HEIGHT / 2.0);
    transform.rotate_x(-PI / 2.0);
    transform.rotate_y(PI);
    commands.spawn((Mesh3d(mat_mesh), MeshMaterial3d(playmat), transform));
    spawn_hand(0, &mut commands);
    commands.spawn((
        Transform::from_xyz(0.0, -T, 0.0),
        Collider::cuboid(2.0 * W, 2.0 * T, 2.0 * W),
        RigidBody::Static,
        Floor,
        Mesh3d(meshes.add(Cuboid::new(2.0 * W, 2.0 * T - 2.0, 2.0 * W))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::BLACK,
            ..default()
        })),
    ));
    commands.spawn((
        Transform::from_xyz(0.0, 2.0 * (W - T), 0.0),
        Collider::cuboid(2.0 * W, 2.0 * T, 2.0 * W),
        RigidBody::Static,
        Ceiling,
        Mesh3d(meshes.add(Cuboid::new(2.0 * W, 2.0 * T, 2.0 * W))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::BLACK,
            ..default()
        })),
    ));
    commands.spawn((
        Transform::from_xyz(W + T / 2.0, W - T, 0.0),
        Collider::cuboid(2.0 * T, 2.0 * W, 2.0 * W),
        RigidBody::Static,
        Wall,
        Mesh3d(meshes.add(Cuboid::new(2.0 * T, 2.0 * W, 2.0 * W))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::BLACK,
            ..default()
        })),
    ));
    commands.spawn((
        Transform::from_xyz(-(W + T / 2.0), W - T, 0.0),
        Collider::cuboid(2.0 * T, 2.0 * W, 2.0 * W),
        RigidBody::Static,
        Wall,
        Mesh3d(meshes.add(Cuboid::new(2.0 * T, 2.0 * W, 2.0 * W))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::BLACK,
            ..default()
        })),
    ));
    commands.spawn((
        Transform::from_xyz(0.0, W - T, W + T / 2.0),
        Collider::cuboid(2.0 * W, 2.0 * W, 2.0 * T),
        RigidBody::Static,
        Wall,
        Mesh3d(meshes.add(Cuboid::new(2.0 * W, 2.0 * W, 2.0 * T))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::BLACK,
            ..default()
        })),
    ));
    commands.spawn((
        Transform::from_xyz(0.0, W - T, -(W + T / 2.0)),
        Collider::cuboid(2.0 * W, 2.0 * W, 2.0 * T),
        RigidBody::Static,
        Wall,
        Mesh3d(meshes.add(Cuboid::new(2.0 * W, 2.0 * W, 2.0 * T))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::BLACK,
            ..default()
        })),
    ));
    commands.spawn((
        Camera3d::default(),
        Msaa::Sample8,
        Transform::from_xyz(0.0, START_Y, START_Z).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    let mut cube = spawn_cube(
        256.0,
        Transform::from_xyz(0.0, 128.0, 0.0),
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    cube.insert(SyncObjectMe::new(&mut rand, &mut count));
    let mut tetra = spawn_tetra(
        128.0,
        Transform::from_xyz(-256.0, 192.0, 0.0),
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    tetra.insert(SyncObjectMe::new(&mut rand, &mut count));
    let mut ico = spawn_ico(
        96.0,
        Transform::from_xyz(320.0, 128.0, 0.0),
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    ico.insert(SyncObjectMe::new(&mut rand, &mut count));
    let mut oct = spawn_oct(
        192.0,
        Transform::from_xyz(672.0, 128.0, 0.0),
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    oct.insert(SyncObjectMe::new(&mut rand, &mut count));
    let mut dodec = spawn_dodec(
        96.0,
        Transform::from_xyz(1056.0, 128.0, 0.0),
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    dodec.insert(SyncObjectMe::new(&mut rand, &mut count));
    let mut coin = spawn_coin(
        96.0,
        Transform::from_xyz(0.0, 128.0, 256.0),
        &mut commands,
        &mut meshes,
        &mut materials,
    );
    coin.insert(SyncObjectMe::new(&mut rand, &mut count));
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
#[derive(Component)]
pub struct EscMenu;
#[derive(Component)]
pub struct SideMenu;
#[cfg(feature = "steam")]
#[derive(Component)]
pub struct SteamInfo;
#[derive(Resource)]
#[allow(dead_code)]
pub struct FontRes(Handle<Font>);
#[derive(Component)]
pub struct Wall;
#[derive(Component)]
pub struct Floor;
#[derive(Component)]
pub struct Ceiling;
