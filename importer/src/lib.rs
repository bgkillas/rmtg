#![expect(incomplete_features)]
#![feature(array_try_map)]
#![feature(min_generic_const_args)]
#![feature(extern_item_impls)]
#![feature(arc_is_unique)]
#![feature(type_alias_impl_trait)]
#![feature(iter_array_chunks)]
#![cfg_attr(test, feature(test))]
extern crate core;
pub mod card;
pub mod image;
use bevy::log::warn;
pub use bitcode;
pub use reqwest;
use std::fmt::Debug;
pub use uuid;
pub mod card_cache;
pub mod circle;
pub mod coder;
#[cfg(test)]
mod image_bench;
pub mod moxfield;
#[cfg(test)]
mod moxfield_tests;
pub mod oracle_card;
pub mod scryfall;
#[cfg(test)]
mod scryfall_tests;
pub const CARD_CORNER_RADIUS: f32 = 1.0 / 20.0;
#[eii(app_name)]
pub fn app_name() -> &'static str {
    "com.github.bgkillas.importer"
}
pub fn warn_if<T, E: Debug>(val: Result<T, E>) -> Option<T> {
    match val {
        Ok(v) => Some(v),
        Err(e) => {
            warn!("{e:?}");
            None
        }
    }
}
