use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::lifecycle::Remove;
use bevy_ecs::observer::On;
use bevy_query_fn_macro::query_fn;
#[derive(Event)]
pub struct NewSelectDrag {
    pub source: Entity,
    pub target: Entity,
}
#[derive(Component)]
pub struct SelectDragSource {
    pub target: Entity,
}
#[derive(Component)]
pub struct SelectDragTarget {
    pub source: Entity,
}
#[query_fn]
pub fn on_select_drag_source_removed(event: On<Remove, SelectDragSource>) {
    _ = event;
}
#[query_fn]
pub fn on_select_drag_target_removed(event: On<Remove, SelectDragTarget>) {
    _ = event;
}
#[query_fn]
pub fn update_select_drags() {}
#[query_fn]
pub fn add_select_drags() {}
