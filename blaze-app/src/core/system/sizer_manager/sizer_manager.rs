use std::{collections::HashSet, os::unix::fs::MetadataExt, path::{Path, PathBuf}, sync::{Arc, atomic::{AtomicU64, Ordering}}, time::UNIX_EPOCH};
use jwalk::{Parallelism, WalkDir};
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, info};
use uuid::Uuid;
use crate::{core::{system::{cache::cache_manager::CacheManager, clipboard::TOKIO_RUNTIME}}, utils::channel_pool::{FileOperation, NotifyingSender, with_channel_pool}};



pub enum SizerMessages {
    StartCal(PathBuf),
}

pub struct SizerManager {
    pub cache_manager: &'static CacheManager,
}

impl SizerManager {
    pub fn new() -> Self {
        let cache_manager = CacheManager::global();
        TOKIO_RUNTIME.spawn(async move {
            cache_manager.load_size_cache().await;
        });

        let sm = Self { 
            cache_manager
        };

        sm.tests();

        sm
    }


    pub fn tests(&self) {
        let e = self.cache_manager.size_cache.read();
        debug!("Test de tamaños {:?}", e);
    }

    fn get_real_mtime(path: &PathBuf) -> u64 {
        std::fs::metadata(path)
            .and_then(|m| m.modified())
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs())
            .unwrap_or(0)
    }

    pub fn process_messages(&mut self, active_id: Uuid, sender: NotifyingSender) {
        let sizer_messages: Vec<SizerMessages> = with_channel_pool(|pool|{
            let mut msgs = Vec::new();
            pool.process_sizer_events(active_id, |msg|{
                msgs.push(msg);
                true
            });
            msgs
        });


        let cm = self.cache_manager;

        for msg in sizer_messages {
            match msg {
                SizerMessages::StartCal(path_buf) => {
                    let current_mtime = Self::get_real_mtime(&path_buf);
                    let key = path_buf.to_string_lossy();
                    
                    let cache_valid = {
                        let guard = cm.size_cache.read();
                        guard.get(key.as_ref())
                            .map(|c| c.modified == current_mtime)
                            .unwrap_or(false)
                    };

                    if cache_valid {
                        if let Some(cached_size) = cm.get_cached_size(&path_buf) {
                            info!("Usando caché: {}, para {:?}", cached_size, path_buf);

                            sender.send_fileop(
                                FileOperation::UpdateDirSize {
                                    full_path: path_buf,
                                    size: cached_size,
                                    gene: 0,
                                }
                            ).ok();
                        }
                    } else {
                        let path_to_task = path_buf.clone();
                        let mtime_to_task = current_mtime.clone();
                        let sender_clone = sender.clone();

                        TOKIO_RUNTIME.spawn(async move {
                            let  calculated_size = Self::get_recursive_size(
                                &path_to_task, 
                                12,
                                sender_clone.clone(),
                                path_buf.clone()
                            ).await;

                            CacheManager::global()
                                .update_cache_size(
                                    path_to_task.to_string_lossy().into_owned(),
                                    calculated_size,
                                mtime_to_task,
                                ).await;

                            sender_clone.send_fileop(FileOperation::UpdateDirSize {
                                full_path: path_buf,
                                size: calculated_size,
                                gene: 0,
                            }).ok();


                            cm.save_size_cache().await;
                        });
                    }
                },
            }
        }

    }

    pub async fn get_recursive_size(root: impl AsRef<Path>, max_concurrency: usize, sender: NotifyingSender, path_buf: PathBuf) -> u64 {
        let total_size = Arc::new(AtomicU64::new(0));
        let seen_inodes = Arc::new(Mutex::new(HashSet::new()));
        let semaphore = Arc::new(Semaphore::new(max_concurrency));
        let last_reported = Arc::new(AtomicU64::new(0));

        const REPORT_THRESHOLD: u64 = 10 * 1024 * 1024;

        let walker = WalkDir::new(root)
            .max_depth(50)
            .skip_hidden(false)
            .follow_links(false)
            .parallelism(Parallelism::RayonNewPool(0));

        let mut tasks = vec![];

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            if entry.file_type().is_file() {
                let size_arc = total_size.clone();
                let inodes_arc = seen_inodes.clone();
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let sender_task = sender.clone();
                let path_task = path_buf.clone();
                let last_reported = last_reported.clone();
                
                let task = TOKIO_RUNTIME.spawn(async move {
                    let _permit = permit;

                    if let Ok(meta) = entry.metadata() {
                        let inode = (meta.dev(), meta.ino());
                        let mut guard = inodes_arc.lock().await;

                        if guard.insert(inode) {
                            let new_total = size_arc.fetch_add(meta.len(), Ordering::Relaxed) + meta.len();

                            let last = last_reported.load(Ordering::Relaxed);

                            if new_total - last >= REPORT_THRESHOLD {
                                if last_reported.compare_exchange(
                                    last, new_total,
                                    Ordering::Relaxed, Ordering::Relaxed
                                ).is_ok() {
                                    sender_task.send_fileop(
                                        FileOperation::UpdateDirSize {
                                            full_path: path_task,
                                            size: new_total,
                                            gene: 0,
                                        }
                                    ).ok();
                                }
                            }
                        }
                    }
                });
                
                tasks.push(task);
            }
        }

        for task in tasks {
            let _ = task.await;
        }

        total_size.load(Ordering::Relaxed)
    }   
}
