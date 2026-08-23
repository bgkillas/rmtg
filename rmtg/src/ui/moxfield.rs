use bevy::color::Color;
use bevy::prelude::{Component, Visibility};
use bevy::ui::{BackgroundColor, Node, Val};
use bevy_ecs::bundle::Bundle;
#[derive(Component)]
pub struct MoxfieldMenu;
impl MoxfieldMenu {
    #[must_use]
    pub fn bundle() -> impl Bundle {
        (
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..Node::default()
            },
            Self,
            Visibility::Hidden,
            BackgroundColor(Color::srgba_u8(0, 0, 0, 128)),
        )
    }
}
