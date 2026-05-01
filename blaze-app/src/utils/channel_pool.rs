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



//Macro para evitar repeticion de código eliminando canales
macro_rules! remove_arc_pair {
    ($self:expr, $tab_id:expr, $($map:ident), *) => {{
        let mut removed = false;
        $(
            if let Some(arc_pair) = $self.$map.remove($tab_id) {
                while let Ok(_) = arc_pair.1.try_recv() {}
                removed = true;
            }
        )*
        removed
    }};
}


macro_rules! get_sender {
    ($self:expr, $channels:ident, $tab_id:expr) => {{
        if let Some(sender) = $self.$channels.get(&$tab_id) {
            sender.0.clone()
        } else {
            let (tx, rx) = unbounded();
            $self.$channels.insert($tab_id.clone(), (tx, rx).into());
            $self.$channels.get(&$tab_id).unwrap().0.clone()
        }
    }};
}


//trait y macro para el sender, ya no se usa una para cada una, send global
pub trait Routable: Sized {
    fn sender(pool: &NotifyingSender) -> &Sender<Self>;
}

macro_rules! route {
    ($($ty:ty => $field:ident),* $(,)?) => {
        $(impl Routable for $ty {
            fn sender(p: &NotifyingSender) -> &Sender<Self> {&p.$field}
        })*
    };
}

route! {
    FileLoadingMessage => file_sender,
    TaskMessage => task_sender,
    RecursiveMessages => recursive_search_sender,
    UiEvent => ui_event_sender,
    FileOperation => file_operation_sender,
    SizerMessages => sizer_sender,
    ExtendedInfoMessages => extended_info_sender,
    ThumbnailMessages => thumbnails_sender,
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
    pub fn send<M: Routable>(&self, msg: M) -> Result<(), SendError<M>> {
        let res = M::sender(self).send(msg);
        if res.is_ok() { (self.notifier)(); }
        res
    }
}

pub struct ChannelPool {
    pub file_channels: HashMap<Uuid, Arc<(Sender<FileLoadingMessage>, Receiver<FileLoadingMessage>)>>,
    pub task_channels: HashMap<Uuid, Arc<(Sender<TaskMessage>, Receiver<TaskMessage>)>>,
    pub recursive_channels: HashMap<Uuid, Arc<(Sender<RecursiveMessages>, Receiver<RecursiveMessages>)>>,
    pub ui_event_channels: HashMap<Uuid, Arc<(Sender<UiEvent>, Receiver<UiEvent>)>>,
    pub fileops_channels: HashMap<Uuid, Arc<(Sender<FileOperation>, Receiver<FileOperation>)>>,
    pub sizer_channels: HashMap<Uuid, Arc<(Sender<SizerMessages>, Receiver<SizerMessages>)>>,
    pub extended_info_channels: HashMap<Uuid, Arc<(Sender<ExtendedInfoMessages>, Receiver<ExtendedInfoMessages>)>>,
    pub thumbnails_channels: HashMap<Uuid, Arc<(Sender<ThumbnailMessages>, Receiver<ThumbnailMessages>)>>,

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
        let file_sender = get_sender! {
            self, 
            file_channels, 
            &tab_id
        };
        let task_sender = get_sender! {
            self, 
            task_channels, 
            &tab_id
        };
        let recursive_search_sender = get_sender! {
            self, 
            recursive_channels, 
            &tab_id
        };
        let ui_event_sender = get_sender! {
            self, 
            ui_event_channels, 
            &tab_id
        };
        let file_operation_sender = get_sender! {
            self, 
            fileops_channels, 
            &tab_id
        };
        let sizer_sender = get_sender! {
            self, 
            sizer_channels, 
            &tab_id
        };
        let extended_info_sender = get_sender! {
            self, 
            extended_info_channels, 
            &tab_id
        };
        let thumbnails_sender = get_sender! {
            self, 
            thumbnails_channels, 
            &tab_id
        };


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

    pub fn drain<M, F>(&self, channels: &HashMap<Uuid, Arc<(Sender<M>, Receiver<M>)>>,tab_id: Uuid, mut processor: F) -> bool
        where
            F: FnMut(M) -> bool {                
            let mut any = false;

            if let Some(arc_pair) = channels.get(&tab_id) {
                while let Ok(msg) = arc_pair.1.try_recv() {
                    if processor(msg) {
                        any = true;
                    }
                }
            }
            any
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


    pub fn remove_tab(&mut self, tab_id: Uuid) -> bool {
        self.ui_notifier.remove(&tab_id);

        let removed;

        removed = remove_arc_pair! {
            self,
            &tab_id,
            file_channels,
            task_channels,
            ui_event_channels,
            fileops_channels,
            sizer_channels,
            extended_info_channels,
            thumbnails_channels
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