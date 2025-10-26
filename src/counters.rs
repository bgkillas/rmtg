use crate::shapes::Shape;
use crate::{ANG_DAMPING, GRAVITY, LIN_DAMPING, SLEEP};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAnchor, TextAtlas};
use bitcode::{Decode, Encode};
pub fn make_counter<'a>(
    m: f32,
    transform: Transform,
    commands: &'a mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    value: Value,
) -> EntityCommands<'a> {
    let s = value.0.to_string();
    let mut cmds = commands.spawn((
        transform,
        Collider::cuboid(m, m / 8.0, m),
        CollisionLayers::new(0b11, LayerMask::ALL),
        RigidBody::Dynamic,
        LinearDamping(LIN_DAMPING),
        AngularDamping(ANG_DAMPING),
        SLEEP,
        GravityScale(GRAVITY),
        Mesh3d(meshes.add(Cuboid::new(m, m / 8.0, m))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        })),
        Shape::Counter(value),
    ));
    cmds.with_children(|p| {
        p.spawn((
            Transform::from_xyz(0.0, m / 16.0 + 1.0, 0.0).looking_at(Vec3::default(), Dir3::Z),
            Text3d::new(s),
            Mesh3d(meshes.add(Rectangle::new(m / 2.0, m / 2.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(TextAtlas::DEFAULT_IMAGE),
                unlit: true,
                alpha_mode: AlphaMode::Multiply,
                base_color: Color::BLACK,
                ..default()
            })),
            Text3dStyling {
                size: m / 2.0,
                anchor: TextAnchor::CENTER,
                ..default()
            },
        ));
    });
    cmds
}
#[derive(Encode, Decode, Debug, Clone)]
pub struct Value(pub i128);
