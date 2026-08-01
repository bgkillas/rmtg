use crate::CARD_THICKNESS;
use crate::shapes::{NewShape, Shape, ShapeMesh, ShapeOutline};
use avian3d::prelude::Collider;
use bevy::math::{Dir3, Vec3};
use bevy::mesh::{CylinderMeshBuilder, Mesh, MeshBuilder, TorusMeshBuilder};
use bevy::prelude::{Torus, Transform};
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
    fn collider(height: f32, _: &Mesh) -> Collider {
        Collider::cylinder(height / 2.0, height * HEIGHT_MULT)
    }
    fn text_size(height: f32) -> f32 {
        height
    }
    fn face_string(i: usize) -> String {
        match i {
            0 => "T",
            1 => "H",
            _ => unreachable!(),
        }
        .to_owned()
    }
    fn faces(height: f32) -> [Transform; 2] {
        let one = Self::convert_height(height) * HEIGHT_MULT + CARD_THICKNESS / 2.0;
        [[0.0, -one, 0.0], [0.0, one, 0.0]]
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
    fn convert_to_triangles(_: [u16; Self::FACE_VERTICES]) -> [[u16; 3]; Self::TRIANGLES] {
        unreachable!()
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
    fn mesh(self) -> Mesh {
        Mesh::from(CylinderMeshBuilder::new(
            self.unit_length,
            self.unit_length * HEIGHT_MULT * 2.0,
            64,
        ))
    }
}
impl ShapeOutline for CoinOutline {
    type Mesh = Coin;
    type const EDGES: usize = 2;
    fn edge_indices() -> [[usize; 2]; Self::EDGES] {
        unreachable!()
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }

    fn mesh(self) -> Mesh {
        let mut mesh = Mesh::from(TorusMeshBuilder {
            torus: Torus::new(
                self.unit_length - Self::THICKNESS,
                self.unit_length + Self::THICKNESS,
            ),
            minor_resolution: 32,
            major_resolution: 64,
            ..TorusMeshBuilder::default()
        });
        let mut bot = mesh.clone();
        mesh.translate_by(Vec3::new(0.0, self.unit_length * HEIGHT_MULT, 0.0));
        bot.translate_by(Vec3::new(0.0, -self.unit_length * HEIGHT_MULT, 0.0));
        mesh.merge(&bot).unwrap();
        mesh
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
