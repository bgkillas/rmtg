use bevy::math::Vec2;
use bevy::ui::{ComputedNode, Node, OverflowAxis, ScrollPosition};
use bevy_ecs::entity::Entity;
use bevy_ecs::message::{Message, MessageReader};
use bevy_ecs::system::Query;
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
    pub fn down(entity: Entity) -> Self {
        Self {
            entity,
            delta: Vec2::splat(f32::MAX),
        }
    }
}
pub fn scroll(
    mut messages: MessageReader<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    for msg in messages.read() {
        let Ok((mut scroll_position, node, computed)) = query.get_mut(msg.entity) else {
            return;
        };
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
