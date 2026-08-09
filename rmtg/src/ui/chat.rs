use crate::{FONT_HEIGHT, FONT_SIZE};
use bevy::color::Color;
use bevy::prelude::{BackgroundColor, Visibility};
use bevy::text::{EditableText, FontSize, TextFont};
use bevy::ui::{AlignContent, Display, Node, Overflow, PositionType, RepeatedGridTrack, Val};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::children;
use bevy_ecs::component::Component;
#[derive(Component)]
pub struct TextMenu;
#[derive(Component)]
pub struct TextChat;
#[derive(Component)]
pub struct TextInput;
#[must_use]
pub fn chat_bundle() -> impl Bundle {
    (
        Node {
            width: Val::Percent(25.0),
            height: Val::Percent(25.0),
            left: Val::Percent(0.0),
            top: Val::Percent(75.0),
            ..Node::default()
        },
        TextMenu,
        Visibility::Visible,
        BackgroundColor(Color::srgba_u8(0, 0, 0, 64)),
        children![
            (
                Node {
                    width: Val::Percent(100.0),
                    bottom: Val::Percent(0.0),
                    position_type: PositionType::Absolute,
                    height: Val::Px(FONT_HEIGHT * 1.5),
                    ..Node::default()
                },
                EditableText::default(),
                TextFont {
                    font_size: FontSize::Px(FONT_SIZE),
                    ..TextFont::default()
                },
                TextInput,
                Visibility::Inherited,
            ),
            (
                Node {
                    width: Val::Percent(100.0),
                    top: Val::Percent(0.0),
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(FONT_HEIGHT * 1.5),
                    overflow: Overflow::scroll_y(),
                    display: Display::Grid,
                    grid_template_columns: vec![RepeatedGridTrack::percent(1, 100.0)],
                    align_content: AlignContent::Start,
                    ..Node::default()
                },
                TextChat,
                Visibility::Inherited,
            ),
        ],
    )
}
