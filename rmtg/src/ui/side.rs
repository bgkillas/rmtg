use crate::FONT_HEIGHT;
use crate::assets::AssetManager;
use crate::events::repaint::Repaint;
use crate::events::scroll::Scrollable;
use crate::focus::Hover;
use crate::keybinds::Keybind;
use crate::pile::{ImageCard, PendingCards, Pile};
use crate::spatial::Spatial;
use crate::ui::menu::{Menu, SetMenu};
use crate::ui::text_box::TextSource;
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::prelude::{Component, ImageNode, Visibility};
use bevy::ui::{
    AlignContent, AlignItems, BackgroundColor, Display, FlexDirection, FlexWrap, JustifyContent,
    Node, Overflow, PositionType, Val,
};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::Res;
use bevy_ecs::children;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::observer::On;
use bevy_ecs::prelude::Commands;
use bevy_ecs::query::With;
use bevy_ecs::system::{Query, Single};
use bevy_query_fn_macro::query_fn;
#[derive(Component)]
pub struct SearchList {
    pub list: Option<Entity>,
}
#[derive(Component)]
pub struct SideMenu;
#[derive(Event)]
pub struct NewSearch {
    pub entity: Entity,
}
impl SideMenu {
    #[must_use]
    pub fn bundle() -> impl Bundle {
        (
            Node {
                width: Val::Percent(100.0 / 3.0),
                height: Val::Percent(100.0),
                left: Val::Percent(2.0 * 100.0 / 3.0),
                ..Node::default()
            },
            Self,
            Visibility::Hidden,
            BackgroundColor(Color::srgba_u8(0, 0, 0, 128)),
            children![
                (
                    Node {
                        height: Val::Px(FONT_HEIGHT * 1.5),
                        width: Val::Percent(100.0),
                        min_width: Val::Percent(100.0),
                        ..Node::default()
                    },
                    BackgroundColor(Color::srgba_u8(0, 0, 0, 32)),
                    Visibility::Inherited,
                    TextSource::Search.bundle(),
                ),
                (
                    Node {
                        left: Val::Percent(0.0),
                        position_type: PositionType::Absolute,
                        top: Val::Px(FONT_HEIGHT * 1.5),
                        bottom: Val::Percent(0.0),
                        width: Val::Percent(100.0),
                        overflow: Overflow::scroll_y(),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        align_content: AlignContent::FlexStart,
                        align_items: AlignItems::FlexStart,
                        justify_content: JustifyContent::FlexStart,
                        ..Node::default()
                    },
                    Visibility::Inherited,
                    SearchList { list: None },
                    Scrollable,
                )
            ],
        )
    }
}
#[query_fn]
pub fn activate_side_menu(
    keybinds: Res<ButtonInput<Keybind>>,
    mut commands: Commands,
    spatial: Spatial,
    hover: Hover,
    cards: Query<Entity, With<Pile>>,
) {
    if keybinds.just_pressed(Keybind::Search) && hover.get().is_none() {
        let Some((hit, _, _)) = spatial.ray() else {
            return;
        };
        if cards.contains(hit.entity) {
            commands.trigger(NewSearch { entity: hit.entity });
        }
    }
}
#[query_fn]
pub fn on_new_search(
    event: On<NewSearch>,
    menu: Res<Menu>,
    mut commands: Commands,
    search_list: Single<(Entity, &mut SearchList)>,
    piles: Query<&Pile>,
    assets: AssetManager,
) {
    if !matches!(*menu, Menu::Side) {
        commands.trigger(SetMenu::new(Menu::Side));
    }
    let pile = piles.get(event.entity).unwrap();
    let mut ent = commands.entity(search_list.entity);
    ent.despawn_children();
    ent.with_children(|parent| {
        for card in pile {
            let bundle = (
                Node {
                    width: Val::Percent(100.0 / 3.0),
                    max_width: Val::Percent(100.0 / 3.0),
                    ..Node::default()
                },
                ImageCard {
                    id: card.data.id,
                    flipped: card.flipped,
                },
            );
            if let Some(handles) = card.face_handles() {
                parent.spawn((bundle, ImageNode::new(handles.image)));
            } else {
                parent.spawn((
                    bundle,
                    PendingCards,
                    ImageNode::new(assets.card.back_image.clone()),
                ));
            }
        }
    });
}
#[query_fn]
pub fn on_repaint_side_menu(
    on: On<Repaint>,
    piles: Query<&Pile>,
    search_list: Single<(Entity, &mut SearchList)>,
) {
    let pile = piles.get(on.entity).unwrap();
}
