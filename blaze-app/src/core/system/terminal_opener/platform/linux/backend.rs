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

use once_cell::sync::Lazy;
use std::{path::Path, process::Command, sync::Arc};
use tokio::sync::Mutex;
use tracing::warn;

use crate::core::system::knowndirs::knowndirs_manager::KnownDirsManager;

pub static LINUX_TERMINAL_OPENER: Lazy<Arc<Mutex<LinuxTerminalOpener>>> =
    Lazy::new(|| Arc::new(Mutex::new(LinuxTerminalOpener::init())));

pub struct LinuxTerminalOpener;

impl LinuxTerminalOpener {
    fn init() -> Self {
        Self {}
    }

    pub fn find_terminals_from_desktop(&self) -> Vec<String> {
        let mut terminals = Vec::new();
        let mut search_dirs = Vec::new();

        let home = KnownDirsManager::get().home.clone();

        if home.exists() {
            search_dirs.push(home.join(".local/share/applications/"));
        }

        let data_dirs = std::env::var("XDG_DATA_DIRS")
            .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());

        for dir in data_dirs.split(":") {
            search_dirs.push(Path::new(dir).join("applications"));
        }

        for dir in search_dirs {
            if !dir.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        let mut name: String = String::new();
                        let mut is_terminal: bool = false;

                        for line in content.lines() {
                            if let Some(v) = line.strip_prefix("GenericName=") {
                                is_terminal |= v.eq_ignore_ascii_case("Terminal emulator");
                            }

                            if let Some(v) = line.strip_prefix("Exec=") {
                                name = v.split_whitespace().next().unwrap_or("").into();
                            }

                            if let Some(v) = line.strip_prefix("Categories=") {
                                is_terminal |= v
                                    .split(";")
                                    .any(|c| c.eq_ignore_ascii_case("TerminalEmulator"));
                            }
                        }

                        if !is_terminal {
                            continue;
                        }

                        if !terminals.contains(&name) {
                            terminals.push(name.clone());
                        }
                    }
                }
            }
        }

        terminals
    }

    pub fn load_terminals(&self) -> Vec<String> {
        let mut terminals = Vec::new();
        let mut seen_names = std::collections::HashSet::new();

        let found = self.find_terminals_from_desktop();

        for term in found {
            let bin_name = Path::new(&term)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(&term)
                .to_string();

            if seen_names.insert(bin_name) {
                terminals.push(term);
            }
        }

        const TARGET_TERMINALS_FALLBACK: [&str; 15] = [
            "kitty",
            "alacritty",
            "wezterm",
            "terminator",
            "st",
            "termite",
            "rxvt",
            "urxvt",
            "xterm",
            "foot",
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
            "mate-terminal",
            "lxterminal",
        ];

        if terminals.is_empty() {
            terminals = TARGET_TERMINALS_FALLBACK
                .iter()
                .filter(|&&term| {
                    if let Ok(path) = std::env::var("PATH") {
                        std::env::split_paths(&path).any(|dir| dir.join(term).is_file())
                    } else {
                        false
                    }
                })
                .map(|term| (*term).to_owned())
                .collect();
        }

        terminals
    }

    pub fn open_terminal(
        &self,
        path: &Path,
        preferred_terminal: Option<&str>,
    ) -> std::io::Result<()> {
        if let Some(term) = preferred_terminal
            && !term.trim().is_empty()
        {
            if let Ok(_status) = Command::new(term).current_dir(path).spawn() {
                return Ok(());
            }
            warn!(
                "Terminal preferido '{}' no se pudo lanzar, usando fallback",
                term
            );
        }

        if let Ok(term) = std::env::var("TERMINAL") {
            return Command::new(term).current_dir(path).spawn().map(|_| ());
        }

        match Command::new("xdg-terminal-exec")
            .current_dir(path)
            .spawn()
            .map(|_| ())
        {
            Ok(_) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }

        match Command::new("x-terminal-emulator")
            .current_dir(path)
            .spawn()
            .map(|_| ())
        {
            Ok(_) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }

        //fallback por terminales posibles en sistema
        for term in self.load_terminals() {
            match Command::new(term).current_dir(path).spawn().map(|_| ()) {
                Ok(_) => return Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e),
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No se encontró ningún emulador de terminal",
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::core::system::{
        knowndirs::knowndirs_manager::KnownDirsManager,
        terminal_opener::platform::linux::backend::LinuxTerminalOpener,
    };

    #[test]
    fn test_load() {
        KnownDirsManager::init();
        let terminals = LinuxTerminalOpener;
        eprintln!(
            "Terminales desde load_terminals: {:?}",
            terminals.load_terminals()
        );
    }
}
