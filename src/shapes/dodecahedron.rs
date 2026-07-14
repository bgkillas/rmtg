use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, MeshBuilder, Meshable, PrimitiveTopology};
use std::f32::consts::GOLDEN_RATIO;
pub struct Dodecahedron {
    pub length: f32,
}
impl Dodecahedron {
    #[must_use]
    pub fn new(length: f32) -> Self {
        Self { length }
    }
}
pub struct DodecahedronMeshBuilder {
    pub length: f32,
}
impl Meshable for Dodecahedron {
    type Output = DodecahedronMeshBuilder;
    fn mesh(&self) -> Self::Output {
        DodecahedronMeshBuilder {
            length: self.length,
        }
    }
}
impl MeshBuilder for DodecahedronMeshBuilder {
    fn build(&self) -> Mesh {
        let grt = GOLDEN_RATIO;
        let position: Vec<[f32; 3]> = vec![
            [1.0, grt, 0.0],
            [0.0, 1.0, grt],
            [grt, 0.0, 1.0],
            [1.0, -grt, 0.0],
            [0.0, 1.0, -grt],
            [-grt, 0.0, 1.0],
            [-1.0, grt, 0.0],
            [0.0, -1.0, grt],
            [grt, 0.0, -1.0],
            [-1.0, -grt, 0.0],
            [0.0, -1.0, -grt],
            [-grt, 0.0, -1.0],
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
impl From<Dodecahedron> for Mesh {
    fn from(dodec: Dodecahedron) -> Self {
        dodec.mesh().build()
    }
}
