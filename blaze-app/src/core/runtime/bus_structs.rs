use crate::core::bootstrap::quick_access_manager::platform::structs::QuickLinks;
use crate::core::files::blaze_motor::motor_structs::FileEntry;
use crate::core::system::fileopener_module::platform::linux::structs::AppsIconData;
use crate::core::system::fileopener_module::AppAssociation;
use crate::core::system::updater::updater_manager::UpdateMessages;
use crate::ui::image_preview::image_preview_handler::ImagePreviewState;
use egui::Color32;
use file_id::FileId;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub enum FileOperation {
    Move {
        sources: Vec<Arc<Path>>,
        dest: Arc<Path>,
    },

    PasteCut {
        sources: Vec<Arc<Path>>,
        final_targets: Vec<Arc<Path>>,
    },

    PasteCopy {
        final_targets: Vec<Arc<Path>>,
    },

    Rename {
        original_path: Arc<Path>,
        new_path: Arc<Path>,
    },

    Trash {
        files: Vec<Arc<Path>>,
    },

    CreateDir {
        path: Arc<Path>,
    },

    CreateFile {
        path: Arc<Path>,
    },

    Update,
    UpdateDirSize {
        full_path: Arc<Path>,
        size: u64,
        tab_id: Uuid,
    },
    RestoreDeletedFiles {
        file_names: Vec<String>,
    },
    ExtendedInfoReady {
        full_path: Arc<Path>,
    },

    ExtractHere {
        entry: Arc<FileEntry>,
        dest_dir: Arc<Path>,
    },

    NavigateTo(Arc<Path>),
    OpenFileByPath(Arc<Path>),
}

#[derive(Debug)]
pub enum SureTo {
    SureToMove {
        files: Vec<Arc<Path>>,
        dest: Arc<Path>,
    },
    SureToDelete {
        files: Vec<Arc<Path>>,
        tab_id: Uuid,
    },
}

#[derive(Debug)]
pub enum FileConflict {
    AlreadyExist { name: String, path: Arc<Path> },
}

pub enum UiEvent {
    OpenWithSelector {
        path: Arc<Path>,
        mime: String,
        apps: Vec<AppAssociation>,
        icon_data: Vec<AppsIconData>,
        show_all_apps: bool,
    },

    ThumbnailReady {
        full_path: Arc<Path>,
    },

    ShowImagePvw {
        pvw: Option<ImagePreviewState>,
    },

    SureTo(SureTo),

    UpdateMessages(UpdateMessages),

    FileConflict(FileConflict),

    ShowError(Box<str>),

    ShowGeneric {
        title: Box<str>,
        message: Box<str>,
    },

    ShowFolderColorSelector {
        folder_id: FileId,
    },

    ShowWantToInstall,

    OpenConfigs,

    QuickTagEvent(QuickTagEvent),
}

pub enum QuickTagEvent {
    CreateNewTag {
        title: String,
        temp_color: Color32,
    },
    AddQuickLinkToTag {
        quicks: Vec<QuickLinks>,
    },
    EditCurrentTag {
        id: Uuid,
        title: String,
        temp_color: Color32,
    },
    DeleteCurrentTag {
        title: Box<str>,
        id: Uuid,
    },

    DeleteQuickLink {
        tag_id: Uuid,
        quick_title: Box<str>,
        quick_id: Uuid,
    },

    EditCurrentQuickLink {
        tag_id: Uuid,
        quick_id: Uuid,
        title: String,
        temp_color: Color32,
    },
}
