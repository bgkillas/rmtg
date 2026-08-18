use crate::focus::Menu;
use crate::keybinds::Keybind;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::prelude::{BackgroundColor, Component, Visibility};
use bevy::ui::{Node, Val};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::query::With;
use bevy_ecs::system::{Res, ResMut, Single};
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
pub fn toggle_esc_menu(
    mut esc: Single<&mut Visibility, With<EscMenu>>,
    keybinds: Res<ButtonInput<Keybind>>,
    mut menu: ResMut<Menu>,
) {
    if keybinds.just_pressed(Keybind::Menu) {
        match *menu {
            Menu::World | Menu::Side | Menu::Counter => {
                **esc = Visibility::Visible;
                *menu = Menu::Esc;
            }
            Menu::Esc => {
                **esc = Visibility::Hidden;
                *menu = Menu::World;
            }
        }
    }
}
