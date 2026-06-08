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

use crate::core::{
    runtime::{
        bus_structs::FileOperation,
        event_bus::{with_event_bus, Dispatcher},
    },
    system::{cache::cache_manager::CacheManager, clipboard::global_clipboard::TOKIO_RUNTIME},
};
use jwalk::{Parallelism, WalkDir};
use parking_lot::Mutex;
use std::{
    collections::{HashMap, HashSet},
    os::unix::fs::MetadataExt,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, UNIX_EPOCH},
};
use tokio::task::AbortHandle;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
#[error("Operación Cancelada")]
pub struct CancelledError;

pub enum SizerMessages {
    StartCal(Arc<Path>, Uuid),
    #[allow(unused)]
    CancelCal {
        path: Arc<Path>,
        request_id: Uuid,
    },
    CancelAll,
}

type SizerTaskMap = HashMap<Uuid, (AbortHandle, Arc<AtomicBool>)>;

pub struct SizerManager {
    pub cache_manager: &'static CacheManager,
    active_tasks: Arc<Mutex<SizerTaskMap>>,
}

impl SizerManager {
    pub fn new() -> Self {
        let cache_manager = CacheManager::global();
        TOKIO_RUNTIME.spawn(async move {
            cache_manager.load_size_cache().await;
        });
        Self {
            cache_manager,
            active_tasks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn cancel_task(&self, request_id: Uuid) {
        if let Some((handle, cancel)) = self.active_tasks.lock().remove(&request_id) {
            cancel.store(true, Ordering::Release);
            handle.abort();
            info!("Tarea cancelada: {}", request_id);
        }
    }

    pub fn cancel_all_tasks(&self) {
        let mut tasks = self.active_tasks.lock();
        for (id, (handle, cancel)) in tasks.drain() {
            cancel.store(true, Ordering::Release);
            handle.abort();
            info!("Tarea cancelada: {}", id);
        }
    }

    fn get_real_mtime(path: &Path) -> u64 {
        std::fs::metadata(path)
            .and_then(|m| m.modified())
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs())
            .unwrap_or(0)
    }

    pub fn process_messages(&mut self, active_id: Uuid, sender: Dispatcher) {
        let sizer_messages: Vec<SizerMessages> = with_event_bus(|pool| {
            let mut msgs = Vec::new();
            pool.drain(active_id, |msg| {
                msgs.push(msg);
                true
            });
            msgs
        });

        let cm = self.cache_manager;
        let tab_id = sender.tab_id;

        for msg in sizer_messages {
            match msg {
                SizerMessages::StartCal(path, request_id) => {
                    self.cancel_task(request_id);

                    let force = cm.is_invalidated(&path);
                    let current_mtime = Self::get_real_mtime(&path);
                    let key = path.to_string_lossy();

                    let cache_valid = if force {
                        false
                    } else {
                        cm.size_cache
                            .try_read()
                            .ok()
                            .and_then(|g| g.get(key.as_ref()).map(|c| c.modified == current_mtime))
                            .unwrap_or(false)
                    };

                    if cache_valid {
                        if let Some(cached_size) = cm.get_cached_size(&path) {
                            sender
                                .send(FileOperation::UpdateDirSize {
                                    full_path: path,
                                    size: cached_size,
                                    tab_id,
                                })
                                .ok();
                        }
                    } else {
                        cm.clear_invalidated(&path);

                        let mtime_to_task = current_mtime;
                        let cancel = Arc::new(AtomicBool::new(false));
                        let cancel_clone = cancel.clone();
                        let sender_clone = sender.clone();
                        let path_to_task = path.clone();
                        let active_tasks = self.active_tasks.clone();

                        let abort_handle = TOKIO_RUNTIME
                            .spawn(async move {
                                let result = tokio::time::timeout(
                                    Duration::from_secs(300),
                                    Self::get_recursive_size(
                                        path_to_task.clone(),
                                        sender_clone.clone(),
                                        path.clone(),
                                        tab_id,
                                        cancel_clone,
                                    ),
                                )
                                .await;

                                active_tasks.lock().remove(&request_id);

                                match result {
                                    Ok(Ok(size)) => {
                                        CacheManager::global()
                                            .update_cache_size(
                                                path_to_task.to_string_lossy().into_owned(),
                                                size,
                                                mtime_to_task,
                                            )
                                            .await;

                                        sender_clone
                                            .send(FileOperation::UpdateDirSize {
                                                full_path: path,
                                                size,
                                                tab_id,
                                            })
                                            .ok();
                                    }
                                    Ok(Err(_)) => {
                                        info!("Cálculo cancelado para {:?}", path_to_task)
                                    }
                                    Err(_) => warn!("Timeout en cálculo para {:?}", path_to_task),
                                }
                            })
                            .abort_handle();

                        self.active_tasks
                            .lock()
                            .insert(request_id, (abort_handle, cancel));
                    }
                }

                SizerMessages::CancelCal { path, request_id } => {
                    self.cancel_task(request_id);
                    cm.clear_invalidated(&path);
                }

                SizerMessages::CancelAll => {
                    self.cancel_all_tasks();
                }
            }
        }
    }

    pub async fn get_recursive_size(
        root: Arc<Path>,
        sender: Dispatcher,
        path_buf: Arc<Path>,
        tab_id: Uuid,
        cancel: Arc<AtomicBool>,
    ) -> Result<u64, CancelledError> {
        tokio::task::spawn_blocking(move || {
            let total = Arc::new(AtomicU64::new(0));
            let seen_inodes = Arc::new(Mutex::new(HashSet::new()));
            let last_reported = Arc::new(AtomicU64::new(0));
            const REPORT_THRESHOLD: u64 = 10 * 1024 * 1024;

            let walker = WalkDir::new(root)
                .max_depth(50)
                .skip_hidden(false)
                .follow_links(false)
                .parallelism(Parallelism::RayonNewPool(0));

            for entry in walker {
                if cancel.load(Ordering::Acquire) {
                    return Err(CancelledError);
                }

                let Ok(entry) = entry else { continue };
                if !entry.file_type().is_file() {
                    continue;
                }

                if let Ok(meta) = entry.metadata() {
                    let inode = (meta.dev(), meta.ino());
                    if seen_inodes.lock().insert(inode) {
                        let new_total = total.fetch_add(meta.len(), Ordering::Relaxed) + meta.len();
                        let last = last_reported.load(Ordering::Relaxed);

                        if new_total - last >= REPORT_THRESHOLD
                            && last_reported
                                .compare_exchange(
                                    last,
                                    new_total,
                                    Ordering::Relaxed,
                                    Ordering::Relaxed,
                                )
                                .is_ok()
                        {
                            sender
                                .send(FileOperation::UpdateDirSize {
                                    full_path: path_buf.to_owned(),
                                    size: new_total,
                                    tab_id,
                                })
                                .ok();
                        }
                    }
                }
            }

            Ok(total.load(Ordering::Relaxed))
        })
        .await
        .map_err(|_| CancelledError)?
    }

    pub async fn calc_size_for(
        root: Arc<Path>,
        cancel: Arc<AtomicBool>,
    ) -> Result<u64, CancelledError> {
        tokio::task::spawn_blocking(move || {
            let total = Arc::new(AtomicU64::new(0));
            let seen_inodes = Arc::new(Mutex::new(HashSet::new()));

            let walker = WalkDir::new(root)
                .max_depth(50)
                .skip_hidden(false)
                .follow_links(false)
                .parallelism(Parallelism::RayonNewPool(0));

            for entry in walker {
                if cancel.load(Ordering::Acquire) {
                    break;
                }

                let Ok(entry) = entry else { continue };
                if !entry.file_type().is_file() {
                    continue;
                }

                if let Ok(meta) = entry.metadata() {
                    let inode = (meta.dev(), meta.ino());
                    if seen_inodes.lock().insert(inode) {
                        total.fetch_add(meta.len(), Ordering::Relaxed);
                    }
                }
            }

            Ok(total.load(Ordering::Relaxed))
        })
        .await
        .map_err(|_| CancelledError)?
    }
}
