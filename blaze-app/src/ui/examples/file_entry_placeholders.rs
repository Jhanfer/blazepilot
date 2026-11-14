use crate::core::files::{
    blaze_motor::motor_structs::{FileEntry, FileKind},
    file_extension::FileExtension,
};
use std::{path::Path, sync::Arc};

impl FileEntry {
    /// Crea un placeholder genérico para un directorio (Carpeta)
    pub fn placeholder_dir(name: impl Into<Box<str>>, path: impl Into<Arc<Path>>) -> Self {
        Self {
            name: name.into(),
            full_path: path.into(),
            extension: FileExtension::default(),
            kind: FileKind::Dir,
            size: 0,
            modified: 0,
            created: 0,
            is_hidden: false,
            unique_id: None,
            accessed: 0,
            permissions: 0o755,
            #[cfg(unix)]
            inode: 0,
            #[cfg(unix)]
            nlink: 1,
            #[cfg(unix)]
            device: 0,
            #[cfg(windows)]
            attributes: 0x10, // FILE_ATTRIBUTE_DIRECTORY
        }
    }

    /// Crea un placeholder para un archivo sin extensión
    pub fn placeholder_file(
        name: impl Into<Box<str>>,
        path: impl Into<Arc<Path>>,
        extension: FileExtension,
    ) -> Self {
        Self {
            name: name.into(),
            full_path: path.into(),
            extension,
            kind: FileKind::File,
            size: 1024,
            modified: 0,
            created: 0,
            is_hidden: false,
            unique_id: None,
            accessed: 0,
            permissions: 0o644,
            #[cfg(unix)]
            inode: 0,
            #[cfg(unix)]
            nlink: 1,
            #[cfg(unix)]
            device: 0,
            #[cfg(windows)]
            attributes: 0x20, // FILE_ATTRIBUTE_ARCHIVE
        }
    }
}
