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




use std::{path::{Path, PathBuf}, sync::{Arc, RwLock}, time::UNIX_EPOCH};
use crossbeam_channel::SendError;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use tracing::{error, warn};
use users::{get_group_by_gid, get_user_by_uid};
use uuid::Uuid;
use crate::{core::system::{cache::cache_manager::CacheManager, clipboard::TOKIO_RUNTIME, extended_info::error::{ExtendedInfoError, ExtendedInfoResult}}, utils::channel_pool::{FileOperation, NotifyingSender, with_channel_pool}};
use lru::LruCache;
use std::num::NonZeroUsize;

pub enum ExtendedInfoMessages {
    StartScan(PathBuf),
    ForceScan(PathBuf),
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtendedInfoCache {
    pub owner: Option<String>,
    pub group_name: Option<String>,
    pub symlink_target: Option<PathBuf>,
    pub dimensions: Option<(u32, u32)>,
    pub git_status: Option<GitStatus>,
    pub modified: u64,
}

impl From<ExtendedInfo> for ExtendedInfoCache {
    fn from(info: ExtendedInfo) -> Self {
        Self {
            owner: info.owner,
            group_name: info.group_name,
            symlink_target: info.symlink_target,
            dimensions: info.dimensions,
            git_status: info.git_status,
            modified: 0,
        }
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ExtendedInfo {
    pub owner: Option<String>,
    pub group_name: Option<String>,
    pub symlink_target: Option<PathBuf>,
    pub dimensions: Option<(u32, u32)>,
    pub git_status: Option<GitStatus>,
}


impl From<ExtendedInfoCache> for ExtendedInfo {
    fn from(cached: ExtendedInfoCache) -> Self {
        Self {
            owner: cached.owner,
            group_name: cached.group_name,
            symlink_target: cached.symlink_target,
            dimensions: cached.dimensions,
            git_status: cached.git_status,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GitStatus {
    #[default]
    Clean,
    Modified,
    Staged,
    Untracked,
    Ignored,
    Conflict,
    Deleted,
}


const DEFAULT_INFO_CACHE_CAPACITY: usize = 2_000;
pub struct ExtendedInfoManager {
    pub cache_manager: &'static CacheManager,
    pub info_map: Arc<RwLock<LruCache<PathBuf, ExtendedInfo>>>,
    pub semaphore: Arc<Semaphore>,
}


impl ExtendedInfoManager {
    pub fn new() -> Self {
        let cache_manager = CacheManager::global();
        TOKIO_RUNTIME.spawn(async move {
            cache_manager.load_extended_info_cache().await;
        });

        Self::with_capacity(DEFAULT_INFO_CACHE_CAPACITY, cache_manager)
    }

    fn with_capacity(cap: usize, cache_manager: &'static CacheManager,) -> Self {
        let def_cap = match NonZeroUsize::new(2000) {
            Some(cap) => cap,
            None => unreachable!(),
        };
        let cap = NonZeroUsize::new(cap)
            .unwrap_or(def_cap);

        Self {
            cache_manager, 
            info_map: Arc::new(RwLock::new(LruCache::new(cap))),
            semaphore: Arc::new(tokio::sync::Semaphore::new(8)),
        }
    }

    pub fn clear_directory(&self, dir: &Path) {
        let Ok(mut guard) = self.info_map.write() else {return;};

        let to_move: Vec<PathBuf> = guard
            .iter()
            .filter(|(k, _)| k.parent().map_or(false, |p| p == dir))
            .map(|(k, _)| k.clone())
            .collect();

        for key in to_move {
            guard.pop(&key);
        }

    }

    fn get_real_mtime(path: &PathBuf) -> u64 {
        std::fs::metadata(path)
            .and_then(|m| m.modified())
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs())
            .unwrap_or(0)
    }


    fn requst_scan(&self, path_buf: PathBuf, current_mtime: u64, sender: &NotifyingSender, tab_id: Uuid) {
        let info_map = self.info_map.clone();
        let path_to_task = path_buf.clone();
        let sem = self.semaphore.clone();
        let tab_id = tab_id.clone();
        let sender = sender.clone();

        TOKIO_RUNTIME.spawn(async move {
            let _permit = match sem.acquire_owned().await {
                Ok(p) => p,
                Err(e) => {
                    error!("Semáforo cerrado: {}", e);
                    return;
                }
            };

            match Self::scan(&path_buf).await {
                Ok(info) => {
                    match info_map.write() {
                        Ok(mut g) => {
                            g.put(path_buf.clone(), info.clone());
                        },
                        Err(e) => {
                            warn!("info_map lock envenendado: {}", e);
                            return ;
                        },
                    }

                    let mut cached: ExtendedInfoCache = info.into();
                    cached.modified = current_mtime;
                    
                    CacheManager::global().update_extended_info_cache(
                        path_to_task.to_string_lossy().into_owned(),
                        cached,
                    ).await;


                    if let Err(e) = sender.send_fileop(FileOperation::ExtendedInfoReady {
                        full_path: path_buf,
                        tab_id,
                    }) {
                        warn!("Error al enviar ExtendedInfo: {}", e);
                    }
                },
                Err(e) => {
                    warn!("Error escaneando {:?}: {e}", path_buf);
                    return ;
                }
            }
        });
    }


    pub fn process_messages(&self, active_id: Uuid, sender: NotifyingSender) -> ExtendedInfoResult<()> {
        let messages: Vec<ExtendedInfoMessages> = with_channel_pool(|pool| {
            let mut msgs = Vec::new();
            pool.process_extended_info_events(active_id, |msg| {
                msgs.push(msg);
                true
            });
            msgs
        });

        let cm = self.cache_manager;

        for msg in messages {
            match msg {
                ExtendedInfoMessages::StartScan(path_buf) => {
                    let key = path_buf.to_string_lossy();
                    let current_mtime = Self::get_real_mtime(&path_buf);
                    let tab_id = sender.tab_id;
                    
                    let cache_valid= match cm.extended_info_cache.try_read() {
                        Ok(g) => {
                            g.get(key.as_ref())
                            .map(|c| c.modified == current_mtime)
                            .unwrap_or(false)
                        },
                        Err(e) => {
                            warn!("No se ha podido validar la caché de 'ExtendedInfo': {}", e);
                            false
                        },
                    };


                    if cache_valid {
                        if let Some(cached) = cm.get_cached_extended_info(&path_buf) {
                            match self.info_map.write() {
                                Ok(mut g) => {
                                    g.put(path_buf.clone(), cached.into());
                                },
                                Err(e) => {
                                    warn!("info_map lock envenendado: {}", e);
                                    return Err(ExtendedInfoError::PoisonedLock);
                                },
                            }

                            sender.send_fileop(FileOperation::ExtendedInfoReady {
                                full_path: path_buf,
                                tab_id,
                            }).map_err(|e| ExtendedInfoError::SendError(e))?;
                        }
                    } else {
                        self.requst_scan(path_buf, current_mtime, &sender, tab_id);
                    }
                },

                ExtendedInfoMessages::ForceScan(path_buf) => {
                    let current_mtime = Self::get_real_mtime(&path_buf);
                    let tab_id = sender.tab_id;

                    self.requst_scan(path_buf, current_mtime, &sender, tab_id);
                }
            }
        }


        Ok(())
    }

    async fn scan(path: &PathBuf) -> ExtendedInfoResult<ExtendedInfo> {
        let m = match tokio::fs::symlink_metadata(path).await {
            Ok(m) => m,
            Err(e) => {
                warn!("No se ha podido leer metadata en 'ExtendedInfoManager::scan':  {}", e);
                return Ok(ExtendedInfo::default())
            },
        };

        let symlink_target = if m.file_type().is_symlink() {
            match tokio::fs::read_link(path).await {
                Ok(p) => Some(p),
                Err(e) => {
                    warn!("Error leyendo los symlinks: {}", e);
                    return Err(ExtendedInfoError::SymlinkError);
                },
            }
        } else {
            None
        };

        #[cfg(unix)]
        let (owner, group_name) = {
            use std::os::unix::fs::MetadataExt;
            let uid = m.uid();
            let gid = m.gid();

            tokio::task::spawn(async move {
                let owner = get_user_by_uid(uid)
                    .map(|u| u.name().to_string_lossy().into_owned());
                let group_name = get_group_by_gid(gid)
                    .map(|g| g.name().to_string_lossy().into_owned());
                (owner, group_name)
            })
            .await
            .map_err(ExtendedInfoError::ThreadError)?
        };

        #[cfg(not(unix))]
        let (owner, group_name) = (None, None);

        let dimensions = if m.is_file() {
            let is_image = path.extension()
                .and_then(|e| e.to_str())
                .map(|e| matches!(e.to_lowercase().as_str(),
                    "jpg" | "jpeg" | "png" | "gif" | "webp" |
                    "bmp" | "tiff" | "tif" | "ico" | "avif" | "qoi"
                ))
                .unwrap_or(false);
            
            if is_image {
                let path_clone = path.clone();

                Some(
                    tokio::task::spawn(async move {
                        match image::image_dimensions(&path_clone) {
                            Ok(i) => Ok(i),
                            Err(e) => {
                                warn!("Error leyendo las dimensiones de la imagen: {}", e);
                                return Err(ExtendedInfoError::DimensionError);
                            }
                        }
                    })
                    .await
                    .map_err(ExtendedInfoError::ThreadError)??
                )
            } else {
                None
            }
        } else {
            None
        };


        let git_status = {
            let path_clone = path.clone();
            tokio::task::spawn(async move {
                Self::get_git_status(&path_clone)
            })
            .await
            .map_err(ExtendedInfoError::ThreadError)?
        }?;


        Ok(
            ExtendedInfo {
                owner,
                group_name,
                symlink_target,
                dimensions,
                git_status: Some(git_status),
            }
        )
    }


    fn get_git_status(path: &PathBuf) -> ExtendedInfoResult<GitStatus> {
        let repo = git2::Repository::discover(path)
            .map_err(|e| ExtendedInfoError::GitError(e))?;

        if path.is_dir() {
            return Ok(GitStatus::Clean);
        }

        let workdir = repo.workdir()
            .unwrap_or_else(|| Path::new("."));
        let relative = path.strip_prefix(workdir)
            .map_err(|e| ExtendedInfoError::StripPrefixError(e))?;
        let statuses = repo.statuses(None)
            .map_err(|e| ExtendedInfoError::GitError(e))?;

        let status = statuses
            .iter()
            .find(|e| e.path().map(|p| Path::new(p) == relative).unwrap_or(false))
            .map(|e| e.status())
            .unwrap_or(git2::Status::empty());

        let git_status = if status.is_conflicted() {
            GitStatus::Conflict
        } else if status.is_index_new() || status.is_index_modified() || status.is_index_deleted() {
            GitStatus::Staged
        } else if status.is_wt_modified() {
            GitStatus::Modified
        } else if status.is_wt_deleted() {
            GitStatus::Deleted
        } else if status.is_wt_new() {
            GitStatus::Untracked
        } else if status.is_ignored() {
            GitStatus::Ignored
        } else {
            GitStatus::Clean
        };

        Ok(git_status)
    }

}