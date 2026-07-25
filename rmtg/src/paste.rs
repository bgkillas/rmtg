use crate::CARD_HEIGHT;
use crate::app::{Client, Runtime};
use crate::assets::Asset;
use crate::deck::Pile;
use crate::events::clipboard::{ClipboardData, ClipboardEvent, GetClipboard, GotClipboard};
use crate::keybinds::{Keybind, Keybinds};
use bevy::image::Image;
use bevy::prelude::{Commands, On, Res, ResMut, Resource, Transform};
use importer::card::SubCard;
use importer::scryfall::Quality;
use importer::tokio::task::JoinHandle;
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
    mut polls: ResMut<PollCardSpawn>,
) {
    if !matches!(clipboard.event, ClipboardEvent::CardSpawn) {
        return;
    }
    let ClipboardData::Text(str) = &clipboard.data else {
        return;
    };
    if let Ok(uuid) = Uuid::from_str(str) {
        polls.uuid.push(runtime.runtime.spawn(SubCard::get(
            client.client.clone(),
            uuid,
            Quality::Png,
        )));
    } else if let Some(rest) = str.strip_prefix("https://scryfall.com/card/")
        && let Some((set, after)) = rest.split_once('/')
        && let Some((cn_str, _)) = after.split_once('/')
        && let Ok(cn) = cn_str.parse()
    {
        polls
            .set
            .push(runtime.runtime.spawn(SubCard::get_set_cn_owned(
                client.client.clone(),
                set.to_owned(),
                cn,
                Quality::Png,
            )));
    } else if let Some(rest) = str.strip_prefix("https://scryfall.com/card/")
        && let Ok(uuid) = Uuid::from_str(rest)
    {
        polls.uuid.push(runtime.runtime.spawn(SubCard::get(
            client.client.clone(),
            uuid,
            Quality::Png,
        )));
    }
}
pub fn poll_paste_card(
    runtime: Res<Runtime>,
    mut asset: Asset,
    mut commands: Commands,
    mut polls: ResMut<PollCardSpawn>,
) {
    for i in (0..polls.set.len()).rev() {
        if polls.set[i].is_finished()
            && let Ok(Ok((mut card, front, back))) = runtime.runtime.block_on(polls.set.remove(i))
        {
            asset.register(&mut card, front, back);
            commands.spawn((
                Transform::from_xyz(0.0, CARD_HEIGHT, 0.0),
                Pile::from(card).bundle(&mut asset),
            ));
        }
    }
    for i in (0..polls.uuid.len()).rev() {
        if polls.uuid[i].is_finished()
            && let Ok(Ok((mut card, front, back))) = runtime.runtime.block_on(polls.uuid.remove(i))
        {
            asset.register(&mut card, front, back);
            commands.spawn((
                Transform::from_xyz(0.0, CARD_HEIGHT, 0.0),
                Pile::from(card).bundle(&mut asset),
            ));
        }
    }
}
#[derive(Default, Resource)]
pub struct PollCardSpawn {
    pub uuid: Vec<JoinHandle<Result<(SubCard, Image, Option<Image>), Uuid>>>,
    pub set: Vec<JoinHandle<Result<(SubCard, Image, Option<Image>), (String, u16)>>>,
}
