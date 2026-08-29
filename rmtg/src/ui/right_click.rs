use crate::FONT_WIDTH;
use crate::events::clone::{CloneObjects, PasteObjects};
use crate::events::hover::HoveredObject;
use crate::keybinds::Keybind;
use crate::spatial::Spatial;
use crate::ui::esc_menu::button;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::Event;
use bevy::ui::{BackgroundColor, Display, FlexDirection, Node, PositionType, Val};
use bevy::ui_widgets::{Activate, observe};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::On;
use bevy_ecs::query::With;
use bevy_ecs::system::{Commands, Query, Res, Single};
#[derive(Event)]
pub struct TriggerRightClickMenu {
    pub entities: Box<[Entity]>,
    pub target_pos: Vec3,
    pub screen_pos: Vec2,
}
#[derive(Event)]
pub struct RemoveRightClickMenu;
#[derive(Component)]
pub struct RightClickMenu {
    pub entities: Box<[Entity]>,
    pub target_pos: Vec3,
}
#[derive(Event)]
pub struct AddMenus;
impl TriggerRightClickMenu {
    pub fn new(entities: Box<[Entity]>, target_pos: Vec3, screen_pos: Vec2) -> Self {
        Self {
            entities,
            target_pos,
            screen_pos,
        }
    }
}
pub fn trigger_right_click_menu(
    hovered: Query<Entity, With<HoveredObject>>,
    spatial: Spatial,
    keybinds: Res<ButtonInput<Keybind>>,
    mut commands: Commands,
) {
    let Some((_, target_pos, _)) = spatial.ray() else {
        return;
    };
    if !keybinds.just_pressed(Keybind::ObjectMenu) {
        return;
    }
    commands.trigger(TriggerRightClickMenu::new(
        hovered.into_iter().collect(),
        target_pos,
        spatial.cursor.pos,
    ));
}
pub fn on_right_click(on: On<TriggerRightClickMenu>, mut commands: Commands) {
    commands.trigger(RemoveRightClickMenu);
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(on.screen_pos.x),
            top: Val::Px(on.screen_pos.y),
            width: Val::Px(16.0 * FONT_WIDTH),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..Node::default()
        },
        BackgroundColor(Color::srgba_u8(0, 0, 0, 128)),
        RightClickMenu {
            entities: on.entities.clone(),
            target_pos: on.target_pos,
        },
    ));
    commands.trigger(AddMenus);
}
pub fn remove_right_click_menus(
    _: On<RemoveRightClickMenu>,
    menu: Option<Single<Entity, With<RightClickMenu>>>,
    mut commands: Commands,
) {
    if let Some(ent) = menu {
        commands.entity(*ent).despawn();
    }
}
pub fn add_copy_menu(
    _: On<AddMenus>,
    menu: Single<Entity, With<RightClickMenu>>,
    mut commands: Commands,
) {
    let mut ent = commands.entity(*menu);
    ent.with_child((button("Copy"), observe(on_copy)));
    ent.with_child((button("Paste"), observe(on_paste)));
}
pub fn on_copy(_: On<Activate>, mut commands: Commands, menu: Single<&RightClickMenu>) {
    commands.trigger(CloneObjects {
        objects: menu.entities.clone(),
        pos: menu.target_pos,
    });
    commands.trigger(RemoveRightClickMenu);
}
pub fn on_paste(_: On<Activate>, mut commands: Commands, menu: Single<&RightClickMenu>) {
    commands.trigger(PasteObjects {
        pos: menu.target_pos,
    });
    commands.trigger(RemoveRightClickMenu);
}
