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




use cfg_if::cfg_if;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use std::{fs, thread, vec};
use dirs::{home_dir};
use file_id::{FileId, get_file_id};
use jwalk::{Parallelism, WalkDir};
use once_cell::sync::Lazy;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::RwLock; 
use lru::LruCache;
use std::num::NonZeroUsize;
use crate::core::files::file_extension::FileExtension;

use crate::core::configs::config_state::{with_configs, OrderingMode};
use crate::core::system::disk_reader::disk_manager::DiskManager;
use crate::utils::channel_pool::{NotifyingSender, UiEvent, cache_sender, remove_cached_sender, with_channel_pool};
use uuid::Uuid;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use notify::event::{CreateKind, ModifyKind, RemoveKind};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::channel as std_channel;
use crate::core::system::clipboard::TOKIO_RUNTIME;



static NEXT_TASK: AtomicU64 = AtomicU64::new(1);
pub fn new_task_id() -> u64 {
    NEXT_TASK.fetch_add(1, Ordering::Relaxed)
}


#[derive(Debug, Clone)]
pub enum TaskType {
    FileLoading,
    CopyPaste,
    CutPaste,
    MoveTrash,
    Delete,
    RestoreTrash
}

#[derive(Debug, Clone)]
pub enum FileLoadingMessage {
    Batch(u64, Vec<Arc<FileEntry>>),
    Finished(u64),
    ProgressUpdate {
        total: usize,
        done: usize,
        text: String,
    },

    FileAdded { name: String },
    FileRemoved { name: String },
    FileModified { name: String },
    FullRefresh,

    RecursiveBatch {
        generation: u64,
        batch: Vec<Arc<FileEntry>>,
        source_dir: PathBuf,
    }
}


#[derive(Debug, Clone)]
pub enum RecursiveMessages {
    Started {
        task_id: u64,
        text: String,
    },
    Progress {
        task_id: u64,       
        files_found: usize,  
        current_dir: PathBuf, 
        text: String, 
    },
    Finished {
        task_id: u64,
        success: bool,
        text: String,
    }
}


#[derive(Debug, Default, Clone)]
pub struct FileEntry {
    pub name: Box<str>,
    pub extension: FileExtension,
    pub kind: FileKind,
    pub size: u64,
    pub modified: u64,
    pub created: u64,
    pub is_hidden: bool,
    pub is_dir: bool,
    pub full_path: PathBuf,
    pub unique_id: Option<FileId>,

    pub accessed: u64,
    pub permissions: u32,

    #[cfg(unix)]
    pub inode: u64,
    #[cfg(unix)]
    pub nlink: u64,
    #[cfg(unix)]
    pub device: u64,

    #[cfg(windows)]
    pub attributes: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum FileKind {
    #[default]
    File,
    Dir,
    Symlink
}

#[derive(Default)]
pub struct TabState {
    pub id: Uuid,
    pub cwd: PathBuf,
    pub history: Vec<PathBuf>,
    pub future: Vec<PathBuf>,
    pub loading_flag: Arc<AtomicBool>,

    pub lower_names: Vec<(usize, String)>,
    pub loading_generation: u64,
    pub active_generation: u64,
    loading_handles: Vec<tokio::task::JoinHandle<()>>, 

    pub files: Vec<Arc<FileEntry>>,
    pub sorted_indices: Vec<usize>,
    pub current_ordering: OrderingMode,

    pub watcher: Option<Box<dyn Watcher + Send>>,
    pub watching: Arc<AtomicBool>,

    pub recursive_entries: Vec<Arc<FileEntry>>,
    pub is_recursive_active: bool,

}

pub static FILE_CACHE: Lazy<RwLock<LruCache<PathBuf, Vec<Arc<FileEntry>>>>> = Lazy::new(|| {
    RwLock::new(LruCache::new(NonZeroUsize::new(50).unwrap()))
});

impl TabState {
    pub fn new(start_path: PathBuf, tab_id: Uuid) -> Self {

        //registrar el notificador por ventana
        with_channel_pool(|pool| {
            pool.register_notifier(tab_id, || {});
        });

        let sender = with_channel_pool(|pool| pool.get_notifying_sender(tab_id)).unwrap();
        cache_sender(tab_id, sender);

        let ordering = with_configs(|ccgf|ccgf.configs.app_ordering_mode.clone());

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
            loading_handles: Vec::new(),
            sorted_indices: Vec::new(),
            current_ordering: ordering,
            watcher: None,
            watching: Arc::new(AtomicBool::new(false)),

            recursive_entries: Vec::new(),
            is_recursive_active: false,
        }
    }

    pub fn stop_watching(&mut self) {
        self.watching.store(false, Ordering::Relaxed);
        self.watcher = None;
    }


    pub fn start_watching(&mut self, sender: NotifyingSender) {
        self.stop_watching();
    
        let path = self.cwd.clone();
        let watching = Arc::new(AtomicBool::new(true));
        self.watching = watching.clone();
        
        let (fs_tx, fs_rx) = std_channel();
        
        // se crea el watcher
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                fs_tx.send(event).ok();
            }
        }).ok();
        
        if let Some(ref mut w) = watcher {
            w.watch(&path, RecursiveMode::NonRecursive).ok();
        }
        
        self.watcher = watcher.map(|w| Box::new(w) as Box<dyn Watcher + Send>);
        
        
        TOKIO_RUNTIME.spawn(async move {
            while watching.load(Ordering::Relaxed) {
                if let Ok(event) = fs_rx.recv_timeout(Duration::from_millis(100)) {
                    match event.kind {
                        EventKind::Create(CreateKind::Folder) |
                        EventKind::Create(CreateKind::File) |
                        EventKind::Create(CreateKind::Any) => {
                            // Archivo creado
                            if let Some(path) = event.paths.first() {
                                let name = path.file_name()
                                    .unwrap()
                                    .to_string_lossy()
                                    .into_owned();
                                
                                sender.send_files_batch(FileLoadingMessage::FileAdded { name }).ok();
                            }
                        },
                        EventKind::Remove(RemoveKind::File) |
                        EventKind::Remove(RemoveKind::Folder) |
                        EventKind::Remove(RemoveKind::Any) => {
                            // Archivo eliminado
                            if let Some(path) = event.paths.first() {
                                let name = path.file_name()
                                    .unwrap()
                                    .to_string_lossy()
                                    .into_owned();
                                
                                sender.send_files_batch(FileLoadingMessage::FileRemoved { name }).ok();
                            }
                        },
                        EventKind::Modify(ModifyKind::Name(_)) => {
                            // Archivo renombrado
                            sender.send_files_batch(FileLoadingMessage::FullRefresh).ok();
                        },
                        _ => {}
                    }
                }
            }
        });
    }


    pub fn load_path(&mut self, skip_cache: bool,  sender: NotifyingSender) {
        debug!("🚀 Load_path llamado para: {:?}", self.cwd);
        self.loading_generation += 1;
        let current_generation = self.loading_generation;

        debug!("Sender id, tab id? {}, {}",sender.tab_id, self.id);

        sender.send_files_batch(
            FileLoadingMessage::ProgressUpdate {
                total: 0,
                done: 0,
                text: "Iniciando carga...".to_string(),
            }
        ).ok();


        self.files.clear();
        self.sorted_indices.clear();
        self.lower_names.clear();

        self.active_generation = 0;
        
        self.loading_flag.store(true, Ordering::Relaxed);
        let path = self.cwd.clone();

        if !path.exists() || !path.is_dir() {

            sender.send_files_batch(FileLoadingMessage::Finished(current_generation)).ok();

            self.loading_flag.store(false, Ordering::Relaxed);
            return;
        }

        let flag = self.loading_flag.clone();
        //comporbar caché
        if !skip_cache {
            if let Ok(cache) = FILE_CACHE.read() {
                if let Some(cached_files) = cache.peek(&path) {
                    sender.send_files_batch(
                        FileLoadingMessage::ProgressUpdate {
                            total: cached_files.len(),
                            done: 0,
                            text: "Cargando desde caché...".to_string(),
                        }
                    ).ok();

                    debug!("Usando caché para: {:?} ({} archivos)", path, cached_files.len());

                    let batch = cached_files.clone();


                    if sender.send_files_batch(FileLoadingMessage::Batch(current_generation, batch)).is_err() {
                        self.loading_flag.store(false, Ordering::Relaxed);
                        return;
                    }


                    sender.send_files_batch(
                        FileLoadingMessage::ProgressUpdate {
                            total: cached_files.len(),
                            done: cached_files.len(),
                            text: "Caché completado".to_string(),
                        }
                    ).ok();


                    sender.send_files_batch(FileLoadingMessage::Finished(current_generation)).ok();

                    
                    self.loading_flag.store(false, Ordering::Relaxed);
                    return;
                }
            }
        }

        let sender_clone = sender.clone();
        let handle = TOKIO_RUNTIME.spawn(async move {
            let mut entries_buffer = Vec::with_capacity(500);
            let mut processed: usize = 0;
            
            let total_files = match fs::read_dir(&path) {
                Ok(entries) => entries.count(),
                Err(_) => 0,
            };

            if total_files == 0 {
                sender.send_files_batch(FileLoadingMessage::Finished(current_generation)).ok();
                flag.store(false, Ordering::Relaxed);
                return;
            }

            let estimated_total = total_files;

            if !flag.load(Ordering::Relaxed) {
                return;
            }

            sender.send_files_batch(
                FileLoadingMessage::ProgressUpdate {
                    total: estimated_total,
                    done: processed,
                    text: format!("Leyendo {} archivos...", processed),
                }
            ).ok();


            if let Ok(mut entries) = tokio::fs::read_dir(&path).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if processed % 10 == 0 && !flag.load(Ordering::Relaxed) {
                        return;
                    }

                    processed += 1;

                    if processed % 100 == 0 || processed == estimated_total {
                        sender.send_files_batch(
                            FileLoadingMessage::ProgressUpdate {
                                total: estimated_total, 
                                done: processed, 
                                text: format!("Leyendo {} archivos...", processed),
                            }
                        ).ok();
                    }

                    let name_os = entry.file_name();
                    let name = name_os.to_string_lossy().into_owned().into_boxed_str();
                    
                    let Ok(m) = entry.metadata().await else { continue };

                    let file_type = m.file_type();

                    let kind = if m.file_type().is_symlink() {
                        FileKind::Symlink
                    } else if m.is_dir() {
                        FileKind::Dir
                    } else {
                        FileKind::File
                    };
                
                    let size = if m.is_dir() {
                        0
                    } else {
                        m.len()
                    };

                    let modified = m.modified()
                        .unwrap_or(SystemTime::UNIX_EPOCH)
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    let created = m.created()
                        .unwrap_or(SystemTime::UNIX_EPOCH)
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    let accessed = m.accessed()
                        .unwrap_or(SystemTime::UNIX_EPOCH)
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();


                    #[cfg(unix)]
                    let (inode, nlink, device, permissions) = (
                        m.ino(),
                        m.nlink(),
                        m.dev(),
                        m.mode(),
                    );

                    #[cfg(windows)]
                    let attributes = m.file_attributes();

                    let is_hidden = name.starts_with(".");

                    let is_dir = file_type.is_dir();
                
                    let extension = FileExtension::from_path(&entry.path());

                    let unique_id = get_file_id(&entry.path()).ok();

                    let file_entry = Arc::new(
                    FileEntry {
                            name,
                            is_dir,
                            kind,
                            extension,
                            size,
                            modified, 
                            created,
                            is_hidden,
                            full_path: entry.path(),
                            unique_id,
                            accessed,
                            permissions,
                            inode,
                            nlink,
                            device,

                            #[cfg(windows)]
                            attributes,
                        }
                    );

                    entries_buffer.push(file_entry);

                    if entries_buffer.len() >= 500 {
                        if !flag.load(Ordering::Relaxed) {
                            return;
                        }

                        let batch_to_send = std::mem::take(&mut entries_buffer);

                        sender.send_files_batch(
                            FileLoadingMessage::ProgressUpdate {
                                total: estimated_total, 
                                done: processed, 
                                text: format!("Enviando batch {}...", processed),
                            }
                        ).ok();
                        if sender.send_files_batch(FileLoadingMessage::Batch(current_generation, batch_to_send)).is_err() {
                            return;
                        }

                    }
                }

                if !entries_buffer.is_empty() {
                    sender.send_files_batch(FileLoadingMessage::Batch(current_generation, entries_buffer)).ok();

                    sender.send_files_batch(
                        FileLoadingMessage::ProgressUpdate {
                            total: estimated_total,
                            done: processed,
                            text: "Ordenando...".to_string(),
                        }
                    ).ok();
                }

                sender.send_files_batch(FileLoadingMessage::Finished(current_generation)).ok();
                
            }

            flag.store(false, Ordering::Relaxed);
        });

        self.loading_handles.push(handle);
        self.start_watching(sender_clone);
    }

    pub fn cancel_loading(&mut self) {
        self.loading_flag.store(false, Ordering::Relaxed);

        if self.loading_handles.is_empty() {
            return;
        }

        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_millis(50) {
            if self.loading_handles.iter().all(|h| h.is_finished()) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        for handle in self.loading_handles.drain(..) {
            if !handle.is_finished() {
                handle.abort();
            }
        }
    }


    fn recursive_search(cwd: PathBuf, query: String, max_depth: usize, sender: NotifyingSender, show_hidden: bool, loading_generation: u64, flag: Arc<AtomicBool>) {
        TOKIO_RUNTIME.spawn(async move {
            let query_lower = query.to_lowercase().trim().to_string();
            let mut total_files = 0usize;
            let mut batch: Vec<Arc<FileEntry>> = Vec::with_capacity(150);

            sender.send_recursive(RecursiveMessages::Started {
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
                            let file_entry = Self::create_file_entry(name, metadata, path.to_path_buf());
                            batch.push(file_entry);
                            total_files += 1;

                            if batch.len() >= 150 {
                                let send_batch = std::mem::take(&mut batch);
                                sender_clone.send_files_batch(FileLoadingMessage::RecursiveBatch {
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
                        sender.send_files_batch(FileLoadingMessage::RecursiveBatch {
                            generation: loading_generation,
                            batch: remaining_batch,
                            source_dir: cwd.clone(),
                        }).ok();
                    }

                    sender.send_recursive(RecursiveMessages::Finished {
                        task_id: loading_generation as u64,
                        success: true,
                        text: format!("Completado: {} archivos encontrados", total_files),
                    }).ok();

                    debug!("Búsqueda recursiva completada: {} archivos", total_files);
                }
                Err(e) => {
                    sender.send_ui_event(
                        UiEvent::ShowError(format!("Error buscando archivos: {}", e))
                    ).ok();
                }
            }

            flag.store(false, std::sync::atomic::Ordering::Relaxed);
            debug!("Búsqueda recursiva completada: {} archivos", total_files);
        });
    }

    fn create_file_entry(name: String, m: std::fs::Metadata, path: PathBuf) -> Arc<FileEntry> {
        let is_dir = m.is_dir();
        let kind = if m.file_type().is_symlink() {
            FileKind::Symlink
        } else if is_dir {
            FileKind::Dir
        } else {
            FileKind::File
        };

        let modified = m.modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let created = m.created()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let accessed = m.accessed()
            .unwrap_or(SystemTime::UNIX_EPOCH)
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();


        #[cfg(unix)]
        let (inode, nlink, device, permissions) = (
            m.ino(),
            m.nlink(),
            m.dev(),
            m.mode(),
        );

        #[cfg(windows)]
        let attributes = m.file_attributes();

        let extension = FileExtension::from_path(&path);

        let unique_id = get_file_id(&path).ok();

        Arc::new(FileEntry {
            name: name.clone().into_boxed_str(),
            is_dir,
            extension,
            kind,
            size: m.len(),
            modified,
            created,
            is_hidden: name.starts_with("."),
            full_path: path,
            unique_id,
            accessed,
            permissions,
            inode,
            nlink,
            device,
            #[cfg(windows)]
            attributes,
        })
    }


    pub fn start_recursive_search(&mut self, query: String, max_depth: usize, sender: NotifyingSender) {
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




    pub fn navigate_to(&mut self, new_path: PathBuf) {
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

    pub fn foward(&mut self) {
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
        let fist_tab = TabState::new(home_dir().unwrap(), tab_id);

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
            limit: 100,
        }
    }

    pub fn active_tab_mut(&mut self) -> &mut TabState {
        &mut self.tabs[self.active_tab_index]
    }

    pub fn active_tab(&self) -> &TabState {
        &self.tabs[self.active_tab_index]
    }

    pub fn switch_to_tab(&mut self, index:usize) {
        if index < self.tabs.len() {
            self.active_tab_index = index;
        }
    }

    pub fn next_tab(&mut self) {
        if self.tabs.is_empty() || self.tabs.len() <= 1 {
            return;
        }
        self.active_tab_index = (self.active_tab_index + 1) % self.tabs.len();
    }

    pub fn prev_tab(&mut self) {
        if self.tabs.is_empty() || self.tabs.len() <= 1 {
            return;
        }
        self.active_tab_index = if self.active_tab_index == 0 {
            self.tabs.len() - 1
        } else {
            self.active_tab_index - 1
        };
    }
    
    pub fn close_tab(&mut self, index:usize) -> bool{
        if self.tabs.len() <= 1 {
            return false;
        }

        let tab = &mut self.tabs[index];
        tab.files.clear();
        tab.files.shrink_to_fit();
        tab.lower_names.clear();
        tab.lower_names.shrink_to_fit();
        tab.cwd.clear();

        remove_cached_sender(tab.id);


        self.tabs.remove(index);
        if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
        }
        true
    }

    pub fn add_tab_from_file(&mut self, tab_path: PathBuf) {
        let tab_id = Uuid::new_v4();
        let new_tab = TabState::new(tab_path, tab_id);

        let insert_index = self.active_tab_index + 1;
        self.tabs.insert(insert_index, new_tab);
        self.active_tab_index = insert_index;
    }

    pub fn create_tab(&mut self) {
        let path = home_dir().unwrap();
        let tab_id = Uuid::new_v4();
        let new_tab = TabState::new(path, tab_id);
        let insert_index = self.active_tab_index + 1;
        self.tabs.insert(insert_index, new_tab);
    }

    pub fn tab_title(&self, index:usize) -> String {
        self.tabs.get(index)
        .and_then(|tab|tab.cwd.file_name())
        .and_then(|name|name.to_str())
        .unwrap_or("Home")
        .to_owned()
    }


    pub fn add_tab(&mut self, path: PathBuf) {
        let tab_id = Uuid::new_v4();

        let new_tab = TabState::new(path, tab_id);
        self.tabs.push(new_tab);
        self.active_tab_index = self.tabs.len() - 1;
    }



    fn get_home_trash_dir(&mut self) -> Option<PathBuf> {
        let data_home = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| home_dir().map(|h| h.join(".local/share")))
            .unwrap();

        let trash_root = data_home.join("Trash");
        let files_dir = trash_root.join("files");
        let info_dir = trash_root.join("info");

        if !files_dir.exists() {
            fs::create_dir_all(&files_dir).ok()?;
            fs::create_dir_all(&info_dir).ok()?;
        }

        #[cfg(unix)]
        {
            if let Ok(metadata) = fs::metadata(&trash_root) {
                let permissions = metadata.permissions();
                if !permissions.readonly() {
                    return files_dir.canonicalize().ok();
                }
            }
        }

        files_dir.canonicalize().ok()
    } 



    fn get_external_trash_dir(&mut self, file_path: &PathBuf, mount_point: PathBuf) -> Option<PathBuf> {
        let uid = unsafe {libc::getuid()};
        let trash_dir_name = format!(".Trash-{}", uid);
        let external_trash_root = mount_point.join(trash_dir_name);

        let files_dir = external_trash_root.join("files");
        let info_dir = external_trash_root.join("info");

        if !files_dir.exists() {
            fs::create_dir_all(&files_dir).ok()?;
            fs::create_dir_all(&info_dir).ok()?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                fs::set_permissions(&external_trash_root, fs::Permissions::from_mode(0o700)).ok()?;
            }
        }

        files_dir.canonicalize().ok()
    }


    fn is_same_device(&mut self, path1: &Path, path2: &Path) -> bool {
        #[cfg(unix)]
        {
            use std::os::linux::fs::MetadataExt;
            if let (Ok(meta1), Ok(meta2)) = (fs::metadata(path1), fs::metadata(path2)) {
                return meta1.st_dev() == meta2.st_dev();
            }
        }

        false
    }

    fn is_mount_point(&mut self, path: &PathBuf) -> bool {
        #[cfg(unix)]
        {
            if let (Ok(meta), Some(parent)) = (std::fs::metadata(path), path.parent()) {
                if let Ok(parent_meta) = std::fs::metadata(parent) {
                    use std::os::linux::fs::MetadataExt;
                    return meta.st_dev() != parent_meta.st_dev(); 
                }
            }
        }

        path.to_str() == Some("/")
    }

    fn get_mount_point(&mut self, path: &PathBuf) -> Option<PathBuf> {
        let mut current = path.canonicalize().ok()?;

        loop {
            if self.is_mount_point(&current) {
                return Some(current);
            }
            
            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => return Some(PathBuf::from("/")),
            }
        }
    }

    pub fn get_trash_dir(&mut self, file_path: Option<&Path>) -> Option<PathBuf> {
        #[cfg(target_os = "linux")]
        {
            match file_path {
                None => {
                    return self.get_home_trash_dir()
                },

                Some(path) => {
                    let path_canonical = path.canonicalize().ok()?;
                    
                    if let Some(home) = home_dir() {
                        if self.is_same_device(&path_canonical, &home) {
                            return self.get_home_trash_dir();
                        }
                    }

                    let mount_point = self.get_mount_point(&path_canonical)?;
                    return self.get_external_trash_dir(&path_canonical, mount_point);
                },
            };
        }

        #[cfg(target_os = "windows")]
        {
            let drives = ["C", "D", "E", "F"];
            for drive in drives.iter().chain(&["A", "Z", "H"]) {
                let recycle = PathBuf::from(format!("{}:\\$Recycle.Bin", drive));
                if recycle.exists() { return recycle.canonicalize().ok(); }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(home) = home_dir() {
                let trash = home.join(".Trash");
                if trash.exists() { return trash.canonicalize().ok(); }
            }
        }

    }




}