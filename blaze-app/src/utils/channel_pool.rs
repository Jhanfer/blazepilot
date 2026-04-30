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



use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;
use file_id::FileId;
use once_cell::sync::Lazy;
use uuid::Uuid;
use std::sync::Mutex;
use crossbeam_channel::{Receiver, SendError, Sender, unbounded};
use crate::core::files::blaze_motor::motor_structs::{FileLoadingMessage, RecursiveMessages};
use crate::core::files::blaze_motor::{motor::with_motor, motor_structs::FileEntry};
use crate::core::system::extended_info::extended_info_manager::ExtendedInfoMessages;
use crate::core::system::fileopener_module::AppAssociation;
use crate::core::system::fileopener_module::platform::linux::linux::AppsIconData;
use crate::core::system::sizer_manager::sizer_manager::SizerMessages;
use crate::core::system::updater::updater::UpdateMessages;
use crate::ui::icons_cache::thumbnails::thumbnails_manager::ThumbnailMessages;
use crate::ui::image_preview::image_preview::ImagePreviewState;
use crate::ui::task_manager::task_manager::TaskMessage;
use tracing::{info, warn};
use std::sync::RwLock;


static SENDER_CACHE: Lazy<RwLock<HashMap<Uuid, NotifyingSender>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub fn cache_sender(tab_id: Uuid, sender: NotifyingSender) {
    if let Ok(mut cache) = SENDER_CACHE.write() {
        cache.insert(tab_id, sender);
    }
}

pub fn remove_cached_sender(tab_id: Uuid) {
    if let Ok(mut cache) = SENDER_CACHE.write() {
        cache.remove(&tab_id);
    }
}

pub fn with_active_sender_for<R>(tab_id: Uuid, f: impl FnOnce(&NotifyingSender) -> R) -> Option<R> {
    SENDER_CACHE.read().ok()?.get(&tab_id).map(f)
}

pub fn with_active_sender<R>(f: impl FnOnce(&NotifyingSender) -> R) -> Option<R> {
    let tab_id = with_motor(|m| m.active_tab().id);
    SENDER_CACHE.read().ok()?.get(&tab_id).map(f)
}


#[derive(Debug)]
pub enum FileOperation {
    Move { files: Vec<PathBuf>, dest: PathBuf, tab_id: Uuid},
    Delete { files: Vec<PathBuf> },
    Copy { files: Vec<PathBuf>, dest: PathBuf },
    Update,
    UpdateDirSize {
        full_path: PathBuf, 
        size: u64, 
        tab_id: Uuid,
    },
    RestoreDeletedFiles {
        file_names: Vec<String>,
    },
    ExtendedInfoReady {
        full_path: PathBuf,
        tab_id: Uuid,
    },

    ExtractHere {
        entry: Arc<FileEntry>, 
        dest_dir: PathBuf,
    },
}

#[derive(Debug)]
pub enum SureTo {
    SureToMove {
        files: Vec<PathBuf>,
        dest: PathBuf,
        tab_id: Uuid,
    },
    SureToDelete {
        files: Vec<PathBuf>,
        tab_id: Uuid,
    },
    SureToCopy,
}

#[derive(Debug)]
pub enum FileConflict {
    AlreadyExist {
        name: String,
        path: PathBuf
    }
}


pub enum UiEvent {
    OpenWithSelector {
        path: PathBuf,
        mime: String,
        apps: Vec<AppAssociation>,
        icon_data: Vec<AppsIconData>,
        show_all_apps: bool,
    },

    ThumbnailReady {
        full_path: PathBuf,
        tab_id: Uuid,
    },


    ShowImagePvw {
        pvw: Option<ImagePreviewState>,

    },

    SureTo(SureTo),

    UpdateMessages(UpdateMessages),

    FileConflict(FileConflict),

    ShowError(String),

    ShowFolderColorSelector {
        folder_id: FileId,
    },

    OpenConfigs,

    RefreshList,
}


#[derive(Clone)]
pub struct NotifyingSender {
    pub tab_id: Uuid,
    file_sender: Sender<FileLoadingMessage>,
    task_sender: Sender<TaskMessage>,
    recursive_search_sender: Sender<RecursiveMessages>,
    ui_event_sender: Sender<UiEvent>,
    file_operation_sender: Sender<FileOperation>,
    sizer_sender: Sender<SizerMessages>,
    extended_info_sender: Sender<ExtendedInfoMessages>,
    thumbnails_sender: Sender<ThumbnailMessages>,

    notifier: Arc<dyn Fn() + Send + Sync>,
}

impl NotifyingSender {
    pub fn send_files_batch(&self, msg: FileLoadingMessage) -> Result<(), SendError<FileLoadingMessage>> {
        let result = self.file_sender.send(msg);
        if result.is_ok() {(self.notifier)();}result
    }

    pub fn send_tasks(&self, msg: TaskMessage) -> Result<(), SendError<TaskMessage>> {
        let result = self.task_sender.send(msg);
        if result.is_ok() {(self.notifier)();}
        result
    }

    pub fn send_recursive(&self, msg: RecursiveMessages) -> Result<(), SendError<RecursiveMessages>> {
        let result = self.recursive_search_sender.send(msg);
        if result.is_ok() {(self.notifier)();}
        result
    }

    pub fn send_ui_event(&self, msg: UiEvent) -> Result<(), SendError<UiEvent>> {
        let result = self.ui_event_sender.send(msg);
        if result.is_ok() {(self.notifier)();}
        result
    }


    pub fn send_fileop(&self, msg: FileOperation) -> Result<(), SendError<FileOperation>> {
        let result = self.file_operation_sender.send(msg);
        if result.is_ok() {(self.notifier)();}
        result
    }

    pub fn send_sizer(&self, msg: SizerMessages) -> Result<(), SendError<SizerMessages>> {
        let result = self.sizer_sender.send(msg);
        if result.is_ok() {(self.notifier)();}
        result
    }

    pub fn send_extended_info(&self, msg: ExtendedInfoMessages) -> Result<(), SendError<ExtendedInfoMessages>> {
        let result = self.extended_info_sender.send(msg);
        if result.is_ok() {(self.notifier)();}
        result
    }

    pub fn send_thumbnails(&self, msg: ThumbnailMessages) -> Result<(), SendError<ThumbnailMessages>> {
        let result = self.thumbnails_sender.send(msg);
        if result.is_ok() {(self.notifier)();}
        result
    }


}

pub struct ChannelPool {
    file_channels: HashMap<Uuid, Arc<(Sender<FileLoadingMessage>, Receiver<FileLoadingMessage>)>>,
    task_channels: HashMap<Uuid, Arc<(Sender<TaskMessage>, Receiver<TaskMessage>)>>,
    recursive_channels: HashMap<Uuid, Arc<(Sender<RecursiveMessages>, Receiver<RecursiveMessages>)>>,
    ui_event_channels: HashMap<Uuid, Arc<(Sender<UiEvent>, Receiver<UiEvent>)>>,
    fileops_channels: HashMap<Uuid, Arc<(Sender<FileOperation>, Receiver<FileOperation>)>>,
    sizer_channels: HashMap<Uuid, Arc<(Sender<SizerMessages>, Receiver<SizerMessages>)>>,
    extended_info_channels: HashMap<Uuid, Arc<(Sender<ExtendedInfoMessages>, Receiver<ExtendedInfoMessages>)>>,
    thumbnails_channels: HashMap<Uuid, Arc<(Sender<ThumbnailMessages>, Receiver<ThumbnailMessages>)>>,

    ui_notifier: HashMap<Uuid, Arc<dyn Fn() + Send + Sync>>,
}


pub static CHANNEL_POOL: Lazy<Mutex<ChannelPool>> = Lazy::new(|| {
    Mutex::new(ChannelPool::new())
});

pub fn with_channel_pool<R>(f: impl FnOnce(&mut ChannelPool) -> R) -> R {
    match CHANNEL_POOL.lock() {
        Ok(mut guard) => f(&mut *guard),
        Err(poisoned) => {
            warn!("ChannelPool estaba poisoned, recuperando...");
            let mut guard = poisoned.into_inner();
            f(&mut *guard)
        }
    }
}


impl ChannelPool {
    pub fn new() -> Self {
        Self {
            file_channels: HashMap::new(),
            task_channels: HashMap::new(),
            recursive_channels: HashMap::new(),
            ui_event_channels: HashMap::new(),
            fileops_channels: HashMap::new(),
            sizer_channels: HashMap::new(),
            extended_info_channels: HashMap::new(),
            thumbnails_channels: HashMap::new(),
            ui_notifier: HashMap::new(),
        }
    }

    pub fn register_notifier<F>(&mut self, tab_id: Uuid, notifier: F) 
        where 
            F: Fn() + Send + Sync + 'static {
                self.ui_notifier.insert(tab_id, Arc::new(notifier));
    }

    pub fn notify(&self, tab_id: Uuid) {
        if let Some(notifier) = self.ui_notifier.get(&tab_id) {
            notifier();
        }
    }

    pub fn get_notifying_sender(&mut self, tab_id: Uuid) -> Option<NotifyingSender> {
        let file_sender = self.get_file_sender(tab_id);
        let task_sender = self.get_task_sender(tab_id);
        let recursive_search_sender = self.get_recursive_sender(tab_id);
        let ui_event_sender = self.get_ui_event_channels_sender(tab_id);
        let file_operation_sender = self.get_fileop_sender(tab_id);
        let sizer_sender = self.get_sizer_sender(tab_id);
        let extended_info_sender = self.get_extended_info_sender(tab_id);
        let thumbnails_sender = self.get_thumbnails_sender(tab_id);

        let Some(notifier) = self.ui_notifier.get(&tab_id) else {
            warn!(tab_id = %tab_id, "NO HAY NOTIFIER para tab_id");
            return None;
        };

        info!(tab_id = %tab_id, "Notifier encontrado para tab_id");
        Some(
            NotifyingSender { 
                tab_id, 
                file_sender, 
                task_sender, 
                recursive_search_sender,
                ui_event_sender,
                file_operation_sender,
                sizer_sender,
                extended_info_sender,
                thumbnails_sender,
                notifier: notifier.clone() 
            }
        )
    }

    pub fn get_file_sender(&mut self, tab_id: Uuid) -> Sender<FileLoadingMessage> {
        if !self.file_channels.contains_key(&tab_id) {
            let (tx, rx) = unbounded();
            self.file_channels.insert(tab_id, (tx, rx).into());
        }
        self.file_channels.get(&tab_id).unwrap().0.clone()
    }

    pub fn get_task_sender(&mut self, tab_id: Uuid) -> Sender<TaskMessage> {
        if !self.task_channels.contains_key(&tab_id) {
            let (tx, rx) = unbounded();
            self.task_channels.insert(tab_id, (tx, rx).into());
        }
        self.task_channels.get(&tab_id).unwrap().0.clone()
    }

    pub fn get_recursive_sender(&mut self, tab_id: Uuid) -> Sender<RecursiveMessages> {
        if !self.recursive_channels.contains_key(&tab_id) {
            let (tx, rx) = unbounded();
            self.recursive_channels.insert(tab_id, (tx, rx).into());
        }
        self.recursive_channels.get(&tab_id).unwrap().0.clone()
    }

    pub fn get_ui_event_channels_sender(&mut self, tab_id: Uuid) -> Sender<UiEvent> {
        if !self.ui_event_channels.contains_key(&tab_id) {
            let (tx, rx) = unbounded();
            self.ui_event_channels.insert(tab_id, (tx, rx).into());
        }
        self.ui_event_channels.get(&tab_id).unwrap().0.clone()
    }

    pub fn get_fileop_sender(&mut self, tab_id: Uuid) -> Sender<FileOperation> {
        if !self.fileops_channels.contains_key(&tab_id) {
            let (tx, rx) = unbounded();
            self.fileops_channels.insert(tab_id, (tx, rx).into());
        }
        self.fileops_channels.get(&tab_id).unwrap().0.clone()
    }

    pub fn get_sizer_sender(&mut self, tab_id: Uuid) -> Sender<SizerMessages> {
        if !self.sizer_channels.contains_key(&tab_id) {
            let (tx, rx) = unbounded();
            self.sizer_channels.insert(tab_id, (tx, rx).into());
        }
        self.sizer_channels.get(&tab_id).unwrap().0.clone()
    }

    pub fn get_extended_info_sender(&mut self, tab_id: Uuid) -> Sender<ExtendedInfoMessages> {
        if !self.extended_info_channels.contains_key(&tab_id) {
            let (tx, rx) = unbounded();
            self.extended_info_channels.insert(tab_id, (tx, rx).into());
        }
        self.extended_info_channels.get(&tab_id).unwrap().0.clone()
    }

    pub fn get_thumbnails_sender(&mut self, tab_id: Uuid) -> Sender<ThumbnailMessages> {
        if !self.thumbnails_channels.contains_key(&tab_id) {
            let (tx, rx) = unbounded();
            self.thumbnails_channels.insert(tab_id, (tx, rx).into());
        }
        self.thumbnails_channels.get(&tab_id).unwrap().0.clone()
    }

    pub fn process_file_messages<F>(&self, tab_id: Uuid, mut processor: F) -> bool
        where
            F: FnMut(FileLoadingMessage) -> bool
        {
            let mut processed_any = false;
            
            if let Some(arc_pair) = self.file_channels.get(&tab_id) {
                while let Ok(msg) = arc_pair.1.try_recv() {
                    if processor(msg) {
                        processed_any = true;
                    }
                }
            }
            processed_any
    }

    pub fn process_task_messages<F>(&self, tab_id: Uuid, mut processor: F) -> bool
        where 
            F: FnMut(TaskMessage) -> bool
        {
            let mut processed_any = false;
            if let Some(arc_pair) = self.task_channels.get(&tab_id) {
                while let Ok(msg) = arc_pair.1.try_recv() {
                    if processor(msg) {
                        processed_any = true;
                    }
                }
            }
            processed_any
        }


    pub fn process_recursive_messages<F>(&self, tab_id: Uuid, mut processor: F) -> bool
        where 
            F: FnMut(RecursiveMessages) -> bool
        {
            let mut processed_any = false;
            if let Some(arc_pair) = self.recursive_channels.get(&tab_id) {
                while let Ok(msg) = arc_pair.1.try_recv() {
                    if processor(msg) {
                        processed_any = true;
                    }
                }
            }
            processed_any
        }

    pub fn process_ui_events<F>(&self, tab_id: Uuid, mut processor: F) -> bool 
        where 
            F: FnMut(UiEvent) -> bool 
        {
            let mut process_any = false;
            if let Some(arc_pair) = self.ui_event_channels.get(&tab_id) {
                while let Ok(msg) = arc_pair.1.try_recv() {
                    if processor(msg) {
                        process_any = true;
                    }
                }
            }
            process_any
        }


    pub fn process_fileops_events<F>(&self, tab_id: Uuid, mut processor: F) -> bool 
        where 
            F: FnMut(FileOperation) -> bool 
        {
            let mut process_any = false;
            if let Some(arc_pair) = self.fileops_channels.get(&tab_id) {
                while let Ok(msg) = arc_pair.1.try_recv() {
                    if processor(msg) {
                        process_any = true;
                    }
                }
            }
            process_any
        }

    pub fn process_sizer_events<F>(&self, tab_id: Uuid, mut processor: F) -> bool 
        where 
            F: FnMut(SizerMessages) -> bool 
        {
            let mut process_any = false;
            if let Some(arc_pair) = self.sizer_channels.get(&tab_id) {
                while let Ok(msg) = arc_pair.1.try_recv() {
                    if processor(msg) {
                        process_any = true;
                    }
                }
            }
            process_any
        }

    pub fn process_extended_info_events<F>(&self, tab_id: Uuid, mut processor: F) -> bool 
        where 
            F: FnMut(ExtendedInfoMessages) -> bool 
        {
            let mut process_any = false;
            if let Some(arc_pair) = self.extended_info_channels.get(&tab_id) {
                while let Ok(msg) = arc_pair.1.try_recv() {
                    if processor(msg) {
                        process_any = true;
                    }
                }
            }
            process_any
        }

    pub fn process_thumbnail_events<F>(&self, tab_id: Uuid, mut processor: F) -> bool 
        where 
            F: FnMut(ThumbnailMessages) -> bool 
        {
            let mut process_any = false;
            if let Some(arc_pair) = self.thumbnails_channels.get(&tab_id) {
                while let Ok(msg) = arc_pair.1.try_recv() {
                    if processor(msg) {
                        process_any = true;
                    }
                }
            }
            process_any
        }

    pub fn drain_file_loading_messages(&mut self, tab_id: Uuid) {
        if let Some(arc_pair) = self.file_channels.get(&tab_id) {
            let mut pending = Vec::new();
            while let Ok(msg) = arc_pair.1.try_recv() {
                match msg {
                    FileLoadingMessage::Batch(..) | 
                    FileLoadingMessage::ProgressUpdate{..} |
                    FileLoadingMessage::RecursiveBatch{..} |
                    FileLoadingMessage::Finished(..) => {
                        pending.push(msg);
                    },
                    FileLoadingMessage::FileAdded {..} |
                    FileLoadingMessage::FileRemoved {..} |
                    FileLoadingMessage::FileModified {..} |
                    FileLoadingMessage::GitStatusChanged {..} |
                    FileLoadingMessage::FullRefresh => {
                        pending.push(msg);
                    },
                }
            }

            for msg in pending {
                let _ = arc_pair.0.send(msg);
            }
        }
    }



    pub fn get_or_create_file_channel(&mut self, tab_id: Uuid) -> (Sender<FileLoadingMessage>, Receiver<FileLoadingMessage>) {
        let arc_pair = self.file_channels
            .entry(tab_id)
            .or_insert_with(|| {
                let (tx, rx) = unbounded();
                Arc::new((tx, rx))
            });
        (arc_pair.0.clone(), arc_pair.1.clone())
    }

    pub fn get_or_create_task_channel(&mut self, tab_id: Uuid) -> (Sender<TaskMessage>, Receiver<TaskMessage>) {
        let arc_pair = self.task_channels
            .entry(tab_id)
            .or_insert_with(||{
                let (tx, rx) = unbounded();
                Arc::new((tx, rx))
            });
        (arc_pair.0.clone(), arc_pair.1.clone())
    }


    pub fn remove_tab(&mut self, tab_id: Uuid) -> bool {
        self.ui_notifier.remove(&tab_id);

        let mut removed = false;

        if let Some(arc_pair) = self.file_channels.remove(&tab_id) {
            while let Ok(_) = arc_pair.1.try_recv() {}
            removed = true;
        };

        if let Some(arc_pair) = self.task_channels.remove(&tab_id) {
            while let Ok(_) = arc_pair.1.try_recv() {}
            removed = true;
        };

        if let Some(arc_pair) = self.recursive_channels.remove(&tab_id) {
            while let Ok(_) = arc_pair.1.try_recv() {}
            removed = true;
        };

        if let Some(arc_pair) = self.ui_event_channels.remove(&tab_id) {
            while let Ok(_) = arc_pair.1.try_recv() {}
            removed = true;
        };

        if let Some(arc_pair) = self.fileops_channels.remove(&tab_id) {
            while let Ok(_) = arc_pair.1.try_recv() {}
            removed = true;
        };

        if removed {
            info!(tab_id = %tab_id, "Tab y canales removidos");
        } else {
            warn!(tab_id = %tab_id, "Se intentó remover tab inexistente");
        }
        removed
    }


    pub fn has_channel(&self, tab_id: Uuid) -> bool {
        self.file_channels.contains_key(&tab_id)
    }

    pub fn active_file_channels(&self) -> usize {
        self.file_channels.len()
    }

}