use crate::assets::AssetManager;
use crate::events::hover::HoveredObject;
use crate::events::move_up::MoveUp;
use crate::events::repaint::Repaint;
use crate::events::scroll::{Scroll, ScrollToContentSize, Scrollable};
use crate::focus::Hover;
use crate::keybinds::Keybind;
use crate::pile::{ImageCard, Pile};
use crate::spatial::Spatial;
use crate::ui::menu::{Menu, SetMenu};
use crate::ui::text_box::TextSource;
use crate::{FONT_HEIGHT, PLAYER};
use bevy::color::Color;
use bevy::input::ButtonInput;
use bevy::picking::hover::Hovered;
use bevy::prelude::{Component, ImageNode, Transform, Visibility};
use bevy::text::EditableText;
use bevy::ui::{
    AlignContent, AlignItems, BackgroundColor, Display, FlexDirection, FlexWrap, JustifyContent,
    Node, Overflow, PositionType, Pressed, Val,
};
use bevy::ui_widgets::{Button, observe};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::change_detection::Res;
use bevy_ecs::children;
use bevy_ecs::entity::Entity;
use bevy_ecs::event::Event;
use bevy_ecs::lifecycle::{Add, Insert, Remove};
use bevy_ecs::message::{Message, MessageWriter, PopulatedMessageReader};
use bevy_ecs::observer::On;
use bevy_ecs::prelude::Commands;
use bevy_ecs::query::{Changed, With, Without};
use bevy_ecs::system::{Local, Query, Single};
use bevy_ecs::world::EntityWorldMut;
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
#[derive(Component)]
pub struct SideHold;
#[derive(Component)]
pub struct SideMenuText;
#[derive(Component)]
pub struct SideMenuEntry {
    pub entry: usize,
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
                    SideMenuText,
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
pub fn update_side_search(
    text: Single<&EditableText, With<SideMenuText>>,
    mut commands: Commands,
    search_list: Single<(Entity, &SearchList)>,
    mut last: Local<String>,
) {
    if text.editor().raw_text() == *last {
        return;
    }
    text.editor().raw_text().clone_into(&mut last);
    if let Some(entity) = search_list.search_list.list {
        commands.trigger(NewSearch { entity });
        commands.write_message(ScrollToContentSize::new(search_list.entity));
    }
}
pub fn on_side_set_menu(
    on: On<SetMenu>,
    list: Single<Entity, With<SearchList>>,
    mut commands: Commands,
) {
    if !matches!(on.menu, Menu::Side) {
        return;
    }
    commands.write_message(Scroll::up(*list));
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
    mut search_list: Single<(Entity, &mut SearchList)>,
    piles: Query<&Pile>,
    assets: AssetManager,
    side_hold: Option<Single<&ImageCard, With<SideHold>>>,
    text: Single<&EditableText, (With<SideMenuText>, Changed<EditableText>)>,
) {
    if !matches!(*menu, Menu::Side) {
        commands.trigger(SetMenu::new(Menu::Side));
    }
    search_list.search_list.list = Some(event.entity);
    let pile = piles.get(event.entity).unwrap();
    let mut ent = commands.entity(search_list.entity);
    ent.despawn_children();
    let filter = text.editor().raw_text();
    ent.with_children(|parent| {
        for (entry, card) in pile.iter().enumerate() {
            if !card.filter(filter) {
                continue;
            }
            let mut ent = parent.spawn((
                Node {
                    width: Val::Percent(100.0 / 3.0),
                    max_width: Val::Percent(100.0 / 3.0),
                    ..Node::default()
                },
                ImageCard {
                    id: card.data.id,
                    quality: card.quality,
                    flipped: card.flipped,
                    global_id: card.global_id,
                },
                SideMenuEntry { entry },
                Button,
                Hovered::default(),
                observe(on_side_pressed),
                observe(on_side_unpressed),
                observe(on_side_hover),
                ImageNode::new(if let Some(handles) = card.face_handles() {
                    handles.image()
                } else {
                    assets.card.back_image.clone()
                }),
            ));
            if let Some(inner) = &side_hold
                && inner.global_id == card.global_id
            {
                ent.insert(Pressed);
            }
        }
    });
}
fn on_side_pressed(event: On<Add, Pressed>, mut commands: Commands) {
    commands
        .entity(event.entity)
        .insert((SideHold, BackgroundColor(PLAYER[0])));
}
fn on_side_unpressed(event: On<Remove, Pressed>, mut commands: Commands) {
    commands
        .entity(event.entity)
        .queue_silenced(|mut c: EntityWorldMut| {
            c.remove::<(SideHold, BackgroundColor)>();
        });
}
#[derive(Message)]
pub struct DelayedHoveredMessage {
    pub entity: Entity,
}
#[query_fn]
fn on_side_hover(
    event: On<Insert, Hovered>,
    hovered: Query<&Hovered>,
    mut messages: MessageWriter<DelayedHoveredMessage>,
    side_hold: Single<Entity, With<SideHold>>,
) {
    if *side_hold == event.entity || !hovered.get(event.entity).unwrap().0 {
        return;
    }
    messages.write(DelayedHoveredMessage {
        entity: event.entity,
    });
}
#[query_fn]
pub fn on_side_delayed_hover(
    mut events: PopulatedMessageReader<DelayedHoveredMessage>,
    mut piles: Query<&mut Pile>,
    entries: Query<&SideMenuEntry>,
    search_list: Single<&SearchList>,
    side_hold: Single<&SideMenuEntry, With<SideHold>>,
    mut commands: Commands,
) {
    for event in events.read() {
        let pile_entity = search_list.list.unwrap();
        let mut pile = piles.get_mut(pile_entity).unwrap();
        let to = entries.get(event.entity).unwrap().entry;
        let card = pile.remove(side_hold.entry);
        pile.insert(to, card);
        commands.trigger(Repaint::new(pile_entity));
    }
}
#[query_fn]
pub fn move_cards_out(
    side_hold: Single<&SideMenuEntry, With<SideHold>>,
    hover: Hover,
    spatial: Spatial,
    search_list: Single<&SearchList>,
    mut piles: Query<&mut Pile>,
    mut commands: Commands,
) {
    if hover.get().is_some() {
        return;
    }
    let Some((_, _, pos)) = spatial.ray() else {
        return;
    };
    let pile_entity = search_list.list.unwrap();
    let mut pile = piles.get_mut(pile_entity).unwrap();
    let card = pile.remove(side_hold.entry);
    let ent = commands
        .spawn((
            Transform::from_translation(pos),
            Pile::new(vec![card]).bundle(),
            HoveredObject { held: false },
        ))
        .id();
    commands.trigger(MoveUp::new(ent));
    if pile.is_empty() {
        commands.entity(pile_entity).despawn();
    } else {
        commands.trigger(Repaint::new(pile_entity));
    }
}
#[query_fn]
pub fn move_cards_in(
    hover: Hover,
    search_list: Single<(Entity, &SearchList)>,
    mut piles: Query<&mut Pile, Without<HoveredObject>>,
    mut commands: Commands,
    mut hovered: Single<(Entity, &mut Pile), With<HoveredObject>>,
    entries: Query<&SideMenuEntry>,
) {
    if hovered.pile.len() != 1 {
        return;
    }
    let Some(ent) = hover.get() else {
        return;
    };
    let Ok(entry) = entries.get(ent) else {
        return;
    };
    let pile_entity = search_list.search_list.list.unwrap();
    let mut pile = piles.get_mut(pile_entity).unwrap();
    let card = hovered.pile.pop();
    commands.entity(hovered.entity).despawn();
    commands.entity(search_list.entity).with_child((
        ImageCard {
            id: card.data.id,
            quality: card.quality,
            flipped: card.flipped,
            global_id: card.global_id,
        },
        SideHold,
    ));
    commands.trigger(Repaint::new(pile_entity));
    pile.insert(entry.entry, card);
}
#[query_fn]
pub fn on_remove_side_menu(
    on: On<Remove, Pile>,
    mut commands: Commands,
    search_list: Single<&SearchList>,
) {
    if search_list.list == Some(on.entity) {
        commands.trigger(SetMenu { menu: Menu::World });
    }
}
#[query_fn]
pub fn on_repaint_side_menu(
    on: On<Repaint>,
    mut commands: Commands,
    search_list: Single<(Entity, &SearchList)>,
) {
    if search_list.search_list.list == Some(on.entity) {
        commands.trigger(NewSearch { entity: on.entity });
        commands.write_message(ScrollToContentSize::new(search_list.entity));
    }
}
