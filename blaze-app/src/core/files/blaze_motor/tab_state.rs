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

use file_id::get_file_id;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use jwalk::{Parallelism, WalkDir};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use tracing::{debug, warn};
use uuid::Uuid;

use crate::core::bootstrap::configs::config_manager::with_configs;
use crate::core::bootstrap::configs::platform::linux::conf_structs::{
    OrderingDirection, OrderingKind,
};
use crate::core::files::blaze_motor::blaze_loader::BlazeLoader;
use crate::core::files::blaze_motor::error::{MotorError, MotorResult};
use crate::core::files::blaze_motor::motor_structs::{
    FileEntry, FileLoadingMessage, RecursiveMessages,
};
use crate::core::files::blaze_motor::utilities::build_entry;
use crate::core::files::blaze_motor::watcher::FileWatcher;
use crate::core::runtime::bus_structs::UiEvent;
use crate::core::runtime::event_bus::{Dispatcher, with_event_bus};
use crate::core::system::clipboard::global_clipboard::TOKIO_RUNTIME;
use crate::core::system::knowndirs::knowndirs_manager::KnownDirsManager;
use crate::core::system::sizer_manager::manager::SizerManager;

static NEXT_TASK: AtomicU64 = AtomicU64::new(1);
pub fn new_task_id() -> u64 {
    NEXT_TASK.fetch_add(1, Ordering::Relaxed)
}

#[must_use = "llama .build() para construir la tab"]
pub struct BlazeTabBuilder {
    start_path: Arc<Path>,
    tab_id: Uuid,
}

impl BlazeTabBuilder {
    pub fn new() -> Self {
        Self {
            start_path: KnownDirsManager::get().home.clone(),
            tab_id: Uuid::new_v4(),
        }
    }

    pub fn with_start_path(mut self, path: Arc<Path>) -> Self {
        self.start_path = path;
        self
    }

    pub fn with_uuid(mut self, id: Uuid) -> Self {
        self.tab_id = id;
        self
    }

    #[must_use]
    pub fn build(self) -> BlazeTabState {
        // Crear dispatcher para la tab
        with_event_bus(|bus| {
            bus.create_tab(self.tab_id);
        });

        BlazeTabState {
            id: self.tab_id,
            cwd: self.start_path,
            history: Vec::new(),
            future: Vec::new(),
            files: Arc::new(RwLock::new(Vec::new())),
            loading_flag: Arc::new(AtomicBool::new(false)),
            lower_names: Vec::new(),
            loading_generation: 0,
            active_generation: 0,
            sorted_indices: Arc::new(RwLock::new(Vec::new())),
            recursive_entries: Arc::new(RwLock::new(Vec::new())),
            is_recursive_active: false,
            watcher: FileWatcher::start(),
            loader: BlazeLoader::default(),
        }
    }
}

impl Default for BlazeTabBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BlazeTabState {
    pub id: Uuid,
    pub cwd: Arc<Path>,
    pub history: Vec<Arc<Path>>,
    pub future: Vec<Arc<Path>>,
    pub loading_flag: Arc<AtomicBool>,

    pub lower_names: Vec<(usize, Box<str>)>,
    pub loading_generation: u64,
    pub active_generation: u64,

    pub files: Arc<RwLock<Vec<Arc<FileEntry>>>>,
    pub sorted_indices: Arc<RwLock<Vec<usize>>>,

    pub recursive_entries: Arc<RwLock<Vec<Arc<FileEntry>>>>,
    pub is_recursive_active: bool,

    pub watcher: FileWatcher,
    pub loader: BlazeLoader,
}

impl BlazeTabState {
    pub fn get_active_files(
        &self,
        search_filter: &str,
        needs_sort: bool,
        sizer_manager: &SizerManager,
    ) -> MotorResult<Vec<Arc<FileEntry>>> {
        let show_hidden = with_configs(|c| c.get_show_hidden_files());
        let query_lower = search_filter.to_lowercase();
        let matcher = SkimMatcherV2::default();

        if self.is_recursive_active {
            let recursive_guard = self
                .recursive_entries
                .read()
                .map_err(|_| MotorError::PoisonedLock)?;

            let result = recursive_guard
                .iter()
                .filter(|f| {
                    if !show_hidden && f.is_hidden {
                        return false;
                    }
                    true
                })
                .cloned()
                .collect();

            return Ok(result);
        }

        self.ensure_sorted(needs_sort, sizer_manager)?;

        let file_guard = self.files.read().map_err(|_| MotorError::PoisonedLock)?;

        let indices_guard = self
            .sorted_indices
            .read()
            .map_err(|_| MotorError::PoisonedLock)?;

        let sorted = indices_guard
            .iter()
            .map(|&i| file_guard[i].clone())
            .filter(|f| {
                if !show_hidden && f.is_hidden {
                    return false;
                }
                if search_filter.is_empty() {
                    return true;
                }
                matcher
                    .fuzzy_match(&f.name.to_lowercase(), &query_lower)
                    .is_some()
            })
            .collect();

        Ok(sorted)
    }

    fn ensure_sorted(&self, needs_sort: bool, sizer_manager: &SizerManager) -> MotorResult<()> {
        if !needs_sort {
            return Ok(());
        }

        let mode = with_configs(|c| c.get_ordering_mode());

        let file_guard = self.files.write().map_err(|_| MotorError::PoisonedLock)?;
        let mut indices_guard = self
            .sorted_indices
            .write()
            .map_err(|_| MotorError::PoisonedLock)?;

        let mut indices: Vec<usize> = (0..file_guard.len()).collect();

        indices.sort_by(|&a, &b| {
            let (ea, eb) = (&file_guard[a], &file_guard[b]);

            // Carpetas primero
            match (ea.is_dir(), eb.is_dir()) {
                (true, false) => return std::cmp::Ordering::Less,
                (false, true) => return std::cmp::Ordering::Greater,
                _ => {}
            }

            let ord = match mode.kind {
                OrderingKind::Size => {
                    let (sa, sb) = (
                        self.get_effective_size(ea, sizer_manager),
                        self.get_effective_size(eb, sizer_manager),
                    );
                    sa.cmp(&sb)
                }
                OrderingKind::Name => ea.name.to_lowercase().cmp(&eb.name.to_lowercase()),
                OrderingKind::Date => ea.modified.cmp(&eb.modified),
            };

            // Invertir si es descendente
            if mode.direction == OrderingDirection::Desc {
                ord.reverse()
            } else {
                ord
            }
        });

        *indices_guard = indices;
        Ok(())
    }

    fn get_effective_size(&self, entry: &FileEntry, sizer_manager: &SizerManager) -> u64 {
        if !entry.is_dir() {
            return entry.size;
        }
        let key = entry.full_path.to_string_lossy();
        sizer_manager
            .cache_manager
            .size_cache
            .read()
            .get(key.as_ref())
            .map(|c| c.size)
            .unwrap_or(0)
    }

    pub fn get_item_to_delete(
        &self,
        files: Vec<Arc<Path>>,
    ) -> MotorResult<Vec<(Arc<str>, Arc<Path>)>> {
        let file_guard = self.files.read().map_err(|_| MotorError::PoisonedLock)?;

        let ftd = file_guard
            .iter()
            .filter(|f| files.contains(&f.full_path))
            .map(|f| (Arc::from(f.name.to_owned()), f.full_path.to_owned()))
            .collect();

        Ok(ftd)
    }

    pub fn update_dir_size(&self, full_path: Arc<Path>, new_size: u64) -> MotorResult<bool> {
        let mut guard = self.files.write().map_err(|_| MotorError::PoisonedLock)?;

        if let Some(entry) = guard
            .iter_mut()
            .find(|f| *f.full_path.as_ref() == *full_path)
        {
            let mut new_entry = (**entry).clone();
            new_entry.size = new_size;
            *entry = Arc::new(new_entry);
        } else {
            return Ok(false);
        }
        Ok(true)
    }

    pub fn clear_recursive_files(&self) -> MotorResult<()> {
        {
            let mut recursive_entries_guard = self
                .recursive_entries
                .write()
                .map_err(|_| MotorError::PoisonedLock)?;
            recursive_entries_guard.clear();
            recursive_entries_guard.shrink_to_fit();
        }
        Ok(())
    }

    pub fn clear_files(&self) -> MotorResult<()> {
        {
            let mut file_guard = self.files.write().map_err(|_| MotorError::PoisonedLock)?;
            file_guard.clear();
        }
        Ok(())
    }

    pub fn clear_sorted_indices(&self) -> MotorResult<()> {
        {
            let mut sorted_indices_guard = self
                .sorted_indices
                .write()
                .map_err(|_| MotorError::PoisonedLock)?;
            sorted_indices_guard.clear();
        }
        Ok(())
    }

    pub fn reset_for_new_path(&mut self) -> MotorResult<()> {
        self.clear_files()?;
        self.clear_sorted_indices()?;
        self.clear_recursive_files()?;
        self.lower_names.clear();
        Ok(())
    }

    pub fn load_path(&mut self, _skip_cache: bool, sender: Dispatcher) -> MotorResult<()> {
        let path = self.cwd.clone();

        if !path.exists() || !path.is_dir() {
            return Err(MotorError::InvalidPath(path));
        }

        self.loading_generation += 1;

        self.reset_for_new_path()?;

        self.active_generation = 0;
        self.loader
            .load_path(path.clone(), sender.clone(), self.loading_generation)?;

        self.watcher.start_watching(path, sender)
    }

    fn recursive_search(
        cwd: Arc<Path>,
        query: String,
        max_depth: usize,
        sender: Dispatcher,
        show_hidden: bool,
        loading_generation: u64,
        flag: Arc<AtomicBool>,
    ) {
        TOKIO_RUNTIME.spawn(async move {
            let query_lower = query.to_lowercase().trim().to_string();
            let mut total_files = 0usize;
            let mut batch: Vec<Arc<FileEntry>> = Vec::with_capacity(150);

            sender
                .send(RecursiveMessages::Started {
                    task_id: loading_generation,
                    text: format!("Buscando \"{}\"...", query),
                })
                .ok();

            let cwd_clone = cwd.clone();
            let flag_clone = flag.clone();
            let sender_clone = sender.clone();

            let walk_result = tokio::task::spawn_blocking(move || {
                let walker = WalkDir::new(&cwd_clone)
                    .max_depth(max_depth)
                    .follow_links(false)
                    .skip_hidden(!show_hidden)
                    .parallelism(Parallelism::RayonNewPool(0));

                for entry in walker {
                    if !flag_clone.load(Ordering::Relaxed) {
                        return (vec![], total_files);
                    }

                    let entry = match entry {
                        Ok(e) => e,
                        Err(e) => {
                            warn!("Error caminando: {}", e);
                            continue;
                        }
                    };

                    let path = entry.path();

                    if entry.file_type().is_dir() {
                        continue;
                    }

                    if !show_hidden
                        && let Some(name) = path.file_name()
                        && name.to_string_lossy().starts_with('.')
                    {
                        continue;
                    }

                    let name = entry.file_name().to_string_lossy().to_string();
                    let name_lower = name.to_lowercase();

                    let is_match = query_lower.is_empty() || name_lower.contains(&query_lower) || {
                        let name_norm = name_lower.replace(['-', '_', ' ', '.'], "");
                        let query_norm = query_lower.replace(['-', '_', ' ', '.'], "");
                        name_norm.contains(&query_norm)
                    };

                    if is_match && let Ok(metadata) = entry.metadata() {
                        let entry_path = path.to_path_buf();
                        let unique_id = get_file_id(&entry_path).ok();

                        let file_entry = build_entry(&entry_path, metadata, unique_id);

                        let arc_entry = Arc::from(file_entry);

                        batch.push(arc_entry);
                        total_files += 1;

                        if batch.len() >= 150 {
                            let send_batch = std::mem::take(&mut batch);
                            sender_clone
                                .send(FileLoadingMessage::RecursiveBatch {
                                    generation: loading_generation,
                                    batch: send_batch,
                                    source_dir: cwd_clone.clone(),
                                })
                                .ok();
                        }
                    }
                }
                (batch, total_files)
            })
            .await;

            match walk_result {
                Ok((remaining_batch, found_total)) => {
                    total_files = found_total;

                    if !remaining_batch.is_empty() {
                        sender
                            .send(FileLoadingMessage::RecursiveBatch {
                                generation: loading_generation,
                                batch: remaining_batch,
                                source_dir: cwd,
                            })
                            .ok();
                    }

                    sender
                        .send(RecursiveMessages::Finished {
                            task_id: loading_generation,
                            success: true,
                            text: format!("Completado: {} archivos encontrados", total_files),
                        })
                        .ok();

                    debug!("Búsqueda recursiva completada: {} archivos", total_files);
                }
                Err(e) => {
                    sender
                        .send(UiEvent::ShowError(
                            format!("Error buscando archivos: {}", e).into(),
                        ))
                        .ok();
                }
            }

            flag.store(false, std::sync::atomic::Ordering::Relaxed);
            debug!("Búsqueda recursiva completada: {} archivos", total_files);
        });
    }

    pub fn start_recursive_search(
        &mut self,
        query: String,
        max_depth: usize,
        sender: Dispatcher,
    ) -> MotorResult<()> {
        {
            let mut recursive_entries_guard = self
                .recursive_entries
                .write()
                .map_err(|_| MotorError::PoisonedLock)?;
            recursive_entries_guard.clear();
            recursive_entries_guard.shrink_to_fit();
        }

        self.is_recursive_active = true;

        self.loading_generation += 1;
        let current_generation = self.loading_generation;
        self.loading_flag.store(true, Ordering::Relaxed);

        let path = self.cwd.clone();
        let flag = self.loading_flag.clone();

        let show_hidden = with_configs(|c| c.get_show_hidden_files());

        Self::recursive_search(
            path,
            query,
            max_depth,
            sender,
            show_hidden,
            current_generation,
            flag,
        );

        Ok(())
    }

    pub fn navigate_to(&mut self, new_path: Arc<Path>) {
        if new_path.is_dir() && new_path != self.cwd {
            let old_path = self.cwd.clone();
            if self.history.last() != Some(&old_path) {
                self.history.push(old_path);
            }

            self.future.clear();

            if self.history.len() > 100 {
                self.history.remove(0);
            }

            self.cwd = new_path;
        }
    }

    pub fn up(&mut self) {
        if let Some(new_path) = self.cwd.parent() {
            let old_path = self.cwd.clone();

            if *new_path == *old_path {
                return;
            }

            self.history.push(old_path.clone());

            self.future.clear();
            self.future.push(old_path);

            if self.history.len() > 100 {
                self.history.remove(0);
            }

            self.cwd = new_path.into();
        }
    }

    pub fn back(&mut self) {
        if let Some(prev) = self.history.pop() {
            self.future.push(self.cwd.clone());
            self.cwd = prev;
        }
    }

    pub fn forward(&mut self) {
        if let Some(next) = self.future.pop() {
            self.history.push(self.cwd.clone());
            self.cwd = next;
        }
    }

    pub fn can_go_back(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn can_go_forward(&self) -> bool {
        !self.future.is_empty()
    }

    pub fn can_go_up(&self) -> bool {
        match self.cwd.parent() {
            Some(parent) => parent != self.cwd.iter().as_path(),
            None => false,
        }
    }
}
