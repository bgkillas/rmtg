use crate::mat::{MAT_DELTA_X, MAT_EDGE_Z};
use crate::net::Peer;
use crate::{CARD_HEIGHT, CARD_WIDTH, MAT_BAR, MAT_WIDTH};
use avian3d::prelude::ColliderAabb;
use bevy::math::Vec3;
use bevy::prelude::Transform;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::component::Component;
use bevy_ecs::system::Commands;
use bevy_query_fn_macro::query_fn;
#[derive(Component)]
pub struct Hand {
    pub collider: ColliderAabb,
}
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
            Vec3::new(width / 2.0, CARD_HEIGHT / 2.0, CARD_HEIGHT / 2.0),
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
pub fn add_near_to_hand() {
    //TODO
}
