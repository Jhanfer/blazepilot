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





use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use tracing::error;
use std::path::Path;
use crate::{core::{files::blaze_motor::{error::MotorResult, motor_structs::FileLoadingMessage}, system::{clipboard::TOKIO_RUNTIME}}, utils::channel_pool::NotifyingSender};



pub struct FileWatcher {
    pub watcher: Option<Box<dyn Watcher + Send>>,
    pub watching: Arc<AtomicBool>,
    pub watching_handle: Option<tokio::task::JoinHandle<()>>,
}

impl FileWatcher {
    pub fn start() -> Self {
        Self {
            watcher: None,
            watching: Arc::new(AtomicBool::new(false)),
            watching_handle: None,
        }
    }

    pub fn stop_watching(&mut self) {
        self.watching.store(false, Ordering::Relaxed);
        self.watcher = None;

        if let Some(handle) = self.watching_handle.take() {
            handle.abort();
        }
    }


    fn handle_event_match<F>(event: &Event, callback: F) 
        where F: Fn(&str) {
        if let Some(path) = event.paths.first() {
            if let Some(name) = path.file_name() {
                let name = name.to_string_lossy();
                callback(&name)
            }
        }
    }

    fn handle_watcher_event(event: Event, sender: &NotifyingSender) {
        match event.kind {
            EventKind::Create(_) => {
                Self::handle_event_match(&event, |name|{
                    if let Err(e) = sender.send_files_batch(FileLoadingMessage::FileAdded { name: name.to_owned() }) {
                        error!("Error de enviado en watcher: {}", e);
                    }
                });
            },

            EventKind::Remove(_) => {
                Self::handle_event_match(&event, |name|{
                    if let Err(e) = sender.send_files_batch(FileLoadingMessage::FileRemoved { name: name.to_owned() }) {
                        error!("Error de enviado en watcher: {}", e);
                    }
                });
            },

            EventKind::Modify(_) => {

                let is_git_change = event.paths.iter().any(|p|{
                    p.components().any(|c| c.as_os_str() == ".git")
                });

                if is_git_change {
                    if let Err(e) = sender.send_files_batch(FileLoadingMessage::GitStatusChanged) {
                        error!("Error enviando GitStatusChanged: {}", e);
                    }
                } else {
                    Self::handle_event_match(&event, |name| {
                        let _ = sender.send_files_batch(
                            FileLoadingMessage::FileModified { name: name.to_owned() }
                        );
                    });
                }
            }

            _ => {},
        }
    }


    pub fn start_watching(&mut self, path: &Path, sender: NotifyingSender) -> MotorResult<()> {
        self.stop_watching();

        let watching = Arc::new(AtomicBool::new(true));
        self.watching = watching.clone();
        
        let (fs_tx, mut fs_rx) = tokio::sync::mpsc::channel(100);
        
        // se crea el watcher
        let mut watcher = notify::recommended_watcher(move |res| {
            match res {
                Ok(ev) => {
                    match fs_tx.try_send(ev) {
                        Ok(_) => {},
                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                            error!("!Canales de watcher llenos!");
                        },

                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                            error!("watcher channel cerrado");
                        },
                    }
                },
                Err(err) => {
                    error!("recommended_watcher no ha podido crear el watcher: {}", err);
                },
            }
        })?;
        
        watcher.watch(path, RecursiveMode::NonRecursive)?;
        self.watcher = Some(Box::new(watcher) as Box<dyn Watcher + Send>);
        
        
        let handle = TOKIO_RUNTIME.spawn(async move {
            while watching.load(Ordering::Acquire) {
                if let Some(event) = fs_rx.recv().await {
                    Self::handle_watcher_event(event, &sender)
                }
            }
        });

        self.watching_handle = Some(handle);
        Ok(())
    }
}