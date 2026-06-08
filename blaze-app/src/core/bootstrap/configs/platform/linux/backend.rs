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

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::core::bootstrap::{
    configs::{
        error::{ConfigError, ConfigResult},
        platform::{
            linux::conf_structs::{DisplayBackend, OrderingMode},
            PlatformConfigTrait,
        },
    },
    i18n::I18n,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct LinuxConfigs {
    #[serde(default)]
    pub app_ordering_mode: OrderingMode,

    #[serde(default)]
    config_file_path: PathBuf,

    #[serde(default)]
    pub show_hidden_files: bool,

    #[serde(default)]
    pub item_file_list_size: usize,

    #[serde(default)]
    pub default_terminal: String,

    #[serde(default)]
    pub display_backend: DisplayBackend,

    #[serde(default)]
    pub theme: String,

    #[serde(default)]
    pub accent_color: Option<String>,

    #[serde(default)]
    pub font_size: f32,

    #[serde(default)]
    pub confirm_on_delete: bool,

    #[serde(default)]
    pub should_ask_to_install: bool,

    #[serde(default)]
    pub last_time_asked_installation: Option<SystemTime>,

    #[serde(default)]
    pub locale: Box<str>,

    #[serde(skip)]
    pub i18n: Arc<I18n>,
}

impl LinuxConfigs {
    fn init_config_path() -> ConfigResult<PathBuf> {
        let proj = ProjectDirs::from("com", "blazepilot", "blazepilotapp")
            .ok_or(ConfigError::ProjectDirsNotFound)?;

        let dir = proj.config_dir();
        std::fs::create_dir_all(dir).map_err(ConfigError::Io)?;

        let path = dir.join("config.json");
        if !path.exists() {
            std::fs::File::create(&path).map_err(ConfigError::Io)?;
        }

        Ok(path)
    }
}

impl Default for LinuxConfigs {
    fn default() -> Self {
        let path = Self::init_config_path().unwrap_or_default();

        let lang = std::env::var("LANG").unwrap_or(String::from("en_US.UTF-8"));
        let locale = lang.split("_").next().unwrap_or("en");

        let i18n = Arc::new(I18n::load(locale));
        info!("Llamando i18n en default");

        Self {
            app_ordering_mode: OrderingMode::Az,
            config_file_path: path,
            show_hidden_files: false,
            item_file_list_size: 10,
            default_terminal: String::new(),
            display_backend: DisplayBackend::Auto,
            theme: "system".to_string(),
            accent_color: None,
            font_size: 14.0,
            confirm_on_delete: true,
            should_ask_to_install: true,
            last_time_asked_installation: None,
            locale: locale.into(),
            i18n,
        }
    }
}

impl PlatformConfigTrait for LinuxConfigs {
    fn config_dir(&self) -> &Path {
        self.config_file_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
    }

    fn load(&mut self) -> ConfigResult<()> {
        let path = self.config_file_path.clone();

        if !path.exists() {
            return self.save();
        }

        let data = std::fs::read_to_string(&path).map_err(ConfigError::Io)?;

        if data.trim().is_empty() {
            return self.save();
        }

        let mut loaded: LinuxConfigs =
            serde_json::from_str(&data).map_err(|_| ConfigError::Deserialize)?;

        let locale = &loaded.locale;
        let i18n = Arc::new(I18n::load(locale));
        loaded.i18n = i18n;
        info!("Llamando i18n en load de linux");

        loaded.config_file_path = path;
        *self = loaded;

        Ok(())
    }

    fn save(&self) -> ConfigResult<()> {
        let data = serde_json::to_string_pretty(self).map_err(|_| ConfigError::Serialize)?;

        std::fs::write(&self.config_file_path, data).map_err(ConfigError::Io)?;

        Ok(())
    }
}
