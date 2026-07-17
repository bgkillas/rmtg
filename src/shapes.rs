use crate::assets::Asset;
use crate::physics::physics;
use bevy::color::Color;
use bevy::ecs::children;
use bevy::math::Vec3;
use bevy::mesh::{Mesh, Mesh3d, MeshBuilder};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::Bundle;
use bevy_polyline::material::{PolylineMaterial, PolylineMaterialHandle};
use bevy_polyline::polyline::{Polyline, PolylineHandle};
pub mod cube;
pub mod dodecahedron;
pub mod icosahedron;
pub mod octahedron;
pub mod tetrahedron;
fn average_normalized<const N: usize>(elems: [[f32; 3]; N]) -> Vec3 {
    elems.map(Vec3::from).into_iter().sum::<Vec3>().normalize()
}
pub trait NewShape {
    fn from_height(height: f32) -> Self;
}
pub trait ShapeMesh: NewShape + MeshBuilder + Sized {
    type Outline: ShapeOutline;
    fn bundle(
        height: f32,
        base_color: Color,
        outline_color: Color,
        asset: &mut Asset,
    ) -> impl Bundle {
        let mesh = Mesh::from(Self::from_height(height));
        (
            physics(&mesh),
            Mesh3d(asset.meshes.add(mesh)),
            MeshMaterial3d(asset.materials.add(StandardMaterial {
                base_color,
                ..StandardMaterial::default()
            })),
            children![(
                PolylineHandle(asset.polylines.add(Self::Outline::from_height(height))),
                PolylineMaterialHandle(asset.polyline_materials.add(PolylineMaterial {
                    width: 16.0 * height,
                    color: outline_color.to_linear(),
                    perspective: true,
                    depth_bias: Self::Outline::DEPTH_BIAS,
                })),
            )],
        )
    }
}
pub trait ShapeOutline: NewShape + Into<Polyline> {
    type Mesh: ShapeMesh;
    const DEPTH_BIAS: f32 = -0.00001;
}
