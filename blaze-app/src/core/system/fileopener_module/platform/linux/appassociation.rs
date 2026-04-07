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





use crate::core::{configs::config_state::with_configs, system::fileopener_module::fileopener_manager::AppAssociation};
use std::{fs, path::PathBuf, collections::HashMap};

pub type UserAssociations = HashMap<String, AppAssociation>;


pub struct AssociationManager {
    associations: UserAssociations,
    config_path: PathBuf,
}

impl AssociationManager {
    pub fn new() -> Self {
        let mut config_path = with_configs(|c| c.configs.get_config_folder());
        config_path.push("BlazePilot");
        config_path.push("associations.json");

        let associations = if config_path.exists() {
            fs::read_to_string(&config_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            UserAssociations::new()
        };

        Self {
            associations,
            config_path,
        }
    }


    pub fn get_associations(&mut self, mime: &str) -> Option<&AppAssociation> {
        self.associations.get(mime)
    }

    pub fn set_association(&mut self, mime: &str, app: AppAssociation) {
        self.associations.insert(mime.to_string(), app);
        self.save();
    }

    fn save(&self) {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).ok();
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.associations) {
            fs::write(&self.config_path, json).ok();
        }
    }
}

