use crate::assets::AssetManager;
use crate::events::hover::NoBoxSelect;
use crate::events::select_drag::{IgnoreSelectableObject, SelectDragTarget, SelectableObject};
use crate::keybinds::Keybind;
use crate::mat::{MAT_DELTA_X, MAT_DELTA_Z};
use crate::net::Peer;
use crate::pile::Pile;
use crate::spatial::Spatial;
use crate::{CARD_THICKNESS, WORLD_FONT_SIZE};
use avian3d::parry::glamx::Vec2;
use avian3d::prelude::{Collider, RigidBody};
use bevy::color::Srgba;
use bevy::input::ButtonInput;
use bevy::math::Vec3;
use bevy::mesh::Mesh3d;
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::Transform;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::entity::{Entity, EntityHashMap, EntityHashSet};
use bevy_ecs::event::Event;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::observer::On;
use bevy_ecs::query::{With, Without};
use bevy_ecs::relationship::RelatedSpawner;
use bevy_ecs::resource::Resource;
use bevy_ecs::spawn::{Spawn, SpawnRelated as _, SpawnWith};
use bevy_ecs::system::{Commands, Query, Res, ResMut};
use bevy_query_fn_macro::query_fn;
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAnchor};
use enum_map::{Enum, EnumMap};
use enumset::EnumSet;
use importer::card::SubCard;
use importer::combat_damage::CombatData;
#[derive(Enum, Clone, Copy, PartialEq)]
pub enum CommanderCounter {
    None,
    First(Peer),
    Second(Peer),
}
impl CommanderCounter {
    pub fn peer(self) -> Option<Peer> {
        Some(match self {
            CommanderCounter::None => return None,
            CommanderCounter::First(peer) | CommanderCounter::Second(peer) => peer,
        })
    }
}
#[derive(Component)]
pub struct LifeCounter {
    pub life: i32,
    pub commander: CommanderCounter,
}
#[derive(Event)]
pub struct NewLifeCount {
    pub peer: Peer,
    pub commander: CommanderCounter,
    pub life: i32,
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
            let mut list = defenders.iter().map(|&e| get(e)).collect::<Vec<_>>();
            data = data + CombatData::get(get(attacker), &mut list).unwrap();
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
    counters: Query<(&Peer, &Children, &LifeCounter)>,
    mut texts: Query<(&mut CombatDamageText, &mut Text3d)>,
    expected: Res<ExpectedDamage>,
    piles: Query<&Pile>,
) {
    let counter = counters
        .iter()
        .find(|c| *c.peer == event.peer && c.life_counter.commander.peer().is_none())
        .unwrap();
    let mut text = texts.get_mut(counter.children[1]).unwrap();
    let data = expected.peers[event.peer].sum(|e| piles.get(e).unwrap().first());
    text.combat_damage_text.delta = -data.damage;
    *text.text3d = Text3d::new((-data.damage).to_string());
}
#[query_fn]
pub fn update_lifetotal(
    on: On<NewLifeCount>,
    mut counters: Query<(&mut LifeCounter, &Peer, &Children)>,
    mut texts: Query<&mut Text3d>,
) {
    let mut counter = counters
        .iter_mut()
        .find(|c| *c.peer == on.peer && c.life_counter.commander == on.commander)
        .unwrap();
    counter.life_counter.life = on.life;
    let mut text = texts.get_mut(counter.children[0]).unwrap();
    *text = Text3d::new(on.life.to_string());
}
#[derive(Component, Default)]
pub struct CombatDamageText {
    pub delta: i32,
}
impl LifeCounter {
    pub fn bundle(peer: Peer, commander: CommanderCounter, assets: &AssetManager) -> impl Bundle {
        let (rev_x, rev_z) = match peer {
            Peer::Zero => (false, false),
            Peer::One => (true, false),
            Peer::Two => (true, true),
            Peer::Three => (false, true),
        };
        let (mut x, mut z) = match commander {
            CommanderCounter::None => (MAT_DELTA_X / 2.0, MAT_DELTA_Z / 2.0),
            CommanderCounter::First(Peer::Zero) => {
                (7.0 * MAT_DELTA_X / 4.0, 3.0 * MAT_DELTA_Z / 4.0)
            }
            CommanderCounter::First(Peer::One) => (7.0 * MAT_DELTA_X / 4.0, MAT_DELTA_Z / 4.0),
            CommanderCounter::First(Peer::Two) => (5.0 * MAT_DELTA_X / 4.0, MAT_DELTA_Z / 4.0),
            CommanderCounter::First(Peer::Three) => {
                (5.0 * MAT_DELTA_X / 4.0, 3.0 * MAT_DELTA_Z / 4.0)
            }
            CommanderCounter::Second(Peer::Zero) => {
                (3.0 * MAT_DELTA_Z / 4.0, 7.0 * MAT_DELTA_X / 4.0)
            }
            CommanderCounter::Second(Peer::One) => (MAT_DELTA_Z / 4.0, 7.0 * MAT_DELTA_X / 4.0),
            CommanderCounter::Second(Peer::Two) => (MAT_DELTA_Z / 4.0, 5.0 * MAT_DELTA_X / 4.0),
            CommanderCounter::Second(Peer::Three) => {
                (3.0 * MAT_DELTA_Z / 4.0, 5.0 * MAT_DELTA_X / 4.0)
            }
        };
        if rev_x {
            x = -x;
        }
        if rev_z {
            z = -z;
        }
        let mult = if matches!(commander, CommanderCounter::None) {
            1.0
        } else {
            0.5
        };
        let mesh = assets.text_mesh.mesh.clone();
        let life = if commander.peer().is_some() { 0 } else { 40 };
        (
            peer,
            Self { life, commander },
            NoBoxSelect,
            Mesh3d(assets.meshes.square.clone()),
            MeshMaterial3d(assets.outlines.players[commander.peer().unwrap_or(peer)].clone()),
            Collider::cuboid(1.0, 1.0, CARD_THICKNESS / 64.0),
            RigidBody::Static,
            Transform::from_translation(Vec3::new(x, 0.0, z))
                .with_scale(Vec3::splat(MAT_DELTA_X * mult))
                .looking_to(Vec3::NEG_Y, if rev_z { Vec3::Z } else { Vec3::NEG_Z }),
            Children::spawn((
                Spawn((
                    Transform::from_xyz(0.0, 0.0, CARD_THICKNESS / 64.0),
                    Text3d::new(life.to_string()),
                    Mesh3d::default(),
                    MeshMaterial3d(assets.text_mesh.mesh.clone()),
                    Text3dStyling {
                        size: WORLD_FONT_SIZE,
                        anchor: TextAnchor::CENTER,
                        color: Srgba::BLACK,
                        world_scale: Some(Vec2::splat(0.5)),
                        ..Text3dStyling::default()
                    },
                )),
                SpawnWith(move |parent: &mut RelatedSpawner<ChildOf>| {
                    if commander.peer().is_none() {
                        parent.spawn((
                            peer,
                            CombatDamageText::default(),
                            Collider::cuboid(1.0, 1.0 / 5.0, CARD_THICKNESS / 32.0),
                            RigidBody::Static,
                            NoBoxSelect,
                            IgnoreSelectableObject,
                            Transform::from_xyz(0.0, -1.0 / 3.0, CARD_THICKNESS / 64.0),
                            Text3d::new(0.to_string()),
                            Mesh3d::default(),
                            MeshMaterial3d(mesh),
                            Text3dStyling {
                                size: WORLD_FONT_SIZE,
                                anchor: TextAnchor::CENTER,
                                color: Srgba::BLACK,
                                world_scale: Some(Vec2::splat(0.25)),
                                ..Text3dStyling::default()
                            },
                        ));
                    }
                }),
            )),
        )
    }
}
pub fn startup_life_counters(mut commands: Commands, assets: AssetManager) {
    for peer in EnumSet::<Peer>::all() {
        commands.spawn((
            LifeCounter::bundle(peer, CommanderCounter::None, &assets),
            SelectableObject,
        ));
        for commander in EnumSet::<Peer>::all() {
            commands.spawn(LifeCounter::bundle(
                peer,
                CommanderCounter::First(commander),
                &assets,
            ));
            commands.spawn(LifeCounter::bundle(
                peer,
                CommanderCounter::Second(commander),
                &assets,
            ));
        }
    }
}
#[query_fn]
pub fn life_counter_button(
    spatial: Spatial,
    counters: Query<(&LifeCounter, &Peer)>,
    combat_damage: Query<&CombatDamageText>,
    parents: Query<&ChildOf>,
    keybinds: Res<ButtonInput<Keybind>>,
    mut commands: Commands,
) {
    let increment = keybinds.just_pressed(Keybind::Increase);
    let decrement = keybinds.just_pressed(Keybind::Decrease);
    if !(increment ^ decrement) {
        return;
    }
    let Some((hit, _, _)) = spatial.ray() else {
        return;
    };
    let (delta, counter) = match (counters.get(hit.entity), combat_damage.get(hit.entity)) {
        (Ok(counter), Err(_)) => (1, counter),
        (Err(_), Ok(counter)) => (
            counter.delta,
            counters.get(parents.get(hit.entity).unwrap().0).unwrap(),
        ),
        _ => {
            return;
        }
    };
    let life = if increment {
        counter.life_counter.life + delta
    } else {
        counter.life_counter.life - delta
    };
    commands.trigger(NewLifeCount {
        peer: *counter.peer,
        commander: counter.life_counter.commander,
        life,
    });
}
