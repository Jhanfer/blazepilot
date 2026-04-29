use std::{path::PathBuf, sync::Arc};
use file_id::FileId;
use crate::core::files::file_extension::FileExtension;




//------------------------------------------------
#[derive(Debug, Clone)]
pub enum TaskType {
    FileLoading,
    CopyPaste,
    CutPaste,
    MoveTrash,
    Delete,
    RestoreTrash
}

#[derive(Debug, Clone)]
pub enum FileLoadingMessage {
    Batch(u64, Vec<Arc<FileEntry>>),
    Finished(u64),
    ProgressUpdate {
        total: usize,
        done: usize,
        text: String,
    },

    FileAdded { name: String },
    FileRemoved { name: String },
    FileModified { name: String },
    FullRefresh,

    RecursiveBatch {
        generation: u64,
        batch: Vec<Arc<FileEntry>>,
        source_dir: PathBuf,
    }
}


#[derive(Debug, Clone)]
pub enum RecursiveMessages {
    Started {
        task_id: u64,
        text: String,
    },
    Progress {
        task_id: u64,       
        files_found: usize,  
        current_dir: PathBuf, 
        text: String, 
    },
    Finished {
        task_id: u64,
        success: bool,
        text: String,
    }
}


#[derive(Debug, Default, Clone)]
pub struct FileEntry {
    pub name: Box<str>,
    pub extension: FileExtension,
    pub kind: FileKind,
    pub size: u64,
    pub modified: u64,
    pub created: u64,
    pub is_hidden: bool,
    pub is_dir: bool,
    pub full_path: PathBuf,
    pub unique_id: Option<FileId>,

    pub accessed: u64,
    pub permissions: u32,

    #[cfg(unix)]
    pub inode: u64,
    #[cfg(unix)]
    pub nlink: u64,
    #[cfg(unix)]
    pub device: u64,

    #[cfg(windows)]
    pub attributes: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum FileKind {
    #[default]
    File,
    Dir,
    Symlink
}


//------------------------------------------------