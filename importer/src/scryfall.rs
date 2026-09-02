use crate::card::{CardData, CardInfo, Layout, MaybeHandles, SubCard, SubCardInner};
use crate::card::{Colors, Cost, Types};
use crate::card_cache::{CacheReadImage, CacheResult, CardCache, Identifier, get_images};
use crate::image::parse_bytes;
use crate::warn_if;
use bevy::image::Image;
use bitcode::{self, Decode, Encode};
use futures::future::join_all;
use jzon::{JsonValue, parse};
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use ratelimit::Ratelimiter;
use reqwest::Client;
use rustc_hash::FxBuildHasher;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::iter;
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
#[derive(Debug, Clone, Copy, Encode, Decode, Default, Eq, Hash, PartialEq)]
pub enum Quality {
    Small,
    Normal,
    #[default]
    Large,
    Png,
    Art,
}
#[derive(Clone, Copy)]
pub enum Side {
    Front,
    Back,
}
pub const SEARCH_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'!')
    .add(b'#')
    .add(b'$')
    .add(b'%')
    .add(b'&')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b'+')
    .add(b',')
    .add(b'-')
    .add(b'.')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'_')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}')
    .add(b'~');
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
            Quality::Art => "art",
        }
    }
    #[must_use]
    pub fn extension(self) -> &'static str {
        match self {
            Quality::Small | Quality::Normal | Quality::Large => "jpg",
            Quality::Png => "png",
            Quality::Art => "webp",
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
    async fn get_bytes(
        client: &Client,
        uuid: Uuid,
        quality: Quality,
        side: Side,
    ) -> Result<bytes::Bytes, reqwest::Error> {
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
            .await?;
        request.bytes().await
    }
    let bytes = warn_if(get_bytes(client, uuid, quality, side).await)?;
    crate::card_cache::write_image(&bytes, set_cn, uuid, quality, side).await;
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
    let (front, back) = join!(
        front_image.get_image(client, &set_cn, uuid, quality, Side::Front),
        back_image.get_image(client, &set_cn, uuid, quality, Side::Back)
    );
    IMAGES_TO_PROCESS
        .lock()
        .await
        .insert((uuid, quality), (front, back));
}
async fn read_cards_check(
    client: &Client,
    set_cn: Box<str>,
    uuid: Uuid,
    quality: Quality,
    mut front_image: CacheReadImage,
    mut back_image: CacheReadImage,
) {
    if let Some((front, back)) = get_images(
        &set_cn,
        uuid,
        matches!(back_image, CacheReadImage::Missing),
        quality,
    )
    .await
    {
        front_image = front;
        back_image = back;
    }
    read_cards(client, set_cn, uuid, quality, front_image, back_image).await;
}
async fn read_cards_check_owned(
    client: Client,
    set_cn: Box<str>,
    uuid: Uuid,
    quality: Quality,
    front_image: CacheReadImage,
    back_image: CacheReadImage,
) {
    read_cards_check(&client, set_cn, uuid, quality, front_image, back_image).await;
}
pub type ReadCardsCheckedFuture = impl Future<Output = ()>;
impl From<&MaybeHandles> for CacheReadImage {
    fn from(value: &MaybeHandles) -> Self {
        match value {
            MaybeHandles::Waiting | MaybeHandles::Downloading => Self::Missing,
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
static NAMED_THROTTLE: LazyLock<Ratelimiter<Clock>> =
    LazyLock::new(|| Ratelimiter::with_clock(1, Clock::default()));
static RANDOM_THROTTLE: LazyLock<Ratelimiter<Clock>> =
    LazyLock::new(|| Ratelimiter::with_clock(1, Clock::default()));
static SEARCH_THROTTLE: LazyLock<Ratelimiter<Clock>> =
    LazyLock::new(|| Ratelimiter::with_clock(1, Clock::default()));
static COLLECTION_THROTTLE: LazyLock<Ratelimiter<Clock>> =
    LazyLock::new(|| Ratelimiter::with_clock(1, Clock::default()));
pub const SLEEP_TIME: Duration = Duration::new(0, 1_048_576);
async fn get_uuid(client: &Client, uuid: Uuid, quality: Quality) -> Option<SubCard> {
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
    SubCard::from_scryfall(json, quality).ok()
}
pub static IMAGES_TO_PROCESS: LazyLock<
    Mutex<HashMap<(Uuid, Quality), (Option<Image>, Option<Image>), FxBuildHasher>>,
> = LazyLock::new(|| Mutex::new(HashMap::with_capacity_and_hasher(512, FxBuildHasher)));
pub static IMAGES_IN_PROGRESS: LazyLock<Mutex<HashSet<(Uuid, Quality), FxBuildHasher>>> =
    LazyLock::new(|| Mutex::new(HashSet::with_capacity_and_hasher(512, FxBuildHasher)));
impl SubCard {
    pub async fn get_prints_id(
        client: &Client,
        id: Uuid,
        quality: Quality,
    ) -> Result<Vec<Result<Self, Uuid>>, Uuid> {
        let card = Self::get_id(client, id, quality, false).await?;
        Self::get_prints(client, card.data.front.oracle_id, quality)
            .await
            .map_err(|_| id)
    }
    pub async fn get_prints_set_cn(
        client: &Client,
        set_cn: &str,
        quality: Quality,
    ) -> Result<Vec<Result<Self, Uuid>>, Box<str>> {
        let card = Self::get_set_cn(client, set_cn, quality, false).await?;
        Self::get_prints(client, card.data.front.oracle_id, quality)
            .await
            .map_err(|_| set_cn.into())
    }
    pub async fn get_prints_str(
        client: &Client,
        set_cn: &str,
        quality: Quality,
    ) -> Result<Vec<Result<Self, Uuid>>, Box<str>> {
        let card = Self::get_str(client, set_cn, quality).await?;
        Self::get_prints(client, card.data.front.oracle_id, quality)
            .await
            .map_err(|_| set_cn.into())
    }
    #[must_use]
    pub async fn get_list<'a>(
        client: &Client,
        mut vec: Vec<Identifier<'a>>,
        quality: Quality,
    ) -> Option<Vec<Result<Self, Identifier<'a>>>> {
        async fn do_chunk(client: &Client, vec: &[(usize, Identifier<'_>)]) -> Option<JsonValue> {
            while COLLECTION_THROTTLE.try_wait().is_err() {
                sleep(SLEEP_TIME).await;
            }
            let mut array = JsonValue::new_array();
            for (_, id) in vec.iter().cloned() {
                let val = match id {
                    Identifier::Uuid(uuid) => {
                        let mut val = JsonValue::new_object();
                        val.insert("id", uuid.to_string()).ok()?;
                        val
                    }
                    Identifier::SetCn(set_cn) => {
                        let mut val = JsonValue::new_object();
                        let (set, cn) = set_cn.split_once('/')?;
                        val.insert("set", set).ok()?;
                        val.insert("collector_number", cn).ok()?;
                        val
                    }
                };
                array.push(val).ok()?;
            }
            let mut json = JsonValue::new_object();
            json.insert("identifiers", array).ok()?;
            let request = warn_if(
                client
                    .post(format!("https://{URL}/cards/collection"))
                    .header(reqwest::header::CONTENT_TYPE, "application/json")
                    .body(json.dump())
                    .send()
                    .await,
            )?;
            let json_raw = warn_if(request.text().await)?;
            let mut json_res = warn_if(parse(&json_raw))?;
            Some(json_res.remove("data"))
        }
        if vec.is_empty() {
            return Some(Vec::new());
        }
        vec.sort_unstable();
        let mut uniqued = Vec::with_capacity(vec.len());
        let mut cards = Vec::with_capacity(vec.len());
        let mut jsons = Vec::with_capacity(vec.len());
        {
            let mut iter = vec.into_iter();
            let mut last = iter.next().unwrap();
            let mut len = 1;
            for val in iter {
                if val == last {
                    len += 1;
                } else {
                    match SubCard::get_cache_result(last.clone(), quality, false).await {
                        Ok(card) => cards.extend(iter::repeat_n(Ok(card), len)),
                        Err(_) => uniqued.push((len, last)),
                    }
                    last = val;
                    len = 1;
                }
            }
            match SubCard::get_cache_result(last.clone(), quality, false).await {
                Ok(card) => cards.extend(iter::repeat_n(Ok(card), len)),
                Err(_) => uniqued.push((len, last)),
            }
        }
        let (chunks, remainder) = uniqued.as_chunks::<75>();
        for chunk in chunks {
            jsons.append(do_chunk(client, chunk).await?.as_array_mut()?);
        }
        if remainder.len() > 9 {
            jsons.append(do_chunk(client, remainder).await?.as_array_mut()?);
        } else if !remainder.is_empty() {
            let list =
                join_all(remainder.iter().map(|(_, ident)| {
                    SubCard::get_identifier(client, ident.clone(), quality, true)
                }))
                .await;
            cards.extend(
                list.into_iter()
                    .zip(remainder)
                    .flat_map(|(card, (n, _))| iter::repeat_n(card, *n)),
            );
        }
        if !jsons.is_empty() {
            let mut cache = CACHE.lock().await;
            for (card_json, (n, _)) in jsons.into_iter().zip(uniqued) {
                let res = match SubCard::from_scryfall(card_json, quality) {
                    Ok(card) => {
                        cache.insert(card.inner.clone()).await;
                        Ok(card)
                    }
                    Err(e) => Err(Identifier::Uuid(e)),
                };
                cards.extend(iter::repeat_n(res, n));
            }
            drop(cache);
        }
        Some(cards)
    }
    pub async fn get_prints(
        client: &Client,
        oracle: Uuid,
        quality: Quality,
    ) -> Result<Vec<Result<Self, Uuid>>, Uuid> {
        let query = format!("oracleid={oracle}");
        Self::search(client, &[&query, "game=paper", "unique:prints"], quality)
            .await
            .map_err(|_| oracle)
    }
    pub async fn search_str(
        client: &Client,
        query: &str,
        quality: Quality,
    ) -> Result<Vec<Result<Self, Uuid>>, Box<str>> {
        let mut in_quote = false;
        let querys = query
            .split(|c| {
                if c == '"' {
                    in_quote = !in_quote;
                    false
                } else if in_quote {
                    false
                } else {
                    c == ' '
                }
            })
            .collect::<Vec<_>>();
        Self::search(client, &querys, quality)
            .await
            .map_err(|_| query.into())
    }
    pub async fn search(
        client: &Client,
        querys: &[&str],
        quality: Quality,
    ) -> Result<Vec<Result<Self, Uuid>>, Box<[Box<str>]>> {
        async fn inner(
            client: &Client,
            querys: &[&str],
            quality: Quality,
        ) -> Option<Vec<Result<SubCard, Uuid>>> {
            let mut jsons = Vec::new();
            for i in 1..9 {
                while SEARCH_THROTTLE.try_wait().is_err() {
                    sleep(SLEEP_TIME).await;
                }
                let url = format!(
                    "https://{URL}/cards/search?q={}",
                    querys
                        .iter()
                        .map(|query| utf8_percent_encode(query, SEARCH_SET).to_string())
                        .collect::<Vec<_>>()
                        .join("+")
                );
                let request = warn_if(client.get(url).query(&(("page", i),)).send().await)?;
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
                        .map(|card_json| SubCard::get_json(card_json, quality)),
                )
                .await,
            )
        }
        inner(client, querys, quality)
            .await
            .ok_or_else(|| querys.iter().map(|s| Box::from(*s)).collect::<Box<[_]>>())
    }
    pub async fn get_json(json: JsonValue, quality: Quality) -> Result<Self, Uuid> {
        if json["object"].as_str() == Some("error") {
            return Err(Uuid::max());
        }
        let uuid =
            Uuid::from_str(json["id"].as_str().ok_or(Uuid::max())?).map_err(|_| Uuid::max())?;
        Self::get_cache_result_or(Identifier::Uuid(uuid), quality, false, async || {
            Self::from_scryfall(json, quality).ok()
        })
        .await
        .ok_or(uuid)
    }
    pub async fn get_cache_result_or(
        identifier: Identifier<'_>,
        quality: Quality,
        no_wait: bool,
        on_none: impl AsyncFnOnce() -> Option<Self>,
    ) -> Option<Self> {
        match Self::get_cache_result(identifier, quality, no_wait).await {
            Ok(card) => Some(card),
            Err(_) if let Some(card) = on_none().await => {
                CACHE.lock().await.insert(card.inner.clone()).await;
                Some(card)
            }
            Err(e) => {
                CACHE.lock().await.remove_in_progress(e);
                None
            }
        }
    }
    pub async fn get_cache_result(
        identifier: Identifier<'_>,
        quality: Quality,
        no_wait: bool,
    ) -> Result<Self, Identifier<'_>> {
        let cache_result = {
            let mut cache = CACHE.lock().await;
            match identifier {
                Identifier::Uuid(uuid) => cache.get(uuid, quality),
                Identifier::SetCn(set_cn) => cache.get_set_cn(set_cn, quality),
            }
        };
        let card = match cache_result {
            CacheResult::Some(card) => Self::from(card),
            CacheResult::Cached(set_cn, uuid) => {
                if let Some(data) = CardData::read_files(&set_cn, uuid).await {
                    let back_handles = if data.back.as_ref().is_some_and(|c| c.has_unique_face) {
                        MaybeHandles::Waiting
                    } else {
                        MaybeHandles::None
                    };
                    Self::from(SubCardInner {
                        data: Arc::new(data),
                        quality,
                        face_handles: MaybeHandles::Waiting,
                        back_handles,
                    })
                } else {
                    return Err(Identifier::Uuid(uuid));
                }
            }
            CacheResult::Wait(ident) if no_wait => return Err(ident),
            CacheResult::Wait(Identifier::Uuid(uuid)) => loop {
                sleep(SLEEP_TIME).await;
                let (data, (face_handles, back_handles)) = {
                    let cache = CACHE.lock().await;
                    if cache.in_progress.contains(&uuid) {
                        continue;
                    }
                    (
                        cache.cards.get(&uuid).unwrap().clone(),
                        cache
                            .handles
                            .get(&(uuid, quality))
                            .cloned()
                            .unwrap_or_default(),
                    )
                };
                return Ok(Self::from(SubCardInner {
                    data,
                    quality,
                    face_handles,
                    back_handles,
                }));
            },
            CacheResult::Wait(Identifier::SetCn(str)) => loop {
                sleep(SLEEP_TIME).await;
                let (data, (face_handles, back_handles)) = {
                    let cache = CACHE.lock().await;
                    if cache.in_progress_set_cn.contains(str) {
                        continue;
                    }
                    let &uuid = cache.set_cn.get_by_left(str).unwrap();
                    (
                        cache.cards.get(&uuid).unwrap().clone(),
                        cache
                            .handles
                            .get(&(uuid, quality))
                            .cloned()
                            .unwrap_or_default(),
                    )
                };
                return Ok(Self::from(SubCardInner {
                    data,
                    quality,
                    face_handles,
                    back_handles,
                }));
            },
            CacheResult::None(identifier) => {
                return Err(identifier);
            }
        };
        CACHE.lock().await.insert(card.inner.clone()).await;
        Ok(card)
    }
    pub async fn get_random(client: &Client, quality: Quality) -> Option<Self> {
        while RANDOM_THROTTLE.try_wait().is_err() {
            sleep(SLEEP_TIME).await;
        }
        let request = warn_if(
            client
                .get(format!(
                    "https://{URL}/cards/random?q={}",
                    utf8_percent_encode("game:paper", SEARCH_SET)
                ))
                .send()
                .await,
        )?;
        let json_raw = warn_if(request.text().await)?;
        let json = warn_if(parse(&json_raw))?;
        let card = SubCard::from_scryfall(json, quality).ok()?;
        CACHE.lock().await.insert(card.inner.clone()).await;
        Some(card)
    }
    pub async fn get_identifier<'a>(
        client: &Client,
        identifier: Identifier<'a>,
        quality: Quality,
        no_wait: bool,
    ) -> Result<Self, Identifier<'a>> {
        match identifier {
            Identifier::Uuid(uuid) => Self::get_id(client, uuid, quality, no_wait)
                .await
                .map_err(|_| identifier),
            Identifier::SetCn(set_cn) => Self::get_set_cn(client, set_cn, quality, no_wait)
                .await
                .map_err(|_| identifier),
        }
    }
    pub async fn get_id(
        client: &Client,
        uuid: Uuid,
        quality: Quality,
        no_wait: bool,
    ) -> Result<Self, Uuid> {
        Self::get_cache_result_or(Identifier::Uuid(uuid), quality, no_wait, async || {
            get_uuid(client, uuid, quality).await
        })
        .await
        .ok_or(uuid)
    }
    pub async fn get_set_cn(
        client: &Client,
        set_cn: &str,
        quality: Quality,
        no_wait: bool,
    ) -> Result<Self, Box<str>> {
        Self::get_cache_result_or(Identifier::SetCn(set_cn), quality, no_wait, async || {
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
            Self::from_scryfall(json, quality).ok()
        })
        .await
        .ok_or_else(|| set_cn.into())
    }
    pub async fn get_str(client: &Client, name: &str, quality: Quality) -> Result<Self, Box<str>> {
        async fn get_str(client: &Client, name: &str, quality: Quality) -> Option<SubCard> {
            while NAMED_THROTTLE.try_wait().is_err() {
                sleep(SLEEP_TIME).await;
            }
            let request = warn_if(
                client
                    .get(format!("https://{URL}/cards/named"))
                    .query(&(("fuzzy", name),))
                    .send()
                    .await,
            )?;
            let json_raw = warn_if(request.text().await)?;
            let json = warn_if(parse(&json_raw))?;
            let card = SubCard::from_scryfall(json, quality).ok()?;
            CACHE.lock().await.insert(card.inner.clone()).await;
            Some(card)
        }
        get_str(client, name, quality)
            .await
            .ok_or_else(|| name.into())
    }
    pub fn from_scryfall(json: JsonValue, quality: Quality) -> Result<Self, Uuid> {
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
        fn inner(id: Uuid, json: JsonValue, quality: Quality) -> Option<SubCard> {
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
            let set = json["set"].as_str()?;
            let cn = json["collector_number"].as_str()?;
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
            let card = SubCard::from(SubCardInner {
                data: Arc::new(data),
                quality,
                face_handles,
                back_handles,
            });
            Some(card)
        }
        if json["object"].as_str() == Some("error") {
            return Err(Uuid::max());
        }
        let id =
            Uuid::from_str(json["id"].as_str().ok_or(Uuid::max())?).map_err(|_| Uuid::max())?;
        inner(id, json, quality).ok_or(id)
    }
    #[define_opaque(ReadCardsCheckedFuture)]
    pub fn spawn_image_getters(
        &self,
        client: &Client,
        set: &mut HashSet<(Uuid, Quality), FxBuildHasher>,
        quality: Quality,
        spawn: impl FnOnce(ReadCardsCheckedFuture),
    ) {
        let face = CacheReadImage::from(&self.face_handles);
        let back = CacheReadImage::from(&self.back_handles);
        let id = self.data.id;
        if (matches!(face, CacheReadImage::Missing) || matches!(back, CacheReadImage::Missing))
            && set.insert((id, quality))
        {
            let set_cn = self.data.set_cn.clone();
            let cloned = client.clone();
            spawn(read_cards_check_owned(
                cloned, set_cn, id, quality, face, back,
            ));
        }
    }
}
