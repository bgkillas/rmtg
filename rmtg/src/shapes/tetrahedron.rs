use crate::shapes::{NewShape, Shape, ShapeMesh, ShapeOutline};
use bevy::mesh::{Mesh, MeshBuilder};
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
