use bevy::color::Color;
use bevy::picking::hover::Hovered;
use bevy::prelude::Component;
use bevy::ui::{
    AlignItems, BackgroundColor, BorderRadius, Display, FlexDirection, JustifyContent, Node,
    PositionType, Val,
};
use bevy::ui_widgets::{Slider, SliderRange, SliderValue, TrackClick, observe, slider_self_update};
use bevy_ecs::children;
use bevy_ecs::prelude::Bundle;
const SLIDER_TRACK: Color = Color::srgb(0.05, 0.05, 0.05);
const SLIDER_THUMB: Color = Color::srgb(0.35, 0.75, 0.35);
#[derive(Component)]
pub struct SliderThumb;
pub fn horizontal_slider() -> impl Bundle {
    (
        observe(slider_self_update),
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            column_gap: Val::Px(4.0),
            height: Val::Px(12.0),
            width: Val::Px(200.0),
            ..Node::default()
        },
        Hovered::default(),
        Slider {
            track_click: TrackClick::Snap,
            ..Slider::default()
        },
        SliderValue(100.0),
        SliderRange::new(0.0, 100.0),
        children![
            (
                Node {
                    height: Val::Px(6.0),
                    border_radius: BorderRadius::all(Val::Px(3.0)),
                    ..Node::default()
                },
                BackgroundColor(SLIDER_TRACK)
            ),
            (
                Node {
                    display: Display::Flex,
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(12.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..Node::default()
                },
                children![(
                    SliderThumb,
                    Node {
                        display: Display::Flex,
                        width: Val::Px(12.0),
                        height: Val::Px(12.0),
                        position_type: PositionType::Absolute,
                        left: Val::Percent(0.0),
                        border_radius: BorderRadius::MAX,
                        ..Node::default()
                    },
                    BackgroundColor(SLIDER_THUMB),
                )],
            )
        ],
    )
}
