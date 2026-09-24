use crate::{BUTTON_BACKGROUND, BUTTON_HOVER, FONT_SIZE};
use bevy::prelude::{BackgroundColor, Node, Out, Over, Pointer, Text, Val, Visibility};
use bevy::text::{FontSize, TextFont};
use bevy::ui_widgets::{Button, observe};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::children;
use bevy_ecs::observer::On;
use bevy_ecs::prelude::Query;
use bevy_query_fn_macro::query_fn;
pub fn button(str: &str) -> impl Bundle {
    (
        Node {
            width: Val::Percent(100.0),
            min_width: Val::Percent(100.0),
            ..Node::default()
        },
        BackgroundColor(BUTTON_BACKGROUND),
        Visibility::Inherited,
        Button,
        observe(hover),
        observe(stop_hover),
        children![(
            Node { ..Node::default() },
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
fn hover(event: On<Pointer<Over>>, mut query: Query<&mut BackgroundColor>) {
    let mut bg = query.get_mut(event.entity).unwrap();
    bg.0 = BUTTON_HOVER;
}
#[query_fn]
fn stop_hover(event: On<Pointer<Out>>, mut query: Query<&mut BackgroundColor>) {
    let mut bg = query.get_mut(event.entity).unwrap();
    bg.0 = BUTTON_BACKGROUND;
}
