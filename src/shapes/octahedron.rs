use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, MeshBuilder, Meshable, PrimitiveTopology};
pub struct Octahedron {
    pub unit_length: f32,
}
impl Octahedron {
    #[must_use]
    pub fn new(length: f32) -> Self {
        Self {
            unit_length: length / 2.0f32.sqrt(),
        }
    }
}
pub struct OctahedronMeshBuilder {
    pub unit_length: f32,
}
impl Meshable for Octahedron {
    type Output = OctahedronMeshBuilder;
    fn mesh(&self) -> Self::Output {
        OctahedronMeshBuilder {
            unit_length: self.unit_length,
        }
    }
}
impl MeshBuilder for OctahedronMeshBuilder {
    fn build(&self) -> Mesh {
        let one = self.unit_length;
        let position: Vec<[f32; 3]> = vec![
            [one, 0.0, 0.0],
            [0.0, one, 0.0],
            [0.0, 0.0, one],
            [-one, 0.0, 0.0],
            [0.0, -one, 0.0],
            [0.0, 0.0, -one],
        ];
        #[rustfmt::skip]
        let indices = Indices::U32(vec![
            0, 1, 2,
            0, 2, 4,
            0, 5, 1,
            0, 4, 5,
            3, 2, 1,
            3, 4, 2,
            3, 1, 5,
            3, 5, 4,
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
impl From<Octahedron> for Mesh {
    fn from(ico: Octahedron) -> Self {
        ico.mesh().build()
    }
}
