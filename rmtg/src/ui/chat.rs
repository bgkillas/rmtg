use crate::events::scroll::{Scroll, Scrollable};
use crate::ui::text_box::{TextSource, TextSubmission};
use crate::{FONT_HEIGHT, FONT_SIZE};
use bevy::color::Color;
use bevy::prelude::{BackgroundColor, FlexDirection, Text, Visibility};
use bevy::text::{FontSize, TextFont};
use bevy::ui::{Display, Node, Overflow, PositionType, Val};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::children;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::message::MessageWriter;
use bevy_ecs::observer::On;
use bevy_ecs::prelude::{Commands, Single, With};
#[derive(Component)]
pub struct TextMenu;
#[derive(Component)]
pub struct TextChat;
impl TextMenu {
    #[must_use]
    pub fn bundle() -> impl Bundle {
        (
            Node {
                width: Val::Percent(25.0),
                height: Val::Percent(25.0),
                left: Val::Percent(0.0),
                top: Val::Percent(75.0),
                ..Node::default()
            },
            Self,
            Visibility::Visible,
            BackgroundColor(Color::srgba_u8(0, 0, 0, 64)),
            children![
                (
                    BackgroundColor(Color::srgba_u8(0, 0, 0, 32)),
                    Node {
                        width: Val::Percent(100.0),
                        bottom: Val::Percent(0.0),
                        position_type: PositionType::Absolute,
                        height: Val::Px(FONT_HEIGHT * 1.5),
                        ..Node::default()
                    },
                    Visibility::Inherited,
                    TextSource::Chat.bundle()
                ),
                (
                    Node {
                        width: Val::Percent(100.0),
                        top: Val::Percent(0.0),
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(FONT_HEIGHT * 1.5),
                        overflow: Overflow::scroll_y(),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        ..Node::default()
                    },
                    TextChat,
                    Visibility::Inherited,
                    Scrollable,
                ),
            ],
        )
    }
}
#[must_use]
pub fn text_node(str: String) -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            flex_shrink: 0.0,
            ..Node::default()
        },
        Text(str),
        Visibility::Inherited,
        TextFont {
            font_size: FontSize::Px(FONT_SIZE),
            ..TextFont::default()
        },
    )
}
pub fn text_message(
    event: On<TextSubmission>,
    mut commands: Commands,
    text_chat: Single<Entity, With<TextChat>>,
    mut msgs: MessageWriter<Scroll>,
) {
    if !matches!(event.source, TextSource::Chat) {
        return;
    }
    commands
        .entity(*text_chat)
        .with_child(text_node(event.string.clone()));
    msgs.write(Scroll::down(*text_chat));
}
