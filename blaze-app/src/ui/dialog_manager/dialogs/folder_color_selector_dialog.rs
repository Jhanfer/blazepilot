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

use crate::core::bootstrap::configs::config_manager::with_configs;
use crate::core::system::{cache::cache_manager, clipboard::global_clipboard::TOKIO_RUNTIME};
use crate::ui::custom_components::UiExt;
use crate::ui::dialog_manager::{
    dialogs::{
        DialogButtonKind, dialog_button, dialog_button_sized, dialog_button_spacing, dialog_frame,
    },
    manager::ModalDialog,
};
use crate::ui::themes::theme_manager::with_theme;
use egui::{Color32, Order, RichText, Ui, Window};
use file_id::FileId;

pub struct FolderColorSelector {
    pub folder_id: Option<FileId>,
    pub show_modal: bool,
    temp_color: Option<Color32>,
}

impl ModalDialog for FolderColorSelector {
    fn is_open(&self) -> bool {
        self.show_modal
    }
    fn close(&mut self) {
        self.close()
    }
    fn render(&mut self, ui: &mut Ui) -> bool {
        self.render_dialog(ui)
    }
}

impl FolderColorSelector {
    pub fn new() -> Self {
        Self {
            folder_id: None,
            show_modal: false,
            temp_color: None,
        }
    }

    pub fn close(&mut self) {
        self.show_modal = false;
    }

    pub fn open(&mut self, folder_id: FileId) {
        let cm = cache_manager::CacheManager::global();
        let initial_color = cm.get_cached_color(&folder_id);

        self.temp_color = Some(initial_color);
        self.folder_id = Some(folder_id);
        self.show_modal = true;
    }

    pub fn render_dialog(&mut self, ui: &mut Ui) -> bool {
        let current_theme = with_theme(|t| t.current());
        let palette = current_theme.dialogs.palette(&current_theme.global);
        let i18n = with_configs(|c| c.get_i18n());

        let mut should_close = false;

        let Some(folder_id) = self.folder_id.as_ref() else {
            return false;
        };
        let Some(temp_color) = self.temp_color.as_mut() else {
            return false;
        };

        Window::new(
            RichText::new(i18n.t("select_folder_color_dialog.title")).color(palette.fg_primary),
        )
        .frame(dialog_frame(
            &palette,
            current_theme.global.panel_stroke_width,
        ))
        .order(Order::Foreground)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .open(&mut self.show_modal)
        .show(ui, |ui| {
            ui.set_min_width(250.0);
            ui.set_max_width(300.0);
            ui.set_min_height(100.0);

            ui.vertical_centered(|ui| {
                ui.color_picker(&mut *temp_color, "folder_color")
                    .background_color(palette.bg_elevated)
                    .show();

                ui.add_space(20.0);

                if dialog_button_sized(
                    ui,
                    i18n.t("select_folder_color_dialog.reset_pred"),
                    &palette,
                    DialogButtonKind::Secondary,
                    (190.0, 30.0),
                )
                .show()
                .clicked()
                {
                    *temp_color = Color32::YELLOW;
                }
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let spacing = dialog_button_spacing(ui, 2);

                ui.add_space(spacing);

                if dialog_button(
                    ui,
                    i18n.t("general_dialog.accept"),
                    &palette,
                    DialogButtonKind::Primary,
                )
                .show()
                .clicked()
                {
                    let cm = cache_manager::CacheManager::global();
                    let (folder_id, temp_color) = (*folder_id, *temp_color);

                    TOKIO_RUNTIME.spawn(async move {
                        cm.update_color_cache(folder_id, temp_color).await;
                        cm.save_color_cache().await;
                    });

                    should_close = true;
                }

                ui.add_space(spacing);

                if dialog_button(
                    ui,
                    i18n.t("general_dialog.cancel"),
                    &palette,
                    DialogButtonKind::Secondary,
                )
                .show()
                .clicked()
                {
                    should_close = true;
                }

                ui.add_space(spacing);
            });
        });

        should_close
    }
}
