use crate::shapes::average_normalized;
use avian3d::parry::glamx::{Quat, Vec3};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, Mesh, MeshBuilder, Meshable, PrimitiveTopology};
use bevy_polyline::polyline::Polyline;
use std::f32::consts::GOLDEN_RATIO;
pub struct Icosahedron {
    pub unit_length: f32,
}
impl Icosahedron {
    #[must_use]
    pub fn new(length: f32) -> Self {
        Self {
            unit_length: length / 2.0,
        }
    }
}
pub struct IcosahedronMeshBuilder {
    pub unit_length: f32,
}
impl Meshable for Icosahedron {
    type Output = IcosahedronMeshBuilder;
    fn mesh(&self) -> Self::Output {
        IcosahedronMeshBuilder {
            unit_length: self.unit_length,
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
        average_normalized(position_pre[0], position_pre[1], position_pre[2]),
        -Vec3::Y,
    );
    position_pre
        .map(|p| dir * Vec3::new(p[0], p[1], p[2]))
        .map(|v| [v.x, v.y, v.z])
}
impl MeshBuilder for IcosahedronMeshBuilder {
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
impl From<Icosahedron> for Mesh {
    fn from(ico: Icosahedron) -> Self {
        ico.mesh().build()
    }
}
pub struct IcosahedronOutline {
    pub unit_length: f32,
}
impl IcosahedronOutline {
    #[must_use]
    pub fn new(length: f32) -> Self {
        Self {
            unit_length: length / 2.0,
        }
    }
}
impl IcosahedronOutline {
    #[must_use]
    pub fn build(&self) -> Polyline {
        let mesh = Icosahedron {
            unit_length: self.unit_length,
        }
        .mesh()
        .build();
        let mut vertices = Vec::new();
        let pos =
            Vec::<[f32; 3]>::try_from(mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap().clone())
                .unwrap();
        let mut indices = mesh.indices().unwrap().iter();
        let mut used = Vec::new();
        let mut end = Vec::new();
        let mut i = 0;
        let mut ind = Vec::new();
        let mut ind_end = Vec::new();
        while let Some(x) = indices.next()
            && let Some(y) = indices.next()
            && let Some(z) = indices.next()
        {
            let mut arr = [x, y, z];
            arr.sort_unstable();
            let [a, b, c] = arr;
            if !used.contains(&(a, b)) {
                used.push((a, b));
                vertices.push(Vec3::from(pos[a]));
                end.push(Vec3::from(pos[b]));
                ind.push(i);
                ind_end.push(i + 1);
            }
            if !used.contains(&(b, c)) {
                used.push((b, c));
                vertices.push(Vec3::from(pos[b]));
                end.push(Vec3::from(pos[c]));
                ind.push(i + 1);
                ind_end.push(i + 2);
            }
            if !used.contains(&(c, a)) {
                used.push((c, a));
                vertices.push(Vec3::from(pos[c]));
                end.push(Vec3::from(pos[a]));
                ind.push(i + 2);
                ind_end.push(i);
            }
            i += 3;
        }
        ind.extend(ind_end);
        println!("{ind:?}");
        vertices.extend(end);
        Polyline { vertices }
    }
}
