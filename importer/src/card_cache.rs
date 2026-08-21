use crate::card::{CardData, MaybeHandles};
use crate::scryfall::Side;
use bevy::asset::Handle;
use bevy::image::Image;
use bevy::platform::dirs::preferences_dir;
use bitcode::{decode, encode};
use rustc_hash::FxBuildHasher;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;
pub const CACHE_FOLDER: &str = "cache";
pub const DATA: &str = "card.data";
pub const FRONT: &str = "front.png";
pub const BACK: &str = "back.png";
pub struct CardCache {
    pub cards: HashMap<Uuid, CardInCache, FxBuildHasher>,
    pub in_storage: HashSet<Uuid, FxBuildHasher>,
    pub in_progress: HashSet<Uuid, FxBuildHasher>,
    pub set_cn: HashMap<Box<str>, Uuid, FxBuildHasher>,
}
#[derive(Clone)]
pub enum CacheImage {
    Some(Handle<Image>),
    Waiting,
    None,
}
#[derive(Clone)]
pub struct CardInCache {
    pub strong: Arc<CardData>,
    pub face_handles: MaybeHandles,
    pub back_handles: MaybeHandles,
}
pub enum CacheReadImage {
    Some(Box<[u8]>),
    Missing,
    None,
}
pub struct CacheRead {
    pub strong: Arc<CardData>,
    pub front_image: CacheReadImage,
    pub back_image: CacheReadImage,
}
fn folder() -> Option<PathBuf> {
    preferences_dir().map(|p| p.join(crate::app_name()).join(CACHE_FOLDER))
}
impl CardData {
    pub fn folder_path(&self) -> String {
        format!("{}/{}", self.set_cn, self.id)
    }
}
impl Default for CardCache {
    fn default() -> Self {
        let mut in_storage = HashSet::with_hasher(FxBuildHasher);
        let mut set_cn = HashMap::with_hasher(FxBuildHasher);
        if let Some(folder_name) = folder() {
            if fs::exists(&folder_name).is_ok_and(|b| b)
                && let Ok(dir) = fs::read_dir(&folder_name)
            {
                for entry in dir.filter_map(Result::ok) {
                    if let Some(str) = entry.file_name().to_str()
                        && let Some((set_cn_str, uuid_str)) = str.rsplit_once('/')
                        && let Ok(uuid) = uuid_str.parse()
                    {
                        in_storage.insert(uuid);
                        set_cn.insert(set_cn_str.into(), uuid);
                    }
                }
            } else {
                let _ = fs::create_dir_all(folder_name);
            }
        }
        set_cn.reserve(512);
        Self {
            cards: HashMap::with_capacity_and_hasher(512, FxBuildHasher),
            in_storage,
            in_progress: HashSet::with_capacity_and_hasher(512, FxBuildHasher),
            set_cn,
        }
    }
}
pub enum CacheResult {
    Some(CardInCache),
    Cached(Uuid),
    Wait(Uuid),
    None(Option<Uuid>),
}
impl CardCache {
    pub fn clean(&mut self) {
        self.cards.retain(|_, card| !Arc::is_unique(&card.strong));
    }
    pub fn get(&mut self, uuid: Uuid) -> CacheResult {
        if let Some(val) = self.cards.get(&uuid) {
            CacheResult::Some(val.clone())
        } else if self.in_storage.contains(&uuid) {
            self.in_progress.insert(uuid);
            CacheResult::Cached(uuid)
        } else if self.in_progress.contains(&uuid) {
            CacheResult::Wait(uuid)
        } else {
            self.in_progress.insert(uuid);
            CacheResult::None(Some(uuid))
        }
    }
    pub fn get_set_cn(&mut self, set_cn: &str) -> CacheResult {
        if let Some(&uuid) = self.set_cn.get(set_cn) {
            self.get(uuid)
        } else {
            CacheResult::None(None)
        }
    }
    pub fn insert(&mut self, card: CardInCache) {
        let uuid = card.strong.id;
        let set_cn = card.strong.set_cn.clone();
        self.set_cn.insert(set_cn, uuid);
        self.cards.insert(uuid, card);
        self.in_progress.remove(&uuid);
    }
}
impl CardInCache {
    pub fn write_files(&self) {
        if let Some(folder_name) = folder().map(|f| f.join(self.strong.folder_path())) {
            let _ = fs::create_dir_all(&folder_name);
            let data = encode::<CardData>(&self.strong);
            let _ = fs::write(folder_name.join(DATA), data);
        }
    }
}
impl CacheRead {
    pub fn read_files(uuid: Uuid) -> Option<Self> {
        let folder_name = folder()?.join(uuid.to_string());
        let card_data = fs::read(folder_name.join(DATA)).ok()?;
        let data = decode::<CardData>(&card_data).ok()?;
        let (front_image, back_image) =
            get_images(uuid, data.back.as_ref().is_some_and(|c| c.has_unique_face))?;
        let card = Self {
            strong: Arc::new(data),
            front_image,
            back_image,
        };
        Some(card)
    }
}
pub fn get_images(uuid: Uuid, has_unique_face: bool) -> Option<(CacheReadImage, CacheReadImage)> {
    let folder_name = folder()?.join(uuid.to_string());
    let mut front_image = CacheReadImage::Missing;
    let mut back_image = CacheReadImage::None;
    if let Ok(data) = fs::read(folder_name.join(FRONT)) {
        front_image = CacheReadImage::Some(data.into_boxed_slice());
    }
    if let Ok(data) = fs::read(folder_name.join(BACK)) {
        back_image = CacheReadImage::Some(data.into_boxed_slice());
    } else if has_unique_face {
        back_image = CacheReadImage::Missing;
    }
    Some((front_image, back_image))
}
pub fn write_image(bytes: &[u8], set_cn: &str, uuid: Uuid, side: Side) {
    if let Some(folder_name) = folder().map(|f| f.join(format!("{set_cn}/{uuid}"))) {
        let _ = fs::create_dir_all(&folder_name);
        let _ = fs::write(
            folder_name.join(match side {
                Side::Front => FRONT,
                Side::Back => BACK,
            }),
            bytes,
        );
    }
}
