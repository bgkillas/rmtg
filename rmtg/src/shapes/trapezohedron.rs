use crate::CARD_THICKNESS;
use crate::shapes::{NewShape, Shape, ShapeMesh, ShapeOutline};
use avian3d::parry::glamx::Vec3;
use bevy::math::Dir3;
use bevy::mesh::{Mesh, MeshBuilder};
use bevy::prelude::Transform;
use core::direct_const_arg;
use std::f32::consts::GOLDEN_RATIO;
#[derive(Clone, Copy)]
pub struct Trapezohedron {
    pub unit_length: f32,
}
impl ShapeMesh for Trapezohedron {
    type Outline = TrapezohedronOutline;
    type const VERTICES: usize = 12;
    type const FACES: usize = 10;
    type const FACE_VERTICES: usize = 4;
    type const TRIANGLES: usize = 2;
    const SHAPE: Shape = Shape::Trapezohedron;
    fn text_size(height: f32) -> f32 {
        height / 4.0
    }
    fn face(old: [Vec3; direct_const_arg!(Self::FACE_VERTICES)], rev: bool) -> Transform {
        let elems = [old[0], old[1], old[3]];
        let pos = elems.into_iter().sum::<Vec3>() / elems.len() as f32;
        let norm =
            Dir3::try_from((elems[1] - elems[0]).cross(elems[2] - elems[0]).normalize()).unwrap();
        let end = elems[0];
        let (n, l) = pos.normalize_and_length();
        let pos_epsilon = n * if rev {
            l - CARD_THICKNESS / 64.0
        } else {
            l + CARD_THICKNESS / 64.0
        };
        let mut trans = Transform::from_translation(pos_epsilon).looking_to(-norm, end - pos);
        trans.translation += trans.down() * CARD_THICKNESS * 5.0;
        trans
    }
    fn convert_height(height: f32) -> f32 {
        height * ((25.0f32 + 11.0f32 * 5.0f32.sqrt()) / 10.0f32).sqrt()
            / 8.0
            / (5.0f32.sqrt() - 1.0)
    }
    fn face_indices() -> [[u16; 4]; 10] {
        [
            [8, 2, 6, 11],
            [8, 11, 7, 3],
            [8, 4, 0, 2],
            [8, 3, 1, 5],
            [8, 5, 10, 4],
            [9, 7, 11, 6],
            [9, 6, 2, 0],
            [9, 1, 3, 7],
            [9, 0, 4, 10],
            [9, 10, 5, 1],
        ]
    }
    fn vertices(self) -> [[f32; 3]; 12] {
        const POLE_HEIGHT: f32 = 2.0 / 3.0;
        let one = self.unit_length();
        let vc1 = GOLDEN_RATIO * one;
        let vc2 = vc1 + one;
        let vc0 = vc1 - one;
        [
            [0.0, vc0, vc1],
            [0.0, vc0, -vc1],
            [0.0, -vc0, vc1],
            [0.0, -vc0, -vc1],
            [one, one, one],
            [one, one, -one],
            [-one, -one, one],
            [-one, -one, -one],
            [vc2 * POLE_HEIGHT, -vc1 * POLE_HEIGHT, 0.0],
            [-vc2 * POLE_HEIGHT, vc1 * POLE_HEIGHT, 0.0],
            [vc0, vc1, 0.0],
            [-vc0, -vc1, 0.0],
        ]
    }
    fn convert_to_triangles(face: [u16; Self::FACE_VERTICES]) -> [[u16; 3]; Self::TRIANGLES] {
        [[0, 1, 3], [1, 2, 3]].map(|a| a.map(|i| face[i]))
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
}
impl ShapeOutline for TrapezohedronOutline {
    type Mesh = Trapezohedron;
    type const EDGES: usize = 20;
    fn edge_indices() -> [[usize; 2]; Self::EDGES] {
        [
            [8, 2],
            [2, 6],
            [6, 11],
            [11, 8],
            [11, 7],
            [7, 3],
            [3, 8],
            [8, 5],
            [5, 10],
            [10, 4],
            [4, 8],
            [4, 0],
            [0, 2],
            [9, 0],
            [10, 9],
            [5, 1],
            [1, 9],
            [1, 3],
            [7, 9],
            [6, 9],
        ]
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
}
impl NewShape for Trapezohedron {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: Self::convert_height(height),
        }
    }
}
impl NewShape for TrapezohedronOutline {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: <Self as ShapeOutline>::Mesh::convert_height(height),
        }
    }
}
impl MeshBuilder for Trapezohedron {
    fn build(&self) -> Mesh {
        self.mesh()
    }
}
#[derive(Clone, Copy)]
pub struct TrapezohedronOutline {
    pub unit_length: f32,
}
impl MeshBuilder for TrapezohedronOutline {
    fn build(&self) -> Mesh {
        self.mesh()
    }
}
impl From<TrapezohedronOutline> for Trapezohedron {
    fn from(value: TrapezohedronOutline) -> Self {
        Self {
            unit_length: value.unit_length,
        }
    }
}
impl From<Trapezohedron> for TrapezohedronOutline {
    fn from(value: Trapezohedron) -> Self {
        Self {
            unit_length: value.unit_length,
        }
    }
}
