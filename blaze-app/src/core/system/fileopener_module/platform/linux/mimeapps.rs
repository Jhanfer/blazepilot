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





use std::{env, fs, collections::HashSet, path::PathBuf};
use crate::core::system::fileopener_module::platform::linux::mimeappsfile::MimeAppsFile;


pub struct MimeApps {
    files: Vec<PathBuf>,
    parsed: Vec<MimeAppsFile>,
}

impl MimeApps {
    pub fn load() -> Self {
        let mut files = Vec::new();


        // XDG_CONFIG_HOME o ~/.config
        let config_home = env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let mut p = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
                p.push(".config");
                p
            });

        let user_mimeapps = config_home.join("mimeapps.list");
        if user_mimeapps.is_file() {
            files.push(user_mimeapps);
        }


        // XDG_CONFIG_DIRS (p.ej. /etc/xdg)
        if let Some(config_dirs) = env::var_os("XDG_CONFIG_DIRS") {
            for dir in env::split_paths(&config_dirs) {
                let path = dir.join("mimeapps.list");
                if path.is_file() {
                    files.push(path);
                }
            }
        } else {
            // valor por defecto de XDG_CONFIG_DIRS es /etc/xdg
            let default_dir = PathBuf::from("/etc/xdg/mimeapps.list");
            if default_dir.is_file() {
                files.push(default_dir);
            }
        }


        // XDG_DATA_HOME/applications/mimeapps.list 
        let data_home = env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let mut p = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));
                p.push(".local");
                p.push("share");
                p
            });

        let data_home_mimeapps = data_home.join("applications").join("mimeapps.list");
        if data_home_mimeapps.is_file() {
            files.push(data_home_mimeapps);
        }


        // XDG_DATA_DIRS/applications/mimeapps.list (p.ej. /usr/share/applications)
        if let Some(data_dirs) = env::var_os("XDG_DATA_DIRS") {
            for dir in env::split_paths(&data_dirs) {
                let path = dir.join("applications").join("mimeapps.list");
                if path.is_file() {
                    files.push(path);
                }
            }
        } else {
            // valor por defecto de XDG_DATA_DIRS es "/usr/local/share:/usr/share"
            for dir in &["/usr/local/share", "/usr/share"] {
                let path = PathBuf::from(dir).join("applications").join("mimeapps.list");
                if path.is_file() {
                    files.push(path);
                }
            }
        }

        let mut parsed = Vec::new();
        for path in &files {
            if let Ok(content) = fs::read_to_string(path) {
                let mf = MimeAppsFile::parse(&content);
                parsed.push(mf);
            }
        }

        Self { files, parsed }
    }


    pub fn apps_for_mime(&self, mime: &str) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        let mut blacklist: HashSet<String> = HashSet::new();

        for mf in &self.parsed {
            if let Some(removed) = mf.removed.get(mime) {
                for app in removed {
                    blacklist.insert(app.clone());
                }
            }

            if let Some(added) = mf.added.get(mime) {
                for app in added {
                    if !blacklist.contains(app) && seen.insert(app.clone()) {
                        result.push(app.clone());
                    }
                }
            }
        }

        result
    }

    pub fn default_for_mime(&self, mime: &str) -> Option<String> {
        for mf in &self.parsed {
            if let Some(defaults) = mf.added.get(mime) {
                if let Some(first) = defaults.first() {
                    return Some(first.clone());
                }
            }
            if let Some(defaults) = mf.default_apps.get(mime) {
                if let Some(first) = defaults.first() {
                    return Some(first.clone());
                }
            }
        }
        None
    }


    pub fn is_removed(&self, mime: &str, desktop_id: &str) -> bool {
        self.parsed.iter().any(|mf| {
            mf.removed.get(mime)
                .map(|v| v.iter().any(|d| d == desktop_id ))
                .unwrap_or(false)
        })
    }


    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    pub fn parsed(&self) -> &[MimeAppsFile] {
        &self.parsed
    }
}