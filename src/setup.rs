use crate::*;
use bevy_framepace::{FramepaceSettings, Limiter};
use std::f32::consts::PI;
const MAT_SCALE: f32 = 10.0;
pub const MAT_WIDTH: f32 = 872.0 * MAT_SCALE;
pub const MAT_HEIGHT: f32 = 525.0 * MAT_SCALE;
pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut framepace: ResMut<FramepaceSettings>,
) {
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
    let mut transform = Transform::from_xyz(MAT_WIDTH / 2.0, -8.0, MAT_HEIGHT / 2.0);
    transform.rotate_x(-PI / 2.0);
    commands.spawn((
        Mesh3d(mat_mesh.clone_weak()),
        MeshMaterial3d(playmat.clone_weak()),
        transform,
    ));
    let mut transform = Transform::from_xyz(-MAT_WIDTH / 2.0, -8.0, MAT_HEIGHT / 2.0);
    transform.rotate_x(-PI / 2.0);
    commands.spawn((
        Mesh3d(mat_mesh.clone_weak()),
        MeshMaterial3d(playmat.clone_weak()),
        transform,
    ));
    let mut transform = Transform::from_xyz(MAT_WIDTH / 2.0, -8.0, -MAT_HEIGHT / 2.0);
    transform.rotate_x(-PI / 2.0);
    transform.rotate_y(PI);
    commands.spawn((
        Mesh3d(mat_mesh.clone_weak()),
        MeshMaterial3d(playmat.clone_weak()),
        transform,
    ));
    let mut transform = Transform::from_xyz(-MAT_WIDTH / 2.0, -8.0, -MAT_HEIGHT / 2.0);
    transform.rotate_x(-PI / 2.0);
    transform.rotate_y(PI);
    commands.spawn((Mesh3d(mat_mesh), MeshMaterial3d(playmat), transform));
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
        Collider::cuboid(128.0, 128.0, 128.0),
        Transform::from_xyz(0.0, 256.0, 0.0),
        RigidBody::Dynamic,
        GravityScale(GRAVITY),
        Ccd::enabled(),
        Velocity::zero(),
        Damping {
            linear_damping: DAMPING,
            angular_damping: 0.0,
        },
        AdditionalMassProperties::Mass(4.0),
        Mesh3d(meshes.add(RegularPolygon::new(128.0, 4))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::prelude::Color::WHITE,
            unlit: true,
            ..Default::default()
        })),
    ));
}
