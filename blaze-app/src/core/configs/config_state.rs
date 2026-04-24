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






use serde::{Serialize, Deserialize};
use directories::ProjectDirs;
use std::{collections::HashSet, path::PathBuf};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use tracing::debug;

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub enum OrderingMode {
    #[default]
    Az,
    Za,
    SizeAsc,
    SizeDesc,
    DateAsc,
    DateDesc,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct FavoriteLinks {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}


#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub enum DisplayBackend {
    #[default]
    Auto,
    X11,
    Wayland
}

impl DisplayBackend {
    pub fn name(&self) -> &'static str {
        match self {
            DisplayBackend::Auto => "Auto",
            DisplayBackend::X11 => "X11",
            DisplayBackend::Wayland => "Wayland",
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigsFlags {
    #[serde(default)]
    pub app_ordering_mode: OrderingMode,
    #[serde(default)]
    config_file_path: PathBuf,
    
    #[serde(default)]
    pub show_hidden_files: bool,

    #[serde(default)]
    pub item_file_list_size: usize,

    #[serde(default)]
    pub favorite_list: HashSet<FavoriteLinks>,

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

}   

impl ConfigsFlags {
    pub fn default() -> Self {
        let path = ConfigsFlags::init_project_folder().unwrap();
        debug!("Path config: {:?}", path);

        let mut cfg = Self {
            app_ordering_mode: OrderingMode::Az,
            config_file_path: path,
            show_hidden_files: false,
            item_file_list_size: 10,
            favorite_list: HashSet::new(),
            display_backend: DisplayBackend::Auto,
            default_terminal: String::new(),
            theme: "system".to_string(),
            accent_color: None,
            font_size: 14.0,
            confirm_on_delete: true,
        };
        cfg.save().unwrap();
        cfg
    }

    pub fn get_config_folder(&self) -> PathBuf {
        self.config_file_path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default()
    }

    fn init_project_folder() -> Result<PathBuf, String> {
        let proj = ProjectDirs::from("com", "blazepilot", "blazepilotapp").expect("No se pudo obtener ProjectDirs.");

        let mut path = proj.config_dir().to_path_buf();
        
        std::fs::create_dir_all(&path)
            .map_err(|e| format!("Error al crear la carpeta de configuración: {}.", e))?;

        path.push("config.json");

        if !path.exists() {
            std::fs::File::create_new(&path)
                .map_err(|e| format!("Error al crear el archivo de configuración / archivo existente: {}.", e))?;
        }

        Ok(path)
    }

    pub fn set_ordering_mode(&mut self, mode: OrderingMode) -> &mut Self {
        self.app_ordering_mode = mode;
        self.save().ok();
        self
    }


    pub fn set_show_hidden_files(&mut self, mode: bool) -> &mut Self {
        self.show_hidden_files = mode;
        self.save().ok();
        self
    }

    pub fn set_item_file_list_size(&mut self, size: usize) -> &mut Self {
        self.item_file_list_size = size;
        self.save().ok();
        self
    }

    pub fn set_display_backend(&mut self, backend: DisplayBackend) -> &mut Self {
        self.display_backend = backend;
        self.save().ok();
        self
    }

    pub fn set_default_terminal(&mut self, terminal: String) -> &mut Self {
        self.default_terminal = terminal;
        self.save().ok();
        self
    }

    
    pub fn add_to_favorites(&mut self, name: String,  path: PathBuf, is_dir: bool) -> &mut Self {
        if !self.is_in_favorite(&path) {
            self.favorite_list.insert(FavoriteLinks {name, path, is_dir});
        }
        self.save().ok();
        self
    }

    pub fn delete_from_favorites(&mut self, name: String,  path: PathBuf) -> &mut Self {
        self.favorite_list.retain(|f| !(f.name == name && f.path == path));
        self.save().ok();
        self
    }

    pub fn is_in_favorite(&self, path: &PathBuf) -> bool {
        self.favorite_list
            .iter()
            .any(|f| f.path == *path)
    }


    fn save(&mut self) -> Result<(), String> {
        let path = &self.config_file_path;
        let data = serde_json::to_string_pretty(self).expect("Error al serializar.");
        std::fs::write(path, data).expect("No se pudo guardar las configs.");
        Ok(())
    }
}


pub struct ConfigHandler {
    pub configs: ConfigsFlags,
}


static CONFIGS: Lazy<Mutex<ConfigHandler>> = Lazy::new(|| {
    Mutex::new(ConfigHandler::init())
});

pub fn with_configs<R>(f:impl FnOnce(&mut ConfigHandler) -> R ) -> R {
    match CONFIGS.lock() {
        Ok(mut guard) => f(&mut *guard),
        Err(poisoned) => {
            let mut guard = poisoned.into_inner();
            f(&mut *guard)
        }
    }
}


impl ConfigHandler {
    pub fn init() -> Self {
        let path = ConfigsFlags::init_project_folder().unwrap_or_else(|_|{
            PathBuf::new()
        });
        Self { configs: ConfigsFlags {
                app_ordering_mode: OrderingMode::Az,
                config_file_path: path,
                show_hidden_files: false,
                item_file_list_size: 10,
                favorite_list: HashSet::new(),
                display_backend: DisplayBackend::Auto,
                default_terminal: String::new(),
                theme: "system".to_string(),
                accent_color: None,
                font_size: 14.0,
                confirm_on_delete: true,
            } 
        }
    }

    pub fn set_ordering_mode(&mut self, mode: OrderingMode) {
        self.configs.set_ordering_mode(mode);
    }

    pub fn set_show_hidden_files(&mut self, mode: bool) {
        self.configs.set_show_hidden_files(mode);
    }

    pub fn set_item_file_list_size(&mut self, size: usize) {
        self.configs.set_item_file_list_size(size);
    }

    pub fn add_to_favorites(&mut self, name: String,  path: PathBuf, is_dir: bool) {
        self.configs.add_to_favorites(name, path, is_dir);
    }

    pub fn delete_from_favorites(&mut self, name: String,  path: PathBuf) {
        self.configs.delete_from_favorites(name, path);
    }

    pub fn is_in_favorite(&mut self, path: &PathBuf) -> bool {
        self.configs.is_in_favorite(path)
    }

    pub fn set_display_backend(&mut self, backend: DisplayBackend) {
        self.configs.set_display_backend(backend);
    }

    pub fn set_default_terminal(&mut self, terminal: String) {
        self.configs.set_default_terminal(terminal);
    }
    

    pub fn load_or_init_cofigs(&mut self) -> Result<(), String> {
        let path = self.configs.config_file_path.clone();

        debug!("{:?}", path.clone());

        if path.exists() {
            let data = std::fs::read_to_string(&path)
                .expect("No se pudo leer las configs.");
            
            match serde_json::from_str::<ConfigsFlags>(&data) {
                Ok(cfg) => {
                    self.configs = cfg;
                },
                Err(_) => {
                    self.configs = ConfigsFlags::default();
                }
            }

        } else {
            self.configs.save().unwrap();
        }



        Ok(())
    }

    
}