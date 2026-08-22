use crate::card::{CardData, CardInfo, Layout, MaybeHandles};
use crate::card::{Colors, Cost, SubCard, Types};
use crate::card_cache::{
    CacheRead, CacheReadImage, CacheResult, CardCache, CardInCache, Identifier, get_images,
    write_image,
};
use crate::image::parse_bytes;
use bevy::image::Image;
use jzon::{JsonValue, parse};
use ratelimit::Ratelimiter;
use reqwest::Client;
use rustc_hash::FxBuildHasher;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::mem;
use std::str::FromStr as _;
use std::sync::Mutex;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::join;
use tokio::task::JoinSet;
#[cfg(not(target_family = "wasm"))]
use tokio::time::sleep;
#[cfg(target_family = "wasm")]
use tokio_with_wasm as tokio;
use uuid::Uuid;
#[cfg(target_family = "wasm")]
use wasmtimer::tokio::sleep;
pub static CACHE: LazyLock<Mutex<CardCache>> = LazyLock::new(|| Mutex::new(CardCache::default()));
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
async fn get_image(
    client: &Client,
    set_cn: &str,
    uuid: Uuid,
    quality: Quality,
    side: Side,
) -> Option<Image> {
    use bevy::log::warn;
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
            Ok(bytes) => {
                write_image(&bytes, set_cn, uuid, side);
                parse_bytes(&bytes)
            }
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
async fn get_image(_: &Client, _: &str, _: Uuid, _: Quality, _: Side) -> Option<Image> {
    None
}
impl CacheReadImage {
    pub async fn get_image(
        self,
        client: &Client,
        set_cn: &str,
        uuid: Uuid,
        quality: Quality,
        side: Side,
    ) -> Option<Image> {
        match self {
            CacheReadImage::Some(bytes) => {
                if let Some(image) = parse_bytes(&bytes) {
                    Some(image)
                } else {
                    get_image(client, set_cn, uuid, quality, side).await
                }
            }
            CacheReadImage::Missing => get_image(client, set_cn, uuid, quality, side).await,
            CacheReadImage::None => None,
        }
    }
}
async fn read_cards(
    client: Client,
    set_cn: Box<str>,
    uuid: Uuid,
    quality: Quality,
    front_image: CacheReadImage,
    back_image: CacheReadImage,
) {
    let (front, back) = join!(
        front_image.get_image(&client, &set_cn, uuid, quality, Side::Front),
        back_image.get_image(&client, &set_cn, uuid, quality, Side::Back)
    );
    IMAGES_TO_PROCESS
        .lock()
        .unwrap()
        .insert(uuid, (front, back));
}
async fn read_cards_check(
    client: Client,
    set_cn: Box<str>,
    uuid: Uuid,
    quality: Quality,
    mut front_image: CacheReadImage,
    mut back_image: CacheReadImage,
) {
    if let Some((front, back)) =
        get_images(&set_cn, uuid, matches!(back_image, CacheReadImage::Missing))
    {
        front_image = front;
        back_image = back;
    }
    read_cards(client, set_cn, uuid, quality, front_image, back_image).await;
}
impl From<&MaybeHandles> for CacheReadImage {
    fn from(value: &MaybeHandles) -> Self {
        match value {
            MaybeHandles::Waiting => Self::Missing,
            MaybeHandles::Some(_) | MaybeHandles::None => Self::None,
        }
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
async fn get_uuid(client: Client, uuid: Uuid, quality: Quality) -> Option<SubCard> {
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
pub static IMAGES_TO_PROCESS: LazyLock<
    Mutex<HashMap<Uuid, (Option<Image>, Option<Image>), FxBuildHasher>>,
> = LazyLock::new(|| Mutex::new(HashMap::with_capacity_and_hasher(512, FxBuildHasher)));
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
                set.push(
                    Self::get_json(
                        client.clone(),
                        mem::replace(card_json, JsonValue::Null),
                        quality,
                    )
                    .await,
                );
            }
            if !json["has_more"].as_bool()? {
                break;
            }
        }
        Some(set)
    }
    pub async fn get_json(client: Client, json: JsonValue, quality: Quality) -> Result<Self, Uuid> {
        let uuid = Uuid::parse_str(json["id"].as_str().unwrap_or_default()).unwrap_or_default();
        let res = {
            let mut cache = CACHE.lock().unwrap();
            cache.get(uuid)
        };
        Self::get_cache_result(client, res, quality, async |client, quality| {
            Self::from_scryfall(client, json, uuid, quality)
        })
        .await
        .ok_or(uuid)
    }
    pub async fn get_cache_result<F>(
        client: Client,
        cache_result: CacheResult<'_>,
        quality: Quality,
        on_none: impl FnOnce(Client, Quality) -> F,
    ) -> Option<Self>
    where
        F: Future<Output = Option<Self>>,
    {
        match cache_result {
            CacheResult::Some(card) => Some(Self {
                data: card.strong,
                face_handles: card.face_handles,
                back_handles: card.back_handles,
                flipped: false,
            }),
            CacheResult::Cached(set_cn, uuid) => {
                if let Some(read) = CacheRead::read_files(&set_cn, uuid) {
                    let data = read.strong.clone();
                    let back_handles = if matches!(read.back_image, CacheReadImage::None) {
                        MaybeHandles::None
                    } else {
                        MaybeHandles::Waiting
                    };
                    tokio::spawn(read_cards(
                        client,
                        data.set_cn.clone(),
                        uuid,
                        quality,
                        read.front_image,
                        read.back_image,
                    ));
                    let card = Self {
                        data: data.clone(),
                        face_handles: MaybeHandles::Waiting,
                        back_handles: back_handles.clone(),
                        flipped: false,
                    };
                    CACHE.lock().unwrap().insert(CardInCache {
                        strong: data,
                        face_handles: MaybeHandles::Waiting,
                        back_handles,
                    });
                    Some(card)
                } else if let Some(card) = on_none(client, quality).await {
                    Some(card)
                } else {
                    CACHE.lock().unwrap().in_progress.remove(&uuid);
                    None
                }
            }
            CacheResult::Wait(Identifier::Uuid(uuid)) => loop {
                sleep(SLEEP_TIME).await;
                let card = {
                    let cache = CACHE.lock().unwrap();
                    if cache.in_progress.contains(&uuid) {
                        continue;
                    }
                    cache.cards.get(&uuid)?.clone()
                };
                return Some(Self {
                    data: card.strong,
                    face_handles: card.face_handles,
                    back_handles: card.back_handles,
                    flipped: false,
                });
            },
            CacheResult::Wait(Identifier::SetCn(str)) => loop {
                sleep(SLEEP_TIME).await;
                let card = {
                    let cache = CACHE.lock().unwrap();
                    if cache.in_progress_set_cn.contains(str) {
                        continue;
                    }
                    let &uuid = cache.set_cn.get_by_left(str)?;
                    cache.cards.get(&uuid)?.clone()
                };
                return Some(Self {
                    data: card.strong,
                    face_handles: card.face_handles,
                    back_handles: card.back_handles,
                    flipped: false,
                });
            },
            CacheResult::None(_) if let Some(card) = on_none(client, quality).await => Some(card),
            CacheResult::None(Identifier::Uuid(uuid)) => {
                CACHE.lock().unwrap().in_progress.remove(&uuid);
                None
            }
            CacheResult::None(Identifier::SetCn(str)) => {
                CACHE.lock().unwrap().in_progress_set_cn.remove(str);
                None
            }
        }
    }
    pub async fn get(client: Client, uuid: Uuid, quality: Quality) -> Result<Self, Uuid> {
        let res = {
            let mut cache = CACHE.lock().unwrap();
            cache.get(uuid)
        };
        Self::get_cache_result(client, res, quality, async |client, quality| {
            get_uuid(client, uuid, quality).await
        })
        .await
        .ok_or(uuid)
    }
    pub async fn get_set_cn(
        client: Client,
        set_cn: &str,
        quality: Quality,
    ) -> Result<Self, Box<str>> {
        let res = {
            let mut cache = CACHE.lock().unwrap();
            cache.get_set_cn(set_cn)
        };
        Self::get_cache_result(client, res, quality, async |client, quality| {
            while CARDS_THROTTLE.try_wait().is_err() {
                sleep(SLEEP_TIME).await;
            }
            let request = client
                .get(format!("https://{URL}/cards/{set_cn}"))
                .send()
                .await
                .ok()?;
            let json_raw = request.text().await.ok()?;
            let json = parse(&json_raw).ok()?;
            let uuid = Uuid::parse_str(json["id"].as_str()?).ok()?;
            Self::from_scryfall(client, json, uuid, quality)
        })
        .await
        .ok_or_else(|| set_cn.into())
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
            let has_unique_face = face["image_uris"].is_array();
            Some(CardInfo {
                oracle_id,
                name: name.into_boxed_str(),
                mana_cost,
                type_line,
                oracle_text: oracle_text.into_boxed_str(),
                colors,
                color_identity,
                power,
                toughness,
                loyalty,
                has_unique_face,
            })
        }
        let layout_str = json["layout"].as_str()?;
        let layout = Layout::from(layout_str);
        let face_handles = MaybeHandles::Waiting;
        let mut back_handles = MaybeHandles::None;
        let (front, back) = if json["card_faces"].is_null() {
            let front = get_face(&json, &JsonValue::Null)?;
            (front, None)
        } else {
            let faces = json["card_faces"].as_array()?;
            let front = get_face(&json, &faces[0])?;
            let back = get_face(&json, &faces[1])?;
            if back.has_unique_face {
                back_handles = MaybeHandles::Waiting;
            }
            (front, Some(Box::new(back)))
        };
        let set = json["set"].as_str().unwrap();
        let cn = json["collector_number"].as_str().unwrap();
        let set_cn = format!("{set}/{cn}").into_boxed_str();
        tokio::spawn(read_cards_check(
            client,
            set_cn.clone(),
            uuid,
            quality,
            (&face_handles).into(),
            (&back_handles).into(),
        ));
        let tokens = json["all_parts"]
            .as_array()
            .map(|v| {
                v.iter()
                    .filter(|p| p["component"].as_str() == Some("token"))
                    .filter_map(|p| p["id"].as_str())
                    .filter_map(|s| Uuid::from_str(s).ok())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let data = CardData {
            id: uuid,
            set_cn,
            tokens: tokens.into(),
            front,
            back,
            layout,
        };
        let cache = CardInCache {
            strong: Arc::new(data),
            face_handles,
            back_handles,
        };
        let card = Self {
            data: cache.strong.clone(),
            face_handles: cache.face_handles.clone(),
            back_handles: cache.back_handles.clone(),
            flipped: false,
        };
        cache.write_files();
        CACHE.lock().unwrap().insert(cache);
        Some(card)
    }
}
