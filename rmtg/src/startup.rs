use crate::assets::{Asset, CardBase};
use crate::camera::{CameraVelocity, default_cam_pos};
use crate::net::Peer;
use crate::physics::WorldLayer;
use crate::shapes::coin::Coin;
use crate::shapes::cube::Cube;
use crate::shapes::dodecahedron::Dodecahedron;
use crate::shapes::icosahedron::Icosahedron;
use crate::shapes::octahedron::Octahedron;
use crate::shapes::tetrahedron::Tetrahedron;
use crate::shapes::trapezohedron::Trapezohedron;
use crate::shapes::{OUTLINE_COLOR, ShapeMesh as _};
use crate::ui::chat::chat_bundle;
use crate::ui::esc_menu::esc_menu_bundle;
use crate::{
    CARD_HEIGHT, CARD_STOCK_COLOR, CARD_THICKNESS, CARD_WIDTH, CEILING_COLOR, FLOOR_COLOR, FONT, T,
    W, WALL_COLOR,
};
use avian3d::prelude::{Collider, CollisionLayers, LayerMask, RigidBody};
use bevy::anti_alias::contrast_adaptive_sharpening::ContrastAdaptiveSharpening;
use bevy::asset::{AssetId, Assets};
use bevy::camera::{
    Camera3d, Exposure, PerspectiveProjection, PhysicalCameraParameters, Projection,
};
use bevy::color::Color;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::image::Image;
use bevy::light::GlobalAmbientLight;
use bevy::material::AlphaMode;
use bevy::math::Vec3;
use bevy::mesh::{Mesh, Mesh3d};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Commands, Component, Cuboid, Msaa, Rectangle, ResMut, Transform};
use bevy::text::Font;
use importer::image::parse_bytes;
use std::f32::consts::PI;
pub fn startup(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut light: ResMut<GlobalAmbientLight>,
) {
    light.brightness = 100.0;
    let stock = meshes.add(Rectangle::new(CARD_WIDTH, CARD_HEIGHT));
    let back_img = parse_bytes(include_bytes!("../../assets/back.png")).unwrap();
    let back = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(back_img)),
        alpha_mode: AlphaMode::Premultiplied,
        unlit: true,
        ..StandardMaterial::default()
    });
    let color = materials.add(StandardMaterial {
        base_color: CARD_STOCK_COLOR,
        unlit: true,
        ..StandardMaterial::default()
    });
    commands.insert_resource(CardBase { stock, back, color });
    let font = Font::from_bytes(FONT.to_vec());
    fonts.insert(AssetId::<Font>::DEFAULT_UUID, font).unwrap();
    commands.spawn((
        default_cam_pos(Peer::default()),
        Camera3d::default(),
        Exposure::from_physical_camera(PhysicalCameraParameters {
            aperture_f_stops: 1.0,
            shutter_speed_s: 1.0 / 125.0,
            sensitivity_iso: 100.0,
            sensor_height: 0.01866,
        }),
        Projection::Perspective(PerspectiveProjection {
            fov: PI / 3.0,
            near: CARD_THICKNESS,
            far: 4.0 * W,
            ..PerspectiveProjection::default()
        }),
        Tonemapping::None,
        Msaa::Sample4,
        ContrastAdaptiveSharpening {
            enabled: true,
            sharpening_strength: 1.0,
            denoise: false,
        },
        CameraVelocity::default(),
    ));
    commands.spawn(chat_bundle());
    commands.spawn(esc_menu_bundle());
}
pub fn spawn_objects(mut commands: Commands, mut asset: Asset) {
    for i in 0..4 {
        let (rev_x, rev_z) = match i {
            0 => (1.0, 1.0),
            1 => (-1.0, 1.0),
            2 => (-1.0, -1.0),
            _ => (1.0, -1.0),
        };
        let color = Color::WHITE;
        Icosahedron::insert_dice(
            color,
            OUTLINE_COLOR,
            &mut asset,
            commands.spawn(Transform::from_xyz(
                rev_x * 9.0,
                Cube::HEIGHT / 2.0,
                rev_z * 1.0,
            )),
        );
        Dodecahedron::insert_dice(
            color,
            OUTLINE_COLOR,
            &mut asset,
            commands.spawn(Transform::from_xyz(
                rev_x * 9.0,
                Cube::HEIGHT / 2.0,
                rev_z * 1.5,
            )),
        );
        Trapezohedron::insert_dice(
            color,
            OUTLINE_COLOR,
            &mut asset,
            commands.spawn(Transform::from_xyz(
                rev_x * 9.0,
                Cube::HEIGHT / 2.0,
                rev_z * 2.0,
            )),
        );
        Octahedron::insert_dice(
            color,
            OUTLINE_COLOR,
            &mut asset,
            commands.spawn(Transform::from_xyz(
                rev_x * 9.0,
                Cube::HEIGHT / 2.0,
                rev_z * 2.5,
            )),
        );
        Cube::insert_dice(
            color,
            OUTLINE_COLOR,
            &mut asset,
            commands.spawn(Transform::from_xyz(
                rev_x * 9.0,
                Cube::HEIGHT / 2.0,
                rev_z * 3.0,
            )),
        );
        Tetrahedron::insert_dice(
            color,
            OUTLINE_COLOR,
            &mut asset,
            commands.spawn(Transform::from_xyz(
                rev_x * 9.0,
                Cube::HEIGHT / 2.0,
                rev_z * 3.5,
            )),
        );
        Coin::insert_dice(
            color,
            OUTLINE_COLOR,
            &mut asset,
            commands.spawn(Transform::from_xyz(
                rev_x * 9.0,
                Cube::HEIGHT / 2.0,
                rev_z * 4.0,
            )),
        );
    }
    let mesh = asset.meshes.add(Cuboid::new(2.0 * W, T, 2.0 * W));
    commands.spawn((
        Transform::from_xyz(0.0, -T / 2.0, 0.0),
        Collider::cuboid(2.0 * W + T, T, 2.0 * W + T),
        RigidBody::Static,
        Mesh3d(mesh.clone()),
        MeshMaterial3d(asset.materials.add(StandardMaterial {
            base_color: FLOOR_COLOR,
            unlit: true,
            depth_bias: f32::NEG_INFINITY,
            ..StandardMaterial::default()
        })),
        Floor,
        CollisionLayers::new(WorldLayer::Floor, LayerMask::ALL),
    ));
    let wall = asset.materials.add(StandardMaterial {
        base_color: WALL_COLOR,
        unlit: true,
        ..StandardMaterial::default()
    });
    for i in 0..4 {
        let s = W + T / 2.0;
        let (x, y) = match i {
            0 => (s, 0.0),
            1 => (-s, 0.0),
            2 => (0.0, s),
            3 => (0.0, -s),
            _ => unreachable!(),
        };
        commands.spawn((
            Transform::from_xyz(x, W, y).looking_to(Vec3::Y, -Vec3::new(x, 0.0, y)),
            Collider::cuboid(2.0 * W + T, T, 2.0 * W + T),
            RigidBody::Static,
            Wall,
            Mesh3d(mesh.clone()),
            MeshMaterial3d(wall.clone()),
            CollisionLayers::new(WorldLayer::Default, LayerMask::ALL),
        ));
    }
    commands.spawn((
        Transform::from_xyz(0.0, 2.0 * W + T / 2.0, 0.0),
        Collider::cuboid(2.0 * W + T, T, 2.0 * W + T),
        RigidBody::Static,
        Ceiling,
        Mesh3d(mesh),
        MeshMaterial3d(asset.materials.add(StandardMaterial {
            base_color: CEILING_COLOR,
            unlit: true,
            ..StandardMaterial::default()
        })),
        CollisionLayers::new(WorldLayer::Default, LayerMask::ALL),
    ));
}
#[derive(Component, Clone)]
pub struct Floor;
#[derive(Component, Clone)]
pub struct Wall;
#[derive(Component, Clone)]
pub struct Ceiling;
