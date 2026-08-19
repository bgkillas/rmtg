use crate::card::{CardData, CardInfo, Layout, MaybeHandles};
use crate::card::{Colors, Cost, SubCard, Types};
use crate::id::Id;
use crate::image::parse_bytes;
use bevy::image::Image;
use bevy::log::warn;
use jzon::{JsonValue, parse};
use ratelimit::Ratelimiter;
use reqwest::Client;
use std::fmt::{Display, Formatter};
use std::mem;
use std::str::FromStr as _;
use std::sync::LazyLock;
use std::sync::mpmc::{Receiver, channel};
use std::time::Duration;
use tokio::task::JoinSet;
#[cfg(not(target_family = "wasm"))]
use tokio::time::sleep;
#[cfg(target_family = "wasm")]
use tokio_with_wasm as tokio;
use uuid::Uuid;
#[cfg(target_family = "wasm")]
use wasmtimer::tokio::sleep;
const URL: &str = "api.scryfall.com";
const CARD_URL: &str = "cards.scryfall.io";
#[derive(Clone, Copy)]
pub enum Quality {
    Small,
    Normal,
    Large,
    Png,
}
pub enum Side {
    Front,
    Back,
}
impl Display for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Front => {
                write!(f, "front")
            }
            Self::Back => {
                write!(f, "back")
            }
        }
    }
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
#[cfg(not(target_family = "wasm"))]
async fn get_image(client: Client, uuid: Uuid, quality: Quality, side: Side) -> Option<Image> {
    let byte = uuid.as_bytes()[0];
    match client
        .get(format!(
            "https://{CARD_URL}/{}/{side}/{:x}/{:x}/{uuid}.{}",
            quality.name(),
            byte / 16,
            byte % 16,
            quality.extension(),
        ))
        .send()
        .await
    {
        Ok(request) => match request.bytes().await {
            Ok(bytes) => parse_bytes(&bytes),
            Err(e) => {
                warn!("{e:?}");
                None
            }
        },
        Err(e) => {
            warn!("{e:?}");
            None
        }
    }
}
#[cfg(target_family = "wasm")]
async fn get_image(_: Client, _: Uuid, _: Quality, _: Side) -> Option<Image> {
    None
}
fn get_image_receiver(
    client: Client,
    uuid: Uuid,
    quality: Quality,
    side: Side,
) -> Receiver<Option<Image>> {
    let (send, recv) = channel();
    tokio::spawn(async move {
        let _ = send.send(get_image(client, uuid, quality, side).await);
    });
    recv
}
impl CardInfo {
    pub fn set_image_receiver(&mut self, client: Client, uuid: Uuid, quality: Quality, side: Side) {
        self.handles = MaybeHandles::Waiting(get_image_receiver(client, uuid, quality, side));
    }
}
#[cfg(not(target_family = "wasm"))]
pub type Clock = ratelimit::StdClock;
#[cfg(target_family = "wasm")]
pub struct Clock {
    instant: wasmtimer::std::Instant,
}
#[cfg(target_family = "wasm")]
impl Default for Clock {
    fn default() -> Self {
        Self {
            instant: wasmtimer::std::Instant::now(),
        }
    }
}
#[cfg(target_family = "wasm")]
impl ratelimit::Clock for Clock {
    fn elapsed(&self) -> Duration {
        self.instant.elapsed()
    }
}
static CARDS_THROTTLE: LazyLock<Ratelimiter<Clock>> =
    LazyLock::new(|| Ratelimiter::with_clock(9, Clock::default()));
static SEARCH_THROTTLE: LazyLock<Ratelimiter<Clock>> =
    LazyLock::new(|| Ratelimiter::with_clock(1, Clock::default()));
const SLEEP_TIME: Duration = Duration::new(0, 1_048_576);
impl SubCard {
    #[must_use]
    pub fn get_list(
        client: Client,
        iter: &[Uuid],
        quality: Quality,
    ) -> JoinSet<Result<Self, Uuid>> {
        let mut set = JoinSet::new();
        for &uuid in iter {
            set.spawn(Self::get(client.clone(), uuid, quality));
        }
        set
    }
    pub async fn get_prints(
        client: Client,
        oracle: Uuid,
        quality: Quality,
    ) -> Option<Vec<Result<Self, Uuid>>> {
        let mut set = Vec::new();
        for i in 1.. {
            while SEARCH_THROTTLE.try_wait().is_err() {
                sleep(SLEEP_TIME).await;
            }
            let request = client
                .get(format!(
                    "https://{URL}/cards/search?q=oracleid%3D{oracle}+game%3Dpaper+unique%3Aprints"
                ))
                .query(&(("page", i),))
                .send()
                .await
                .ok()?;
            let json_raw = request.text().await.ok()?;
            let mut json = parse(&json_raw).ok()?;
            for card_json in json["data"].as_array_mut()? {
                set.push(Self::get_json(
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
    pub fn get_json(client: Client, json: JsonValue, quality: Quality) -> Result<Self, Uuid> {
        let uuid = Uuid::parse_str(json["id"].as_str().unwrap_or_default()).unwrap_or_default();
        if let Some(card) = SubCard::from_scryfall(client, json, uuid, quality) {
            Ok(card)
        } else {
            Err(uuid)
        }
    }
    pub async fn get(client: Client, uuid: Uuid, quality: Quality) -> Result<Self, Uuid> {
        async fn get_card(client: Client, uuid: Uuid, quality: Quality) -> Option<SubCard> {
            while CARDS_THROTTLE.try_wait().is_err() {
                sleep(SLEEP_TIME).await;
            }
            let request = client
                .get(format!("https://{URL}/cards/{uuid}"))
                .send()
                .await
                .ok()?;
            let json_raw = request.text().await.ok()?;
            let json = parse(&json_raw).ok()?;
            SubCard::from_scryfall(client, json, uuid, quality)
        }
        if let Some(card) = get_card(client, uuid, quality).await {
            Ok(card)
        } else {
            Err(uuid)
        }
    }
    pub async fn get_set_cn(
        client: Client,
        set: &str,
        cn: u16,
        quality: Quality,
    ) -> Result<Self, (String, u16)> {
        async fn get_card(client: Client, quality: Quality, set: &str, cn: u16) -> Option<SubCard> {
            while CARDS_THROTTLE.try_wait().is_err() {
                sleep(SLEEP_TIME).await;
            }
            let request = client
                .get(format!("https://{URL}/cards/{set}/{cn}"))
                .send()
                .await
                .ok()?;
            let json_raw = request.text().await.ok()?;
            let json = parse(&json_raw).ok()?;
            let uuid = Uuid::parse_str(json["id"].as_str()?).ok()?;
            SubCard::from_scryfall(client, json, uuid, quality)
        }
        if let Some(card) = get_card(client, quality, set, cn).await {
            Ok(card)
        } else {
            Err((set.to_owned(), cn))
        }
    }
    #[must_use]
    pub fn from_scryfall(
        client: Client,
        json: JsonValue,
        uuid: Uuid,
        quality: Quality,
    ) -> Option<Self> {
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
                handles: MaybeHandles::None,
            })
        }
        let layout_str = json["layout"].as_str()?;
        let layout = Layout::from(layout_str);
        let (front, back) = if json["card_faces"].is_null() {
            let mut front = get_face(&json, &JsonValue::Null)?;
            front.set_image_receiver(client.clone(), uuid, quality, Side::Front);
            (front, None)
        } else {
            let faces = json["card_faces"].as_array()?;
            let mut front = get_face(&json, &faces[0])?;
            let mut back = get_face(&json, &faces[1])?;
            front.set_image_receiver(client.clone(), uuid, quality, Side::Front);
            if faces[1]["image_uris"].is_array() {
                back.set_image_receiver(client, uuid, quality, Side::Back);
            }
            (front, Some(Box::new(back)))
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
        let card = Self {
            id: Id::from(uuid),
            tokens,
            data,
            flipped: false,
        };
        Some(card)
    }
}
