use lru::LruCache;
use once_cell::sync::Lazy;
use std::{num::NonZeroUsize, sync::Arc};
use tokio::sync::RwLock;

use crate::core::system::fileopener_module::AppAssociation;

pub enum OpenStrategy {
    LaunchDirect,                      // ELF ejecutable, AppImage
    LaunchWithApp(AppAssociation),     // app conocida del assoc_manager o mimeapps
    ShowSelector(Vec<AppAssociation>), // varias opciones
    Fallback,                          // xdg-open fallback
}

pub struct DesktopEntry {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub mimes: Vec<String>,
    pub is_private: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OpenerFileKind {
    AppImage(u8),
    ElfExecutable,
    ShellScript,
    PythonScript,
    RubyScript,
    PerlScript,
    NodeScript,
    OtherScript,
    Png,
    Jpeg,
    Pdf,
    Zip,
    Unknown,
}

impl OpenerFileKind {
    pub fn mime(&self) -> &'static str {
        match self {
            OpenerFileKind::AppImage(_) => "application/x-executable",
            OpenerFileKind::ElfExecutable => "application/x-executable",
            OpenerFileKind::ShellScript => "application/x-shellscript",
            OpenerFileKind::PythonScript => "text/x-python",
            OpenerFileKind::RubyScript => "text/x-ruby",
            OpenerFileKind::PerlScript => "text/x-perl",
            OpenerFileKind::NodeScript => "application/javascript",
            OpenerFileKind::OtherScript => "text/x-script",
            OpenerFileKind::Png => "image/png",
            OpenerFileKind::Jpeg => "image/jpeg",
            OpenerFileKind::Pdf => "application/pdf",
            OpenerFileKind::Zip => "application/zip",
            OpenerFileKind::Unknown => "application/octet-stream",
        }
    }

    pub fn is_directly_executable(&self) -> bool {
        matches!(
            self,
            OpenerFileKind::AppImage(_) | OpenerFileKind::ElfExecutable
        )
    }
}

#[derive(Debug, Clone)]
pub enum AppsIconData {
    Rgba {
        data: Vec<u8>,
        width: f32,
        height: f32,
    },
    #[allow(unused)]
    Path(String),
    None,
}

pub static APPS_ICON_CACHE: Lazy<RwLock<LruCache<String, Arc<AppsIconData>>>> =
    Lazy::new(|| RwLock::new(LruCache::new(NonZeroUsize::new(80).unwrap())));
