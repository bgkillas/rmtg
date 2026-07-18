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
        let one = height / 2.0;
        let v = pos(one);
        [face([v[0], v[1], v[2], v[5]])].into_iter()
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
            unit_length: height / 2.0,
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
