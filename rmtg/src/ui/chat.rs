use crate::focus::Focus;
use crate::keybinds::{Keybind, Keybinds};
use crate::{FONT_HEIGHT, FONT_SIZE};
use bevy::color::Color;
use bevy::prelude::{BackgroundColor, Event, Visibility};
use bevy::text::{EditableText, FontSize, TextCursorStyle, TextFont};
use bevy::ui::{AlignContent, Display, Node, Overflow, PositionType, RepeatedGridTrack, Val};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::children;
use bevy_ecs::component::Component;
use bevy_ecs::system::{Commands, Query};
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
#[derive(Event)]
pub struct TextSubmission {
    pub string: String,
    pub source: TextSource,
}
pub fn text_submission(
    focus: Focus,
    keybinds: Keybinds,
    mut text_input: Query<(&mut EditableText, &TextSource)>,
    mut commands: Commands,
) {
    if keybinds.just_pressed(Keybind::Chat)
        && let Some(focused_entity) = focus.active_input.get()
        && let Ok((mut text, &source)) = text_input.get_mut(focused_entity)
    {
        commands.trigger(TextSubmission {
            string: text.value().to_string(),
            source,
        });
        text.clear();
    }
}
