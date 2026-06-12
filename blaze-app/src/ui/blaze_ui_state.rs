// Copyright 2026 Jhanfer
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use egui::TextureHandle;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::Arc,
};
use tracing::{debug, info};

use crate::{
    core::{
        files::blaze_motor::motor::with_motor,
        runtime::{
            bus_structs::{FileConflict, SureTo, UiEvent},
            event_bus::with_event_bus,
        },
        system::{
            cache::color_cache::color_cache_logic::FolderColorManager,
            updater::updater_manager::UpdateMessages,
        },
    },
    ui::{
        dialog_manager::manager::DialogManager,
        icons_cache::{icon_cache::IconCache, thumbnails::thumbnails_manager::ThumbnailManager},
        modules::custom_context_menu::context_state::ContextMenuState,
    },
};

pub struct BlazeUiState {
    pub dialog_manager: DialogManager,
    pub icon_cache: IconCache,
    pub folder_color_manager: FolderColorManager,
    pub context_menu_state: ContextMenuState,
    pub thumb_texture_cache: HashMap<Arc<Path>, TextureHandle>,
    pub thumbnail_manager: ThumbnailManager,
    pub calculating_thumbnails: HashSet<Arc<Path>>,
    pub calculated_thumbnails: HashSet<Arc<Path>>,
    pub needs_repaint: bool,
    pub newly_calculated_thumbnails: HashSet<Arc<Path>>,
    last_thumb_cache_dir: Option<Arc<Path>>,
}

impl Default for BlazeUiState {
    fn default() -> Self {
        Self::new()
    }
}

impl BlazeUiState {
    fn new() -> Self {
        let dialog_manager = DialogManager::new();
        Self {
            dialog_manager,
            icon_cache: IconCache::new(),
            folder_color_manager: FolderColorManager::new(),
            context_menu_state: ContextMenuState::new(),
            thumb_texture_cache: HashMap::new(),
            thumbnail_manager: ThumbnailManager::new(),
            calculating_thumbnails: HashSet::new(),
            calculated_thumbnails: HashSet::new(),
            needs_repaint: false,
            newly_calculated_thumbnails: HashSet::new(),
            last_thumb_cache_dir: None,
        }
    }

    pub fn evict_thumbnail_cache_if_dir_changed(&mut self, cwd: &Path) {
        let changed = self.last_thumb_cache_dir.as_deref() != Some(cwd);

        if changed {
            self.thumb_texture_cache.clear();
            self.calculating_thumbnails.clear();
            self.calculated_thumbnails.clear();
            self.last_thumb_cache_dir = Some(cwd.into());
        }
    }

    pub fn enforce_texture_cache_limit(&mut self, max_entries: usize) {
        if self.thumb_texture_cache.len() > max_entries {
            let to_remove = self.thumb_texture_cache.len() - max_entries;
            let keys: Vec<Arc<Path>> = self
                .thumb_texture_cache
                .keys()
                .take(to_remove)
                .cloned()
                .collect();
            for key in keys {
                self.thumb_texture_cache.remove(&key);
            }
        }
    }

    pub fn process_events(&mut self) {
        let active_id = with_motor(|m| m.active_tab().id);
        let dispatcher = with_event_bus(|e| e.dispatcher(active_id));

        let events: Vec<UiEvent> = with_event_bus(|pool| {
            let mut msgs = Vec::new();
            pool.drain(active_id, |msg| {
                msgs.push(msg);
                true
            });
            msgs
        });

        self.thumbnail_manager
            .process_messages(active_id, dispatcher.clone());

        for envent in events {
            match envent {
                UiEvent::OpenWithSelector { path } => {
                    self.dialog_manager.open_selector_dialog(path);
                }
                UiEvent::ShowError(message) => {
                    info!("Error recibido");
                    self.dialog_manager.open_error_dialog(&message);
                }
                UiEvent::SureTo(sureto) => match sureto {
                    SureTo::SureToMove { files, dest } => {
                        debug!("Mover {:?} → {:?}", files, dest);
                        self.dialog_manager.open_sure_move_dialog(files, dest);
                    }
                    SureTo::SureToDelete { files, tab_id } => {
                        self.dialog_manager.open_sure_to_delete(files, tab_id);
                    }
                },
                UiEvent::UpdateMessages(update_message) => match update_message {
                    UpdateMessages::NewVersionAvailable {
                        current_version,
                        new_version,
                        tab_id,
                    } => {
                        info!("NUEVA VERSIÓN {:?}", new_version);
                        self.dialog_manager.open_updater_dialog(
                            current_version,
                            new_version,
                            tab_id,
                        );
                    }
                    UpdateMessages::UpToDate => {
                        info!("Estás en la última versión.");
                    }
                },

                UiEvent::FileConflict(file_conflict) => match file_conflict {
                    FileConflict::AlreadyExist { name, path } => {
                        info!("Ya existe {} en {:?}", name, path);
                    }
                },

                UiEvent::ShowFolderColorSelector { folder_id } => {
                    self.dialog_manager
                        .open_folder_color_selector_dialog(folder_id);
                }

                UiEvent::OpenConfigs => {
                    self.dialog_manager.open_configs();
                }

                UiEvent::ThumbnailReady { full_path } => {
                    self.calculating_thumbnails.remove(&full_path);
                    self.calculated_thumbnails.insert(full_path.clone());
                    self.newly_calculated_thumbnails.insert(full_path);
                    self.needs_repaint = true;
                }

                UiEvent::ShowImagePvw { pvw } => {
                    if let Some(img_pvw) = pvw {
                        self.dialog_manager.open_img_pvw_dialog(img_pvw);
                    }
                }

                UiEvent::ShowWantToInstall => {
                    self.dialog_manager.open_want_to_install_dialog();
                }

                UiEvent::ShowGeneric { title, message } => {
                    self.dialog_manager.open_show_generic(&title, &message);
                }

                UiEvent::QuickTagEvent(event) => {
                    self.dialog_manager.open_quick_acc_dialog(event);
                }
            }
        }
    }
}
