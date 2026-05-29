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





use parking_lot::Mutex;
use std::{ 
    path::Path, 
    sync::LazyLock,
    time::SystemTime
};
use tracing::warn;
use crate::core::bootstrap::{
    configs::{
        error::ConfigResult, 
        platform::{
            PlatformConfigTrait, 
            PlatformConfigs, 
            linux::conf_structs::{
                DisplayBackend,
                OrderingMode
            }
        }
    }
};


pub static GLOBAL_CONFIGS: LazyLock<Mutex<ConfigManager>> = LazyLock::new(|| {
    Mutex::new(ConfigManager::new())
});

pub fn with_configs<R>(f: impl FnOnce(&mut ConfigManager) -> R) -> R {
    f(&mut GLOBAL_CONFIGS.lock())
}

pub struct ConfigManager {
    platform: PlatformConfigs,
}

impl ConfigManager {
    fn new() -> Self {
        let mut manager = Self {
            platform: PlatformConfigs::default(),
        };

        if let Err(e) = manager.platform.load() {
            warn!("Ha fallado la carga de las configuraciones. Usando confs por defecto: {e}");
        }

        manager
    }

    #[must_use]
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

//--__--__--__--__ Setters  __--__--__--__--__--__--__

    pub fn set_ordering_mode(&mut self, mode: OrderingMode) {
        self.platform.app_ordering_mode = mode;
        self.platform.save().ok();
    }

    pub fn set_show_hidden_files(&mut self, show: bool) {
        self.platform.show_hidden_files = show;
        self.platform.save().ok();
    }

    pub fn set_display_backend(&mut self, backend: DisplayBackend) {
        self.platform.display_backend = backend;
        self.platform.save().ok();
    }

    pub fn set_default_terminal(&mut self, terminal: String) {
        self.platform.default_terminal = terminal;
        self.platform.save().ok();
    }

    pub fn set_should_ask_install(&mut self, ask: bool) {
        self.platform.should_ask_to_install = ask;
        self.platform.save().ok();
    }

//--__--__--__--__ Recarga y Guardado  __--__--__--__--__--__--__


    #[must_use]
    pub fn save(&self) -> ConfigResult<()> {
        self.platform.save()
    }

    #[must_use]
    pub fn reload(&mut self) -> ConfigResult<()> {
        self.platform.load()
    }
}