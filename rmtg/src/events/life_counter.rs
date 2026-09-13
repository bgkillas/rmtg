use crate::assets::AssetManager;
use crate::events::select_drag::{SelectDragTarget, SelectableObject};
use crate::mat::{MAT_DELTA_X, MAT_DELTA_Z};
use crate::net::Peer;
use crate::pile::Pile;
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
use bevy_ecs::entity::{Entity, EntityHashMap, EntityHashSet};
use bevy_ecs::event::Event;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::observer::On;
use bevy_ecs::query::{With, Without};
use bevy_ecs::resource::Resource;
use bevy_ecs::system::{Commands, Query, Res, ResMut};
use bevy_query_fn_macro::query_fn;
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAnchor};
use enum_map::EnumMap;
use importer::card::SubCard;
use importer::combat_damage::CombatData;
#[derive(Component)]
pub struct LifeCounter {
    pub life: u32,
}
#[derive(Event)]
pub struct NewLifeCount {
    pub peer: Peer,
    pub life: u32,
}
#[derive(Event)]
pub struct NewExpectedDamage {
    pub peer: Peer,
}
#[derive(Resource, Default)]
pub struct ExpectedDamage {
    pub peers: EnumMap<Peer, CombatState>,
}
#[derive(Default)]
pub struct CombatState {
    pub state: EntityHashMap<EntityHashSet>,
}
impl CombatState {
    pub fn sum<'a>(&self, get: impl Fn(Entity) -> &'a SubCard) -> CombatData {
        let mut data = CombatData::default();
        for (&attacker, defenders) in &self.state {
            let list = defenders.iter().map(|&e| get(e)).collect::<Vec<_>>();
            data = data + CombatData::get(get(attacker), &list).unwrap();
        }
        data
    }
}
#[query_fn]
pub fn update_expected_damage(
    counters: Query<(&Peer, &Children), With<LifeCounter>>,
    children: Query<&Children, Without<LifeCounter>>,
    mut expected: ResMut<ExpectedDamage>,
    piles: Query<&Pile>,
    targets: Query<&SelectDragTarget>,
    mut commands: Commands,
) {
    for counter in counters {
        let state = &mut expected.peers[*counter.peer];
        let was_empty = state.state.is_empty();
        state.state.clear();
        for child in counter.children {
            if let Ok(target) = targets.get(*child)
                && let Ok(pile) = piles.get(target.source)
                && pile.first().can_be_in_combat()
            {
                let mut set = EntityHashSet::new();
                for card_child in children.get(target.source).unwrap() {
                    if let Ok(card_target) = targets.get(*card_child)
                        && let Ok(blocker_pile) = piles.get(card_target.source)
                        && blocker_pile.first().can_be_in_combat()
                    {
                        set.insert(card_target.source);
                    }
                }
                state.state.insert(target.source, set);
            }
        }
        if !state.state.is_empty() || !was_empty {
            commands.trigger(NewExpectedDamage {
                peer: *counter.peer,
            });
        }
    }
}
#[query_fn]
pub fn on_expected_damage(
    event: On<NewExpectedDamage>,
    counters: Query<(&Peer, &Children), With<LifeCounter>>,
    mut texts: Query<&mut Text3d>,
    expected: Res<ExpectedDamage>,
    piles: Query<&Pile>,
) {
    let counter = counters.iter().find(|c| *c.peer == event.peer).unwrap();
    let mut text = texts.get_mut(counter.children[1]).unwrap();
    let data = expected.peers[event.peer].sum(|e| piles.get(e).unwrap().first());
    *text = Text3d::new(data.damage.to_string());
}
#[query_fn]
pub fn update_lifetotal(
    on: On<NewLifeCount>,
    mut counters: Query<(&mut LifeCounter, &Peer, &Children)>,
    mut texts: Query<&mut Text3d>,
) {
    let mut counter = counters.iter_mut().find(|c| *c.peer == on.peer).unwrap();
    counter.life_counter.life = on.life;
    let mut text = texts.get_mut(counter.children[0]).unwrap();
    *text = Text3d::new(on.life.to_string());
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
                    Text3d::new(0.to_string()),
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
