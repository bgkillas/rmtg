#![allow(clippy::shadow_reuse)]
use bevy::ecs::system::SystemParam;
use bevy::input_focus::InputFocus;
use bevy::picking::hover::HoverMap;
use bevy::prelude::{Res, ResMut, Resource};
#[derive(Resource, Default, Debug)]
pub enum Menu {
    #[default]
    World,
    Counter,
    Esc,
    Side,
}
#[derive(SystemParam)]
pub struct Focus<'w> {
    menu: ResMut<'w, Menu>,
    active_input: Res<'w, InputFocus>,
    hover_map: Res<'w, HoverMap>,
}
impl Focus<'_> {
    #[must_use]
    pub fn key_lock(&self) -> bool {
        self.active_input
            .get()
            .is_some_and(|e| e.to_bits() == u64::from(u32::MAX))
            || matches!(*self.menu, Menu::Esc)
    }
    #[must_use]
    pub fn mouse_lock(&self) -> bool {
        self.hover_map
            .values()
            .any(|a| a.keys().any(|e| e.to_bits() == u64::from(u32::MAX)))
    }
}
