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






use egui::{Button, Color32, CornerRadius, Frame, Key, Margin, Order, Rect, ScrollArea, Sense, Ui, Vec2, Window, pos2, vec2};
use crate::{ui::{blaze_ui_state::ModalDialog, image_preview::image_preview::ImagePreviewState}};


pub struct ImagePreviewDialog {
    pub preview: Option<ImagePreviewState>,
    pub show_modal: bool,
    needs_initial_load: bool,
}

impl ModalDialog for ImagePreviewDialog {
    fn is_open(&self) -> bool { self.show_modal }
    fn close(&mut self) { self.close() }
    fn render(&mut self, ui: &mut Ui) { self.render_dialog(ui); }
}

impl ImagePreviewDialog {
    pub fn new() -> Self {
        Self {
            preview: None,
            show_modal: false,
            needs_initial_load: false,
        }
    }

    pub fn close(&mut self) {
        self.show_modal = false; 
    }

    pub fn open(&mut self, img_pvw: ImagePreviewState) {
        self.preview = Some(img_pvw);
        self.show_modal = true;
        self.needs_initial_load = true;
    }


    fn render_preview(ui: &mut Ui, preview: &mut ImagePreviewState, needs_initial_load: &mut bool, should_close: &mut bool) {
        if *needs_initial_load {
            preview.initial_load(ui);
            *needs_initial_load = false;
        }

        preview.poll_loading(ui);

        if preview.loading {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.spinner();
                ui.add_space(20.0);
                ui.label("Cargando imagen...");
            });
            return;
        }

        let Some(texture) = &preview.current_texture else {
            ui.centered_and_justified(|ui| ui.label("No se pudo cargar la imagen"));
            return;
        };

        let tex_size = texture.size_vec2();
        let available = ui.available_size();

        if tex_size.x <= 0.0 || tex_size.y <= 0.0 {
            return;
        }

        let fit_zoom = (available.x / tex_size.x).min(available.y / tex_size.y);
        let min_zoom = fit_zoom.min(1.0);
        let max_zoom = 12.0;

        preview.zoom = preview.zoom.clamp(min_zoom, max_zoom);

        let displayed_size = tex_size * preview.zoom;

        let image_area = Rect::from_min_size(
            ui.cursor().min,
            available
        );

        let img_resp = ui.allocate_rect(image_area, Sense::click_and_drag());

        let max_offset_x = (displayed_size.x - available.x).max(0.0) / displayed_size.x;
        let max_offset_y = (displayed_size.y - available.y).max(0.0) / displayed_size.y;
        
        preview.offset.x = preview.offset.x.clamp(0.0, max_offset_x);
        preview.offset.y = preview.offset.y.clamp(0.0, max_offset_y);

        let uv_rect = Rect::from_min_max(
            pos2(preview.offset.x, preview.offset.y),
            pos2(
                (preview.offset.x + available.x / displayed_size.x).min(1.0),
                (preview.offset.y + available.y / displayed_size.y).min(1.0)
            ),
        );


        ui.painter().image(
            texture.id(),
            image_area,
            uv_rect,
            egui::Color32::WHITE,
        );

        if img_resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);

            if scroll != 0.0 {
                let zoom_factor = 1.0 + (scroll * 0.003);
                let new_zoom = (preview.zoom * zoom_factor).clamp(min_zoom, max_zoom);

                if (new_zoom - preview.zoom).abs() > 0.001  {
                    if let Some(cursor_pos) = img_resp.hover_pos() {
                        let cursor_ratio_x = (cursor_pos.x - image_area.min.x) / available.x;
                        let cursor_ratio_y = (cursor_pos.y - image_area.min.y) / available.y;
                        
                        let old_world_x = (preview.offset.x + cursor_ratio_x) * tex_size.x * preview.zoom;
                        let old_world_y = (preview.offset.y + cursor_ratio_y) * tex_size.y * preview.zoom;
                        
                        preview.zoom = new_zoom;
                        
                        let new_uv_x = old_world_x / (tex_size.x * preview.zoom) - cursor_ratio_x;
                        let new_uv_y = old_world_y / (tex_size.y * preview.zoom) - cursor_ratio_y;
                        
                        preview.offset.x = new_uv_x.clamp(0.0, max_offset_x);
                        preview.offset.y = new_uv_y.clamp(0.0, max_offset_y);
                    } else {
                        preview.zoom = new_zoom;
                    }
                }
            } 

            if img_resp.dragged() {
                ui.set_cursor_icon(egui::CursorIcon::Grabbing);
                let drag = img_resp.drag_delta();
                preview.offset.x -= drag.x / displayed_size.x;
                preview.offset.y -= drag.y / displayed_size.y;
            }
        }

        if img_resp.double_clicked() {
            preview.zoom = min_zoom;
            preview.offset = Vec2::ZERO;

            let image_displayed = tex_size * preview.zoom;

            if image_displayed.x > available.x {
                preview.offset.x = (image_displayed.x - available.x) / (2.0 * image_displayed.x);
            }

            if image_displayed.y > available.y {
                preview.offset.y = (image_displayed.y - available.y) / (2.0 * image_displayed.y);
            }
        }

        ui.add_space(8.0);
        ui.horizontal_centered(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;

            if ui.button("-").clicked() {
                preview.zoom = (preview.zoom * 0.75).max(min_zoom);
            }

            ui.label(format!("{:.0}%", preview.zoom * 100.0));

            if ui.button("+").clicked() {
                preview.zoom = (preview.zoom * 1.3).min(max_zoom);
            }

            if ui.button("Reset").clicked() {
                preview.zoom = min_zoom;
                preview.offset = Vec2::ZERO;
            }
        });

        let input = ui.input(|i| i.clone());
        if input.key_pressed(Key::ArrowRight) {
            preview.zoom = min_zoom;
            preview.offset = Vec2::ZERO;
            preview.next(ui);
        }
        if input.key_pressed(Key::ArrowLeft) {
            preview.zoom = min_zoom;
            preview.offset = Vec2::ZERO;
            preview.prev(ui);
        }
        if input.key_pressed(Key::Escape) {
            *should_close = true;
        }
    }


    pub fn render_dialog(&mut self, ui: &mut Ui) {
        let mut should_close = false;
        let Some(pvw) = self.preview.as_mut() else { return; };
        
        let is_portrait = pvw.current_texture.as_ref().map_or(false, |tex| {
            let size = tex.size_vec2();
            size.y > size.x * 1.5
        });

        let is_landscape = pvw.current_texture.as_ref().map_or(false, |tex| {
            let size = tex.size_vec2();
            size.x > size.y * 1.5
        });


        let custom_frame = Frame::NONE
            .fill(Color32::from_rgb(16, 21, 25))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(10));


        let mut window = Window::new(pvw.current_name())
            .frame(custom_frame)
            .order(Order::Foreground)
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut self.show_modal);

        if is_portrait {
            window = window.default_size([400.0, 700.0]);
        } else if is_landscape {
            window = window.default_size([900.0, 500.0]);
        } else {
            window = window.default_size([600.0, 600.0]);
        }

        window.show(ui, |ui|{
            let available_width = ui.available_width();
            let available_height = ui.available_height();
            ui.set_min_width(300.0);
            ui.set_min_height(300.0);

            let preview_height = (available_height * 0.7).max(200.0);

            ScrollArea::vertical()
                .max_height(available_height)
                .show(ui, |ui|{
                    Frame::NONE
                        .fill(Color32::from_rgb(10, 15, 19))
                        .corner_radius(8.0)
                        .inner_margin(10.0)
                        .show(ui, |ui|{
                            ui.set_height(preview_height);
                            ui.set_width(available_width - 20.0);

                            ui.centered_and_justified(|ui|{
                                Self::render_preview(ui, pvw, &mut self.needs_initial_load, &mut should_close);
                            });
                    });
            });

            ui.add_space(10.0);

            ui.horizontal_centered(|ui| {
                ui.spacing_mut().item_spacing.x = 20.0;

                if ui.add(Button::new("◀").min_size(vec2(120.0, 42.0))).clicked() {
                    pvw.zoom = 1.0;
                    pvw.offset = Vec2::ZERO;
                    pvw.prev(ui);
                }

                if ui.add(Button::new("▶").min_size(vec2(120.0, 42.0))).clicked() {
                    pvw.zoom = 1.0;
                    pvw.offset = Vec2::ZERO;
                    pvw.next(ui);
                }
            });
            
            ui.horizontal(|ui| {
                let width = ui.available_width();
                let button_width = 120.0;
                let spacing = (width - button_width) / 2.0;
                
                ui.add_space(spacing);
                if ui.add(Button::new("Cerrar").min_size(vec2(button_width, 42.0))).clicked() {
                    should_close = true;
                }
            });
            
            ui.add_space(20.0);
        });

        if should_close {
            pvw.cleanup();
            self.close();
        }
    }
}