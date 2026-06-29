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

use crate::core::{
    blaze_state::ViewMode,
    bootstrap::{
        configs::{
            error::ConfigResult,
            platform::{
                PlatformConfigTrait, PlatformConfigs,
                linux::conf_structs::{DisplayBackend, OrderingMode},
            },
        },
        i18n::I18n,
    },
};
use parking_lot::Mutex;
use std::{
    path::Path,
    sync::{Arc, LazyLock},
    time::{Duration, Instant, SystemTime},
};
use tracing::warn;

pub static GLOBAL_CONFIGS: LazyLock<Mutex<ConfigManager>> =
    LazyLock::new(|| Mutex::new(ConfigManager::new()));

pub fn with_configs<R>(f: impl FnOnce(&mut ConfigManager) -> R) -> R {
    f(&mut GLOBAL_CONFIGS.lock())
}

pub struct ConfigManager {
    platform: PlatformConfigs,
    last_change: Option<Instant>,
    is_dirty: bool,
    debounce: Duration,
}

impl ConfigManager {
    fn new() -> Self {
        let mut manager = Self {
            platform: PlatformConfigs::default(),
            last_change: None,
            is_dirty: false,
            debounce: Duration::from_millis(1000),
        };

        if let Err(e) = manager.platform.load() {
            warn!("Ha fallado la carga de las configuraciones. Usando confs por defecto: {e}");
        }

        manager
    }

    #[allow(unused)]
    pub fn config_dir(&self) -> &Path {
        self.platform.config_dir()
    }

    //--__--__--__--__ Getters  __--__--__--__--__--__--__

    pub fn get_ordering_mode(&self) -> OrderingMode {
        self.platform.app_ordering_mode.to_owned()
    }

    pub fn get_show_hidden_files(&self) -> bool {
        self.platform.show_hidden_files
    }

    pub fn get_display_backend(&self) -> DisplayBackend {
        self.platform.display_backend.to_owned()
    }

    pub fn get_default_terminal(&self) -> String {
        self.platform.default_terminal.to_owned()
    }

    pub fn get_should_ask_install(&self) -> bool {
        self.platform.should_ask_to_install
    }

    pub fn get_last_time_asked_install(&self) -> Option<SystemTime> {
        self.platform.last_time_asked_installation
    }

    pub fn get_locale(&self) -> Box<str> {
        self.platform.locale.to_owned()
    }

    pub fn get_i18n(&self) -> Arc<I18n> {
        self.platform.i18n.clone()
    }

    pub fn get_row_icon_size(&self) -> f32 {
        self.platform.row_icon_size
    }

    pub fn get_grid_icon_size(&self) -> f32 {
        self.platform.grid_icon_size
    }

    pub fn get_view_mode(&self) -> ViewMode {
        self.platform.view_mode.to_owned()
    }

    pub fn get_current_theme_name(&self) -> Box<str> {
        self.platform.theme.clone()
    }

    //--__--__--__--__ Setters  __--__--__--__--__--__--__

    pub fn set_ordering_mode(&mut self, mode: OrderingMode) {
        self.platform.app_ordering_mode = mode;
        self.save();
    }

    pub fn set_show_hidden_files(&mut self, show: bool) {
        self.platform.show_hidden_files = show;
        self.save();
    }

    pub fn set_display_backend(&mut self, backend: DisplayBackend) {
        self.platform.display_backend = backend;
        self.save();
    }

    pub fn set_default_terminal(&mut self, terminal: String) {
        self.platform.default_terminal = terminal;
        self.save();
    }

    pub fn set_should_ask_install(&mut self, ask: bool) {
        self.platform.should_ask_to_install = ask;
        self.save();
    }

    pub fn set_locale(&mut self, locale: &str) {
        self.platform.locale = locale.into();
        self.switch_i18n(locale);
        self.save();
    }

    pub fn set_row_icon_size(&mut self, size: f32) {
        self.platform.row_icon_size = size;
        self.save();
    }

    pub fn set_grid_icon_size(&mut self, size: f32) {
        self.platform.grid_icon_size = size;
        self.save();
    }

    pub fn set_view_mode(&mut self, view_mode: ViewMode) {
        self.platform.view_mode = view_mode;
        self.save();
    }

    pub fn set_current_theme_name(&mut self, theme_name: &str) {
        self.platform.theme = theme_name.into();
        self.save();
    }

    //--__--__--__--__ Recarga y Guardado  __--__--__--__--__--__--__

    fn switch_i18n(&self, locale: &str) {
        self.platform.i18n.switch_locale(locale);
    }

    pub fn save(&mut self) {
        self.is_dirty = true;
        self.last_change = Some(Instant::now());
    }

    #[must_use = "el resultado de save() debe comprobarse por si falla"]
    pub fn force_save(&mut self) -> ConfigResult<()> {
        self.platform.save()
    }

    #[allow(unused)]
    #[must_use = "el resultado de reload() debe comprobarse por si falla"]
    pub fn reload(&mut self) -> ConfigResult<()> {
        self.platform.load()
    }

    #[must_use = "el resultado de save() debe comprobarse por si falla"]
    pub fn tick(&mut self) -> ConfigResult<()> {
        if !self.is_dirty {
            return Ok(());
        }

        let Some(last_change) = self.last_change else {
            return Ok(());
        };

        if last_change.elapsed() >= self.debounce {
            tracing::info!("Guardando...");
            self.platform.save()?;
            self.is_dirty = false;
            self.last_change = None;
        }

        Ok(())
    }
}
