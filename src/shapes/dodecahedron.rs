use avian3d::parry::glamx::approx::AbsDiffEq;
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
        let check = |a: &[f32; 3], b: &[f32; 3]| -> bool {
            ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2))
                .abs_diff_eq(&((5.0f32.sqrt() - 1.0) * self.unit_length).powi(2), 0.00001)
        };
        let mut done = Vec::new();
        for (ai, a) in position.iter().enumerate() {
            for (bi, b) in position.iter().enumerate() {
                if a == b {
                    continue;
                }
                if !check(a, b) {
                    continue;
                }
                for (ci, c) in position.iter().enumerate() {
                    if a == c || b == c {
                        continue;
                    }
                    if !check(b, c) {
                        continue;
                    }
                    for (di, d) in position.iter().enumerate() {
                        if a == d || b == d || c == d {
                            continue;
                        }
                        if !check(c, d) {
                            continue;
                        }
                        for (ei, e) in position.iter().enumerate() {
                            if a == e || b == e || c == e || d == e {
                                continue;
                            }
                            if !check(d, e) {
                                continue;
                            }
                            if !check(e, a) {
                                continue;
                            }
                            let v = [ai, bi, ci, di, ei];
                            let mut s = v;
                            s.sort_unstable();
                            if !done.contains(&s) {
                                done.push(s);
                                println!(
                                    "{}, {}, {}, {}, {}, {}, {}, {}, {},",
                                    s[0], s[1], s[3], s[1], s[2], s[3], s[3], s[4], s[0]
                                );
                            }
                        }
                    }
                }
            }
        }
        #[rustfmt::skip]
        let indices = Indices::U32(vec![
0, 1, 9, 1, 8, 9, 0, 9, 15,
0, 2, 10, 2, 8, 10, 0, 10, 14,
0, 3, 10, 3, 9, 10, 0, 10, 16,
1, 5, 13, 5, 8, 13, 1, 13, 14,
1, 4, 15, 4, 13, 15, 1, 15, 19,
2, 6, 12, 6, 10, 12, 2, 12, 16,
2, 5, 14, 5, 12, 14, 2, 14, 18,
3, 4, 11, 4, 9, 11, 3, 11, 15,
3, 6, 16, 6, 11, 16, 3, 16, 17,
4, 7, 17, 7, 11, 17, 4, 17, 19,
5, 7, 18, 7, 13, 18, 5, 18, 19,
6, 7, 17, 7, 12, 17, 6, 17, 18,
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
