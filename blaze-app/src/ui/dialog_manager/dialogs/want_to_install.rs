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
use crate::ui::dialog_manager::manager::ModalDialog;
use crate::ui::themes::platform::structs::ToColor;
use crate::ui::themes::theme_manager::with_theme;
use egui::{CornerRadius, Frame, Margin, Order, Ui, Window};

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

        let mut should_close = false;
        let custom_frame = Frame::NONE
            .fill(current_theme.bg_main.to_color())
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(10));

        Window::new(i18n.t("want_to_donwload_dialog.title"))
            .frame(custom_frame)
            .order(Order::Foreground)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut self.show_modal)
            .show(ui, |ui| {
                ui.set_min_width(250.0);
                ui.set_min_height(100.0);

                ui.vertical_centered(|ui| {
                    ui.add_space(8.0);
                    ui.label(i18n.t("want_to_donwload_dialog.message"));

                    ui.label(i18n.t("want_to_donwload_dialog.hint"));
                });

                ui.add_space(50.0);

                ui.horizontal(|ui| {
                    let width = ui.available_width();
                    let button_width = 120.0;
                    let spacing = (width - button_width * 2.0) / 2.0;

                    ui.add_space(spacing);
                    if ui.button(i18n.t("want_to_donwload_dialog.yes")).clicked() {
                        let dispatcher = Dispatcher::current();

                        with_installation_manager(|im| {
                            let installresult = im.install();
                            match installresult {
                                InstallResult::InstalledSystem(path)
                                | InstallResult::InstalledLocal(path) => dispatcher
                                    .send(UiEvent::ShowGeneric {
                                        title: i18n
                                            .t("want_to_donwload_dialog.installation_success"),
                                        message: i18n.t_args(
                                            "want_to_donwload_dialog.installation_path",
                                            &[("query", &path.display().to_string())],
                                        ),
                                    })
                                    .ok(),
                                InstallResult::Failed(e) => {
                                    dispatcher.send(UiEvent::ShowError(e)).ok()
                                }
                                _ => None,
                            }
                        });
                        should_close = true;
                    }

                    if ui.button(i18n.t("")).clicked() {
                        with_configs(|im| {
                            im.set_should_ask_install(false);
                        });
                        should_close = true;
                    }
                });
            });

        should_close
    }
}
