use crate::app::Client;
use crate::events::move_up::MoveUp;
use crate::events::scroll::{Scroll, Scrollable};
use crate::mat::PlayMat;
use crate::pile::Pile;
use crate::ui::esc_menu::button;
use crate::ui::menu::{Menu, SetMenu};
use crate::ui::text_box::{TextSource, TextSubmission};
use crate::{CARD_WIDTH, FONT_HEIGHT, QUALITY};
use bevy::color::Color;
use bevy::log::warn;
use bevy::prelude::{Component, PositionType, Resource, Transform, Visibility};
use bevy::reflect::Reflect;
use bevy::settings::ReflectSettingsGroup;
use bevy::settings::SettingsGroup;
use bevy::ui::{
    AlignContent, AlignItems, BackgroundColor, Display, FlexDirection, JustifyContent, Node,
    Overflow, Val,
};
use bevy::ui_widgets::{Activate, observe};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::children;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::On;
use bevy_ecs::prelude::ReflectResource;
use bevy_ecs::query::With;
use bevy_ecs::system::{Commands, In, Query, Res, ResMut, Single};
use bevy_p2p::runtime::Runtime;
use bevy_query_fn_macro::query_fn;
use bevy_reflect::prelude::ReflectDefault;
use bevy_settings::SaveSettings;
use importer::moxfield::{Boards, DeckId, MaybeBoards, MoxfieldDeck};
#[derive(Resource, SettingsGroup, Default, Reflect)]
#[reflect(Resource, SettingsGroup, Default)]
pub struct SearchedPlayer {
    pub name: String,
}
#[derive(Component)]
pub struct MoxfieldMenu;
#[derive(Component)]
pub struct MoxfieldDeckList {
    pub list: Option<Vec<MoxfieldDeck>>,
}
#[derive(Component)]
pub struct DeckNumber {
    pub number: usize,
}
impl MoxfieldMenu {
    #[must_use]
    pub fn bundle() -> impl Bundle {
        (
            Node {
                width: Val::Percent(100.0 / 3.0),
                height: Val::Percent(100.0),
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
                    TextSource::Moxfield.bundle(),
                ),
                (
                    Node {
                        left: Val::Percent(0.0),
                        position_type: PositionType::Absolute,
                        top: Val::Px(FONT_HEIGHT * 1.5),
                        width: Val::Percent(100.0),
                        bottom: Val::Percent(0.0),
                        overflow: Overflow::scroll_y(),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        align_content: AlignContent::FlexStart,
                        align_items: AlignItems::FlexStart,
                        justify_content: JustifyContent::FlexStart,
                        ..Node::default()
                    },
                    Visibility::Inherited,
                    MoxfieldDeckList { list: None },
                    Scrollable,
                )
            ],
        )
    }
}
pub fn on_moxfield_set_menu(
    on: On<SetMenu>,
    list: Single<Entity, With<MoxfieldDeckList>>,
    mut commands: Commands,
) {
    if !matches!(on.menu, Menu::Moxfield) {
        return;
    }
    commands.write_message(Scroll::up(*list));
}
pub fn submit_moxfield(
    on: On<TextSubmission>,
    client: Res<Client>,
    runtime: Res<Runtime>,
    mut last: ResMut<SearchedPlayer>,
    mut commands: Commands,
) {
    if !matches!(on.source, TextSource::Moxfield) {
        return;
    }
    last.name.clone_from(&on.string);
    commands.queue(SaveSettings::IfChanged);
    let owned_client = client.client.clone();
    let owned_str = on.string.clone();
    runtime.spawn_hook(deck_hook, async move {
        MoxfieldDeck::get_decks(&owned_client, &owned_str).await
    });
}
pub fn startup_moxfield(last: Res<SearchedPlayer>, client: Res<Client>, runtime: Res<Runtime>) {
    if last.name.is_empty() {
        return;
    }
    let owned_client = client.client.clone();
    let owned_str = last.name.clone();
    runtime.spawn_hook(deck_hook, async move {
        MoxfieldDeck::get_decks(&owned_client, &owned_str).await
    });
}
#[query_fn]
fn deck_hook(
    In(list): In<Option<Vec<MoxfieldDeck>>>,
    mut ui_list: Single<(Entity, &mut MoxfieldDeckList)>,
    mut commands: Commands,
) {
    ui_list.moxfield_deck_list.list = list;
    let mut ent = commands.entity(ui_list.entity);
    ent.despawn_children();
    ent.with_children(|parent| {
        if let Some(inner) = &ui_list.moxfield_deck_list.list {
            for (number, deck) in inner.iter().enumerate() {
                parent.spawn((
                    button(&deck.name),
                    DeckNumber { number },
                    observe(on_deck_pressed),
                ));
            }
        }
    });
}
fn on_deck_pressed(
    event: On<Activate>,
    query: Query<&DeckNumber>,
    mut list: Single<&mut MoxfieldDeckList>,
    runtime: Res<Runtime>,
    client: Res<Client>,
    mut commands: Commands,
) {
    let number = query.get(event.entity).unwrap().number;
    let deck_ref = &mut list.list.as_mut().unwrap()[number];
    match deck_ref.boards.clone() {
        MaybeBoards::None => {
            deck_ref.boards = MaybeBoards::Waiting;
            let mut deck = deck_ref.clone();
            let owned_client = client.client.clone();
            runtime.spawn_hook(on_deck_get, async move {
                match deck.get_deck(&owned_client, QUALITY).await {
                    Ok(()) => Ok((number, deck)),
                    Err(val) => Err(val),
                }
            });
        }
        MaybeBoards::Full(boards) => {
            commands.trigger(boards);
        }
        MaybeBoards::Waiting => {}
    }
}
fn on_deck_get(
    In(res): In<Result<(usize, MoxfieldDeck), DeckId>>,
    mut ui_list: Single<&mut MoxfieldDeckList>,
    mut commands: Commands,
) {
    let (number, deck) = match res {
        Ok(val) => val,
        Err(e) => {
            warn!("{e:?}");
            return;
        }
    };
    let Some(ui_decks) = &mut ui_list.list else {
        return;
    };
    if ui_decks[number].name != deck.name {
        return;
    }
    commands.trigger(deck.boards.clone().unwrap());
    ui_decks[number] = deck;
}
#[query_fn]
pub fn spawn_boards(
    boards: On<Boards>,
    mut commands: Commands,
    playmats: Query<(&PlayMat, &Transform)>,
) {
    let mut transform = *playmats
        .iter()
        .find(|p| p.play_mat.player.id == 0)
        .unwrap()
        .transform;
    let owned = boards.clone();
    if let Some(pile) = owned.commanders {
        let ent = commands.spawn((transform, Pile::new(pile).bundle())).id();
        commands.trigger(MoveUp::new(ent));
        transform.translation.x += CARD_WIDTH * 1.125;
    }
    if let Some(pile) = owned.mainboard {
        let ent = commands.spawn((transform, Pile::new(pile).bundle())).id();
        commands.trigger(MoveUp::new(ent));
    }
}
