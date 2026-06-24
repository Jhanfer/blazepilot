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
        blaze_state::BlazeCoreState,
        runtime::{bus_structs::UiEvent, event_bus::with_event_bus},
    },
    ui::{
        blaze_ui_state::BlazeUiState,
        icons_cache::icons::{self},
        modules::utilities::ensure_min_lightness,
        themes::colors::*,
    },
};
use egui::{
    pos2, vec2, Color32, CornerRadius, Frame, Margin, Panel, Rect, RichText, Sense, Stroke, Ui,
};
use std::path::PathBuf;

fn render_bar_button<F>(
    ui: &mut Ui,
    total_height: f32,
    label: &'static str,
    bytes: &[u8],
    ui_state: &mut BlazeUiState,
    active: bool,
    mut callback: F,
) where
    F: FnMut(),
{
    let ball_size = 35.0;
    Frame::new()
        .fill(COLOR_BG_PANEL)
        .stroke(Stroke::new(0.5, COLOR_ACCENT_GLOW))
        .corner_radius(CornerRadius::same((ball_size / 1.5) as u8))
        .show(ui, |ui| {
            ui.set_width(ball_size);
            ui.set_height(total_height);
            let icon_size = vec2(16.0, 16.0);
            let (rect, resp) =
                ui.allocate_exact_size(vec2(ball_size, total_height), Sense::click());

            let icon_pos = rect.left_center() - vec2(-10.0, icon_size.y / 2.0);
            let icon_rect = Rect::from_min_size(icon_pos, icon_size);

            let rounded_rect = Rect::from_min_max(
                pos2(icon_rect.min.x.round(), icon_rect.min.y.round()),
                pos2(icon_rect.max.x.round(), icon_rect.max.y.round()),
            );

            let color = if resp.hovered() && active {
                ensure_min_lightness(COLOR_ACCENT_GLOW, 0.90)
            } else if !active {
                COLOR_TEXT_MUTED
            } else {
                Color32::WHITE
            };

            let icon = ui_state
                .icon_cache
                .get_or_load(ui, label, bytes, color, icon_size);

            ui.painter().image(
                icon.id(),
                rounded_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                color,
            );

            if resp.clicked() && active {
                callback();
            }

            if resp.hovered() && active {
                ui.set_cursor_icon(egui::CursorIcon::PointingHand);
            }
        });
}

pub fn toolbar_component(ui: &mut Ui, state: &mut BlazeCoreState, ui_state: &mut BlazeUiState) {
    Panel::top("toolbar")
        .frame(Frame::new().fill(COLOR_BG_MAIN).inner_margin(10))
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
                    .fill(COLOR_BG_PANEL)
                    .show(ui, |ui| {
                        ui.set_height(total_height);
                        ui.set_width(ui.available_width());

                        let back_active = state.can_go_back();

                        render_bar_button(
                            ui,
                            total_height,
                            "<",
                            icons::ICON_ARROW_LEFT,
                            ui_state,
                            back_active,
                            || state.back(),
                        );

                        let forward_active = state.can_go_forward();

                        render_bar_button(
                            ui,
                            total_height,
                            ">",
                            icons::ICON_ARROW_RIGHT,
                            ui_state,
                            forward_active,
                            || state.forward(),
                        );

                        let up_active = state.can_go_up();

                        render_bar_button(
                            ui,
                            total_height,
                            "UP",
                            icons::ICON_ARROW_UP,
                            ui_state,
                            up_active,
                            || state.up(),
                        );

                        render_bar_button(
                            ui,
                            total_height,
                            "⚙️",
                            icons::ICON_SETTINGS,
                            ui_state,
                            true,
                            || {
                                let tab_id = state.active_id;
                                let dispatcher = with_event_bus(|e| e.dispatcher(tab_id));
                                dispatcher.send(UiEvent::OpenConfigs).ok();
                            },
                        );

                        Frame::new()
                            .fill(COLOR_BG_PANEL)
                            .stroke(Stroke::new(0.5, COLOR_ACCENT_GLOW))
                            .corner_radius(CornerRadius::same(20))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.set_height(total_height);

                                ui.horizontal_centered(|ui| {
                                    ui.add_space(15.0);

                                    let components: Vec<_> = cwd.components().collect();

                                    let mut current_path = PathBuf::new();

                                    for (i, component) in components.iter().enumerate() {
                                        let name =
                                            component.as_os_str().to_string_lossy().to_string();
                                        if name.is_empty() {
                                            continue;
                                        }

                                        current_path.push(component);

                                        let is_last = i == components.len() - 1;

                                        let button = egui::Button::new(
                                            RichText::new(name)
                                                .color(if is_last {
                                                    Color32::WHITE
                                                } else {
                                                    Color32::LIGHT_GRAY
                                                })
                                                .strong(),
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
                                            state.navigate_to(current_path.to_owned().into());
                                        }

                                        // Separador ">"
                                        if !is_last {
                                            ui.label(
                                                RichText::new("›").color(Color32::GRAY).size(16.0),
                                            );
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
