pub mod colors_trait;
pub mod structs;

pub use crate::ui::themes::platform::colors_trait::ColorsTrait;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub use crate::ui::themes::platform::linux::backend::LinuxTheme as PlatformTheme;

#[cfg(target_os = "macos")]
pub use crate::ui::themes::platform::macos::backend::MacosTheme as PlatformTheme;

#[cfg(target_os = "windows")]
pub use crate::ui::themes::platform::windows::backend::WindowsTheme as PlatformTheme;
