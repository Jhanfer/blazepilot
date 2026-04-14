use std::{collections::HashMap, env, path::PathBuf, sync::OnceLock};
use dirs::cache_dir;
use egui::mutex::RwLock;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tracing::{error};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SizeCache {
    pub size: u64,
    pub modified: u64,
}



static CACHE_MANAGER: OnceLock<CacheManager> = OnceLock::new();
pub struct CacheManager {
    pub cache_dir: PathBuf,
    pub size_cache: RwLock<HashMap<String, SizeCache>>,
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
            }
        })
    }

    async fn load_cache<T: DeserializeOwned>(&self, filename: &str) -> Option<HashMap<String, T>>{
        let cache_path = self.cache_dir.join(filename);

        match tokio::fs::read(&cache_path).await {
            Ok(data) => {
                match postcard::from_bytes::<HashMap<String, T>>(&data) {
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


    pub async fn save_cache<T: Serialize>(&self, filename: &str, data: &HashMap<String, T>) {
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


    pub async fn load_size_cache(&self) {
        if let Some(cache) = self.load_cache::<SizeCache>("cache_sizes.bin").await {
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

}