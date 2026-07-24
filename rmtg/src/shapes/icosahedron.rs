use crate::shapes::{NewShape, Shape, ShapeMesh, ShapeOutline};
use bevy::mesh::{Mesh, MeshBuilder};
use std::f32::consts::GOLDEN_RATIO;
#[derive(Clone, Copy)]
pub struct Icosahedron {
    pub unit_length: f32,
}
impl ShapeMesh for Icosahedron {
    type Outline = IcosahedronOutline;
    type const VERTICES: usize = 12;
    type const FACES: usize = 20;
    const SHAPE: Shape = Shape::Icosahedron;
    fn text_size(height: f32) -> f32 {
        height / 3.0
    }
    fn convert_height(height: f32) -> f32 {
        height / (48.0f32.sqrt() / GOLDEN_RATIO.powi(2))
    }
    fn face_indices() -> [[u16; 3]; 20] {
        [
            [0, 1, 2],
            [0, 6, 1],
            [8, 0, 2],
            [8, 4, 0],
            [3, 8, 2],
            [3, 2, 7],
            [7, 2, 1],
            [0, 4, 6],
            [4, 11, 6],
            [6, 11, 5],
            [1, 5, 7],
            [4, 10, 11],
            [4, 8, 10],
            [10, 8, 3],
            [10, 3, 9],
            [11, 10, 9],
            [11, 9, 5],
            [5, 9, 7],
            [9, 3, 7],
            [1, 6, 5],
        ]
    }
    fn vertices(one: f32) -> [[f32; 3]; 12] {
        let grt = GOLDEN_RATIO * one;
        [
            [one, grt, 0.0],
            [0.0, one, grt],
            [grt, 0.0, one],
            [one, -grt, 0.0],
            [0.0, one, -grt],
            [-grt, 0.0, one],
            [-one, grt, 0.0],
            [0.0, -one, grt],
            [grt, 0.0, -one],
            [-one, -grt, 0.0],
            [0.0, -one, -grt],
            [-grt, 0.0, -one],
        ]
    }
    fn convert_to_triangles(face: [u16; Self::FACE_VERTICES]) -> [[u16; 3]; Self::TRIANGLES] {
        [face]
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
}
impl ShapeOutline for IcosahedronOutline {
    type Mesh = Icosahedron;
    type const EDGES: usize = 30;
    fn edge_indices() -> [[usize; 2]; Self::EDGES] {
        [
            [0, 1],
            [1, 2],
            [2, 0],
            [1, 6],
            [6, 0],
            [2, 8],
            [8, 0],
            [0, 4],
            [4, 8],
            [2, 3],
            [3, 8],
            [3, 7],
            [7, 2],
            [7, 1],
            [4, 6],
            [11, 4],
            [5, 6],
            [11, 5],
            [1, 5],
            [5, 7],
            [4, 10],
            [10, 11],
            [8, 10],
            [10, 3],
            [3, 9],
            [9, 10],
            [11, 9],
            [5, 9],
            [7, 9],
            [6, 11],
        ]
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
}
impl NewShape for Icosahedron {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: Self::convert_height(height),
        }
    }
}
impl NewShape for IcosahedronOutline {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: <Self as ShapeOutline>::Mesh::convert_height(height),
        }
    }
}
impl MeshBuilder for Icosahedron {
    fn build(&self) -> Mesh {
        self.mesh()
    }
}
#[derive(Clone, Copy)]
pub struct IcosahedronOutline {
    pub unit_length: f32,
}
impl MeshBuilder for IcosahedronOutline {
    fn build(&self) -> Mesh {
        self.mesh()
    }
}
