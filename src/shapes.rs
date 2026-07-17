use bevy::math::Vec3;
pub mod cube;
pub mod dodecahedron;
pub mod icosahedron;
pub mod octahedron;
pub mod tetrahedron;
fn average_normalized(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Vec3 {
    Vec3::new(a[0] + b[0] + c[0], a[1] + b[1] + c[1], a[2] + b[2] + c[2]).normalize()
}
