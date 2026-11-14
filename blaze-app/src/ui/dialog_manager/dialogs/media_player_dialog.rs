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
use crate::ui::custom_components::UiExt;
use crate::ui::dialog_manager::{
    dialogs::{DialogButtonKind, dialog_button_sized, dialog_frame},
    manager::ModalDialog,
};
use crate::ui::icons_cache::icons::ICON_MUSIC;
use crate::ui::media_player::media_player_backend::MediaPlayer;
use crate::ui::themes::{platform::palette::SectionPalette, theme_manager::with_theme};
use crate::utils::formating::format_hms;
use egui::{
    Align2, Color32, Frame, Key, Order, Rect, RichText, Sense, Stroke, Ui, Vec2, Window, pos2, vec2,
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
        self.media_player.stop();
        self.has_started = false;
    }

    pub fn open(&mut self, media_path: Arc<Path>, media_name: Box<str>, is_audio_only: bool) {
        let mut buf = [0u8; 32];
        match File::open(&media_path) {
            Ok(mut file) => match file.read(&mut buf) {
                Ok(_) => {
                    let file_ext = sniff_magic_bytes(&buf).unwrap_or(FileExtension::Unknown);

                    if !file_ext.is_audio() && !file_ext.is_video() {
                        warn!("No es un media file");
                        self.close();
                        return;
                    }
                }
                Err(e) => {
                    warn!("Ha ocurrido un error al intentear leer el file: {e}");
                    self.close();
                    return;
                }
            },
            Err(e) => {
                warn!("Ha ocurrido un error al intentear abrir el file: {e}");
                self.close();
                return;
            }
        }

        self.show_modal = true;
        self.media_path = Some(media_path);
        self.media_name = Some(media_name);
        self.is_audio_only = is_audio_only;
    }

    fn render_video(&mut self, ui: &mut Ui, path: Arc<Path>, palette: SectionPalette) {
        if !self.has_started {
            self.media_player.load_path(path);

            if !self.is_audio_only {
                self.media_player.play_video_audio();
            } else {
                self.media_player.play_audio();
            }

            self.has_started = true;
        }

        // icono de audio centrado en modo auidio
        if self.is_audio_only {
            ui.centered_and_justified(|ui| {
                ui.draw_icon(("media_player_audio", ICON_MUSIC))
                    .size(vec2(96.0, 96.0))
                    .tint(palette.icon_secondary)
                    .show();
            });
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
    }

    fn render_controls(&mut self, ui: &mut Ui, palette: SectionPalette, should_close: &mut bool) {
        let (elapsed, media_duration) = {
            let clock = self.media_player.clock.lock();
            (clock.elapsed(), clock.media_duration.unwrap_or(0.0))
        };

        if !self.timeline_dragging {
            self.timeline = elapsed;
        }

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 10.0;

            ui.label_ns(RichText::new(format_hms(elapsed as u32)).color(palette.fg_secondary));

            let slider_width = (ui.available_width() - 90.0).max(80.0);

            let response = ui
                .blaze_slider(&mut self.timeline, 0.0..=media_duration.max(0.1))
                .size((slider_width, 10.0))
                .background_color(palette.bg_hover)
                .accent(palette.accent)
                .handle_color(palette.fg_primary)
                .handle_stroke_color(palette.accent)
                .show();

            // Mantiene el valor mientras se pulsa
            if response.drag_started() || response.is_pointer_button_down_on() {
                self.timeline_dragging = true;
            }

            // click simple hace seek
            if response.drag_stopped() || response.clicked() {
                self.timeline_dragging = false;
                self.media_player.seek(self.timeline);
            }

            ui.label_ns(
                RichText::new(format_hms(media_duration as u32)).color(palette.fg_secondary),
            );
        });

        ui.add_space(6.0);

        // Fila de controles centrada
        let is_playing = self.media_player.clock.lock().is_playing();
        let play_label = if is_playing { "⏸" } else { "▶" };

        ui.horizontal(|ui| {
            let button_width = 56.0;
            let button_height = 30.0;
            let button_spacing = 14.0;

            let volume_label_width = 30.0;
            let volume_slider_width = 140.0;
            let volume_pct_width = 42.0;

            let controls_width = button_width * 3.0 + button_spacing * 2.0;
            let total_width = controls_width
                + button_spacing
                + volume_label_width
                + volume_slider_width
                + volume_pct_width
                + button_spacing * 2.0;

            let leading_space = ((ui.available_width() - total_width) / 2.0).max(0.0);
            ui.add_space(leading_space);

            ui.spacing_mut().item_spacing.x = button_spacing;

            if dialog_button_sized(
                ui,
                "-5s",
                &palette,
                DialogButtonKind::Secondary,
                (button_width, button_height),
            )
            .show()
            .clicked()
            {
                self.media_player.seek_5s_back();
                self.timeline = self.media_player.clock.lock().elapsed();
            }

            if dialog_button_sized(
                ui,
                play_label,
                &palette,
                DialogButtonKind::Primary,
                (button_width, button_height),
            )
            .show()
            .clicked()
            {
                self.media_player.toggle_pause();
            }

            if dialog_button_sized(
                ui,
                "+5s",
                &palette,
                DialogButtonKind::Secondary,
                (button_width, button_height),
            )
            .show()
            .clicked()
            {
                self.media_player.seek_5s_forward();
                self.timeline = self.media_player.clock.lock().elapsed();
            }

            ui.spacing_mut().item_spacing.x = 8.0;

            ui.label_ns(RichText::new("Vol").color(palette.fg_secondary));

            ui.blaze_slider(&mut self.target_volume, 0.0..=1.0)
                .size((volume_slider_width, 10.0))
                .background_color(palette.bg_hover)
                .accent(palette.accent)
                .handle_color(palette.fg_primary)
                .handle_stroke_color(palette.accent)
                .show();

            ui.label_ns(
                RichText::new(format!("{:.0}%", self.target_volume * 100.0))
                    .color(palette.fg_secondary),
            );
        });

        // Suavizado del volumen
        let speed = 3.0;
        let dt = ui.input(|i| i.stable_dt);

        self.current_volume += (self.target_volume - self.current_volume) * speed * dt;
        self.current_volume = self.current_volume.clamp(0.0, 1.0);
        self.media_player.volume(self.current_volume);

        if ui.input(|i| i.key_pressed(Key::Space)) {
            self.media_player.toggle_pause();
        }
        if ui.input(|i| i.key_pressed(Key::ArrowRight)) {
            self.media_player.seek_5s_forward();
            self.timeline = self.media_player.clock.lock().elapsed();
        }
        if ui.input(|i| i.key_pressed(Key::ArrowLeft)) {
            self.media_player.seek_5s_back();
            self.timeline = self.media_player.clock.lock().elapsed();
        }
        if ui.input(|i| i.key_pressed(Key::Escape)) {
            *should_close = true;
        }
    }

    pub fn render_dialog(&mut self, ui: &mut Ui) -> bool {
        let current_theme = with_theme(|t| t.current());
        let palette = current_theme.dialogs.palette(&current_theme.global);

        let (Some(path), Some(media_name)) = (self.media_path.as_ref(), self.media_name.as_ref())
        else {
            return false;
        };

        let screen_size = ui.content_rect().size();
        let max_window_size = vec2(screen_size.x * 0.9, screen_size.y * 0.9);

        let was_open = self.show_modal;
        let mut show_modal = was_open;
        let mut should_close = false;

        let window = Window::new(RichText::new(media_name.to_string()).color(palette.fg_primary))
            .frame(dialog_frame(
                &palette,
                current_theme.global.panel_stroke_width,
            ))
            .order(Order::Foreground)
            .collapsible(false)
            .resizable(true)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .open(&mut show_modal)
            .fixed_size(max_window_size);

        let path = path.clone();
        let stroke_width = current_theme.global.panel_stroke_width;

        window.show(ui, |ui| {
            let available_width = ui.available_width();
            let available_height = ui.available_height();
            let preview_height = (available_height - 110.0).max(160.0);

            Frame::NONE
                .fill(palette.bg_panel)
                .stroke(Stroke::new(stroke_width, palette.border_panel))
                .corner_radius(8.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.set_width(available_width - 20.0);

                    ui.allocate_ui(vec2(ui.available_width(), preview_height), |ui| {
                        ui.centered_and_justified(|ui| {
                            self.render_video(ui, path, palette);
                        });
                    });

                    self.render_controls(ui, palette, &mut should_close);
                });
        });

        if should_close || (was_open && !show_modal) {
            self.close();
        }

        should_close
    }
}
