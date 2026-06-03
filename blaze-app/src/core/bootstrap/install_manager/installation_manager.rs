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
    sync::{Arc, LazyLock},
};
use tracing::{debug, info, warn};

use crate::core::bootstrap::install_manager::platform::{
    installation_trait::InstallationTrait, PlatformInstallation,
};

pub static GLOBAL_INSTALLATION_MANAGER: LazyLock<Mutex<InstallationManager>> =
    LazyLock::new(|| Mutex::new(InstallationManager::new()));

pub fn with_installation_manager<R>(f: impl FnOnce(&InstallationManager) -> R) -> R {
    f(&GLOBAL_INSTALLATION_MANAGER.lock())
}

pub enum InstallResult {
    AlreadyInstalled,
    InstalledSystem(Arc<Path>),
    InstalledLocal(Arc<Path>),
    Failed(Box<str>),
}

pub struct InstallationManager {
    platform: PlatformInstallation,
}

impl InstallationManager {
    pub fn new() -> Self {
        Self {
            platform: PlatformInstallation::default(),
        }
    }

    #[must_use]
    pub fn is_installed(&self) -> bool {
        let current = match std::env::current_exe() {
            Ok(p) => p,
            Err(_) => return false,
        };

        let system = self.platform.installation_path();
        let fallback = self.platform.fallback_path();

        let runs_from_installation = (current == system || current == fallback) && current.exists();

        let exists_in_system_paths =
            (system.exists() && system.is_file()) || (fallback.exists() && fallback.is_file());

        debug!(
            "Está instalado? {}",
            runs_from_installation || exists_in_system_paths
        );

        runs_from_installation || exists_in_system_paths
    }

    #[must_use]
    pub fn install(&self) -> InstallResult {
        if self.is_installed() {
            return InstallResult::AlreadyInstalled;
        }

        let result = self.platform.install();

        match &result {
            InstallResult::InstalledSystem(path) | InstallResult::InstalledLocal(path) => {
                if let Err(e) = self.platform.post_install() {
                    warn!("post_install failed: {e}");
                } else {
                    info!("Instalado en: {}", path.display());
                }
            }
            _ => {}
        }

        result
    }
}
