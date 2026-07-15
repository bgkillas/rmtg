use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, MeshBuilder, Meshable, PrimitiveTopology};
use std::f32::consts::GOLDEN_RATIO;
pub struct Dodecahedron {
    pub unit_length: f32,
}
impl Dodecahedron {
    #[must_use]
    pub fn new(length: f32) -> Self {
        Self {
            unit_length: length / (5.0f32.sqrt() - 1.0),
        }
    }
}
pub struct DodecahedronMeshBuilder {
    pub unit_length: f32,
}
impl Meshable for Dodecahedron {
    type Output = DodecahedronMeshBuilder;
    fn mesh(&self) -> Self::Output {
        DodecahedronMeshBuilder {
            unit_length: self.unit_length,
        }
    }
}
impl MeshBuilder for DodecahedronMeshBuilder {
    fn build(&self) -> Mesh {
        let grt = GOLDEN_RATIO * self.unit_length;
        let rgr = GOLDEN_RATIO.recip() * self.unit_length;
        let one = self.unit_length;
        let position: Vec<[f32; 3]> = vec![
            [one, one, one],
            [-one, one, one],
            [one, -one, one],
            [one, one, -one],
            [-one, one, -one],
            [-one, -one, one],
            [one, -one, -one],
            [-one, -one, -one],
            [0.0, rgr, grt],
            [rgr, grt, 0.0],
            [grt, 0.0, rgr],
            [0.0, rgr, -grt],
            [rgr, -grt, 0.0],
            [-grt, 0.0, rgr],
            [0.0, -rgr, grt],
            [-rgr, grt, 0.0],
            [grt, 0.0, -rgr],
            [0.0, -rgr, -grt],
            [-rgr, -grt, 0.0],
            [-grt, 0.0, -rgr],
        ];
        #[rustfmt::skip]
        let indices = Indices::U32(vec![
            0, 15,  8,  8, 15,  1, 15,  0,  9,
            0,  8,  2,  8, 14,  2,  2, 10,  0,
            0, 16,  9,  9, 16,  3, 16,  0, 10,
            1,  5,  8,  8,  5, 14,  5,  1, 13,
            1,  4, 13, 13,  4, 19,  4,  1, 15,
            2,  6, 10, 10,  6, 16,  6,  2, 12,
            2,  5, 12, 12,  5, 18,  5,  2, 14,
            3,  4,  9,  9,  4, 15,  4,  3, 11,
            3,  6, 11, 11,  6, 17,  6,  3, 16,
            4, 11,  7, 11, 17,  7,  7, 19,  4,
            5, 13,  7, 13, 19,  7,  7, 18,  5,
            6, 12,  7, 12, 18,  7,  7, 17,  6,
        ]);
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
impl From<Dodecahedron> for Mesh {
    fn from(dodec: Dodecahedron) -> Self {
        dodec.mesh().build()
    }
}
