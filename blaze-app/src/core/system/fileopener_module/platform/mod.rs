pub mod opener_trait;
pub use crate::core::system::fileopener_module::platform::opener_trait::FileOpener;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub use crate::core::system::fileopener_module::platform::linux::backend::LinuxOpener as PlatformOpener;

#[cfg(target_os = "macos")]
pub use crate::core::system::fileopener_module::platform::macos::backend::MacosOpener as PlatformOpener;

#[cfg(target_os = "windows")]
pub use crate::core::system::fileopener_module::platform::windows::backend::WindowsOpener as PlatformOpener;
