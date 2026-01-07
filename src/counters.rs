use crate::shapes::{Shape, WORLD_FONT_SIZE};
use crate::{ANG_DAMPING, CARD_THICKNESS, GRAVITY, LIN_DAMPING, SLEEP};
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
    color: Color,
    player: usize,
) -> EntityCommands<'a> {
    let m = 2.0 * m;
    let s = value.to_string();
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
            base_color: color,
            unlit: true,
            ..default()
        })),
        Shape::Counter(value, player),
    ));
    cmds.with_children(|p| {
        p.spawn((
            Transform::from_xyz(0.0, m / 16.0 + CARD_THICKNESS, 0.0)
                .looking_at(Vec3::default(), Dir3::NEG_Z),
            Text3d::new(s),
            Mesh3d(meshes.add(Rectangle::new(m, m))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(TextAtlas::DEFAULT_IMAGE),
                unlit: true,
                alpha_mode: AlphaMode::Multiply,
                base_color: Color::BLACK,
                ..default()
            })),
            Text3dStyling {
                size: WORLD_FONT_SIZE,
                world_scale: Some(Vec2::splat(m / 2.0)),
                anchor: TextAnchor::CENTER,
                ..default()
            },
        ));
    });
    cmds
}
#[derive(Encode, Decode, Debug, Clone, PartialEq, Deref, DerefMut)]
pub struct Value(pub i128);
