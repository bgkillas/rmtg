use crate::app::Client;
use crate::assets::Asset;
use crate::events::clipboard::{ClipboardData, ClipboardEvent, GetClipboard, GotClipboard};
use crate::events::move_up::MoveUp;
use crate::keybinds::{Keybind, Keybinds};
use crate::pile::Pile;
use crate::spatial::Spatial;
use bevy::image::Image;
use bevy::math::Vec3;
use bevy::prelude::{Commands, On, Res, Transform};
use bevy_ecs::system::{In, IntoSystem, System as _};
use bevy_p2p::bevy_tokio_tasks::TokioTasksRuntime;
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
        let client_owned = client.client.clone();
        runtime.spawn_background_task(move |mut tasks| async move {
            if let Ok((card, front, back)) = SubCard::get(client_owned, uuid, Quality::Png).await {
                tasks
                    .run_on_main_thread(move |main| {
                        let mut system = IntoSystem::into_system(on_paste_card);
                        system.initialize(main.world);
                        system.run((card, front, back, pos), main.world).unwrap();
                    })
                    .await;
            }
        });
    } else if let Some(rest) = str.strip_prefix("https://scryfall.com/card/")
        && let Some((set, after)) = rest.split_once('/')
        && let Some((cn_str, _)) = after.split_once('/')
        && let Ok(cn) = cn_str.parse()
    {
        let client_owned = client.client.clone();
        let owned = set.to_owned();
        runtime.spawn_background_task(move |mut tasks| async move {
            if let Ok((card, front, back)) =
                SubCard::get_set_cn_owned(client_owned, owned, cn, Quality::Png).await
            {
                tasks
                    .run_on_main_thread(move |main| {
                        let mut system = IntoSystem::into_system(on_paste_card);
                        system.initialize(main.world);
                        system.run((card, front, back, pos), main.world).unwrap();
                    })
                    .await;
            }
        });
    } else if let Some(rest) = str.strip_prefix("https://scryfall.com/card/")
        && let Ok(uuid) = Uuid::from_str(rest)
    {
        let client_owned = client.client.clone();
        runtime.spawn_background_task(move |mut tasks| async move {
            if let Ok((card, front, back)) = SubCard::get(client_owned, uuid, Quality::Png).await {
                tasks
                    .run_on_main_thread(move |main| {
                        let mut system = IntoSystem::into_system(on_paste_card);
                        system.initialize(main.world);
                        system.run((card, front, back, pos), main.world).unwrap();
                    })
                    .await;
            }
        });
    }
}
fn on_paste_card(
    In((mut card, front, back, pos)): In<(SubCard, Image, Option<Image>, Vec3)>,
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
