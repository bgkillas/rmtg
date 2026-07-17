use crate::assets::Asset;
use crate::camera::default_cam_pos;
use crate::shapes::ShapeMesh as _;
use crate::shapes::cube::Cube;
use crate::shapes::dodecahedron::Dodecahedron;
use crate::shapes::icosahedron::Icosahedron;
use crate::shapes::octahedron::Octahedron;
use crate::shapes::tetrahedron::Tetrahedron;
use crate::{FLOOR_COLOR, T, W};
use avian3d::prelude::{Collider, RigidBody};
use bevy::camera::{Camera3d, Exposure, PhysicalCameraParameters};
use bevy::color::Color;
use bevy::light::light_consts::lux::OVERCAST_DAY;
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLight};
use bevy::math::{Quat, Vec3};
use bevy::mesh::Mesh3d;
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::picking::Pickable;
use bevy::prelude::{Commands, Cuboid, MeshPickingCamera, MeshPickingSettings, ResMut, Transform};
use std::f32::consts::PI;
pub fn startup(mut commands: Commands, mut pick: ResMut<MeshPickingSettings>, mut asset: Asset) {
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
        default_cam_pos(0),
        Camera3d::default(),
        Exposure::from_physical_camera(PhysicalCameraParameters {
            aperture_f_stops: 1.0,
            shutter_speed_s: 1.0 / 125.0,
            sensitivity_iso: 100.0,
            sensor_height: 0.01866,
        }),
        MeshPickingCamera,
    ));
    commands.spawn((
        Cube::bundle_dice(1.0, Color::WHITE, Color::BLACK, &mut asset),
        Transform::from_xyz(-8.0, 1.0, 0.0),
        Pickable::default(),
    ));
    commands.spawn((
        Dodecahedron::bundle_dice(1.0, Color::WHITE, Color::BLACK, &mut asset),
        Transform::from_xyz(-6.0, 1.0, 0.0),
        Pickable::default(),
    ));
    commands.spawn((
        Icosahedron::bundle_dice(1.0, Color::WHITE, Color::BLACK, &mut asset),
        Transform::from_xyz(-4.0, 1.0, 0.0),
        Pickable::default(),
    ));
    commands.spawn((
        Octahedron::bundle_dice(1.0, Color::WHITE, Color::BLACK, &mut asset),
        Transform::from_xyz(-2.0, 1.0, 0.0),
        Pickable::default(),
    ));
    commands.spawn((
        Tetrahedron::bundle_dice(1.0, Color::WHITE, Color::BLACK, &mut asset),
        Transform::from_xyz(0.0, 1.0, 0.0),
        Pickable::default(),
    ));
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
