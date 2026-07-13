use bevy::camera::Camera3d;
use bevy::prelude::Commands;
pub fn startup(mut commands: Commands) {
    commands.spawn(Camera3d::default());
}
