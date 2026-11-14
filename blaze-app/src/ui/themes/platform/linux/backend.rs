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

use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc};
use tracing::{debug, info, warn};

use crate::ui::themes::platform::structs::LegacyTheme;
#[allow(deprecated)]
use crate::{
    core::{
        bootstrap::configs::config_manager::with_configs,
        system::knowndirs::knowndirs_manager::KnownDirsManager,
    },
    ui::themes::platform::{
        ColorsTrait,
        structs::{BlazeAppTheme, ThemeDefault},
    },
};

#[derive(Serialize, Deserialize)]
pub struct LinuxTheme {
    path: Arc<Path>,
    active_theme: Arc<BlazeAppTheme>,
    available_themes_names: Vec<Box<str>>,
    available_themes: Vec<Arc<BlazeAppTheme>>,
}

impl ColorsTrait for LinuxTheme {
    fn init() -> Self {
        Self::init_defaults()
    }

    fn update_theme<V>(&mut self, mutator: fn(&mut BlazeAppTheme, V), value: V) {
        let theme_mutable = Arc::make_mut(&mut self.active_theme);
        mutator(theme_mutable, value);
    }

    fn set_theme(&mut self, name: &str) {
        if let Some(theme) = self
            .available_themes
            .iter()
            .find(|t| t.meta.name.as_ref() == name)
        {
            self.active_theme = Arc::clone(theme);
        }
    }

    fn available_themes(&self) -> Vec<Box<str>> {
        self.available_themes_names.clone()
    }

    fn current_theme(&self) -> Arc<BlazeAppTheme> {
        self.active_theme.clone()
    }

    fn save(&mut self) -> Result<(), String> {
        let json =
            serde_json::to_string_pretty(self.active_theme.as_ref()).map_err(|e| e.to_string())?;

        let file_name = format!(
            "{}.json",
            self.active_theme.meta.name.to_lowercase().replace(' ', "_")
        );
        let path = self.path.join(file_name);

        std::fs::write(&path, json).map_err(|e| e.to_string())
    }

    fn load(&mut self) -> Result<(), String> {
        if !self.path.exists() {
            std::fs::create_dir_all(&self.path).map_err(|e| e.to_string())?;
        }

        self.write_missing_defaults()?;
        self.scan_themes();

        let current_theme_config = with_configs(|c| c.get_current_theme_name());

        if let Some(first) = self
            .available_themes
            .iter()
            .find(|t| *t.meta.name.as_ref() == *current_theme_config)
        {
            self.active_theme = Arc::clone(first);
        } else if let Some(first) = self.available_themes.first() {
            self.active_theme = Arc::clone(first);
        }

        debug!(
            "ThemeManager cargado: {} temas disponibles",
            self.available_themes.len()
        );
        Ok(())
    }

    fn reload(&mut self) -> Result<(), String> {
        self.scan_themes();

        let current_name = self.active_theme.meta.name.clone();
        if let Some(theme) = self
            .available_themes
            .iter()
            .find(|t| t.meta.name == current_name)
        {
            self.active_theme = Arc::clone(theme);
        }

        Ok(())
    }

    fn reset_to_default(&mut self) -> Result<(), String> {
        let current_name = self.active_theme.meta.name.clone();

        let default_theme = match &*current_name {
            "Blaze Dark" => BlazeAppTheme::default_dark(),
            "Blaze Light" => BlazeAppTheme::default_light(),
            "VS Code Dark" => BlazeAppTheme::default_vscode_dark(),
            "VS Code Light" => BlazeAppTheme::default_vscode_light(),
            _ => BlazeAppTheme::default_dark(),
        };

        self.active_theme = Arc::new(default_theme);
        self.save()?;

        info!("Tema '{}' reseteado a valores por defecto", current_name);
        Ok(())
    }

    fn save_as_custom_theme(&mut self, new_name: &str) -> Result<(), String> {
        let mut custom_theme = (*self.active_theme).clone();
        custom_theme.meta.name = new_name.into();
        custom_theme.meta.autor = "CustomUser".into();
        custom_theme.meta.version = "1.0.0".into();

        let json = serde_json::to_string_pretty(&custom_theme).map_err(|e| e.to_string())?;

        let file_name = format!("{}.json", new_name.to_lowercase().replace(' ', "_"));
        let path = self.path.join(file_name);

        std::fs::write(&path, json).map_err(|e| e.to_string())?;

        self.reload()?;

        self.set_theme(new_name);

        info!("Tema personalizado '{}' guardado", new_name);
        Ok(())
    }
}

impl LinuxTheme {
    fn init_defaults() -> Self {
        let config_path = KnownDirsManager::get().app_config.clone();

        let theme_path: Arc<Path> = if config_path.exists() {
            config_path.join("themes").into()
        } else {
            KnownDirsManager::get()
                .home
                .clone()
                .join(".config")
                .join("blazepilotapp")
                .join("themes")
                .into()
        };

        match std::fs::create_dir_all(&theme_path) {
            Ok(_) => debug!("generado path de temas: {}", theme_path.display()),
            Err(e) => warn!("Ha ocurrido un error generando el directorio de temas: {e}."),
        }

        Self {
            path: theme_path,
            active_theme: Arc::new(BlazeAppTheme::default_dark()),
            available_themes: Vec::new(),
            available_themes_names: Vec::new(),
        }
    }

    fn write_missing_defaults(&self) -> Result<(), String> {
        let defaults = [
            BlazeAppTheme::default_dark(),
            BlazeAppTheme::default_light(),
            BlazeAppTheme::default_vscode_dark(),
            BlazeAppTheme::default_vscode_light(),
        ];

        for theme in &defaults {
            let file_name = format!("{}.json", theme.meta.name.to_lowercase().replace(' ', "_"));
            let path = self.path.join(file_name);

            if path.exists() {
                continue;
            }

            let json = serde_json::to_string_pretty(theme).map_err(|e| e.to_string())?;
            std::fs::write(&path, json).map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    #[allow(deprecated)]
    fn scan_themes(&mut self) {
        let Ok(entries) = std::fs::read_dir(&self.path) else {
            return;
        };

        let mut themes: Vec<Arc<BlazeAppTheme>> = vec![];
        let mut names: Vec<Box<str>> = vec![];

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };

            let theme = if let Ok(t) = serde_json::from_str::<BlazeAppTheme>(&content) {
                t
            } else if let Ok(old) = serde_json::from_str::<LegacyTheme>(&content) {
                let migrated = BlazeAppTheme::from(old);

                match serde_json::to_string_pretty(&migrated) {
                    Ok(json) => {
                        if let Err(e) = std::fs::write(&path, json) {
                            warn!(
                                "No se ha podido guardar el tema migrado {}: {}",
                                path.display(),
                                e
                            );
                        } else {
                            info!("Tema migrado a v2: {:?}", path);
                        }
                    }

                    Err(e) => {
                        warn!(
                            "No se ha podido serializar el tema migrado {}: {}",
                            path.display(),
                            e
                        );
                    }
                }

                migrated
            } else {
                warn!("JSON inválido, no se ha podido migrar: {}", path.display());
                continue;
            };

            names.push(theme.meta.name.clone());
            themes.push(Arc::new(theme));
        }

        self.available_themes = themes;
        self.available_themes_names = names;
    }
}
