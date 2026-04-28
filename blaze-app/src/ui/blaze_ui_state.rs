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




use std::{collections::{HashMap, HashSet}, path::PathBuf};
use egui::{Area, Order, Sense, TextureHandle, Ui};
use file_id::FileId;
use uuid::Uuid;
use crate::{core::{files::motor::with_motor, system::{cache::color_cache::color_cache::FolderColorManager, fileopener_module::{AppAssociation, platform::linux::linux::AppsIconData}, updater::updater::UpdateMessages}}, ui::{dialogs::{configs_dialog::ConfigDialog, error_dialog::ErrorDialog, folder_color_selector_dialog::FolderColorSelector, image_preview_dialog::ImagePreviewDialog, selector_dialog::AppSelectorDialog, sure_to_delete::SureToDeleteDialog, sure_to_move_to::SureToMoveToDialog, update_dialog::UpdateDialog}, icons_cache::{icon_cache::IconCache, thumbnails::thumbnails_manager::ThumbnailManager}, image_preview::image_preview::ImagePreviewState, modules::custom_context_menu::context_state::ContextMenuState}, utils::channel_pool::{FileConflict, NotifyingSender, SureTo, UiEvent, with_active_sender_for, with_channel_pool}};
use tracing::{debug, info};


pub trait ModalDialog {
    fn is_open(&self) -> bool;
    fn close(&mut self);
    fn render(&mut self, ui: &mut Ui);
}


pub struct DialogManager {
    pub selector_dialog: AppSelectorDialog,
    pub sure_to_dialog: SureToMoveToDialog,
    pub update_dialog: UpdateDialog,
    pub error_dialog: ErrorDialog,
    pub sure_to_delete_dialog: SureToDeleteDialog,
    pub folder_color_dialog: FolderColorSelector,
    pub config_dialog: ConfigDialog,
    pub img_pvw_dialog: ImagePreviewDialog,
}

impl DialogManager {
    pub fn new() -> Self {
        Self {
            selector_dialog: AppSelectorDialog::new(),
            sure_to_dialog: SureToMoveToDialog::new(),
            update_dialog: UpdateDialog::new(),
            error_dialog: ErrorDialog::new(),
            sure_to_delete_dialog: SureToDeleteDialog::new(),
            folder_color_dialog: FolderColorSelector::new(),
            config_dialog: ConfigDialog::new(),
            img_pvw_dialog: ImagePreviewDialog::new(),
        }
    }

    pub fn open_selector_dialog(&mut self, path: PathBuf, mime: String, apps: Vec<AppAssociation>, icon_data: Vec<AppsIconData>, show_all_apps: bool) {
        self.selector_dialog.open(path, mime, apps, icon_data, show_all_apps);
    }

    pub fn open_sure_move_dialog(&mut self, sources: Vec<PathBuf>, dest: PathBuf, tab_id: Uuid) {
        self.sure_to_dialog.open(sources, dest, tab_id);
    }

    pub fn open_sure_to_delete(&mut self, sources: Vec<PathBuf>, tab_id: Uuid) {
        self.sure_to_delete_dialog.open(sources, tab_id);
    }

    pub fn open_updater_dialog(&mut self, current_version: String, new_version: String, tab_id: Uuid) {
        self.update_dialog.open(current_version, new_version, tab_id);
    }

    pub fn open_error_dialog(&mut self, message: String) {
        self.error_dialog.open(message);
    }

    pub fn open_folder_color_selector_dialog(&mut self, folder_id: FileId) {
        self.folder_color_dialog.open(folder_id);
    }

    pub fn open_configs(&mut self) {
        self.config_dialog.open();
    }

    pub fn open_img_pvw_dialog(&mut self, imp_pvw: ImagePreviewState) {
        self.img_pvw_dialog.open(imp_pvw);
    }

    pub fn render_area(&mut self, ui: &mut Ui) {
        let dialogs: Vec<&mut dyn ModalDialog> = vec![
            &mut self.selector_dialog,
            &mut self.sure_to_dialog,
            &mut self.update_dialog,
            &mut self.error_dialog,
            &mut self.sure_to_delete_dialog,
            &mut self.folder_color_dialog,
            &mut self.config_dialog,
            &mut self.img_pvw_dialog,
        ];

        let open_dialog = dialogs.into_iter().find(|d| d.is_open());

        if let Some(dialog) = open_dialog {
            let mut should_close = false;

            Area::new("blocker".into())
                .fixed_pos(egui::pos2(0.0, 0.0))
                .order(Order::Middle)
                .sense(Sense::click())
                .interactable(true)
                .show(ui, |ui|{
                    let screen_rect = ui.ctx().content_rect();
                    ui.painter().rect_filled(
                        screen_rect,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180),
                    );

                    if ui.allocate_rect(screen_rect, egui::Sense::click_and_drag()).clicked() {
                        should_close = true;
                    }
                });
            
            if should_close {
                dialog.close();
            }

            dialog.render(ui);
        }
    }

}




pub struct BlazeUiState {
    pub dialog_manager: DialogManager,
    pub icon_cache: IconCache,
    pub folder_color_manager: FolderColorManager,
    pub context_menu_state: ContextMenuState,
    pub thumb_texture_cache: HashMap<PathBuf, TextureHandle>,
    pub thumbnail_manager: ThumbnailManager,
    pub calculating_thumbnails: HashSet<PathBuf>,
    pub calculated_thumbnails: HashSet<PathBuf>,
    pub cached_sender: Option<NotifyingSender>,
    cached_sender_tab_id: Option<Uuid>,
}


impl BlazeUiState {
    pub fn new() -> Self {
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
            cached_sender: None,
            cached_sender_tab_id: None,
        }
    }

    pub fn sender(&mut self) -> Option<&NotifyingSender> {
        let active_tab_id = with_motor(|m| m.active_tab().id);

        if self.cached_sender_tab_id != Some(active_tab_id) {
            self.cached_sender = with_active_sender_for(active_tab_id, |s| s.clone());
            self.cached_sender_tab_id = Some(active_tab_id);
        }
        self.cached_sender.as_ref()
    }
    
    pub fn invalidate_sender(&mut self) {
        self.cached_sender = None;
    }

    pub fn process_events(&mut self) {
        let sender = {
            let Some(sender) = self.sender() else {return;};
            sender.clone()
        };
        let active_id = sender.tab_id;

        let events: Vec<UiEvent> = with_channel_pool(|pool| {
            let  mut msgs = Vec::new();
            pool.process_ui_events(active_id, |msg|{
                msgs.push(msg);
                true
            });
            msgs
        });


        self.thumbnail_manager.process_messages(active_id, sender.clone());


        for envent in events {
            match envent {
                UiEvent::OpenWithSelector { path, mime, apps, icon_data, show_all_apps} => {
                    self.dialog_manager.open_selector_dialog(path, mime, apps, icon_data, show_all_apps);
                },
                UiEvent::ShowError(message) => {
                    info!("Error recibido");
                    self.dialog_manager.open_error_dialog(message);
                },
                UiEvent::RefreshList => {
                    info!("RECIBIDO!!");
                },
                UiEvent::SureTo(sureto) => {
                    match sureto {
                        SureTo::SureToMove { files, dest, tab_id } => {
                            debug!("Mover {:?} → {:?}", files, dest);
                            self.dialog_manager.open_sure_move_dialog(files, dest, tab_id);
                        },
                        SureTo::SureToDelete{files, tab_id} => {
                            self.dialog_manager.open_sure_to_delete(files, tab_id);
                        },
                        SureTo::SureToCopy => todo!(),
                    }
                },
                UiEvent::UpdateMessages(update_message) => {
                    match update_message {
                        UpdateMessages::NewVersionAvailable { current_version, new_version , tab_id} => {
                            info!("NUEVA VERSIÓN {:?}", new_version);
                            self.dialog_manager.open_updater_dialog(current_version, new_version, tab_id);
                        },
                        UpdateMessages::UpToDate => {
                            info!("Estás en la última versión.");
                        },
                        UpdateMessages::ProcedToUpdate => {

                        },
                    }
                },

                UiEvent::FileConflict(file_conflict) => {
                    match file_conflict {
                        FileConflict::AlreadyExist { name, path } => {
                            info!("Ya existe {} en {:?}", name, path);
                        },
                    }
                },

                UiEvent::ShowFolderColorSelector { folder_id } => {
                    self.dialog_manager.open_folder_color_selector_dialog(folder_id);
                }

                UiEvent::OpenConfigs => {
                    self.dialog_manager.open_configs();
                },

                UiEvent::ThumbnailReady { full_path, tab_id:_ } => {
                    self.calculating_thumbnails.remove(&full_path);
                    self.calculated_thumbnails.insert(full_path);
                },

                UiEvent::ShowImagePvw { pvw } => {
                    if let Some(img_pvw) = pvw {
                        self.dialog_manager.open_img_pvw_dialog(img_pvw);
                    }
                }
            }
        }
    }
}