use crate::{CARD_HEIGHT, CARD_THICKNESS};
use avian3d::prelude::{
    AngularDamping, CoefficientCombine, CollisionLayers, GravityScale, LinearDamping, PhysicsLayer,
    Restitution, RigidBody, SleepThreshold,
};
use bevy::prelude::Bundle;
pub const GRAVITY: f32 = CARD_HEIGHT;
pub const LIN_DAMPING: f32 = CARD_THICKNESS;
pub const ANG_DAMPING: f32 = 0.25;
pub const LIN_SLEEP: f32 = 4.0 * CARD_THICKNESS;
pub const ANG_SLEEP: f32 = 0.25;
pub const BOUNCE: f32 = 0.5;
#[derive(PhysicsLayer, Default)]
pub enum GameLayer {
    #[default]
    Default,
    Floor,
}
#[must_use]
pub fn physics_base() -> impl Bundle + use<> {
    (
        RigidBody::Dynamic,
        LinearDamping(LIN_DAMPING),
        AngularDamping(ANG_DAMPING),
        SleepThreshold {
            linear: LIN_SLEEP,
            angular: ANG_SLEEP,
        },
        GravityScale(GRAVITY),
        CollisionLayers::new(GameLayer::Default, [GameLayer::Default, GameLayer::Floor]),
    )
}
#[must_use]
pub fn bounce() -> impl Bundle {
    Restitution::new(BOUNCE).with_combine_rule(CoefficientCombine::Max)
}
