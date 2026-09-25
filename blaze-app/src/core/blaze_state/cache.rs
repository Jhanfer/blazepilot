use super::BlazeCoreState;
use crate::core::system::{
    cache::cache_manager::CacheManager, clipboard::global_clipboard::TOKIO_RUNTIME,
};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

/// Timestamp unix de la última solicitud de guardado en caché
///
/// se utiliza para implementar un delay de 3 segundos evitando
/// realizar múltiples guardados cuando se solicitan varios rapidamente
static LAST_SAVE_REQUEST: AtomicU64 = AtomicU64::new(0);

impl BlazeCoreState {
    pub fn save_caches(&self, force: bool) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        LAST_SAVE_REQUEST.store(now, Ordering::Relaxed);

        TOKIO_RUNTIME.spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            let stored = LAST_SAVE_REQUEST.load(Ordering::Relaxed);

            let current = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            if current - stored >= 3 || force {
                let cm = CacheManager::global();
                cm.save_extended_info_cache().await;
                cm.save_size_cache().await;
                cm.save_color_cache().await
            }
        });
    }
}
