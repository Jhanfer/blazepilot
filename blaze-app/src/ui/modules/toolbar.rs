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





use egui::{Color32, CornerRadius, Frame, Margin, Panel, Rect, RichText, Sense, Stroke, Ui, pos2, vec2};
use std::path::PathBuf;
use crate::{core::blaze_state::BlazeCoreState, ui::{blaze_ui_state::BlazeUiState, icons_cache::icons}, utils::channel_pool::UiEvent};


fn render_bar_button<F>(ui: &mut Ui, total_height: f32, label: &'static str, bytes: &[u8], ui_state: &mut BlazeUiState, mut callback: F)
where F: FnMut(),
{
    let ball_size = 35.0;
    Frame::new()
    .fill(Color32::from_rgb(80, 40, 140))
    .corner_radius(CornerRadius::same((ball_size / 1.5) as u8))
    .show(ui, |ui| {
        ui.set_width(ball_size as f32);
        ui.set_height(total_height);

        let (rect, resp) = ui.allocate_exact_size(
            vec2(ball_size, total_height),
            Sense::click(),
        );

        let icon = ui_state.icon_cache.get_or_load(ui, label, bytes, Color32::WHITE);

        let icon_size = vec2(16.0, 16.0);
        let icon_pos = rect.left_center() - vec2(-10.0, icon_size.y / 2.0);
        let icon_rect = Rect::from_min_size(icon_pos, icon_size);

        ui.painter().image(
            icon.id(),
            icon_rect,
            Rect::from_min_max(pos2(0.0, 0.0),
            pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        if resp.clicked() {
            callback();
        }

        if resp.hovered() {
            ui.set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    });
}


pub fn toolbar_component(ui: &mut Ui, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {
    Panel::top("toolbar")
        .frame(Frame::new().fill(Color32::from_rgb(16, 21, 25)).inner_margin(10))
        .exact_size(80.0)
        .show_separator_line(false)
        .show_inside(ui, |ui| {
            let total_height = 35.0;
            let spacing = 8.0;

            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = spacing;

                let cwd = state.cwd.clone();

                Frame::new()
                .corner_radius(20)
                .inner_margin(Margin::same(10))
                .fill(Color32::from_rgb(27, 31, 35))
                .show(ui, |ui|{
                    ui.set_height(total_height);
                    ui.set_width(ui.available_width());

                    render_bar_button(ui, total_height, "<", icons::ICON_ARROW_LEFT, ui_state, || state.back());

                    render_bar_button(ui, total_height, ">", icons::ICON_ARROW_RIGHT, ui_state, || state.forward());

                    render_bar_button(ui, total_height, "UP", icons::ICON_ARROW_UP, ui_state, || state.up());

                    render_bar_button(ui, total_height, "⚙️", icons::ICON_SETTINGS, ui_state, || {
                        if let Some(sender) = state.sender().cloned() {
                            sender.send_ui_event(
                                UiEvent::OpenConfigs
                            ).ok();
                        }
                    });
                    
                    
                    Frame::new()
                        .fill(Color32::from_rgb(80, 40, 140))
                        .corner_radius(CornerRadius::same(20))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(total_height);


                            ui.horizontal_centered(|ui| {
                                    ui.add_space(15.0);


                                    let components: Vec<_> = cwd.components().collect();

                                    let mut current_path = PathBuf::new();

                                    for (i, component) in components.iter().enumerate() {
                                        let name = component.as_os_str().to_string_lossy().to_string();
                                        if name.is_empty() { continue; }

                                        current_path.push(component);

                                        let is_last = i == components.len() - 1;

                                        let button = egui::Button::new(
                                            RichText::new(name)
                                                .color(if is_last { Color32::WHITE } else { Color32::LIGHT_GRAY })
                                                .strong()
                                        )
                                        .frame(true)
                                        .fill(if is_last {
                                            Color32::from_rgb(120, 80, 200)
                                        } else {
                                            Color32::TRANSPARENT
                                        })
                                        .stroke(Stroke::NONE)
                                        .corner_radius(CornerRadius::same(6.0 as u8))
                                        .min_size(egui::vec2(0.0, 28.0));

                                        let response = ui.add(button);

                                        if response.clicked() && !is_last {
                                            state.navigate_to(current_path.clone());
                                        }

                                        // Separador ">"
                                        if !is_last {
                                            ui.label(RichText::new("›").color(Color32::GRAY).size(16.0));
                                        }
                                    }
                                

                            });

                            
                            let remaining = ui.available_width();
                            ui.add_space(remaining - 36.0);
                            if ui.small_button("📋").clicked() {
                                ui.copy_text(cwd.to_string_lossy().to_string());
                            }

                        });
                });
            });
        });
}