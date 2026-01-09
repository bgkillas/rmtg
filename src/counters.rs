use crate::shapes::{Shape, WORLD_FONT_SIZE};
use crate::{
    ANG_DAMPING, CARD_HEIGHT, CARD_THICKNESS, CARD_WIDTH, Card, GRAVITY, LIN_DAMPING, SLEEP,
};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAnchor, TextAtlas};
use bitcode::{Decode, Encode};
use enum_map::{Enum, enum_map};
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
    cmds.with_child((
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
    cmds
}
#[derive(Encode, Decode, Debug, Clone, PartialEq, Deref, DerefMut)]
pub struct Value(pub i128);
pub fn spawn_modify(
    ent: Entity,
    card: &Card,
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
    children: &Children,
    counters: &Query<(), With<Counter>>,
) {
    for c in children {
        if counters.contains(*c) {
            commands.entity(*c).despawn();
        }
    }
    let set = enum_map! {
        Counter::Power=>card.power.clone(),
        Counter::Toughness=>card.toughness.clone(),
        Counter::Loyalty=>card.loyalty.clone(),
        Counter::Misc=>card.misc.clone(),
        Counter::Counters=>card.counters.clone(),
    };
    commands.entity(ent).with_children(|p| {
        for (counter, value) in set {
            let Some(value) = value else { continue };
            let width = 24.0 * CARD_THICKNESS;
            let n = match counter {
                Counter::Power => 2,
                Counter::Toughness => 1,
                Counter::Loyalty => 0,
                Counter::Counters => 1,
                Counter::Misc => 0,
            };
            p.spawn((
                Transform::from_xyz(
                    (CARD_WIDTH - width) / 2.0 - width * n as f32,
                    0.0,
                    (CARD_HEIGHT + width) / 2.0,
                ),
                counter,
                Mesh3d(meshes.add(Cuboid::new(width, CARD_THICKNESS, width))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::BLACK,
                    unlit: true,
                    ..default()
                })),
                InheritedVisibility::default(),
            ))
            .with_child((
                Transform::from_xyz(0.0, CARD_THICKNESS / 2.0 + CARD_THICKNESS / 16.0, 0.0)
                    .looking_at(Vec3::default(), Dir3::NEG_Z),
                Text3d::new(value.to_string()),
                Mesh3d(meshes.add(Rectangle::new(width, width))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color_texture: Some(TextAtlas::DEFAULT_IMAGE),
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
                Text3dStyling {
                    size: WORLD_FONT_SIZE,
                    world_scale: Some(Vec2::splat(width / 2.0)),
                    anchor: TextAnchor::CENTER,
                    ..default()
                },
                InheritedVisibility::default(),
            ));
        }
    });
}
#[derive(Component, Enum)]
pub enum Counter {
    Power,
    Toughness,
    Loyalty,
    Counters,
    Misc,
}
