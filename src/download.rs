use crate::Color;
use crate::*;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bytes::Bytes;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use image::{GenericImageView, ImageReader};
use json::JsonValue;
use std::io::Cursor;
pub fn get_from_img(bytes: Bytes, asset_server: &AssetServer) -> Option<Handle<Image>> {
    fn to_asset(bytes: Bytes, asset_server: &AssetServer) -> Option<Handle<Image>> {
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
    to_asset(bytes, asset_server)
}
pub async fn parse(
    value: &JsonValue,
    client: reqwest::Client,
    asset_server: AssetServer,
) -> Option<Card> {
    async fn get_bytes(id: &str, client: &reqwest::Client, normal: bool) -> Option<Bytes> {
        let url = if normal {
            format!("https://assets.moxfield.net/cards/card-{id}-normal.webp")
        } else {
            format!("https://assets.moxfield.net/cards/card-face-{id}-normal.webp")
        };
        let res = client.get(url).send().await.ok()?;
        res.bytes().await.ok()
    }
    let normal = value["card_faces"].members().next().is_none();
    let id = value["card_faces"]
        .members()
        .next()
        .and_then(|a| a["id"].as_str())
        .unwrap_or_else(|| value["id"].as_str().unwrap());
    let bytes = get_bytes(id, &client, normal).await?;
    let alt_bytes = if let Some(id) = value["meld_result"]["id"].as_str().or(value["card_faces"]
        .members()
        .nth(1)
        .and_then(|a| a["id"].as_str()))
    {
        get_bytes(id, &client, normal).await
    } else {
        None
    };
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
    let image = get_from_img(bytes, &asset_server)?;
    let alt_image = alt_bytes.and_then(|bytes| get_from_img(bytes, &asset_server));
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
    Some(Card {
        normal: CardInfo {
            name,
            mana_cost,
            card_type,
            text,
            color,
            power,
            toughness,
            image,
        },
        alt: alt_image.map(|image| CardInfo {
            name: alt_name.unwrap(),
            mana_cost: alt_mana_cost,
            card_type: alt_card_type,
            text: alt_text,
            color: alt_color,
            power: alt_power,
            toughness: alt_toughness,
            image,
        }),
        is_alt: false,
    })
}
pub async fn get_deck(
    url: String,
    client: reqwest::Client,
    asset_server: AssetServer,
) -> Option<Deck> {
    if let Ok(res) = client.get(url).send().await
        && let Ok(text) = res.text().await
        && let Ok(json) = json::parse(&text)
    {
        macro_rules! get {
            ($b:expr) => {
                $b.map(|p| parse(p, client.clone(), asset_server.clone()))
                    .collect::<FuturesUnordered<_>>()
                    .filter_map(async |a| a)
                    .collect::<Vec<Card>>()
                    .await
            };
        }
        let tokens = get!(json["tokens"].members());
        let board = &json["boards"];
        let main = get!(
            board["mainboard"]["cards"]
                .entries()
                .map(|(_, c)| &c["card"])
        );
        let side = get!(
            board["sideboard"]["cards"]
                .entries()
                .map(|(_, c)| &c["card"])
        );
        let commanders = get!(
            board["commanders"]["cards"]
                .entries()
                .map(|(_, c)| &c["card"])
        );
        Some(Deck {
            commanders,
            main,
            tokens,
            side,
        })
    } else {
        None
    }
}
