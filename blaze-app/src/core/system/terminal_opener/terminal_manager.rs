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







use std::{path::Path, sync::Arc};
use once_cell::sync::Lazy;
use tokio::sync::Mutex;

#[cfg(target_os = "linux")]
use crate::core::system::terminal_opener::platform::linux::linux::LinuxTerminalOpener;


#[derive(Clone)]
enum PlatformTerminal {
    #[cfg(target_os = "linux")]
    Linux(Arc<Mutex<LinuxTerminalOpener>>),
}


pub static GLOBAL_TERMINAL_MANAGER: Lazy<Arc<tokio::sync::Mutex<TerminalManager>>> = Lazy::new(|| {
    Arc::new(tokio::sync::Mutex::new(TerminalManager::new()))
});


pub struct TerminalManager {
    terminal_opener: PlatformTerminal,
}

impl TerminalManager {
    pub fn new() -> Self {
        Self {
            terminal_opener : {
                #[cfg(target_os = "linux")]
                {
                    use tracing::debug;

                    use crate::core::system::terminal_opener::platform::linux::linux::LINUX_TERMINAL_OPENER;

                    debug!("Usando terminal opener de linux");

                    PlatformTerminal::Linux(LINUX_TERMINAL_OPENER.clone())
                }
            }
        }
    }

    //pedir la lista disponible de terminales
    pub async fn request_load_terminals(&mut self) -> Vec<String> {
        match &mut self.terminal_opener {
            #[cfg(target_os = "linux")]
            PlatformTerminal::Linux(tm) => {
                let guard = tm.lock().await;
                guard.load_terminals()
            },
        }
    }

    //lanzar la terminal
    pub async fn request_open_terminal(&mut self, path: &Path, preferred_terminal: Option<String>) -> std::io::Result<()> {
        match &mut self.terminal_opener {
            #[cfg(target_os = "linux")]
            PlatformTerminal::Linux(tm) => {
                let guard = tm.lock().await;
                guard.open_terminal(path, preferred_terminal.as_deref())
            },
        }
    }
}