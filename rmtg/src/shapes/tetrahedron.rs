use crate::CARD_THICKNESS;
use crate::shapes::{NewShape, Shape, ShapeMesh, ShapeOutline, average_normalized};
use avian3d::parry::glamx::{Quat, Vec3};
use avian3d::prelude::Collider;
use bevy::mesh::{Mesh, MeshBuilder};
use core::direct_const_arg;
#[derive(Clone, Copy)]
pub struct Tetrahedron {
    pub unit_length: f32,
}
impl ShapeMesh for Tetrahedron {
    type Outline = TetrahedronOutline;
    type const VERTICES: usize = 4;
    type const FACES: usize = 4;
    const IS_REVERSED: bool = true;
    const SHAPE: Shape = Shape::Tetrahedron;
    fn collider(height: f32, _: &Mesh) -> Collider {
        let mesh = Mesh::from(Self::from_height(height + CARD_THICKNESS * 8.0));
        Collider::convex_hull_from_mesh(&mesh).unwrap()
    }
    fn text_size(height: f32) -> f32 {
        height / 1.5
    }
    fn convert_height(height: f32) -> f32 {
        height / (16.0f32 / 3.0f32).sqrt()
    }
    fn face_indices() -> [[u16; 3]; 4] {
        [[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]]
    }
    fn vertices(one: f32) -> [[f32; 3]; 4] {
        [
            [one, one, one],
            [one, -one, -one],
            [-one, one, -one],
            [-one, -one, one],
        ]
    }
    fn convert_to_triangles(face: [u16; Self::FACE_VERTICES]) -> [[u16; 3]; Self::TRIANGLES] {
        [face]
    }
    fn oriented_vertices(one: f32) -> [[f32; 3]; direct_const_arg!(Self::VERTICES)] {
        let vertices = Self::vertices(one);
        let dir = Quat::from_rotation_arc(
            average_normalized(Self::face_indices()[3].map(|i| vertices[usize::from(i)])),
            -Vec3::Y,
        );
        vertices.map(|p| dir * Vec3::from(p)).map(|v| v.to_array())
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
}
impl ShapeOutline for TetrahedronOutline {
    type Mesh = Tetrahedron;
    type const EDGES: usize = 6;
    fn edge_indices() -> [[usize; 2]; Self::EDGES] {
        [[0, 1], [0, 2], [0, 3], [1, 2], [2, 3], [3, 1]]
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
}
impl NewShape for Tetrahedron {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: Self::convert_height(height),
        }
    }
}
impl NewShape for TetrahedronOutline {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: <Self as ShapeOutline>::Mesh::convert_height(height),
        }
    }
}
impl MeshBuilder for Tetrahedron {
    fn build(&self) -> Mesh {
        self.mesh()
    }
}
#[derive(Clone, Copy)]
pub struct TetrahedronOutline {
    pub unit_length: f32,
}
impl MeshBuilder for TetrahedronOutline {
    fn build(&self) -> Mesh {
        self.mesh()
    }
}
