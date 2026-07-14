use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, MeshBuilder, Meshable, PrimitiveTopology};
use std::f32::consts::GOLDEN_RATIO;
pub struct Icosahedron {
    pub half_length: f32,
}
impl Icosahedron {
    #[must_use]
    pub fn new(length: f32) -> Self {
        Self {
            half_length: length / 2.0,
        }
    }
}
pub struct IcosahedronMeshBuilder {
    pub half_length: f32,
}
impl Meshable for Icosahedron {
    type Output = IcosahedronMeshBuilder;
    fn mesh(&self) -> Self::Output {
        IcosahedronMeshBuilder {
            half_length: self.half_length,
        }
    }
}
impl MeshBuilder for IcosahedronMeshBuilder {
    fn build(&self) -> Mesh {
        let grt = GOLDEN_RATIO * self.half_length;
        let one = self.half_length;
        let position: Vec<[f32; 3]> = vec![
            [one, grt, 0.0],
            [0.0, one, grt],
            [grt, 0.0, one],
            [one, -grt, 0.0],
            [0.0, one, -grt],
            [-grt, 0.0, one],
            [-one, grt, 0.0],
            [0.0, -one, grt],
            [grt, 0.0, -one],
            [-one, -grt, 0.0],
            [0.0, -one, -grt],
            [-grt, 0.0, -one],
        ];
        #[rustfmt::skip]
        let indices = Indices::U32(vec![
            0,  1,  2,  0,  6,  1,
            8,  0,  2,  8,  4,  0,
            3,  8,  2,  3,  2,  7,
            7,  2,  1,  0,  4,  6,
            4, 11,  6,  6, 11,  5,
            1,  5,  7,  4, 10, 11,
            4,  8, 10, 10,  8,  3,
           10,  3,  9, 11, 10,  9,
           11,  9,  5,  5,  9,  7,
            9,  3,  7,  1,  6,  5,
        ]);
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, position)
        .with_inserted_indices(indices);
        mesh.compute_normals();
        mesh
    }
}
impl From<Icosahedron> for Mesh {
    fn from(ico: Icosahedron) -> Self {
        ico.mesh().build()
    }
}
