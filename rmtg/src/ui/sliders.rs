use bevy::color::{Color, Luminance as _};
use bevy::picking::hover::Hovered;
use bevy::prelude::{DragEnd, DragStart, Out, Over, Pointer};
use bevy::ui::{
    AlignItems, BackgroundColor, BorderRadius, Display, FlexDirection, JustifyContent, Node,
    PositionType, Val,
};
use bevy::ui_widgets::{
    Slider, SliderDragState, SliderRange, SliderThumb, SliderValue, TrackClick, observe,
    slider_self_update,
};
use bevy_ecs::children;
use bevy_ecs::event::EntityEvent;
use bevy_ecs::hierarchy::Children;
use bevy_ecs::lifecycle::Insert;
use bevy_ecs::observer::On;
use bevy_ecs::prelude::Bundle;
use bevy_ecs::query::With;
use bevy_ecs::system::Query;
use bevy_query_fn_macro::query_fn;
const SLIDER_TRACK: Color = Color::srgb(0.05, 0.05, 0.05);
const SLIDER_THUMB: Color = Color::srgb(0.35, 0.75, 0.35);
pub fn horizontal_slider(initial: f32, start: f32, end: f32) -> impl Bundle {
    (
        observe(slider_self_update),
        observe(update_slider_visuals::<Insert, SliderValue>),
        observe(update_slider_visuals::<Pointer<Over>, SliderValue>),
        observe(update_slider_visuals::<Pointer<Out>, SliderValue>),
        observe(update_slider_visuals::<Pointer<DragStart>, SliderValue>),
        observe(update_slider_visuals::<Pointer<DragEnd>, SliderValue>),
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Stretch,
            column_gap: Val::Px(4.0),
            height: Val::Px(32.0),
            width: Val::Percent(25.0),
            left: Val::Percent(37.5),
            ..Node::default()
        },
        Hovered::default(),
        Slider {
            track_click: TrackClick::Snap,
            ..Slider::default()
        },
        SliderValue(initial),
        SliderRange::new(start, end),
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
                    right: Val::Px(32.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..Node::default()
                },
                children![(
                    SliderThumb,
                    Node {
                        display: Display::Flex,
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
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
#[query_fn]
fn update_slider_visuals<T: EntityEvent, E: Bundle>(
    event: On<T, E>,
    sliders: Query<(&SliderValue, &SliderRange, &Hovered, &SliderDragState)>,
    children: Query<&Children>,
    mut thumbs: Query<(&mut Node, &mut BackgroundColor), With<SliderThumb>>,
) {
    let slider = sliders.get(event.event_target()).unwrap();
    for child in children.iter_descendants(event.event_target()) {
        if let Ok(mut thumb) = thumbs.get_mut(child) {
            let position = slider.slider_range.thumb_position(slider.slider_value.0) * 100.0;
            thumb.node.left = Val::Percent(position);
            let is_active = slider.hovered.0 | slider.slider_drag_state.dragging;
            thumb.background_color.0 = if is_active {
                SLIDER_THUMB.lighter(0.3)
            } else {
                SLIDER_THUMB
            };
            return;
        }
    }
}
