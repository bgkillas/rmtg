use crate::FONT_HEIGHT;
use crate::app::Client;
use crate::ui::chat::text_node;
use crate::ui::text_box::{TextSource, TextSubmission};
use bevy::color::Color;
use bevy::prelude::{Component, Resource, Visibility};
use bevy::settings::SettingsGroup;
use bevy::ui::{BackgroundColor, Node, Val};
use bevy_ecs::bundle::Bundle;
use bevy_ecs::children;
use bevy_ecs::entity::Entity;
use bevy_ecs::observer::On;
use bevy_ecs::system::{Commands, In, Single};
use bevy_p2p::runtime::Runtime;
use bevy_query_fn_macro::query_fn;
use importer::moxfield::MoxfieldDeck;
#[derive(Resource, SettingsGroup, Default)]
pub struct SearchedPlayer {
    pub name: String,
}
#[derive(Component)]
pub struct MoxfieldMenu;
#[derive(Component)]
pub struct MoxfieldDeckList {
    pub list: Option<Vec<MoxfieldDeck>>,
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
                        top: Val::Px(FONT_HEIGHT * 1.5),
                        width: Val::Percent(100.0),
                        ..Node::default()
                    },
                    Visibility::Inherited,
                    MoxfieldDeckList { list: None }
                )
            ],
        )
    }
}
pub fn submit_moxfield(on: On<TextSubmission>, client: Client, runtime: Runtime) {
    if !matches!(on.source, TextSource::Moxfield) {
        return;
    }
    println!("{}", on.string);
    let owned_client = client.client.clone();
    let owned_str = on.string.clone();
    runtime.spawn_hook(deck_hook, async move {
        MoxfieldDeck::get_decks(&owned_client, &dbg!(owned_str)).await
    });
}
#[query_fn]
fn deck_hook(
    In(list): In<Option<Vec<MoxfieldDeck>>>,
    mut ui_list: Single<(Entity, &mut MoxfieldDeckList)>,
    mut commands: Commands,
) {
    ui_list.moxfield_deck_list.list = dbg!(list);
    let mut ent = commands.entity(ui_list.entity);
    ent.despawn_children();
    ent.with_children(|parent| {
        if let Some(inner) = &ui_list.moxfield_deck_list.list {
            for deck in inner {
                parent.spawn(text_node(deck.name.to_string()));
            }
        }
    });
}
