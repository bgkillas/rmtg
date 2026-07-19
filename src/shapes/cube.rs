use crate::shapes::{NewShape, ShapeMesh, ShapeOutline};
use avian3d::parry::glamx::Vec3;
use bevy::mesh::{Mesh, MeshBuilder};
use bevy::prelude::Cuboid;
use bevy_polyline::polyline::Polyline;
#[derive(Clone, Copy)]
pub struct Cube {
    pub unit_length: f32,
}
impl ShapeMesh for Cube {
    type Outline = CubeOutline;
    type const VERTICES: usize = 8;
    type const FACES: usize = 6;
    type const FACE: usize = 4;
    fn convert_height(height: f32) -> f32 {
        height / 2.0
    }
    fn face_indices() -> [[u16; Self::FACE]; Self::FACES] {
        [
            [0, 1, 2, 5],
            [0, 1, 3, 4],
            [0, 2, 3, 6],
            [7, 6, 5, 2],
            [7, 6, 4, 3],
            [7, 5, 4, 1],
        ]
    }
    fn vertices(one: f32) -> [[f32; 3]; Self::VERTICES] {
        [
            [one, one, one],
            [-one, one, one],
            [one, -one, one],
            [one, one, -one],
            [-one, one, -one],
            [-one, -one, one],
            [one, -one, -one],
            [-one, -one, -one],
        ]
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
}
impl ShapeOutline for CubeOutline {
    type Mesh = Cube;
}
pub struct CubeOutline {
    pub unit_length: f32,
}
impl MeshBuilder for Cube {
    fn build(&self) -> Mesh {
        Mesh::from(Cuboid::from_length(self.unit_length))
    }
}
impl NewShape for Cube {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: height,
        }
    }
}
impl NewShape for CubeOutline {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: Cube::convert_height(height),
        }
    }
}
impl From<CubeOutline> for Polyline {
    fn from(value: CubeOutline) -> Self {
        let v = Cube::oriented_vertices(value.unit_length).map(Vec3::from);
        #[rustfmt::skip]
        let vertices = vec![
            v[0], v[0], v[0],
            v[7], v[7], v[7],
            v[1], v[2], v[3],
            v[4], v[5], v[6],
            v[1], v[2], v[3],
            v[4], v[5], v[6],
            v[5], v[6], v[4],
            v[1], v[2], v[3],
        ];
        Polyline { vertices }
    }
}
