use crate::focus::Menu;
use crate::keybinds::Keybind;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::{BackgroundColor, Component, Visibility, Window};
use bevy::ui::{Node, Val};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::system::{Res, ResMut, Single};
use bevy_query_fn_macro::query_fn;
#[derive(Component)]
pub struct EscMenu;
#[must_use]
pub fn esc_menu_bundle() -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..Node::default()
        },
        EscMenu,
        Visibility::Hidden,
        BackgroundColor(Color::srgba_u8(0, 0, 0, 127)),
    )
}
#[query_fn]
pub fn toggle_esc_menu(
    mut esc: Single<(Entity, &mut Visibility), With<EscMenu>>,
    keybinds: Res<ButtonInput<Keybind>>,
    mut menu: ResMut<Menu>,
    mut active_input: ResMut<InputFocus>,
    window: Single<Entity, With<Window>>,
) {
    if keybinds.just_pressed(Keybind::Menu) {
        match *menu {
            Menu::World | Menu::Side | Menu::Counter => {
                *esc.visibility = Visibility::Visible;
                *menu = Menu::Esc;
                active_input.set(esc.entity, FocusCause::Pressed);
            }
            Menu::Esc => {
                *esc.visibility = Visibility::Hidden;
                *menu = Menu::World;
                active_input.set(*window, FocusCause::Pressed);
            }
        }
    }
}
