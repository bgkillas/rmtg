use crate::shapes::{NewShape, ShapeMesh, ShapeOutline, average_normalized, face};
use bevy::asset::RenderAssetUsages;
use bevy::math::{Quat, Vec3};
use bevy::mesh::{Indices, Mesh, MeshBuilder, PrimitiveTopology};
use bevy::prelude::Transform;
use bevy_polyline::polyline::Polyline;
pub struct Tetrahedron {
    pub unit_length: f32,
}
impl NewShape for Tetrahedron {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: to_height(height),
        }
    }
}
impl NewShape for TetrahedronOutline {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: to_height(height),
        }
    }
}
fn to_height(height: f32) -> f32 {
    height / (16.0f32 / 3.0f32).sqrt()
}
fn face_indices() -> [[u16; 3]; 4] {
    [[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]]
}
impl ShapeMesh for Tetrahedron {
    type Outline = TetrahedronOutline;
    fn faces(height: f32) -> impl ExactSizeIterator<Item = Transform> {
        let v = pos(to_height(height)).map(Vec3::from);
        face_indices()
            .map(|l| l.map(|i| v[usize::from(i)]))
            .map(|vec| face(vec, true))
            .into_iter()
    }
}
impl ShapeOutline for TetrahedronOutline {
    type Mesh = Tetrahedron;
    const DEPTH_BIAS: f32 = 0.0;
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
        average_normalized([position_pre[0], position_pre[1], position_pre[2]]),
        -Vec3::Y,
    );
    position_pre
        .map(|p| dir * Vec3::new(p[0], p[1], p[2]))
        .map(|v| [v.x, v.y, v.z])
}
impl MeshBuilder for Tetrahedron {
    fn build(&self) -> Mesh {
        let position = pos(self.unit_length).to_vec();
        let indices = Indices::U16(face_indices().as_flattened().to_vec());
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
        let position = pos(value.unit_length);
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
