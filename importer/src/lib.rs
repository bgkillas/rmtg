use bevy::math::Vec3;
use bevy::prelude::Transform;
pub mod card;
pub mod deck;
pub mod id;
#[must_use]
pub fn is_reversed(transform: &Transform) -> bool {
    (transform.rotation * Vec3::Y).y < 0.0
}
