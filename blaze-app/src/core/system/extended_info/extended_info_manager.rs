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




use std::{collections::HashMap, path::{Path, PathBuf}, sync::{Arc, RwLock}, time::UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use users::{get_group_by_gid, get_user_by_uid};
use uuid::Uuid;
use crate::{core::system::{cache::cache_manager::CacheManager, clipboard::TOKIO_RUNTIME}, utils::channel_pool::{FileOperation, NotifyingSender, with_channel_pool}};


pub enum ExtendedInfoMessages {
    StartScan(PathBuf),
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


pub struct ExtendedInfoManager {
    pub cache_manager: &'static CacheManager,
    pub info_map: Arc<RwLock<HashMap<PathBuf, ExtendedInfo>>>,
    pub semaphore: Arc<Semaphore>,
}


impl ExtendedInfoManager {
    pub fn new() -> Self {
        let cache_manager = CacheManager::global();
        TOKIO_RUNTIME.spawn(async move {
            cache_manager.load_extended_info_cache().await;
        });
        Self { 
            cache_manager, 
            info_map: Arc::new(RwLock::new(HashMap::new())),
            semaphore: Arc::new(tokio::sync::Semaphore::new(8)),
        }
    }

    fn get_real_mtime(path: &PathBuf) -> u64 {
        std::fs::metadata(path)
            .and_then(|m| m.modified())
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs())
            .unwrap_or(0)
    }

    pub fn process_messages(&self, active_id: Uuid, sender: NotifyingSender) {
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
                    let sender_clone = sender.clone();
                    let tab_id = sender.tab_id;

                    let guard = cm.extended_info_cache.read().unwrap();
                    let cache_valid = guard.get(key.as_ref())
                        .map(|c| c.modified == current_mtime)
                        .unwrap_or(false);

                    if cache_valid {
                        if let Some(cached) = cm.get_cached_extended_info(&path_buf) {
                            self.info_map.write().unwrap().insert(path_buf.clone(), cached.into());
                            sender_clone.send_fileop(FileOperation::ExtendedInfoReady {
                                full_path: path_buf,
                                tab_id,
                            }).ok();
                        }
                    } else {
                        let info_map = self.info_map.clone();
                        let path_to_task = path_buf.clone();
                        let sem = self.semaphore.clone();

                        TOKIO_RUNTIME.spawn(async move {
                            let _permit = sem.acquire_owned().await.unwrap();
                            let info = Self::scan(&path_buf).await;
                            info_map.write().unwrap().insert(path_buf.clone(), info.clone());

                            let mut cached: ExtendedInfoCache = info.into();
                            cached.modified = current_mtime;
                            
                            CacheManager::global().update_extended_info_cache(
                                path_to_task.to_string_lossy().into_owned(),
                                cached,
                            ).await;


                            sender_clone.send_fileop(FileOperation::ExtendedInfoReady {
                                full_path: path_buf,
                                tab_id,
                            }).ok();
                        });
                    }
                },
            }
        }
    }

    async fn scan(path: &PathBuf) -> ExtendedInfo {
        let m = match tokio::fs::symlink_metadata(path).await {
            Ok(m) => m,
            Err(_) => return ExtendedInfo::default(),
        };

        let symlink_target = if m.file_type().is_symlink() {
            tokio::fs::read_link(path).await.ok()
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
            }).await.unwrap_or((None, None))
        };

        #[cfg(not(unix))]
        let (owner, group_name) = (None, None);

        let dimensions = if m.is_file() {
            let path_clone = path.clone();
            tokio::task::spawn(async move {
                image::image_dimensions(&path_clone).ok()
            }).await.unwrap_or(None)
        } else {
            None
        };


        let git_status = {
            let path_clone = path.clone();
            tokio::task::spawn(async move {
                Self::get_git_status(&path_clone)
            }).await.unwrap_or(None)
        };


        ExtendedInfo {
            owner,
            group_name,
            symlink_target,
            dimensions,
            git_status,
        }
    }


    fn get_git_status(path: &PathBuf) -> Option<GitStatus> {
        let repo = git2::Repository::discover(path).ok()?;

        if path.is_dir() {
            return Some(GitStatus::Clean);
        }

        let relative = path.strip_prefix(repo.workdir()?).ok()?;
        let statuses = repo.statuses(None).ok()?;

        let status = statuses
            .iter()
            .find(|e| e.path().map(|p| Path::new(p) == relative).unwrap_or(false))?
            .status();

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

        Some(git_status)
    }

}