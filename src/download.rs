use crate::Color;
use crate::sync::SyncObject;
use crate::*;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bytes::Bytes;
use futures::StreamExt;
use futures::future::join_all;
use futures::stream::FuturesUnordered;
use image::{GenericImageView, ImageReader};
use json::JsonValue;
use json::iterators::Members;
use std::fs;
use std::io::Cursor;
pub fn get_from_img(bytes: Bytes, asset_server: &AssetServer) -> Option<Handle<Image>> {
    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    let rgba = image.to_rgba8();
    let (width, height) = image.dimensions();
    let image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba.into_raw(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    Some(asset_server.add(image))
}
pub async fn spawn_singleton(
    client: reqwest::Client,
    asset_server: AssetServer,
    get_deck: GetDeck,
    v: Vec2,
    set: String,
    cn: String,
) -> Option<()> {
    let url = format!("https://api.scryfall.com/cards/{set}/{cn}");
    let res = client.get(url).send().await.ok()?;
    let res = res.text().await.ok()?;
    let json = json::parse(&res).ok()?;
    if let Some(card) = parse(&json, &client, &asset_server, 1).await {
        get_deck
            .0
            .lock()
            .unwrap()
            .push((Pile(vec![card.0]), v, None));
    }
    None
}
pub async fn process_data(
    json: Members<'_>,
    client: reqwest::Client,
    asset_server: AssetServer,
) -> Vec<Card> {
    json.map(async |a| parse(a, &client, &asset_server, 1))
        .collect::<FuturesUnordered<_>>()
        .filter_map(async |a| a.await)
        .map(|(a, b)| vec![a; b])
        .collect::<Vec<Vec<Card>>>()
        .await
        .into_iter()
        .flatten()
        .collect()
}
pub async fn get_alts(
    id: &str,
    client: reqwest::Client,
    asset_server: AssetServer,
    get_deck: GetDeck,
    v: Vec2,
) -> Option<()> {
    let url = format!("https://api.scryfall.com/cards/{id}");
    let res = client.get(url).send().await.ok()?;
    let res = res.text().await.ok()?;
    let json = json::parse(&res).ok()?;
    let id = &json["oracle_id"];
    let url = format!(
        "https://api.scryfall.com/cards/search?order=released&q=oracleid%3A{id}&unique=prints"
    );
    let res = client.get(url).send().await.ok()?;
    let res = res.text().await.ok()?;
    let mut json = json::parse(&res).ok()?;
    let size = json["total_cards"].as_usize()?;
    let mut futures = Vec::new();
    futures.push(
        process_data(
            json["data"].members().clone(),
            client.clone(),
            asset_server.clone(),
        )
        .await,
    );
    while json["has_more"].as_bool()? {
        let url = json["next_page"].as_str()?;
        let res = client.get(url).send().await.ok()?;
        let res = res.text().await.ok()?;
        json = json::parse(&res).ok()?;
        futures
            .push(process_data(json["data"].members(), client.clone(), asset_server.clone()).await);
    }
    let mut vec = Vec::with_capacity(size);
    vec.extend(
        futures
            /*.collect::<Vec<Vec<Card>>>()
            .await*/
            .into_iter()
            .flatten(),
    );
    get_deck.0.lock().unwrap().push((Pile(vec), v, None));
    None
}
pub async fn add_images(
    mut pile: Pile,
    transform: Transform,
    id: SyncObject,
    deck: GetDeck,
    client: reqwest::Client,
    asset_server: AssetServer,
) -> Option<()> {
    join_all(pile.0.iter_mut().map(|p| async {
        let bytes = get_bytes(&p.id, &client, true);
        if let Some(c) = p.alt.as_mut() {
            let bytes = get_bytes(&p.id, &client, false);
            c.image = get_from_img(bytes.await.unwrap(), &asset_server)
                .unwrap()
                .into();
        }
        p.normal.image = get_from_img(bytes.await.unwrap(), &asset_server)
            .unwrap()
            .into();
    }))
    .await;
    let v = Vec2::new(transform.translation.x, transform.translation.z);
    deck.0.lock().unwrap().push((pile, v, Some(id)));
    None
}
async fn get_bytes(id: &str, client: &reqwest::Client, normal: bool) -> Option<Bytes> {
    let path = format!("./cache/{id}-{}.jpg", if normal { 0 } else { 1 });
    if let Ok(data) = fs::read(&path) {
        Some(data.into())
    } else {
        let url = if normal {
            format!(
                "https://cards.scryfall.io/large/front/{}/{}/{id}.jpg",
                &id[0..1],
                &id[1..2]
            )
        } else {
            format!(
                "https://cards.scryfall.io/large/back/{}/{}/{id}.jpg",
                &id[0..1],
                &id[1..2]
            )
        };
        let res = client.get(url).send().await.ok()?;
        if let Ok(bytes) = res.bytes().await {
            let _ = fs::write(path, &bytes);
            Some(bytes)
        } else {
            None
        }
    }
}
fn get<T: Default, F>(value: &JsonValue, index: &str, f: F) -> (T, T)
where
    F: Fn(&JsonValue) -> T,
{
    (
        value["card_faces"]
            .members()
            .next()
            .map(|a| f(&a[index]))
            .unwrap_or_else(|| f(&value[index])),
        value["card_faces"]
            .members()
            .nth(1)
            .map(|a| f(&a[index]))
            .unwrap_or_default(),
    )
}
pub async fn parse(
    value: &JsonValue,
    client: &reqwest::Client,
    asset_server: &AssetServer,
    n: usize,
) -> Option<(Card, usize)> {
    let double = value["card_faces"].members().next().is_some();
    let id = value["scryfall_id"]
        .as_str()
        .or_else(|| value["id"].as_str())?;
    let bytes = get_bytes(id, client, true);
    let alt_bytes = if double {
        get_bytes(id, client, false).await
    } else {
        None
    };
    let bytes = bytes.await?;
    let alt_name = value["meld_result"]["name"]
        .as_str()
        .or(value["card_faces"]
            .members()
            .nth(1)
            .and_then(|a| a["name"].as_str()))
        .map(|a| a.to_string());
    let name = value["card_faces"]
        .members()
        .next()
        .and_then(|a| a["name"].as_str())
        .unwrap_or_else(|| value["name"].as_str().unwrap())
        .to_string();
    let image = get_from_img(bytes, asset_server)?;
    let alt_image = alt_bytes.and_then(|bytes| get_from_img(bytes, asset_server));
    let (mana_cost, alt_mana_cost) = get(value, "mana_cost", |a| {
        a.as_str().unwrap_or_default().into()
    });
    let (card_type, alt_card_type) = get(value, "type_line", |a| {
        a.as_str().unwrap_or_default().into()
    });
    let (text, alt_text) = get(value, "oracle_text", |a| {
        a.as_str().unwrap_or_default().to_string()
    });
    let (color, alt_color) = get(value, "colors", |a| {
        Color::parse(a.members().map(|a| a.as_str().unwrap()))
    });
    let (power, alt_power) = get(value, "power", |a| a.as_u16().unwrap_or_default());
    let (toughness, alt_toughness) = get(value, "toughness", |a| a.as_u16().unwrap_or_default());
    Some((
        Card {
            normal: CardInfo {
                name,
                mana_cost,
                card_type,
                text,
                color,
                power,
                toughness,
                image: image.into(),
            },
            alt: alt_image.map(|image| CardInfo {
                name: alt_name.unwrap(),
                mana_cost: alt_mana_cost,
                card_type: alt_card_type,
                text: alt_text,
                color: alt_color,
                power: alt_power,
                toughness: alt_toughness,
                image: image.into(),
            }),
            id: id.to_string(),
            is_alt: false,
        },
        n,
    ))
}
pub async fn get_pile(
    iter: impl Iterator<Item = (&JsonValue, usize)>,
    client: reqwest::Client,
    asset_server: AssetServer,
    decks: GetDeck,
    v: Vec2,
) {
    let pile = iter
        .map(|(p, n)| parse(p, &client, &asset_server, n))
        .collect::<FuturesUnordered<_>>()
        .filter_map(async |a| a)
        .map(|(a, b)| vec![a; b])
        .collect::<Vec<Vec<Card>>>()
        .await
        .into_iter()
        .flatten()
        .collect();
    let mut decks = decks.0.lock().unwrap();
    decks.push((Pile(pile), v, None));
}
pub struct Exact {
    pub count: usize,
    pub cn: String,
    pub set: String,
}
pub async fn get_exact(input: Exact, client: &reqwest::Client) -> Vec<JsonValue> {
    let url = format!("https://api.scryfall.com/cards/{}/{}", input.set, input.cn);
    let t = client.get(url).send().await;
    if let Ok(res) = t
        && let Ok(text) = res.text().await
        && let Ok(json) = json::parse(&text)
    {
        vec![json; input.count]
    } else {
        Vec::new()
    }
}
pub async fn get_deck_export(
    export: Vec<Exact>,
    client: reqwest::Client,
    asset_server: AssetServer,
    decks: GetDeck,
    v: Vec2,
) {
    let pile = export
        .into_iter()
        .map(|a| get_exact(a, &client))
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<Vec<JsonValue>>>()
        .await
        .iter()
        .flatten()
        .map(|p| parse(p, &client, &asset_server, 1))
        .collect::<FuturesUnordered<_>>()
        .filter_map(async |a| a)
        .map(|(a, b)| vec![a; b])
        .collect::<Vec<Vec<Card>>>()
        .await
        .into_iter()
        .flatten()
        .collect();
    let mut decks = decks.0.lock().unwrap();
    decks.push((Pile(pile), v, None));
}
pub async fn get_deck(
    url: String,
    client: reqwest::Client,
    asset_server: AssetServer,
    decks: GetDeck,
    v: Vec2,
) {
    let t = client.get(url).send().await;
    if let Ok(res) = t
        && let Ok(text) = res.text().await
        && let Ok(json) = json::parse(&text)
    {
        let board = &json["boards"];
        let commanders = get_pile(
            board["commanders"]["cards"]
                .entries()
                .map(|(_, c)| (&c["card"], c["quantity"].as_usize().unwrap())),
            client.clone(),
            asset_server.clone(),
            decks.clone(),
            v + Vec2::new(CARD_WIDTH + 1.0, 0.0),
        );
        let main = get_pile(
            board["mainboard"]["cards"]
                .entries()
                .map(|(_, c)| (&c["card"], c["quantity"].as_usize().unwrap())),
            client.clone(),
            asset_server.clone(),
            decks.clone(),
            v,
        );
        let tokens = get_pile(
            json["tokens"]
                .members()
                .filter(|json| {
                    matches!(
                        json["layout"].as_str().unwrap_or(""),
                        "double_faced_token" | "token" | "emblem"
                    )
                })
                .map(|a| (a, 1)),
            client.clone(),
            asset_server.clone(),
            decks.clone(),
            v - Vec2::new(CARD_WIDTH + 1.0, 0.0),
        );
        let side = get_pile(
            board["sideboard"]["cards"]
                .entries()
                .map(|(_, c)| (&c["card"], c["quantity"].as_usize().unwrap())),
            client.clone(),
            asset_server.clone(),
            decks.clone(),
            v - Vec2::new(2.0 * CARD_WIDTH + 2.0, 0.0),
        );
        commanders.await;
        main.await;
        tokens.await;
        side.await;
    }
}
