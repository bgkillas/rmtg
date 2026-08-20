use crate::app_name;
use crate::card::CardData;
use bevy::platform::dirs::preferences_dir;
use bitcode::Buffer;
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
    pub buffer: Buffer,
}
pub struct CardInCache {
    pub weak: Arc<CardData>,
    pub front_image: Option<Box<[u8]>>,
    pub back_image: Option<Box<[u8]>>,
}
impl CardInCache {
    pub fn downgrade(&self) -> CachedCard {
        CachedCard {
            weak: Some(Arc::downgrade(&self.weak)),
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
    preferences_dir().map(|p| p.join(app_name()).join(CACHE_FOLDER))
}
impl CardCache {
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
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
            buffer: Buffer::new(),
        }
    }
    pub fn clean(&mut self) {
        self.cards.retain(|_, card| !Arc::is_unique(&card.weak));
    }
    pub fn get_card(&mut self, uuid: Uuid) -> Option<CachedCard> {
        if let Some(val) = self.cards.get(&uuid) {
            Some(val.downgrade())
        } else if self.in_storage.contains(&uuid) {
            let folder_name = folder()?.join(uuid.to_string());
            let card_data = fs::read(folder_name.join(DATA)).ok()?;
            let mut card = CardInCache {
                weak: Arc::new(self.buffer.decode(&card_data).ok()?),
                front_image: None,
                back_image: None,
            };
            if let Ok(data) = fs::read(folder_name.join(FRONT)) {
                card.front_image = Some(data.into_boxed_slice());
            }
            if let Ok(data) = fs::read(folder_name.join(BACK)) {
                card.back_image = Some(data.into_boxed_slice());
            }
            self.cards.insert(uuid, card);
            Some(self.cards.get(&uuid)?.downgrade())
        } else {
            None
        }
    }
    pub fn insert_card(&mut self, uuid: Uuid, card: CardInCache) {
        self.cards.insert(uuid, card);
        #[cfg(not(target_family = "wasm"))]
        if let Some(folder_name) = folder().map(|f| f.join(uuid.to_string())) {
            let card_ref = self.cards.get(&uuid).unwrap();
            let data = self.buffer.encode::<CardData>(&card_ref.weak);
            let _ = fs::write(folder_name.join(DATA), data);
            if let Some(val) = &card_ref.front_image {
                _ = fs::write(folder_name.join(FRONT), val);
            }
            if let Some(val) = &card_ref.back_image {
                _ = fs::write(folder_name.join(BACK), val);
            }
        }
    }
}
