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





use std::{path::PathBuf, sync::Arc};
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use tokio::sync::Mutex;
use tracing::{error, debug};


#[cfg(target_os = "linux")]
use crate::core::system::fileopener_module::platform::linux::linux::LinuxOpener;

#[cfg(target_os = "macos")]
use crate::core::system::fileopener_module::platform::macos::{MacosOpener, MACOS_FILE_OPENER};

#[cfg(target_os = "windows")]
use crate::core::system::fileopener_module::platform::windows::{WindowsOpener, WINDOWS_FILE_OPENER};
use crate::core::runtime::event_bus::Dispatcher;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppAssociation {
    pub id: String,
    pub name: String, 
    pub exec: String,
    pub icon: Option<String>,
    pub is_private: bool,
    pub is_recommended: bool
}


enum PlatformOpener {
    #[cfg(target_os = "linux")]
    Linux(Arc<Mutex<LinuxOpener>>),
    #[cfg(target_os = "macos")]
    Macos(Arc<Mutex<MacosOpener>>),
    #[cfg(target_os = "windows")]
    Windows(Arc<Mutex<WindowsOpener>>),
}


pub static GLOBAL_FILE_OPENER: Lazy<Arc<tokio::sync::Mutex<FileOpenerManager>>> = Lazy::new(|| {
    Arc::new(tokio::sync::Mutex::new(FileOpenerManager::new()))
});


pub struct FileOpenerManager {
    opener: PlatformOpener,
}


impl FileOpenerManager {
    pub fn new() -> Self {
        Self { 
            opener: {
                #[cfg(target_os = "linux")]
                {
                    use crate::core::system::fileopener_module::platform::linux::linux::LINUX_FILE_OPENER;

                    debug!("Usando FileOpener Linux");
                    PlatformOpener::Linux(LINUX_FILE_OPENER.clone())
                }

                #[cfg(target_os = "macos")]
                {
                    debug!("Usando FileOpener MacOS");
                    PlatformOpener::Macos(MACOS_FILE_OPENER.clone())
                }

                #[cfg(target_os = "windows")]
                {
                    debug!("Usando FileOpener Windows");
                    PlatformOpener::Windows(WINDOWS_FILE_OPENER.clone())
                }
            }
        }
    }


    pub async fn request_open_file(&mut self, path: PathBuf, sender: Dispatcher) {
        match &mut self.opener {
            #[cfg(target_os = "linux")]
            PlatformOpener::Linux(op) => {
                let mut guard = op.lock().await;
                guard.open_file_linux(path, sender).await;
            }
            #[cfg(target_os = "macos")]
            PlatformOpener::Macos(op) => {
                let mut guard = op.lock().await;
                guard.open_file_windows(path, ui).await;
            }
            #[cfg(target_os = "windows")]
            PlatformOpener::Windows(op) => {
                let mut guard = op.lock().await;
                guard.open_file_macos(path, ui).await;
            }
        }
    }

    pub async fn request_open_file_with(&mut self, path: PathBuf, sender: Dispatcher) {
        match &mut self.opener {
            #[cfg(target_os = "linux")]
            PlatformOpener::Linux(op) => {
                let mut guard = op.lock().await;
                guard.open_file_with_linux(path, sender).await;
            }
            #[cfg(target_os = "macos")]
            PlatformOpener::Macos(op) => {
                let mut guard = op.lock().await;
                guard.open_file_with_windows(path, ui).await;
            }
            #[cfg(target_os = "windows")]
            PlatformOpener::Windows(op) => {
                let mut guard = op.lock().await;
                guard.open_file_with_macos(path, ui).await;
            }
        }
    }


    pub async fn take_pending_mime(&mut self) -> Option<String> {
        match &mut self.opener {
            #[cfg(target_os = "linux")]
            PlatformOpener::Linux(op) => op.lock().await.pending_mime.take(),
            #[cfg(target_os = "macos")]
            PlatformOpener::Macos(op) => op.lock().await.pending_mime.take(),
            #[cfg(target_os = "windows")]
            PlatformOpener::Windows(op) => op.lock().await.pending_mime.take(),
        }
    }


    pub async fn take_pending_path(&mut self) -> Option<PathBuf> {
        match &mut self.opener {
            #[cfg(target_os = "linux")]
            PlatformOpener::Linux(op) => op.lock().await.pending_path.take(),
            #[cfg(target_os = "macos")]
            PlatformOpener::Macos(op) => op.lock().await.pending_path.take(),
            #[cfg(target_os = "windows")]
            PlatformOpener::Windows(op) => op.lock().await.pending_path.take(),
        }
    }

    pub async fn take_pending_default_app_name(&mut self) -> Option<String> {
        match &mut self.opener {
            #[cfg(target_os = "linux")]
            PlatformOpener::Linux(op) => op.lock().await.pending_default_app_name.take(),
            #[cfg(target_os = "macos")]
            PlatformOpener::Macos(op) => op.lock().await.pending_default_app_name.take(),
            #[cfg(target_os = "windows")]
            PlatformOpener::Windows(op) => op.lock().await.pending_default_app_name.take(),
        }
    }

    pub async fn set_association(&mut self, mime: &str, app: AppAssociation) {
        match &mut self.opener {
            #[cfg(target_os = "linux")]
            PlatformOpener::Linux(op) => op.lock().await.assoc_manager.set_association(mime, app),
            #[cfg(target_os = "macos")]
            PlatformOpener::Macos(op) => op.lock().await.assoc_manager.set_association(mime, app),
            #[cfg(target_os = "windows")]
            PlatformOpener::Windows(op) => op.lock().await.assoc_manager.set_association(mime, app),
        }
    }

    pub async fn set_pending(&mut self, path: PathBuf, mime: String) {
        match &mut self.opener {
            #[cfg(target_os = "linux")]
            PlatformOpener::Linux(op) => {
                op.lock().await.pending_path = Some(path);
                op.lock().await.pending_mime = Some(mime);
            },
            #[cfg(target_os = "macos")]
            PlatformOpener::Macos(op) => {
                op.lock().await.pending_path = Some(path);
                op.lock().await.pending_mime = Some(mime);
            },
            #[cfg(target_os = "windows")]
            PlatformOpener::Windows(op) => {
                op.lock().await.pending_path = Some(path);
                op.lock().await.pending_mime = Some(mime);
            }
        }
    }

    pub async fn set_pending_default_app_name(&mut self, mime: String) {
        match &mut self.opener {
            #[cfg(target_os = "linux")]
            PlatformOpener::Linux(op) => {
                op.lock().await.pending_default_app_name = Some(mime);
            },
            #[cfg(target_os = "macos")]
            PlatformOpener::Macos(op) => {
                op.lock().await.pending_default_app_name = Some(mime);
            },
            #[cfg(target_os = "windows")]
            PlatformOpener::Windows(op) => {
                op.lock().await.pending_default_app_name = Some(mime);
            }
        }
    }


    pub async fn request_launch(&mut self, app: &AppAssociation, path: &PathBuf) {
        match &mut self.opener {
            #[cfg(target_os = "linux")]
            PlatformOpener::Linux(op) => {
                let guard = op.lock().await;
                if let Err(e) = guard.launch_app_linux(app, path) {
                    error!("Error lanzando app en Linux: {}", e);
                }
            }
            #[cfg(target_os = "macos")]
            PlatformOpener::Macos(op) => {
                let guard = op.lock().await;
                guard.launch_app_macos(app, path).await;
            }
            #[cfg(target_os = "windows")]
            PlatformOpener::Windows(op) => {
                let guard = op.lock().await;
                guard.launch_app_windows(app, path).await;
            }
        }
    }
}