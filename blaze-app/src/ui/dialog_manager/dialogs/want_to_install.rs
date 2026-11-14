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

use crate::core::bootstrap::install_manager::installation_manager::InstallResult;
use crate::core::{
    bootstrap::{
        configs::config_manager::with_configs,
        install_manager::installation_manager::with_installation_manager,
    },
    runtime::{bus_structs::UiEvent, event_bus::Dispatcher},
};
use crate::ui::custom_components::UiExt;
use crate::ui::dialog_manager::{
    dialogs::{DialogButtonKind, dialog_button, dialog_button_sized, dialog_frame},
    manager::ModalDialog,
};
use crate::ui::themes::theme_manager::with_theme;
use egui::{Order, RichText, Ui, Window};

pub struct WantToInstallDialog {
    pub show_modal: bool,
}

impl ModalDialog for WantToInstallDialog {
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

impl WantToInstallDialog {
    pub fn new() -> Self {
        Self { show_modal: false }
    }

    pub fn close(&mut self) {
        self.show_modal = false;
    }

    pub fn open(&mut self) {
        self.show_modal = true;
    }

    pub fn render_dialog(&mut self, ui: &mut Ui) -> bool {
        let i18n = with_configs(|c| c.get_i18n());

        let current_theme = with_theme(|t| t.current());
        let palette = current_theme.dialogs.palette(&current_theme.global);

        let mut should_close = false;

        Window::new(
            RichText::new(i18n.t("want_to_donwload_dialog.title")).color(palette.fg_primary),
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
                ui.add_space(8.0);
                ui.label_ns(
                    RichText::new(i18n.t("want_to_donwload_dialog.message"))
                        .color(palette.fg_primary),
                );
                ui.label_ns(
                    RichText::new(i18n.t("want_to_donwload_dialog.hint"))
                        .color(palette.fg_secondary),
                );
            });

            ui.add_space(30.0);

            ui.horizontal(|ui| {
                let spacing = ((ui.available_width() - 230.0) / 3.0).max(0.0);

                ui.add_space(spacing);

                if dialog_button(
                    ui,
                    i18n.t("want_to_donwload_dialog.yes"),
                    &palette,
                    DialogButtonKind::Primary,
                )
                .show()
                .clicked()
                {
                    let dispatcher = Dispatcher::current();

                    with_installation_manager(|im| {
                        let installresult = im.install();
                        match installresult {
                            InstallResult::InstalledSystem(path)
                            | InstallResult::InstalledLocal(path) => dispatcher
                                .send(UiEvent::ShowGeneric {
                                    title: i18n.t("want_to_donwload_dialog.installation_success"),
                                    message: i18n.t_args(
                                        "want_to_donwload_dialog.installation_path",
                                        &[("query", &path.display().to_string())],
                                    ),
                                })
                                .ok(),
                            InstallResult::Failed(e) => dispatcher.send(UiEvent::ShowError(e)).ok(),
                            _ => None,
                        }
                    });
                    should_close = true;
                }

                ui.add_space(spacing);

                if dialog_button_sized(
                    ui,
                    i18n.t("want_to_donwload_dialog.remind_me_later"),
                    &palette,
                    DialogButtonKind::Secondary,
                    (150.0, 30.0),
                )
                .show()
                .clicked()
                {
                    with_configs(|im| {
                        im.set_should_ask_install(false);
                    });
                    should_close = true;
                }

                ui.add_space(spacing);
            });
        });

        should_close
    }
}
