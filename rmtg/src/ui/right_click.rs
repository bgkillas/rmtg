use crate::FONT_WIDTH;
use crate::events::clone::{CloneObjects, CloneObjs, PasteObjects};
use crate::events::hover::HoveredObject;
use crate::focus::Hover;
use crate::keybinds::Keybind;
use crate::pile::{PendingCards, Pile};
use crate::shapes::Shape;
use crate::spatial::Spatial;
use crate::ui::esc_menu::button;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Event, MouseButton};
use bevy::time::Time;
use bevy::ui::{BackgroundColor, Display, FlexDirection, Node, PositionType, Val};
use bevy::ui_widgets::{Activate, observe};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::observer::On;
use bevy_ecs::prelude::Without;
use bevy_ecs::query::With;
use bevy_ecs::system::{Commands, Local, Query, Res, Single};
use bevy_query_fn_macro::query_fn;
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
#[query_fn]
pub fn trigger_right_click_menu(
    hovered: Query<Entity, With<HoveredObject>>,
    spatial: Spatial,
    keybinds: Res<ButtonInput<Keybind>>,
    mouse: Res<ButtonInput<MouseButton>>,
    menu: Option<Single<Entity, With<RightClickMenu>>>,
    parent: Query<&ChildOf>,
    mut commands: Commands,
    hover: Hover,
    time: Res<Time>,
    mut local: Local<f32>,
) {
    let Some((_, target_pos, _)) = spatial.ray() else {
        return;
    };
    if keybinds.pressed(Keybind::ObjectMenu) {
        *local += time.delta_secs();
    } else if keybinds.just_released(Keybind::ObjectMenu) && *local < 0.125 {
        *local = 0.0;
        commands.trigger(TriggerRightClickMenu::new(
            hovered.into_iter().collect(),
            target_pos,
            spatial.cursor.pos,
        ));
        return;
    } else {
        *local = 0.0;
    }
    if !mouse.get_just_pressed().is_empty()
        && menu.is_some_and(|target| {
            let mut current = hover.get();
            while let Some(e) = current {
                if e == *target {
                    return false;
                }
                current = parent.get(e).ok().map(ChildOf::parent);
            }
            true
        })
    {
        commands.trigger(RemoveRightClickMenu);
    }
}
pub fn on_right_click(
    on: On<TriggerRightClickMenu>,
    mut commands: Commands,
    menu: Option<Single<(), With<RightClickMenu>>>,
) {
    if menu.is_some() {
        commands.trigger(RemoveRightClickMenu);
    }
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
    menu: Single<Entity, With<RightClickMenu>>,
    mut commands: Commands,
) {
    commands.entity(*menu).despawn();
}
#[derive(Event)]
pub struct Intermediary;
pub fn ensure_has_menus(_: On<AddMenus>, mut commands: Commands) {
    commands.trigger(Intermediary);
}
pub fn ensure_has_menus_post(
    _: On<Intermediary>,
    mut commands: Commands,
    menu: Single<&Children, With<RightClickMenu>>,
) {
    if menu.is_empty() {
        commands.trigger(RemoveRightClickMenu);
    }
}
#[query_fn]
pub fn add_copy_menu(
    _: On<AddMenus>,
    menu: Single<(Entity, &RightClickMenu)>,
    mut commands: Commands,
    clones: Res<CloneObjs>,
    hovered_entities: Query<(Option<&Shape>, Option<&Pile>), Without<PendingCards>>,
) {
    let mut ent = commands.entity(menu.entity);
    if !menu.right_click_menu.entities.is_empty()
        && menu.right_click_menu.entities.iter().all(|e| {
            hovered_entities
                .get(*e)
                .is_ok_and(|c| c.shape.is_some() || c.pile.is_some())
        })
    {
        ent.with_child((button("Copy"), observe(on_copy)));
    }
    if !clones.objects.is_empty() {
        ent.with_child((button("Paste"), observe(on_paste)));
    }
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
