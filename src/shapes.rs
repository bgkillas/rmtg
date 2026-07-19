use crate::assets::Asset;
use crate::physics::{bounce, physics};
use crate::{CARD_THICKNESS, WORLD_FONT_SIZE};
use avian3d::parry::glamx::Quat;
use bevy::asset::RenderAssetUsages;
use bevy::color::{Color, Srgba};
use bevy::ecs::children;
use bevy::material::AlphaMode;
use bevy::math::{Vec2, Vec3};
use bevy::mesh::{Indices, Mesh, Mesh3d, MeshBuilder, PrimitiveTopology};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Bundle, EntityCommands, InheritedVisibility, Transform};
use bevy_polyline::material::{PolylineMaterial, PolylineMaterialHandle};
use bevy_polyline::polyline::{Polyline, PolylineHandle};
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAnchor, TextAtlas};
pub mod cube;
pub mod dodecahedron;
pub mod icosahedron;
pub mod octahedron;
pub mod tetrahedron;
fn average_normalized<const N: usize>(elems: [[f32; 3]; N]) -> Vec3 {
    elems.map(Vec3::from).into_iter().sum::<Vec3>().normalize()
}
fn face<const N: usize>(elems: [Vec3; N], rev: bool) -> Transform {
    let pos = elems.into_iter().sum::<Vec3>() / N as f32;
    let end = if N.is_multiple_of(2) {
        (elems[0] + elems[1]) / 2.0
    } else {
        elems[0]
    };
    let (n, l) = pos.normalize_and_length();
    let pos_epsilon = n * if rev {
        l - CARD_THICKNESS
    } else {
        l + CARD_THICKNESS
    };
    Transform::from_translation(pos_epsilon).looking_to(if rev { pos } else { -pos }, end - pos)
}
pub trait NewShape {
    fn from_height(height: f32) -> Self;
}
pub trait ShapeMesh: NewShape + MeshBuilder + Sized + Copy {
    type Outline: ShapeOutline;
    type const VERTICES: usize;
    type const FACES: usize;
    type const FACE_VERTICES: usize;
    type const TRIANGLES: usize;
    const IS_REVERSED: bool = false;
    #[must_use]
    fn bundle(
        height: f32,
        base_color: Color,
        outline_color: Color,
        asset: &mut Asset,
    ) -> impl Bundle {
        let mesh = Mesh::from(Self::from_height(height));
        (
            physics(&mesh),
            Mesh3d(asset.meshes.add(mesh)),
            MeshMaterial3d(asset.materials.add(StandardMaterial {
                base_color,
                ..StandardMaterial::default()
            })),
            children![(
                PolylineHandle(asset.polylines.add(Self::Outline::from_height(height))),
                PolylineMaterialHandle(asset.polyline_materials.add(PolylineMaterial {
                    width: 16.0 * height,
                    color: outline_color.to_linear(),
                    perspective: true,
                    depth_bias: Self::Outline::DEPTH_BIAS,
                })),
            )],
            InheritedVisibility::VISIBLE,
        )
    }
    fn insert_dice(
        height: f32,
        base_color: Color,
        outline_color: Color,
        asset: &mut Asset,
        mut ent: EntityCommands<'_>,
    ) {
        ent.insert((
            Self::bundle(height, base_color, outline_color, asset),
            bounce(),
        ));
        ent.with_children(|parent| {
            for (i, t) in Self::faces(height).into_iter().enumerate() {
                parent.spawn((
                    t,
                    Text3d::new((i + 1).to_string()),
                    Mesh3d::default(),
                    MeshMaterial3d(asset.materials.add(StandardMaterial {
                        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..StandardMaterial::default()
                    })),
                    Text3dStyling {
                        size: WORLD_FONT_SIZE,
                        anchor: TextAnchor::CENTER,
                        color: Srgba::BLACK,
                        world_scale: Some(Vec2::splat(0.5)),
                        ..Text3dStyling::default()
                    },
                ));
            }
        });
    }
    #[must_use]
    fn faces(height: f32) -> [Transform; Self::FACES] {
        let v = Self::oriented_vertices(Self::convert_height(height)).map(Vec3::from);
        Self::face_indices()
            .map(|l| l.map(|i| v[usize::from(i)]))
            .map(|vec| face(vec, Self::IS_REVERSED))
    }
    #[must_use]
    fn convert_height(height: f32) -> f32;
    #[must_use]
    fn face_indices() -> [[u16; Self::FACE_VERTICES]; Self::FACES];
    #[must_use]
    fn vertices(one: f32) -> [[f32; 3]; Self::VERTICES];
    #[must_use]
    fn convert_to_triangles(face: [u16; Self::FACE_VERTICES]) -> [[u16; 3]; Self::TRIANGLES];
    #[must_use]
    fn oriented_vertices(one: f32) -> [[f32; 3]; Self::VERTICES] {
        let vertices = Self::vertices(one);
        let dir = Quat::from_rotation_arc(
            average_normalized(Self::face_indices()[0].map(|i| vertices[usize::from(i)])),
            -Vec3::Y,
        );
        vertices.map(|p| dir * Vec3::from(p)).map(|v| v.to_array())
    }
    #[must_use]
    fn unit_length(self) -> f32;
    #[must_use]
    fn mesh(self) -> Mesh {
        let position = Self::oriented_vertices(self.unit_length()).to_vec();
        let indices = Indices::U16(
            Self::face_indices()
                .map(|v| Self::convert_to_triangles(v))
                .as_flattened()
                .as_flattened()
                .to_vec(),
        );
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, position);
        mesh.insert_indices(indices);
        mesh.compute_normals();
        mesh
    }
}
pub trait ShapeOutline: NewShape + Into<Polyline> {
    type Mesh: ShapeMesh;
    const DEPTH_BIAS: f32 = -1.0 / 65536.0;
}
