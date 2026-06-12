use crate::core::bootstrap::configs::error::ConfigResult;
use std::path::Path;

pub trait PlatformConfigTrait: Send + Sync {
    #[allow(unused)]
    fn config_dir(&self) -> &Path;
    fn load(&mut self) -> ConfigResult<()>;
    fn save(&self) -> ConfigResult<()>;
}
