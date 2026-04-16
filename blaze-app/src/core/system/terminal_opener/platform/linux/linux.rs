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





use std::{collections::{HashMap, HashSet}, ffi::OsString, fs, num::NonZeroUsize, path::{Path, PathBuf}, process::Command, sync::Arc};
use once_cell::sync::Lazy;
use tokio::sync::{Mutex, RwLock};
use tracing::{warn, info};
use uuid::Uuid;
use crate::{core::system::fileopener_module::{AppAssociation, platform::linux::{appassociation::AssociationManager, mimeapps::MimeApps}}, utils::channel_pool::{NotifyingSender, UiEvent}};



pub static LINUX_TERMINAL_OPENER: Lazy<Arc<tokio::sync::Mutex<LinuxTerminalOpener>>> = Lazy::new(|| {
    Arc::new(tokio::sync::Mutex::new(LinuxTerminalOpener::init()))
});

pub struct LinuxTerminalOpener {
    terminal: Option<String>,
}

impl LinuxTerminalOpener {
    fn init() -> Self {


        Self {
            terminal: None,
        }
    }

    pub fn load_terminals(&self) -> Vec<String> {
        let target_terminals = vec![
            "gnome-terminal", "konsole", "xfce4-terminal", "terminator",
            "alacritty", "kitty", "rxvt", "urxvt", "st", "termite",
            "lxterminal", "mate-terminal", "xterm", "wezterm", "foot"
        ];

        target_terminals
            .iter()
            .filter(|&&term| {
                if let Ok(path) = std::env::var("PATH") {
                    std::env::split_paths(&path).any(|dir|{
                        dir.join(term).is_file()
                    })
                } else {
                    false
                }
            })
            .map(|term| (*term).to_owned())
            .collect()
    }

    pub fn open_terminal(&self, path: &Path) -> std::io::Result<()> {
        if let Ok(term) = std::env::var("TERMINAL") {
            return Command::new(term)
                .current_dir(path)
                .spawn()
                .map(|_| ());
        }

        match Command::new("xdg-terminal-exec")
            .current_dir(path)
            .spawn()
            .map(|_| ()) {
                Ok(_) => return Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}, 
                Err(e) => return Err(e),
        }

        match Command::new("x-terminal-emulator")
            .current_dir(path)
            .spawn()
            .map(|_| ()) {
                Ok(_) => return Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}, 
                Err(e) => return Err(e),
        }


        //fallback por terminales posibles en sistema
        for term in self.load_terminals() {
            match Command::new(term)
                .current_dir(path)
                .spawn()
                .map(|_| ()) {
                    Ok(_) => return Ok(()),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}, 
                    Err(e) => return Err(e),
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No se encontró ningún emulador de terminal"
        ))
    }

}