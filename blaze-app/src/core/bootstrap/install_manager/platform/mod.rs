pub mod installation_trait;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;


#[cfg(target_os = "linux")]
pub use crate::core::bootstrap::install_manager::platform::linux::linux::LinuxInstallation as PlatformInstallation;

#[cfg(target_os = "macos")]
use crate::core::bootstrap::configs::platform::macos::macos::MacosInstallation as PlatformInstallation;

#[cfg(target_os = "windows")]
use crate::core::bootstrap::configs::platform::windows::windows::WindowsInstallation as PlatformInstallation;