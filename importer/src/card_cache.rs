use crate::card::{CardData, MaybeHandles, SetCn, SubCardInner};
use crate::scryfall::{Quality, Side};
use bevy::asset::Handle;
use bevy::image::Image;
use bevy::platform::dirs::preferences_dir;
use bimap::BiHashMap;
use bitcode::{decode, encode};
use rustc_hash::FxBuildHasher;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
#[cfg(target_family = "wasm")]
use tokio_with_wasm as tokio;
use uuid::Uuid;
pub const CACHE_FOLDER: &str = "cache";
pub const DATA: &str = "card.data";
pub struct CardCache {
    pub cards: HashMap<Uuid, Arc<CardData>, FxBuildHasher>,
    pub handles: HashMap<(Uuid, Quality), (MaybeHandles, MaybeHandles), FxBuildHasher>,
    pub in_storage: HashSet<Uuid, FxBuildHasher>,
    pub in_progress: HashSet<Uuid, FxBuildHasher>,
    pub in_progress_set_cn: HashSet<SetCn, FxBuildHasher>,
    pub set_cn: BiHashMap<SetCn, Uuid, FxBuildHasher, FxBuildHasher>,
}
impl Quality {
    pub fn file_name(self, side: Side) -> &'static str {
        match (self, side) {
            (Self::Small, Side::Front) => "front_small.jpg",
            (Self::Normal, Side::Front) => "front_normal.jpg",
            (Self::Large, Side::Front) => "front_large.jpg",
            (Self::Png, Side::Front) => "front.png",
            (Self::Art, Side::Front) => "front_art.webp",
            (Self::Small, Side::Back) => "back_small.jpg",
            (Self::Normal, Side::Back) => "back_normal.jpg",
            (Self::Large, Side::Back) => "back_large.jpg",
            (Self::Png, Side::Back) => "back.png",
            (Self::Art, Side::Back) => "back_art.webp",
        }
    }
}
#[derive(Clone)]
pub enum CacheImage {
    Some(Handle<Image>),
    Waiting,
    None,
}
#[derive(Debug)]
pub enum CacheReadImage {
    Some(Box<[u8]>),
    Missing,
    None,
}
fn folder() -> Option<PathBuf> {
    preferences_dir().map(|p| p.join(crate::app_name()).join(CACHE_FOLDER))
}
impl CardData {
    pub fn folder_path(&self) -> String {
        folder_path(&self.set_cn, self.id)
    }
}
fn folder_path(set_cn: &SetCn, id: Uuid) -> String {
    format!("{set_cn}_{id}")
}
impl Default for CardCache {
    fn default() -> Self {
        let mut in_storage = HashSet::with_hasher(FxBuildHasher);
        let mut set_cn = BiHashMap::with_hashers(FxBuildHasher, FxBuildHasher);
        if let Some(folder_name) = folder() {
            if let Ok(dir) = std::fs::read_dir(&folder_name) {
                for set_path in dir.filter_map(Result::ok) {
                    if let Some(set) = set_path.file_name().to_str()
                        && let Ok(set_folder) = std::fs::read_dir(set_path.path())
                    {
                        for entry in set_folder.filter_map(Result::ok) {
                            if let Some(set_uuid) = entry.file_name().to_str()
                                && let Some((cn, uuid_str)) = set_uuid.rsplit_once('_')
                                && let Ok(uuid) = uuid_str.parse()
                            {
                                in_storage.insert(uuid);
                                set_cn.insert(SetCn::new(set, cn), uuid);
                            }
                        }
                    }
                }
            } else {
                let _ = std::fs::create_dir_all(folder_name);
            }
        }
        set_cn.reserve(512);
        Self {
            cards: HashMap::with_capacity_and_hasher(512, FxBuildHasher),
            handles: HashMap::with_capacity_and_hasher(512, FxBuildHasher),
            in_storage,
            in_progress: HashSet::with_capacity_and_hasher(512, FxBuildHasher),
            in_progress_set_cn: HashSet::with_capacity_and_hasher(512, FxBuildHasher),
            set_cn,
        }
    }
}
#[derive(Debug, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub enum Identifier {
    Uuid(Uuid),
    SetCn(SetCn),
}
impl From<Uuid> for Identifier {
    fn from(value: Uuid) -> Self {
        Self::Uuid(value)
    }
}
impl From<SetCn> for Identifier {
    fn from(value: SetCn) -> Self {
        Self::SetCn(value)
    }
}
#[derive(Debug)]
pub enum CacheResult {
    Some(SubCardInner),
    Cached(SetCn, Uuid),
    Wait(Identifier),
    None(Identifier),
}
impl CardCache {
    pub fn clean(&mut self) {
        self.cards.retain(|_, card| !Arc::is_unique(card));
        self.handles
            .retain(|_, (face, back)| !(face.is_unique() && back.is_unique()));
    }
    pub fn get(&mut self, uuid: Uuid, quality: Quality) -> CacheResult {
        if let Some(data) = self.cards.get(&uuid).cloned() {
            let (face_handles, back_handles) = self
                .handles
                .get(&(uuid, quality))
                .cloned()
                .unwrap_or_default();
            CacheResult::Some(SubCardInner {
                data,
                quality,
                face_handles,
                back_handles,
            })
        } else if self.in_storage.contains(&uuid) {
            self.in_progress.insert(uuid);
            let set_cn = self.set_cn.get_by_right(&uuid).unwrap();
            CacheResult::Cached(set_cn.clone(), uuid)
        } else if self.in_progress.contains(&uuid) {
            CacheResult::Wait(Identifier::Uuid(uuid))
        } else {
            self.in_progress.insert(uuid);
            CacheResult::None(Identifier::Uuid(uuid))
        }
    }
    pub fn get_set_cn(&mut self, set_cn: &SetCn, quality: Quality) -> CacheResult {
        if let Some(&uuid) = self.set_cn.get_by_left(set_cn) {
            self.get(uuid, quality)
        } else if self.in_progress_set_cn.contains(set_cn) {
            CacheResult::Wait(Identifier::SetCn(set_cn.clone()))
        } else {
            self.in_progress_set_cn.insert(set_cn.clone());
            CacheResult::None(Identifier::SetCn(set_cn.clone()))
        }
    }
    pub async fn insert(&mut self, card: SubCardInner) {
        card.write_files().await;
        let uuid = card.data.id;
        let set_cn = card.data.set_cn.clone();
        self.in_progress_set_cn.remove(&set_cn);
        self.in_progress.remove(&uuid);
        self.set_cn.insert(set_cn, uuid);
        self.cards.insert(uuid, card.data);
        self.handles
            .insert((uuid, card.quality), (card.face_handles, card.back_handles));
    }
    pub fn remove_in_progress(&mut self, identifier: Identifier) {
        match identifier {
            Identifier::Uuid(uuid) => {
                self.in_progress.remove(&uuid);
            }
            Identifier::SetCn(set_cn) => {
                self.in_progress_set_cn.remove(&set_cn);
            }
        }
    }
}
impl SubCardInner {
    pub async fn write_files(&self) {
        if let Some(folder_name) = folder().map(|f| f.join(self.data.folder_path())) {
            let _ = fs::create_dir_all(&folder_name).await;
            let data = encode::<CardData>(&self.data);
            let _ = fs::write(folder_name.join(DATA), data).await;
        }
    }
}
impl CardData {
    pub async fn read_files(set_cn: &SetCn, uuid: Uuid) -> Option<Self> {
        let folder_name = folder()?.join(folder_path(set_cn, uuid));
        let card_data = fs::read(folder_name.join(DATA)).await.ok()?;
        let data = decode::<CardData>(&card_data).ok()?;
        Some(data)
    }
}
pub async fn get_images(
    set_cn: &SetCn,
    uuid: Uuid,
    has_unique_face: bool,
    quality: Quality,
) -> Option<(CacheReadImage, CacheReadImage)> {
    let folder_name = folder()?.join(folder_path(set_cn, uuid));
    let mut front_image = CacheReadImage::Missing;
    let mut back_image = CacheReadImage::None;
    if let Ok(data) = fs::read(folder_name.join(quality.file_name(Side::Front))).await {
        front_image = CacheReadImage::Some(data.into_boxed_slice());
    }
    if let Ok(data) = fs::read(folder_name.join(quality.file_name(Side::Back))).await {
        back_image = CacheReadImage::Some(data.into_boxed_slice());
    } else if has_unique_face {
        back_image = CacheReadImage::Missing;
    }
    Some((front_image, back_image))
}
pub async fn write_image(bytes: &[u8], set_cn: &SetCn, uuid: Uuid, quality: Quality, side: Side) {
    if let Some(folder_name) = folder().map(|f| f.join(folder_path(set_cn, uuid))) {
        let _ = fs::create_dir_all(&folder_name).await;
        let _ = fs::write(folder_name.join(quality.file_name(side)), bytes).await;
    }
}
