#![allow(clippy::shadow_reuse)]
use bevy::ecs::system::SystemParam;
use bevy::input::ButtonInput;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::picking::hover::HoverMap;
use bevy::prelude::{MouseButton, Res, Resource};
use bevy::ui::Node;
use bevy::window::Window;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::system::{Query, ResMut, Single};
use enumset::{EnumSet, EnumSetType};
#[derive(Resource, EnumSetType, Default, Debug)]
pub enum Menu {
    #[default]
    World,
    Counter,
    Esc,
    Side,
}
#[derive(SystemParam)]
pub struct Hover<'w, 's> {
    pub window: Single<'w, 's, Entity, With<Window>>,
    pub hover_map: Res<'w, HoverMap>,
}
impl Hover<'_, '_> {
    #[must_use]
    pub fn get(&self) -> Option<Entity> {
        if self.hover_map.is_empty() {
            return None;
        }
        assert_eq!(self.hover_map.len(), 1);
        let val = self.hover_map.values().next().unwrap();
        if val.is_empty() {
            return None;
        }
        assert_eq!(val.len(), 1);
        let &ent = val.keys().next().unwrap();
        if ent == *self.window { None } else { Some(ent) }
    }
}
#[derive(SystemParam)]
pub struct Focus<'w, 's> {
    pub active_input: Res<'w, InputFocus>,
    pub hover: Hover<'w, 's>,
    pub menu: Res<'w, Menu>,
}
impl Focus<'_, '_> {
    #[must_use]
    pub fn key_lock(&self, set: EnumSet<Menu>) -> bool {
        !set.contains(*self.menu)
            || self
                .active_input
                .get()
                .is_some_and(|e| e != *self.hover.window)
    }
    #[must_use]
    pub fn mouse_lock(&self, set: EnumSet<Menu>) -> bool {
        !set.contains(*self.menu) || self.hover.get().is_some()
    }
}
pub fn update_focus(
    mut active_input: ResMut<InputFocus>,
    hover: Hover,
    nodes: Query<(), With<Node>>,
    window: Single<Entity, With<Window>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if !mouse.any_just_pressed([
        MouseButton::Left,
        MouseButton::Right,
        MouseButton::Middle,
        MouseButton::Back,
        MouseButton::Forward,
    ]) {
        return;
    }
    if let Some(ent) = hover.get()
        && nodes.contains(ent)
    {
        active_input.set(ent, FocusCause::Pressed);
    } else {
        active_input.set(*window, FocusCause::Pressed);
    }
}
