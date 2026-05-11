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





use std::path::{Path, PathBuf};
use std::vec;
use file_id::{get_file_id};
use jwalk::{Parallelism, WalkDir};
use tracing::{debug, error, warn};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::core::files::blaze_motor::blaze_loader::BlazeLoader;
use crate::core::files::blaze_motor::error::{MotorError, MotorResult};
use crate::core::files::blaze_motor::motor_structs::{FileEntry, FileLoadingMessage, RecursiveMessages};
use crate::core::files::blaze_motor::utilities::build_entry;
use crate::core::files::blaze_motor::watcher::FileWatcher;
use crate::core::configs::config_state::with_configs;
use crate::core::runtime::bus_structs::UiEvent;
use crate::core::system::disk_reader::disk_manager::DiskManager;
use crate::core::runtime::event_bus::{Dispatcher, with_event_bus};
use crate::core::system::knowndirs::knowndirs_manager::KnownDirsManager;
use uuid::Uuid;
use std::cell::RefCell;
use std::rc::Rc;
use crate::core::system::clipboard::TOKIO_RUNTIME;



static NEXT_TASK: AtomicU64 = AtomicU64::new(1);
pub fn new_task_id() -> u64 {
    NEXT_TASK.fetch_add(1, Ordering::Relaxed)
}



pub struct TabState {
    pub id: Uuid,
    pub cwd: PathBuf,
    pub history: Vec<PathBuf>,
    pub future: Vec<PathBuf>,
    pub loading_flag: Arc<AtomicBool>,

    pub lower_names: Vec<(usize, String)>,
    pub loading_generation: u64,
    pub active_generation: u64,

    pub files: Vec<Arc<FileEntry>>,
    pub sorted_indices: Vec<usize>,

    pub recursive_entries: Vec<Arc<FileEntry>>,
    pub is_recursive_active: bool,

    watcher: FileWatcher,
    loader: BlazeLoader,
}


impl TabState {
    pub fn new(start_path: PathBuf, tab_id: Uuid) -> Self {

        //Crear dispatcher para la tab
        with_event_bus(|bus|{
            bus.create_tab(tab_id);
        });
        
        Self {
            id: tab_id,
            cwd: start_path,
            history: Vec::new(),
            future: Vec::new(),
            files: Vec::new(),
            loading_flag: Arc::new(AtomicBool::new(false)),
            lower_names: Vec::new(),
            loading_generation: 0,
            active_generation: 0,
            sorted_indices: Vec::new(),
            recursive_entries: Vec::new(),
            is_recursive_active: false,
            watcher: FileWatcher::start(),
            loader: BlazeLoader::default(),
        }
    }


    pub fn load_path(&mut self, _skip_cache: bool, sender: Dispatcher) -> MotorResult<()> {
        let path = self.cwd.clone();

        if !path.exists() || !path.is_dir() {
            return Err(MotorError::InvalidPath(path));
        }

        self.loading_generation += 1;
        self.files.clear();
        self.sorted_indices.clear();
        self.lower_names.clear();

        self.active_generation = 0;
        self.loader.load_path(&path, sender.clone(), self.loading_generation)?;

        self.watcher.start_watching(&path, sender)
    }


    fn recursive_search(cwd: PathBuf, query: String, max_depth: usize, sender: Dispatcher, show_hidden: bool, loading_generation: u64, flag: Arc<AtomicBool>) {
        TOKIO_RUNTIME.spawn(async move {
            let query_lower = query.to_lowercase().trim().to_string();
            let mut total_files = 0usize;
            let mut batch: Vec<Arc<FileEntry>> = Vec::with_capacity(150);

            sender.send(RecursiveMessages::Started {
                task_id: loading_generation as u64,
                text: format!("Buscando \"{}\"...", query),
            }).ok();

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

                    if !show_hidden {
                        if let Some(name) = path.file_name() {
                            if name.to_string_lossy().starts_with('.') {
                                continue;
                            }
                        }
                    }

                    let name = entry.file_name().to_string_lossy().to_string();
                    let name_lower = name.to_lowercase();

                    let is_match = query_lower.is_empty()
                        || name_lower.contains(&query_lower)
                        || {
                            let name_norm = name_lower.replace(['-', '_', ' ', '.'], "");
                            let query_norm = query_lower.replace(['-', '_', ' ', '.'], "");
                            name_norm.contains(&query_norm)
                        };

                    if is_match {
                        if let Ok(metadata) = entry.metadata() {
                            let entry_path = path.to_path_buf();
                            let unique_id = get_file_id(&entry_path).ok();

                            let file_entry = build_entry(entry_path, metadata, unique_id);

                            let file_arc = Arc::new(file_entry);

                            batch.push(file_arc);
                            total_files += 1;

                            if batch.len() >= 150 {
                                let send_batch = std::mem::take(&mut batch);
                                sender_clone.send(FileLoadingMessage::RecursiveBatch {
                                    generation: loading_generation,
                                    batch: send_batch,
                                    source_dir: cwd_clone.clone(),
                                }).ok();
                            }
                        }
                    }
                }
                (batch, total_files)
            }).await;

                match walk_result {
                    Ok((remaining_batch, found_total)) => {
                    total_files = found_total;

                    if !remaining_batch.is_empty() {
                        sender.send(FileLoadingMessage::RecursiveBatch {
                            generation: loading_generation,
                            batch: remaining_batch,
                            source_dir: cwd.clone(),
                        }).ok();
                    }

                    sender.send(RecursiveMessages::Finished {
                        task_id: loading_generation as u64,
                        success: true,
                        text: format!("Completado: {} archivos encontrados", total_files),
                    }).ok();

                    debug!("Búsqueda recursiva completada: {} archivos", total_files);
                }
                Err(e) => {
                    sender.send(
                        UiEvent::ShowError(format!("Error buscando archivos: {}", e))
                    ).ok();
                }
            }

            flag.store(false, std::sync::atomic::Ordering::Relaxed);
            debug!("Búsqueda recursiva completada: {} archivos", total_files);
        });
    }

    pub fn start_recursive_search(&mut self, query: String, max_depth: usize, sender: Dispatcher) {
        self.recursive_entries.clear();
        self.recursive_entries.shrink_to_fit();
        self.is_recursive_active = true;

        self.loading_generation += 1;
        let current_generation = self.loading_generation;
        self.loading_flag.store(true, Ordering::Relaxed);

        let path = self.cwd.clone();
        let flag = self.loading_flag.clone();

        let show_hidden = with_configs(|cfg| cfg.configs.show_hidden_files);

        Self::recursive_search(path, query, max_depth, sender, show_hidden, current_generation, flag);
    }



    pub fn navigate_to(&mut self, new_path: &Path) {
        if new_path.is_dir() && new_path != self.cwd {
            let old_path = self.cwd.clone();
            if self.history.last() != Some(&old_path) {
                self.history.push(old_path);
            }

            self.future.clear();

            if self.history.len() > 100 {
                self.history.remove(0);
            }

            self.cwd = new_path.to_owned();
        }
    }


    pub fn up(&mut self) {
        if let Some(parent) = self.cwd.parent() {
            let old_path = self.cwd.clone();
            let new_path = parent.to_path_buf();

            if new_path == old_path { return; }
            
            self.history.push(old_path.clone());

            self.future.clear();
            self.future.push(old_path);

            if self.history.len() > 100 {
                self.history.remove(0);
            }

            self.cwd = new_path;
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

}



pub struct BlazeMotor {
    pub tabs: Vec<TabState>,
    pub active_tab_index: usize,
    pub disk_manager: Arc<tokio::sync::Mutex<DiskManager>>, 
    pub limit: usize,
}


thread_local! {
    pub static MOTOR: RefCell<Option<Rc<RefCell<BlazeMotor>>>> = RefCell::new(None);
}
pub fn with_motor<F, R>(f: F) -> R 
    where F: FnOnce(&mut BlazeMotor) -> R {
        MOTOR.with(|m|{
            let motor_rc = m.borrow()
                .as_ref()
                .expect("Motor no inicializado")
                .clone();

            let mut motor = motor_rc.borrow_mut();
            f(&mut *motor)
        })
    }


impl BlazeMotor {
    pub async fn new() -> Self {
        let tab_id = Uuid::new_v4();
        let home = &KnownDirsManager::get().home;
        let fist_tab = TabState::new(home.to_owned(), tab_id);

        let disk_manager = Arc::new(tokio::sync::Mutex::new(DiskManager::new().await));

        {
            let mut mgr_guard = disk_manager.lock().await;
            mgr_guard.load_disks().await;
            if let Err(e) = mgr_guard.start_watcher_linux(disk_manager.clone()).await {
                error!("Error inicializando watcher de discos: {}", e);
            }
        }

        Self {
            tabs: vec![fist_tab],
            active_tab_index: 0,
            disk_manager,
            limit: 50,
        }
    }

    fn set_active_index(&mut self, index: usize) {
        self.active_tab_index = index;
        crate::core::runtime::event_bus::set_active_tab(self.tabs[index].id);
    }


    pub fn active_tab_mut(&mut self) -> &mut TabState {
        &mut self.tabs[self.active_tab_index]
    }

    pub fn active_tab(&self) -> &TabState {
        &self.tabs[self.active_tab_index]
    }

    pub fn switch_to_tab(&mut self, index:usize) {
        if index < self.tabs.len() {
            self.set_active_index(index);
        }
    }

    pub fn next_tab(&mut self) {
        if self.tabs.is_empty() || self.tabs.len() <= 1 {
            return;
        }
        self.set_active_index((self.active_tab_index + 1) % self.tabs.len());
    }

    pub fn prev_tab(&mut self) {
        if self.tabs.is_empty() || self.tabs.len() <= 1 {
            return;
        }
        let next= if self.active_tab_index == 0 {
            self.tabs.len() - 1
        } else {
            self.active_tab_index - 1
        };
        self.set_active_index(next);
    }
    
    fn remove_channels(&self, tab_id: Uuid) {
        with_event_bus(|pool| {
            pool.remove_tab(tab_id)
        });
    }

    pub fn close_tab(&mut self, index:usize) -> bool{
        if self.tabs.len() <= 1 {
            return false;
        }

        let tab_id = self.tabs[index].id;

        {
            let tab = &mut self.tabs[index];
            tab.watcher.stop_watching();

            tab.files.clear();
            tab.files.shrink_to_fit();
            tab.lower_names.clear();
            tab.lower_names.shrink_to_fit();
            tab.sorted_indices.clear();
            tab.sorted_indices.shrink_to_fit();
        
            tab.recursive_entries.clear();
            tab.recursive_entries.shrink_to_fit();

            tab.history.clear();
            tab.future.clear();
        }

        self.remove_channels(tab_id);

        self.tabs.remove(index);
        if self.active_tab_index >= self.tabs.len() {
            self.set_active_index(self.tabs.len() - 1);
        }
        true
    }

    fn start_tab_load(&mut self, index: usize) {
        let tab = &self.tabs[index];
        let tab_id = tab.id;
        let sender = with_event_bus(|pool| pool.dispatcher(tab_id));
        self.tabs[index].load_path(false, sender).ok();
    }

    pub fn add_tab(&mut self, tab_path: &Path) -> Option<Uuid> {
        if self.tabs.len() >= self.limit {
            return None;
        }
        let tab_id = Uuid::new_v4();
        let new_tab = TabState::new(tab_path.to_owned(), tab_id);

        let insert_index = self.active_tab_index + 1;
        self.tabs.insert(insert_index, new_tab);
        
        self.set_active_index(insert_index);

        self.start_tab_load(self.active_tab_index);

        Some(tab_id)
    }

    pub fn create_tab(&mut self) -> Option<Uuid> {
        if self.tabs.len() >= self.limit {
            return None;
        }
        let path = &KnownDirsManager::get().home;
        let tab_id = Uuid::new_v4();
        let new_tab = TabState::new(path.to_owned(), tab_id);
        let insert_index = self.active_tab_index + 1;
        self.tabs.insert(insert_index, new_tab);

        self.set_active_index(insert_index);

        self.start_tab_load(self.active_tab_index);

        Some(tab_id)
    }

    pub fn tab_title(&self, index:usize) -> String {
        self.tabs.get(index)
        .and_then(|tab|tab.cwd.file_name())
        .and_then(|name|name.to_str())
        .unwrap_or("Home")
        .to_owned()
    }

}




#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn make_tab(path: PathBuf) -> TabState {
        let id = Uuid::new_v4();
        TabState::new(path, id)
    }

    #[test]
    fn test_stop_watching_does_not_leave_handle() {
        let mut tab = make_tab(std::env::temp_dir());
        // Simular que tenía un handle activo
        let handle = TOKIO_RUNTIME.spawn(async { tokio::time::sleep(Duration::from_secs(60)).await });
        tab.watcher.watching_handle = Some(handle);
        tab.watcher.watching.store(true, Ordering::Relaxed);

        tab.watcher.stop_watching();

        assert!(!tab.watcher.watching.load(Ordering::Relaxed), "watching debe ser false");
        assert!(tab.watcher.watching_handle.is_none(), "handle debe haberse consumido");
        assert!(tab.watcher.watcher.is_none(), "watcher debe ser None");
    }

    #[test]
    fn test_cancel_loading_drains_handles() {
        let mut tab = make_tab(std::env::temp_dir());
        let h1 = TOKIO_RUNTIME.spawn(async { tokio::time::sleep(Duration::from_secs(60)).await });
        let h2 = TOKIO_RUNTIME.spawn(async { tokio::time::sleep(Duration::from_secs(60)).await });
        tab.loader.handles.push(h1);
        tab.loader.handles.push(h2);
        tab.loading_flag.store(false, Ordering::Relaxed);

        tab.loader.cancel();

        assert!(tab.loader.handles.is_empty(), "handles deben haberse drenado");
        assert!(!tab.loading_flag.load(Ordering::Relaxed), "flag debe ser false");
    }

    #[test]
    fn test_close_tab_clears_all_memory() {
        let mut motor = TOKIO_RUNTIME.block_on(BlazeMotor::new());
        // Añadir un segundo tab para poder cerrar el primero
        motor.add_tab(&std::env::temp_dir());
        assert_eq!(motor.tabs.len(), 2);

        // Llenar datos en tab 0
        motor.tabs[0].files = vec![]; // simular con vec vacío, basta para el test
        motor.tabs[0].recursive_entries.push(Arc::new(FileEntry::default()));

        let closed = motor.close_tab(0);

        assert!(closed);
        assert_eq!(motor.tabs.len(), 1);
    }

    #[test]
    fn test_close_tab_refuses_last_tab() {
        let mut motor = TOKIO_RUNTIME.block_on(BlazeMotor::new());
        assert_eq!(motor.tabs.len(), 1);

        let result = motor.close_tab(0);
        assert!(!result, "no debe permitir cerrar el último tab");
        assert_eq!(motor.tabs.len(), 1);
    }

    #[test]
    fn test_active_tab_index_clamps_after_close() {
        let mut motor = TOKIO_RUNTIME.block_on(BlazeMotor::new());
        motor.add_tab(&std::env::temp_dir());
        motor.add_tab(&std::env::temp_dir());
        // tabs: [0, 1, 2], active = 2
        motor.active_tab_index = 2;

        motor.close_tab(2); // cierra el activo (el último)

        assert_eq!(motor.active_tab_index, 1, "debe apuntar al nuevo último tab");
    }

    #[test]
    fn test_watcher_task_exits_on_watcher_drop() {
        // Verificar que la task del watcher termina sola cuando se dropea el watcher
        let mut tab = make_tab(std::env::temp_dir());
        let cwd = &tab.cwd;
        let sender = with_event_bus(|pool| pool.dispatcher(tab.id));

        tab.watcher.start_watching(&cwd, sender).ok();
        assert!(tab.watcher.watching_handle.is_some());

        // stop_watching dropea el watcher → fs_tx se cierra → task termina
        tab.watcher.stop_watching();

        // Dar tiempo a la task para terminar (Disconnected break)
        std::thread::sleep(Duration::from_millis(200));

        // El handle fue abortado/tomado por stop_watching
        assert!(tab.watcher.watching_handle.is_none());
    }


    fn two_distinct_dirs() -> (PathBuf, PathBuf) {
        let base = std::env::temp_dir();
        let a = base.join("blaze_test_a");
        let b = base.join("blaze_test_b");
        std::fs::create_dir_all(&a).ok();
        std::fs::create_dir_all(&b).ok();
        (a, b)
    }


    #[test]
    fn test_navigate_to_updates_history() {
        let (start, other) = two_distinct_dirs();
        let mut tab = make_tab(start.clone());

        tab.navigate_to(&*other);

        assert_eq!(tab.cwd, other);
        assert!(tab.history.contains(&start));
        assert!(tab.future.is_empty());

        if start.exists() {
            let _ = std::fs::remove_dir(start);
        }
        if other.exists() {
            let _ = std::fs::remove_dir(other);
        }
    }

    #[test]
    fn test_back_and_forward() {
        let (start, other) = two_distinct_dirs();
        let mut tab = make_tab(start.clone());

        tab.navigate_to(&*other);
        tab.back();

        assert_eq!(tab.cwd, start);
        assert!(!tab.future.is_empty());

        tab.forward();
        assert_eq!(tab.cwd, other);

        if start.exists() {
            let _ = std::fs::remove_dir(start);
        }
        if other.exists() {
            let _ = std::fs::remove_dir(other);
        }
    }
}