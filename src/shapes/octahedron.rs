use crate::shapes::{NewShape, ShapeMesh, ShapeOutline};
use bevy::math::Vec3;
use bevy::mesh::{Mesh, MeshBuilder};
use bevy_polyline::polyline::Polyline;
#[derive(Clone, Copy)]
pub struct Octahedron {
    pub unit_length: f32,
}
impl ShapeMesh for Octahedron {
    type Outline = OctahedronOutline;
    type const VERTICES: usize = 6;
    type const FACES: usize = 8;
    type const FACE_VERTICES: usize = 3;
    type const TRIANGLES: usize = 1;
    fn convert_height(height: f32) -> f32 {
        height / (6.0f32 / 4.0f32).sqrt()
    }
    fn face_indices() -> [[u16; 3]; 8] {
        [
            [0, 1, 2],
            [0, 2, 4],
            [0, 5, 1],
            [0, 4, 5],
            [3, 2, 1],
            [3, 4, 2],
            [3, 1, 5],
            [3, 5, 4],
        ]
    }
    fn vertices(one: f32) -> [[f32; 3]; 6] {
        [
            [one, 0.0, 0.0],
            [0.0, one, 0.0],
            [0.0, 0.0, one],
            [-one, 0.0, 0.0],
            [0.0, -one, 0.0],
            [0.0, 0.0, -one],
        ]
    }
    fn convert_to_triangles(face: [u16; Self::FACE_VERTICES]) -> [[u16; 3]; Self::TRIANGLES] {
        [face]
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
}
impl ShapeOutline for OctahedronOutline {
    type Mesh = Octahedron;
}
impl NewShape for Octahedron {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: Self::convert_height(height),
        }
    }
}
impl NewShape for OctahedronOutline {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: <Self as ShapeOutline>::Mesh::convert_height(height),
        }
    }
}
impl MeshBuilder for Octahedron {
    fn build(&self) -> Mesh {
        self.mesh()
    }
}
pub struct OctahedronOutline {
    pub unit_length: f32,
}
impl From<OctahedronOutline> for Polyline {
    fn from(value: OctahedronOutline) -> Self {
        let position = Octahedron::oriented_vertices(value.unit_length);
        #[rustfmt::skip]
        let ind = [
            0, 1, 2, 2,
            4, 1, 5, 4,
            2, 3, 3, 3,
            1, 2, 0, 4,
            0, 5, 0, 5,
            3, 1, 4, 5,
        ];
        let vertices = ind.map(|i| position[i]).map(Vec3::from).to_vec();
        Polyline { vertices }
    }
}
