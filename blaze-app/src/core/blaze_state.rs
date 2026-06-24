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

use crate::{
    core::{
        bootstrap::{
            configs::config_manager::with_configs,
            install_manager::installation_manager::with_installation_manager,
        },
        files::blaze_motor::{
            motor::{BlazeMotor, BlazeMotorBuilder, MOTOR},
            motor_structs::{FileEntry, FileLoadingMessage, RecursiveMessages},
        },
        runtime::{
            bus_structs::{FileOperation, UiEvent},
            event_bus::with_event_bus,
        },
        system::{
            cache::cache_manager::CacheManager,
            clipboard::global_clipboard::{GlobalClipboard, TOKIO_RUNTIME},
            extended_info::extended_info_manager::{ExtendedInfoManager, ExtendedInfoMessages},
            fileopener_module::{FileOpenerManager, GLOBAL_FILE_OPENER},
            operationstate::operation_manager::with_history,
            sizer_manager::manager::{SizerManager, SizerMessages},
            terminal_opener::terminal_manager::{TerminalManager, GLOBAL_TERMINAL_MANAGER},
            trash_manager::manager::{get_backend, TrashDestination},
            updater::updater_manager::Updater,
            zip_manager::manager::ZipManager,
        },
    },
    ui::task_manager::tasks::TaskManager,
};
use bitvec::vec::BitVec;
use egui::{pos2, Pos2};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::{
    cell::{RefCell, RefMut},
    collections::HashSet,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::sync::Mutex as TokioMutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// Para el guardado en caché
static LAST_SAVE_REQUEST: AtomicU64 = AtomicU64::new(0);

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum LayoutMode {
    Row,
    Grid,
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum ViewMode {
    Normal(LayoutMode),
    Tags(LayoutMode),
}

pub enum TagViewFilter {
    All { all_items_len: usize },
    Tag { name: String, items_len: usize },
}

pub struct RubberBand {
    pub rubber_band_start: Option<egui::Pos2>,
    pub rubber_band_current: Option<egui::Pos2>,
    pub is_rubber_banding: bool,
    pub rubber_band_start_content_y: f32,
}

pub struct RowView {
    pub is_dragging_files: bool,
    pub drag_ghost_pos: Option<egui::Pos2>,
    pub drop_target: Option<Arc<Path>>,
    pub drop_invalid_target: Option<Arc<Path>>,
    pub scroll_area_origin_y: f32,
    pub first_visible: usize,
    pub last_visible: usize,
    pub viewport_height: f32,
    pub icon_size: f32,
}

pub struct GridView {
    pub is_dragging_files: bool,
    pub drag_ghost_pos: Option<egui::Pos2>,
    pub drop_target: Option<Arc<Path>>,
    pub drop_invalid_target: Option<Arc<Path>>,
    pub scroll_area_origin_y: f32,
    pub first_visible: usize,
    pub last_visible: usize,
    pub viewport_height: f32,
    pub cols: usize,
    pub cell_size: f32,
    pub actual_origin: Pos2,
    pub row_height: f32,
    pub icon_size: f32,
}

#[derive(Clone, PartialEq)]
pub enum NewItemType {
    Folder,
    File,
}

#[must_use = "llama .build() para crear el state"]
pub struct BlazeCoreBuilder {
    start_path: Option<Arc<Path>>,
}

impl BlazeCoreBuilder {
    fn new() -> Self {
        Self { start_path: None }
    }

    pub fn with_start_path(mut self, path: Option<Arc<Path>>) -> Self {
        self.start_path = path;
        self
    }

    #[must_use]
    pub async fn build(self) -> BlazeCoreState {
        let motor = Rc::new(RefCell::new(
            BlazeMotorBuilder::default()
                .with_start_path(self.start_path)
                .build()
                .await,
        ));
        let active_id = motor.borrow().active_tab().id;

        //Asignar el id de la ventana inicial al active_id
        crate::core::runtime::event_bus::set_active_tab(active_id);

        MOTOR.with(|m| {
            *m.borrow_mut() = Some(motor.clone());
        });

        let rubber_band = RubberBand {
            rubber_band_start: None,
            rubber_band_current: None,
            is_rubber_banding: false,
            rubber_band_start_content_y: 0.0,
        };

        // Traer los tamaños desde las configs
        let (row_icon_size, grid_icon_size, view_mode) = with_configs(|c| {
            (
                c.get_row_icon_size(),
                c.get_grid_icon_size(),
                c.get_view_mode(),
            )
        });

        let row_view = RowView {
            is_dragging_files: false,
            drag_ghost_pos: None,
            drop_target: None,
            drop_invalid_target: None,
            scroll_area_origin_y: 0.0,
            first_visible: 0,
            last_visible: 0,
            viewport_height: 0.0,
            icon_size: row_icon_size,
        };

        let grid_view = GridView {
            is_dragging_files: false,
            drag_ghost_pos: None,
            drop_target: None,
            drop_invalid_target: None,
            scroll_area_origin_y: 0.0,
            first_visible: 0,
            last_visible: 0,
            viewport_height: 0.0,
            cols: 0,
            cell_size: 0.0,
            actual_origin: pos2(0.0, 0.0),
            row_height: 0.0,
            icon_size: grid_icon_size,
        };

        let file_opener_manager = GLOBAL_FILE_OPENER.clone();

        let task_manager = TaskManager::global();

        let sizer_manager = SizerManager::new();

        let terminal_manager = GLOBAL_TERMINAL_MANAGER.clone();

        let mut state = BlazeCoreState {
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
            grid_view,
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
            cwd: PathBuf::new().into(),
            last_navigation_time: None,
            navigation_cooldown: Duration::from_millis(100),
            view_mode,
            tag_filter: TagViewFilter::All { all_items_len: 0 },
        };

        let dispatcher = with_event_bus(|e| e.dispatcher(active_id));

        let new_cwd = {
            let mut motor = state.motor.borrow_mut();
            let tab = motor.active_tab_mut();

            if let Err(e) = tab.load_path(true, dispatcher.clone()) {
                warn!("Ha ocurrido un error al cargar los archivos: {}", e);
            }
            state.updater.check_for_update(dispatcher.clone());
            tab.cwd.clone()
        };

        state.cwd = new_cwd;

        let is_installed = with_installation_manager(|im| !im.is_installed());

        with_configs(|c| {
            let day_elapsed = match c.get_last_time_asked_install() {
                None => true,
                Some(time) => time.elapsed().unwrap_or_default() >= Duration::from_hours(24),
            };

            if is_installed && (c.get_should_ask_install() || day_elapsed) {
                dispatcher.send(UiEvent::ShowWantToInstall).ok();
            } else {
                info!(
                    "Instalado {} {} {}",
                    !is_installed,
                    c.get_should_ask_install(),
                    day_elapsed
                )
            }
        });

        state
    }
}

impl Default for BlazeCoreBuilder {
    fn default() -> Self {
        Self::new()
    }
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
    pub file_opener_manager: Arc<Mutex<FileOpenerManager>>,
    pub pending_scroll_to: Option<usize>,
    pub scroll_offset: f32,
    pub rubber_band: RubberBand,
    pub row_view: RowView,
    pub grid_view: GridView,
    pub renaming_file: Option<PathBuf>,
    pub rename_buffer: String,
    pub creating_new: Option<NewItemType>,
    pub new_item_buffer: String,
    pub focus_requested: bool,
    pub updater: Updater,
    pub calculated_dir_sizes: HashSet<Arc<Path>>,
    pub calculating_dir_sizes: HashSet<Arc<Path>>,
    pub last_fs_event: Option<Instant>,
    pub task_manager: &'static TaskManager,
    pub sizer_manager: SizerManager,
    pub needs_sort: bool,
    pub _cwd_input: String,
    pub terminal_manager: Arc<TokioMutex<TerminalManager>>,
    pub extended_info_manager: ExtendedInfoManager,
    pub calculating_extended_info: HashSet<Arc<Path>>,
    pub calculated_extended_info: HashSet<Arc<Path>>,
    pub zip_manager: ZipManager,
    pub cwd: Arc<Path>,
    last_navigation_time: Option<Instant>,
    navigation_cooldown: Duration,
    pub view_mode: ViewMode,
    pub tag_filter: TagViewFilter,
}

impl BlazeCoreState {
    pub fn get_active_files(&mut self) -> Vec<Arc<FileEntry>> {
        let motor = self.motor.borrow();
        let tab = motor.active_tab();

        match tab.get_active_files(&self.search_filter, self.needs_sort, &self.sizer_manager) {
            Ok(files) => {
                self.needs_sort = false;
                files
            }
            Err(e) => {
                warn!("Ha ocurrido un error obteniendo los archivos: {e}");
                vec![]
            }
        }
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

    pub fn get_selected_paths(&self, files: &[Arc<FileEntry>]) -> Vec<Arc<Path>> {
        files
            .iter()
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

    pub fn im_navigating(&self) -> bool {
        if let Some(time) = self.last_navigation_time {
            time.elapsed() < self.navigation_cooldown
        } else {
            false
        }
    }

    pub fn navigate_to(&mut self, path: Arc<Path>) {
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
        let prev_dir = self.cwd.clone();

        dispatcher.send(SizerMessages::CancelAll).ok();
        self.calculating_dir_sizes.clear();

        self.extended_info_manager.clear_directory(&prev_dir);
        self.motor.borrow_mut().active_tab_mut().navigate_to(path);
        self.save_caches(false);
        self.last_navigation_time = Some(Instant::now());

        self.refresh();
    }

    pub fn up(&mut self) {
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
        let prev_dir = self.cwd.clone();
        self.motor.borrow_mut().active_tab_mut().up();
        self.extended_info_manager.clear_directory(&prev_dir);
        dispatcher.send(SizerMessages::CancelAll).ok();
        self.refresh();
        self.save_caches(false);
    }

    pub fn can_go_up(&self) -> bool {
        self.motor.borrow().active_tab().can_go_up()
    }

    pub fn back(&mut self) {
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
        let prev_dir = self.cwd.clone();
        self.motor.borrow_mut().active_tab_mut().back();
        self.extended_info_manager.clear_directory(&prev_dir);
        dispatcher.send(SizerMessages::CancelAll).ok();
        self.refresh();
        self.save_caches(false);
    }

    pub fn can_go_back(&self) -> bool {
        self.motor.borrow().active_tab().can_go_back()
    }

    pub fn forward(&mut self) {
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
        let prev_dir = self.cwd.clone();
        self.motor.borrow_mut().active_tab_mut().forward();
        self.extended_info_manager.clear_directory(&prev_dir);
        dispatcher.send(SizerMessages::CancelAll).ok();
        self.refresh();
        self.save_caches(false);
    }

    pub fn can_go_forward(&self) -> bool {
        self.motor.borrow().active_tab().can_go_forward()
    }

    pub fn refresh(&mut self) {
        self.clean_search();
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
        dispatcher.send(SizerMessages::CancelAll).ok();

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
            return vec![];
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

    #[allow(unused)]
    pub fn clear_clipboard(&self) {
        match self.clipboard.clear() {
            Ok(_) => {
                info!("Se limpia el clipboard");
            }
            Err(e) => warn!("Eror en clipboard: {e}"),
        }
    }

    pub fn copy(&self, files: &[Arc<FileEntry>]) {
        let cwd = self.motor.borrow_mut().active_tab().cwd.clone();
        let items = self.selected_as_entries(files);
        match self.clipboard.copy_items(items, cwd) {
            Ok(_) => {
                info!("Se copia");
            }
            Err(e) => warn!("Eror en clipboard: {e}"),
        }
    }

    pub fn cut(&self, files: &[Arc<FileEntry>]) {
        let cwd = self.motor.borrow().tabs[self.motor.borrow().active_tab_index]
            .cwd
            .clone();
        let items = self.selected_as_entries(files);
        match self.clipboard.cut_items(items, cwd) {
            Ok(_) => {
                info!("Se corta");
            }
            Err(e) => warn!("Eror en clipboard: {e}"),
        }
    }

    pub fn rename(&self, file_name: &str) {
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
        if let Err(e) = self
            .clipboard
            .rename_file(file_name, &self.rename_buffer, &dispatcher)
        {
            error!("Error renombrando: {}", e);
        }
    }

    pub fn create_new(&self, nit: NewItemType) {
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));

        let res = match nit {
            NewItemType::File => {
                self.clipboard
                    .create_new_file(&self.new_item_buffer, self.cwd.clone(), &dispatcher)
            }
            NewItemType::Folder => {
                self.clipboard
                    .create_new_dir(&self.new_item_buffer, self.cwd.clone(), &dispatcher)
            }
        };

        if let Err(e) = res {
            warn!("Ha ocurrido un error en el clipboard: {e}");
        }
    }

    pub fn move_to_trash(&mut self, items: Vec<(Arc<str>, Arc<Path>)>) {
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
        self.clipboard.move_to_trash(items, &dispatcher).ok();
    }

    fn move_to_trash_event_only(&self, items: Vec<(Arc<str>, Arc<Path>)>) {
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
        self.clipboard.move_to_trash(items, &dispatcher).ok();
    }

    pub fn move_files(&mut self, sources: Vec<Arc<Path>>, dest: Arc<Path>) {
        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));

        match self.clipboard.move_files(sources, dest, &dispatcher) {
            Ok(_) => {}
            Err(e) => warn!("Ha ocurrido un error al mover: {e}"),
        }
    }

    pub fn paste(&mut self, path: Arc<Path>) {
        match self.clipboard.set_dest(path) {
            Ok(_) => {}
            Err(e) => {
                warn!("Eror en clipboard: {e}");
                return;
            }
        }

        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));

        match self.clipboard.paste(&dispatcher) {
            Ok(_) => {}
            Err(e) => warn!("Eror en clipboard: {e}"),
        }
    }

    pub fn clean_search(&mut self) {
        let mut motor = self.motor.borrow_mut();
        if let Err(e) = motor.active_tab_mut().clear_recursive_files() {
            warn!("Ha ocirrido un error: {}", e);
        }
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
                if let Err(e) = self
                    .motor
                    .borrow_mut()
                    .active_tab_mut()
                    .start_recursive_search(clean, 30, dispatcher)
                {
                    warn!("Ha ocirrido un error: {}", e);
                }
            }
        }

        self.search_filter = query;
    }

    pub fn open_file_by_path(&mut self, path: Arc<Path>) {
        let manager_arc = self.file_opener_manager.clone();
        info!("intentando abrir");
        let mut manager = manager_arc.lock();
        match manager.open_file(path) {
            Ok(_) => {}
            Err(e) => {
                warn!("Ha fallado la apertura del archivo: {}", e);
            }
        }
    }

    pub fn open_file(&mut self, file: &Arc<FileEntry>) {
        let path = file.full_path.to_owned();
        let manager_arc = self.file_opener_manager.clone();
        info!("intentando abrir");
        let mut manager = manager_arc.lock();
        match manager.open_file(path) {
            Ok(_) => {}
            Err(e) => {
                warn!("Ha fallado la apertura del archivo: {}", e);
            }
        }
    }

    pub fn open_file_with(&mut self, file: &Arc<FileEntry>) {
        let path = file.full_path.to_owned();
        info!("intentando abrir");

        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
        dispatcher.send(UiEvent::OpenWithSelector { path }).ok();
    }

    pub fn open_terminal_here(&self) {
        let cwd = self.motor.borrow_mut().active_tab().cwd.clone();

        let preferred_terminal = with_configs(|c| {
            if c.get_default_terminal().trim().is_empty() {
                None
            } else {
                Some(c.get_default_terminal())
            }
        });

        let tm_manager = self.terminal_manager.clone();
        TOKIO_RUNTIME.spawn(async move {
            let mut tm_manager = tm_manager.lock().await;
            if let Err(e) = tm_manager
                .request_open_terminal(&cwd, preferred_terminal)
                .await
            {
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

    pub fn close_tab(&mut self, index: usize) -> bool {
        let (new_id, closed) = {
            let mut motor = self.motor_mut();
            let closed = motor.close_tab(index).is_ok();
            let new_id = motor.active_tab().id;
            (new_id, closed)
        };
        self.active_id = new_id;
        self.refresh();
        closed
    }

    pub fn add_tab_from_file(&mut self, tab_path: &Path) {
        let new_id = {
            let mut motor = self.motor_mut();
            motor.add_tab(tab_path)
        };
        let Some(new_id) = new_id else {
            return;
        };
        self.active_id = new_id;
        self.refresh();
    }

    pub fn create_tab(&mut self) {
        let new_id = {
            let mut motor = self.motor_mut();
            motor.create_tab()
        };
        let Some(new_id) = new_id else {
            return;
        };
        self.active_id = new_id;
    }

    pub fn tab_title(&mut self, index: usize) -> String {
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

        self.sizer_manager
            .process_messages(active_id, dispatcher.clone());

        if let Err(e) = self
            .extended_info_manager
            .process_messages(active_id, dispatcher.clone())
        {
            warn!("Error procesando mensajes de ExtendedInfo: {}", e);
        }

        let file_messages: Vec<FileLoadingMessage> = with_event_bus(|pool| {
            let mut msgs = Vec::new();
            pool.drain(active_id, |msg| {
                msgs.push(msg);
                true
            });
            msgs
        });

        let recursive_messages: Vec<RecursiveMessages> = with_event_bus(|pool| {
            let mut msgs = Vec::new();
            pool.drain(active_id, |msg| {
                msgs.push(msg);
                true
            });
            msgs
        });

        let fileops_events: Vec<FileOperation> = with_event_bus(|pool| {
            let mut msgs = Vec::new();
            pool.drain(active_id, |msg| {
                msgs.push(msg);
                true
            });
            msgs
        });

        for msg in file_messages {
            match msg {
                FileLoadingMessage::Batch(gene, batch) => {
                    let motor = self.motor.borrow();
                    let tab = motor.active_tab();

                    debug!(
                        "Batch recibido: generation={}, tamaño={}",
                        gene,
                        batch.len()
                    );

                    if gene != tab.loading_generation {
                        warn!(
                            "Generation no coincide: esperado={}, recibido={}",
                            tab.loading_generation, gene
                        );

                        return;
                    }
                    debug!("Batch aplicado a tab");

                    {
                        let mut files_guard = match tab.files.write() {
                            Ok(guard) => guard,
                            Err(e) => {
                                warn!("Lock envenenado: {}", e);
                                return;
                            }
                        };

                        let mut indices_guard = match tab.sorted_indices.write() {
                            Ok(guard) => guard,
                            Err(e) => {
                                warn!("Lock envenenado: {}", e);
                                return;
                            }
                        };

                        let start = files_guard.len();

                        files_guard.extend(batch.iter().cloned());

                        indices_guard.extend(start..files_guard.len());
                    }

                    self.needs_sort = true;
                }
                FileLoadingMessage::ProgressUpdate { total, done, text } => {
                    debug!("Progress: {} - {}", done as f32 / total as f32, text);
                }

                FileLoadingMessage::RecursiveBatch {
                    generation,
                    batch,
                    source_dir: _,
                } => {
                    let mut motor = self.motor.borrow_mut();
                    let tab = motor.active_tab_mut();

                    if generation == tab.loading_generation {
                        {
                            let mut recursive_entries_guard = match tab.recursive_entries.write() {
                                Ok(guard) => guard,
                                Err(e) => {
                                    warn!("Lock envenenado: {}", e);
                                    return;
                                }
                            };

                            recursive_entries_guard.extend(batch);
                        }
                    }
                }

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
                            {
                                let mut files_guard = match tab.files.write() {
                                    Ok(guard) => guard,
                                    Err(e) => {
                                        warn!("Lock envenenado: {}", e);
                                        return;
                                    }
                                };

                                files_guard.shrink_to_fit();

                                tab.lower_names.clear();
                                tab.lower_names.extend(
                                    files_guard
                                        .iter()
                                        .enumerate()
                                        .map(|(i, e)| (i, e.name.to_lowercase().into_boxed_str())),
                                );
                                tab.lower_names.shrink_to_fit();
                            }
                            self.needs_sort = true;
                        }
                    }
                }

                FileLoadingMessage::FileRemoved { name } => {
                    debug!("- Archivo eliminado: {}", name);
                    self.last_fs_event = Some(std::time::Instant::now());
                }
                FileLoadingMessage::FileAdded { name } => {
                    debug!("+ Archivo añadido: {}", name);
                    self.last_fs_event = Some(std::time::Instant::now());
                }
                FileLoadingMessage::FileModified { name } => {
                    debug!("Archivo {} modificado", name);
                    self.last_fs_event = Some(std::time::Instant::now());
                }
                FileLoadingMessage::FullRefresh => {
                    debug!("FullRefresh solicitado");
                    self.last_fs_event = Some(std::time::Instant::now());
                }

                FileLoadingMessage::GitStatusChanged => {
                    let motor = self.motor.borrow();
                    let tab = motor.active_tab();

                    {
                        let files_guard = match tab.files.read() {
                            Ok(guard) => guard,
                            Err(e) => {
                                warn!("Lock envenenado: {}", e);
                                return;
                            }
                        };

                        let paths: Vec<Arc<Path>> =
                            files_guard.iter().map(|f| f.full_path.clone()).collect();

                        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
                        for path in paths {
                            dispatcher.send(ExtendedInfoMessages::ForceScan(path)).ok();
                        }
                    }
                }
            }
        }

        for msg in recursive_messages {
            match &msg {
                RecursiveMessages::Started { .. } => {
                    debug!("Started: {:?}", msg);
                    self.is_loading = true;
                }
                RecursiveMessages::Progress { .. } => {
                    debug!("Progress: {:?}", msg);
                }
                RecursiveMessages::Finished { .. } => {
                    debug!("Finished: {:?}", msg);
                    self.is_loading = false;
                }
            }
        }

        for msg in fileops_events {
            match msg {
                // Operaciones de Archivos
                // __--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--
                FileOperation::PasteCut { .. } => {
                    with_history(|h| h.push_completed(&msg));
                }

                FileOperation::PasteCopy { .. } => {
                    with_history(|h| h.push_completed(&msg));
                }

                FileOperation::Rename { .. } => {
                    with_history(|h| h.push_completed(&msg));
                }

                FileOperation::CreateDir { .. } => {
                    with_history(|h| h.push_completed(&msg));
                }

                FileOperation::CreateFile { .. } => {
                    with_history(|h| h.push_completed(&msg));
                }

                FileOperation::Move { sources, dest, .. } => {
                    self.move_files(sources, dest);
                }

                FileOperation::Trash { files } => {
                    let motor = self.motor.borrow();
                    let tab = motor.active_tab();

                    if let Ok(ftd) = tab.get_item_to_delete(files) {
                        self.move_to_trash_event_only(ftd);
                    }
                }

                FileOperation::RestoreDeletedFiles { file_names } => {
                    let Some(trash_path) =
                        get_backend().get_trash_files(&TrashDestination::Home).ok()
                    else {
                        return;
                    };

                    if let Some(trash_root) = trash_path.parent() {
                        let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
                        self.clipboard
                            .restore_from_trash(file_names, trash_root.into(), dispatcher)
                            .ok();
                    }
                }

                FileOperation::ExtractHere { entry, dest_dir } => {
                    info!(
                        "Solicitando extracción de: [{}] -> [{:?}]",
                        entry.name, dest_dir
                    );
                    let res = self.zip_manager.extract(&entry, &dest_dir);
                    res.map_err(|e| warn!("Error: {}", e)).ok();
                }

                // Operaciones Extra
                // __--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--__--
                FileOperation::Update => {
                    self.updater.start_update_process();
                }

                FileOperation::UpdateDirSize {
                    full_path,
                    size,
                    tab_id,
                } => {
                    let mut motor = self.motor.borrow_mut();

                    if let Some(tab) = motor.tabs.iter_mut().find(|t| t.id == tab_id) {
                        if let Ok(e) = tab.update_dir_size(full_path.to_owned(), size) {
                            if e {
                                self.calculating_dir_sizes.remove(&full_path);
                                self.calculated_dir_sizes.insert(full_path);
                                self.needs_sort = true;
                            }
                        }
                    }
                }

                FileOperation::ExtendedInfoReady { full_path } => {
                    self.calculating_extended_info.remove(&full_path);
                    self.calculated_extended_info.insert(full_path);
                }

                FileOperation::NavigateTo(path) => {
                    let current_layout = match &self.view_mode {
                        ViewMode::Normal(layout) => layout,
                        ViewMode::Tags(layout) => layout,
                    };

                    self.view_mode = ViewMode::Normal(current_layout.to_owned());
                    self.navigate_to(path);
                }

                FileOperation::OpenFileByPath(path) => {
                    self.open_file_by_path(path);
                }
            }
        }

        if let Some(last_event) = self.last_fs_event {
            if last_event.elapsed() > std::time::Duration::from_millis(50) {
                self.last_fs_event = None;
                if self.active_tasks == 0 {
                    let dispatcher = with_event_bus(|e| e.dispatcher(self.active_id));
                    self.calculating_dir_sizes.clear();
                    self.calculated_dir_sizes.clear();

                    if let Err(e) = self
                        .motor
                        .borrow_mut()
                        .active_tab_mut()
                        .load_path(true, dispatcher)
                    {
                        warn!("Ha ocurrido un error al cargar los archivos: {}", e);
                    }
                }
            }
        }
    }
}
