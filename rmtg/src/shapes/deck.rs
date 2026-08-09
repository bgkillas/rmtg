use crate::shapes::{NewShape, Shape, ShapeMesh, ShapeOutline};
use crate::{CARD_HEIGHT, CARD_THICKNESS, CARD_WIDTH};
use bevy::math::Vec3;
use bevy::mesh::{Mesh, MeshBuilder, TorusMeshBuilder};
use bevy::prelude::Torus;
use importer::CARD_CORNER_RADIUS;
use std::f32::consts::PI;
#[derive(Clone, Copy)]
pub struct Deck {
    pub unit_length: f32,
}
impl ShapeMesh for Deck {
    type Outline = DeckOutline;
    type const VERTICES: usize = 16;
    type const FACES: usize = 6;
    type const FACE_VERTICES: usize = 4;
    type const TRIANGLES: usize = 2;
    const SHAPE: Shape = Shape::Cube;
    fn text_size(height: f32) -> f32 {
        height
    }
    fn convert_height(height: f32) -> f32 {
        height / 2.0
    }
    fn face_indices() -> [[u16; Self::FACE_VERTICES]; Self::FACES] {
        unreachable!()
    }
    fn vertices(self) -> [[f32; 3]; Self::VERTICES] {
        let one = self.unit_length();
        let wid = CARD_WIDTH / 2.0;
        let hei = CARD_HEIGHT / 2.0;
        let del = CARD_WIDTH * CARD_CORNER_RADIUS;
        [
            [wid - del, one, hei],
            [del - wid, one, hei],
            [wid - del, -one, hei],
            [wid - del, one, -hei],
            [del - wid, one, -hei],
            [del - wid, -one, hei],
            [wid - del, -one, -hei],
            [del - wid, -one, -hei],
            [wid, one, hei - del],
            [-wid, one, hei - del],
            [wid, -one, hei - del],
            [wid, one, del - hei],
            [-wid, one, del - hei],
            [-wid, -one, hei - del],
            [wid, -one, del - hei],
            [-wid, -one, del - hei],
        ]
    }
    fn convert_to_triangles(face: [u16; Self::FACE_VERTICES]) -> [[u16; 3]; Self::TRIANGLES] {
        [[0, 1, 2], [3, 2, 1]].map(|a| a.map(|i| face[i]))
    }
    fn oriented_vertices(self) -> [[f32; 3]; Self::VERTICES] {
        self.vertices()
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
}
impl ShapeOutline for DeckOutline {
    type Mesh = Deck;
    type const EDGES: usize = 8;
    const THICKNESS: f32 = CARD_THICKNESS / 3.0;
    fn edge_indices() -> [[usize; 2]; Self::EDGES] {
        [
            [0, 1],
            [6, 7],
            [3, 4],
            [2, 5],
            [8, 11],
            [10, 14],
            [9, 12],
            [13, 15],
        ]
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
}
#[derive(Clone, Copy)]
pub struct DeckOutline {
    pub unit_length: f32,
}
impl MeshBuilder for Deck {
    fn build(&self) -> Mesh {
        self.mesh()
    }
}
impl NewShape for Deck {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: Self::convert_height(height),
        }
    }
}
impl NewShape for DeckOutline {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: <Self as ShapeOutline>::Mesh::convert_height(height),
        }
    }
}
impl MeshBuilder for DeckOutline {
    fn build(&self) -> Mesh {
        let one = self.unit_length();
        let wid = CARD_WIDTH / 2.0;
        let hei = CARD_HEIGHT / 2.0;
        let del = CARD_WIDTH * CARD_CORNER_RADIUS;
        let mut mesh = self.mesh();
        for corner in 0..4 {
            let mut torus = TorusMeshBuilder {
                torus: Torus::new(del - Self::THICKNESS, del + Self::THICKNESS),
                minor_resolution: 32,
                major_resolution: 64,
                angle_range: corner as f32 * PI / 2.0..=(corner + 1) as f32 * PI / 2.0,
            }
            .build();
            let vec = match corner {
                0 => Vec3::new(wid - del, one, hei - del),
                1 => Vec3::new(del - wid, one, hei - del),
                2 => Vec3::new(del - wid, one, del - hei),
                3 => Vec3::new(wid - del, one, del - hei),
                _ => unreachable!(),
            };
            torus.translate_by(vec);
            mesh.merge(&torus).unwrap();
            torus.translate_by(Vec3::new(0.0, -2.0 * one, 0.0));
            mesh.merge(&torus).unwrap();
        }
        mesh
    }
}
impl From<DeckOutline> for Deck {
    fn from(value: DeckOutline) -> Self {
        Self {
            unit_length: value.unit_length,
        }
    }
}
impl From<Deck> for DeckOutline {
    fn from(value: Deck) -> Self {
        Self {
            unit_length: value.unit_length,
        }
    }
}
