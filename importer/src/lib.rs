#![feature(array_try_map)]
use bevy::math::Vec3;
use bevy::prelude::Transform;
pub mod card;
pub mod deck;
pub mod id;
pub mod image;
pub mod scryfall;
#[cfg(test)]
mod scryfall_tests;
#[must_use]
pub fn is_reversed(transform: &Transform) -> bool {
    (transform.rotation * Vec3::Y).y < 0.0
}
