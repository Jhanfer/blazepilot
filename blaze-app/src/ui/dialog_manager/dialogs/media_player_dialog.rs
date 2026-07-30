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

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;

use crate::core::files::file_extension::{FileExtension, sniff_magic_bytes};
use crate::ui::media_player::media_player_backend::MediaPlayer;
use crate::ui::themes::platform::structs::ToColor;
use crate::ui::{dialog_manager::manager::ModalDialog, themes::theme_manager::with_theme};
use crate::utils::formating::format_hms;
use egui::{
    Align2, Color32, CornerRadius, Frame, Key, Margin, Order, Rect, Sense, Slider, SliderClamping,
    Stroke, Ui, Vec2, Window, pos2, vec2,
};
use tracing::warn;

pub struct MediaPlayerDialog {
    pub show_modal: bool,
    media_player: MediaPlayer,
    current_volume: f32,
    target_volume: f32,
    timeline: f32,
    timeline_dragging: bool,
    media_path: Option<Arc<Path>>,
    media_name: Option<Box<str>>,
    has_started: bool,
    is_audio_only: bool,
}

impl ModalDialog for MediaPlayerDialog {
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

impl MediaPlayerDialog {
    pub fn new() -> Self {
        Self {
            show_modal: false,
            media_player: MediaPlayer::init(),
            current_volume: 1.0,
            target_volume: 1.0,
            timeline: 0.0,
            timeline_dragging: false,
            media_path: None,
            media_name: None,
            has_started: false,
            is_audio_only: false,
        }
    }

    pub fn close(&mut self) {
        self.show_modal = false;
        if self.is_playing() {
            self.media_player.stop();
        }
        self.has_started = false;
    }

    pub fn open(&mut self, media_path: Arc<Path>, media_name: Box<str>, is_audio_only: bool) {
        self.show_modal = true;
        self.media_path = Some(media_path);
        self.media_name = Some(media_name);
        self.is_audio_only = is_audio_only;
    }

    fn is_playing(&self) -> bool {
        self.media_player.is_playing()
    }

    fn render_preview(&mut self, ui: &mut Ui, path: Arc<Path>, should_close: &mut bool) {
        if !self.has_started {
            self.media_player.load_path(path);

            if !self.is_audio_only {
                self.media_player.play_video_audio();
            } else {
                self.media_player.play_audio();
            }

            self.has_started = true;
        }

        let mut action = false;

        self.media_player.update(ui, &mut |tex, ui, (w, h)| {
            let available = ui.available_size();

            if w == 0 || h == 0 {
                return;
            }

            let aspect = w as f32 / h as f32;

            let (width, height) = if available.x / available.y > aspect {
                (available.y * aspect, available.y)
            } else {
                (available.x, available.x / aspect)
            };

            let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click());

            if resp.clicked() {
                action = true;
            }

            let uv_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));

            ui.painter().image(tex.id(), rect, uv_rect, Color32::WHITE);
        });

        if action {
            self.media_player.toggle_pause();
        }

        {
            let (elapse, media_duration) = {
                let clock = self.media_player.clock.lock();
                (clock.elapsed(), clock.media_duration.unwrap_or(0.1))
            };

            if !self.timeline_dragging {
                self.timeline = elapse;
            }

            ui.horizontal_centered(|ui| {
                let response = ui.add(
                    egui::Slider::new(&mut self.timeline, 0.0..=media_duration)
                        .clamping(SliderClamping::Always)
                        .show_value(false)
                        .text("Timeline"),
                );

                if response.drag_started() {
                    eprintln!("DragStarted");
                    self.timeline_dragging = true;
                }

                if response.drag_stopped() {
                    self.timeline_dragging = false;
                    eprintln!("DragStopped: {}", self.timeline);
                    self.media_player.seek(self.timeline);
                }

                ui.add_space(10.0);

                ui.colored_label(
                    egui::Color32::RED,
                    format!(
                        "{} / {}",
                        format_hms(elapse as u32),
                        format_hms(media_duration as u32)
                    ),
                );

                ui.add_space(20.0);

                ui.add(
                    Slider::new(&mut self.target_volume, 0.0..=1.0)
                        .clamping(SliderClamping::Always)
                        .text("Volumen"),
                );

                let speed = 3.0;
                let dt = ui.input(|i| i.stable_dt);

                self.current_volume += (self.target_volume - self.current_volume) * speed * dt;

                self.media_player.volume(self.current_volume);

                ui.request_repaint();
            });
        }

        let input = ui.input(|i| i.clone());
        if input.key_pressed(Key::ArrowRight) {
            self.media_player.seek_5s_forward();
        }
        if input.key_pressed(Key::ArrowLeft) {
            self.media_player.seek_5s_back();
        }
        if input.key_pressed(Key::Escape) {
            self.close();
            *should_close = true;
        }
    }

    pub fn render_dialog(&mut self, ui: &mut Ui) -> bool {
        let current_theme = with_theme(|t| t.current());

        let (Some(path), Some(media_name)) = (self.media_path.as_ref(), self.media_name.as_ref())
        else {
            return false;
        };

        let mut buf = [0u8; 32];
        match File::open(path) {
            Ok(mut file) => match file.read(&mut buf) {
                Ok(_) => {
                    let file_ext = sniff_magic_bytes(&buf).unwrap_or(FileExtension::Unknown);

                    if !file_ext.is_audio() && !file_ext.is_video() {
                        warn!("No es un media file");
                        self.close();
                        return false;
                    }
                }
                Err(e) => {
                    warn!("Ha ocurrido un error al intentear leer el file: {e}");
                    self.close();
                    return false;
                }
            },
            Err(e) => {
                warn!("Ha ocurrido un error al intentear abrir el file: {e}");
                self.close();
                return false;
            }
        }

        let custom_frame = Frame::NONE
            .fill(Color32::TRANSPARENT)
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(10));

        let screen_size = ui.content_rect().size();
        let max_window_size = vec2(screen_size.x * 0.9, screen_size.y * 0.9);

        let was_open = self.show_modal;
        let mut show_modal = was_open;
        let mut should_close = false;

        let window = Window::new(media_name)
            .frame(custom_frame)
            .order(Order::Foreground)
            .collapsible(false)
            .resizable(true)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .open(&mut show_modal)
            .fixed_size(max_window_size);

        let path = path.clone();

        window.show(ui, |ui| {
            let available_height = ui.available_height();
            let preview_height = (available_height * 0.9).max(200.0);

            Frame::NONE
                .fill(current_theme.semantic.bg_container.to_color())
                .stroke(Stroke::new(
                    0.5,
                    current_theme.semantic.accent_glow.to_color(),
                ))
                .corner_radius(8.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.allocate_ui(vec2(ui.available_width(), preview_height), |ui| {
                        ui.centered_and_justified(|ui| {
                            self.render_preview(ui, path, &mut should_close);
                        });
                    });

                    ui.horizontal(|ui| {
                        let button_width = 50.0;
                        let button_spacing = 20.0;

                        ui.spacing_mut().item_spacing.x = button_spacing;

                        let total_width = button_width * 2.0 + button_spacing;
                        let spacing = (ui.available_width() - total_width) / 2.0;

                        ui.add_space(spacing.max(0.0));
                    });
                });
        });

        if was_open && !show_modal {
            self.close();
        }

        self.show_modal = show_modal;

        should_close
    }
}
