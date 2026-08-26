use crate::{FONT_HEIGHT, FONT_SIZE, FONT_WIDTH};
use bevy::color::Color;
use bevy::prelude::{BackgroundColor, Component, FontSize, Node, Text, TextFont, Val, Visibility};
use bevy::ui::{AlignItems, FlexDirection, JustifyContent};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::children;
use bevy_ecs::query::With;
use bevy_ecs::system::Single;
use importer::scryfall::{CACHE, IMAGES_IN_PROGRESS};
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
                    width: Val::Px(FONT_WIDTH * 16.0),
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
    mut visibility: Single<&mut Visibility, With<TasksCounter>>,
    mut text: Single<&mut Text, With<TasksCounterText>>,
) {
    let cache = CACHE.blocking_lock();
    let uuid = cache.in_progress.len();
    let set_cn = cache.in_progress_set_cn.len();
    drop(cache);
    let images_in_progress = IMAGES_IN_PROGRESS.blocking_lock();
    let images = images_in_progress.len();
    drop(images_in_progress);
    **visibility = if uuid == 0 && set_cn == 0 && images == 0 {
        Visibility::Hidden
    } else {
        text.0 = format!("{} {images}", uuid + set_cn);
        Visibility::Visible
    };
}
