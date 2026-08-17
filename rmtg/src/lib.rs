#![allow(incomplete_features)]
#![feature(min_generic_const_args)]
#![feature(inherent_associated_types)]
#![feature(associated_type_defaults)]
#![feature(mpmc_channel)]
extern crate core;
use bevy::color::Color;
use importer::scryfall::Quality;
pub mod app;
pub mod assets;
pub mod camera;
pub mod card_spot;
pub mod drag;
pub mod events;
pub mod focus;
pub mod keybinds;
pub mod mat;
pub mod net;
pub mod paste;
pub mod physics;
pub mod pile;
pub mod shapes;
pub mod spatial;
pub mod startup;
pub mod ui;
//TODO oracle card
pub const APP_NAME: &str = "com.github.bgkillas.rmtg";
pub const USER_AGENT: &str = concat!("com.github.bgkillas.rmtg/", env!("CARGO_PKG_VERSION"));
pub const ALPN: &[u8] = USER_AGENT.as_bytes();
pub const STEAM_APP_ID: u32 = 4046880;
pub const CARD_WIDTH: f32 = CARD_HEIGHT * IMAGE_WIDTH / IMAGE_HEIGHT;
pub const CARD_HEIGHT: f32 = (MAT_HEIGHT - MAT_BAR) / 5.0 - MAT_BAR;
pub const IMAGE_WIDTH: f32 = 744.0;
pub const IMAGE_HEIGHT: f32 = 1039.0;
pub const EQUIP_SCALE: f32 = 0.5;
pub const CARD_THICKNESS: f32 = CARD_WIDTH / 128.0;
pub const START_Y: f32 = MAT_WIDTH;
pub const PLAYER0: Color = Color::srgb_u8(255, 85, 85);
pub const PLAYER1: Color = Color::srgb_u8(85, 85, 255);
pub const PLAYER2: Color = Color::srgb_u8(255, 85, 255);
pub const PLAYER3: Color = Color::srgb_u8(85, 255, 85);
pub const PLAYER4: Color = Color::srgb_u8(85, 255, 255);
pub const PLAYER5: Color = Color::srgb_u8(255, 255, 85);
pub const PLAYER: [Color; 6] = [PLAYER0, PLAYER1, PLAYER2, PLAYER3, PLAYER4, PLAYER5];
pub const MAT_WIDTH: f32 = 8.0;
pub const MAT_HEIGHT: f32 = MAT_WIDTH * 9.0 / 16.0;
pub const MAT_BAR: f32 = MAT_HEIGHT / 64.0;
pub const QUALITY: Quality = Quality::Large;
pub const T: f32 = W / 2.0;
pub const W: f32 = MAT_WIDTH * 2.0;
pub const CEILING_COLOR: Color = Color::srgb_u8(103, 73, 40);
pub const WALL_COLOR: Color = Color::srgb_u8(103, 73, 40);
pub const FLOOR_COLOR: Color = Color::srgb_u8(103, 73, 40);
pub const CARD_STOCK_COLOR: Color = Color::srgb_u8(0, 0, 0);
pub const SCROLLBAR: Color = Color::srgb(0.486, 0.486, 0.529);
pub const SCROLLBAR_OUTLINE: Color = Color::srgb(0.71, 0.71, 0.772);
pub const SCROLLBAR_HOVER: Color = Color::srgb(1.0, 1.0, 1.0);
pub const WORLD_FONT_SIZE: f32 = 280.0;
pub const FONT: &[u8] = include_bytes!("../../assets/noto.ttf");
pub const FONT_SIZE: f32 = 16.0;
pub const FONT_HEIGHT: f32 = FONT_SIZE;
pub const FONT_WIDTH: f32 = FONT_HEIGHT * 3.0 / 5.0;
pub const PHYSICS_SCALE: f32 = CARD_HEIGHT * MAT_WIDTH / 8.0;
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
fn wasm_hook() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    let _ = app::app_run();
}
