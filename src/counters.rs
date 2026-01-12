use crate::shapes::{Shape, WORLD_FONT_SIZE};
use crate::sync::{Net, SyncObject, SyncObjectMe};
use crate::{
    ANG_DAMPING, CARD_HEIGHT, CARD_THICKNESS, CARD_WIDTH, Card, GRAVITY, Keybind, Keybinds,
    LIN_DAMPING, Pile, SLEEP,
};
use avian3d::prelude::*;
use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::picking::backend::PointerHits;
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
    commands.spawn((
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
        children![(
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
        )],
    ))
}
#[derive(Encode, Decode, Debug, Clone, PartialEq, Deref, DerefMut)]
pub struct Value(pub i128);
#[allow(unused_variables)]
pub fn modify_view(card: &Card, parent: &mut RelatedSpawnerCommands<ChildOf>, font: Handle<Font>) {
    let set = enum_map! {
        Counter::Power=>card.power.clone(),
        Counter::Toughness=>card.toughness.clone(),
        Counter::Loyalty=>card.loyalty.clone(),
        Counter::Misc=>card.misc.clone(),
        Counter::Counters=>card.counters.clone(),
    };
    for (counter, value) in set {
        let Some(value) = value else { continue };
        let width = 24.0 * CARD_THICKNESS;
        let n = match counter {
            Counter::Power => 2.0,
            Counter::Toughness => 1.0,
            Counter::Loyalty => 0.0,
            Counter::Counters => 1.5,
            Counter::Misc => 0.0,
        };
        let is_misc = matches!(counter, Counter::Misc);
        //TODO
    }
}
pub fn del_modify(
    children: &Children,
    commands: &mut Commands,
    query: Query<&Counter>,
    counter: Counter,
) {
    for ent in children {
        if query.get(*ent).is_ok_and(|c| *c == counter) {
            commands.entity(*ent).despawn();
        }
    }
}
pub fn spawn_all_modify(
    ent: Entity,
    card: &Card,
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
) {
    for counter in [
        Counter::Power,
        Counter::Toughness,
        Counter::Loyalty,
        Counter::Misc,
        Counter::Counters,
    ] {
        spawn_modify(ent, card, commands, materials, meshes, counter)
    }
}
//TODO can be done without del_modify
//TODO can spawn/del optionally depending if some=>none, etc
pub fn modify(
    ent: Entity,
    card: &Card,
    children: &Children,
    commands: &mut Commands,
    query: Query<&Counter>,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
    counter: Counter,
) {
    del_modify(children, commands, query, counter);
    spawn_modify(ent, card, commands, materials, meshes, counter);
}
pub fn spawn_modify(
    ent: Entity,
    card: &Card,
    commands: &mut Commands,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
    counter: Counter,
) {
    let value = match counter {
        Counter::Power => card.power.clone(),
        Counter::Toughness => card.toughness.clone(),
        Counter::Loyalty => card.loyalty.clone(),
        Counter::Misc => card.misc.clone(),
        Counter::Counters => card.counters.clone(),
    };
    let Some(value) = value else { return };
    let width = 24.0 * CARD_THICKNESS;
    let n = match counter {
        Counter::Power => 2.0,
        Counter::Toughness => 1.0,
        Counter::Loyalty => 0.0,
        Counter::Counters => 1.5,
        Counter::Misc => 0.0,
    };
    let is_misc = matches!(counter, Counter::Misc);
    let (vec, size) = if is_misc {
        (
            Vec3::new(0.0, 0.0, -CARD_HEIGHT / 6.0),
            Vec3::new(CARD_WIDTH, CARD_THICKNESS / 2.0, CARD_HEIGHT / 3.0),
        )
    } else {
        (
            Vec3::new(
                (CARD_WIDTH - width) / 2.0 - width * n,
                0.0,
                (CARD_HEIGHT + width) / 2.0
                    + if matches!(counter, Counter::Counters) {
                        width
                    } else {
                        0.0
                    },
            ),
            Vec3::new(width, CARD_THICKNESS, width),
        )
    };
    commands.entity(ent).with_child((
        Transform::from_translation(vec),
        counter,
        Pickable::default(),
        Mesh3d(meshes.add(Cuboid::from_size(size))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            ..default()
        })),
        InheritedVisibility::default(),
        children![(
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
                world_scale: Some(Vec2::splat(if is_misc {
                    CARD_WIDTH / 2.0
                } else {
                    2.0 * width / 3.0
                })),
                anchor: TextAnchor::CENTER,
                ..default()
            },
            InheritedVisibility::default(),
        )],
    ));
}
pub fn counter_hit(
    mut hits: MessageReader<PointerHits>,
    children: Query<(&Children, &ChildOf, &Counter)>,
    mut card: Query<&mut Pile>,
    mut text: Query<&mut Text3d>,
    keybinds: Keybinds,
    net: Net,
    ids: Query<&SyncObjectMe>,
    others_ids: Query<&SyncObject>,
) {
    let add = keybinds.just_pressed(Keybind::Add);
    let sub = keybinds.just_pressed(Keybind::Sub);
    if !add && !sub {
        hits.clear();
        return;
    }
    for hit in hits.read() {
        for (hit, _) in &hit.picks {
            let Ok((child, parent, counter)) = children.get(*hit) else {
                continue;
            };
            let Ok(card) = card.get_mut(parent.0) else {
                continue;
            };
            let Pile::Single(card) = card.into_inner() else {
                continue;
            };
            let Ok(mut text) = text.get_mut(child[0]) else {
                continue;
            };
            let obj = match counter {
                Counter::Power => card.power.as_mut().unwrap(),
                Counter::Toughness => card.toughness.as_mut().unwrap(),
                Counter::Loyalty => card.loyalty.as_mut().unwrap(),
                Counter::Counters => card.counters.as_mut().unwrap(),
                Counter::Misc => card.misc.as_mut().unwrap(),
            };
            if add {
                obj.0 += 1;
            } else if sub {
                obj.0 -= 1;
            }
            *text.get_single_mut().unwrap() = obj.0.to_string();
            if let Ok(id) = ids.get(parent.0) {
                net.modify_me(*id, *counter, Some(obj.clone()));
            } else if let Ok(id) = others_ids.get(parent.0) {
                net.modify(*id, *counter, Some(obj.clone()));
            }
        }
    }
}
#[derive(Component, Enum, Debug, Encode, Decode, Clone, Copy, PartialEq)]
pub enum Counter {
    Power,
    Toughness,
    Loyalty,
    Counters,
    Misc,
}
