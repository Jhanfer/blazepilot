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

use crate::ui::image_preview::image_preview_handler::ImagePreviewState;
use crate::ui::themes::platform::structs::ToColor;
use crate::ui::{dialog_manager::manager::ModalDialog, themes::theme_manager::with_theme};
use egui::{
    Button, Color32, CornerRadius, Frame, Key, Margin, Order, Rect, Sense, Stroke, Ui, Vec2,
    Window, pos2, vec2,
};

pub struct ImagePreviewDialog {
    pub preview: Option<ImagePreviewState>,
    pub show_modal: bool,
    needs_initial_load: bool,
}

impl ModalDialog for ImagePreviewDialog {
    fn is_open(&self) -> bool {
        self.show_modal
    }
    fn close(&mut self) {
        self.close()
    }
    fn render(&mut self, ui: &mut Ui) -> bool {
        self.render_dialog(ui)
    }
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

    fn render_preview(
        ui: &mut Ui,
        preview: &mut ImagePreviewState,
        needs_initial_load: &mut bool,
        should_close: &mut bool,
    ) {
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

        let image_area = Rect::from_min_size(ui.cursor().min, available);

        let img_resp = ui.allocate_rect(image_area, Sense::click_and_drag());

        let fit_zoom = (available.x / tex_size.x).min(available.y / tex_size.y);

        let min_zoom = fit_zoom * 0.5;
        let max_zoom = 20.0;

        if preview.zoom <= 0.0 {
            preview.zoom = fit_zoom;
        } else {
            preview.zoom = preview.zoom.clamp(min_zoom, max_zoom);
        }

        let dest_size = tex_size * preview.zoom;

        let max_pan_x = (dest_size.x - available.x).max(0.0) / 2.0;
        let max_pan_y = (dest_size.y - available.y).max(0.0) / 2.0;

        preview.offset.x = preview.offset.x.clamp(-max_pan_x, max_pan_x);
        preview.offset.y = preview.offset.y.clamp(-max_pan_y, max_pan_y);

        let dest_rect = Rect::from_center_size(image_area.center() + preview.offset, dest_size);

        if img_resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);

            if scroll != 0.0 {
                let zoom_factor = 1.0 + (scroll * 0.003);
                let new_zoom = (preview.zoom * zoom_factor).clamp(min_zoom, max_zoom);

                if (new_zoom - preview.zoom).abs() > 0.001 {
                    if let Some(cursor_pos) = img_resp.hover_pos() {
                        let current_center = image_area.center() + preview.offset;
                        let relative_cursor = cursor_pos - current_center;

                        let zoom_ratio = new_zoom / preview.zoom;
                        let new_relative_cursor = relative_cursor * zoom_ratio;

                        preview.offset.x += relative_cursor.x - new_relative_cursor.x;
                        preview.offset.y += relative_cursor.y - new_relative_cursor.y;

                        preview.zoom = new_zoom;
                    } else {
                        preview.zoom = new_zoom;
                    }
                }
            }

            if img_resp.dragged() {
                ui.set_cursor_icon(egui::CursorIcon::Grabbing);
                let drag = img_resp.drag_delta();
                preview.offset.x += drag.x;
                preview.offset.y += drag.y;
            }

            preview.offset.x = preview.offset.x.clamp(-max_pan_x, max_pan_x);
            preview.offset.y = preview.offset.y.clamp(-max_pan_y, max_pan_y);
        }

        if img_resp.double_clicked() {
            preview.zoom = fit_zoom;
            preview.offset = Vec2::ZERO;
        }

        let uv_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));

        ui.painter().with_clip_rect(image_area).image(
            texture.id(),
            dest_rect,
            uv_rect,
            Color32::WHITE,
        );

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
                preview.zoom = fit_zoom;
                preview.offset = Vec2::ZERO;
            }
        });

        let input = ui.input(|i| i.clone());
        if input.key_pressed(Key::ArrowRight) {
            preview.zoom = 0.0;
            preview.offset = Vec2::ZERO;
            preview.next(ui);
        }
        if input.key_pressed(Key::ArrowLeft) {
            preview.zoom = 0.0;
            preview.offset = Vec2::ZERO;
            preview.prev(ui);
        }
        if input.key_pressed(Key::Escape) {
            *should_close = true;
        }
    }

    pub fn render_dialog(&mut self, ui: &mut Ui) -> bool {
        let current_theme = with_theme(|t| t.current());

        let mut should_close = false;
        let Some(pvw) = self.preview.as_mut() else {
            return false;
        };

        let custom_frame = Frame::NONE
            .fill(Color32::TRANSPARENT)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(10));

        let screen_size = ui.content_rect().size();
        let max_window_size = vec2(screen_size.x * 0.9, screen_size.y * 0.9);

        let window = Window::new(pvw.current_name())
            .frame(custom_frame)
            .order(Order::Foreground)
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut self.show_modal)
            .default_size(max_window_size)
            .resizable(false)
            .max_size(max_window_size);

        window.show(ui, |ui| {
            let available_width = ui.available_width();
            let available_height = ui.available_height();
            let preview_height = (available_height * 0.7).max(200.0);

            Frame::NONE
                .fill(pvw.background_color.linear_multiply(0.8))
                .stroke(Stroke::new(0.5, current_theme.accent_glow.to_color()))
                .corner_radius(8.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.set_height(preview_height);
                    ui.set_width(available_width - 20.0);

                    ui.allocate_ui(vec2(ui.available_width(), preview_height), |ui| {
                        ui.centered_and_justified(|ui| {
                            Self::render_preview(
                                ui,
                                pvw,
                                &mut self.needs_initial_load,
                                &mut should_close,
                            );
                        });
                    });

                    ui.horizontal(|ui| {
                        let button_width = 50.0;
                        let button_spacing = 20.0;

                        ui.spacing_mut().item_spacing.x = button_spacing;

                        let total_width = button_width * 2.0 + button_spacing;
                        let spacing = (ui.available_width() - total_width) / 2.0;

                        ui.add_space(spacing.max(0.0));

                        if ui
                            .add(Button::new("◀").min_size(vec2(button_width, 30.0)))
                            .clicked()
                        {
                            pvw.zoom = 0.0;
                            pvw.offset = Vec2::ZERO;
                            pvw.prev(ui);
                        }

                        if ui
                            .add(Button::new("▶").min_size(vec2(button_width, 30.0)))
                            .clicked()
                        {
                            pvw.zoom = 0.0;
                            pvw.offset = Vec2::ZERO;
                            pvw.next(ui);
                        }
                    });
                });
        });

        if should_close {
            pvw.cleanup();
        }

        should_close
    }
}
