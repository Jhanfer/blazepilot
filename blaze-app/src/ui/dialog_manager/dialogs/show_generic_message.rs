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

use crate::{
    core::bootstrap::configs::config_manager::with_configs,
    ui::{
        custom_components::UiExt,
        dialog_manager::{
            dialogs::{DialogButtonKind, dialog_button, dialog_button_spacing, dialog_frame},
            manager::ModalDialog,
        },
        themes::theme_manager::with_theme,
    },
};
use egui::{Order, RichText, Ui, Window};

pub struct ShowGenericDialog {
    pub title: Option<Box<str>>,
    pub message: Option<Box<str>>,
    pub show_modal: bool,
}

impl ModalDialog for ShowGenericDialog {
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

impl ShowGenericDialog {
    pub fn new() -> Self {
        Self {
            title: None,
            message: None,
            show_modal: false,
        }
    }

    pub fn close(&mut self) {
        self.show_modal = false;
    }

    pub fn open(&mut self, title: &str, message: &str) {
        self.title = Some(title.into());
        self.message = Some(message.into());
        self.show_modal = true;
    }

    pub fn render_dialog(&mut self, ui: &mut Ui) -> bool {
        let i18n = with_configs(|c| c.get_i18n());
        let current_theme = with_theme(|t| t.current());
        let palette = current_theme.dialogs.palette(&current_theme.global);
        let mut should_close = false;

        let (Some(title), Some(message)) = (self.title.as_ref(), self.message.as_ref()) else {
            return false;
        };

        Window::new(RichText::new(title.to_string()).color(palette.fg_primary))
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
                    ui.label_ns(RichText::new(message.to_string()).color(palette.fg_primary));
                    ui.add_space(8.0);
                });

                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    let spacing = dialog_button_spacing(ui, 1);

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
                        should_close = true;
                    }

                    ui.add_space(spacing);
                });
            });

        should_close
    }
}
