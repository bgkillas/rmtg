use crate::assets::{Asset, CardBase, TextMesh};
use crate::camera::default_cam_pos;
use crate::net::Peer;
use crate::shapes::ShapeMesh as _;
use crate::shapes::cube::Cube;
use crate::shapes::dodecahedron::Dodecahedron;
use crate::shapes::icosahedron::Icosahedron;
use crate::shapes::octahedron::Octahedron;
use crate::shapes::tetrahedron::Tetrahedron;
use crate::{CARD_HEIGHT, CARD_STOCK_COLOR, CARD_THICKNESS, CARD_WIDTH, FLOOR_COLOR, FONT, T, W};
use avian3d::prelude::{Collider, RigidBody};
use bevy::asset::{AssetId, Assets};
use bevy::camera::{
    Camera3d, Exposure, PerspectiveProjection, PhysicalCameraParameters, Projection,
};
use bevy::color::Color;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::image::Image;
use bevy::light::light_consts::lux::OVERCAST_DAY;
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLight};
use bevy::material::AlphaMode;
use bevy::math::{Quat, Vec3};
use bevy::mesh::{Mesh, Mesh3d};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::picking::Pickable;
use bevy::prelude::{
    Commands, Cuboid, MeshPickingCamera, MeshPickingSettings, Msaa, Rectangle, ResMut, Transform,
};
use bevy::text::Font;
use bevy_rich_text3d::TextAtlas;
use importer::image::parse_bytes;
use std::f32::consts::PI;
pub fn startup(
    mut commands: Commands,
    mut pick: ResMut<MeshPickingSettings>,
    mut fonts: ResMut<Assets<Font>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mesh = materials.add(StandardMaterial {
        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..StandardMaterial::default()
    });
    let stock = meshes.add(Rectangle::new(CARD_WIDTH, CARD_HEIGHT));
    let back_img = parse_bytes(include_bytes!("../../assets/back.png")).unwrap();
    let back = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(back_img)),
        alpha_mode: AlphaMode::Opaque,
        unlit: true,
        ..StandardMaterial::default()
    });
    let color = materials.add(StandardMaterial {
        base_color: CARD_STOCK_COLOR,
        unlit: true,
        ..StandardMaterial::default()
    });
    commands.insert_resource(CardBase { stock, back, color });
    commands.insert_resource(TextMesh { mesh });
    let font = Font::from_bytes(FONT.to_vec());
    fonts.insert(AssetId::<Font>::DEFAULT_UUID, font).unwrap();
    pick.require_markers = true;
    commands.spawn((
        DirectionalLight {
            illuminance: OVERCAST_DAY,
            shadow_maps_enabled: true,
            ..DirectionalLight::default()
        },
        Transform {
            translation: Vec3::new(0.0, 4.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.0),
            ..Transform::default()
        },
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..CascadeShadowConfigBuilder::default()
        }
        .build(),
    ));
    commands.spawn((
        default_cam_pos(Peer::default()),
        Camera3d::default(),
        Exposure::from_physical_camera(PhysicalCameraParameters {
            aperture_f_stops: 1.0,
            shutter_speed_s: 1.0 / 125.0,
            sensitivity_iso: 100.0,
            sensor_height: 0.01866,
        }),
        MeshPickingCamera,
        Projection::Perspective(PerspectiveProjection {
            fov: PI / 3.0,
            near: CARD_THICKNESS / 32.0,
            far: W * 2.0,
            ..PerspectiveProjection::default()
        }),
        Tonemapping::None,
        Msaa::Sample8,
    ));
}
pub fn spawn_objects(mut commands: Commands, mut asset: Asset) {
    Cube::insert_dice(
        Color::WHITE,
        Color::BLACK,
        &mut asset,
        commands.spawn((Transform::from_xyz(-8.0, 1.0, 0.0), Pickable::default())),
    );
    Dodecahedron::insert_dice(
        Color::WHITE,
        Color::BLACK,
        &mut asset,
        commands.spawn((Transform::from_xyz(-6.0, 1.0, 0.0), Pickable::default())),
    );
    Icosahedron::insert_dice(
        Color::WHITE,
        Color::BLACK,
        &mut asset,
        commands.spawn((Transform::from_xyz(-4.0, 1.0, 0.0), Pickable::default())),
    );
    Octahedron::insert_dice(
        Color::WHITE,
        Color::BLACK,
        &mut asset,
        commands.spawn((Transform::from_xyz(-2.0, 1.0, 0.0), Pickable::default())),
    );
    Tetrahedron::insert_dice(
        Color::WHITE,
        Color::BLACK,
        &mut asset,
        commands.spawn((Transform::from_xyz(0.0, 1.0, 0.0), Pickable::default())),
    );
    commands.spawn((
        Transform::from_xyz(0.0, -T / 2.0, 0.0),
        Collider::cuboid(2.0 * W + T, T, 2.0 * W + T),
        RigidBody::Static,
        Mesh3d(asset.meshes.add(Cuboid::new(2.0 * W, T, 2.0 * W))),
        MeshMaterial3d(asset.materials.add(StandardMaterial {
            base_color: FLOOR_COLOR,
            unlit: true,
            depth_bias: f32::NEG_INFINITY,
            ..StandardMaterial::default()
        })),
    ));
}
