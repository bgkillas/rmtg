use crate::QUALITY;
use crate::app::Client;
use crate::events::move_up::MoveUp;
use crate::pile::Pile;
use crate::spatial::Spatial;
use crate::ui::chat::TextSubmission;
use bevy::log::warn;
use bevy::math::Vec3;
use bevy::prelude::{Commands, Res, Transform};
use bevy_ecs::observer::On;
use bevy_ecs::system::In;
use bevy_p2p::runtime::Runtime;
use importer::card::SubCard;
use importer::uuid::Uuid;
use std::str::FromStr as _;
pub fn react_paste_card(
    event: On<TextSubmission>,
    client: Res<Client>,
    runtime: Res<Runtime>,
    spatial: Spatial,
) {
    let Some((_, pos, _)) = spatial.ray() else {
        return;
    };
    if let Ok(uuid) = Uuid::from_str(&event.string) {
        let client_owned = client.client.clone();
        runtime.spawn_hook(on_paste_card_uuid, async move {
            SubCard::get(client_owned, uuid, QUALITY)
                .await
                .map(|c| (c, pos))
        });
    } else if let Some((_, rest)) = event.string.split_once("scryfall.com/card/")
        && let Some((set, after)) = rest.split_once('/')
        && let Some((cn, _)) = after.split_once('/')
    {
        let client_owned = client.client.clone();
        let set_cn = format!("{set}/{cn}");
        runtime.spawn_hook(on_paste_card_set, async move {
            SubCard::get_set_cn(client_owned, &set_cn, QUALITY)
                .await
                .map(|c| (c, pos))
        });
    } else if let Some((_, rest)) = event.string.split_once("scryfall.com/card/")
        && let Ok(uuid) = Uuid::from_str(rest)
    {
        let client_owned = client.client.clone();
        runtime.spawn_hook(on_paste_card_uuid, async move {
            SubCard::get(client_owned, uuid, QUALITY)
                .await
                .map(|c| (c, pos))
        });
    }
}
fn on_paste_card_uuid(In(is_ok): In<Result<(SubCard, Vec3), Uuid>>, mut commands: Commands) {
    match is_ok {
        Ok(val) => commands.run_system_cached_with(on_paste_card, val),
        Err(e) => warn!("{e:?}"),
    }
}
fn on_paste_card_set(In(is_ok): In<Result<(SubCard, Vec3), Box<str>>>, mut commands: Commands) {
    match is_ok {
        Ok(val) => commands.run_system_cached_with(on_paste_card, val),
        Err(e) => warn!("{e:?}"),
    }
}
fn on_paste_card(In((card, pos)): In<(SubCard, Vec3)>, mut commands: Commands) {
    let ent = commands
        .spawn((Transform::from_translation(pos), Pile::from(card).bundle()))
        .id();
    commands.trigger(MoveUp::new(ent));
}
