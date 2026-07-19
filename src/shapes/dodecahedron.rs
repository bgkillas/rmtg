use crate::shapes::{NewShape, ShapeMesh, ShapeOutline};
use avian3d::parry::glamx::Vec3;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, MeshBuilder, PrimitiveTopology};
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
    type const FACE: usize = 5;
    fn convert_height(height: f32) -> f32 {
        height * ((25.0f32 + 11.0f32 * 5.0f32.sqrt()) / 10.0f32).sqrt()
            / 4.0
            / (5.0f32.sqrt() - 1.0)
    }
    //TODO
    fn face_indices() -> [[u16; 5]; 12] {
        [
            [0, 15, 8, 1, 9],
            [0, 8, 2, 14, 10],
            [0, 16, 9, 3, 10],
            [1, 5, 8, 14, 13],
            [1, 4, 13, 19, 15],
            [2, 6, 10, 16, 12],
            [2, 5, 12, 18, 14],
            [3, 4, 9, 15, 11],
            [3, 6, 11, 17, 16],
            [4, 11, 7, 17, 19],
            [5, 13, 19, 7, 5],
            [6, 12, 18, 7, 6],
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
        let position = Self::oriented_vertices(self.unit_length).to_vec();
        /*TODO let indices = Indices::U16(
            face_indices()
                .map(|v| [v[0], v[3], v[1], v[1], v[3], v[2], v[3], v[0], v[4]])
                .as_flattened()
                .to_vec(),
        );*/
        #[rustfmt::skip]
        let indices = Indices::U32(vec![
            0, 15,  8,  8, 15,  1, 15,  0,  9,
            0,  8,  2,  8, 14,  2,  2, 10,  0,
            0, 16,  9,  9, 16,  3, 16,  0, 10,
            1,  5,  8,  8,  5, 14,  5,  1, 13,
            1,  4, 13, 13,  4, 19,  4,  1, 15,
            2,  6, 10, 10,  6, 16,  6,  2, 12,
            2,  5, 12, 12,  5, 18,  5,  2, 14,
            3,  4,  9,  9,  4, 15,  4,  3, 11,
            3,  6, 11, 11,  6, 17,  6,  3, 16,
            4, 11,  7, 11, 17,  7,  7, 19,  4,
            5, 13,  7, 13, 19,  7,  7, 18,  5,
            6, 12,  7, 12, 18,  7,  7, 17,  6,
        ]);
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
