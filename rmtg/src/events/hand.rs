use crate::mat::{MAT_DELTA_X, MAT_EDGE_Z};
use crate::net::Peer;
use crate::pile::Pile;
use crate::spatial::Spatial;
use crate::{CARD_HEIGHT, CARD_WIDTH, MAT_BAR, MAT_WIDTH};
use avian3d::prelude::ColliderAabb;
use bevy::math::Vec3;
use bevy::prelude::Transform;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::lifecycle::{Add, Remove};
use bevy_ecs::observer::On;
use bevy_ecs::query::Without;
use bevy_ecs::system::{Commands, Query};
use bevy_query_fn_macro::query_fn;
#[derive(Component)]
pub struct Hand {
    pub collider: ColliderAabb,
}
#[derive(Component)]
pub struct InHand;
impl Hand {
    pub fn bundle(peer: Peer) -> impl Bundle {
        let (rev_x, rev_z) = match peer {
            Peer::Zero => (false, false),
            Peer::One => (true, false),
            Peer::Two => (true, true),
            Peer::Three => (false, true),
        };
        let width = MAT_WIDTH - 3.0 * MAT_BAR - CARD_WIDTH;
        let mut x = MAT_DELTA_X + MAT_BAR + width / 2.0;
        let mut z = MAT_EDGE_Z + CARD_HEIGHT / 2.0;
        if rev_x {
            x = -x;
        }
        if rev_z {
            z = -z;
        }
        let collider = ColliderAabb::new(
            Vec3::new(x, CARD_HEIGHT / 2.0, z),
            Vec3::new(width / 2.0, CARD_HEIGHT / 2.0, CARD_HEIGHT / 4.0),
        );
        (
            Self { collider },
            peer,
            Transform::from_translation(collider.center()),
        )
    }
}
#[query_fn]
pub fn hand_startup(mut commands: Commands) {
    commands.spawn(Hand::bundle(Peer::Zero));
    commands.spawn(Hand::bundle(Peer::One));
    commands.spawn(Hand::bundle(Peer::Two));
    commands.spawn(Hand::bundle(Peer::Three));
}
#[query_fn]
pub fn add_to_hand(event: On<Add, InHand>) {
    //TODO
    _ = event;
}
#[query_fn]
pub fn remove_from_hand(event: On<Remove, InHand>) {
    //TODO
    _ = event;
}
#[query_fn]
pub fn update_hand() {
    //TODO
}
#[query_fn]
pub fn add_near_to_hand(
    hands: Query<&Hand>,
    piles: Query<&Pile, Without<InHand>>,
    mut commands: Commands,
    spatial: Spatial,
) {
    for hand in hands {
        spatial
            .spatial
            .aabb_intersections_with_aabb_callback(hand.collider, |ent| {
                if let Ok(pile) = piles.get(ent)
                    && pile.len() == 1
                {
                    commands.entity(ent).insert(InHand);
                }
                true
            });
    }
}
