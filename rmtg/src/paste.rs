use crate::app::Client;
use crate::assets::Asset;
use crate::events::clipboard::{ClipboardData, ClipboardEvent, GetClipboard, GotClipboard};
use crate::events::move_up::MoveUp;
use crate::keybinds::{Keybind, Keybinds};
use crate::pile::Pile;
use crate::spatial::Spatial;
use bevy::image::Image;
use bevy::math::Vec3;
use bevy::prelude::{Commands, On, Res, ResMut, Resource, Transform};
use bevy_p2p::bevy_tokio_tasks::TokioTasksRuntime;
use bevy_p2p::tokio::task::JoinHandle;
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
    runtime: Res<TokioTasksRuntime>,
    mut polls: ResMut<PollCardSpawn>,
    spatial: Spatial,
) {
    if !matches!(clipboard.event, ClipboardEvent::CardSpawn) {
        return;
    }
    let ClipboardData::Text(str) = &clipboard.data else {
        return;
    };
    let Some((_, pos)) = spatial.ray() else {
        return;
    };
    if let Ok(uuid) = Uuid::from_str(str) {
        let card = runtime
            .runtime()
            .spawn(SubCard::get(client.client.clone(), uuid, Quality::Png));
        polls.uuid.push((card, pos));
    } else if let Some(rest) = str.strip_prefix("https://scryfall.com/card/")
        && let Some((set, after)) = rest.split_once('/')
        && let Some((cn_str, _)) = after.split_once('/')
        && let Ok(cn) = cn_str.parse()
    {
        let card = runtime.runtime().spawn(SubCard::get_set_cn_owned(
            client.client.clone(),
            set.to_owned(),
            cn,
            Quality::Png,
        ));
        polls.set.push((card, pos));
    } else if let Some(rest) = str.strip_prefix("https://scryfall.com/card/")
        && let Ok(uuid) = Uuid::from_str(rest)
    {
        let card = runtime
            .runtime()
            .spawn(SubCard::get(client.client.clone(), uuid, Quality::Png));
        polls.uuid.push((card, pos));
    }
}
pub fn poll_paste_card(
    runtime: Res<TokioTasksRuntime>,
    mut asset: Asset,
    mut commands: Commands,
    mut polls: ResMut<PollCardSpawn>,
) {
    for i in (0..polls.set.len()).rev() {
        let pos = polls.set[i].1;
        if polls.set[i].0.is_finished()
            && let Ok(Ok((mut card, front, back))) =
                runtime.runtime().block_on(polls.set.remove(i).0)
        {
            asset.register(&mut card, front, back);
            let ent = commands
                .spawn((
                    Transform::from_translation(pos),
                    Pile::from(card).bundle(&mut asset),
                ))
                .id();
            commands.trigger(MoveUp::new(ent));
        }
    }
    for i in (0..polls.uuid.len()).rev() {
        let pos = polls.uuid[i].1;
        if polls.uuid[i].0.is_finished()
            && let Ok(Ok((mut card, front, back))) =
                runtime.runtime().block_on(polls.uuid.remove(i).0)
        {
            asset.register(&mut card, front, back);
            let ent = commands
                .spawn((
                    Transform::from_translation(pos),
                    Pile::from(card).bundle(&mut asset),
                ))
                .id();
            commands.trigger(MoveUp::new(ent));
        }
    }
}
#[derive(Default, Resource)]
pub struct PollCardSpawn {
    pub uuid: Vec<(
        JoinHandle<Result<(SubCard, Image, Option<Image>), Uuid>>,
        Vec3,
    )>,
    pub set: Vec<(
        JoinHandle<Result<(SubCard, Image, Option<Image>), (String, u16)>>,
        Vec3,
    )>,
}
