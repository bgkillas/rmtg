use crate::card::{CardData, CardInfo, Layout};
use crate::card::{Colors, Cost, SubCard, Types};
use crate::id::Id;
use crate::image::parse_bytes;
use bevy::image::Image;
use jzon::{JsonValue, parse};
use reqwest::Client;
use std::mem;
use std::str::FromStr as _;
use std::sync::LazyLock;
use std::time::Duration;
use stream_throttle::{ThrottlePool, ThrottleRate};
use tokio::task::JoinSet;
use uuid::Uuid;
const URL: &str = "api.scryfall.com";
const CARD_URL: &str = "cards.scryfall.io";
#[derive(Clone, Copy)]
pub enum Quality {
    Small,
    Normal,
    Large,
    Png,
}
impl Quality {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Quality::Small => "small",
            Quality::Normal => "normal",
            Quality::Large => "large",
            Quality::Png => "png",
        }
    }
    #[must_use]
    pub fn extension(self) -> &'static str {
        match self {
            Quality::Small | Quality::Normal | Quality::Large => "jpg",
            Quality::Png => "png",
        }
    }
}
async fn get_image(client: &Client, uuid: Uuid, quality: Quality, side: &str) -> Option<Image> {
    let byte = uuid.as_bytes()[0];
    let request = client
        .get(format!(
            "https://{CARD_URL}/{}/{side}/{:x}/{:x}/{uuid}.{}",
            quality.name(),
            byte / 16,
            byte % 16,
            quality.extension(),
        ))
        .send()
        .await
        .ok()?;
    let bytes = request.bytes().await.ok()?;
    parse_bytes(&bytes)
}
static CARDS_THROTTLE: LazyLock<ThrottlePool> =
    LazyLock::new(|| ThrottlePool::new(ThrottleRate::new(9, Duration::new(1, 0))));
static SEARCH_THROTTLE: LazyLock<ThrottlePool> =
    LazyLock::new(|| ThrottlePool::new(ThrottleRate::new(1, Duration::new(1, 0))));
impl SubCard {
    #[must_use]
    pub fn get_list(
        client: Client,
        iter: &[Uuid],
        quality: Quality,
    ) -> JoinSet<Result<(Self, Image, Option<Image>), Uuid>> {
        JoinSet::from_iter(
            iter.iter()
                .copied()
                .map(|uuid| Self::get(client.clone(), uuid, quality)),
        )
    }
    pub async fn get_prints(
        client: Client,
        oracle: Uuid,
        quality: Quality,
    ) -> Option<JoinSet<Result<(Self, Image, Option<Image>), Uuid>>> {
        let mut set = JoinSet::new();
        for i in 1.. {
            let json_raw = {
                let _hold = SEARCH_THROTTLE.queue_with_hold().await;
                let request = client
                    .get(format!(
                        "https://{URL}/cards/search?q=oracleid%3A{oracle}+game%3Apaper&unique=prints&page={i}"
                    ))
                    .send()
                    .await
                    .ok()?;
                request.text().await.ok()?
            };
            let mut json = parse(&json_raw).ok()?;
            for card_json in json["data"].as_array_mut()? {
                set.spawn(Self::get_json(
                    client.clone(),
                    mem::replace(card_json, JsonValue::Null),
                    quality,
                ));
            }
            if !json["has_more"].as_bool()? {
                break;
            }
        }
        Some(set)
    }
    pub async fn get_json(
        client: Client,
        json: JsonValue,
        quality: Quality,
    ) -> Result<(Self, Image, Option<Image>), Uuid> {
        let uuid = Uuid::parse_str(json["id"].as_str().unwrap_or_default())
            .ok()
            .unwrap_or_default();
        if let Some((card, has_back)) = SubCard::from_scryfall(&json, uuid)
            && let Some(image) = get_image(&client, uuid, quality, "front").await
        {
            let back = if has_back {
                Some(
                    get_image(&client, uuid, quality, "back")
                        .await
                        .ok_or(uuid)?,
                )
            } else {
                None
            };
            Ok((card, image, back))
        } else {
            Err(uuid)
        }
    }
    pub async fn get(
        client: Client,
        uuid: Uuid,
        quality: Quality,
    ) -> Result<(Self, Image, Option<Image>), Uuid> {
        async fn get_card(client: &Client, uuid: Uuid) -> Option<(SubCard, bool)> {
            let json_raw = {
                let _hold = CARDS_THROTTLE.queue_with_hold().await;
                let request = client
                    .get(format!("https://{URL}/cards/{uuid}"))
                    .send()
                    .await
                    .ok()?;
                request.text().await.ok()?
            };
            let json = parse(&json_raw).ok()?;
            SubCard::from_scryfall(&json, uuid)
        }
        if let (Some((card, has_back)), Some(image)) = tokio::join!(
            get_card(&client, uuid),
            get_image(&client, uuid, quality, "front")
        ) {
            let back = if has_back {
                Some(
                    get_image(&client, uuid, quality, "back")
                        .await
                        .ok_or(uuid)?,
                )
            } else {
                None
            };
            Ok((card, image, back))
        } else {
            Err(uuid)
        }
    }
    pub async fn get_set_cn_owned(
        client: Client,
        set: String,
        cn: u16,
        quality: Quality,
    ) -> Result<(Self, Image, Option<Image>), (String, u16)> {
        Self::get_set_cn(client, &set, cn, quality).await
    }
    pub async fn get_set_cn(
        client: Client,
        set: &str,
        cn: u16,
        quality: Quality,
    ) -> Result<(Self, Image, Option<Image>), (String, u16)> {
        async fn get_card(client: &Client, set: &str, cn: u16) -> Option<(SubCard, bool)> {
            let json_raw = {
                let _hold = CARDS_THROTTLE.queue_with_hold().await;
                let request = client
                    .get(format!("https://{URL}/cards/{set}/{cn}"))
                    .send()
                    .await
                    .ok()?;
                request.text().await.ok()?
            };
            let json = parse(&json_raw).ok()?;
            let uuid = Uuid::parse_str(json["id"].as_str()?).ok()?;
            SubCard::from_scryfall(&json, uuid)
        }
        if let Some((card, has_back)) = get_card(&client, set, cn).await
            && let Some(image) = get_image(&client, card.id.id, quality, "front").await
        {
            let back = if has_back {
                Some(
                    get_image(&client, card.id.id, quality, "back")
                        .await
                        .ok_or_else(|| (set.to_owned(), cn))?,
                )
            } else {
                None
            };
            Ok((card, image, back))
        } else {
            Err((set.to_owned(), cn))
        }
    }
    #[must_use]
    pub fn from_scryfall(json: &JsonValue, uuid: Uuid) -> Option<(Self, bool)> {
        fn get_face(json: &JsonValue, face: &JsonValue) -> Option<CardInfo> {
            fn get<'a>(face: &'a JsonValue, json: &'a JsonValue, s: &str) -> &'a JsonValue {
                if face[s].is_null() {
                    &json[s]
                } else {
                    &face[s]
                }
            }
            let oracle_id = Uuid::parse_str(get(face, json, "oracle_id").as_str()?).ok()?;
            let [name_raw, mana_cost_raw, type_line_raw, oracle_text_raw] =
                ["name", "mana_cost", "type_line", "oracle_text"]
                    .try_map(|s| get(face, json, s).as_str())?;
            let [colors, color_identity] = ["colors", "color_identity"]
                .try_map(|s| {
                    Some(
                        get(face, json, s)
                            .as_array()?
                            .iter()
                            .map(|c| c.as_str().unwrap_or_default()),
                    )
                })?
                .map(Colors::parse);
            let [power, toughness, loyalty] = ["power", "toughness", "loyalty"]
                .map(|s| get(face, json, s).as_str().and_then(|l| l.parse().ok()));
            let name = name_raw.to_owned();
            let oracle_text = oracle_text_raw.to_owned();
            let mana_cost = Cost::from(mana_cost_raw);
            let type_line = Types::from(type_line_raw);
            Some(CardInfo {
                oracle_id: Id::from(oracle_id),
                name,
                mana_cost,
                type_line,
                oracle_text,
                colors,
                color_identity,
                power,
                toughness,
                loyalty,
                handles: None,
            })
        }
        let layout_str = json["layout"].as_str()?;
        let layout = Layout::from(layout_str);
        let (front, back, has_back) = if json["card_faces"].is_null() {
            let front = get_face(json, &JsonValue::Null)?;
            (front, None, false)
        } else {
            let faces = json["card_faces"].as_array()?;
            let front = get_face(json, &faces[0])?;
            let back = get_face(json, &faces[1])?;
            (
                front,
                Some(Box::new(back)),
                !faces[1]["image_uris"].is_null(),
            )
        };
        let tokens = json["all_parts"]
            .as_array()
            .map(|v| {
                v.iter()
                    .filter(|p| p["component"].as_str() == Some("token"))
                    .filter_map(|p| p["id"].as_str())
                    .filter_map(|s| Uuid::from_str(s).ok())
                    .map(Id::from)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let data = CardData {
            front,
            back,
            layout,
        };
        Some((
            Self {
                id: Id::from(uuid),
                tokens,
                data,
                flipped: false,
            },
            has_back,
        ))
    }
}
