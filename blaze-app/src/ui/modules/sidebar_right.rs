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




use core::f32;
use std::path::PathBuf;
use std::{sync::Arc, time::{Duration, UNIX_EPOCH}};
use chrono::{DateTime, Local, TimeDelta};
use egui::{Button, Color32, CornerRadius, Frame, Key, Margin, SidePanel, TextEdit, Ui};
use tracing::info;

use crate::core::files::motor::FileEntry;
use crate::core::{blaze_state::BlazeCoreState, configs::config_state::{OrderingMode, with_configs}};



pub fn format_size(size: u64) -> String {
    let size_f = size as f64;
    let kb = 1024.0;
    let mb = 1024.0 * 1024.0;
    let gb = 1024.0 * 1024.0 * 1024.0;

    if size_f < kb {
        format!("{} B", size)
    } else if size_f < mb {
        format!("{:.1} KB", size_f / kb)
    } else if size_f < gb {
        format!("{:.1} MB", size_f / mb)
    } else {
        format!("{:.2} GB", size_f / gb)
    }
}

fn format_date(seconds: u64) -> String {
    if seconds == 0 { return "---".to_string(); }

    let d = UNIX_EPOCH + Duration::from_secs(seconds);

    let modified_date: DateTime<Local> = d.into();
    let now: DateTime<Local> = Local::now();
    
    let diff: TimeDelta = now - modified_date;
    
    let min: i32 = diff.num_minutes() as i32;
    let hours: i32 = diff.num_hours() as i32;
    let days: i32 = diff.num_days() as i32;
    let weeks: i32 = days / 7;
    let months: i32 = weeks / 4;
    let years: i32 = months / 12;

    
    if min < 60 {
        return format!("Hace {:?}min", min).to_string();
    } else if hours < 24 {
        return format!("Hace {:?}h", hours).to_string();
    } else if days < 7 {
        return format!("Hace {:?} dia/s", days).to_string();
    } else if weeks < 4 {
        return format!("Hace {:?} semana/s", weeks).to_string();
    } else if months < 12 {
        return format!("Hace {:?} mes/ses", months).to_string();
    } else if years > 1 {
        return format!("Hace {:?} año/s", years).to_string();
    }
    return "desconocido".to_string();
}



pub fn sidebar_right_component(state: &mut BlazeCoreState, ui: &mut Ui, files: &Vec<Arc<FileEntry>>) {

    let custom_frame = Frame::NONE
        .fill(Color32::from_rgb(27, 31, 35))
        .inner_margin(Margin::same(10));

    SidePanel::right("info_panel")
        .resizable(false)
        .default_width(200.0)
        .min_width(120.0)
        .exact_width(240.0)
        .frame(custom_frame)
        .show_separator_line(false)
        .show_inside(ui, |ui| {


        Frame::NONE
            .inner_margin(egui::Margin::same(10))
            .fill(Color32::from_rgb(36, 42, 47))
            .corner_radius(CornerRadius::same(10))
            .show(ui, |ui|{

                ui.horizontal(|ui|{
                    ui.label("🔍");

                    let mut search = state.search_filter.clone();
                    let response = ui.add(
                        TextEdit::singleline(&mut search)
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

                ui.label("Ordén");

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
                        OrderingMode::SizeAsc => "Size ↗",
                        OrderingMode::SizeDesc => "Size ↘",
                        _ => "Size ↗",
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
                        OrderingMode::DateAsc => "Date ⏳",
                        OrderingMode::DateDesc => "Date ⌛",
                        _ => "Date ⏳",
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

                    let is_hidden = with_configs(|c| {c.configs.show_hidden_files.clone()});

                    let date_label = if is_hidden { "Oculto" } else {"Mostrar"};

                    if ui.button(date_label).clicked() {
                        with_configs(|c| {
                            c.set_show_hidden_files(!is_hidden);
                        });
                        state.refresh();
                    };
                });


                if let Some(selected_path) = state.selected_files.iter().next() {
                    let matching_file = files.iter().find(|file| &file.full_path == selected_path);
                    
                    if let Some(file) = matching_file {
                        ui.heading("Info");
                        ui.separator();
                        
                        ui.heading(file.name.clone());
                        ui.label(format_date(file.modified));
                        // etc.
                    }
                }
            });
    });
}