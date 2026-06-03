mod quick_acc_trait;
pub mod structs;

pub use crate::core::bootstrap::quick_access_manager::platform::structs::QuickTag;

pub use crate::core::bootstrap::quick_access_manager::platform::quick_acc_trait::QuickAccessTrait;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub use crate::core::bootstrap::quick_access_manager::platform::linux::linux::QuickAccessLinux as PlatformQuickAccess;

#[cfg(target_os = "macos")]
pub use crate::core::bootstrap::quick_access_manager::platform::macos::macos::QuickAccessMacos as PlatformQuickAccess;

#[cfg(target_os = "windows")]
pub use crate::core::bootstrap::quick_access_manager::platform::windows::windows::QuickAccessWindows as PlatformQuickAccess;
