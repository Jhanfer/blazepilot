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

use crate::ui::{dialog_manager::dialog_manager::ModalDialog, themes::colors::COLOR_BG_MAIN};
use egui::{CornerRadius, Frame, Margin, Order, Ui, Window};
use tracing::info;

pub struct ShowGenericDialog {
    pub title: Option<Box<str>>,
    pub message: Option<Box<str>>,
    pub show_modal: bool,
}

impl ModalDialog for ShowGenericDialog {
    fn is_open(&self) -> bool {
        self.show_modal
    }
    fn close(&mut self) {
        self.close()
    }
    fn render(&mut self, ui: &mut Ui) {
        self.render_dialog(ui);
    }
}

impl ShowGenericDialog {
    pub fn new() -> Self {
        Self {
            title: None,
            message: None,
            show_modal: false,
        }
    }

    pub fn close(&mut self) {
        self.show_modal = false;
    }

    pub fn open(&mut self, title: &str, message: &str) {
        self.title = Some(title.into());
        self.message = Some(message.into());
        self.show_modal = true;
    }

    pub fn render_dialog(&mut self, ui: &mut Ui) {
        let mut should_close = false;

        let (Some(title), Some(message)) = (self.title.as_ref(), self.message.as_ref()) else {
            return;
        };

        let custom_frame = Frame::NONE
            .fill(COLOR_BG_MAIN)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(10));

        Window::new(title)
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
                    ui.label(message);
                    ui.add_space(8.0);
                });

                ui.add_space(50.0);

                ui.horizontal(|ui| {
                    let width = ui.available_width();
                    let button_width = 120.0;
                    let spacing = (width - button_width * 2.0) / 2.0;

                    ui.add_space(spacing);
                    if ui.button("Cerrar").clicked() {
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
