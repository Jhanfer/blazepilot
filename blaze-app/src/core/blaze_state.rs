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





use std::{cell::{RefCell, RefMut}, collections::HashSet, path::PathBuf, rc::Rc, sync::{Arc, atomic::{AtomicU64, Ordering}}, time::{Instant, SystemTime, UNIX_EPOCH}};
use bitvec::vec::BitVec;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use tracing::{debug, error, info, warn};
use tokio::sync::Mutex as TokioMutex;
use uuid::Uuid;
use crate::{core::{configs::config_state::{OrderingMode, with_configs}, files::blaze_motor::{motor::{BlazeMotor, MOTOR}, motor_structs::{FileEntry, FileLoadingMessage, RecursiveMessages}}, runtime::{bus_structs::FileOperation, event_bus::with_event_bus}, system::{cache::cache_manager::CacheManager, clipboard::{GlobalClipboard, TOKIO_RUNTIME}, extended_info::extended_info_manager::{ExtendedInfoManager, ExtendedInfoMessages}, fileopener_module::{FileOpenerManager, GLOBAL_FILE_OPENER}, sizer_manager::sizer_manager::SizerManager, terminal_opener::terminal_manager::{GLOBAL_TERMINAL_MANAGER, TerminalManager}, updater::updater::Updater, zip_manager::zip_manager::ZipManager}}, ui::task_manager::task_manager::TaskManager};


// Para el guardado en caché
static LAST_SAVE_REQUEST: AtomicU64 = AtomicU64::new(0);


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
    pub first_visible: usize,
    pub last_visible: usize,
}

#[derive(Clone, PartialEq)]
pub enum NewItemType {
    Folder,
    File,
}

pub struct BlazeCoreState {
    pub active_id: Uuid,
    pub is_loading: bool,
    pub search_filter: String,
    pub clipboard: GlobalClipboard,
    pub last_selected_index: Option<usize>,
    pub select_all_mode: bool,
    pub selection_anchor: Option<usize>,
    pub selection: BitVec,
    pub active_tasks: usize,
    pub motor: Rc<RefCell<BlazeMotor>>,
    pub file_opener_manager: Arc<TokioMutex<FileOpenerManager>>,
    pub pending_scroll_to: Option<usize>,
    pub scroll_offset: f32,
    pub rubber_band: RubberBand,
    pub row_view: RowView,
    pub renaming_file:Option<PathBuf>,
    pub rename_buffer: String,
    pub creating_new: Option<NewItemType>,
    pub new_item_buffer: String,
    pub focus_requested: bool,
    pub updater: Updater,
    pub calculated_dir_sizes: HashSet<PathBuf>,
    pub calculating_dir_sizes: HashSet<PathBuf>,
    pub last_fs_event: Option<Instant>,
    pub task_manager: &'static TaskManager,
    pub sizer_manager: SizerManager,
    pub needs_sort: bool,
    pub _cwd_input: String,
    pub terminal_manager: Arc<TokioMutex<TerminalManager>>,
    pub extended_info_manager: ExtendedInfoManager,
    pub calculating_extended_info: HashSet<PathBuf>,
    pub calculated_extended_info: HashSet<PathBuf>,
    pub zip_manager: ZipManager,
    pub cwd: PathBuf,
}

impl BlazeCoreState {
    pub async fn new() -> Self {
        let motor = Rc::new(RefCell::new(BlazeMotor::new().await));
        let active_id = motor.borrow().active_tab().id.clone();

        //Asignar el id de la ventana inicial al active_id
        crate::core::runtime::event_bus::set_active_tab(active_id);

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
            first_visible: 0,
            last_visible: 0,
        };

        let file_opener_manager = GLOBAL_FILE_OPENER.clone();

        let task_manager = TaskManager::global();

        let sizer_manager = SizerManager::new();

        let terminal_manager = GLOBAL_TERMINAL_MANAGER.clone();

        let mut state = Self {
            active_id,
            motor,
            is_loading: false,
            search_filter: String::new(),
            clipboard: GlobalClipboard::new(),
            last_selected_index: None,
            selection_anchor: None,
            selection: BitVec::new(),
            select_all_mode: false,
            active_tasks: 0,
            pending_scroll_to: None,
            scroll_offset: 0.0,
            rubber_band,
            row_view,
            renaming_file: None,
            rename_buffer: String::new(),
            creating_new: None,
            new_item_buffer: String::new(),
            focus_requested: false,
            updater: Updater::init(),
            calculating_dir_sizes: HashSet::new(),
            calculated_dir_sizes: HashSet::new(),
            file_opener_manager,
            last_fs_event: None,
            task_manager,
            sizer_manager,
            needs_sort: false,
            _cwd_input: String::new(),
            terminal_manager,
            extended_info_manager: ExtendedInfoManager::new(),
            calculating_extended_info: HashSet::new(),
            calculated_extended_info: HashSet::new(),
            zip_manager: ZipManager::new(),
            cwd: PathBuf::new(),
        };


        let new_cwd = {
            let mut motor = state.motor.borrow_mut();
            let tab = motor.active_tab_mut();
            let dispatcher = with_event_bus(|e| e.dispatcher(active_id));

            if let Err(e) = tab.load_path(true, dispatcher.clone()) {
                warn!("Ha ocurrido un error al cargar los archivos: {}", e);
            }
            state.updater.check_for_update(dispatcher.clone());
            tab.cwd.clone()
        };

        state.cwd = new_cwd;

        state
    }

    pub fn sort_indices(&mut self, files: &mut Vec<Arc<FileEntry>>) -> Vec<Arc<FileEntry>> {
        let new_mode = with_configs(|cfg| cfg.configs.app_ordering_mode.clone());
        let by_size = matches!(new_mode, OrderingMode::SizeAsc | OrderingMode::SizeDesc);
        
        if by_size {
            files.sort_by(|a, b|{
                match (a.is_dir, b.is_dir) {
                    (true, false) => return std::cmp::Ordering::Less,
                    (false, true) => return std::cmp::Ordering::Greater,
                    _ => (),
                }

                let size_a = if a.is_dir {
                    let key = &a.full_path.to_string_lossy();
                    self.sizer_manager.cache_manager.size_cache
                        .try_read()
                        .ok()
                        .and_then(|g|{
                            g.get(key.as_ref()).map(|c| c.size)
                        })
                        .unwrap_or(0)
                } else {
                    a.size
                };

                let size_b = if b.is_dir {
                    let key = &b.full_path.to_string_lossy();
                    self.sizer_manager.cache_manager.size_cache
                        .try_read()
                        .ok()
                        .and_then(|g|{
                            g.get(key.as_ref()).map(|c| c.size)
                        })
                        .unwrap_or(0)
                } else {
                    b.size
                };

                match new_mode {
                    OrderingMode::SizeAsc => size_a.cmp(&size_b),
                    OrderingMode::SizeDesc => size_b.cmp(&size_a),
                    OrderingMode::Az => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    OrderingMode::Za => b.name.to_lowercase().cmp(&a.name.to_lowercase()),
                    OrderingMode::DateAsc => a.modified.cmp(&b.modified),
                    OrderingMode::DateDesc => b.modified.cmp(&a.modified),
                }
            });
        } else {
            files.sort_by(|a, b|{
                match (a.is_dir, b.is_dir) {
                    (true, false) => return std::cmp::Ordering::Less,
                    (false, true) => return std::cmp::Ordering::Greater,
                    _ => {}
                }

                match new_mode {
                    OrderingMode::Az => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    OrderingMode::Za => b.name.to_lowercase().cmp(&a.name.to_lowercase()),
                    OrderingMode::DateAsc => a.modified.cmp(&b.modified),
                    OrderingMode::DateDesc => b.modified.cmp(&a.modified),
                    _ => std::cmp::Ordering::Equal,
                }
            });
        }

        files.to_vec()
    }

    pub fn is_selected(&self, index: usize) -> bool {
        if index >= self.selection.len() {
            return false;
        }

        if self.select_all_mode {
            !self.selection[index]
        } else {
            self.selection[index]
        }
    }


    pub fn select_all(&mut self, files_len: usize) {
        self.select_all_mode = true;
        self.selection.clear();
        self.resize_selection(files_len);
        self.last_selected_index = if files_len > 0 {
            Some(files_len - 1)
        } else {
            None
        };
    }


    pub fn deselect_all(&mut self) {
        self.select_all_mode = false;
        self.selection.clear();
        self.last_selected_index = None;
        self.selection_anchor = None;
    }

    pub fn toggle_select_all(&mut self, files_len: usize) {
        if self.select_all_mode && self.selection.not_any() {
            self.deselect_all();
        } else {
            self.select_all(files_len);
        }
    }

    pub fn select_range(&mut self, start: usize, end: usize) {
        let start = start.min(end);
        let end = start.max(end);

        if start >= self.selection.len() {
            return;
        }
        let end = end.min(self.selection.len() - 1);

        if self.select_all_mode {
            for i in start..=end {
                self.selection.set(i, false);
            }
        } else {
            for i in start..=end {
                self.selection.set(i, true);
            }
        }
    }


    pub fn selected_count(&self, files_len: usize) -> usize {
        if self.select_all_mode {
            files_len - self.selection.count_ones()
        } else {
            self.selection.count_ones()
        }
    }

    pub fn get_selected_paths(&self, files: &[Arc<FileEntry>]) -> Vec<PathBuf> {
        files.iter()
            .enumerate()
            .filter(|(i, _)| self.is_selected(*i))
            .map(|(_, f)| f.full_path.clone())
            .collect()
    }


    pub fn resize_selection(&mut self, new_len: usize) {
        if self.selection.len() != new_len {
            self.selection.resize(new_len, false);
        }
    }

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

    pub fn navigate_to(&mut self, path: PathBuf) {
        let prev_dir = self.cwd.clone();
        self.motor.borrow_mut().active_tab_mut().navigate_to(path);
        self.extended_info_manager.clear_directory(&prev_dir);
        self.refresh();
        self.save_caches(false);
    }

    pub fn up(&mut self) {
        let prev_dir = self.cwd.clone();
        self.motor.borrow_mut().active_tab_mut().up();
        self.extended_info_manager.clear_directory(&prev_dir);
        self.refresh();
        self.save_caches(false);
    }

    pub fn back(&mut self) {
        let prev_dir = self.cwd.clone();
        self.motor.borrow_mut().active_tab_mut().back();
        self.extended_info_manager.clear_directory(&prev_dir);
        self.refresh();
        self.save_caches(false);
    }

    pub fn forward(&mut self) {
        let prev_dir = self.cwd.clone();
        self.motor.borrow_mut().active_tab_mut().forward();
        self.extended_info_manager.clear_directory(&prev_dir);
        self.refresh();
        self.save_caches(false);
    }

    pub fn refresh(&mut self) {
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
        let new_cwd = {
            let mut motor = self.motor.borrow_mut();
            let tab = motor.active_tab_mut();
            if let Err(e) = tab.load_path(false, dispatcher.clone()) {
                warn!("Ha ocurrido un error al cargar los archivos: {}", e);
            }
            tab.cwd.clone()
        };
        self.calculating_dir_sizes.clear();
        self.calculated_dir_sizes.clear();
        self.calculating_extended_info.clear();
        self.calculated_extended_info.clear();
        self.deselect_all();
        self.cwd = new_cwd;
    }


    pub fn selected_as_entries(&self, files: &[Arc<FileEntry>]) -> Vec<Arc<FileEntry>> {
        if files.is_empty() {
            return  vec![];
        }

        let mut result = Vec::with_capacity(files.len() / 2);

        if self.select_all_mode {
            for (i, file) in files.iter().enumerate() {
                if !self.selection.get(i).map(|b| *b).unwrap_or(false) {
                    result.push(file.clone());
                }
            }
        } else {
            for (i, file) in files.iter().enumerate() {
                if self.selection.get(i).map(|b| *b).unwrap_or(false) {
                    result.push(file.clone());
                }
            }
        }
        result
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
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
        self.clipboard.move_to_trash(items, cwd,&dispatcher).ok();
    }

    fn move_to_trash_event_only(&mut self, items: Vec<Arc<FileEntry>>) {
        let cwd = self.motor.borrow().tabs[self.motor.borrow().active_tab_index].cwd.clone();
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
        self.clipboard.move_to_trash(items, cwd, &dispatcher).ok();
    }

    pub fn move_files(&mut self, files: Vec<PathBuf>, dest: PathBuf) {
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
        self.clipboard.move_files(files, dest, &dispatcher).ok();
    }

    pub fn paste(&mut self, path: PathBuf) {
        self.clipboard.set_dest(path);
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
        self.clipboard.paste(&dispatcher).ok();
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
            if clean.len() >= 2 && clean != self.search_filter.replacen("rec:", "", 1) {
                let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
                self.motor.borrow_mut().active_tab_mut().start_recursive_search(clean, 30, dispatcher);
            }
        }

        self.search_filter = query;
    }


    pub fn open_file_by_path(&mut self, path: PathBuf) {
        let manager_arc = self.file_opener_manager.clone();

        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
        TOKIO_RUNTIME.spawn(async move {
            info!("intentando abrir");
            let mut manager = manager_arc.lock().await;
            manager.request_open_file(path, dispatcher).await;
        });
    }

    pub fn open_file(&mut self, file:&Arc<FileEntry>) {
        let path = file.full_path.clone();
        let manager_arc = self.file_opener_manager.clone();
        
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));

        TOKIO_RUNTIME.spawn(async move {
            info!("intentando abrir");
            let mut manager = manager_arc.lock().await;
            manager.request_open_file(path, dispatcher).await;
        });
    }

    pub fn open_file_with(&mut self, file:&Arc<FileEntry>) {
        let path = file.full_path.clone();
        let manager_arc = self.file_opener_manager.clone();
        
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
        TOKIO_RUNTIME.spawn(async move {
            info!("intentando abrir");
            let mut manager = manager_arc.lock().await;

            manager.request_open_file_with(path, dispatcher).await;
        });
    }


    pub fn open_terminal_here(&self) {
        let cwd = self.motor.borrow_mut().active_tab().cwd.clone();

        let preferred_terminal = with_configs(|c| {
            if c.configs.default_terminal.trim().is_empty() {
                None
            } else {
                Some(c.configs.default_terminal.clone())
            }
        });

        let tm_manager = self.terminal_manager.clone();
        TOKIO_RUNTIME.spawn(async move {
            let mut tm_manager = tm_manager.lock().await;
            if let Err(e) = tm_manager.request_open_terminal(&cwd, preferred_terminal).await {
                error!("No se pudo abrir la terminal: {}", e);
            }
        });
    }
    

    pub fn motor_mut(&mut self) -> RefMut<'_, BlazeMotor> {
        self.motor.borrow_mut()
    }


    pub fn switch_to_tab(&mut self, index: usize) {
        let new_id = {
            let mut motor = self.motor_mut();
            motor.switch_to_tab(index);
            motor.active_tab().id
        };
        self.active_id = new_id;
    }

    pub fn next_tab(&mut self) {
        let new_id = {
            let mut motor = self.motor_mut();
            motor.next_tab();
            motor.active_tab().id
        };
        self.active_id = new_id;
    }

    pub fn prev_tab(&mut self) {
        let new_id = {
            let mut motor = self.motor_mut();
            motor.prev_tab();
            motor.active_tab().id
        };
        self.active_id = new_id;
    }
    
    pub fn close_tab(&mut self, index:usize) -> bool {
        let (new_id, closed) = {
            let mut motor = self.motor_mut();
            (motor.active_tab().id, motor.close_tab(index))
        };
        self.active_id = new_id;
        closed
    }

    pub fn add_tab_from_file(&mut self, tab_path: PathBuf) {
        let new_id = {
            let mut motor = self.motor_mut();
            motor.add_tab(tab_path);
            motor.active_tab().id
        };
        self.active_id = new_id;
    }

    pub fn create_tab(&mut self) {
        let new_id = {
            let mut motor = self.motor_mut();
            motor.create_tab();
            motor.active_tab().id
        };
        self.active_id = new_id;
    }

    pub fn tab_title(&mut self, index:usize) -> String {
        self.motor_mut().tab_title(index)
    }


    pub fn process_messages(&mut self) {
        let active_id = {
            let motor = self.motor.borrow();
            let tab = motor.active_tab();
            tab.id
        };

        self.task_manager.process_message(active_id);

        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));

        self.sizer_manager.process_messages(active_id, dispatcher.clone());

        if let Err(e) = self.extended_info_manager.process_messages(active_id, dispatcher.clone()) {
            warn!("Error procesando mensajes de ExtendedInfo: {}",e);
        }
        
        let file_messages: Vec<FileLoadingMessage> = with_event_bus(|pool|{
            let mut msgs = Vec::new();
            pool.drain(active_id, |msg|{
                msgs.push(msg);
                true
            });
            msgs
        });


        let recursive_messages: Vec<RecursiveMessages> = with_event_bus(|pool| {
            let mut msgs = Vec::new();
            pool.drain(active_id, |msg|{
                msgs.push(msg);
                true
            });
            msgs
        });

        let fileops_events: Vec<FileOperation> = with_event_bus(|pool|{
            let mut msgs = Vec::new();
            pool.drain(active_id, |msg|{
                msgs.push(msg);
                true
            });
            msgs
        });

        
        for msg in file_messages {
            match msg {
                FileLoadingMessage::Batch(gene, batch) => {
                    let mut motor = self.motor.borrow_mut();
                    let tab = motor.active_tab_mut();


                    debug!("Batch recibido: generation={}, tamaño={}", gene, batch.len());
                    if gene == tab.loading_generation {
                        debug!("Batch aplicado a tab");
    
                        tab.files.extend(batch.iter().cloned());
                        let new_start = tab.sorted_indices.len();
                        tab.sorted_indices.extend(new_start..tab.files.len());                        
                        self.needs_sort = true;

                    } else {
                        warn!("Generation no coincide: esperado={}, recibido={}", tab.loading_generation, gene);
                    }
                },
                FileLoadingMessage::ProgressUpdate { total, done, text } => {
                    debug!("Progress: {} - {}", done as f32 / total as f32, text);
                },

                FileLoadingMessage::RecursiveBatch { generation, batch, source_dir:_ } => {
                    let mut motor = self.motor.borrow_mut();
                    let tab = motor.active_tab_mut();

                    if generation == tab.loading_generation {
                        tab.recursive_entries.extend(batch);
                    }
                },

                FileLoadingMessage::Finished(gene) => {
                    let mut motor = self.motor.borrow_mut();
                    let tab = motor.active_tab_mut();

                    debug!("Finished recibido: generation={}", gene);
                    if gene == tab.loading_generation {
                        debug!("Finished aplicado a tab");

                        tab.active_generation = gene;
                        tab.loading_flag.store(false, Ordering::Relaxed);
                        self.is_loading = false;

                        if tab.is_recursive_active {
                            
                        } else {
                            tab.files.shrink_to_fit();
                            tab.lower_names.clear();
                            tab.lower_names.extend(
                                tab.files.iter().enumerate()
                                    .map(|(i, e)| (i, e.name.to_lowercase()))
                            );
                            tab.lower_names.shrink_to_fit();

                            self.needs_sort = true;
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
                    debug!("FullRefresh solicitado");
                    self.last_fs_event = Some(std::time::Instant::now());
                },

                FileLoadingMessage::GitStatusChanged => {
                    let paths: Vec<PathBuf> = self.motor.borrow()
                        .active_tab()
                        .files
                        .iter()
                        .map(|f| f.full_path.clone())
                        .collect();

                    let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
                    for path in paths {
                        dispatcher.send(
                            ExtendedInfoMessages::ForceScan(path)
                        ).ok();
                    }
                
                }
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
                FileOperation::Copy { files:_, dest:_ } => {}, 
                FileOperation::Delete { files } => {
                    let files: Vec<Arc<FileEntry>> = self.motor.borrow_mut()
                        .active_tab_mut()
                        .files
                        .iter()
                        .filter(|f| files.contains(&f.full_path))
                        .cloned()
                        .collect();
                    self.move_to_trash_event_only(files);
                },

                FileOperation::Move { files, dest, tab_id:_ } => {
                    info!("Se intenta mover");
                    self.move_files(files, dest);
                },

                FileOperation::Update => {
                    self.updater.start_update_process();
                },

                FileOperation::UpdateDirSize { full_path, size, tab_id } => {
                    let mut motor = self.motor.borrow_mut();
                    if let Some(tab) = motor.tabs.iter_mut().find(|t| t.id == tab_id) {
                        if let Some((idx, _)) = tab.files.iter().enumerate().find(|(_, f)| f.full_path == full_path) {
                            let mut new_file = (*tab.files[idx]).clone();
                            new_file.size = size;
                            tab.files[idx] = Arc::new(new_file);
                        }
                        
                        self.calculating_dir_sizes.remove(&full_path);
                        self.calculated_dir_sizes.insert(full_path.clone());
                        self.needs_sort = true;
                    }
                },

                FileOperation::ExtendedInfoReady { full_path, tab_id:_ } => {
                    self.calculating_extended_info.remove(&full_path);
                    self.calculated_extended_info.insert(full_path);
                },

                FileOperation::RestoreDeletedFiles { file_names } => {
                    let trash_path = self.motor.borrow_mut().get_trash_dir(None).unwrap_or_default();
                    
                    if let Some(trash_root) = trash_path.parent() {
                        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
                        self.clipboard.restore_from_trash(file_names, trash_root.to_path_buf(), dispatcher).ok();
                    }
                },


                FileOperation::ExtractHere {entry, dest_dir} => {
                    info!("Solicitando extracción de: [{}] -> [{:?}]", entry.name, dest_dir);
                    let res = self.zip_manager.extract(&entry, &dest_dir);
                    res.map_err(|e| warn!("Error: {}", e)).ok();
                },
            }
        }



        if let Some(last_event) = self.last_fs_event {
            if last_event.elapsed() > std::time::Duration::from_millis(50) {
                self.last_fs_event = None;
                if self.active_tasks == 0 {
                    
                    let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
                    self.calculating_dir_sizes.clear();
                    self.calculated_dir_sizes.clear();

                    if let Err(e) = self.motor.borrow_mut().active_tab_mut().load_path(true, dispatcher) {
                        warn!("Ha ocurrido un error al cargar los archivos: {}", e);
                    }
                    
                }
            }
        }

    }
}
