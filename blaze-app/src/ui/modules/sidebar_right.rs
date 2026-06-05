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

use crate::core::blaze_state::BlazeCoreState;
use crate::core::bootstrap::configs::config_manager::with_configs;
use crate::core::files::blaze_motor::motor_structs::FileEntry;
use crate::core::system::extended_info::extended_info_manager::ExtendedInfo;
use crate::ui::themes::colors::*;
use crate::utils::formating::{format_date, format_size};
use egui::{Button, CornerRadius, Frame, Margin, Panel, Stroke, TextEdit, Ui};
use std::sync::Arc;
use tracing::error;

pub fn sidebar_right_component(ui: &mut Ui, state: &mut BlazeCoreState, files: &[Arc<FileEntry>]) {
    let i18n = with_configs(|c| c.get_i18n());
    let custom_frame = Frame::NONE.fill(COLOR_BG_MAIN).inner_margin(Margin {
        left: 0,
        right: 15,
        top: 0,
        bottom: 0,
    });

    Panel::right("info_panel")
        .resizable(false)
        .frame(custom_frame)
        .show_separator_line(false)
        .show_inside(ui, |ui| {
            Frame::NONE
                .inner_margin(egui::Margin::same(10))
                .fill(COLOR_BG_PANEL)
                .corner_radius(CornerRadius::same(20))
                .stroke(Stroke {
                    width: 0.5,
                    color: COLOR_ACCENT_GLOW,
                })
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("🔍");

                        let mut search = state.search_filter.clone();
                        let response = ui.add(
                            TextEdit::singleline(&mut search)
                                .id("search_bar".into())
                                .hint_text(i18n.t("search.placeholder"))
                                .desired_width(150.0),
                        );

                        if response.changed() {
                            state.set_search(search);
                            response.request_focus();
                        }

                        if ui
                            .add_enabled(!state.search_filter.is_empty(), Button::new("X"))
                            .clicked()
                        {
                            state.clean_search();
                        }
                    });

                    if let Some(first_selected_idx) =
                        (0..files.len()).find(|&i| state.is_selected(i))
                    {
                        let file = &files[first_selected_idx];

                        ui.heading(i18n.t("right_sidebar.info"));
                        ui.separator();

                        ui.heading(file.name.clone());
                        ui.label(format_date(file.modified));

                        if file.is_dir() {
                            ui.label(i18n.t("right_sidebar.info"));
                        } else {
                            ui.label(i18n.t_args(
                                "right_sidebar.ext",
                                &[("query", &format!("{:?}", file.extension))],
                            ));
                            ui.label(i18n.t_args(
                                "right_sidebar.size",
                                &[("query", &format_size(file.size))],
                            ));
                        }

                        let extended_info =
                            if state.calculating_extended_info.contains(&file.full_path) {
                                None
                            } else if state.calculated_extended_info.contains(&file.full_path) {
                                match state.extended_info_manager.info_map.write() {
                                    Ok(mut map) => map.get(&file.full_path).cloned(),
                                    Err(e) => {
                                        error!(
                                            "Ha ocurrio un error intentando leer ExtendedInfo: {}",
                                            e
                                        );
                                        None
                                    }
                                }
                            } else {
                                state
                                    .extended_info_manager
                                    .cache_manager
                                    .get_cached_extended_info(&file.full_path)
                                    .map(|c| ExtendedInfo {
                                        owner: c.owner,
                                        group_name: c.group_name,
                                        symlink_target: c.symlink_target,
                                        dimensions: c.dimensions,
                                        git_status: c.git_status,
                                    })
                            };

                        if let Some(extended_info) = extended_info {
                            if let Some(owner) = extended_info.owner {
                                ui.label(i18n.t_args("right_sidebar.owner", &[("query", &owner)]));
                            }
                            if let Some(group_name) = extended_info.group_name {
                                ui.label(
                                    i18n.t_args(
                                        "right_sidebar.group_name",
                                        &[("query", &group_name)],
                                    ),
                                );
                            }
                            if let Some(symlink_target) = extended_info.symlink_target {
                                ui.label(i18n.t_args(
                                    "right_sidebar.type",
                                    &[("query", &symlink_target.display().to_string())],
                                ));
                            }
                            if let Some(dimensions) = extended_info.dimensions {
                                let w_str = dimensions.0.to_string();
                                let h_str = dimensions.1.to_string();

                                let text = i18n.t_args(
                                    "right_sidebar.dimensions",
                                    &[("w", &w_str), ("h", &h_str)],
                                );
                                ui.label(&*text);
                            }
                            if let Some(git_status) = extended_info.git_status {
                                let status_debug = format!("{:?}", git_status);
                                let text = i18n.t_args(
                                    "right_sidebar.git_status",
                                    &[("query", &status_debug)],
                                );
                                ui.label(&*text);
                            }
                        }
                    }
                });
        });
}
