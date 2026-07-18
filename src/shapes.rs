use crate::WORLD_FONT_SIZE;
use crate::assets::Asset;
use crate::physics::{bounce, physics};
use bevy::color::Color;
use bevy::ecs::children;
use bevy::material::AlphaMode;
use bevy::math::{Vec2, Vec3};
use bevy::mesh::{Mesh, Mesh3d, MeshBuilder};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::{Bundle, EntityCommands, Rectangle, Transform};
use bevy_polyline::material::{PolylineMaterial, PolylineMaterialHandle};
use bevy_polyline::polyline::{Polyline, PolylineHandle};
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAnchor, TextAtlas};
pub mod cube;
pub mod dodecahedron;
pub mod icosahedron;
pub mod octahedron;
pub mod tetrahedron;
fn average_normalized<const N: usize>(elems: [[f32; 3]; N]) -> Vec3 {
    elems.map(Vec3::from).into_iter().sum::<Vec3>().normalize()
}
fn face<const N: usize>(elems: [[f32; 3]; N]) -> Transform {
    let vecs = elems.map(Vec3::from);
    let pos = vecs.into_iter().sum::<Vec3>() / N as f32;
    let end = if N.is_multiple_of(2) {
        (vecs[0] + vecs[1]) / 2.0
    } else {
        vecs[0]
    };
    Transform::from_translation(pos).looking_to(pos, end - pos)
}
pub trait NewShape {
    fn from_height(height: f32) -> Self;
}
pub trait ShapeMesh: NewShape + MeshBuilder + Sized {
    type Outline: ShapeOutline;
    #[must_use]
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
    fn spawn_dice(
        height: f32,
        base_color: Color,
        outline_color: Color,
        asset: &mut Asset,
        mut ent: EntityCommands<'_>,
    ) {
        ent.insert((
            Self::bundle(height, base_color, outline_color, asset),
            bounce(),
        ));
        ent.with_children(|parent| {
            for (i, t) in Self::faces(height).enumerate() {
                parent.spawn((
                    t,
                    Text3d::new((i + 1).to_string()),
                    Mesh3d(asset.meshes.add(Rectangle::new(1.0, 1.0))),
                    MeshMaterial3d(asset.materials.add(StandardMaterial {
                        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE),
                        unlit: true,
                        alpha_mode: AlphaMode::Multiply,
                        base_color: Color::BLACK,
                        ..StandardMaterial::default()
                    })),
                    Text3dStyling {
                        size: WORLD_FONT_SIZE,
                        world_scale: Some(Vec2::splat(1.0)),
                        anchor: TextAnchor::CENTER,
                        ..Text3dStyling::default()
                    },
                ));
            }
        });
    }
    #[must_use]
    fn faces(height: f32) -> impl ExactSizeIterator<Item = Transform>;
}
pub trait ShapeOutline: NewShape + Into<Polyline> {
    type Mesh: ShapeMesh;
    const DEPTH_BIAS: f32 = -0.00001;
}
