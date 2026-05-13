use std::fs::Metadata;
use std::path::Path;
use std::time::SystemTime;
use file_id::FileId;
use crate::core::files::blaze_motor::motor_structs::{FileEntry, FileKind};
use crate::core::files::file_extension::FileExtension;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;


pub fn build_entry(path: &Path, m: Metadata, unique_id: Option<FileId>) -> FileEntry {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into())
        .unwrap_or_else(|| "".to_string().into_boxed_str());

    let is_dir = m.is_dir();

    let kind = if m.file_type().is_symlink() {
        FileKind::Symlink
    } else if is_dir {
        FileKind::Dir
    } else {
        FileKind::File
    };

    let ts = |t: Result<SystemTime, _>| {
        t.unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    };

    #[cfg(unix)]
    let (inode, nlink, device, permissions) = (
        m.ino(),
        m.nlink(),
        m.dev(),
        m.mode(),
    );

    #[cfg(windows)]
    let attributes = m.file_attributes();

    let extension = FileExtension::from_path(&path);

    FileEntry {
        name: name.clone(),
        extension,
        kind,
        size: m.len(),
        modified: ts(m.modified()),
        created: ts(m.created()),
        accessed: ts(m.accessed()),
        is_hidden: name.starts_with("."),
        full_path: path.into(),
        unique_id,
        permissions,
        inode,
        nlink,
        device,
        #[cfg(windows)]
        attributes,
    }
}