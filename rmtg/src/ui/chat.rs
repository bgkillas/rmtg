use crate::events::scroll::{Scroll, Scrollable};
use crate::focus::Focus;
use crate::keybinds::{Keybind, Keybinds};
use crate::{FONT_HEIGHT, FONT_SIZE};
use bevy::color::Color;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::{BackgroundColor, Event, FlexDirection, Text, Visibility};
use bevy::text::{EditableText, FontSize, TextCursorStyle, TextFont};
use bevy::ui::{Display, Node, Overflow, PositionType, Val};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::children;
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::message::MessageWriter;
use bevy_ecs::observer::On;
use bevy_ecs::query::With;
use bevy_ecs::system::{Commands, ParamSet, Query, ResMut, Single};
#[derive(Component)]
pub struct TextMenu;
#[derive(Component)]
pub struct TextChat;
#[derive(Component)]
pub struct TextInput;
#[derive(Component, Clone, Copy)]
pub enum TextSource {
    Chat,
}
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
                BackgroundColor(Color::srgba_u8(0, 0, 0, 32)),
                Node {
                    width: Val::Percent(100.0),
                    bottom: Val::Percent(0.0),
                    position_type: PositionType::Absolute,
                    height: Val::Px(FONT_HEIGHT),
                    ..Node::default()
                },
                EditableText::default(),
                TextCursorStyle {
                    color: Color::WHITE,
                    selection_color: Color::srgb_u8(53, 132, 228),
                    unfocused_selection_color: Color::srgb_u8(176, 176, 176),
                    selected_text_color: None,
                },
                TextFont {
                    font_size: FontSize::Px(FONT_SIZE),
                    ..TextFont::default()
                },
                TextInput,
                Visibility::Inherited,
                TextSource::Chat
            ),
            (
                Node {
                    width: Val::Percent(100.0),
                    top: Val::Percent(0.0),
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(FONT_HEIGHT),
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
#[derive(Event)]
pub struct TextSubmission {
    pub string: String,
    pub source: TextSource,
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
    commands
        .entity(*text_chat)
        .with_child(text_node(event.string.clone()));
    msgs.write(Scroll::down(*text_chat));
}
pub fn text_submission(
    mut focus: ParamSet<(Focus, ResMut<InputFocus>)>,
    keybinds: Keybinds,
    mut text_input: Query<(&mut EditableText, &TextSource)>,
    mut commands: Commands,
    chat: Single<Entity, With<TextInput>>,
) {
    if keybinds.just_pressed(Keybind::Chat) {
        if let Some(focused_entity) = focus.p0().active_input.get()
            && let Ok((mut text, &source)) = text_input.get_mut(focused_entity)
        {
            commands.trigger(TextSubmission {
                string: text.value().to_string(),
                source,
            });
            text.clear();
            let ent = *focus.p0().window;
            focus.p1().set(ent, FocusCause::Pressed);
        } else {
            focus.p1().set(*chat, FocusCause::Pressed);
        }
    }
}
