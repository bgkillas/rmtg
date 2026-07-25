use crate::CARD_HEIGHT;
use crate::app::{Client, Runtime};
use crate::assets::Asset;
use crate::deck::Pile;
use crate::events::clipboard::{ClipboardData, ClipboardEvent, GetClipboard, GotClipboard};
use crate::keybinds::{Keybind, Keybinds};
use bevy::image::Image;
use bevy::prelude::{Commands, On, Res, Transform};
use importer::card::SubCard;
use importer::scryfall::Quality;
use importer::uuid::Uuid;
use std::str::FromStr as _;
pub fn paste_card(keybind: Keybinds, mut commands: Commands) {
    if keybind.just_pressed(Keybind::Paste) {
        commands.trigger(GetClipboard::text(ClipboardEvent::CardSpawn));
    }
}
pub fn react_paste_card(
    clipboard: On<GotClipboard>,
    client: Res<Client>,
    runtime: Res<Runtime>,
    mut asset: Asset,
    mut commands: Commands,
) {
    if !matches!(clipboard.event, ClipboardEvent::CardSpawn) {
        return;
    }
    let ClipboardData::Text(str) = &clipboard.data else {
        return;
    };
    if let Some((mut card, front, back)) = get_card(&client, &runtime, str) {
        asset.register(&mut card, front, back);
        commands.spawn((
            Transform::from_xyz(0.0, CARD_HEIGHT, 0.0),
            Pile::from(card).bundle(&mut asset),
        ));
    }
}
fn get_card(
    client: &Client,
    runtime: &Runtime,
    str: &str,
) -> Option<(SubCard, Image, Option<Image>)> {
    if let Ok(uuid) = Uuid::from_str(str) {
        //TODO async
        runtime
            .runtime
            .block_on(SubCard::get(client.client.clone(), uuid, Quality::Png))
            .ok()
    } else if let Some(rest) = str.strip_prefix("https://scryfall.com/card/")
        && let Some((set, after)) = rest.split_once('/')
        && let Some((cn_str, _)) = after.split_once('/')
        && let Ok(cn) = cn_str.parse()
    {
        //TODO async
        runtime
            .runtime
            .block_on(SubCard::get_set_cn(
                client.client.clone(),
                set,
                cn,
                Quality::Png,
            ))
            .ok()
    } else if let Some(rest) = str.strip_prefix("https://scryfall.com/card/")
        && let Ok(uuid) = Uuid::from_str(rest)
    {
        //TODO async
        runtime
            .runtime
            .block_on(SubCard::get(client.client.clone(), uuid, Quality::Png))
            .ok()
    } else {
        None
    }
}
