pub mod config_trait;
pub use crate::core::bootstrap::configs::platform::config_trait::PlatformConfigTrait;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub use crate::core::bootstrap::configs::platform::linux::linux::LinuxConfigs as PlatformConfigs;

#[cfg(target_os = "macos")]
use crate::core::bootstrap::configs::platform::macos::macos::MacosConfigs as PlatformConfigs;

#[cfg(target_os = "windows")]
use crate::core::bootstrap::configs::platform::windows::windows::WindowsConfigs as PlatformConfigs;
