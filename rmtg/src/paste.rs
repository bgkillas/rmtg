use crate::QUALITY;
use crate::app::Client;
use crate::assets::Asset;
use crate::events::clipboard::{ClipboardEvent, GetClipboard};
use crate::events::move_up::MoveUp;
use crate::keybinds::{Keybind, Keybinds};
use crate::pile::Pile;
use crate::spatial::Spatial;
use bevy::image::Image;
use bevy::log::warn;
use bevy::math::Vec3;
use bevy::prelude::{Commands, Res, Transform};
use bevy_ecs::system::In;
use bevy_p2p::runtime::Runtime;
use importer::card::SubCard;
use importer::uuid::Uuid;
use std::str::FromStr as _;
pub fn paste_card(keybind: Keybinds, mut commands: Commands) {
    if keybind.just_pressed(Keybind::Paste) {
        commands.trigger(GetClipboard::text(ClipboardEvent::CardSpawn));
    }
}
pub fn react_paste_card(
    In(str): In<String>,
    client: Res<Client>,
    runtime: Res<Runtime>,
    spatial: Spatial,
) {
    let Some((_, pos, _)) = spatial.ray() else {
        return;
    };
    if let Ok(uuid) = Uuid::from_str(&str) {
        let client_owned = client.client.clone();
        runtime.spawn_hook(on_paste_card_uuid, async move {
            SubCard::get(client_owned, uuid, QUALITY)
                .await
                .map(|(c, i, b)| (c, i, b, pos))
        });
    } else if let Some(rest) = str.strip_prefix("https://scryfall.com/card/")
        && let Some((set, after)) = rest.split_once('/')
        && let Some((cn_str, _)) = after.split_once('/')
        && let Ok(cn) = cn_str.parse()
    {
        let client_owned = client.client.clone();
        let owned = set.to_owned();
        runtime.spawn_hook(on_paste_card_set, async move {
            SubCard::get_set_cn(client_owned, &owned, cn, QUALITY)
                .await
                .map(|(c, i, b)| (c, i, b, pos))
        });
    } else if let Some(rest) = str.strip_prefix("https://scryfall.com/card/")
        && let Ok(uuid) = Uuid::from_str(rest)
    {
        let client_owned = client.client.clone();
        runtime.spawn_hook(on_paste_card_uuid, async move {
            SubCard::get(client_owned, uuid, QUALITY)
                .await
                .map(|(c, i, b)| (c, i, b, pos))
        });
    }
}
fn on_paste_card_uuid(
    In(is_ok): In<Result<(SubCard, Option<Image>, Option<Image>, Vec3), Uuid>>,
    mut commands: Commands,
) {
    match is_ok {
        Ok(val) => commands.run_system_cached_with(on_paste_card, val),
        Err(e) => warn!("{e:?}"),
    }
}
fn on_paste_card_set(
    In(is_ok): In<Result<(SubCard, Option<Image>, Option<Image>, Vec3), (String, u16)>>,
    mut commands: Commands,
) {
    match is_ok {
        Ok(val) => commands.run_system_cached_with(on_paste_card, val),
        Err(e) => warn!("{e:?}"),
    }
}
fn on_paste_card(
    In((mut card, front, back, pos)): In<(SubCard, Option<Image>, Option<Image>, Vec3)>,
    mut asset: Asset,
    mut commands: Commands,
) {
    asset.register(&mut card, front, back);
    let ent = commands
        .spawn((
            Transform::from_translation(pos),
            Pile::from(card).bundle(&mut asset),
        ))
        .id();
    commands.trigger(MoveUp::new(ent));
}
