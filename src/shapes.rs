use bevy::math::Vec3;
use bevy::mesh::Mesh;
use bevy_polyline::polyline::Polyline;
pub mod cube;
pub mod dodecahedron;
pub mod icosahedron;
pub mod octahedron;
pub mod tetrahedron;
fn average_normalized(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> Vec3 {
    (Vec3::from(a) + Vec3::from(b) + Vec3::from(c)).normalize()
}
pub trait NewShape {
    fn from_length(length: f32) -> Self;
    fn from_height(height: f32) -> Self;
}
pub trait ShapeMesh: NewShape + Into<Mesh> {
    type Outline: ShapeOutline;
}
pub trait ShapeOutline: NewShape + Into<Polyline> {
    type Mesh: ShapeMesh;
    const DEPTH_BIAS: f32 = -0.00001;
}
