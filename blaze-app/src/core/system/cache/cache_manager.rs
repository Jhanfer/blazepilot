// Copyright 2026 Jhanfer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::core::system::{
    cache::color_cache::color_cache_logic::ColorCache,
    extended_info::extended_info_manager::ExtendedInfoCache,
    knowndirs::knowndirs_manager::KnownDirsManager,
};
use egui::Color32;
use file_id::FileId;

use lru::LruCache;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    hash::Hash,
    num::NonZeroUsize,
    path::Path,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use dashmap::DashMap;
use tracing::error;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SizeCache {
    pub size: u64,
    pub modified: u64,
}

static CACHE_MANAGER: OnceLock<CacheManager> = OnceLock::new();

const CACHE_MANAGER_LIMIT: usize = 50;

pub struct CacheManager {
    pub cache_dir: Arc<Path>,
    pub size_cache: Mutex<LruCache<String, SizeCache>>,
    pub invalidated: Mutex<LruCache<String, ()>>,
    pub extended_info_cache: Mutex<LruCache<String, ExtendedInfoCache>>,
    pub color_cache: Mutex<LruCache<FileId, ColorCache>>,
    pub size_cache_loaded: AtomicBool,
    pub color_cache_loaded: AtomicBool,
    pub extended_info_loaded: AtomicBool,
}

impl CacheManager {
    pub fn global() -> &'static Self {
        let app_cache = &KnownDirsManager::get().app_cache;
        let cache_dir = app_cache.clone();

        let def_cap: NonZeroUsize = match NonZeroUsize::new(CACHE_MANAGER_LIMIT) {
            Some(n) => n,
            None => unreachable!(),
        };

        let cap = NonZeroUsize::new(50).unwrap_or(def_cap);

        CACHE_MANAGER.get_or_init(|| Self {
            cache_dir,
            invalidated: Mutex::new(LruCache::new(cap)),
            size_cache: Mutex::new(LruCache::new(cap)),
            color_cache: Mutex::new(LruCache::new(cap)),
            extended_info_cache: Mutex::new(LruCache::new(cap)),
            size_cache_loaded: AtomicBool::new(false),
            color_cache_loaded: AtomicBool::new(false),
            extended_info_loaded: AtomicBool::new(false),
        })
    }

    pub fn invalidate(&self, path: &Path) {
        let key = path.to_string_lossy().into_owned();
        self.invalidated.lock().put(key, ());
    }

    pub fn is_invalidated(&self, path: &Path) -> bool {
        let key = path.to_string_lossy().into_owned();
        self.invalidated.lock().get(&key).is_some()
    }

    pub fn clear_invalidated(&self, path: &Path) {
        let key = path.to_string_lossy().into_owned();
        self.invalidated.lock().pop(&key);
    }

    async fn load_cache<K, T>(&self, filename: &str) -> Option<DashMap<K, T>>
    where
        K: DeserializeOwned + Eq + Hash,
        T: DeserializeOwned,
    {
        let cache_path = self.cache_dir.join(filename);

        match tokio::fs::read(&cache_path).await {
            Ok(data) => match postcard::from_bytes::<DashMap<K, T>>(&data) {
                Ok(cache_data) => Some(cache_data),
                Err(e) => {
                    error!("Error al deserializar cache: {}", e);
                    None
                }
            },

            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => {
                error!("Error al leer {}: {}", filename, e);
                None
            }
        }
    }

    pub async fn save_cache<K, T>(&self, filename: &str, data: &Mutex<LruCache<K, T>>)
    where
        K: Serialize + Eq + Hash + Clone,
        T: Serialize + Clone,
    {
        let cache_path = self.cache_dir.join(filename);

        if let Some(parent) = cache_path.parent()
            && let Err(e) = tokio::fs::create_dir_all(parent).await
        {
            error!("Error al crear el directorio de caché {:?}: {}", parent, e);
            return;
        }

        let entries: Vec<(K, T)> = {
            let lock = data.lock();
            lock.iter()
                .take(CACHE_MANAGER_LIMIT)
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        };

        let bytes = match postcard::to_allocvec(&entries) {
            Ok(b) => b,
            Err(e) => {
                error!("Error al serializar {}: {}", filename, e);
                return;
            }
        };

        if let Err(e) = tokio::fs::write(&cache_path, bytes).await {
            error!("Error al guardar el caché en: {:?} : {}.", cache_path, e);
        }
    }

    ///------ Pesos ----
    pub async fn load_size_cache(&self) {
        if let Some(cache) = self
            .load_cache::<String, SizeCache>("cache_sizes.bin")
            .await
        {
            for (key, value) in cache {
                self.size_cache.lock().put(key, value);
            }
        }
        self.size_cache_loaded.store(true, Ordering::SeqCst);
    }

    pub async fn save_size_cache(&self) {
        while !self.size_cache_loaded.load(Ordering::SeqCst) {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        self.save_cache("cache_sizes.bin", &self.size_cache).await;
    }

    pub fn update_cache_size(&self, path: String, size: u64, modified: u64) {
        self.size_cache
            .lock()
            .put(path, SizeCache { size, modified });
    }

    pub fn get_cached_size(&self, path: &Path) -> Option<u64> {
        if !self.size_cache_loaded.load(Ordering::SeqCst) {
            return None;
        }

        let key = path.to_string_lossy();
        self.size_cache.lock().get(key.as_ref()).map(|c| c.size)
    }

    ///------ Colores ----    
    pub async fn load_color_cache(&self) {
        if let Some(cache) = self
            .load_cache::<FileId, ColorCache>("color_cache.bin")
            .await
        {
            for (key, value) in cache {
                self.color_cache.lock().put(key, value);
            }
        }
        self.color_cache_loaded.store(true, Ordering::SeqCst);
    }

    pub async fn save_color_cache(&self) {
        while !self.color_cache_loaded.load(Ordering::SeqCst) {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        self.save_cache("color_cache.bin", &self.color_cache).await;
    }

    pub async fn update_color_cache(&self, file_id: FileId, new_color: Color32) {
        self.color_cache
            .lock()
            .put(file_id, ColorCache { color: new_color });
    }

    pub fn get_cached_color(&self, file_id: &FileId) -> Color32 {
        if !self.color_cache_loaded.load(Ordering::SeqCst) {
            return Color32::YELLOW;
        }

        self.color_cache
            .lock()
            .get(file_id)
            .map(|c| c.color)
            .unwrap_or(Color32::YELLOW)
    }

    ///------ Info extendida ----
    pub async fn load_extended_info_cache(&self) {
        if let Some(cache) = self
            .load_cache::<String, ExtendedInfoCache>("cache_extended_info.bin")
            .await
        {
            for (key, value) in cache {
                self.extended_info_cache.lock().put(key, value);
            }
        }
        self.extended_info_loaded.store(true, Ordering::SeqCst);
    }

    pub async fn save_extended_info_cache(&self) {
        while !self.extended_info_loaded.load(Ordering::SeqCst) {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        self.save_cache("cache_extended_info.bin", &self.extended_info_cache)
            .await;
    }

    pub async fn update_extended_info_cache(&self, path: String, info: ExtendedInfoCache) {
        self.extended_info_cache.lock().put(path, info);
    }

    pub fn get_cached_extended_info(&self, path: &Path) -> Option<ExtendedInfoCache> {
        if !self.extended_info_loaded.load(Ordering::SeqCst) {
            return None;
        }
        let key = path.to_string_lossy();
        self.extended_info_cache.lock().get(key.as_ref()).cloned()
    }
}
