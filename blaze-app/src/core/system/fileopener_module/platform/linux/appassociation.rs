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
    bootstrap::configs::config_manager::with_configs,
    system::fileopener_module::{
        error::{OpenerError, OpenerResult},
        fileopener_manager::AppAssociation,
    },
};
use std::{collections::HashMap, fs, path::PathBuf};
use tracing::warn;

pub type UserAssociations = HashMap<String, AppAssociation>;

pub struct AssociationManager {
    associations: UserAssociations,
    config_path: PathBuf,
}

impl AssociationManager {
    pub fn new() -> Self {
        let mut config_path = with_configs(|c| c.config_dir().to_owned());
        config_path.push("BlazePilot");
        config_path.push("associations.json");

        let associations = if config_path.exists() {
            fs::read_to_string(&config_path)
                .ok()
                .and_then(|assoc_str| serde_json::from_str(&assoc_str).ok())
                .unwrap_or_else(UserAssociations::new)
        } else {
            UserAssociations::new()
        };

        Self {
            associations,
            config_path,
        }
    }

    pub fn get_associations(&self, mime: &str) -> Option<&AppAssociation> {
        self.associations.get(mime)
    }

    pub fn set_association(&mut self, mime: &str, app: AppAssociation) {
        self.associations.insert(mime.to_string(), app);
        if let Err(e) = self.save() {
            warn!("Error guardando la asociación de programas: {}", e)
        }
    }

    fn save(&self) -> OpenerResult<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent).map_err(|e| OpenerError::Io {
                path: parent.into(),
                source: e,
            })?;
        }

        let json =
            serde_json::to_string_pretty(&self.associations).map_err(OpenerError::SerdeError)?;

        fs::write(&self.config_path, json).map_err(|e| OpenerError::Io {
            path: self.config_path.to_owned().into(),
            source: e,
        })?;

        Ok(())
    }
}
