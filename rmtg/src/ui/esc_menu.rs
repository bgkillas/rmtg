use crate::focus::Menu;
use crate::keybinds::Keybind;
use crate::{BUTTON_BACKGROUND, BUTTON_BORDER, BUTTON_HOVER, FONT_SIZE};
use bevy::app::AppExit;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::input_focus::{FocusCause, InputFocus};
use bevy::prelude::{BackgroundColor, Component, Text, Visibility, Window};
use bevy::text::{FontSize, TextFont};
use bevy::ui::{BorderColor, Interaction, Node, PositionType, Val};
use bevy::ui_widgets::{Activate, Button, observe};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::children;
use bevy_ecs::entity::Entity;
use bevy_ecs::message::MessageWriter;
use bevy_ecs::observer::On;
use bevy_ecs::query::{Changed, With};
use bevy_ecs::system::{Query, Res, ResMut, Single};
use bevy_query_fn_macro::query_fn;
#[derive(Component)]
pub struct EscMenu;
#[derive(Component)]
pub struct Exit;
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
        BackgroundColor(Color::srgba_u8(0, 0, 0, 128)),
        children![(
            Node {
                width: Val::Percent(50.0),
                height: Val::Percent(50.0),
                top: Val::Percent(25.0),
                left: Val::Percent(25.0),
                position_type: PositionType::Absolute,
                ..Node::default()
            },
            Visibility::Inherited,
            children![(button("Exit"), observe(on_exit))]
        )],
    )
}
fn on_exit(_: On<Activate>, mut writer: MessageWriter<AppExit>) {
    writer.write(AppExit::Success);
}
pub fn button(str: &str) -> impl Bundle {
    (
        Node { ..Node::default() },
        BorderColor::all(BUTTON_BORDER),
        BackgroundColor(BUTTON_BACKGROUND),
        Visibility::Inherited,
        Button,
        children![(
            Visibility::Inherited,
            Text::new(str),
            TextFont {
                font_size: FontSize::Px(FONT_SIZE),
                ..TextFont::default()
            },
        )],
    )
}
#[query_fn]
pub fn button_system(
    interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        Changed<Interaction>,
    >,
) {
    for mut interaction in interaction_query {
        *interaction.border_color = match *interaction.interaction {
            Interaction::None | Interaction::Pressed => BorderColor::all(BUTTON_BORDER),
            Interaction::Hovered => BorderColor::all(BUTTON_HOVER),
        };
    }
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
