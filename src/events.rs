use crate::misc::{Counter, Equipment, adjust_meshes, is_reversed, repaint_face, spawn_equip};
use crate::setup::SideMenu;
use crate::shapes::{Shape, Side};
use crate::sync::{Net, SyncObject, SyncObjectMe};
use crate::update::{SearchDeck, SearchText, update_search};
use crate::{CARD_HEIGHT, CARD_WIDTH, CardBase, FollowMouse, InHand, Menu, Pile};
use avian3d::prelude::{Collider, ColliderAabb, CollisionStart};
use bevy::prelude::*;
use bevy_ui_text_input::TextInputContents;
use std::mem;
#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct Scroll {
    pub entity: Entity,
    pub delta: Vec2,
}
pub fn on_scroll_handler(
    mut scroll: On<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };
    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();
    let delta = &mut scroll.delta;
    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0.0 {
        let max = if delta.x > 0.0 {
            scroll_position.x >= max_offset.x
        } else {
            scroll_position.x <= 0.0
        };
        if !max {
            scroll_position.x += delta.x;
            scroll_position.x = scroll_position.x.min(max_offset.x).max(0.0);
            delta.x = 0.0;
        }
    }
    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0.0 {
        let max = if delta.y > 0.0 {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.0
        };
        if !max {
            scroll_position.y += delta.y;
            scroll_position.y = scroll_position.y.min(max_offset.y).max(0.0);
            delta.y = 0.0;
        }
    }
    if *delta == Vec2::ZERO {
        scroll.propagate(false);
    }
}
pub fn pile_merge(
    collision: On<CollisionStart>,
    mut piles: Query<
        (
            Entity,
            &mut Pile,
            &mut Transform,
            &Children,
            &mut Collider,
            Option<&SyncObject>,
            Option<&SyncObjectMe>,
        ),
        (Without<InHand>, Without<FollowMouse>),
    >,
    mut mats: Query<&mut MeshMaterial3d<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query_meshes: Query<
        (&mut Mesh3d, &mut Transform),
        (
            Without<Children>,
            With<ChildOf>,
            Without<InHand>,
            Without<Shape>,
            Without<Pile>,
            Without<Side>,
        ),
    >,
    mut commands: Commands,
    search_deck: Option<Single<(Entity, &SearchDeck)>>,
    text: Option<Single<&TextInputContents, With<SearchText>>>,
    equipment: Query<(), Or<(With<Equipment>, With<Counter>)>>,
    (mut menu, side, net, card_base): (
        ResMut<Menu>,
        Option<Single<Entity, With<SideMenu>>>,
        Net,
        Res<CardBase>,
    ),
) {
    if let Ok((e1, p1, t1, _, _, s1, m1)) = piles.get(collision.collider1)
        && let Ok((e2, p2, t2, _, _, s2, m2)) = piles.get(collision.collider2)
        && e1 < e2
        && !p1.is_empty()
        && !p2.is_empty()
        && (t1.translation.x - t2.translation.x).abs() < CARD_WIDTH / 2.0
        && (t1.translation.z - t2.translation.z).abs() < CARD_HEIGHT / 2.0
        && is_reversed(t1) == is_reversed(t2)
    {
        let (
            (ent, mut bottom_pile, mut bottom_transform, children, mut collider, _, sync_me),
            top_pile,
            top_ent,
            top_sync,
            top_sync_me,
        ) = if t1.translation.y < t2.translation.y {
            if s1.is_some() {
                return;
            }
            let s2 = s2.cloned();
            let m2 = m2.cloned();
            let p2 = mem::replace(
                piles.get_mut(collision.collider2).unwrap().1.into_inner(),
                Pile::Empty,
            );
            (piles.get_mut(collision.collider1).unwrap(), p2, e2, s2, m2)
        } else {
            if s2.is_some() {
                return;
            }
            let s1 = s1.cloned();
            let m1 = m1.cloned();
            let p1 = mem::replace(
                piles.get_mut(collision.collider1).unwrap().1.into_inner(),
                Pile::Empty,
            );
            (piles.get_mut(collision.collider2).unwrap(), p1, e1, s1, m1)
        };
        if let Some(mid) = sync_me {
            let at = if is_reversed(&bottom_transform) {
                0
            } else {
                bottom_pile.len()
            };
            if let Some(id) = top_sync {
                net.merge(*mid, id, at)
            } else if let Some(id) = top_sync_me {
                net.merge_me(*mid, id, at)
            }
        }
        let mut equip = false;
        if top_pile.is_modified() {
            bottom_pile.merge(top_pile);
            equip = true;
        } else if is_reversed(&bottom_transform) {
            bottom_pile.extend_start(top_pile);
        } else {
            bottom_pile.extend(top_pile);
        }
        let card = bottom_pile.last();
        repaint_face(&mut mats, &mut materials, card, children);
        adjust_meshes(
            &bottom_pile,
            children,
            &mut meshes,
            &mut query_meshes,
            &mut bottom_transform,
            &mut collider,
            &equipment,
            &mut commands,
        );
        if equip {
            spawn_equip(
                ent,
                &bottom_pile,
                &mut commands,
                card_base.clone(),
                &mut materials,
                &mut meshes,
            );
        }
        if let Some(search_deck) = search_deck {
            if search_deck.1.0 == ent {
                update_search(
                    &mut commands,
                    search_deck.0,
                    &bottom_pile,
                    &bottom_transform,
                    text.as_ref().unwrap().get(),
                    &None,
                    &mut Menu::World,
                );
            } else if search_deck.1.0 == top_ent {
                *menu = Menu::World;
                commands.entity(**side.as_ref().unwrap()).despawn();
            }
        }
        commands.entity(top_ent).despawn();
    }
}
#[derive(EntityEvent, Deref, DerefMut)]
pub struct MoveToFloor(pub Entity);
pub fn move_to_floor(ent: On<MoveToFloor>, mut query: Query<(&mut Transform, &ColliderAabb)>) {
    let (mut transform, aabb) = query.get_mut(**ent).unwrap();
    transform.translation.y -= aabb.min.y;
}
