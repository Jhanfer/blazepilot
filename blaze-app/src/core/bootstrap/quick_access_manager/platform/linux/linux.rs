use std::path::PathBuf;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::core::bootstrap::quick_access_manager::{
    error::{
        QuickAccError, QuickAccResult
    }, 
    platform::{QuickAccessTrait, QuickTag}
};

#[derive(Serialize, Deserialize, Debug)]
pub struct QuickAccessLinux {
    #[serde(default)]
    pub tags: Vec<QuickTag>,

    #[serde(default)]
    file_path: PathBuf,
}

impl QuickAccessLinux {
    fn init_config_path() -> QuickAccResult<PathBuf> {
        let proj = ProjectDirs::from("com", "blazepilot", "blazepilotapp")
            .ok_or(QuickAccError::ProjectDirsNotFound)?;

        let dir = proj.config_dir();
        std::fs::create_dir_all(dir).map_err(|e|QuickAccError::Io(e))?;

        let path = dir.join("quick_access.json");
        if !path.exists() {
            std::fs::File::create(&path).map_err(QuickAccError::Io)?;
        }

        Ok(path)
    }
}

impl Default for QuickAccessLinux {
    fn default() -> Self {
        let path = Self::init_config_path().unwrap_or_default();
        Self { 
            tags: Vec::new(),
            file_path: path,
        }
    }
}


impl QuickAccessTrait for QuickAccessLinux {
    fn load(&mut self) -> QuickAccResult<()> {
        let path = self.file_path.clone();

        if !path.exists() {
            return self.save();
        }

        let data = std::fs::read_to_string(&path)
            .map_err(|e| QuickAccError::Io(e))?;

        if data.trim().is_empty() {
            return self.save();
        }

        let mut loaded: QuickAccessLinux = serde_json::from_str(&data)
            .map_err(|_| QuickAccError::Deserialize)?;

        loaded.file_path = path;
        *self = loaded;

        Ok(())
    }

    fn save(&mut self) -> QuickAccResult<()> {
        let data = serde_json::to_string_pretty(&self)
            .map_err(|_| QuickAccError::Serialize)?;

        std::fs::write(&self.file_path.clone(), data)
            .map_err(|e|QuickAccError::Io(e))?;

        Ok(())
    }
}