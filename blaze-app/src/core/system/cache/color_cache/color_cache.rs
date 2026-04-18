use egui::Color32;
use file_id::FileId;
use serde::{Deserialize, Serialize};

use crate::core::system::{cache::cache_manager::CacheManager, clipboard::TOKIO_RUNTIME};

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ColorCache {
    pub color: Color32,
}

pub struct FolderColorManager {
    pub cache_manager: &'static CacheManager,
}

impl FolderColorManager {
    pub fn new() -> Self {
        let cache_manager = CacheManager::global();
        TOKIO_RUNTIME.spawn(async move {
            cache_manager.load_color_cache().await;
        });

        Self { 
            cache_manager
        }
    }

    pub fn get_color(&self, id: &FileId) -> Color32 {
        self.cache_manager.get_cached_color(id)
    }

}