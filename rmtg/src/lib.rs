#![allow(incomplete_features)]
#![feature(min_generic_const_args)]
#![feature(inherent_associated_types)]
#![feature(associated_type_defaults)]
use bevy::color::Color;
pub mod app;
pub mod assets;
pub mod camera;
pub mod card_spot;
pub mod focus;
pub mod keybinds;
pub mod mat;
pub mod net;
pub mod physics;
pub mod shapes;
pub mod startup;
pub const APP_NAME: &str = "com.github.bgkillas.rmtg";
pub const USER_AGENT: &str = concat!("rmtg/", env!("CARGO_PKG_VERSION"));
pub const CARD_WIDTH: f32 = CARD_HEIGHT * IMAGE_WIDTH / IMAGE_HEIGHT;
pub const CARD_HEIGHT: f32 = (MAT_HEIGHT - MAT_BAR) / 5.0 - MAT_BAR;
pub const IMAGE_WIDTH: f32 = 500.0;
pub const IMAGE_HEIGHT: f32 = 700.0;
pub const EQUIP_SCALE: f32 = 0.5;
pub const CARD_THICKNESS: f32 = CARD_WIDTH / 256.0;
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
pub const T: f32 = W / 2.0;
pub const W: f32 = MAT_WIDTH * 2.0;
pub const WALL_COLOR: Color = Color::srgb_u8(103, 73, 40);
pub const FLOOR_COLOR: Color = Color::srgb_u8(103, 73, 40);
pub const WORLD_FONT_SIZE: f32 = 120.0;
pub const FONT: &[u8] = include_bytes!("../../assets/noto.ttf");
