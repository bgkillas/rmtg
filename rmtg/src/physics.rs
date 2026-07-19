use crate::{CARD_HEIGHT, CARD_THICKNESS};
use avian3d::prelude::{
    AngularDamping, CoefficientCombine, Collider, GravityScale, LinearDamping, Restitution,
    RigidBody, SleepThreshold,
};
use bevy::mesh::Mesh;
use bevy::prelude::Bundle;
pub const GRAVITY: f32 = CARD_HEIGHT;
pub const LIN_DAMPING: f32 = CARD_THICKNESS;
pub const ANG_DAMPING: f32 = 0.25;
pub const LIN_SLEEP: f32 = 4.0 * CARD_THICKNESS;
pub const ANG_SLEEP: f32 = 0.25;
pub const BOUNCE: f32 = 0.5;
#[must_use]
pub fn physics(mesh: &Mesh) -> impl Bundle + use<> {
    (
        Collider::convex_hull_from_mesh(mesh).unwrap(),
        RigidBody::Dynamic,
        LinearDamping(LIN_DAMPING),
        AngularDamping(ANG_DAMPING),
        SleepThreshold {
            linear: LIN_SLEEP,
            angular: ANG_SLEEP,
        },
        GravityScale(GRAVITY),
    )
}
#[must_use]
pub fn bounce() -> impl Bundle {
    Restitution::new(BOUNCE).with_combine_rule(CoefficientCombine::Max)
}
