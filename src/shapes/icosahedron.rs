use crate::shapes::{NewShape, ShapeMesh, ShapeOutline, average_normalized};
use avian3d::parry::glamx::{Quat, Vec3};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, MeshBuilder, PrimitiveTopology};
use bevy::prelude::Transform;
use bevy_polyline::polyline::Polyline;
use std::f32::consts::GOLDEN_RATIO;
pub struct Icosahedron {
    pub unit_length: f32,
}
impl ShapeMesh for Icosahedron {
    type Outline = IcosahedronOutline;
    fn faces(height: f32) -> impl ExactSizeIterator<Item = Transform> {
        [].into_iter()
    }
}
impl ShapeOutline for IcosahedronOutline {
    type Mesh = Icosahedron;
}
impl NewShape for Icosahedron {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: height / (48.0f32.sqrt() / GOLDEN_RATIO.powi(2)),
        }
    }
}
impl NewShape for IcosahedronOutline {
    fn from_height(height: f32) -> Self {
        Self {
            unit_length: height / (48.0f32.sqrt() / GOLDEN_RATIO.powi(2)),
        }
    }
}
fn pos(unit_length: f32) -> [[f32; 3]; 12] {
    let grt = GOLDEN_RATIO * unit_length;
    let one = unit_length;
    let position_pre: [[f32; 3]; _] = [
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
    let dir = Quat::from_rotation_arc(
        average_normalized([position_pre[0], position_pre[1], position_pre[2]]),
        -Vec3::Y,
    );
    position_pre
        .map(|p| dir * Vec3::new(p[0], p[1], p[2]))
        .map(|v| [v.x, v.y, v.z])
}
impl MeshBuilder for Icosahedron {
    fn build(&self) -> Mesh {
        let position = pos(self.unit_length).to_vec();
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
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, position);
        mesh.insert_indices(indices);
        mesh.compute_normals();
        mesh
    }
}
pub struct IcosahedronOutline {
    pub unit_length: f32,
}
impl From<IcosahedronOutline> for Polyline {
    fn from(value: IcosahedronOutline) -> Self {
        let position = pos(value.unit_length);
        #[rustfmt::skip]
        let ind = [
             0,  1,  2,  1,  6,
             2,  8,  0,  4,  2,
             3,  3,  7,  7,  4,
            11,  5, 11,  1,  5,
             4, 10,  8, 10,  3,
             9, 11,  5,  7,  6,
             1,  2,  0,  6,  0,
             8,  0,  4,  8,  3,
             8,  7,  2,  1,  6,
             4,  6,  5,  5,  7,
            10, 11, 10,  3,  9,
            10,  9,  9,  9, 11,
        ];
        let vertices = ind.map(|i| position[i]).map(Vec3::from).to_vec();
        Polyline { vertices }
    }
}
