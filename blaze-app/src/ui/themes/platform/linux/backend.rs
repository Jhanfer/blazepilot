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

use egui::Color32;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{
    core::{
        bootstrap::configs::config_manager::with_configs,
        system::knowndirs::knowndirs_manager::KnownDirsManager,
    },
    ui::themes::platform::{ColorsTrait, structs::Theme},
};

#[derive(Serialize, Deserialize)]
pub struct LinuxTheme {
    path: Arc<Path>,
    active_theme: Arc<Theme>,
    available_themes_names: Vec<Box<str>>,
    available_themes: Vec<Arc<Theme>>,
}

impl ColorsTrait for LinuxTheme {
    fn init() -> Self {
        Self::init_defaults()
    }

    fn update_theme(&mut self, mutator: fn(&mut Theme, Color32), value: Color32) {
        let theme_mutable = Arc::make_mut(&mut self.active_theme);
        mutator(theme_mutable, value);
    }

    fn set_theme(&mut self, name: &str) {
        if let Some(theme) = self
            .available_themes
            .iter()
            .find(|t| t.name.as_ref() == name)
        {
            self.active_theme = Arc::clone(theme);
        }
    }

    fn available_themes(&self) -> Vec<Box<str>> {
        self.available_themes_names.clone()
    }

    fn current_theme(&self) -> Arc<Theme> {
        self.active_theme.clone()
    }

    fn save(&mut self) -> Result<(), String> {
        let json =
            serde_json::to_string_pretty(self.active_theme.as_ref()).map_err(|e| e.to_string())?;

        let file_name = format!(
            "{}.json",
            self.active_theme.name.to_lowercase().replace(' ', "_")
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
            .find(|t| *t.name.as_ref() == *current_theme_config)
        {
            self.active_theme = Arc::clone(first);
        } else if let Some(first) = self.available_themes.first() {
            self.active_theme = Arc::clone(first);
        }

        info!(
            "ThemeManager cargado: {} temas disponibles",
            self.available_themes.len()
        );
        Ok(())
    }

    fn reload(&mut self) -> Result<(), String> {
        self.scan_themes();

        let current_name = self.active_theme.name.clone();
        if let Some(theme) = self
            .available_themes
            .iter()
            .find(|t| t.name == current_name)
        {
            self.active_theme = Arc::clone(theme);
        }

        Ok(())
    }

    fn reset_to_default(&mut self) -> Result<(), String> {
        let current_name = self.active_theme.name.clone();

        let default_theme = match &*current_name {
            "Blaze Dark" => Theme::blaze_dark(),
            "Blaze Light" => Theme::blaze_light(),
            "VS Code Dark" => Theme::vscode_dark(),
            "VS Code Light" => Theme::vscode_light(),
            _ => Theme::default(),
        };

        self.active_theme = Arc::new(default_theme);
        self.save()?;

        info!("Tema '{}' reseteado a valores por defecto", current_name);
        Ok(())
    }

    fn save_as_custom_theme(&mut self, new_name: &str) -> Result<(), String> {
        let mut custom_theme = (*self.active_theme).clone();
        custom_theme.name = new_name.into();
        custom_theme.autor = "CustomUser".into();
        custom_theme.version = "1.0.0".into();

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
            Ok(_) => info!("generado path de temas: {}", theme_path.display()),
            Err(e) => warn!("Ha ocurrido un error generando el directorio de temas: {e}."),
        }

        Self {
            path: theme_path,
            active_theme: Arc::new(Theme::default()),
            available_themes: Vec::new(),
            available_themes_names: Vec::new(),
        }
    }

    fn write_missing_defaults(&self) -> Result<(), String> {
        let defaults = [
            Theme::blaze_dark(),
            Theme::blaze_light(),
            Theme::vscode_dark(),
            Theme::vscode_light(),
        ];

        for theme in &defaults {
            let file_name = format!("{}.json", theme.name.to_lowercase().replace(' ', "_"));
            let path = self.path.join(file_name);

            if path.exists() {
                continue;
            }

            let json = serde_json::to_string_pretty(theme).map_err(|e| e.to_string())?;
            std::fs::write(&path, json).map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    fn scan_themes(&mut self) {
        let Ok(entries) = std::fs::read_dir(&self.path) else {
            return;
        };

        let mut themes: Vec<Arc<Theme>> = vec![];
        let mut names: Vec<Box<str>> = vec![];

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(theme) = serde_json::from_str::<Theme>(&content) else {
                warn!("JSON inválido: {:?}", path);
                continue;
            };
            names.push(theme.name.clone());
            themes.push(Arc::new(theme));
        }

        self.available_themes = themes;
        self.available_themes_names = names;
    }
}
