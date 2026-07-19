use crate::shapes::{NewShape, ShapeMesh, ShapeOutline};
use bevy::asset::RenderAssetUsages;
use bevy::math::Vec3;
use bevy::mesh::{Indices, Mesh, MeshBuilder, PrimitiveTopology};
use bevy_polyline::polyline::Polyline;
#[derive(Clone, Copy)]
pub struct Tetrahedron {
    pub unit_length: f32,
}
impl ShapeMesh for Tetrahedron {
    type Outline = TetrahedronOutline;
    type const VERTICES: usize = 4;
    type const FACES: usize = 4;
    type const FACE: usize = 3;
    const IS_REVERSED: bool = true;
    fn convert_height(height: f32) -> f32 {
        height / (16.0f32 / 3.0f32).sqrt()
    }
    fn face_indices() -> [[u16; 3]; 4] {
        [[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]]
    }
    fn vertices(one: f32) -> [[f32; 3]; 4] {
        [
            [one, one, one],
            [one, -one, -one],
            [-one, one, -one],
            [-one, -one, one],
        ]
    }
    fn unit_length(self) -> f32 {
        self.unit_length
    }
}
impl ShapeOutline for TetrahedronOutline {
    type Mesh = Tetrahedron;
    const DEPTH_BIAS: f32 = 0.0;
}
impl NewShape for Tetrahedron {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: Self::convert_height(height),
        }
    }
}
impl NewShape for TetrahedronOutline {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: <Self as ShapeOutline>::Mesh::convert_height(height),
        }
    }
}
impl MeshBuilder for Tetrahedron {
    fn build(&self) -> Mesh {
        let position = Self::oriented_vertices(self.unit_length).to_vec();
        let indices = Indices::U16(Self::face_indices().as_flattened().to_vec());
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
pub struct TetrahedronOutline {
    pub unit_length: f32,
}
impl From<TetrahedronOutline> for Polyline {
    fn from(value: TetrahedronOutline) -> Self {
        let position = Tetrahedron::oriented_vertices(value.unit_length);
        #[rustfmt::skip]
        let ind = [
            0, 0, 0,
            1, 2, 3,
            1, 2, 3,
            2, 3, 1,
        ];
        let vertices = ind.map(|i| position[i]).map(Vec3::from).to_vec();
        Polyline { vertices }
    }
}
