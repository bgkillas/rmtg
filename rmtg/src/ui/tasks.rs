use crate::{FONT_HEIGHT, FONT_SIZE, FONT_WIDTH};
use bevy::color::Color;
use bevy::prelude::{BackgroundColor, Component, FontSize, Node, Text, TextFont, Val, Visibility};
use bevy::ui::{AlignItems, FlexDirection, JustifyContent};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::children;
use bevy_ecs::query::With;
use bevy_ecs::system::{Res, Single};
use bevy_p2p::runtime::Runtime;
#[derive(Component)]
pub struct TasksCounter;
#[derive(Component)]
pub struct TasksCounterText;
impl TasksCounter {
    #[must_use]
    pub fn bundle() -> impl Bundle {
        (
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..Node::default()
            },
            Visibility::Hidden,
            children![(
                Node {
                    width: Val::Px(FONT_WIDTH * 6.0),
                    height: Val::Px(FONT_HEIGHT * 1.5),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Node::default()
                },
                Self,
                Visibility::Hidden,
                BackgroundColor(Color::srgba_u8(0, 0, 0, 128)),
                children![(
                    Node { ..Node::default() },
                    Visibility::Inherited,
                    TasksCounterText,
                    Text(String::default()),
                    TextFont {
                        font_size: FontSize::Px(FONT_SIZE),
                        ..TextFont::default()
                    },
                )]
            )],
        )
    }
}
pub fn update_tasks_counter(
    runtime: Res<Runtime>,
    mut visibility: Single<&mut Visibility, With<TasksCounter>>,
    mut text: Single<&mut Text, With<TasksCounterText>>,
) {
    let tasks = runtime.runtime.metrics().num_alive_tasks();
    **visibility = if tasks == 0 {
        Visibility::Hidden
    } else {
        text.0 = tasks.to_string();
        Visibility::Visible
    };
}
