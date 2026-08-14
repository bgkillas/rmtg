use crate::focus::Hover;
use bevy::input::ButtonInput;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::math::Vec2;
use bevy::prelude::{Component, KeyCode};
use bevy::ui::{ComputedNode, Node, OverflowAxis, ScrollPosition};
use bevy::ui_widgets::{ControlOrientation, Scrollbar};
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::lifecycle::Add;
use bevy_ecs::message::{Message, MessageReader, MessageWriter, PopulatedMessageReader};
use bevy_ecs::observer::On;
use bevy_ecs::query::{Or, With};
use bevy_ecs::system::{Commands, Query, Res, Single};
use bevy_query_fn_macro::query_fn;
use std::mem;
#[derive(Component)]
pub struct Scrollable;
#[derive(Message)]
pub struct Scroll {
    pub entity: Entity,
    pub delta: Vec2,
}
impl Scroll {
    #[must_use]
    pub fn new(entity: Entity, delta: Vec2) -> Self {
        Self { entity, delta }
    }
    #[must_use]
    pub fn up(entity: Entity) -> Self {
        Self {
            entity,
            delta: Vec2::splat(-f32::MAX),
        }
    }
    #[must_use]
    pub fn down(entity: Entity) -> Self {
        Self {
            entity,
            delta: Vec2::splat(f32::MAX),
        }
    }
    #[must_use]
    pub fn vertical(entity: Entity, delta: f32) -> Self {
        Self {
            entity,
            delta: Vec2::new(0.0, delta),
        }
    }
    #[must_use]
    pub fn horizontal(entity: Entity, delta: f32) -> Self {
        Self {
            entity,
            delta: Vec2::new(delta, 0.0),
        }
    }
}
#[query_fn]
pub fn scroll(
    mut messages: PopulatedMessageReader<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode), With<Scrollable>>,
) {
    for msg in messages.read() {
        let mut scrollable = query.get_mut(msg.entity).unwrap();
        let max_offset = (scrollable.computed_node.content_size()
            - scrollable.computed_node.size())
            * scrollable.computed_node.inverse_scale_factor();
        if scrollable.node.overflow.x == OverflowAxis::Scroll && msg.delta.x != 0.0 {
            let max = if msg.delta.x > 0.0 {
                scrollable.scroll_position.x >= max_offset.x
            } else {
                scrollable.scroll_position.x <= 0.0
            };
            if !max {
                scrollable.scroll_position.x += msg.delta.x;
                scrollable.scroll_position.x =
                    scrollable.scroll_position.x.min(max_offset.x).max(0.0);
            }
        }
        if scrollable.node.overflow.y == OverflowAxis::Scroll && msg.delta.y != 0.0 {
            let max = if msg.delta.y > 0.0 {
                scrollable.scroll_position.y >= max_offset.y
            } else {
                scrollable.scroll_position.y <= 0.0
            };
            if !max {
                scrollable.scroll_position.y += msg.delta.y;
                scrollable.scroll_position.y =
                    scrollable.scroll_position.y.min(max_offset.y).max(0.0);
            }
        }
    }
}
pub fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover: Hover,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut scroll_messages: MessageWriter<Scroll>,
    scrollable: Query<(), Or<(With<Scrollable>, With<Scrollbar>)>>,
    parents: Query<&ChildOf>,
    scrollbars: Query<&Scrollbar>,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);
        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR;
        }
        if keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            mem::swap(&mut delta.x, &mut delta.y);
        }
        if let Some(mut entity) = hover.get() {
            while !scrollable.contains(entity)
                && let Ok(ent) = parents.get(entity)
            {
                entity = ent.0;
            }
            if scrollable.contains(entity) {
                scroll_messages.write(Scroll { entity, delta });
            }
            if let Ok(scroll) = scrollbars.get(entity) {
                scroll_messages.write(Scroll {
                    entity: scroll.target,
                    delta,
                });
            }
        }
    }
}
#[derive(Component)]
pub struct InsertScrollbar {
    pub orientation: ControlOrientation,
    pub min_thumb_length: f32,
}
#[query_fn]
pub fn insert_scroll_bar(
    event: On<Add, InsertScrollbar>,
    scroll_bar: Single<(&ChildOf, &InsertScrollbar)>,
    children: Query<&Children>,
    is_scrollable: Query<(), With<Scrollable>>,
    mut commands: Commands,
) {
    for &child in children.get(scroll_bar.child_of.0).unwrap() {
        if is_scrollable.contains(child) {
            commands
                .entity(event.entity)
                .remove::<InsertScrollbar>()
                .insert(Scrollbar {
                    target: child,
                    orientation: scroll_bar.insert_scrollbar.orientation,
                    min_thumb_length: scroll_bar.insert_scrollbar.min_thumb_length,
                });
            return;
        }
    }
    unreachable!();
}
