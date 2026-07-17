use crate::shapes::average_normalized;
use bevy::asset::RenderAssetUsages;
use bevy::math::{Quat, Vec3};
use bevy::mesh::{Indices, Mesh, MeshBuilder, Meshable, PrimitiveTopology};
use bevy_polyline::polyline::Polyline;
pub struct Tetrahedron {
    pub unit_length: f32,
}
impl Tetrahedron {
    #[must_use]
    pub fn new(length: f32) -> Self {
        Self {
            unit_length: length / 8.0f32.sqrt(),
        }
    }
}
pub struct TetrahedronMeshBuilder {
    pub unit_length: f32,
}
impl Meshable for Tetrahedron {
    type Output = TetrahedronMeshBuilder;
    fn mesh(&self) -> Self::Output {
        TetrahedronMeshBuilder {
            unit_length: self.unit_length,
        }
    }
}
fn pos(unit_length: f32) -> [[f32; 3]; 4] {
    let one = unit_length;
    let position_pre: [[f32; 3]; _] = [
        [one, one, one],
        [one, -one, -one],
        [-one, one, -one],
        [-one, -one, one],
    ];
    let dir = Quat::from_rotation_arc(
        average_normalized(position_pre[0], position_pre[1], position_pre[2]),
        -Vec3::Y,
    );
    position_pre
        .map(|p| dir * Vec3::new(p[0], p[1], p[2]))
        .map(|v| [v.x, v.y, v.z])
}
impl MeshBuilder for TetrahedronMeshBuilder {
    fn build(&self) -> Mesh {
        let position = pos(self.unit_length).to_vec();
        #[rustfmt::skip]
        let indices = Indices::U32(vec![
            0, 2, 1,
            0, 1, 3,
            0, 3, 2,
            1, 2, 3,
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
impl From<Tetrahedron> for Mesh {
    fn from(ico: Tetrahedron) -> Self {
        ico.mesh().build()
    }
}
pub struct TetrahedronOutline {
    pub unit_length: f32,
}
impl TetrahedronOutline {
    #[must_use]
    pub fn new(length: f32) -> Self {
        Self {
            unit_length: length / 8.0f32.sqrt(),
        }
    }
}
impl TetrahedronOutline {
    #[must_use]
    pub fn build(&self) -> Polyline {
        let position = pos(self.unit_length);
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
