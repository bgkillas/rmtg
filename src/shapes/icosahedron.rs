use crate::shapes::{NewShape, ShapeMesh, ShapeOutline};
use avian3d::parry::glamx::Vec3;
use bevy::mesh::{Mesh, MeshBuilder};
use bevy_polyline::polyline::Polyline;
use std::f32::consts::GOLDEN_RATIO;
#[derive(Clone, Copy)]
pub struct Icosahedron {
    pub unit_length: f32,
}
impl ShapeMesh for Icosahedron {
    type Outline = IcosahedronOutline;
    type const VERTICES: usize = 12;
    type const FACES: usize = 20;
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
pub struct IcosahedronOutline {
    pub unit_length: f32,
}
impl From<IcosahedronOutline> for Polyline {
    fn from(value: IcosahedronOutline) -> Self {
        let position = Icosahedron::oriented_vertices(value.unit_length);
        #[rustfmt::skip]
        let ind = [
             0,  1,  2,  1,  6,
             2,  8,  0,  4,  2,
             3,  3,  7,  7,  4,
            11,  5, 11,  1,  5,
             4, 10,  8, 10,  3,
             9, 11,  5,  7,  6,
             1,  2,  0,  6,  0,
             8,  0,  4,  8,  3,
             8,  7,  2,  1,  6,
             4,  6,  5,  5,  7,
            10, 11, 10,  3,  9,
            10,  9,  9,  9, 11,
        ];
        let vertices = ind.map(|i| position[i]).map(Vec3::from).to_vec();
        Polyline { vertices }
    }
}
