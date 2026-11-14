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

use std::{path::Path, sync::Arc};

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

pub struct SureToMoveToDialog {
    pub sources: Option<Vec<Arc<Path>>>,
    pub dest: Option<Arc<Path>>,
    pub show_modal: bool,
}

impl ModalDialog for SureToMoveToDialog {
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

impl SureToMoveToDialog {
    pub fn new() -> Self {
        Self {
            sources: None,
            dest: None,
            show_modal: false,
        }
    }

    pub fn close(&mut self) {
        self.show_modal = false;
    }

    pub fn open(&mut self, sources: Vec<Arc<Path>>, dest: Arc<Path>) {
        self.sources = Some(sources);
        self.dest = Some(dest);
        self.show_modal = true;
    }

    pub fn render_dialog(&mut self, ui: &mut Ui) -> bool {
        let i18n = with_configs(|c| c.get_i18n());
        let current_theme = with_theme(|t| t.current());
        let palette = current_theme.dialogs.palette(&current_theme.global);

        let mut should_close = false;

        let (Some(sources), Some(dest)) = (self.sources.as_ref(), self.dest.as_ref()) else {
            return false;
        };

        Window::new(RichText::new(i18n.t("move_to_dialog.title")).color(palette.fg_primary))
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
                        RichText::new(i18n.t("move_to_dialog.heading"))
                            .color(palette.fg_primary)
                            .size(18.0),
                    );
                });

                const MAX_SHOWN: usize = 5;
                let total = sources.len();

                for source in sources.iter().take(MAX_SHOWN) {
                    let file_name = source
                        .file_name()
                        .map(|f| f.to_string_lossy().into_owned())
                        .unwrap_or_else(|| i18n.t("move_to_dialog.file").into_string());

                    ui.label_ns(
                        RichText::new(
                            i18n.t_args("move_to_dialog.file_dot", &[("query", &file_name)]),
                        )
                        .color(palette.fg_primary),
                    );
                }

                if total > MAX_SHOWN {
                    ui.label_ns(
                        RichText::new(i18n.t_args(
                            "move_to_dialog.and_more",
                            &[("query", &(total - MAX_SHOWN).to_string())],
                        ))
                        .color(palette.fg_secondary)
                        .weak()
                        .italics(),
                    );
                }

                let dest_name = dest
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_else(|| dest.to_string_lossy().into_owned());

                ui.add_space(8.0);
                ui.label_ns(
                    RichText::new(i18n.t_args("move_to_dialog.to", &[("query", &dest_name)]))
                        .color(palette.fg_primary),
                );

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
                        Dispatcher::current()
                            .send(FileOperation::Move {
                                sources: sources.to_vec(),
                                dest: dest.to_owned(),
                            })
                            .ok();
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
