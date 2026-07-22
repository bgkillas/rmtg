use crate::shapes::{NewShape, Shape, ShapeMesh, ShapeOutline};
use avian3d::parry::glamx::Vec3;
use bevy::mesh::{Mesh, MeshBuilder};
use bevy_polyline::polyline::Polyline;
use std::f32::consts::GOLDEN_RATIO;
#[derive(Clone, Copy)]
pub struct Dodecahedron {
    pub unit_length: f32,
}
impl ShapeMesh for Dodecahedron {
    type Outline = DodecahedronOutline;
    type const VERTICES: usize = 20;
    type const FACES: usize = 12;
    type const FACE_VERTICES: usize = 5;
    type const TRIANGLES: usize = 3;
    const SHAPE: Shape = Shape::Dodecahedron;
    fn text_size(height: f32) -> f32 {
        height / 2.0
    }
    fn convert_height(height: f32) -> f32 {
        height * ((25.0f32 + 11.0f32 * 5.0f32.sqrt()) / 10.0f32).sqrt()
            / 4.0
            / (5.0f32.sqrt() - 1.0)
    }
    fn face_indices() -> [[u16; 5]; 12] {
        [
            [15, 1, 8, 0, 9],
            [2, 10, 0, 8, 14],
            [16, 3, 9, 0, 10],
            [5, 14, 8, 1, 13],
            [4, 19, 13, 1, 15],
            [6, 16, 10, 2, 12],
            [5, 18, 12, 2, 14],
            [4, 15, 9, 3, 11],
            [6, 17, 11, 3, 16],
            [17, 7, 19, 4, 11],
            [19, 7, 18, 5, 13],
            [18, 7, 17, 6, 12],
        ]
    }
    fn vertices(one: f32) -> [[f32; 3]; 20] {
        let grt = GOLDEN_RATIO * one;
        let rgr = GOLDEN_RATIO.recip() * one;
        [
            [one, one, one],
            [-one, one, one],
            [one, -one, one],
            [one, one, -one],
            [-one, one, -one],
            [-one, -one, one],
            [one, -one, -one],
            [-one, -one, -one],
            [0.0, rgr, grt],
            [rgr, grt, 0.0],
            [grt, 0.0, rgr],
            [0.0, rgr, -grt],
            [rgr, -grt, 0.0],
            [-grt, 0.0, rgr],
            [0.0, -rgr, grt],
            [-rgr, grt, 0.0],
            [grt, 0.0, -rgr],
            [0.0, -rgr, -grt],
            [-rgr, -grt, 0.0],
            [-grt, 0.0, -rgr],
        ]
    }
    fn convert_to_triangles(face: [u16; Self::FACE_VERTICES]) -> [[u16; 3]; Self::TRIANGLES] {
        [[0, 1, 3], [1, 2, 3], [3, 4, 0]].map(|a| a.map(|i| face[i]))
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
}
impl ShapeOutline for DodecahedronOutline {
    type Mesh = Dodecahedron;
}
impl NewShape for Dodecahedron {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: Self::convert_height(height),
        }
    }
}
impl NewShape for DodecahedronOutline {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: <Self as ShapeOutline>::Mesh::convert_height(height),
        }
    }
}
impl MeshBuilder for Dodecahedron {
    fn build(&self) -> Mesh {
        self.mesh()
    }
}
pub struct DodecahedronOutline {
    pub unit_length: f32,
}
impl From<DodecahedronOutline> for Polyline {
    fn from(value: DodecahedronOutline) -> Self {
        let position = Dodecahedron::oriented_vertices(value.unit_length);
        #[rustfmt::skip]
        let ind = [
            15,  0,  9,
             8,  8, 14,
            10,  3, 16,
            10,  8, 14,
             5, 13, 13,
            19, 10, 16,
             6, 12, 12,
            18, 15, 11,
            11, 17, 11,
            19,  7,  7,
             1,  9, 15,
             0, 14,  2,
             0,  9,  3,
            16,  1,  5,
            13,  1, 19,
             4,  2,  6,
            12,  2, 18,
             5,  4,  3,
            17,  6,  4,
             7, 18, 17,
        ];
        let vertices = ind.map(|i| position[i]).map(Vec3::from).to_vec();
        Polyline { vertices }
    }
}
