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



use std::cell::Cell;
use std::sync::Arc;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use uuid::Uuid;
use crossbeam_channel::{Receiver, SendError, Sender};
use crate::core::files::blaze_motor::motor_structs::{FileLoadingMessage, RecursiveMessages};
use crate::core::system::extended_info::extended_info_manager::ExtendedInfoMessages;
use crate::core::system::sizer_manager::sizer_manager::SizerMessages;
use crate::ui::icons_cache::thumbnails::thumbnails_manager::ThumbnailMessages;
use crate::ui::task_manager::task_manager::TaskMessage;
use tracing::{info, warn};
use crossbeam_channel::unbounded;

use crate::core::runtime::bus_structs::*;
use crate::core::runtime::bus_macros::*;



//Inicializador del par de canales
pub struct ChannelPair<M> {
    pub sender: Sender<M>,
    pub receiver: Receiver<M>,
}

impl<M> ChannelPair<M> {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }
}


pub struct ChannelGroup {
    pub file: ChannelPair<FileLoadingMessage>,
    pub task: ChannelPair<TaskMessage>,
    pub recursive: ChannelPair<RecursiveMessages>,
    pub ui: ChannelPair<UiEvent>,
    pub fileops: ChannelPair<FileOperation>,
    pub sizer: ChannelPair<SizerMessages>,
    pub extended: ChannelPair<ExtendedInfoMessages>,
    pub thumbnails: ChannelPair<ThumbnailMessages>,

    pub notifier: Arc<dyn Fn() + Send + Sync>,
}

impl ChannelGroup {
    pub fn new() -> Self {
        Self {
            file: ChannelPair::new(),
            task: ChannelPair::new(),
            recursive: ChannelPair::new(),
            ui: ChannelPair::new(),
            fileops: ChannelPair::new(),
            sizer: ChannelPair::new(),
            extended: ChannelPair::new(),
            thumbnails: ChannelPair::new(),
            notifier: Arc::new(|| {}),
        }
    }
}


pub struct EventBus {
    pub tabs: DashMap<Uuid, ChannelGroup>,
}

static EVENT_BUS: Lazy<EventBus> = Lazy::new(|| {
    EventBus { tabs: DashMap::new() }
});

pub fn with_event_bus<R>(f: impl FnOnce(&EventBus) -> R) -> R {
    f(&EVENT_BUS)
}



thread_local! {
    static ACTIVE_TAB_ID: Cell<Uuid> = Cell::new(Uuid::nil());
}

pub fn set_active_tab(id: Uuid) {
    ACTIVE_TAB_ID.with(|c| c.set(id));
}

pub fn active_tab_id() -> Uuid {
    ACTIVE_TAB_ID.with(|c| c.get())
}


#[derive(Clone)]
pub struct Dispatcher {
    pub tab_id: Uuid,
}


impl Dispatcher {
    pub fn current() -> Self {
        Self { tab_id: active_tab_id() }
    }

    pub fn send<M: Routable>(&self, msg: M) -> Result<(), SendError<M>> {
        let Some(group) = EVENT_BUS.tabs.get(&self.tab_id) else {
            warn!("Error en send: No se ha encontrado 'ChannelGroup' con esta id: {}", self.tab_id);
            return Err(SendError(msg))
        };
        let res = M::sender(&group).send(msg);
        if res.is_ok() {
            (group.notifier)();
        }
        if res.is_err() {
            warn!("Error en send: {:?}", res);
        }
        res
    }
}


impl EventBus {
    pub fn create_tab(&self, tab_id: Uuid) {
        self.tabs.insert(tab_id, ChannelGroup::new());
    }

    pub fn remove_tab(&self, tab_id: Uuid) {
        if let Some((_, group)) = self.tabs.remove(&tab_id) {
            drop(group);
            info!("Tab {} eliminada", tab_id);
        } else {
            warn!("Intento de eliminar una tab inexistente {}", tab_id);
        }
    }

    pub fn dispatcher(&self, tab_id: Uuid) -> Dispatcher {
        Dispatcher { tab_id }
    }

    pub fn drain<M, F>(&self, tab_id: Uuid, mut processor: F) -> bool
    where
        M: Routable,
        F: FnMut(M) -> bool,
    {
        let mut any = false;
        if let Some(group) = self.tabs.get(&tab_id) {
            let rx = M::receiver(&group);
            while let Ok(msg) = rx.try_recv() {
                if processor(msg) {
                    any = true;
                }
            }
        }
        any
    }
}