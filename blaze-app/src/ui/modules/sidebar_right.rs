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





use std::sync::Arc;
use egui::{Button, Color32, CornerRadius, Frame, Margin, Panel, TextEdit, Ui};

use crate::core::files::motor::FileEntry;
use crate::core::system::extended_info::extended_info_manager::ExtendedInfo;
use crate::core::{blaze_state::BlazeCoreState, configs::config_state::{OrderingMode, with_configs}};
use crate::utils::formating::{format_date, format_size};

pub fn sidebar_right_component(ui: &mut Ui, state: &mut BlazeCoreState, files: &Vec<Arc<FileEntry>>) {

    let custom_frame = Frame::NONE
        .fill(Color32::from_rgb(16, 21, 25))
        .inner_margin(Margin {
            left: 5,
            right: 5,
            top: 0,
            bottom: 0,
        });

    Panel::right("info_panel")
        .resizable(false)
        .exact_size(230.0)
        .frame(custom_frame)
        .show_separator_line(false)
        .show_inside(ui,|ui| {


        Frame::NONE
            .inner_margin(egui::Margin::same(10))
            .fill(Color32::from_rgb(27, 31, 35))
            .corner_radius(CornerRadius::same(20))
            .show(ui, |ui|{

                ui.horizontal(|ui|{
                    ui.label("🔍");

                    let mut search = state.search_filter.clone();
                    let response = ui.add(
                        TextEdit::singleline(&mut search)
                            .id("search_bar".into())
                            .hint_text("Búsqueda")
                            .desired_width(ui.max_rect().width() - 60.0)
                    );
                    
                    if response.changed() {
                        state.set_search(search);
                        response.request_focus(); 
                    }

                    if ui.add_enabled(!state.search_filter.is_empty(), Button::new("X")).clicked() {
                        state.clean_search();
                    }
                });

                ui.label("Orden");

                ui.horizontal_top(|ui|{

                    let current_order = with_configs(|c|{
                        c.configs.app_ordering_mode.clone()
                    });


                    let alphabetic_label = match current_order  {
                        OrderingMode::Az => "A-Z",
                        OrderingMode::Za => "Z-A",
                        _ => "A-Z",
                    };

                    if ui.button(alphabetic_label).clicked() {
                        with_configs(|c| {
                            let new_mode = match current_order {
                                OrderingMode::Az => OrderingMode::Za,
                                _ => OrderingMode::Az,
                            };

                            c.set_ordering_mode(new_mode);
                        });
                        state.refresh();
                    };


                    let size_label = match current_order  {
                        OrderingMode::SizeAsc => "Gb ↗",
                        OrderingMode::SizeDesc => "Gb ↘",
                        _ => "Gb ↗",
                    };

                    if ui.button(size_label).clicked() {
                        with_configs(|c| {
                            let new_mode = match current_order {
                                OrderingMode::SizeAsc => OrderingMode::SizeDesc,
                                _ => OrderingMode::SizeAsc,
                            };

                            c.set_ordering_mode(new_mode);
                        });
                        state.refresh();
                    };


                    let date_label = match current_order {
                        OrderingMode::DateAsc => "⏳",
                        OrderingMode::DateDesc => "⌛",
                        _ => "⏳",
                    };

                    if ui.button(date_label).clicked() {
                        with_configs(|c| {
                            let new_mode = match current_order {
                                OrderingMode::DateAsc => OrderingMode::DateDesc,
                                _ => OrderingMode::DateAsc,
                            };

                            c.set_ordering_mode(new_mode);
                        });
                        state.refresh();
                    };
                });


                if let Some(first_selected_idx) = (0..files.len()).find(|&i| state.is_selected(i)) {
                    let file = &files[first_selected_idx];

                    ui.heading("Info");
                    ui.separator();
                    
                    ui.heading(file.name.clone());
                    ui.label(format_date(file.modified));
                    
                    if file.is_dir {
                        ui.label("Tipo: Carpeta");
                    } else {
                        ui.label(format!("Tipo: {:?}", file.extension));
                        ui.label(format!("Tamaño: {:?}", format_size(file.size)));
                    }

                    let extended_info = if state.calculating_extended_info.contains(&file.full_path) {
                        None
                    } else if state.calculated_extended_info.contains(&file.full_path) {
                        state.extended_info_manager.info_map.read().unwrap().get(&file.full_path).cloned()
                    } else {
                        state.extended_info_manager.cache_manager
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
                            ui.label(format!("Owner: {}", owner));
                        }
                        if let Some(group_name) = extended_info.group_name {
                            ui.label(format!("Group name: {}", group_name));
                        }
                        if let Some(symlink_target) = extended_info.symlink_target {
                            ui.label(format!("Tipo: {:?}", symlink_target));
                        }
                        if let Some(dimensions) = extended_info.dimensions {
                            ui.label(format!("Dimensión: {:?} x {:?}", dimensions.0, dimensions.1));
                        }
                        if let Some(git_status) = extended_info.git_status {
                            ui.label(format!("GitStatus: {:?}", git_status));
                        }
                    }


                }
            });
    });
}