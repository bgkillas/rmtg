use crate::shapes::{NewShape, Shape, ShapeMesh, ShapeOutline};
use bevy::mesh::{Mesh, MeshBuilder};
#[derive(Clone, Copy)]
pub struct Octahedron {
    pub unit_length: f32,
}
impl ShapeMesh for Octahedron {
    type Outline = OctahedronOutline;
    type const VERTICES: usize = 6;
    type const FACES: usize = 8;
    const SHAPE: Shape = Shape::Octahedron;
    fn text_size(height: f32) -> f32 {
        height / 1.5
    }
    fn convert_height(height: f32) -> f32 {
        height / (3.0f32 / 2.0f32).sqrt()
    }
    fn face_indices() -> [[u16; 3]; 8] {
        [
            [0, 1, 2],
            [0, 2, 4],
            [0, 5, 1],
            [0, 4, 5],
            [3, 2, 1],
            [3, 4, 2],
            [3, 1, 5],
            [3, 5, 4],
        ]
    }
    fn vertices(one: f32) -> [[f32; 3]; 6] {
        [
            [one, 0.0, 0.0],
            [0.0, one, 0.0],
            [0.0, 0.0, one],
            [-one, 0.0, 0.0],
            [0.0, -one, 0.0],
            [0.0, 0.0, -one],
        ]
    }
    fn convert_to_triangles(face: [u16; Self::FACE_VERTICES]) -> [[u16; 3]; Self::TRIANGLES] {
        [face]
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
}
impl ShapeOutline for OctahedronOutline {
    type Mesh = Octahedron;
    type const EDGES: usize = 12;
    fn edge_indices() -> [[usize; 2]; Self::EDGES] {
        [
            [0, 1],
            [1, 2],
            [2, 0],
            [2, 4],
            [4, 0],
            [1, 5],
            [5, 0],
            [4, 5],
            [2, 3],
            [3, 1],
            [3, 4],
            [3, 5],
        ]
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
}
impl NewShape for Octahedron {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: Self::convert_height(height),
        }
    }
}
impl NewShape for OctahedronOutline {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: <Self as ShapeOutline>::Mesh::convert_height(height),
        }
    }
}
impl MeshBuilder for Octahedron {
    fn build(&self) -> Mesh {
        self.mesh()
    }
}
#[derive(Clone, Copy)]
pub struct OctahedronOutline {
    pub unit_length: f32,
}
impl MeshBuilder for OctahedronOutline {
    fn build(&self) -> Mesh {
        self.mesh()
    }
}
