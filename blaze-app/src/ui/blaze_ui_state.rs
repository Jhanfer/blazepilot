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




use std::path::PathBuf;
use egui::{Area, CentralPanel, ColorImage, ComboBox, Context, Image, Order, RichText, ScrollArea, Sense, SizeHint, TextureOptions, TopBottomPanel, Window, load::Bytes, scroll_area::ScrollSource};
use tracing_subscriber::fmt::format;
use uuid::Uuid;
use crate::{core::{files::motor::with_motor, system::{clipboard::TOKIO_RUNTIME, fileopener_module::{AppAssociation, GLOBAL_FILE_OPENER, platform::linux::linux::AppsIconData}, updater::updater::UpdateMessages}}, ui::{dialogs::{selector_dialog::{AppSelectorDialog, SelectorData}, sure_to_move_to::SureToMoveToDialog, update_dialog::UpdateDialog}, icons_cache::icon_cache::IconCache}, utils::channel_pool::{FileOperation, SureTo, UiEvent, with_channel_pool}};
use tracing::info;


pub trait ModalDialog {
    fn is_open(&self) -> bool;
    fn close(&mut self);
    fn render(&mut self, ctx: &Context);
}


pub struct DialogManager {
    pub selector_dialog: AppSelectorDialog,
    pub sure_to_dialog: SureToMoveToDialog,
    pub update_dialog: UpdateDialog,
}

impl DialogManager {
    pub fn new() -> Self {
        Self {
            selector_dialog: AppSelectorDialog::new(),
            sure_to_dialog: SureToMoveToDialog::new(),
            update_dialog: UpdateDialog::new(),
        }
    }

    pub fn open_selector_dialog(&mut self, path: PathBuf, mime: String, apps: Vec<AppAssociation>, icon_data: Vec<AppsIconData>, show_all_apps: bool) {
        self.selector_dialog.open(path, mime, apps, icon_data, show_all_apps);
    }

    pub fn open_sure_move_dialog(&mut self, sources: Vec<PathBuf>, dest: PathBuf, tab_id: Uuid) {
        self.sure_to_dialog.open(sources, dest, tab_id);
    }

    pub fn open_updater_dialog(&mut self, current_version: String, new_version: String, tab_id: Uuid) {
        self.update_dialog.open(current_version, new_version, tab_id);
    }

    pub fn render_area(&mut self, ctx: &Context) {
        let dialogs: Vec<&mut dyn ModalDialog> = vec![
            &mut self.selector_dialog,
            &mut self.sure_to_dialog,
            &mut self.update_dialog,
        ];

        let open_dialog = dialogs.into_iter().find(|d| d.is_open());

        if let Some(dialog) = open_dialog {
            let mut should_close = false;

            Area::new("blocker".into())
                .fixed_pos(egui::pos2(0.0, 0.0))
                .order(Order::Middle)
                .sense(Sense::click())
                .interactable(true)
                .show(ctx, |ui|{
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

            dialog.render(ctx);
        }
    }

}




pub struct BlazeUiState {
    pub dialog_manager: DialogManager,
    pub icon_cache: IconCache,
}


impl BlazeUiState {
    pub fn new() -> Self {
        let dialog_manager = DialogManager::new();
        Self { 
            dialog_manager,
            icon_cache: IconCache::new(),
        }
    }

    pub fn process_events(&mut self) {
        let active_id = with_motor(|m| m.active_tab().id);
        let events: Vec<UiEvent> = with_channel_pool(|pool| {
            let  mut msgs = Vec::new();
            pool.process_ui_events(active_id, |msg|{
                msgs.push(msg);
                true
            });
            msgs
        });

        for envent in events {
            match envent {
                UiEvent::OpenWithSelector { path, mime, apps, icon_data, show_all_apps} => {
                    self.dialog_manager.open_selector_dialog(path, mime, apps, icon_data, show_all_apps);
                },


                UiEvent::ShowError(_) => todo!(),
                UiEvent::RefreshList => {
                    info!("RECIBIDO!!");
                },

                UiEvent::SureTo(sureto) => {
                    match sureto {
                        SureTo::SureToMove { files, dest, tab_id } => {
                            info!("Mover {:?} → {:?}", files, dest);
                            self.dialog_manager.open_sure_move_dialog(files, dest, tab_id);
                        },
                        SureTo::SureToDelete => todo!(),
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

                        },
                        UpdateMessages::ProcedToUpdate => {

                        },
                    }
                },
            }
        }
    }
}