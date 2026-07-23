use crate::CARD_HEIGHT;
use crate::app::{Client, Runtime};
use crate::assets::Asset;
use crate::deck::Pile;
use crate::keybinds::{Keybind, Keybinds};
use bevy::clipboard::Clipboard;
use bevy::prelude::{Commands, Res, ResMut, Transform};
use importer::card::SubCard;
use importer::scryfall::Quality;
use importer::uuid::Uuid;
use std::str::FromStr as _;
pub fn paste_card(
    mut clipboard: ResMut<Clipboard>,
    keybind: Keybinds,
    client: Res<Client>,
    runtime: Res<Runtime>,
    mut asset: Asset,
    mut commands: Commands,
) {
    if keybind.just_pressed(Keybind::Paste)
        && let Some(Ok(str)) = clipboard.fetch_text().poll_result()
        && let Some((mut card, front, back)) = if let Ok(uuid) = Uuid::from_str(&str) {
            runtime
                .runtime
                .block_on(SubCard::get(client.client.clone(), uuid, Quality::Png))
                .ok()
        } else if let Some(rest) = str.strip_prefix("https://scryfall.com/card/")
            && let Some((set, after)) = rest.split_once('/')
            && let Some((cn_str, _)) = after.split_once('/')
            && let Ok(cn) = cn_str.parse()
        {
            runtime
                .runtime
                .block_on(SubCard::get_set_cn(
                    client.client.clone(),
                    set,
                    cn,
                    Quality::Png,
                ))
                .ok()
        } else {
            None
        }
    {
        asset.register(&mut card, front, back);
        commands.spawn((
            Transform::from_xyz(0.0, CARD_HEIGHT, 0.0),
            Pile::from(card).bundle(&mut asset),
        ));
    }
}
