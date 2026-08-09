use crate::assets::Asset;
use crate::events::hover::Hoverable;
use crate::physics::{bounce, physics_base};
use crate::shapes::coin::Coin;
use crate::shapes::cube::Cube;
use crate::shapes::dodecahedron::Dodecahedron;
use crate::shapes::icosahedron::Icosahedron;
use crate::shapes::octahedron::Octahedron;
use crate::shapes::tetrahedron::Tetrahedron;
use crate::shapes::trapezohedron::Trapezohedron;
use crate::{CARD_THICKNESS, CARD_WIDTH, WORLD_FONT_SIZE};
use avian3d::parry::glamx::Quat;
use avian3d::prelude::Collider;
use bevy::asset::RenderAssetUsages;
use bevy::color::{Color, Srgba};
use bevy::ecs::children;
use bevy::math::{Dir3, Vec2, Vec3};
use bevy::mesh::{
    CylinderMeshBuilder, Indices, Mesh, Mesh3d, MeshBuilder, PrimitiveTopology, SphereKind,
    SphereMeshBuilder,
};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{
    Bundle, Component, Cylinder, EntityCommands, InheritedVisibility, Sphere, Transform,
};
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAnchor};
use core::direct_const_arg;
pub mod coin;
pub mod cube;
pub mod deck;
pub mod dodecahedron;
pub mod drag;
pub mod icosahedron;
pub mod octahedron;
pub mod tetrahedron;
pub mod trapezohedron;
pub const OUTLINE_COLOR: Color = Color::BLACK;
pub const OUTLINE_DEPTH_BIAS: f32 = 1.0 / 4096.0;
pub const OUTLINE_SUBDIVISIONS: u32 = 5;
pub const OUTLINE_RESOLUTION: u32 = 32;
#[derive(Component, Clone, Copy, Debug)]
pub enum Shape {
    Cube,
    Dodecahedron,
    Icosahedron,
    Octahedron,
    Tetrahedron,
    Trapezohedron,
    Coin,
}
impl Shape {
    #[must_use]
    pub fn faces(self) -> usize {
        match self {
            Shape::Cube => 6,
            Shape::Dodecahedron => 12,
            Shape::Icosahedron => 20,
            Shape::Octahedron => 8,
            Shape::Tetrahedron => 4,
            Shape::Trapezohedron => 10,
            Shape::Coin => 2,
        }
    }
    pub fn insert_dice<'a>(
        self,
        base_color: Color,
        outline_color: Color,
        asset: &mut Asset,
        ent: EntityCommands<'a>,
    ) -> EntityCommands<'a> {
        match self {
            Shape::Cube => Cube::insert_dice(base_color, outline_color, asset, ent),
            Shape::Dodecahedron => Dodecahedron::insert_dice(base_color, outline_color, asset, ent),
            Shape::Icosahedron => Icosahedron::insert_dice(base_color, outline_color, asset, ent),
            Shape::Octahedron => Octahedron::insert_dice(base_color, outline_color, asset, ent),
            Shape::Tetrahedron => Tetrahedron::insert_dice(base_color, outline_color, asset, ent),
            Shape::Trapezohedron => {
                Trapezohedron::insert_dice(base_color, outline_color, asset, ent)
            }
            Shape::Coin => Coin::insert_dice(base_color, outline_color, asset, ent),
        }
    }
}
#[derive(Component, Clone)]
pub struct FaceNumber {
    pub num: u8,
}
fn average_normalized<const N: usize>(elems: [[f32; 3]; N]) -> Vec3 {
    elems.map(Vec3::from).into_iter().sum::<Vec3>().normalize()
}
pub trait NewShape: MeshBuilder + Sized + Copy {
    fn from_height(height: f32) -> Self;
}
pub trait ShapeMesh: NewShape + From<Self::Outline>
where
    Self::Outline: From<Self>,
{
    type Outline: ShapeOutline;
    type const VERTICES: usize;
    type const FACES: usize;
    type const FACE_VERTICES: usize = 3;
    type const TRIANGLES: usize = 1;
    const IS_REVERSED: bool = false;
    const HEIGHT: f32 = CARD_WIDTH / 2.0;
    const SHAPE: Shape;
    #[must_use]
    fn bundle(
        height: f32,
        base_color: Color,
        outline_color: Color,
        asset: &mut Asset,
    ) -> impl Bundle {
        let mesh = Mesh::from(Self::from_height(height));
        (
            Self::SHAPE,
            Self::collider(height, &mesh),
            physics_base(),
            Mesh3d(asset.meshes.add(mesh)),
            MeshMaterial3d(asset.materials.add(StandardMaterial {
                base_color,
                unlit: true,
                ..StandardMaterial::default()
            })),
            children![(
                Mesh3d(asset.meshes.add(Self::Outline::from_height(height))),
                MeshMaterial3d(asset.materials.add(StandardMaterial {
                    base_color: outline_color,
                    unlit: true,
                    depth_bias: OUTLINE_DEPTH_BIAS,
                    ..StandardMaterial::default()
                })),
            )],
            InheritedVisibility::VISIBLE,
        )
    }
    #[must_use]
    fn collider(_: f32, mesh: &Mesh) -> Collider {
        Collider::convex_hull_from_mesh(mesh).unwrap()
    }
    #[must_use]
    fn face_string(i: usize) -> String {
        (i + 1).to_string()
    }
    fn insert_dice<'a>(
        base_color: Color,
        outline_color: Color,
        asset: &mut Asset,
        mut ent: EntityCommands<'a>,
    ) -> EntityCommands<'a> {
        let height = Self::HEIGHT;
        ent.insert((
            Self::bundle(height, base_color, outline_color, asset),
            bounce(),
            Hoverable,
        ));
        ent.with_children(|parent| {
            for (i, t) in Self::from_height(height).faces().into_iter().enumerate() {
                parent.spawn((
                    t,
                    Text3d::new(Self::face_string(i)),
                    Mesh3d::default(),
                    MeshMaterial3d(asset.text()),
                    Text3dStyling {
                        size: WORLD_FONT_SIZE,
                        anchor: TextAnchor::CENTER,
                        color: Srgba::BLACK,
                        world_scale: Some(Vec2::splat(Self::text_size(height))),
                        ..Text3dStyling::default()
                    },
                    FaceNumber {
                        num: u8::try_from(i + 1).unwrap(),
                    },
                ));
            }
        });
        ent
    }
    #[must_use]
    fn text_size(height: f32) -> f32;
    #[must_use]
    fn faces(self) -> [Transform; direct_const_arg!(Self::FACES)] {
        let v = self.oriented_vertices().map(Vec3::from);
        Self::face_indices()
            .map(|l| l.map(|i| v[usize::from(i)]))
            .map(|vec| Self::face(vec, Self::IS_REVERSED))
    }
    #[must_use]
    fn face(elems: [Vec3; direct_const_arg!(Self::FACE_VERTICES)], rev: bool) -> Transform {
        let pos = elems.into_iter().sum::<Vec3>() / Self::FACE_VERTICES as f32;
        let norm =
            Dir3::try_from((elems[1] - elems[0]).cross(elems[2] - elems[0]).normalize()).unwrap();
        let end = if Self::FACE_VERTICES.is_multiple_of(2) {
            (elems[0] + elems[1]) / 2.0
        } else {
            elems[0]
        };
        let (n, l) = pos.normalize_and_length();
        let pos_epsilon = n * if rev {
            l - CARD_THICKNESS / 64.0
        } else {
            l + CARD_THICKNESS / 64.0
        };
        Transform::from_translation(pos_epsilon).looking_to(-norm, end - pos)
    }
    #[must_use]
    fn convert_height(height: f32) -> f32;
    #[must_use]
    fn face_indices()
    -> [[u16; direct_const_arg!(Self::FACE_VERTICES)]; direct_const_arg!(Self::FACES)];
    #[must_use]
    fn vertices(self) -> [[f32; 3]; direct_const_arg!(Self::VERTICES)];
    #[must_use]
    fn convert_to_triangles(
        face: [u16; direct_const_arg!(Self::FACE_VERTICES)],
    ) -> [[u16; 3]; direct_const_arg!(Self::TRIANGLES)];
    #[must_use]
    fn oriented_vertices(self) -> [[f32; 3]; direct_const_arg!(Self::VERTICES)] {
        let vertices = self.vertices();
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
        let position = self.oriented_vertices().to_vec();
        let indices = Indices::U16(
            Self::face_indices()
                .map(|v| Self::convert_to_triangles(v))
                .as_flattened()
                .as_flattened()
                .to_vec(),
        );
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, position);
        mesh.insert_indices(indices);
        mesh
    }
}
pub trait ShapeOutline: NewShape + From<Self::Mesh>
where
    Self::Mesh: From<Self>,
{
    type Mesh: ShapeMesh;
    type const EDGES: usize;
    const THICKNESS: f32 = CARD_THICKNESS * 7.0 / 8.0;
    #[must_use]
    fn edge_indices() -> [[usize; 2]; direct_const_arg!(Self::EDGES)];
    #[must_use]
    fn unit_length(self) -> f32;
    #[must_use]
    fn edges(self) -> [[Vec3; 2]; direct_const_arg!(Self::EDGES)] {
        let position = Self::Mesh::from(self).oriented_vertices().map(Vec3::from);
        let edges = Self::edge_indices();
        edges.map(|[a, b]| [position[a], position[b]])
    }
    fn position(
        self,
    ) -> [Vec3; direct_const_arg!(<<Self as ShapeOutline>::Mesh as ShapeMesh>::VERTICES)] {
        Self::Mesh::from(self).oriented_vertices().map(Vec3::from)
    }
    #[must_use]
    fn mesh(self) -> Mesh {
        let position = self.position();
        let edges_computed = self.edges();
        let mut mesh = Mesh::from(CylinderMeshBuilder {
            cylinder: Cylinder::new(
                Self::THICKNESS,
                (edges_computed[0][0] - edges_computed[0][1]).length(),
            ),
            resolution: OUTLINE_RESOLUTION,
            ..CylinderMeshBuilder::default()
        });
        let sphere = Mesh::from(SphereMeshBuilder {
            sphere: Sphere::new(Self::THICKNESS),
            kind: SphereKind::Ico {
                subdivisions: OUTLINE_SUBDIVISIONS,
            },
        });
        mesh.rotate_by(Quat::from_rotation_arc(
            Vec3::Y,
            (edges_computed[0][1] - edges_computed[0][0]).normalize(),
        ));
        mesh.translate_by((edges_computed[0][0] + edges_computed[0][1]) / 2.0);
        for [a, b] in &edges_computed[1..] {
            let height = (a - b).length();
            let mut line = Mesh::from(CylinderMeshBuilder {
                cylinder: Cylinder::new(Self::THICKNESS, height),
                resolution: OUTLINE_RESOLUTION,
                ..CylinderMeshBuilder::default()
            });
            line.rotate_by(Quat::from_rotation_arc(Vec3::Y, (b - a).normalize()));
            line.translate_by((a + b) / 2.0);
            mesh.merge(&line).unwrap();
        }
        for v in position {
            let mut dot = sphere.clone();
            dot.translate_by(v);
            mesh.merge(&dot).unwrap();
        }
        mesh
    }
}
