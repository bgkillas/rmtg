use crate::card::CardData;
use bevy::platform::dirs::preferences_dir;
use bitcode::{decode, encode};
use rustc_hash::FxBuildHasher;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Weak};
use uuid::Uuid;
pub const CACHE_FOLDER: &str = "cache";
pub const DATA: &str = "card.data";
pub const FRONT: &str = "front.png";
pub const BACK: &str = "back.png";
pub struct CardCache {
    pub cards: HashMap<Uuid, CardInCache, FxBuildHasher>,
    pub in_storage: HashSet<Uuid, FxBuildHasher>,
    pub in_process: HashSet<Uuid, FxBuildHasher>,
    pub set_cn: HashMap<Box<str>, Uuid, FxBuildHasher>,
}
pub struct CardInCache {
    pub strong: Arc<CardData>,
    pub front_image: Option<Box<[u8]>>,
    pub back_image: Option<Box<[u8]>>,
}
impl CardInCache {
    pub fn downgrade(&self) -> CachedCard {
        CachedCard {
            weak: Some(Arc::downgrade(&self.strong)),
            front_image: self.front_image.clone(),
            back_image: self.back_image.clone(),
        }
    }
}
pub struct CachedCard {
    pub weak: Option<Weak<CardData>>,
    pub front_image: Option<Box<[u8]>>,
    pub back_image: Option<Box<[u8]>>,
}
fn folder() -> Option<PathBuf> {
    preferences_dir().map(|p| p.join(crate::app_name()).join(CACHE_FOLDER))
}
impl Default for CardCache {
    fn default() -> Self {
        let mut in_storage = HashSet::with_hasher(FxBuildHasher);
        #[cfg(not(target_family = "wasm"))]
        if let Some(folder_name) = folder() {
            if fs::exists(&folder_name).is_ok_and(|b| b)
                && let Ok(dir) = fs::read_dir(&folder_name)
            {
                for entry in dir.filter_map(Result::ok) {
                    if let Some(str) = entry.file_name().to_str()
                        && let Ok(uuid) = str.parse()
                    {
                        in_storage.insert(uuid);
                    }
                }
            } else {
                let _ = fs::create_dir_all(folder_name);
            }
        }
        Self {
            cards: HashMap::with_capacity_and_hasher(512, FxBuildHasher),
            in_storage,
            in_process: HashSet::with_capacity_and_hasher(512, FxBuildHasher),
            set_cn: HashMap::with_capacity_and_hasher(512, FxBuildHasher),
        }
    }
}
pub enum CacheResult {
    Some(CachedCard),
    Cached,
    Wait,
    None,
}
impl CardCache {
    pub fn clean(&mut self) {
        self.cards.retain(|_, card| !Arc::is_unique(&card.strong));
    }
    pub fn get(&mut self, uuid: Uuid) -> CacheResult {
        if let Some(val) = self.cards.get(&uuid) {
            CacheResult::Some(val.downgrade())
        } else if self.in_storage.contains(&uuid) {
            self.in_process.insert(uuid);
            CacheResult::Cached
        } else if self.in_process.contains(&uuid) {
            CacheResult::Wait
        } else {
            self.in_process.insert(uuid);
            CacheResult::None
        }
    }
    pub fn get_set_cn(&mut self, set_cn: &str) -> CacheResult {
        if let Some(&uuid) = self.set_cn.get(set_cn) {
            self.get(uuid)
        } else {
            CacheResult::None
        }
    }
    pub fn set(&mut self, uuid: Uuid) -> bool {
        self.in_process.insert(uuid)
    }
    pub fn insert(&mut self, card: CardInCache) {
        let uuid = card.strong.id;
        let set_cn = card.strong.set_cn.clone();
        self.set_cn.insert(set_cn, uuid);
        self.cards.insert(uuid, card);
        self.in_process.remove(&uuid);
    }
    pub fn read_files(uuid: Uuid) -> Option<CardInCache> {
        let folder_name = folder()?.join(uuid.to_string());
        let card_data = fs::read(folder_name.join(DATA)).ok()?;
        let mut card = CardInCache {
            strong: Arc::new(decode(&card_data).ok()?),
            front_image: None,
            back_image: None,
        };
        if let Ok(data) = fs::read(folder_name.join(FRONT)) {
            card.front_image = Some(data.into_boxed_slice());
        }
        if let Ok(data) = fs::read(folder_name.join(BACK)) {
            card.back_image = Some(data.into_boxed_slice());
        }
        Some(card)
    }
    pub fn write_files(uuid: Uuid, card: &CardInCache) {
        #[cfg(not(target_family = "wasm"))]
        if let Some(folder_name) = folder().map(|f| f.join(uuid.to_string())) {
            let data = encode::<CardData>(&card.strong);
            let _ = fs::write(folder_name.join(DATA), data);
            if let Some(val) = &card.front_image {
                _ = fs::write(folder_name.join(FRONT), val);
            }
            if let Some(val) = &card.back_image {
                _ = fs::write(folder_name.join(BACK), val);
            }
        }
    }
}
