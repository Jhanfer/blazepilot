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





use egui::{Color32, Ui, CornerRadius, Frame, Margin, Order, Window};
use tracing::info;
use crate::{core::{runtime::{bus_structs::FileOperation, event_bus::Dispatcher}}, ui::blaze_ui_state::ModalDialog};


pub struct ErrorDialog {
    pub message: Option<String>,
    pub show_modal: bool,
}

impl ModalDialog for ErrorDialog {
    fn is_open(&self) -> bool { self.show_modal }
    fn close(&mut self) { self.close() }
    fn render(&mut self, ui: &mut Ui) { self.render_dialog(ui); }
}

impl ErrorDialog {
    pub fn new() -> Self {
        Self {
            message: None,
            show_modal: false,
        }
    }

    pub fn close(&mut self) {
        self.show_modal = false; 
    }

    pub fn open(&mut self, message: String) {
        self.message = Some(message);
        self.show_modal = true;
    }


    pub fn render_dialog(&mut self, ui: &mut Ui) {
        let mut should_close = false;

        let Some(message) = self.message.as_ref() else { return; };
        
        let custom_frame = Frame::NONE
            .fill(Color32::from_rgb(16, 21, 25))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(10));

        Window::new("¡Ha ocurrido un error!")
            .frame(custom_frame)
            .order(Order::Foreground)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut self.show_modal)
            .show(ui, |ui|{
                ui.set_min_width(250.0);
                ui.set_min_height(100.0);
                
                ui.vertical_centered(|ui|{
                    ui.label(message);
                    ui.add_space(8.0);
                });


                ui.add_space(50.0);
                
                ui.horizontal(|ui| {
                    let width = ui.available_width();
                    let button_width = 120.0;
                    let spacing = (width - button_width * 2.0) / 3.0;

                    ui.add_space(spacing);
                    if ui.button("Aceptar").clicked() {

                        Dispatcher::current().send(FileOperation::Update).ok();

                        should_close = true;
                    }
                });
            });

        if should_close {
            info!("Se cierra");
            self.close();
        }
    }
}