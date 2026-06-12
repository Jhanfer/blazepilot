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

use crate::core::system::fileopener_module::error::{OpenerError, OpenerResult};
use crate::core::system::fileopener_module::platform::opener_trait::AppInfo;
use crate::core::system::fileopener_module::platform::{FileOpener, PlatformOpener};
use directories::ProjectDirs;
use once_cell::sync::Lazy;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{path::Path, sync::Arc};
use tracing::warn;

#[derive(Debug, Default, Serialize, Deserialize)]
struct LocalAssociations {
    #[serde(default)]
    associations: HashMap<String, String>,
}

impl LocalAssociations {
    fn load(path: &Path) -> Self {
        let Ok(content) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        toml::from_str(&content).unwrap_or_default()
    }

    fn save(&self, path: &Path) -> OpenerResult<()> {
        let content = toml::to_string_pretty(self).map_err(|e| OpenerError::TomlError {
            path: path.into(),
            msg: "Serialización TOML ha fallado".into(),
            source: e,
        })?;
        std::fs::write(path, content).map_err(|e| OpenerError::Io {
            path: path.into(),
            msg: "Escritura ha fallado".into(),
            source: e,
        })
    }
}

pub static GLOBAL_FILE_OPENER: Lazy<Arc<Mutex<FileOpenerManager>>> =
    Lazy::new(|| Arc::new(Mutex::new(FileOpenerManager::init())));

pub struct FileOpenerManager {
    opener: PlatformOpener,
    local: RwLock<LocalAssociations>,
    local_path: Arc<Path>,
}

impl FileOpenerManager {
    fn init() -> Self {
        let local_path = Self::resolve_config_path();
        let local = LocalAssociations::load(&local_path);
        Self {
            opener: PlatformOpener::init(),
            local: RwLock::new(local),
            local_path,
        }
    }

    fn resolve_config_path() -> Arc<Path> {
        let path = ProjectDirs::from("com", "blazepilot", "blazepilotapp")
            .map(|dirs| {
                let dir = dirs.config_dir();
                let _ = std::fs::create_dir_all(dir);
                dir.join("mime_associations.toml")
            })
            .unwrap_or_else(|| {
                warn!("ProjectDirs no disponible, usando directorio actual");
                PathBuf::from("mime_associations.toml")
            });

        if !path.exists() {
            if let Err(e) = std::fs::File::create(&path) {
                warn!("No se pudo crear mime_associations.toml: {e}");
            }
        }

        path.into()
    }

    #[allow(unused)]
    pub fn get_mime(&self, path: Arc<Path>) -> String {
        self.opener.get_mime(path)
    }

    pub fn open_file(&mut self, path: Arc<Path>) -> OpenerResult<()> {
        let mime = self.opener.get_mime(path.clone());

        if let Some(app_id) = self.local.read().associations.get(&mime).cloned() {
            return self.opener.open_with(&app_id, path);
        }

        self.opener.open_file(path)
    }

    pub fn open_with(&self, app_id: &str, path: Arc<Path>) -> OpenerResult<()> {
        self.opener.open_with(app_id, path)
    }

    #[allow(unused)]
    pub fn get_available_apps(&self, path: Arc<Path>) -> OpenerResult<Vec<AppInfo>> {
        let mime = self.opener.get_mime(path.clone());
        let mut apps = self.opener.get_available_apps(path)?;

        if let Some(local_id) = self.local.read().associations.get(&mime).cloned() {
            for app in &mut apps {
                app.is_default = app.id == local_id;
            }
        }

        Ok(apps)
    }

    #[allow(unused)]
    pub fn get_default_app(&self, path: Arc<Path>) -> OpenerResult<Option<AppInfo>> {
        let mime = self.opener.get_mime(path.clone());

        if let Some(app_id) = self.local.read().associations.get(&mime).cloned() {
            let apps = self.opener.get_available_apps(path)?;
            let found = apps.into_iter().find(|a| a.id == app_id).map(|mut a| {
                a.is_default = true;
                a
            });
            return Ok(found);
        }

        self.opener.get_default_app(path)
    }

    pub fn get_all_apps(&self, path: Arc<Path>) -> OpenerResult<Vec<AppInfo>> {
        self.opener.get_all_apps(path)
    }

    pub fn set_default_app(
        &self,
        path: Arc<Path>,
        app_id: &str,
        save_to_system: bool,
    ) -> OpenerResult<()> {
        let mime = self.opener.get_mime(path.clone());
        {
            let mut local = self.local.write();
            local.associations.insert(mime.clone(), app_id.to_string());
            local.save(&self.local_path)?;
        }

        if save_to_system {
            self.opener.set_system_default(path, app_id)?;
        }

        Ok(())
    }

    #[allow(unused)]
    pub fn clear_local_default(&self, path: Arc<Path>) -> OpenerResult<()> {
        let mime = self.opener.get_mime(path);
        let mut local = self.local.write();
        local.associations.remove(&mime);
        local.save(&self.local_path)
    }
}
