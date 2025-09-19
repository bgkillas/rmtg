use crate::sync::{PollGroup, SyncObjectMe, spawn_hand};
use crate::*;
use bevy_framepace::{FramepaceSettings, Limiter};
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAnchor, TextAtlas};
use bevy_steamworks::Client;
use std::f32::consts::PI;
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
    client: Res<Client>,
    mut peers: ResMut<Peers>,
    mut rand: GlobalEntropy<WyRand>,
    mut count: ResMut<SyncCount>,
) {
    peers.my_id = client.user().steam_id();
    client.networking_utils().init_relay_network_access();
    client.networking_messages().session_request_callback(|r| {
        r.accept();
    });
    let networking_sockets = client.networking_sockets();
    let poll_group = networking_sockets.create_poll_group();
    commands.insert_resource(PollGroup {
        poll: poll_group,
        listen: networking_sockets
            .create_listen_socket_p2p(0, None)
            .expect("handle to be valid")
            .into(),
    });
    framepace.limiter = Limiter::from_framerate(60.0);
    let card_stock = meshes.add(Rectangle::new(CARD_WIDTH, CARD_HEIGHT));
    let card_back = asset_server.load("back.jpg");
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(card_back),
        alpha_mode: AlphaMode::Opaque,
        unlit: true,
        ..default()
    });
    let card_side = materials.add(StandardMaterial {
        base_color: bevy::prelude::Color::srgb_u8(0x11, 0x0F, 0x02),
        unlit: true,
        ..Default::default()
    });
    commands.insert_resource(CardBase {
        stock: card_stock,
        back: material_handle,
        side: card_side,
    });
    let mat = asset_server.load("mat.png");
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
        Mesh3d(mat_mesh.clone_weak()),
        MeshMaterial3d(playmat.clone_weak()),
        transform,
    ));
    let mut transform = Transform::from_xyz(-MAT_WIDTH / 2.0, 0.0, MAT_HEIGHT / 2.0);
    transform.rotate_x(-PI / 2.0);
    commands.spawn((
        Mesh3d(mat_mesh.clone_weak()),
        MeshMaterial3d(playmat.clone_weak()),
        transform,
    ));
    let mut transform = Transform::from_xyz(MAT_WIDTH / 2.0, 0.0, -MAT_HEIGHT / 2.0);
    transform.rotate_x(-PI / 2.0);
    transform.rotate_y(PI);
    commands.spawn((
        Mesh3d(mat_mesh.clone_weak()),
        MeshMaterial3d(playmat.clone_weak()),
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
            unlit: true,
            ..Default::default()
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
            unlit: true,
            ..Default::default()
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
            unlit: true,
            ..Default::default()
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
            unlit: true,
            ..Default::default()
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
            unlit: true,
            ..Default::default()
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
            unlit: true,
            ..Default::default()
        })),
    ));
    commands.spawn((
        Camera3d::default(),
        Msaa::Sample8,
        Transform::from_xyz(0.0, START_Y, START_Z).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands
        .spawn((
            Collider::cuboid(256.0, 256.0, 256.0),
            Transform::from_xyz(0.0, 128.0, 0.0),
            RigidBody::Dynamic,
            GravityScale(GRAVITY),
            Mesh3d(meshes.add(Cuboid::from_length(256.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: bevy::prelude::Color::WHITE,
                unlit: true,
                ..Default::default()
            })),
            SyncObjectMe::new(&mut rand, &mut count),
        ))
        .with_children(|parent| {
            for i in 1..=6 {
                let (x, y, z) = match i {
                    1 => (0.0, 129.0, 0.0),
                    2 => (129.0, 0.0, 0.0),
                    3 => (0.0, 0.0, 129.0),
                    4 => (0.0, 0.0, -129.0),
                    5 => (-129.0, 0.0, 0.0),
                    6 => (0.0, -129.0, 0.0),
                    _ => unreachable!(),
                };
                parent.spawn((
                    Transform::from_xyz(x, y, z).looking_at(Vec3::default(), Dir3::Z),
                    Text3d::new(i.to_string()),
                    Mesh3d(meshes.add(Rectangle::new(256.0, 256.0))),
                    MeshMaterial3d(asset_server.add(StandardMaterial {
                        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
                        base_color: bevy::prelude::Color::BLACK,
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..Default::default()
                    })),
                    Text3dStyling {
                        size: 128.0,
                        anchor: TextAnchor::CENTER,
                        ..Default::default()
                    },
                ));
            }
        });
}
#[derive(Component)]
pub struct Wall;
#[derive(Component)]
pub struct Floor;
#[derive(Component)]
pub struct Ceiling;
