use crate::FONT_SIZE;
use crate::events::scroll::Scroll;
use crate::keybinds::Keybind;
use crate::ui::chat::{TextChat, text_node};
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::{Component, Event, FontSize, TextFont, Window};
use bevy::text::{EditableText, TextCursorStyle};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::{Res, ResMut};
use bevy_ecs::entity::Entity;
use bevy_ecs::message::MessageWriter;
use bevy_ecs::observer::On;
use bevy_ecs::prelude::{Commands, Query, Single, With};
use bevy_query_fn_macro::query_fn;
#[derive(Component, Clone, Copy)]
pub enum TextSource {
    Chat,
    Moxfield,
}
impl TextSource {
    pub fn bundle(self) -> impl Bundle {
        (
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
            self,
        )
    }
}
#[derive(Event)]
pub struct TextSubmission {
    pub string: String,
    pub source: TextSource,
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
#[query_fn]
pub fn text_submission(
    window: Single<Entity, With<Window>>,
    mut active_input: ResMut<InputFocus>,
    keybinds: Res<ButtonInput<Keybind>>,
    mut text_input: Query<(Entity, &mut EditableText, &TextSource)>,
    mut commands: Commands,
) {
    if keybinds.just_pressed(Keybind::Chat) {
        if let Some(focused_entity) = active_input.get()
            && let Ok(mut text) = text_input.get_mut(focused_entity)
        {
            commands.trigger(TextSubmission {
                string: text.editable_text.value().to_string(),
                source: *text.text_source,
            });
            text.editable_text.clear();
            active_input.set(*window, FocusCause::Pressed);
        } else {
            active_input.set(
                text_input
                    .iter()
                    .find(|p| matches!(p.text_source, TextSource::Chat))
                    .unwrap()
                    .entity,
                FocusCause::Pressed,
            );
        }
    }
}
