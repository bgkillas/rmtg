use crate::keybinds::Keybinds;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::math::Vec2;
use bevy::picking::hover::HoverMap;
use bevy::prelude::{Component, KeyCode};
use bevy::ui::{ComputedNode, Node, OverflowAxis, ScrollPosition};
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::message::{Message, MessageReader, MessageWriter, PopulatedMessageReader};
use bevy_ecs::query::With;
use bevy_ecs::system::{Query, Res};
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
pub fn scroll(
    mut messages: PopulatedMessageReader<Scroll>,
    parents: Query<&ChildOf>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode), With<Scrollable>>,
) {
    for msg in messages.read() {
        let mut entity = msg.entity;
        while !query.contains(entity) {
            entity = parents.get(entity).unwrap().0;
        }
        let (mut scroll_position, node, computed) = query.get_mut(entity).unwrap();
        let max_offset =
            (computed.content_size() - computed.size()) * computed.inverse_scale_factor();
        if node.overflow.x == OverflowAxis::Scroll && msg.delta.x != 0.0 {
            let max = if msg.delta.x > 0.0 {
                scroll_position.x >= max_offset.x
            } else {
                scroll_position.x <= 0.0
            };
            if !max {
                scroll_position.x += msg.delta.x;
                scroll_position.x = scroll_position.x.min(max_offset.x).max(0.0);
            }
        }
        if node.overflow.y == OverflowAxis::Scroll && msg.delta.y != 0.0 {
            let max = if msg.delta.y > 0.0 {
                scroll_position.y >= max_offset.y
            } else {
                scroll_position.y <= 0.0
            };
            if !max {
                scroll_position.y += msg.delta.y;
                scroll_position.y = scroll_position.y.min(max_offset.y).max(0.0);
            }
        }
    }
}
pub fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keybinds: Keybinds,
    mut scroll_messages: MessageWriter<Scroll>,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);
        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR;
        }
        if keybinds
            .keyboard
            .any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        {
            mem::swap(&mut delta.x, &mut delta.y);
        }
        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                scroll_messages.write(Scroll { entity, delta });
            }
        }
    }
}
