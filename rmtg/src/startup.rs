use crate::assets::{AssetManager, CardBase, OutlineMaterials, ShapeMeshes, TextMesh};
use crate::camera::{CameraVelocity, default_cam_pos};
use crate::net::Peer;
use crate::physics::WorldLayer;
use crate::pile::Pile;
use crate::shapes::ShapeMesh as _;
use crate::shapes::coin::Coin;
use crate::shapes::cube::Cube;
use crate::shapes::dodecahedron::Dodecahedron;
use crate::shapes::icosahedron::Icosahedron;
use crate::shapes::octahedron::Octahedron;
use crate::shapes::tetrahedron::Tetrahedron;
use crate::shapes::trapezohedron::Trapezohedron;
use crate::ui::calc::CalcMenu;
use crate::ui::chat::TextMenu;
use crate::ui::esc_menu::EscMenu;
use crate::ui::moxfield::MoxfieldMenu;
use crate::ui::side::SideMenu;
use crate::ui::tasks::TasksCounter;
use crate::{
    CARD_HEIGHT, CARD_THICKNESS, CARD_WIDTH, CEILING_COLOR, FLOOR_COLOR, FONT, MAT_WIDTH, T, W,
    WALL_COLOR,
};
use avian3d::prelude::{Collider, CollisionLayers, LayerMask, RigidBody};
use bevy::asset::{AssetId, Assets};
use bevy::camera::{
    Camera3d, Exposure, PerspectiveProjection, PhysicalCameraParameters, Projection,
};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::image::Image;
use bevy::light::GlobalAmbientLight;
use bevy::material::AlphaMode;
use bevy::math::bounding::Aabb3d;
use bevy::math::{Vec3, Vec3A};
use bevy::mesh::{Mesh, Mesh3d};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Commands, Component, Cuboid, Msaa, ResMut, Transform};
use bevy::text::Font;
use bevy_rich_text3d::TextAtlas;
use importer::card::{Handles, MaybeHandles, SubCard};
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
    commands.insert_resource(CardBase::new(&mut meshes, &mut materials, &mut images));
    commands.insert_resource(ShapeMeshes::new(&mut meshes, &mut materials));
    commands.insert_resource(OutlineMaterials::new(&mut materials));
    let mesh = materials.add(StandardMaterial {
        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE),
        alpha_mode: AlphaMode::AlphaToCoverage,
        unlit: true,
        ..StandardMaterial::default()
    });
    commands.insert_resource(TextMesh { mesh });
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
        #[cfg(any(not(target_family = "wasm"), feature = "webgpu"))]
        bevy::anti_alias::contrast_adaptive_sharpening::ContrastAdaptiveSharpening {
            enabled: true,
            sharpening_strength: 1.0,
            denoise: false,
        },
        CameraVelocity::default(),
    ));
    commands.spawn(TextMenu::bundle());
    commands.spawn(EscMenu::bundle());
    commands.spawn(SideMenu::bundle());
    commands.spawn(MoxfieldMenu::bundle());
    commands.spawn(CalcMenu::bundle());
    commands.spawn(TasksCounter::bundle());
}
pub fn spawn_objects(
    mut commands: Commands,
    asset: AssetManager,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut card = SubCard::default();
    card.face_handles = MaybeHandles::Some(Handles {
        image: asset.card.back_image.clone(),
        material: asset.card.back.clone(),
    });
    commands.spawn((
        Transform::from_xyz(MAT_WIDTH + CARD_WIDTH, CARD_THICKNESS, 0.0),
        Pile::from(card).bundle(),
    ));
    let x_unit = MAT_WIDTH + CARD_WIDTH;
    let z_unit = CARD_HEIGHT;
    for i in 0..4 {
        let (rev_x, rev_z) = match i {
            0 => (x_unit, z_unit),
            1 => (-x_unit, z_unit),
            2 => (-x_unit, -z_unit),
            _ => (x_unit, -z_unit),
        };
        Icosahedron::insert_dice(
            &asset,
            commands.spawn(Transform::from_xyz(rev_x, Cube::HEIGHT / 2.0, rev_z)),
        );
        Dodecahedron::insert_dice(
            &asset,
            commands.spawn(Transform::from_xyz(rev_x, Cube::HEIGHT / 2.0, rev_z * 1.5)),
        );
        Trapezohedron::insert_dice(
            &asset,
            commands.spawn(Transform::from_xyz(rev_x, Cube::HEIGHT / 2.0, rev_z * 2.0)),
        );
        Octahedron::insert_dice(
            &asset,
            commands.spawn(Transform::from_xyz(rev_x, Cube::HEIGHT / 2.0, rev_z * 2.5)),
        );
        Cube::insert_dice(
            &asset,
            commands.spawn(Transform::from_xyz(rev_x, Cube::HEIGHT / 2.0, rev_z * 3.0)),
        );
        Tetrahedron::insert_dice(
            &asset,
            commands.spawn(Transform::from_xyz(rev_x, Cube::HEIGHT / 2.0, rev_z * 3.5)),
        );
        Coin::insert_dice(
            &asset,
            commands.spawn(Transform::from_xyz(rev_x, Cube::HEIGHT / 2.0, rev_z * 4.0)),
        );
    }
    let mesh = meshes.add(Cuboid::new(2.0 * W, T, 2.0 * W));
    commands.spawn((
        Transform::from_xyz(0.0, -T / 2.0, 0.0),
        Collider::cuboid(2.0 * W + T, T, 2.0 * W + T),
        RigidBody::Static,
        Mesh3d(mesh.clone()),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: FLOOR_COLOR,
            unlit: true,
            depth_bias: f32::NEG_INFINITY,
            ..StandardMaterial::default()
        })),
        Floor,
        CollisionLayers::new(WorldLayer::Floor, LayerMask::ALL),
    ));
    let wall = materials.add(StandardMaterial {
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
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: CEILING_COLOR,
            unlit: true,
            ..StandardMaterial::default()
        })),
        CollisionLayers::new(WorldLayer::Default, LayerMask::ALL),
    ));
}
#[must_use]
pub fn wall_aabb() -> Aabb3d {
    Aabb3d {
        min: Vec3A::new(-W, 0.0, -W),
        max: Vec3A::new(W, W, W),
    }
}
#[derive(Component, Clone)]
pub struct Floor;
#[derive(Component, Clone)]
pub struct Wall;
#[derive(Component, Clone)]
pub struct Ceiling;
