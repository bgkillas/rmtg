#![allow(incomplete_features)]
#![feature(array_try_map)]
#![feature(min_generic_const_args)]
#![cfg_attr(test, feature(test))]
extern crate core;
use bevy::math::Vec3;
use bevy::prelude::Transform;
pub mod card;
pub mod id;
pub mod image;
pub use bitcode;
pub use reqwest;
pub use uuid;
pub mod coder;
#[cfg(test)]
mod image_bench;
pub mod scryfall;
#[cfg(test)]
mod scryfall_tests;
#[must_use]
pub fn is_reversed(transform: &Transform) -> bool {
    (transform.rotation * Vec3::Y).y < 0.0
}
