use crate::core::files::file_extension::FileExtension;
use file_id::FileId;
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc};

//------------------------------------------------
#[derive(Debug, Clone)]
pub enum TaskType {
    #[allow(unused)]
    FileLoading,
    CopyPaste,
    #[allow(unused)]
    CutPaste,
    MoveTrash,
    #[allow(unused)]
    Delete,
    RestoreTrash,
}

#[derive(Debug, Clone)]
pub enum FileLoadingMessage {
    Batch(u64, Vec<Arc<FileEntry>>),
    Finished(u64),

    #[allow(unused)]
    ProgressUpdate {
        total: usize,
        done: usize,
        text: String,
    },

    FileAdded {
        name: String,
    },
    FileRemoved {
        name: String,
    },
    FileModified {
        name: String,
    },
    FullRefresh,

    RecursiveBatch {
        generation: u64,
        batch: Vec<Arc<FileEntry>>,
        #[allow(unused)]
        source_dir: Arc<Path>,
    },

    GitStatusChanged,
}

#[derive(Debug, Clone)]
pub enum RecursiveMessages {
    #[allow(unused)]
    Started { task_id: u64, text: String },
    #[allow(unused)]
    Progress {
        task_id: u64,
        files_found: usize,
        current_dir: Arc<Path>,
        text: String,
    },
    #[allow(unused)]
    Finished {
        task_id: u64,
        success: bool,
        text: String,
    },
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: Box<str>,
    pub full_path: Arc<Path>,
    pub extension: FileExtension,
    pub kind: FileKind,
    pub size: u64,
    pub modified: u64,
    pub is_hidden: bool,
    pub unique_id: Option<FileId>,

    #[allow(unused)]
    pub accessed: u64,
    #[allow(unused)]
    pub permissions: u32,
    #[allow(unused)]
    pub created: u64,
    #[allow(unused)]
    #[cfg(unix)]
    pub inode: u64,
    #[allow(unused)]
    #[cfg(unix)]
    pub nlink: u64,
    #[allow(unused)]
    #[cfg(unix)]
    pub device: u64,

    #[cfg(windows)]
    pub attributes: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileKind {
    #[default]
    File,
    Dir,
    Symlink,
}

impl Default for FileEntry {
    fn default() -> Self {
        Self {
            name: Default::default(),
            full_path: Arc::from(Path::new("")),
            extension: Default::default(),
            kind: Default::default(),
            size: Default::default(),
            modified: Default::default(),
            created: Default::default(),
            is_hidden: Default::default(),
            unique_id: Default::default(),
            accessed: Default::default(),
            permissions: Default::default(),
            inode: Default::default(),
            nlink: Default::default(),
            device: Default::default(),

            #[cfg(windows)]
            attributes: 0,
        }
    }
}

impl FileEntry {
    pub fn is_dir(&self) -> bool {
        matches!(self.kind, FileKind::Dir)
    }
}

//------------------------------------------------
