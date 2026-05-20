use std::path::Path;
use crate::core::bootstrap::configs::error::ConfigResult;

pub trait PlatformConfigTrait: Send + Sync {
    fn config_dir(&self) -> &Path;
    fn load(&mut self) -> ConfigResult<()>;
    fn save(&self) -> ConfigResult<()>;
}