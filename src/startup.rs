use crate::camera::default_cam_pos;
use crate::shapes::NewShape as _;
use crate::shapes::cube::CubeOutline;
use crate::shapes::dodecahedron::{Dodecahedron, DodecahedronOutline};
use crate::shapes::icosahedron::{Icosahedron, IcosahedronOutline};
use crate::shapes::octahedron::{Octahedron, OctahedronOutline};
use crate::shapes::tetrahedron::{Tetrahedron, TetrahedronOutline};
use bevy::asset::Assets;
use bevy::camera::{Camera3d, Exposure, PhysicalCameraParameters};
use bevy::color::Color;
use bevy::light::light_consts::lux::OVERCAST_DAY;
use bevy::light::{CascadeShadowConfigBuilder, DirectionalLight};
use bevy::math::{Quat, Vec3};
use bevy::mesh::{Mesh, Mesh3d};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::picking::Pickable;
use bevy::prelude::{Commands, Cuboid, MeshPickingCamera, MeshPickingSettings, ResMut, Transform};
use bevy_polyline::material::{PolylineMaterial, PolylineMaterialHandle};
use bevy_polyline::polyline::{Polyline, PolylineHandle};
use std::f32::consts::PI;
pub fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut pick: ResMut<MeshPickingSettings>,
) {
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
    commands
        .spawn((
            Transform::from_xyz(-2.0, 2.0, 0.0),
            Mesh3d(meshes.add(Dodecahedron::from_length(0.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..StandardMaterial::default()
            })),
            Pickable::default(),
        ))
        .with_child((
            PolylineHandle(polylines.add(DodecahedronOutline::from_length(0.5))),
            PolylineMaterialHandle(polyline_materials.add(PolylineMaterial {
                width: 8.0,
                color: Color::BLACK.to_linear(),
                perspective: true,
                depth_bias: -0.00001,
            })),
        ));
    commands
        .spawn((
            Transform::from_xyz(-0.5, 2.0, 0.0),
            Mesh3d(meshes.add(Icosahedron::from_length(0.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..StandardMaterial::default()
            })),
            Pickable::default(),
        ))
        .with_child((
            PolylineHandle(polylines.add(IcosahedronOutline::from_length(0.5))),
            PolylineMaterialHandle(polyline_materials.add(PolylineMaterial {
                width: 8.0,
                color: Color::BLACK.to_linear(),
                perspective: true,
                depth_bias: -0.00001,
            })),
        ));
    commands
        .spawn((
            Transform::from_xyz(0.25, 2.0, 0.0),
            Mesh3d(meshes.add(Cuboid::from_length(0.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..StandardMaterial::default()
            })),
            Pickable::default(),
        ))
        .with_child((
            PolylineHandle(polylines.add(CubeOutline::from_length(0.5))),
            PolylineMaterialHandle(polyline_materials.add(PolylineMaterial {
                width: 8.0,
                color: Color::BLACK.to_linear(),
                perspective: true,
                depth_bias: -0.00001,
            })),
        ));
    commands
        .spawn((
            Transform::from_xyz(1.5, 2.0, 0.0),
            Mesh3d(meshes.add(Tetrahedron::from_height(0.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..StandardMaterial::default()
            })),
            Pickable::default(),
        ))
        .with_child((
            PolylineHandle(polylines.add(TetrahedronOutline::from_height(0.5))),
            PolylineMaterialHandle(polyline_materials.add(PolylineMaterial {
                width: 8.0,
                color: Color::BLACK.to_linear(),
                perspective: true,
                depth_bias: 0.0,
            })),
        ));
    commands
        .spawn((
            Transform::from_xyz(1.0, 2.0, 0.0),
            Mesh3d(meshes.add(Octahedron::from_length(0.5))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..StandardMaterial::default()
            })),
            Pickable::default(),
        ))
        .with_child((
            PolylineHandle(polylines.add(OctahedronOutline::from_length(0.5))),
            PolylineMaterialHandle(polyline_materials.add(PolylineMaterial {
                width: 8.0,
                color: Color::BLACK.to_linear(),
                perspective: true,
                depth_bias: -0.00001,
            })),
        ));
}
