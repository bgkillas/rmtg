use crate::shapes::{NewShape, Shape, ShapeMesh, ShapeOutline};
use avian3d::parry::glamx::Vec3;
use bevy::mesh::{Mesh, MeshBuilder};
use bevy_polyline::polyline::Polyline;
#[derive(Clone, Copy)]
pub struct Cube {
    pub unit_length: f32,
}
impl ShapeMesh for Cube {
    type Outline = CubeOutline;
    type const VERTICES: usize = 8;
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
        [
            [0, 1, 2, 5],
            [0, 3, 1, 4],
            [0, 2, 3, 6],
            [7, 6, 5, 2],
            [7, 4, 6, 3],
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
    fn convert_to_triangles(face: [u16; Self::FACE_VERTICES]) -> [[u16; 3]; Self::TRIANGLES] {
        [[0, 1, 2], [3, 2, 1]].map(|a| a.map(|i| face[i]))
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
        self.mesh()
    }
}
impl NewShape for Cube {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: Self::convert_height(height),
        }
    }
}
impl NewShape for CubeOutline {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: <Self as ShapeOutline>::Mesh::convert_height(height),
        }
    }
}
impl From<CubeOutline> for Polyline {
    fn from(value: CubeOutline) -> Self {
        let v = Cube::oriented_vertices(value.unit_length).map(Vec3::from);
        #[rustfmt::skip]
        let ind = [
            0, 0, 0,
            7, 7, 7,
            1, 2, 3,
            4, 5, 6,
            1, 2, 3,
            4, 5, 6,
            5, 6, 4,
            1, 2, 3,
        ];
        let vertices = ind.map(|i| v[i]).to_vec();
        Polyline { vertices }
    }
}
