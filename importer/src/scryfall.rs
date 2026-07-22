use crate::card::{CardData, CardInfo, Layout, MaybeImage};
use crate::card::{Colors, Cost, SubCard, Types};
use crate::id::Id;
use crate::image::parse_bytes;
use bevy::image::Image;
use jzon::{JsonValue, parse};
use reqwest::Client;
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
static THROTTLE: LazyLock<ThrottlePool> =
    LazyLock::new(|| ThrottlePool::new(ThrottleRate::new(9, Duration::new(1, 0))));
impl SubCard {
    #[must_use]
    pub async fn get_list(
        client: Client,
        iter: &[Uuid],
        quality: Quality,
    ) -> Vec<(Self, Image, Option<Image>)> {
        let mut cards = Vec::with_capacity(iter.len());
        let mut set = JoinSet::new();
        for uuid in iter {
            set.spawn(Self::get(client.clone(), *uuid, quality));
        }
        while let Some(Ok(Some(val))) = set.join_next().await {
            cards.push(val);
        }
        cards
    }
    #[must_use]
    pub async fn get(
        client: Client,
        uuid: Uuid,
        quality: Quality,
    ) -> Option<(Self, Image, Option<Image>)> {
        async fn get_card(client: &Client, uuid: Uuid) -> Option<SubCard> {
            let _hold = THROTTLE.queue_with_hold().await;
            let request = client
                .get(format!("https://{URL}/cards/{uuid}"))
                .send()
                .await
                .ok()?;
            let json_raw = request.text().await.ok()?;
            let json = parse(&json_raw).unwrap();
            SubCard::from_scryfall(json, uuid)
        }
        async fn get_image(
            client: &Client,
            uuid: Uuid,
            quality: Quality,
            side: &str,
        ) -> Option<Image> {
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
        if let (Some(card), Some(image), back) = tokio::join!(
            get_card(&client, uuid),
            get_image(&client, uuid, quality, "front"),
            get_image(&client, uuid, quality, "back")
        ) {
            Some((card, image, back))
        } else {
            None
        }
    }
    #[must_use]
    pub fn from_scryfall(json: JsonValue, uuid: Uuid) -> Option<Self> {
        fn get_face(json: &JsonValue, face: &JsonValue) -> Option<CardInfo> {
            fn get<'a>(face: &'a JsonValue, json: &'a JsonValue, s: &str) -> &'a JsonValue {
                if face[s].is_null() {
                    &json[s]
                } else {
                    &face[s]
                }
            }
            let [name_raw, mana_cost_raw, type_line_raw, oracle_text_raw] =
                ["name", "mana_cost", "type_line", "oracle_text"]
                    .try_map(|s| get(face, json, s).as_str())?;
            let [colors, color_identity] = ["colors", "color_identity"]
                .map(|s| {
                    get(face, json, s)
                        .members()
                        .map(|c| c.as_str().unwrap_or_default())
                })
                .map(Colors::parse);
            let [power, toughness, loyalty] = ["power", "toughness", "loyalty"]
                .map(|s| get(face, json, s).as_str().and_then(|l| l.parse().ok()));
            let name = name_raw.to_owned();
            let oracle_text = oracle_text_raw.to_owned();
            let mana_cost = Cost::from(mana_cost_raw);
            let type_line = Types::from(type_line_raw);
            Some(CardInfo {
                name,
                mana_cost,
                type_line,
                oracle_text,
                colors,
                color_identity,
                power,
                toughness,
                loyalty,
                image: MaybeImage::default(),
            })
        }
        let layout_str = json["layout"].as_str()?;
        let layout = Layout::from(layout_str);
        let (front, back) = if json["card_faces"].is_null() {
            let front = get_face(&json, &JsonValue::Null)?;
            (front, None)
        } else {
            let mut members = json["card_faces"].members();
            let front = get_face(&json, members.next()?)?;
            let back = get_face(&json, members.next()?)?;
            (front, Some(Box::new(back)))
        };
        let tokens = json["all_parts"]
            .members()
            .filter(|p| p["component"].as_str() == Some("token"))
            .filter_map(|p| p["id"].as_str())
            .filter_map(|s| Uuid::from_str(s).ok())
            .map(Id::from)
            .collect();
        let data = CardData {
            front,
            back,
            layout,
        };
        Some(Self {
            id: Id::from(uuid),
            tokens,
            data,
            flipped: false,
        })
    }
}
