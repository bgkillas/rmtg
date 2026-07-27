use crate::CARD_THICKNESS;
use crate::shapes::{NewShape, Shape, ShapeMesh, ShapeOutline};
use bevy::asset::RenderAssetUsages;
use bevy::math::{Dir3, Vec3};
use bevy::mesh::{Mesh, MeshBuilder, PrimitiveTopology};
use bevy::prelude::{Cylinder, Transform};
#[derive(Clone, Copy)]
pub struct Coin {
    pub unit_length: f32,
}
const HEIGHT_MULT: f32 = 1.0 / 16.0;
impl ShapeMesh for Coin {
    type Outline = CoinOutline;
    type const VERTICES: usize = 8;
    type const FACES: usize = 2;
    type const FACE_VERTICES: usize = 4;
    type const TRIANGLES: usize = 2;
    const SHAPE: Shape = Shape::Coin;
    fn text_size(height: f32) -> f32 {
        height
    }
    fn faces(height: f32) -> [Transform; 2] {
        let one = Self::convert_height(height) * HEIGHT_MULT;
        [
            [0.0, one + CARD_THICKNESS / 2.0, 0.0],
            [0.0, CARD_THICKNESS / 2.0 - one, 0.0],
        ]
        .map(Vec3::from)
        .map(|v| Transform::from_translation(v).looking_to(-v, Dir3::NEG_Z))
    }
    fn convert_height(height: f32) -> f32 {
        height / 2.0
    }
    fn face_indices() -> [[u16; Self::FACE_VERTICES]; Self::FACES] {
        unreachable!()
    }
    fn vertices(_: f32) -> [[f32; 3]; Self::VERTICES] {
        unreachable!()
    }
    fn convert_to_triangles(face: [u16; Self::FACE_VERTICES]) -> [[u16; 3]; Self::TRIANGLES] {
        [[0, 1, 2], [3, 2, 1]].map(|a| a.map(|i| face[i]))
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
    fn mesh(self) -> Mesh {
        Mesh::from(Cylinder::new(
            self.unit_length,
            self.unit_length * HEIGHT_MULT * 2.0,
        ))
    }
}
impl ShapeOutline for CoinOutline {
    type Mesh = Coin;
    type const EDGES: usize = 12;
    fn edge_indices() -> [[usize; 2]; Self::EDGES] {
        [
            [0, 1],
            [0, 2],
            [0, 3],
            [7, 4],
            [7, 5],
            [7, 6],
            [1, 5],
            [2, 6],
            [3, 4],
            [4, 1],
            [5, 2],
            [6, 3],
        ]
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
    fn mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD,
        )
    }
}
#[derive(Clone, Copy)]
pub struct CoinOutline {
    pub unit_length: f32,
}
impl MeshBuilder for Coin {
    fn build(&self) -> Mesh {
        self.mesh()
    }
}
impl NewShape for Coin {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: Self::convert_height(height),
        }
    }
}
impl NewShape for CoinOutline {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: <Self as ShapeOutline>::Mesh::convert_height(height),
        }
    }
}
impl MeshBuilder for CoinOutline {
    fn build(&self) -> Mesh {
        self.mesh()
    }
}
