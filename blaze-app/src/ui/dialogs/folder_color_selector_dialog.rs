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





use egui::{Color32, Context, CornerRadius, Frame, Margin, Order, Window};
use file_id::FileId;
use tracing::info;
use crate::{core::system::{cache::cache_manager, clipboard::TOKIO_RUNTIME}, ui::blaze_ui_state::ModalDialog, utils::channel_pool::{FileOperation, with_active_sender}};


pub struct FolderColorSelector {
    pub folder_id: Option<FileId>,
    pub show_modal: bool,
    temp_color: Option<Color32>,
}

impl ModalDialog for FolderColorSelector {
    fn is_open(&self) -> bool { self.show_modal }
    fn close(&mut self) { self.close() }
    fn render(&mut self, ctx: &Context) { self.render_dialog(ctx); }
}

impl FolderColorSelector {
    pub fn new() -> Self {
        Self {
            folder_id: None,
            show_modal: false,
            temp_color: None,
        }
    }

    pub fn close(&mut self) {
        self.show_modal = false; 
    }

    pub fn open(&mut self, folder_id: FileId) {
        let cm = cache_manager::CacheManager::global();
        let initial_color = cm.get_cached_color(&folder_id);

        self.temp_color = Some(initial_color);
        self.folder_id = Some(folder_id);
        self.show_modal = true;
    }


    pub fn render_dialog(&mut self, ctx: &Context) {
        let mut should_close = false;

        let Some(folder_id) = self.folder_id.as_ref() else { return; };
        let Some(temp_color) = &mut self.temp_color else { return; };
        
        let custom_frame = Frame::NONE
            .fill(Color32::from_rgb(16, 21, 25))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(10));

        Window::new("Selecciona un color")
            .frame(custom_frame)
            .order(Order::Foreground)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut self.show_modal)
            .show(ctx, |ui|{
                ui.set_min_width(250.0);
                ui.set_min_height(100.0);

                let mut rgb: [f32; 3] = [
                    temp_color.r() as f32 / 255.0,
                    temp_color.g() as f32 / 255.0,
                    temp_color.b() as f32 / 255.0,
                ];

                ui.color_edit_button_rgb(&mut rgb);

                *temp_color = Color32::from_rgb(
                    (rgb[0] * 255.0) as u8,
                    (rgb[1] * 255.0) as u8,
                    (rgb[2] * 255.0) as u8,
                );

                ui.add_space(50.0);
                
                ui.horizontal(|ui| {
                    let width = ui.available_width();
                    let button_width = 110.0;
                    let spacing = (width - button_width * 3.0) / 4.0;

                    ui.add_space(spacing);

                    if ui.button("Restaurar predeterminado").clicked() {
                        *temp_color = Color32::YELLOW;
                    }
                    
                    if ui.button("Cancelar").clicked() {
                        should_close = true;
                    }

                    if ui.button("Aceptar").clicked() {
                        let cm = cache_manager::CacheManager::global();
                        let (folder_id, temp_color) = (*folder_id, *temp_color);


                        TOKIO_RUNTIME.spawn(async move {
                            cm.update_color_cache(folder_id, temp_color).await;
                            cm.reload_color_cache().await;
                        });
                        

                        ui.ctx().request_repaint();
                        should_close = true;
                    }
                });
            });

        if should_close {
            self.close();
        }
    }
}