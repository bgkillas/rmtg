#![allow(incomplete_features)]
#![feature(array_try_map)]
#![feature(min_generic_const_args)]
#![cfg_attr(test, feature(test))]
extern crate core;
pub mod card;
pub mod id;
pub mod image;
pub use bitcode;
pub use reqwest;
pub use uuid;
pub mod circle;
pub mod coder;
#[cfg(test)]
mod image_bench;
pub mod scryfall;
#[cfg(test)]
mod scryfall_tests;
pub const CARD_CORNER_RADIUS: f32 = 1.0 / 20.0;
