#![allow(clippy::shadow_reuse)]
use crate::shapes::deck_outline::DeckOutline;
use crate::shapes::{NewShape as _, OUTLINE_COLOR, OUTLINE_DEPTH_BIAS, Shape, ShapeOutline as _};
use crate::{
    CARD_HEIGHT, CARD_STOCK_COLOR, CARD_STOCK_INBETWEEN_COLOR, CARD_THICKNESS, CARD_WIDTH, PLAYER,
};
use avian3d::parry::glamx::{Quat, Vec3};
use bevy::asset::{Assets, Handle};
use bevy::color::Color;
use bevy::ecs::system::SystemParam;
use bevy::image::Image;
use bevy::material::AlphaMode;
use bevy::mesh::{
    CircularMeshUvMode, CircularSectorMeshBuilder, ExtrusionBuilder, Mesh, MeshBuilder as _,
    RingMeshBuilder,
};
use bevy::pbr::StandardMaterial;
use bevy::prelude::{CircularSector, Rectangle, Res, Resource, Ring};
use enum_map::EnumMap;
use importer::CARD_CORNER_RADIUS;
use importer::card::Handles;
use importer::image::parse_bytes;
use std::array;
use std::f32::consts::PI;
#[derive(SystemParam)]
pub struct AssetManager<'w> {
    pub card: Res<'w, CardBase>,
    pub text_mesh: Res<'w, TextMesh>,
    pub meshes: Res<'w, ShapeMeshes>,
    pub outlines: Res<'w, OutlineMaterials>,
}
#[derive(Resource)]
pub struct OutlineMaterials {
    pub default: Handle<StandardMaterial>,
    pub players: [Handle<StandardMaterial>; PLAYER.len()],
}
#[derive(Resource)]
pub struct ShapeMeshes {
    pub map: EnumMap<Shape, (Handle<Mesh>, Handle<Mesh>)>,
    pub material: Handle<StandardMaterial>,
}
#[derive(Resource)]
pub struct CardBase {
    pub stock: Handle<Mesh>,
    pub side: Handle<Mesh>,
    pub outline: Handle<Mesh>,
    pub back: Handle<StandardMaterial>,
    pub back_image: Handle<Image>,
    pub color: Handle<StandardMaterial>,
    pub inbetween_color: Handle<StandardMaterial>,
}
impl ShapeMeshes {
    pub fn new(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            map: EnumMap::from_fn(|e: Shape| (meshes.add(e.mesh()), meshes.add(e.outline_mesh()))),
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..StandardMaterial::default()
            }),
        }
    }
}
impl OutlineMaterials {
    pub fn new(materials: &mut Assets<StandardMaterial>) -> Self {
        Self {
            default: materials.add(StandardMaterial {
                base_color: OUTLINE_COLOR,
                unlit: true,
                depth_bias: OUTLINE_DEPTH_BIAS,
                ..StandardMaterial::default()
            }),
            players: array::from_fn(|i| {
                materials.add(StandardMaterial {
                    base_color: PLAYER[i],
                    unlit: true,
                    depth_bias: OUTLINE_DEPTH_BIAS,
                    ..StandardMaterial::default()
                })
            }),
        }
    }
}
impl CardBase {
    pub fn new(
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        images: &mut Assets<Image>,
    ) -> Self {
        let stock = meshes.add(Rectangle::new(CARD_WIDTH, CARD_HEIGHT));
        let back_img = parse_bytes(include_bytes!("../../assets/back.png")).unwrap();
        let back_image = images.add(back_img);
        let back = materials.add(StandardMaterial {
            base_color_texture: Some(back_image.clone()),
            alpha_mode: AlphaMode::AlphaToCoverage,
            unlit: true,
            ..StandardMaterial::default()
        });
        let color = materials.add(StandardMaterial {
            base_color: CARD_STOCK_COLOR,
            unlit: true,
            ..StandardMaterial::default()
        });
        let inbetween_color = materials.add(StandardMaterial {
            base_color: CARD_STOCK_INBETWEEN_COLOR,
            unlit: true,
            ..StandardMaterial::default()
        });
        Self {
            stock,
            side: meshes.add(generate_side_mesh()),
            outline: meshes.add(DeckOutline::from_height(0.0)),
            back,
            back_image,
            color,
            inbetween_color,
        }
    }
}
#[derive(Resource)]
pub struct TextMesh {
    pub mesh: Handle<StandardMaterial>,
}
pub fn register_card(materials: &mut Assets<StandardMaterial>, image: Handle<Image>) -> Handles {
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(image.clone()),
        alpha_mode: AlphaMode::AlphaToCoverage,
        unlit: true,
        ..StandardMaterial::default()
    });
    Handles { image, material }
}
#[must_use]
pub fn generate_side_mesh() -> Mesh {
    let del = CARD_WIDTH * CARD_CORNER_RADIUS;
    let left_right = Mesh::from(Rectangle::new(CARD_THICKNESS, CARD_HEIGHT - 2.0 * del));
    let front_back = Mesh::from(Rectangle::new(CARD_WIDTH - 2.0 * del, CARD_THICKNESS));
    let mut left = left_right.clone();
    left.rotate_by(Quat::from_rotation_arc(Vec3::NEG_Z, Vec3::X));
    left.rotate_by(Quat::from_rotation_x(PI / 2.0));
    left.translate_by(Vec3::new(-CARD_WIDTH / 2.0, 0.0, 0.0));
    let mut right = left_right.clone();
    right.rotate_by(Quat::from_rotation_arc(Vec3::NEG_Z, Vec3::NEG_X));
    right.rotate_by(Quat::from_rotation_x(PI / 2.0));
    right.translate_by(Vec3::new(CARD_WIDTH / 2.0, 0.0, 0.0));
    let mut front = front_back.clone();
    front.rotate_by(Quat::from_rotation_arc(Vec3::NEG_Z, Vec3::Z));
    front.translate_by(Vec3::new(0.0, 0.0, -CARD_HEIGHT / 2.0));
    let mut back = front_back.clone();
    back.translate_by(Vec3::new(0.0, 0.0, CARD_HEIGHT / 2.0));
    left.merge(&right).unwrap();
    left.merge(&front).unwrap();
    left.merge(&back).unwrap();
    for corner in 0..4 {
        let resolution = 32;
        let mut sector = ExtrusionBuilder::<Ring<CircularSector>> {
            base_builder: RingMeshBuilder {
                inner_shape_builder: CircularSectorMeshBuilder {
                    sector: CircularSector::new(del - DeckOutline::THICKNESS / 2.0, PI / 4.0),
                    resolution,
                    uv_mode: CircularMeshUvMode::default(),
                },
                outer_shape_builder: CircularSectorMeshBuilder {
                    sector: CircularSector::new(del, PI / 4.0),
                    resolution,
                    uv_mode: CircularMeshUvMode::default(),
                },
            },
            half_depth: CARD_THICKNESS / 2.0,
            segments: 1,
        }
        .build();
        let wid = CARD_WIDTH / 2.0;
        let hei = CARD_HEIGHT / 2.0;
        let vec = match corner {
            0 => Vec3::new(wid - del, 0.0, hei - del),
            1 => Vec3::new(del - wid, 0.0, hei - del),
            2 => Vec3::new(del - wid, 0.0, del - hei),
            3 => Vec3::new(wid - del, 0.0, del - hei),
            _ => unreachable!(),
        };
        let rotation = Quat::from_rotation_y(PI / 4.0 - corner as f32 * PI / 2.0)
            * Quat::from_rotation_x(PI / 2.0);
        sector.rotate_by(rotation);
        sector.translate_by(vec);
        left.merge(&sector).unwrap();
    }
    left
}
