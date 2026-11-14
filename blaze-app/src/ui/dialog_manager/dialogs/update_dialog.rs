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
    core::{
        bootstrap::configs::config_manager::with_configs,
        runtime::{bus_structs::FileOperation, event_bus::Dispatcher},
    },
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
use uuid::Uuid;

pub struct UpdateDialog {
    pub current_version: Option<Box<str>>,
    pub new_version: Option<Box<str>>,
    pub tab_id: Option<Uuid>,
    pub show_modal: bool,
}

impl ModalDialog for UpdateDialog {
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

impl UpdateDialog {
    pub fn new() -> Self {
        Self {
            current_version: None,
            new_version: None,
            tab_id: None,
            show_modal: false,
        }
    }

    pub fn close(&mut self) {
        self.show_modal = false;
    }

    pub fn open(&mut self, current_version: Box<str>, new_version: Box<str>, tab_id: Uuid) {
        self.current_version = Some(current_version);
        self.new_version = Some(new_version);
        self.tab_id = Some(tab_id);
        self.show_modal = true;
    }

    pub fn render_dialog(&mut self, ui: &mut Ui) -> bool {
        let i18n = with_configs(|c| c.get_i18n());
        let current_theme = with_theme(|t| t.current());
        let palette = current_theme.dialogs.palette(&current_theme.global);

        let mut should_close = false;

        let (Some(current_ver), Some(new_ver), Some(_)) = (
            self.current_version.as_ref(),
            self.new_version.as_ref(),
            self.tab_id.as_ref(),
        ) else {
            return false;
        };

        Window::new(RichText::new(i18n.t("update_dialog.title")).color(palette.fg_primary))
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
                    ui.label_ns(
                        RichText::new(i18n.t("update_dialog.message")).color(palette.fg_primary),
                    );
                    ui.add_space(8.0);

                    ui.label_ns(
                        RichText::new(
                            i18n.t_args("update_dialog.current_version", &[("query", current_ver)]),
                        )
                        .color(palette.fg_primary),
                    );
                    ui.add_space(8.0);

                    ui.label_ns(
                        RichText::new(
                            i18n.t_args("update_dialog.new_version", &[("query", new_ver)]),
                        )
                        .color(palette.fg_primary),
                    );
                    ui.add_space(5.0);

                    if ui
                        .link(
                            RichText::new(i18n.t("update_dialog.changelog"))
                                .color(palette.accent_text),
                        )
                        .clicked()
                    {
                        ui.open_url(egui::OpenUrl::new_tab(
                            "https://github.com/Jhanfer/blazepilot/blob/main/CHANGELOG.md",
                        ));
                    }
                });

                ui.add_space(20.0);

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
                        Dispatcher::current().send(FileOperation::Update).ok();
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
