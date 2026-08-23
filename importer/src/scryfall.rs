use crate::card::{CardData, CardInfo, Layout, MaybeHandles, SubCard, SubCardInner};
use crate::card::{Colors, Cost, Types};
use crate::card_cache::{
    CacheReadImage, CacheResult, CardCache, Identifier, get_images, write_image,
};
use crate::image::parse_bytes;
use bevy::image::Image;
use bevy::log::warn;
use futures::future::join_all;
use jzon::{JsonValue, parse};
use ratelimit::Ratelimiter;
use reqwest::Client;
use rustc_hash::FxBuildHasher;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr as _;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::join;
use tokio::sync::{Mutex, Semaphore};
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
fn warn_if<T, E: Debug>(val: Result<T, E>) -> Option<T> {
    match val {
        Ok(v) => Some(v),
        Err(e) => {
            warn!("{e:?}");
            None
        }
    }
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
pub async fn throttled_parse_bytes(bytes: &[u8]) -> Option<Image> {
    static THROTTLE: LazyLock<Semaphore> = LazyLock::new(|| {
        let cpus = 1;
        Semaphore::new(cpus)
    });
    let lock = THROTTLE.acquire().await.unwrap();
    let res = parse_bytes(bytes);
    drop(lock);
    res
}
#[cfg(not(target_family = "wasm"))]
async fn get_image(
    client: &Client,
    set_cn: &str,
    uuid: Uuid,
    quality: Quality,
    side: Side,
) -> Option<Image> {
    let byte = uuid.as_bytes()[0];
    let request = warn_if(
        client
            .get(format!(
                "https://{CARD_URL}/{}/{side}/{:x}/{:x}/{uuid}.{}",
                quality.name(),
                byte / 16,
                byte % 16,
                quality.extension(),
            ))
            .send()
            .await,
    )?;
    let bytes = warn_if(request.bytes().await)?;
    write_image(&bytes, set_cn, uuid, side).await;
    throttled_parse_bytes(&bytes).await
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
                if let Some(image) = throttled_parse_bytes(&bytes).await {
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
    client: &Client,
    set_cn: Box<str>,
    uuid: Uuid,
    quality: Quality,
    front_image: CacheReadImage,
    back_image: CacheReadImage,
) {
    if IMAGES_IN_PROGRESS.lock().await.insert(uuid) {
        let (front, back) = join!(
            front_image.get_image(client, &set_cn, uuid, quality, Side::Front),
            back_image.get_image(client, &set_cn, uuid, quality, Side::Back)
        );
        IMAGES_TO_PROCESS.lock().await.insert(uuid, (front, back));
        IMAGES_IN_PROGRESS.lock().await.remove(&uuid);
    }
}
async fn read_cards_check(
    client: &Client,
    set_cn: Box<str>,
    uuid: Uuid,
    quality: Quality,
    mut front_image: CacheReadImage,
    mut back_image: CacheReadImage,
) {
    if let Some((front, back)) =
        get_images(&set_cn, uuid, matches!(back_image, CacheReadImage::Missing)).await
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
pub const SLEEP_TIME: Duration = Duration::new(0, 1_048_576);
async fn get_uuid(client: &Client, uuid: Uuid) -> Option<SubCard> {
    while CARDS_THROTTLE.try_wait().is_err() {
        sleep(SLEEP_TIME).await;
    }
    let request = warn_if(
        client
            .get(format!("https://{URL}/cards/{uuid}"))
            .send()
            .await,
    )?;
    let json_raw = warn_if(request.text().await)?;
    let json = warn_if(parse(&json_raw))?;
    SubCard::from_scryfall(json)
}
pub static IMAGES_TO_PROCESS: LazyLock<
    Mutex<HashMap<Uuid, (Option<Image>, Option<Image>), FxBuildHasher>>,
> = LazyLock::new(|| Mutex::new(HashMap::with_capacity_and_hasher(512, FxBuildHasher)));
pub static IMAGES_IN_PROGRESS: LazyLock<Mutex<HashSet<Uuid, FxBuildHasher>>> =
    LazyLock::new(|| Mutex::new(HashSet::with_capacity_and_hasher(512, FxBuildHasher)));
impl SubCard {
    #[must_use]
    pub async fn get_list(
        client: &Client,
        iter: &[Uuid],
        quality: Quality,
    ) -> Vec<Result<Self, Uuid>> {
        join_all(
            iter.iter()
                .copied()
                .map(|uuid| Self::get_id(client, uuid, quality)),
        )
        .await
    }
    #[must_use]
    pub async fn get_list_set_cn(
        client: &Client,
        iter: &[&str],
        quality: Quality,
    ) -> Vec<Result<Self, Box<str>>> {
        join_all(
            iter.iter()
                .map(|set_cn| Self::get_set_cn(client, set_cn, quality)),
        )
        .await
    }
    pub async fn get_prints_id(
        client: &Client,
        id: Uuid,
        quality: Quality,
    ) -> Result<Vec<Result<Self, Uuid>>, Uuid> {
        let card = Self::get_id(client, id, quality).await?;
        Self::get_prints(client, card.data.front.oracle_id, quality)
            .await
            .map_err(|_| id)
    }
    pub async fn get_prints_set_cn(
        client: &Client,
        set_cn: &str,
        quality: Quality,
    ) -> Result<Vec<Result<Self, Uuid>>, Box<str>> {
        let card = Self::get_set_cn(client, set_cn, quality).await?;
        Self::get_prints(client, card.data.front.oracle_id, quality)
            .await
            .map_err(|_| set_cn.into())
    }
    pub async fn get_prints(
        client: &Client,
        oracle: Uuid,
        quality: Quality,
    ) -> Result<Vec<Result<Self, Uuid>>, Uuid> {
        async fn inner(
            client: &Client,
            oracle: Uuid,
            quality: Quality,
        ) -> Option<Vec<Result<SubCard, Uuid>>> {
            let mut jsons = Vec::new();
            for i in 1.. {
                while SEARCH_THROTTLE.try_wait().is_err() {
                    sleep(SLEEP_TIME).await;
                }
                let request = warn_if(client
                    .get(format!(
                        "https://{URL}/cards/search?q=oracleid%3D{oracle}+game%3Dpaper+unique%3Aprints"
                    ))
                    .query(&(("page", i),))
                    .send()
                    .await
                )?;
                let json_raw = warn_if(request.text().await)?;
                let mut json = warn_if(parse(&json_raw))?;
                if jsons.capacity() == 0 {
                    let len = json["total_cards"].as_usize()?;
                    jsons.reserve_exact(len);
                }
                jsons.append(json["data"].as_array_mut()?);
                if !json["has_more"].as_bool()? {
                    break;
                }
            }
            Some(
                join_all(
                    jsons
                        .into_iter()
                        .map(|card_json| SubCard::get_json(client, card_json, quality)),
                )
                .await,
            )
        }
        inner(client, oracle, quality).await.ok_or(oracle)
    }
    pub async fn get_json(
        client: &Client,
        json: JsonValue,
        quality: Quality,
    ) -> Result<Self, Uuid> {
        let uuid = Uuid::parse_str(json["id"].as_str().unwrap_or_default()).unwrap_or_default();
        let res = {
            let mut cache = CACHE.lock().await;
            cache.get(uuid)
        };
        Self::get_cache_result(client, res, quality, async |_| Self::from_scryfall(json))
            .await
            .ok_or(uuid)
    }
    pub async fn get_cache_result(
        client: &Client,
        cache_result: CacheResult<'_>,
        quality: Quality,
        on_none: impl AsyncFnOnce(&Client) -> Option<Self>,
    ) -> Option<Self> {
        if let Some(card) = match cache_result {
            CacheResult::Some(card) => Some(Self::from(card)),
            CacheResult::Cached(set_cn, uuid) => {
                if let Some(data) = CardData::read_files(&set_cn, uuid).await {
                    let back_handles = if data.back.as_ref().is_some_and(|c| c.has_unique_face) {
                        MaybeHandles::Waiting
                    } else {
                        MaybeHandles::None
                    };
                    let card = Self::from(SubCardInner {
                        data: Arc::new(data),
                        face_handles: MaybeHandles::Waiting,
                        back_handles,
                    });
                    CACHE.lock().await.insert(card.inner.clone());
                    Some(card)
                } else if let Some(card) = on_none(client).await {
                    Some(card)
                } else {
                    CACHE.lock().await.in_progress.remove(&uuid);
                    None
                }
            }
            CacheResult::Wait(Identifier::Uuid(uuid)) => loop {
                sleep(SLEEP_TIME).await;
                let card = {
                    let cache = CACHE.lock().await;
                    if cache.in_progress.contains(&uuid) {
                        continue;
                    }
                    cache.cards.get(&uuid)?.clone()
                };
                return Some(Self::from(card));
            },
            CacheResult::Wait(Identifier::SetCn(str)) => loop {
                sleep(SLEEP_TIME).await;
                let card = {
                    let cache = CACHE.lock().await;
                    if cache.in_progress_set_cn.contains(str) {
                        continue;
                    }
                    let &uuid = cache.set_cn.get_by_left(str)?;
                    cache.cards.get(&uuid)?.clone()
                };
                return Some(Self::from(card));
            },
            CacheResult::None(_) if let Some(card) = on_none(client).await => Some(card),
            CacheResult::None(Identifier::Uuid(uuid)) => {
                CACHE.lock().await.in_progress.remove(&uuid);
                None
            }
            CacheResult::None(Identifier::SetCn(str)) => {
                CACHE.lock().await.in_progress_set_cn.remove(str);
                None
            }
        } {
            card.spawn_image_getters(client, quality);
            Some(card)
        } else {
            None
        }
    }
    pub async fn get_id(client: &Client, uuid: Uuid, quality: Quality) -> Result<Self, Uuid> {
        let res = {
            let mut cache = CACHE.lock().await;
            cache.get(uuid)
        };
        Self::get_cache_result(client, res, quality, async |client| {
            get_uuid(client, uuid).await
        })
        .await
        .ok_or(uuid)
    }
    pub async fn get_set_cn(
        client: &Client,
        set_cn: &str,
        quality: Quality,
    ) -> Result<Self, Box<str>> {
        let res = {
            let mut cache = CACHE.lock().await;
            cache.get_set_cn(set_cn)
        };
        Self::get_cache_result(client, res, quality, async |client| {
            while CARDS_THROTTLE.try_wait().is_err() {
                sleep(SLEEP_TIME).await;
            }
            let request = warn_if(
                client
                    .get(format!("https://{URL}/cards/{set_cn}"))
                    .send()
                    .await,
            )?;
            let json_raw = warn_if(request.text().await)?;
            let json = warn_if(parse(&json_raw))?;
            Self::from_scryfall(json)
        })
        .await
        .ok_or_else(|| set_cn.into())
    }
    #[must_use]
    pub fn from_scryfall(json: JsonValue) -> Option<Self> {
        fn get_face(json: &JsonValue, face: &JsonValue) -> Option<CardInfo> {
            fn get<'a>(face: &'a JsonValue, json: &'a JsonValue, s: &str) -> &'a JsonValue {
                if face[s].is_null() {
                    &json[s]
                } else {
                    &face[s]
                }
            }
            let oracle_id = warn_if(Uuid::parse_str(get(face, json, "oracle_id").as_str()?))?;
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
        let id = Uuid::from_str(json["id"].as_str()?).ok()?;
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
            id,
            set_cn,
            tokens: tokens.into(),
            front,
            back,
            layout,
        };
        let card = Self::from(SubCardInner {
            data: Arc::new(data),
            face_handles,
            back_handles,
        });
        Some(card)
    }
    pub fn spawn_image_getters(&self, client: &Client, quality: Quality) {
        let face = (&self.face_handles).into();
        let back = (&self.back_handles).into();
        if matches!(face, CacheReadImage::Missing) || matches!(back, CacheReadImage::Missing) {
            let set_cn = self.data.set_cn.clone();
            let id = self.data.id;
            let cloned = client.clone();
            tokio::spawn(async move {
                read_cards_check(&cloned, set_cn, id, quality, face, back).await;
            });
        }
    }
}
