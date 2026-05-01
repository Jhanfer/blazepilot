use std::path::PathBuf;
use std::sync::Arc;
use file_id::FileId;
use uuid::Uuid;
use crate::core::files::blaze_motor::motor_structs::FileEntry;
use crate::core::system::fileopener_module::AppAssociation;
use crate::core::system::fileopener_module::platform::linux::linux::AppsIconData;
use crate::core::system::updater::updater::UpdateMessages;
use crate::ui::image_preview::image_preview::ImagePreviewState;





#[derive(Debug)]
pub enum FileOperation {
    Move { files: Vec<PathBuf>, dest: PathBuf, tab_id: Uuid},
    Delete { files: Vec<PathBuf> },
    Copy { files: Vec<PathBuf>, dest: PathBuf },
    Update,
    UpdateDirSize {
        full_path: PathBuf, 
        size: u64, 
        tab_id: Uuid,
    },
    RestoreDeletedFiles {
        file_names: Vec<String>,
    },
    ExtendedInfoReady {
        full_path: PathBuf,
        tab_id: Uuid,
    },

    ExtractHere {
        entry: Arc<FileEntry>, 
        dest_dir: PathBuf,
    },
}

#[derive(Debug)]
pub enum SureTo {
    SureToMove {
        files: Vec<PathBuf>,
        dest: PathBuf,
        tab_id: Uuid,
    },
    SureToDelete {
        files: Vec<PathBuf>,
        tab_id: Uuid,
    },
    SureToCopy,
}

#[derive(Debug)]
pub enum FileConflict {
    AlreadyExist {
        name: String,
        path: PathBuf
    }
}

pub enum UiEvent {
    OpenWithSelector {
        path: PathBuf,
        mime: String,
        apps: Vec<AppAssociation>,
        icon_data: Vec<AppsIconData>,
        show_all_apps: bool,
    },

    ThumbnailReady {
        full_path: PathBuf,
        tab_id: Uuid,
    },


    ShowImagePvw {
        pvw: Option<ImagePreviewState>,
    },

    SureTo(SureTo),

    UpdateMessages(UpdateMessages),

    FileConflict(FileConflict),

    ShowError(String),

    ShowFolderColorSelector {
        folder_id: FileId,
    },

    OpenConfigs,

    RefreshList,
}
