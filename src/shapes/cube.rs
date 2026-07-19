use crate::shapes::{NewShape, ShapeMesh, ShapeOutline, face};
use avian3d::parry::glamx::Vec3;
use bevy::mesh::{Mesh, MeshBuilder};
use bevy::prelude::{Cuboid, Transform};
use bevy_polyline::polyline::Polyline;
pub struct Cube {
    pub unit_length: f32,
}
impl ShapeMesh for Cube {
    type Outline = CubeOutline;
    fn faces(height: f32) -> impl ExactSizeIterator<Item = Transform> {
        let v = pos(to_height(height)).map(Vec3::from);
        face_indices()
            .map(|l| l.map(|i| v[usize::from(i)]))
            .map(|vec| face(vec, false))
            .into_iter()
    }
}
fn to_height(height: f32) -> f32 {
    height / 2.0
}
fn face_indices() -> [[u16; 4]; 6] {
    [
        [0, 1, 2, 5],
        [0, 1, 3, 4],
        [0, 2, 3, 6],
        [7, 6, 5, 2],
        [7, 6, 4, 3],
        [7, 5, 4, 1],
    ]
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
            unit_length: to_height(height),
        }
    }
}
fn pos(one: f32) -> [[f32; 3]; 8] {
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
impl From<CubeOutline> for Polyline {
    fn from(value: CubeOutline) -> Self {
        let v = pos(value.unit_length).map(Vec3::from);
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
