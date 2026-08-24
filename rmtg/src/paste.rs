use crate::QUALITY;
use crate::app::Client;
use crate::events::move_up::MoveUp;
use crate::pile::Pile;
use crate::spatial::Spatial;
use crate::ui::text_box::{TextSource, TextSubmission};
use bevy::log::warn;
use bevy::math::Vec3;
use bevy::prelude::{Commands, Res, Transform};
use bevy_ecs::observer::On;
use bevy_ecs::system::In;
use bevy_p2p::runtime::Runtime;
use importer::card::SubCard;
use importer::uuid::Uuid;
use std::str::FromStr as _;
#[derive(Debug)]
pub enum Identifier {
    Uuid(Uuid),
    SetCn(String),
    None,
}
pub fn react_paste_card(
    event: On<TextSubmission>,
    client: Res<Client>,
    runtime: Res<Runtime>,
    spatial: Spatial,
) {
    if !matches!(event.source, TextSource::Chat) {
        return;
    }
    let Some((_, pos, _)) = spatial.ray() else {
        return;
    };
    if let Some(rest) = event.string.strip_prefix("/prints ") {
        let client_owned = client.client.clone();
        match get_identifier(rest) {
            Identifier::Uuid(uuid) => {
                runtime.spawn_hook(on_paste_card_prints_uuid, async move {
                    (
                        SubCard::get_prints_id(&client_owned, uuid, QUALITY).await,
                        pos,
                    )
                });
            }
            Identifier::SetCn(set_cn) => {
                runtime.spawn_hook(on_paste_card_prints_set, async move {
                    (
                        SubCard::get_prints_set_cn(&client_owned, &set_cn, QUALITY).await,
                        pos,
                    )
                });
            }
            Identifier::None => {
                let owned = rest.to_owned();
                runtime.spawn_hook(on_paste_card_prints_set, async move {
                    (
                        SubCard::get_prints_str(&client_owned, &owned, QUALITY).await,
                        pos,
                    )
                });
            }
        }
    } else if let Some(rest) = event.string.strip_prefix("/card ") {
        let client_owned = client.client.clone();
        match get_identifier(rest) {
            Identifier::Uuid(uuid) => {
                runtime.spawn_hook(on_paste_card_uuid, async move {
                    (SubCard::get_id(&client_owned, uuid, QUALITY).await, pos)
                });
            }
            Identifier::SetCn(set_cn) => {
                runtime.spawn_hook(on_paste_card_set, async move {
                    (
                        SubCard::get_set_cn(&client_owned, &set_cn, QUALITY).await,
                        pos,
                    )
                });
            }
            Identifier::None => {
                let owned = rest.to_owned();
                runtime.spawn_hook(on_paste_card_set, async move {
                    (SubCard::get_str(&client_owned, &owned, QUALITY).await, pos)
                });
            }
        }
    }
}
fn get_identifier(string: &str) -> Identifier {
    if let Ok(uuid) = Uuid::from_str(string) {
        Identifier::Uuid(uuid)
    } else if let Some((_, rest)) = string.split_once("scryfall.com/card/")
        && let Some((set, after)) = rest.split_once('/')
        && let Some((cn, _)) = after.split_once('/')
    {
        let set_cn = format!("{set}/{cn}");
        Identifier::SetCn(set_cn)
    } else if let Some((_, rest)) = string.split_once("scryfall.com/card/")
        && let Ok(uuid) = Uuid::from_str(rest)
    {
        Identifier::Uuid(uuid)
    } else {
        Identifier::None
    }
}
fn on_paste_card_uuid(In((is_ok, pos)): In<(Result<SubCard, Uuid>, Vec3)>, mut commands: Commands) {
    match is_ok {
        Ok(val) => commands.run_system_cached_with(on_paste_card, (val, pos)),
        Err(e) => warn!("{e:?}"),
    }
}
fn on_paste_card_set(
    In((is_ok, pos)): In<(Result<SubCard, Box<str>>, Vec3)>,
    mut commands: Commands,
) {
    match is_ok {
        Ok(val) => commands.run_system_cached_with(on_paste_card, (val, pos)),
        Err(e) => warn!("{e:?}"),
    }
}
fn on_paste_card(In((card, pos)): In<(SubCard, Vec3)>, mut commands: Commands) {
    let ent = commands
        .spawn((Transform::from_translation(pos), Pile::from(card).bundle()))
        .id();
    commands.trigger(MoveUp::new(ent));
}
fn on_paste_card_prints_uuid(
    In((is_ok, pos)): In<(Result<Vec<Result<SubCard, Uuid>>, Uuid>, Vec3)>,
    mut commands: Commands,
) {
    match is_ok {
        Ok(val) => commands.run_system_cached_with(on_paste_card_prints, (val, pos)),
        Err(e) => warn!("{e:?}"),
    }
}
fn on_paste_card_prints_set(
    In((is_ok, pos)): In<(Result<Vec<Result<SubCard, Uuid>>, Box<str>>, Vec3)>,
    mut commands: Commands,
) {
    match is_ok {
        Ok(val) => commands.run_system_cached_with(on_paste_card_prints, (val, pos)),
        Err(e) => warn!("{e:?}"),
    }
}
fn on_paste_card_prints(
    In((cards, pos)): In<(Vec<Result<SubCard, Uuid>>, Vec3)>,
    mut commands: Commands,
) {
    let pile: Vec<_> = cards
        .into_iter()
        .filter_map(|c| match c {
            Ok(v) => Some(v),
            Err(e) => {
                warn!("{e:?}");
                None
            }
        })
        .collect();
    let ent = commands
        .spawn((Transform::from_translation(pos), Pile::new(pile).bundle()))
        .id();
    commands.trigger(MoveUp::new(ent));
}
