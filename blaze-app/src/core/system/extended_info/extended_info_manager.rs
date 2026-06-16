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
    system::{
        cache::cache_manager::CacheManager,
        clipboard::global_clipboard::TOKIO_RUNTIME,
        extended_info::error::{ExtendedInfoError, ExtendedInfoResult},
    },
};

use lru::LruCache;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    num::NonZeroUsize,
    time::{Duration, Instant},
};
use std::{
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::UNIX_EPOCH,
};
use tokio::sync::Semaphore;
use tracing::{error, warn};
use uuid::Uuid;
use uzers::{get_group_by_gid, get_user_by_uid};

pub enum ExtendedInfoMessages {
    StartScan(Arc<Path>),
    ForceScan(Arc<Path>),
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

impl GitStatus {
    fn priority(&self) -> u8 {
        match self {
            Self::Conflict => 6,
            Self::Staged => 5,
            Self::Modified => 4,
            Self::Deleted => 3,
            Self::Untracked => 2,
            Self::Ignored => 1,
            Self::Clean => 0,
        }
    }
}

const REPO_CACHE_TTL: Duration = Duration::from_secs(3);
struct RepoStatusCache {
    files: HashMap<PathBuf, GitStatus>,
    dirs: HashMap<PathBuf, GitStatus>,
    refreshed_at: Instant,
}

impl RepoStatusCache {
    fn build(repo: &git2::Repository) -> Self {
        let Ok(statuses) = repo.statuses(None) else {
            return Self::empty();
        };

        let mut files: HashMap<PathBuf, GitStatus> = HashMap::new();
        let mut dirs: HashMap<PathBuf, GitStatus> = HashMap::new();

        for entry in statuses.iter() {
            let status = entry.status();
            if status.is_empty() {
                continue;
            }

            let rel: PathBuf = match entry.path() {
                Ok(p) => PathBuf::from(p),
                Err(e) => {
                    warn!("Ha ocurrido un error: {e}");
                    continue;
                }
            };

            let git_status = Self::classify(status);

            files.insert(rel.clone(), git_status.clone());

            let mut current = rel.parent();
            while let Some(dir) = current {
                if dir == Path::new("") {
                    break;
                };
                let entry = dirs.entry(dir.to_path_buf()).or_insert(GitStatus::Clean);
                if git_status.priority() > entry.priority() {
                    *entry = git_status.clone();
                }
                current = dir.parent();
            }
        }

        Self {
            files,
            dirs,
            refreshed_at: Instant::now(),
        }
    }

    fn empty() -> Self {
        Self {
            files: HashMap::new(),
            dirs: HashMap::new(),
            refreshed_at: Instant::now(),
        }
    }

    fn classify(status: git2::Status) -> GitStatus {
        if status.is_conflicted() {
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
        }
    }

    fn is_stale(&self) -> bool {
        self.refreshed_at.elapsed() > REPO_CACHE_TTL
    }
}

const DEFAULT_INFO_CACHE_CAPACITY: usize = 2_000;
pub struct ExtendedInfoManager {
    pub cache_manager: &'static CacheManager,
    pub info_map: Arc<RwLock<LruCache<Arc<Path>, ExtendedInfo>>>,
    pub semaphore: Arc<Semaphore>,
    repo_cache: Arc<Mutex<HashMap<PathBuf, RepoStatusCache>>>,
}

impl ExtendedInfoManager {
    pub fn new() -> Self {
        let cache_manager = CacheManager::global();
        TOKIO_RUNTIME.spawn(async move {
            cache_manager.load_extended_info_cache().await;
        });

        Self::with_capacity(DEFAULT_INFO_CACHE_CAPACITY, cache_manager)
    }

    fn with_capacity(cap: usize, cache_manager: &'static CacheManager) -> Self {
        let def_cap = match NonZeroUsize::new(2000) {
            Some(cap) => cap,
            None => unreachable!(),
        };
        let cap = NonZeroUsize::new(cap).unwrap_or(def_cap);

        Self {
            cache_manager,
            info_map: Arc::new(RwLock::new(LruCache::new(cap))),
            semaphore: Arc::new(tokio::sync::Semaphore::new(8)),
            repo_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn clear_directory(&self, dir: &Path) {
        let Ok(mut guard) = self.info_map.write() else {
            return;
        };

        let to_move: Vec<Arc<Path>> = guard
            .iter()
            .filter(|(k, _)| k.parent() == Some(dir))
            .map(|(k, _)| k.clone())
            .collect();

        for key in to_move {
            guard.pop(&key);
        }
    }

    fn get_real_mtime(path: &Path) -> u64 {
        std::fs::metadata(path)
            .and_then(|m| m.modified())
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap().as_secs())
            .unwrap_or(0)
    }

    fn requst_scan(&self, path_buf: Arc<Path>, current_mtime: u64, sender: &Dispatcher) {
        let info_map = self.info_map.clone();
        let repo_cache = self.repo_cache.clone();
        let path_to_task = path_buf.clone();
        let sem = self.semaphore.clone();
        let sender = sender.clone();

        TOKIO_RUNTIME.spawn(async move {
            let _permit = match sem.acquire_owned().await {
                Ok(p) => p,
                Err(e) => {
                    error!("Semáforo cerrado: {}", e);
                    return;
                }
            };

            match Self::scan(path_buf.clone(), repo_cache).await {
                Ok(info) => {
                    match info_map.write() {
                        Ok(mut g) => {
                            g.put(path_buf.clone(), info.clone());

                            if let Some(git_status) = &info.git_status {
                                let mut current_parent = path_buf.parent();
                                while let Some(parent) = current_parent {
                                    let parent_arc: Arc<Path> = parent.into();
                                    let parent_info =
                                        g.get(&parent_arc).cloned().unwrap_or_default();

                                    let needs_update = match &parent_info.git_status {
                                        None => true,
                                        Some(existing) => {
                                            git_status.priority() > existing.priority()
                                        }
                                    };

                                    if needs_update {
                                        let mut updated_parent = parent_info;
                                        updated_parent.git_status = Some(git_status.clone());
                                        g.put(parent_arc.clone(), updated_parent);
                                    }

                                    current_parent = parent.parent();
                                }
                            }
                        }
                        Err(e) => {
                            warn!("info_map lock envenendado: {}", e);
                            return;
                        }
                    }

                    let mut cached: ExtendedInfoCache = info.into();
                    cached.modified = current_mtime;

                    CacheManager::global()
                        .update_extended_info_cache(
                            path_to_task.to_string_lossy().into_owned(),
                            cached,
                        )
                        .await;

                    if let Err(e) = sender.send(FileOperation::ExtendedInfoReady {
                        full_path: path_buf,
                    }) {
                        warn!("Error al enviar ExtendedInfo: {}", e);
                    }
                }
                Err(e) => {
                    warn!("Error escaneando {:?}: {e}", path_buf);
                }
            }
        });
    }

    pub fn process_messages(&self, active_id: Uuid, sender: Dispatcher) -> ExtendedInfoResult<()> {
        let messages: Vec<ExtendedInfoMessages> = with_event_bus(|pool| {
            let mut msgs = Vec::new();
            pool.drain(active_id, |msg| {
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

                    let is_dir = path_buf.is_dir();

                    let cache_valid = if is_dir {
                        false
                    } else {
                        match cm.extended_info_cache.try_read() {
                            Ok(g) => g
                                .get(key.as_ref())
                                .map(|c| c.modified == current_mtime)
                                .unwrap_or(false),
                            Err(e) => {
                                warn!("No se ha podido validar la caché de 'ExtendedInfo': {}", e);
                                false
                            }
                        }
                    };

                    if cache_valid {
                        if let Some(cached) = cm.get_cached_extended_info(&path_buf) {
                            match self.info_map.write() {
                                Ok(mut g) => {
                                    g.put(path_buf.clone(), cached.into());
                                }
                                Err(e) => {
                                    warn!("info_map lock envenendado: {}", e);
                                    return Err(ExtendedInfoError::PoisonedLock);
                                }
                            }

                            sender
                                .send(FileOperation::ExtendedInfoReady {
                                    full_path: path_buf,
                                })
                                .map_err(ExtendedInfoError::SendError)?;
                        }
                    } else {
                        self.requst_scan(path_buf, current_mtime, &sender);
                    }
                }

                ExtendedInfoMessages::ForceScan(path_buf) => {
                    let current_mtime = Self::get_real_mtime(&path_buf);

                    self.requst_scan(path_buf, current_mtime, &sender);
                }
            }
        }

        Ok(())
    }

    async fn scan(
        path: Arc<Path>,
        repo_cache: Arc<Mutex<HashMap<PathBuf, RepoStatusCache>>>,
    ) -> ExtendedInfoResult<ExtendedInfo> {
        let m = match tokio::fs::symlink_metadata(&path).await {
            Ok(m) => m,
            Err(e) => {
                warn!(
                    "No se ha podido leer metadata en 'ExtendedInfoManager::scan':  {}",
                    e
                );
                return Ok(ExtendedInfo::default());
            }
        };

        let symlink_target = if m.file_type().is_symlink() {
            match tokio::fs::read_link(path.clone()).await {
                Ok(p) => Some(p),
                Err(e) => {
                    warn!("Error leyendo los symlinks: {}", e);
                    return Err(ExtendedInfoError::SymlinkError);
                }
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
                let owner = get_user_by_uid(uid).map(|u| u.name().to_string_lossy().into_owned());
                let group_name =
                    get_group_by_gid(gid).map(|g| g.name().to_string_lossy().into_owned());
                (owner, group_name)
            })
            .await
            .map_err(ExtendedInfoError::ThreadError)?
        };

        #[cfg(not(unix))]
        let (owner, group_name) = (None, None);

        let dimensions = if m.is_file() {
            let is_image = path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| {
                    matches!(
                        e.to_lowercase().as_str(),
                        "jpg"
                            | "jpeg"
                            | "png"
                            | "gif"
                            | "webp"
                            | "bmp"
                            | "tiff"
                            | "tif"
                            | "ico"
                            | "avif"
                            | "qoi"
                    )
                })
                .unwrap_or(false);

            if is_image {
                let path_clone = path.clone();
                let file = std::fs::File::open(&path_clone)?;
                let dims = imagesize::reader_size(BufReader::new(file))
                    .map_err(|_| ExtendedInfoError::DimensionError)?;

                Some((dims.width as u32, dims.height as u32))
            } else {
                None
            }
        } else {
            None
        };

        let git_status = {
            let path_clone = path.clone();
            let cache_clone = repo_cache.clone();
            let res =
                tokio::task::spawn_blocking(move || Self::get_git_status(path_clone, &cache_clone))
                    .await
                    .map_err(ExtendedInfoError::ThreadError)?;

            match res {
                Ok(status) => Some(status),
                Err(ExtendedInfoError::GitError(_)) => None,
                Err(e) => return Err(e),
            }
        };

        Ok(ExtendedInfo {
            owner,
            group_name,
            symlink_target,
            dimensions,
            git_status,
        })
    }

    fn get_git_status(
        path: Arc<Path>,
        repo_cache: &Mutex<HashMap<PathBuf, RepoStatusCache>>,
    ) -> ExtendedInfoResult<GitStatus> {
        let repo = git2::Repository::discover(&path).map_err(ExtendedInfoError::GitError)?;

        if path.to_string_lossy().contains("/.git/") {
            return Ok(GitStatus::Clean);
        }

        let workdir = repo.workdir().unwrap_or(Path::new(".")).to_path_buf();

        let relative = path
            .strip_prefix(&workdir)
            .map_err(ExtendedInfoError::StripPrefixError)?
            .to_path_buf();

        let mut cache_guard = repo_cache.lock();
        let entry = cache_guard
            .entry(workdir)
            .or_insert_with(|| RepoStatusCache::build(&repo));

        if entry.is_stale() {
            *entry = RepoStatusCache::build(&repo);
        }

        let status = if path.is_dir() {
            entry
                .dirs
                .get(&relative)
                .cloned()
                .unwrap_or(GitStatus::Clean)
        } else {
            entry
                .files
                .get(&relative)
                .cloned()
                .unwrap_or(GitStatus::Clean)
        };

        Ok(status)
    }
}
