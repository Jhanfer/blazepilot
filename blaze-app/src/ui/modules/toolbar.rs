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




use egui::{Align, Color32, CornerRadius, Key, Layout, RichText, Sense, Stroke, TopBottomPanel};
use std::path::PathBuf;
use crate::core::blaze_state::BlazeCoreState;

pub fn toolbar_component(ctx: &egui::Context, state: &mut BlazeCoreState) {

    
    TopBottomPanel::top("toolbar")
        .min_height(42.0)           // altura cómoda
        .frame(egui::Frame::NONE.fill(Color32::from_rgb(80, 40, 140)))
        .show(ctx, |ui| {

            ui.with_layout(Layout::left_to_right(Align::Center), |ui|{
                ui.add_space(20.0);

                ui.horizontal(|ui| {

                    let nav_btn = |ui: &mut egui::Ui, label: &str| -> bool {
                        ui.add(
                            egui::Button::new(RichText::new(label).size(14.0).color(Color32::WHITE))
                                .frame(true)
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::new(1.0, Color32::from_rgb(120, 80, 200)))
                                .corner_radius(CornerRadius::same(6.0 as u8))
                                .min_size(egui::vec2(28.0, 28.0)),
                        )
                        .clicked()
                    };

                    ui.spacing_mut().item_spacing.x = 4.0;


                    // Botones de navegación (izquierda)
                    if nav_btn(ui, "<") {
                        state.back();
                    }

                    if nav_btn(ui,">") {
                        state.forward();
                    }

                    if nav_btn(ui,"UP") {
                        state.up();
                    }

                    if nav_btn(ui,"⟳") {
                        state.refresh();
                    }

                    ui.separator();

                    // === BREADCRUMB EN BLOQUES ===
                    let cwd = state.motor.borrow_mut().active_tab().cwd.clone();
                    let components: Vec<_> = cwd.components().collect();

                    let mut current_path = PathBuf::new();

                    ui.horizontal_centered(|ui| {
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
                                Color32::from_rgb(120, 80, 200) // morado más claro para el último
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

                    // Espacio flexible
                    let remaining = ui.available_width();
                    ui.add_space(remaining - 36.0); // 36 = ancho aprox del botón CO
                    if ui.small_button("CO").clicked() { /* ... */}
                });

                ui.add_space(20.0);
            });
        });
}


pub fn toolbar_component_old(ctx: &egui::Context, state: &mut BlazeCoreState) {

    TopBottomPanel::top("Toolbar")
        .min_height(30.0)
        .show(ctx, |ui|{
            ui.spacing();
        
            ui.horizontal(|ui|{
                
                if ui.button("<").clicked() {
                    state.back();
                }
                if ui.button(">").clicked() {
                    state.forward();
                }
                if ui.button("up").clicked() {
                    state.up();
                }
                if ui.button("refresh").clicked() {
                    state.refresh();
                }


                let response = ui.text_edit_singleline(&mut state.cwd_input);
                if response.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
                    let path = PathBuf::from(&state.cwd_input);
                    if path.is_dir() {
                        state.navigate_to(path);
                    }
                }

                if !response.has_focus() {
                    state.cwd_input = state.motor.borrow().tabs[state.motor.borrow().active_tab_index]
                        .cwd.to_string_lossy().to_string();
                }

            });

    });

} 