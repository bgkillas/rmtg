use crate::shapes::average_normalized;
use bevy::asset::RenderAssetUsages;
use bevy::math::{Quat, Vec3};
use bevy::mesh::{Indices, Mesh, MeshBuilder, Meshable, PrimitiveTopology};
use bevy_polyline::polyline::Polyline;
pub struct Octahedron {
    pub unit_length: f32,
}
impl Octahedron {
    #[must_use]
    pub fn from_length(length: f32) -> Self {
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
fn pos(unit_length: f32) -> [[f32; 3]; 6] {
    let one = unit_length;
    let position_pre: [[f32; 3]; _] = [
        [one, 0.0, 0.0],
        [0.0, one, 0.0],
        [0.0, 0.0, one],
        [-one, 0.0, 0.0],
        [0.0, -one, 0.0],
        [0.0, 0.0, -one],
    ];
    let dir = Quat::from_rotation_arc(
        average_normalized(position_pre[0], position_pre[1], position_pre[2]),
        -Vec3::Y,
    );
    position_pre
        .map(|p| dir * Vec3::new(p[0], p[1], p[2]))
        .map(|v| [v.x, v.y, v.z])
}
impl MeshBuilder for OctahedronMeshBuilder {
    fn build(&self) -> Mesh {
        let position = pos(self.unit_length).to_vec();
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
pub struct OctahedronOutline {
    pub unit_length: f32,
}
impl OctahedronOutline {
    #[must_use]
    pub fn from_length(length: f32) -> Self {
        Self {
            unit_length: length / 2.0f32.sqrt(),
        }
    }
}
impl OctahedronOutline {
    #[must_use]
    pub fn build(&self) -> Polyline {
        let position = pos(self.unit_length);
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
