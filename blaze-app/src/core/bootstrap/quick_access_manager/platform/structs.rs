use std::{
    path::{
        Path,
        PathBuf
    }, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, time::{
        Duration, Instant, SystemTime
    }
};
use egui::Color32;
use serde::{Serialize, Deserialize};
use tracing::warn;
use uuid::Uuid;

use crate::core::{
    files::blaze_motor::motor_structs::FileKind, 
    system::{
        clipboard::clipboard::TOKIO_RUNTIME, 
        sizer_manager::sizer_manager::SizerManager
    }
};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedMeta {
    pub size: u64,
    pub modified: SystemTime,
    pub refreshed_at: Instant,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuickLinks {
    pub id: Uuid,
    pub name: Box<str>,
    pub path: Arc<Path>,
    pub is_dir: bool,
    pub color: Color32,
    pub kind: FileKind,
    #[serde(skip)]
    pub meta: Arc<Mutex<Option<CachedMeta>>>,
    #[serde(skip)]
    pub cancel: Arc<AtomicBool>,
}

impl QuickLinks {
    pub fn new(name: impl Into<Box<str>>, color: Color32) -> Self {
        Self {
            color,
            name: name.into(),
            id: Uuid::new_v4(),
            path: PathBuf::from("").into(),
            is_dir: false,
            meta: Arc::new(None.into()),
            cancel: Arc::new(AtomicBool::new(false)),
            kind: FileKind::File,
        }
    }

    pub fn needs_refresh(&self, ttl: Duration) -> bool {
        self.meta.lock()
            .ok()
            .and_then(|g| g.as_ref().map(|m| m.refreshed_at.elapsed() > ttl))
            .unwrap_or(true)
    }

    pub fn refresh_meta(&self) {
        self.cancel.store(true, Ordering::Release);

        let cancel = Arc::new(AtomicBool::new(false));

        let path = self.path.clone();
        let meta_arc = self.meta.clone();
        let cancel_clone = cancel.clone();

        if self.is_dir {
            TOKIO_RUNTIME.spawn(async move {
                let size_res = SizerManager::calc_size_for(path.clone(), cancel_clone).await;

                match size_res {
                    Ok(size) => {
                        if let Ok(m) = std::fs::metadata(&path) {
                            if let Ok(mut guard) = meta_arc.lock() {
                                *guard = Some(CachedMeta {
                                    size,
                                    modified: m.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                                    refreshed_at: Instant::now(),
                                });
                            }
                        }
                    },
                    Err(e) => {
                        warn!("No se ha podido extraer el tamaño: {e}")
                    },
                }
            });
        } else {
            TOKIO_RUNTIME.spawn_blocking(move || {
                if let Ok(m) = std::fs::metadata(&path) {
                    if let Ok(mut guard) = meta_arc.lock() {
                        *guard = Some(CachedMeta {
                            size: m.len(),
                            modified: m.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                            refreshed_at: Instant::now(),
                        });
                    }
                }
            });
        }
    }

    pub fn with_kind(mut self, kind: FileKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn with_path(mut self, path: Arc<Path>) -> Self {
        self.path = path;
        self
    }

    pub fn with_is_dir(mut self, is_dir: bool) -> Self {
        self.is_dir = is_dir;
        self
    }
}



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QuickTag {
    pub id: Uuid,
    pub title: Box<str>,
    pub color: Color32,
    pub items: Vec<QuickLinks>,
}

impl QuickTag {
    pub fn new(title: impl Into<Box<str>>, color: Color32) -> Self {
        Self { title: title.into(), color, items: Vec::new(), id: Uuid::new_v4() }
    }
}
