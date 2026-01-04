use crate::Color;
use crate::sync::SyncObject;
use crate::*;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_mod_mipmap_generator::{MipmapGeneratorSettings, generate_mips_texture};
#[cfg(not(feature = "wasm"))]
use bitcode::{decode, encode};
use futures::StreamExt;
use futures::future::join_all;
use futures::stream::{FuturesOrdered, FuturesUnordered};
use image::imageops::FilterType;
use image::{GenericImageView, ImageReader};
use json::JsonValue;
#[cfg(not(feature = "wasm"))]
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use std::fs;
use std::io::Cursor;
const QUALITY: &str = "large";
pub fn parse_no_mips(bytes: &[u8]) -> Option<Image> {
    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    let rgba = image.to_rgba8();
    let (width, height) = image.dimensions();
    Some(make_img(rgba.into_raw(), width, height))
}
pub fn make_img(rgba: Vec<u8>, width: u32, height: u32) -> Image {
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
pub fn parse_bytes(bytes: &[u8]) -> Option<Image> {
    let mut image = parse_no_mips(bytes)?;
    generate_mips(&mut image);
    Some(image)
}
pub fn generate_mips(image: &mut Image) {
    generate_mips_texture(
        image,
        &MipmapGeneratorSettings {
            anisotropic_filtering: 1,
            filter_type: FilterType::Lanczos3,
            minimum_mip_resolution: 64,
            ..default()
        },
        &mut 0,
    )
    .unwrap();
}
pub fn get_from_img(bytes: &[u8], asset_server: &AssetServer) -> Option<Handle<Image>> {
    let image = parse_bytes_mip(bytes)?;
    Some(asset_server.add(image))
}
pub async fn get_by_id(
    client: &reqwest::Client,
    asset_server: &AssetServer,
    url: String,
    cull: bool,
) -> Option<SubCard> {
    let res = client.get(url).send().await.ok()?;
    let res = res.text().await.ok()?;
    let json = json::parse(&res).ok()?;
    let layout = json["layout"].as_str();
    if cull
        && let Some(layout) = layout
        && !matches!(layout, "double_faced_token" | "token" | "emblem")
    {
        return None;
    }
    parse_scryfall(&json, client, asset_server).await
}
pub async fn spawn_singleton_id(
    client: reqwest::Client,
    asset_server: AssetServer,
    get_deck: GetDeck,
    v: Vec2,
    id: &str,
) -> Option<()> {
    if let Some(card) = get_by_id(
        &client,
        &asset_server,
        format!("https://api.scryfall.com/cards/{id}"),
        false,
    )
    .await
    {
        get_deck
            .0
            .lock()
            .unwrap()
            .push((Pile::Single(card.into()), DeckType::Single(v)));
    }
    None
}
pub async fn spawn_scryfall_list(
    ids: Vec<Id>,
    client: reqwest::Client,
    asset_server: AssetServer,
    get_deck: GetDeck,
    v: Vec2,
) -> Option<()> {
    let cards: Vec<SubCard> = ids
        .into_iter()
        .map(|id| {
            get_by_id(
                &client,
                &asset_server,
                format!("https://api.scryfall.com/cards/{id}"),
                true,
            )
        })
        .collect::<FuturesUnordered<_>>()
        .filter_map(async |a| a)
        .collect()
        .await;
    if !cards.is_empty() {
        get_deck
            .0
            .lock()
            .unwrap()
            .push((Pile::new(cards), DeckType::Single(v)));
    }
    None
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
    if let Some(card) = parse_scryfall(&json, &client, &asset_server).await {
        get_deck
            .0
            .lock()
            .unwrap()
            .push((Pile::Single(card.into()), DeckType::Single(v)));
    }
    None
}
pub async fn process_data(
    json: Vec<JsonValue>,
    client: &reqwest::Client,
    asset_server: &AssetServer,
) -> Vec<SubCard> {
    json.iter()
        .map(async |a| parse_scryfall(a, client, asset_server))
        .collect::<FuturesUnordered<_>>()
        .filter_map(async |a| a.await)
        .collect::<Vec<SubCard>>()
        .await
        .into_iter()
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
    loop {
        futures.push(process_data(
            json["data"].members().cloned().collect(),
            &client,
            &asset_server,
        ));
        if !json["has_more"].as_bool()? {
            break;
        }
        let url = json["next_page"].as_str()?;
        let res = client.get(url).send().await.ok()?;
        let res = res.text().await.ok()?;
        json = json::parse(&res).ok()?;
    }
    let mut vec = Vec::with_capacity(size);
    vec.extend(
        futures
            .into_iter()
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<Vec<SubCard>>>()
            .await
            .into_iter()
            .flatten(),
    );
    if !vec.is_empty() {
        get_deck
            .0
            .lock()
            .unwrap()
            .push((Pile::new(vec), DeckType::Single(v)));
    }
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
    join_all(pile.iter_mut().map(|p| async {
        let sid = p.data.id.to_string();
        let bytes = get_bytes(&sid, &client, &asset_server, true);
        if let Some(c) = p.data.back.as_mut() {
            let bytes = get_bytes(&sid, &client, &asset_server, false);
            c.image = bytes.await.unwrap()
        }
        p.data.face.image = bytes.await.unwrap()
    }))
    .await;
    if !pile.is_empty() {
        deck.0
            .lock()
            .unwrap()
            .push((pile, DeckType::Other(transform, id)));
    }
    None
}
#[derive(Encode, Decode)]
pub struct ImageData {
    data: Vec<u8>,
    mip: u32,
    width: u32,
    height: u32,
}
#[cfg(not(feature = "wasm"))]
impl ImageData {
    pub fn decode(data: &[u8]) -> Option<Self> {
        let data = decompress_size_prepended(data).ok()?;
        decode::<ImageData>(&data).ok()
    }
    pub fn encode(&self) -> Vec<u8> {
        compress_prepend_size(&encode(self))
    }
}
pub fn parse_bytes_mip(data: &[u8]) -> Option<Image> {
    let image_data = ImageData::decode(data)?;
    let mut image = Image::new_uninit(
        Extent3d {
            width: image_data.width,
            height: image_data.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.data = Some(image_data.data);
    image.texture_descriptor.mip_level_count = image_data.mip;
    Some(image)
}
#[cfg(not(feature = "wasm"))]
pub fn write_file(path: &str, image: &mut Image) {
    let image_data = ImageData {
        data: mem::take(&mut image.data).unwrap(),
        mip: image.texture_descriptor.mip_level_count,
        width: image.width(),
        height: image.height(),
    };
    let _ = fs::write(path, image_data.encode());
    image.data = Some(image_data.data);
}
async fn get_bytes(
    id: &str,
    client: &reqwest::Client,
    asset_server: &AssetServer,
    normal: bool,
) -> Option<UninitImage> {
    let path = format!("./cache/{id}-{}", if normal { 0 } else { 1 });
    if !cfg!(feature = "wasm")
        && let Ok(data) = fs::read(&path)
    {
        #[cfg(not(feature = "wasm"))]
        {
            if let Some(image) = parse_bytes_mip(&data) {
                return Some(asset_server.add(image).into());
            }
            let _ = fs::remove_file(&path);
        }
    }
    let url = if normal {
        format!(
            "https://cards.scryfall.io/{QUALITY}/front/{}/{}/{id}.jpg",
            &id[0..1],
            &id[1..2]
        )
    } else {
        format!(
            "https://cards.scryfall.io/{QUALITY}/back/{}/{}/{id}.jpg",
            &id[0..1],
            &id[1..2]
        )
    };
    #[cfg(feature = "wasm")]
    let (bytes, w, h) = get_image_bytes(&url).await?.into();
    #[cfg(feature = "wasm")]
    let mut image = make_img(bytes, w, h);
    #[cfg(feature = "wasm")]
    generate_mips(&mut image);
    #[cfg(not(feature = "wasm"))]
    let bytes = client.get(url).send().await.ok()?.bytes().await.ok()?;
    #[cfg(not(feature = "wasm"))]
    let mut image = parse_bytes(&bytes)?;
    #[cfg(not(feature = "wasm"))]
    write_file(&path, &mut image);
    Some(asset_server.add(image).into())
}
fn get<T: Default, F>(value: &JsonValue, index: &str, double: bool, f: F) -> (T, T)
where
    F: Fn(&JsonValue) -> T,
{
    if double {
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
    } else {
        (f(&value[index]), Default::default())
    }
}
pub async fn parse(
    id: &str,
    tokens: Vec<Id>,
    back: Option<CardInfo>,
    value: &JsonValue,
    client: &reqwest::Client,
    asset_server: &AssetServer,
) -> Option<SubCard> {
    let layout = value["layout"].as_str();
    let double = value["card_faces"].members().next().is_some();
    let bytes = get_bytes(id, client, asset_server, true);
    let layout = layout == Some("flip");
    let mut alt_image = if double && !layout {
        get_bytes(id, client, asset_server, false).await
    } else {
        None
    };
    let image = bytes.await?;
    if layout {
        alt_image = Some(UninitImage::default());
    }
    let alt_name = value["card_faces"]
        .members()
        .nth(1)
        .and_then(|a| a["name"].as_str())
        .map(|a| a.to_string())
        .unwrap_or_default();
    let name = value["card_faces"]
        .members()
        .next()
        .and_then(|a| a["name"].as_str())
        .unwrap_or_else(|| value["name"].as_str().unwrap())
        .to_string();
    let id: Id = id.parse().unwrap();
    let (mana_cost, alt_mana_cost) = get(value, "mana_cost", double, |a| {
        a.as_str().unwrap_or_default().into()
    });
    let (card_type, alt_card_type): (Types, Types) = get(value, "type_line", double, |a| {
        a.as_str().unwrap_or_default().parse().unwrap()
    });
    let layout = if layout {
        Layout::Flip
    } else if card_type.sub_type.contains(&SubType::Room) {
        Layout::Room
    } else {
        Layout::Normal
    };
    let (text, alt_text) = get(value, "oracle_text", double, |a| {
        a.as_str().unwrap_or_default().to_string()
    });
    let (color, alt_color) = get(value, "colors", double, |a| {
        Color::parse(a.members().map(|a| a.as_str().unwrap()))
    });
    let (power, alt_power) = get(value, "power", double, |a| a.as_u16().unwrap_or_default());
    let (toughness, alt_toughness) = get(value, "toughness", double, |a| {
        a.as_u16().unwrap_or_default()
    });
    Some(SubCard {
        data: CardData {
            face: CardInfo {
                name,
                mana_cost,
                card_type,
                text,
                color,
                power,
                toughness,
                image,
            },
            back: back.or(alt_image.map(|image| CardInfo {
                name: alt_name,
                mana_cost: alt_mana_cost,
                card_type: alt_card_type,
                text: alt_text,
                color: alt_color,
                power: alt_power,
                toughness: alt_toughness,
                image,
            })),
            id,
            tokens,
            layout,
        },
        flipped: false,
    })
}
pub async fn parse_moxfield(
    value: &JsonValue,
    client: &reqwest::Client,
    asset_server: &AssetServer,
    base: &JsonValue,
) -> Option<SubCard> {
    let id = value["uniqueCardId"].as_str()?;
    let tokens = &base["cardsToTokens"][id];
    let tokens = tokens
        .members()
        .filter_map(|json| {
            let id = json.as_str().unwrap();
            let inner = &base["tokenMappings"][id];
            if matches!(
                inner["layout"].as_str().unwrap_or(""),
                "double_faced_token" | "token" | "emblem"
            ) {
                inner["scryfall_id"].as_str().unwrap().parse().ok()
            } else {
                None
            }
        })
        .collect();
    if !value["meld_result"].is_null() {
        let id = value["scryfall_id"].as_str().unwrap();
        let mut c = get_by_id(
            client,
            asset_server,
            format!("https://api.scryfall.com/cards/{id}"),
            false,
        )
        .await?;
        c.data.tokens = tokens;
        Some(c)
    } else {
        parse(
            value["scryfall_id"].as_str()?,
            tokens,
            None,
            value,
            client,
            asset_server,
        )
        .await
    }
}
fn get_tokens(value: &JsonValue, id: Id, name: &str) -> Vec<Id> {
    value["all_parts"]
        .members()
        .filter_map(|a| {
            let n = a["name"].as_str().unwrap();
            let sid = a["id"].as_str().unwrap().parse();
            if let Ok(sid) = sid
                && sid != id
                && n != name
                && a["type_line"].as_str().unwrap() != "Card"
                && !matches!(
                    a["component"].as_str().unwrap(),
                    "meld_part" | "meld_result"
                )
            {
                Some(sid)
            } else {
                None
            }
        })
        .collect()
}
pub async fn parse_scryfall(
    value: &JsonValue,
    client: &reqwest::Client,
    asset_server: &AssetServer,
) -> Option<SubCard> {
    let id = value["id"].as_str()?;
    let tokens = get_tokens(value, id.parse().unwrap(), value["name"].as_str()?);
    let back = if let Some(json) = value["all_parts"]
        .members()
        .find(|a| a["component"].as_str().unwrap() == "meld_result")
    {
        let id = json["id"].as_str()?;
        let url = format!("https://api.scryfall.com/cards/{id}");
        let res = client.get(url).send().await.ok()?;
        let res = res.text().await.ok()?;
        let json = json::parse(&res).ok()?;
        let tokens = get_tokens(&json, id.parse().unwrap(), json["name"].as_str()?);
        let meld = parse(id, tokens, None, &json, client, asset_server).await?;
        Some(meld.data.face)
    } else {
        None
    };
    parse(id, tokens, back, value, client, asset_server).await
}
async fn parse_scryfall_count(
    value: &JsonValue,
    client: &reqwest::Client,
    asset_server: &AssetServer,
    n: usize,
) -> Option<(SubCard, usize)> {
    parse_scryfall(value, client, asset_server)
        .await
        .map(|a| (a, n))
}
async fn parse_count(
    value: &JsonValue,
    client: &reqwest::Client,
    asset_server: &AssetServer,
    base: &JsonValue,
    n: usize,
) -> Option<(SubCard, usize)> {
    parse_moxfield(value, client, asset_server, base)
        .await
        .map(|a| (a, n))
}
pub async fn get_pile(
    value: &JsonValue,
    client: &reqwest::Client,
    asset_server: &AssetServer,
    decks: &GetDeck,
    base: &JsonValue,
    deck_type: DeckType,
) {
    let iter = value
        .entries()
        .map(|(_, c)| (&c["card"], c["quantity"].as_usize().unwrap()));
    let pile: Vec<SubCard> = iter
        .map(|(p, n)| parse_count(p, client, asset_server, base, n))
        .collect::<FuturesUnordered<_>>()
        .filter_map(async |a| a)
        .map(|(a, n)| vec![a; n])
        .collect::<Vec<Vec<SubCard>>>()
        .await
        .into_iter()
        .flatten()
        .collect();
    if !pile.is_empty() {
        let mut decks = decks.0.lock().unwrap();
        decks.push((Pile::new(pile), deck_type));
    }
}
pub async fn get_pile_ordered(
    value: &JsonValue,
    client: &reqwest::Client,
    asset_server: &AssetServer,
    decks: &GetDeck,
    base: &JsonValue,
    deck_type: DeckType,
) {
    let iter = value
        .entries()
        .map(|(_, c)| (&c["card"], c["quantity"].as_usize().unwrap()));
    let mut pile: Vec<SubCard> = iter
        .map(|(p, n)| parse_count(p, client, asset_server, base, n))
        .collect::<FuturesOrdered<_>>()
        .filter_map(async |a| a)
        .map(|(a, n)| vec![a; n])
        .collect::<Vec<Vec<SubCard>>>()
        .await
        .into_iter()
        .flatten()
        .collect();
    if !pile.is_empty() {
        let mut decks = decks.0.lock().unwrap();
        if matches!(deck_type, DeckType::Commander) && pile.len() == 2 {
            decks.push((
                Pile::Single(pile.pop().unwrap().into()),
                DeckType::CommanderAlt,
            ));
        }
        decks.push((Pile::new(pile), deck_type));
    }
}
pub struct Exact {
    pub count: usize,
    pub cn: String,
    pub set: String,
}
pub async fn get_exact(input: Exact, client: &reqwest::Client) -> Option<(JsonValue, usize)> {
    let url = format!("https://api.scryfall.com/cards/{}/{}", input.set, input.cn);
    let t = client.get(url).send().await;
    if let Ok(res) = t
        && let Ok(text) = res.text().await
        && let Ok(json) = json::parse(&text)
    {
        Some((json, input.count))
    } else {
        None
    }
}
pub async fn get_deck_export(
    export: Vec<Exact>,
    client: reqwest::Client,
    asset_server: AssetServer,
    decks: GetDeck,
    v: Vec2,
) {
    let pile: Vec<SubCard> = export
        .into_iter()
        .map(|a| get_exact(a, &client))
        .collect::<FuturesUnordered<_>>()
        .filter_map(async |a| a)
        .collect::<Vec<(JsonValue, usize)>>()
        .await
        .iter()
        .map(|(p, n)| parse_scryfall_count(p, &client, &asset_server, *n))
        .collect::<FuturesUnordered<_>>()
        .filter_map(async |a| a)
        .map(|(a, n)| vec![a; n])
        .collect::<Vec<Vec<SubCard>>>()
        .await
        .into_iter()
        .flatten()
        .collect();
    if !pile.is_empty() {
        let mut decks = decks.0.lock().unwrap();
        decks.push((Pile::new(pile), DeckType::Single(v)));
    }
}
pub async fn get_deck(
    url: String,
    client: reqwest::Client,
    asset_server: AssetServer,
    decks: GetDeck,
) {
    let t = client.get(url).send().await;
    if let Ok(res) = t
        && let Ok(text) = res.text().await
        && let Ok(json) = json::parse(&text)
    {
        let board = &json["boards"];
        let commanders = get_pile_ordered(
            &board["commanders"]["cards"],
            &client,
            &asset_server,
            &decks,
            &json,
            DeckType::Commander,
        );
        let main = get_pile(
            &board["mainboard"]["cards"],
            &client,
            &asset_server,
            &decks,
            &json,
            DeckType::Deck,
        );
        let side = get_pile(
            &board["sideboard"]["cards"],
            &client,
            &asset_server,
            &decks,
            &json,
            DeckType::SideBoard,
        );
        let comp = get_pile(
            &board["companions"]["cards"],
            &client,
            &asset_server,
            &decks,
            &json,
            DeckType::Companion,
        );
        let spell = get_pile(
            &board["signatureSpells"]["cards"],
            &client,
            &asset_server,
            &decks,
            &json,
            DeckType::CommanderAlt,
        );
        let attractions = get_pile(
            &board["attractions"]["cards"],
            &client,
            &asset_server,
            &decks,
            &json,
            DeckType::Attraction,
        );
        let stickers = get_pile(
            &board["stickers"]["cards"],
            &client,
            &asset_server,
            &decks,
            &json,
            DeckType::Sticker,
        );
        commanders.await;
        comp.await;
        spell.await;
        stickers.await;
        attractions.await;
        side.await;
        main.await;
    }
}
