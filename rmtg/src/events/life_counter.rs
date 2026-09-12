use crate::assets::AssetManager;
use crate::events::select_drag::SelectableObject;
use crate::mat::{MAT_DELTA_X, MAT_DELTA_Z};
use crate::net::Peer;
use crate::{CARD_THICKNESS, WORLD_FONT_SIZE};
use avian3d::parry::glamx::Vec2;
use avian3d::prelude::{Collider, RigidBody};
use bevy::color::Srgba;
use bevy::math::Vec3;
use bevy::mesh::Mesh3d;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::Transform;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::children;
use bevy_ecs::component::Component;
use bevy_ecs::system::Commands;
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAnchor};
#[derive(Component)]
pub struct LifeCounter {
    pub life: u64,
}
#[derive(Component)]
pub struct CombatDamageText;
impl Default for LifeCounter {
    fn default() -> Self {
        Self { life: 40 }
    }
}
impl LifeCounter {
    pub fn bundle(peer: Peer, assets: &AssetManager) -> impl Bundle {
        let (rev_x, rev_z) = match peer {
            Peer::Zero => (false, false),
            Peer::One => (true, false),
            Peer::Two => (true, true),
            Peer::Three => (false, true),
        };
        let mut x = MAT_DELTA_X / 2.0;
        let mut z = MAT_DELTA_Z / 2.0;
        if rev_x {
            x = -x;
        }
        if rev_z {
            z = -z;
        }
        (
            peer,
            Self::default(),
            Mesh3d(assets.meshes.square.clone()),
            MeshMaterial3d(assets.outlines.players[peer].clone()),
            Collider::cuboid(1.0, 1.0, CARD_THICKNESS / 64.0),
            RigidBody::Static,
            SelectableObject,
            Transform::from_translation(Vec3::new(x, 0.0, z))
                .with_scale(Vec3::splat(MAT_DELTA_X))
                .looking_to(Vec3::NEG_Y, if rev_z { Vec3::Z } else { Vec3::NEG_Z }),
            children![
                (
                    Transform::from_xyz(0.0, 0.0, CARD_THICKNESS / 64.0),
                    Text3d::new(40.to_string()),
                    Mesh3d::default(),
                    MeshMaterial3d(assets.text_mesh.mesh.clone()),
                    Text3dStyling {
                        size: WORLD_FONT_SIZE,
                        anchor: TextAnchor::CENTER,
                        color: Srgba::BLACK,
                        world_scale: Some(Vec2::splat(0.5)),
                        ..Text3dStyling::default()
                    },
                ),
                (
                    peer,
                    CombatDamageText,
                    Transform::from_xyz(0.0, -0.35, CARD_THICKNESS / 64.0),
                    Text3d::new((-20).to_string()),
                    Mesh3d::default(),
                    MeshMaterial3d(assets.text_mesh.mesh.clone()),
                    Text3dStyling {
                        size: WORLD_FONT_SIZE,
                        anchor: TextAnchor::CENTER,
                        color: Srgba::BLACK,
                        world_scale: Some(Vec2::splat(0.25)),
                        ..Text3dStyling::default()
                    },
                )
            ],
        )
    }
}
pub fn startup_life_counters(mut commands: Commands, assets: AssetManager) {
    commands.spawn(LifeCounter::bundle(Peer::Zero, &assets));
    commands.spawn(LifeCounter::bundle(Peer::One, &assets));
    commands.spawn(LifeCounter::bundle(Peer::Two, &assets));
    commands.spawn(LifeCounter::bundle(Peer::Three, &assets));
}
