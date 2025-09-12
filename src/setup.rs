use crate::*;
use bevy_framepace::{FramepaceSettings, Limiter};
use bevy_prng::WyRand;
use bevy_rand::global::GlobalEntropy;
pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut framepace: ResMut<FramepaceSettings>,
    mut rand: GlobalEntropy<WyRand>,
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
    commands.insert_resource(CardSide(card_side));
    commands.insert_resource(CardBack(material_handle));
    commands.insert_resource(CardStock(card_stock));
    commands.spawn((
        Transform::from_xyz(0.0, 64.0, START_Z / 3.0),
        Hand::default(),
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
}
