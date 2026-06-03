use std::path::Path;

use crate::core::bootstrap::install_manager::installation_manager::InstallResult;

pub trait InstallationTrait: Send + Sync {
    fn installation_path(&self) -> &Path;

    fn fallback_path(&self) -> &Path;

    #[must_use]
    fn install(&self) -> InstallResult;

    fn post_install(&self) -> Result<(), String>;
}
