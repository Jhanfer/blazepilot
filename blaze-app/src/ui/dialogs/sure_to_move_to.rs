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





use std::path::PathBuf;
use egui::{Color32, Context, CornerRadius, Frame, Margin, Order, RichText, Window};
use tracing::info;
use uuid::Uuid;
use crate::{ui::blaze_ui_state::ModalDialog, utils::channel_pool::{FileOperation, with_active_sender}};


pub struct SureToMoveToDialog {
    pub sources: Option<Vec<PathBuf>>,
    pub dest: Option<PathBuf>,
    pub tab_id: Option<Uuid>,
    pub show_modal: bool,
}

impl ModalDialog for SureToMoveToDialog {
    fn is_open(&self) -> bool { self.show_modal }
    fn close(&mut self) { self.close() }
    fn render(&mut self, ctx: &Context) { self.render_dialog(ctx); }
}

impl SureToMoveToDialog {
    pub fn new() -> Self {
        Self {
            sources: None, 
            dest: None,
            tab_id: None,
            show_modal: false,
        }
    }

    pub fn close(&mut self) {
        self.show_modal = false; 
    }

    pub fn open(&mut self, sources: Vec<PathBuf>, dest: PathBuf, tab_id: Uuid) {
        self.sources = Some(sources);
        self.dest = Some(dest);
        self.tab_id = Some(tab_id);
        self.show_modal = true;
    }


    pub fn render_dialog(&mut self, ctx: &Context) {
        let mut should_close = false;

        let (Some(sources), Some(dest), Some(tab_id)) = (self.sources.as_ref(), self.dest.as_ref(), self.tab_id.as_ref()) else { return; };
        
        let custom_frame = Frame::NONE
            .fill(Color32::from_rgb(16, 21, 25))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(10));

        Window::new("Mover...")
            .frame(custom_frame)
            .order(Order::Foreground)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut self.show_modal)
            .show(ctx, |ui|{
                ui.set_min_width(250.0);
                ui.set_min_height(100.0);
                
                ui.heading("¿Deseas mover...");

                const MAX_SHOWN: usize = 5;
                let total = sources.len();

                if total <= MAX_SHOWN {
                    for source in sources {
                        let file_name = source.file_name()
                            .map(|f| f.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "Archivo".to_string());

                        ui.label(format!("• {}", file_name));
                    }
                } else {
                    for source in sources {
                        let file_name = source.file_name()
                            .map(|f| f.to_string_lossy().into_owned())
                            .unwrap_or_else(|| "Archivo".to_string());

                        ui.label(format!("• {}", file_name));
                    }
                    ui.label(
                        RichText::new(format!("...y {} archivos más", total - MAX_SHOWN))
                            .weak()
                            .italics(),
                    );
                }

                let dest_name = dest.file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_else(|| dest.to_string_lossy().into_owned());

                ui.add_space(8.0);
                ui.label(format!("a {}", dest_name));

                ui.add_space(50.0);
                
                ui.horizontal(|ui| {
                    let width = ui.available_width();
                    let button_width = 120.0;
                    let spacing = (width - button_width * 2.0) / 3.0;

                    ui.add_space(spacing);
                    if ui.button("Cancelar").clicked() {
                        should_close = true;
                    }

                    ui.add_space(spacing);
                    if ui.button("Aceptar").clicked() {

                        with_active_sender(|sender| {
                            sender.send_fileop(FileOperation::Move { 
                                files: sources.to_vec(), 
                                dest: dest.to_path_buf(),
                                tab_id: *tab_id,
                            }).ok();
                        });

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