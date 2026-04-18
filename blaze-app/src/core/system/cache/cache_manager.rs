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




use std::{collections::{HashMap, HashSet}, env, hash::Hash, path::PathBuf, sync::OnceLock};
use dirs::cache_dir;
use egui::{Color32, mutex::RwLock};
use file_id::FileId;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tracing::{error, info, warn};
use crate::core::system::cache::color_cache::color_cache::ColorCache;


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SizeCache {
    pub size: u64,
    pub modified: u64,
}



static CACHE_MANAGER: OnceLock<CacheManager> = OnceLock::new();
pub struct CacheManager {
    pub cache_dir: PathBuf,
    pub size_cache: RwLock<HashMap<String, SizeCache>>,
    pub invalidated: RwLock<HashSet<String>>,

    pub color_cache: RwLock<HashMap<FileId, ColorCache>>,
}

impl CacheManager {
    pub fn global() -> &'static Self {
        let cache_dir = cache_dir()
            .unwrap_or(env::temp_dir())
            .join("blazepilot");

        CACHE_MANAGER.get_or_init(|| {
            Self {
                cache_dir,
                size_cache: RwLock::new(HashMap::new()),
                color_cache: RwLock::new(HashMap::new()),
                invalidated: RwLock::new(HashSet::new()),
            }
        })
    }

    pub fn invalidate(&self, path: &PathBuf) {
        let key = path.to_string_lossy().into_owned();
        self.invalidated.write().insert(key);
    }

    pub fn is_invalidated(&self, path: &PathBuf) -> bool {
        let key = path.to_string_lossy();
        self.invalidated.read().contains(key.as_ref())
    }

    pub fn clear_invalidated(&self, path: &PathBuf) {
        let key = path.to_string_lossy().into_owned();
        self.invalidated.write().remove(&key);
    }
    
    async fn load_cache<K, T>(&self, filename: &str) -> Option<HashMap<K, T>>
    where 
        K: DeserializeOwned + Eq + Hash,
        T: DeserializeOwned,
    {
        let cache_path = self.cache_dir.join(filename);

        match tokio::fs::read(&cache_path).await {
            Ok(data) => {
                match postcard::from_bytes::<HashMap<K, T>>(&data) {
                    Ok(cache_data) => Some(cache_data),
                    Err(e) => {
                        error!("Error al deserializar cache: {}", e);
                        None
                    },
                }
            },

            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => {
                error!("Error al leer {}: {}", filename, e);
                None
            }
        }
    }


    pub async fn save_cache<K, T>(&self, filename: &str, data: &HashMap<K, T>)
    where 
        K: Serialize,
        T: Serialize,
    {
        let cache_path = self.cache_dir.join(filename);

        if let Some(parent) = cache_path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                error!("Error al crear el directorio de caché {:?}: {}", parent, e);
                return;
            }
        }

        let bytes = match postcard::to_allocvec(data) {
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
        if let Some(cache) = self.load_cache::<String, SizeCache>("cache_sizes.bin").await {
            let mut guard = self.size_cache.write();
            *guard = cache;
        }
    }


    pub async fn save_size_cache(&self) {
        let data_to_save = {
            let guard = self.size_cache.read();
            guard.clone()
        };
        self.save_cache("cache_sizes.bin", &data_to_save).await;
    }

    pub async fn update_cache_size(&self, path: String, size: u64, modified: u64) {
        {
            let mut guard = self.size_cache.write();
            guard.insert(path, SizeCache { size, modified });
        }
    }

    pub fn get_cached_size(&self, path: &PathBuf) -> Option<u64> {
        let key = path.to_string_lossy();
        self.size_cache.read()
            .get(key.as_ref())
            .map(|c| c.size)
    }


    pub async fn reload_size_cache(&self) {
        match self.load_cache::<String, SizeCache>("cache_sizes.bin").await {
            Some(new_cache) => {
                let mut guard = self.size_cache.write();
                let previous_count = guard.len();
                *guard = new_cache;
                info!("Size cache recargado correctamente. Entradas: {} → {}", 
                            previous_count, guard.len());
            }
            None => {
                warn!("No se pudo cargar el cache_sizes.bin. Manteniendo cache en memoria.");
            }
        }
    }



    ///------ Colores ----
    pub async fn reload_color_cache(&self) {
        match self.load_cache::<FileId, ColorCache>("color_cache.bin").await {
            Some(new_cache) => {
                let mut guard = self.color_cache.write();
                let previous_count = guard.len();
                *guard = new_cache;
                tracing::info!("Color cache recargado correctamente. Entradas: {} → {}", 
                            previous_count, guard.len());
            }
            None => {
                tracing::warn!("No se pudo cargar el color_cache.bin. Manteniendo cache en memoria.");
            }
        }
    }
    
    pub async fn load_color_cache(&self) {
        if let Some(cache) = self.load_cache::<FileId, ColorCache>("color_cache.bin").await {
            let mut guard = self.color_cache.write();
            *guard = cache;
        }
    }

    pub async fn save_color_cache(&self) {
        let data_to_save = {
            let guard = self.color_cache.read();
            guard.clone()
        };
        self.save_cache("color_cache.bin", &data_to_save).await;
    }

    pub async fn update_color_cache(&self, file_id: FileId, new_color: Color32) {
        {
            let mut guard = self.color_cache.write();
            guard.insert(file_id, ColorCache { color: new_color });
        }
        self.save_color_cache().await;
    }

    pub fn get_cached_color(&self, file_id: &FileId) -> Color32 {
        self.color_cache.read()
            .get(file_id)
            .map(|c| c.color)
            .unwrap_or(Color32::YELLOW)
    }

}