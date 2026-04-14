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





use std::{cell::RefCell, collections::{HashMap, HashSet}, path::PathBuf, rc::Rc, sync::{Arc, atomic::Ordering}, time::Instant};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use tracing::{debug, error, info, warn};
use tokio::sync::Mutex as TokioMutex;
use crate::{core::{configs::config_state::with_configs, files::{motor::{BlazeMotor, FileEntry, FileLoadingMessage, MOTOR}, recursive_search::RecursiveMessages}, system::{clipboard::{GlobalClipboard, TOKIO_RUNTIME}, fileopener_module::{FileOpenerManager, GLOBAL_FILE_OPENER}, sizer_manager::{self, sizer_manager::SizerManager}, updater::updater::Updater}}, ui::task_manager::task_manager::TaskManager, utils::channel_pool::{FileOperation, NotifyingSender, with_active_sender_for, with_channel_pool}};

pub struct RubberBand {
    pub rubber_band_start: Option<egui::Pos2>,
    pub rubber_band_current: Option<egui::Pos2>,
    pub is_rubber_banding: bool,
    pub rubber_band_start_content_y: f32,
}


pub struct RowView {
    pub is_dragging_files: bool,
    pub drag_ghost_pos: Option<egui::Pos2>,
    pub drop_target: Option<PathBuf>,
    pub drop_invalid_target: Option<PathBuf>,
    pub scroll_area_origin_y: f32,
}

#[derive(Clone, PartialEq)]
pub enum NewItemType {
    Folder,
    File,
}

pub struct BlazeCoreState {
    pub is_loading: bool,
    pub search_filter: String,
    pub clipboard: GlobalClipboard,
    pub selected_files: HashSet<PathBuf>,
    pub dirty_flag: bool,
    pub active_tasks: usize,
    pub motor: Rc<RefCell<BlazeMotor>>,

    pub file_opener_manager: Arc<TokioMutex<FileOpenerManager>>,

    pub last_selected_index: Option<usize>,
    pub pending_scroll_to: Option<usize>,
    pub scroll_offset: f32,
    pub rubber_band: RubberBand,
    pub row_view: RowView,

    pub renaming_file:Option<PathBuf>,
    pub rename_buffer: String,
    pub creating_new: Option<NewItemType>,
    pub new_item_buffer: String,
    pub focus_requested: bool, 

    pub cached_sender: Option<NotifyingSender>,

    pub updater: Updater,

    pub calculated_dir_sizes: HashSet<PathBuf>,
    pub calculating_dir_sizes: HashSet<PathBuf>,

    pub cwd_input: String,
    pub last_search_was_recursive: bool,
    pub dirty_tasks: bool,

    pub is_testing: bool,

    pub last_fs_event: Option<Instant>,
    pub task_manager: &'static TaskManager,

    pub sizer_manager: SizerManager,
}

impl BlazeCoreState {
    pub async fn new() -> Self {
        let motor = Rc::new(RefCell::new(BlazeMotor::new().await));

        MOTOR.with(|m|{
            *m.borrow_mut() = Some(motor.clone());
        });

        let rubber_band = RubberBand {
            rubber_band_start: None,
            rubber_band_current: None,
            is_rubber_banding: false,
            rubber_band_start_content_y: 0.0,
        };

        let row_view = RowView {
            is_dragging_files: false,
            drag_ghost_pos: None,
            drop_target: None,
            drop_invalid_target: None,
            scroll_area_origin_y: 0.0,
        };

        let file_opener_manager = GLOBAL_FILE_OPENER.clone();

        let task_manager = TaskManager::global();

        let sizer_manager = SizerManager::new();

        let mut state = Self {
            motor,
            is_loading: false,
            search_filter: String::new(),
            clipboard: GlobalClipboard::new(),
            selected_files: HashSet::new(),
            dirty_flag: false,
            dirty_tasks: false,
            active_tasks: 0,
            last_search_was_recursive: false,
            cwd_input: String::new(),
            last_selected_index: None,
            pending_scroll_to: None,
            scroll_offset: 0.0,
            rubber_band,
            row_view,

            renaming_file: None,
            rename_buffer: String::new(),
            creating_new: None,
            new_item_buffer: String::new(),
            focus_requested: false,

            cached_sender: None,

            updater: Updater::init(),

            calculating_dir_sizes: HashSet::new(),
            calculated_dir_sizes: HashSet::new(),

            file_opener_manager,

            is_testing: false,

            last_fs_event: None,
            task_manager,

            sizer_manager,
        };

        if let Some(sender) = state.sender().cloned() {
            state.motor.borrow_mut().active_tab().load_path(true, sender.clone());
            state.updater.check_for_update(sender);
        }

        state
    }

    

    pub fn sender(&mut self) -> Option<&NotifyingSender> {
        if self.cached_sender.is_none() {
            let tab_id = self.motor.borrow_mut().active_tab().id;
            self.cached_sender = with_active_sender_for(tab_id, |s| s.clone());
        }
        self.cached_sender.as_ref()
    }

    pub fn invalidate_seder(&mut self) {
        self.cached_sender = None;
    }


    pub fn clean_selections(&mut self) {
        self.selected_files.shrink_to_fit();
        self.selected_files.clear();
        self.last_selected_index = None;
    }


    pub fn navigate_to(&mut self, path: PathBuf) {
        self.motor.borrow_mut().active_tab_mut().navigate_to(path);
        if let Some(sender) = self.sender().cloned() {
            self.motor.borrow_mut().active_tab_mut().load_path(false, sender);
        }
        self.clean_selections();
    }

    pub fn up(&mut self) {
        self.motor.borrow_mut().active_tab_mut().up();
        if let Some(sender) = self.sender().cloned() {
            self.motor.borrow_mut().active_tab_mut().load_path(false, sender);
        }
        self.clean_selections();
    }

    pub fn back(&mut self) {
        self.motor.borrow_mut().active_tab_mut().back();
        if let Some(sender) = self.sender().cloned() {
            self.motor.borrow_mut().active_tab_mut().load_path(false, sender);
        }
        self.clean_selections();
    }

    pub fn forward(&mut self) {
        self.motor.borrow_mut().active_tab_mut().foward();
        if let Some(sender) = self.sender().cloned() {
            self.motor.borrow_mut().active_tab_mut().load_path(false, sender);
        }
        self.clean_selections();
    }

    pub fn refresh(&mut self) {
        if let Some(sender) = self.sender().cloned() {
            self.motor.borrow_mut().active_tab_mut().load_path(false, sender);
        }
        self.clean_selections();
    }


    pub fn selected_as_entries(&self, files: &Vec<Arc<FileEntry>>) -> Vec<Arc<FileEntry>> {
        files.iter()
            .filter(|f| self.selected_files.contains(&f.full_path))
            .cloned()
            .collect()
    }

    pub fn _get_file_entry(&self, path: &PathBuf) -> Option<Arc<FileEntry>> {
        self.motor.borrow_mut()
            .active_tab()
            .files
            .iter()
            .find(|f| f.full_path == *path)
            .cloned()
    }


    pub fn copy(&self, files: &Vec<Arc<FileEntry>>) {
        let cwd = self.motor.borrow_mut().active_tab().cwd.clone();
        let items = self.selected_as_entries(files);
        self.clipboard.copy_items(items, cwd);
    }

    pub fn cut(&self, files: &Vec<Arc<FileEntry>>) {
        let cwd = self.motor.borrow().tabs[self.motor.borrow().active_tab_index].cwd.clone();
        let items = self.selected_as_entries(files);
        self.clipboard.cut_items(items, cwd);
    }

    pub fn move_to_trash(&mut self, files: &Vec<Arc<FileEntry>>) {
        let cwd = self.motor.borrow().tabs[self.motor.borrow().active_tab_index].cwd.clone();
        let items = self.selected_as_entries(files);
        if let Some(sender) = self.sender().cloned() {
            self.clipboard.move_to_trash(items, cwd, sender).ok();
        }
    }

    pub fn move_files(&mut self, files: Vec<PathBuf>, dest: PathBuf) {
        if let Some(sender) = self.sender().cloned() {
            self.clipboard.move_files(files, dest, sender).ok();
        }
    }

    pub fn paste(&mut self, path: PathBuf) {
        self.clipboard.set_dest(path);
        if let Some(sender) = self.sender().cloned() {
            self.clipboard.paste(sender).ok();
        }
    }

    
    pub fn select_all(&mut self, files: &[Arc<FileEntry>]) {
        self.selected_files.clear();

        for file in files {
            self.selected_files.insert(file.full_path.clone());
        }

        if !files.is_empty() {
            self.last_selected_index = Some(files.len() - 1);
        } else {
            self.last_selected_index = None;
        }
    }

    pub fn toggle_select_all(&mut self, files: &[Arc<FileEntry>]) {
        let all_selected = files.iter().all(|f| self.selected_files.contains(&f.full_path));

        if all_selected {
            self.selected_files.clear();
            self.last_selected_index = None;
        } else {
            self.selected_files.clear();
            for file in files {
                self.selected_files.insert(file.full_path.clone());
            }
            self.last_selected_index = if files.is_empty() { None } else { Some(files.len() - 1) };
        }
    }


    pub fn active_files(&self) -> Vec<Arc<FileEntry>> {
        let mut motor = self.motor.borrow_mut();
        let tab = motor.active_tab_mut();
        let show_hidden = with_configs(|c| c.configs.show_hidden_files);

        let base: Vec<Arc<FileEntry>> = if tab.is_recursive_active {
            tab.recursive_entries.iter().cloned().collect()
        } else {
            tab.sorted_indices.iter().map(|&i| tab.files[i].clone()).collect()
        };

        let matcher = SkimMatcherV2::default();
        let query_lower = self.search_filter.to_lowercase();

        base.into_iter()
            .filter(|f| {
                if !show_hidden && f.is_hidden { return false; }
                if tab.is_recursive_active { return true; }
                if self.search_filter.is_empty() { return true;}
                matcher.fuzzy_match(&f.name.to_lowercase(), &query_lower).is_some()
            })
            .collect()
    }

    pub fn clean_search(&mut self) {
        let mut motor = self.motor.borrow_mut();
        motor.active_tab_mut().recursive_entries.clear();
        motor.active_tab_mut().is_recursive_active = false;
        self.search_filter.clear();
    }

    pub fn set_search(&mut self, query: String) {
        let was_recursive = self.search_filter.starts_with("rec:");
        let is_recursive = query.starts_with("rec:");

        if was_recursive && !is_recursive {
            self.clean_search();
            self.refresh();
        }

        if is_recursive {
            let clean = query.replacen("rec:", "", 1);
            if !clean.is_empty() {
                if let Some(sender) = self.sender().cloned() {
                    self.motor.borrow_mut().active_tab_mut().start_recursive_search(clean, 5, sender);
                }
            }
        }

        self.search_filter = query;
    }


    pub fn open_file_by_path(&mut self, path: PathBuf) {
        let manager_arc = self.file_opener_manager.clone();

        let Some(sender) = self.sender().cloned() else {return;};
        TOKIO_RUNTIME.spawn(async move {
            info!("intentando abrir");
            let mut manager = manager_arc.lock().await;
            manager.request_open_file(path, sender).await;
        });
    }

    pub fn open_file(&mut self, file:&Arc<FileEntry>) {
        let path = file.full_path.clone();
        let manager_arc = self.file_opener_manager.clone();
        
        let Some(sender) = self.sender().cloned() else {return;};

        TOKIO_RUNTIME.spawn(async move {
            info!("intentando abrir");
            let mut manager = manager_arc.lock().await;
            manager.request_open_file(path, sender).await;
        });
    }

    pub fn open_file_with(&mut self, file:&Arc<FileEntry>) {
        let path = file.full_path.clone();
        let manager_arc = self.file_opener_manager.clone();
        
        let Some(sender) = self.sender().cloned() else {return;};
        TOKIO_RUNTIME.spawn(async move {
            info!("intentando abrir");
            let mut manager = manager_arc.lock().await;

            manager.request_open_file_with(path, sender).await;
        });
    }


    pub fn process_messages(&mut self) {
        let active_id = {
            let mut motor = self.motor.borrow_mut();
            let tab = motor.active_tab();
            tab.id
        };

        self.task_manager.process_message(active_id);

        let sender = {
            self.sender().unwrap().clone()
        };

        self.sizer_manager.process_messages(active_id, sender);
        
        let file_messages: Vec<FileLoadingMessage> = with_channel_pool(|pool|{
            let mut msgs = Vec::new();
            pool.process_file_messages(active_id, |msg|{
                msgs.push(msg);
                true
            });
            msgs
        });


        let recursive_messages: Vec<RecursiveMessages> = with_channel_pool(|pool| {
            let mut msgs = Vec::new();
            pool.process_recursive_messages(active_id, |msg| {
                msgs.push(msg);
                true
            });
            msgs
        });

        let fileops_events: Vec<FileOperation> = with_channel_pool(|pool|{
            let mut msgs = Vec::new();
            pool.process_fileops_events(active_id, |msg|{
                msgs.push(msg);
                true
            });
            msgs
        });

        
        for msg in file_messages {
            match msg {
                FileLoadingMessage::Batch(gene, batch) => {
                    let mut motor = self.motor.borrow_mut();
                    let tab = motor.active_tab();


                    debug!("Batch recibido: generation={}, tamaño={}", gene, batch.len());
                    if gene == tab.loading_generation {
                        debug!("Batch aplicado a tab");
    
                        tab.files.extend(batch.iter().cloned());
                        let new_start = tab.sorted_indices.len();
                        tab.sorted_indices.extend(new_start..tab.files.len());
                        let ordering = with_configs(|ccfg| ccfg.configs.app_ordering_mode.clone());
                        tab.sort_indices(ordering);

                    } else {
                        warn!("Generation no coincide: esperado={}, recibido={}", tab.loading_generation, gene);
                    }
                },
                FileLoadingMessage::ProgressUpdate { total, done, text } => {
                    debug!("Progress: {} - {}", done as f32 / total as f32, text);
                },

                FileLoadingMessage::RecursiveBatch { generation, batch, source_dir:_ } => {
                    let mut motor = self.motor.borrow_mut();
                    let tab = motor.active_tab();

                    if generation == tab.loading_generation {
                        tab.recursive_entries.extend(batch);
                    }
                },

                FileLoadingMessage::Finished(gene) => {
                    let mut motor = self.motor.borrow_mut();
                    let tab = motor.active_tab();

                    debug!("Finished recibido: generation={}", gene);
                    if gene == tab.loading_generation {
                        debug!("Finished aplicado a tab");

                        tab.active_generation = gene;
                        tab.loading_flag.store(false, Ordering::Relaxed);
                        self.is_loading = false;

                        if tab.is_recursive_active {
                            self.dirty_flag = true;
                        } else {
                            tab.files.shrink_to_fit();
                            let ordering = with_configs(|cfg| cfg.configs.app_ordering_mode.clone());
                            tab.sort_indices(ordering);
                            tab.lower_names.clear();
                            tab.lower_names.extend(
                                tab.files.iter().enumerate()
                                    .map(|(i, e)| (i, e.name.to_lowercase()))
                            );
                            tab.lower_names.shrink_to_fit();
                        }
                    }

                },

                FileLoadingMessage::FileRemoved { name } => {
                    debug!("- Archivo eliminado: {}", name);
                    self.last_fs_event = Some(std::time::Instant::now());
                },
                FileLoadingMessage::FileAdded { name } => {
                    debug!("+ Archivo añadido: {}", name);
                    self.last_fs_event = Some(std::time::Instant::now());
                },
                FileLoadingMessage::FileModified { name } => {
                    debug!("Archivo {} modificado", name);
                    self.last_fs_event = Some(std::time::Instant::now());
                },
                FileLoadingMessage::FullRefresh => {
                    self.last_fs_event = Some(std::time::Instant::now());
                },

            }
        }


        for msg in recursive_messages {
            match msg {
                RecursiveMessages::Started { .. } => {
                    self.is_loading = true;
                },
                RecursiveMessages::Progress { .. } => {},
                RecursiveMessages::Finished { .. } => {
                    self.is_loading = false;
                },
            }
        }


        for msg in fileops_events {
            match msg {
                FileOperation::Copy { files, dest } => {

                }, 

                FileOperation::Delete { files } => {
                    let files: Vec<Arc<FileEntry>> = self.motor.borrow_mut()
                        .active_tab()
                        .files
                        .iter()
                        .filter(|f| files.contains(&f.full_path))
                        .cloned()
                        .collect();
                    self.move_to_trash(&files);
                },

                FileOperation::Move { files, dest, tab_id } => {
                    info!("Se intenta mover");
                    self.move_files(files, dest);
                },

                FileOperation::Update => {
                    self.updater.start_update_process();
                },

                FileOperation::UpdateDirSize { full_path, size, gene } => {
                    let mut motor = self.motor.borrow_mut();
                    let tab = motor.active_tab();
                    self.calculating_dir_sizes.remove(&full_path);
                    self.calculated_dir_sizes.insert(full_path.clone());

                    if let Some(file) = tab.files.iter_mut().find(|f| f.full_path == full_path) {
                        let file = Arc::make_mut(file);
                        file.size = size;
                    }

                    self.calculating_dir_sizes.remove(&full_path);
                },

                FileOperation::RestoreDeletedFiles { file_names } => {
                    let trash_path = self.motor.borrow_mut().get_trash_dir(None).unwrap_or_default();
                    
                    if let Some(trash_root) = trash_path.parent() {
                        if let Some(sender) = self.sender().cloned() {
                            self.clipboard.restore_from_trash(file_names, trash_root.to_path_buf(), sender).ok();
                        }
                    }
                },
            }
        }



        if let Some(last_event) = self.last_fs_event {
            if last_event.elapsed() > std::time::Duration::from_millis(50) {
                self.last_fs_event = None;
                if self.active_tasks == 0 {
                    if let Some(sender) = self.sender().cloned() {
                        self.motor.borrow_mut().active_tab().load_path(true, sender);
                    }
                }
            }
        }

    }
}
