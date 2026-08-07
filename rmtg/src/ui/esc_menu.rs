use bevy::color::Color;
use bevy::prelude::{BackgroundColor, Component, Visibility};
use bevy::ui::{Node, Val};
use bevy_ecs::bundle::Bundle;
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
