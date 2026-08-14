use crate::{CARD_HEIGHT, CARD_THICKNESS, CARD_WIDTH, PHYSICS_SCALE};
use avian3d::prelude::{
    AngularDamping, CoefficientCombine, CollisionLayers, Friction, GravityScale, LayerMask,
    LinearDamping, PhysicsLayer, Restitution, RigidBody, SleepThreshold,
};
use bevy::prelude::Bundle;
use std::f32::consts::TAU;
pub const GRAVITY: f32 = CARD_HEIGHT;
pub const LIN_DAMPING: f32 = CARD_WIDTH / 4.0;
pub const ANG_DAMPING: f32 = 0.0;
pub const LIN_SLEEP: f32 = 4.0 * CARD_THICKNESS / PHYSICS_SCALE;
pub const ANG_SLEEP: f32 = TAU / 32.0;
pub const BOUNCE: f32 = 0.5;
#[derive(PhysicsLayer, Default)]
pub enum WorldLayer {
    #[default]
    Default,
    Floor,
}
#[must_use]
pub fn physics_base() -> impl Bundle + use<> {
    (
        Friction::new(0.5).with_combine_rule(CoefficientCombine::Max),
        RigidBody::Dynamic,
        LinearDamping(LIN_DAMPING),
        AngularDamping(ANG_DAMPING),
        SleepThreshold {
            linear: LIN_SLEEP,
            angular: ANG_SLEEP,
        },
        GravityScale(GRAVITY),
        CollisionLayers::new(WorldLayer::Default, LayerMask::ALL),
    )
}
#[must_use]
pub fn bounce() -> impl Bundle {
    Restitution::new(BOUNCE).with_combine_rule(CoefficientCombine::Max)
}
