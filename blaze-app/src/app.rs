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

use crate::core::blaze_state::{BlazeCoreBuilder, BlazeCoreState};
use crate::core::bootstrap::configs::config_manager::with_configs;
use crate::core::system::clipboard::global_clipboard::TOKIO_RUNTIME;
use crate::core::system::clipboard_text::keyboard_state::{KeyboardAction, with_keyboard_state};
use crate::core::system::clipboard_text::text_clipboard::with_text_clipboard;
use crate::core::system::knowndirs::knowndirs_manager::KnownDirsManager;
use crate::platform::wayland::wayland_dnd::WaylandDndReceiver;
use crate::platform::wayland::wayland_events::EventTrait;
use crate::platform::x11::x11_events::process_event_x11;
use crate::ui::blaze_ui_state::BlazeUiState;
use crate::ui::modules::ui_callback::connect_ui_components_callback;
use crate::ui::themes::platform::structs::ToColor;
use crate::ui::themes::theme_manager::with_theme;
use eframe::Frame;
use egui::{FontData, FontDefinitions, FontFamily, Ui};
use std::path::Path;
use std::sync::Arc;
use tracing::error;

#[must_use = "llama .build() para construir la aap"]
pub struct BlazeAppBuilder {
    pub start_path: Option<Arc<Path>>,
}

impl BlazeAppBuilder {
    fn new() -> Self {
        Self {
            start_path: Some(KnownDirsManager::get().home.clone()),
        }
    }

    pub fn with_start_path(mut self, path: Option<Arc<Path>>) -> Self {
        self.start_path = path;
        self
    }

    #[must_use]
    pub fn build(self, display_ptr: Option<*mut std::ffi::c_void>) -> BlazeApp {
        let dnd = display_ptr.and_then(WaylandDndReceiver::spawn);

        if let Some(dnd) = &dnd {
            let backend = crate::platform::wayland::clipboard_wayland::WaylandClipboard {
                copy_tx: Some(dnd.copy_tx.clone()),
                clipboard_text: Arc::clone(&dnd.clipboard_text),
            };
            with_text_clipboard(|c| c.init(backend));
        }

        let state = TOKIO_RUNTIME.block_on(
            BlazeCoreBuilder::default()
                .with_start_path(self.start_path)
                .build(),
        );
        let ui_state = BlazeUiState::default();

        BlazeApp {
            state,
            ui_state,
            dnd,
        }
    }
}

impl Default for BlazeAppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BlazeApp {
    pub state: BlazeCoreState,  //motor, archivos, mover
    pub ui_state: BlazeUiState, //visuales
    pub dnd: Option<WaylandDndReceiver>,
}

impl BlazeApp {
    pub fn set_up_custom_font(&self, ui: &mut Ui) {
        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "NotoSans".to_owned(),
            FontData::from_static(include_bytes!("./ui/assets/noto/NotoSans-Regular.ttf")).into(),
        );

        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "NotoSans".to_owned());

        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "NotoSans".to_owned());

        ui.set_fonts(fonts);
    }

    pub fn set_custom_visuals(&self, ui: &mut Ui) {
        let current_theme = with_theme(|t| t.current());
        ui.global_style_mut(|style| {
            let vmut = &mut style.visuals;

            let text_p = current_theme.text_primary.to_color();
            let text_s = current_theme.text_secondary.to_color();
            let border = current_theme.border_panel.to_color();

            // Inactive
            vmut.widgets.inactive.bg_fill = current_theme.main_buttons.to_color();
            vmut.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, border);
            vmut.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text_p);
            vmut.widgets.inactive.weak_bg_fill = current_theme.bg_container.to_color();

            // Hover
            vmut.widgets.hovered.bg_fill = current_theme.bg_hover.to_color();
            vmut.widgets.hovered.bg_stroke =
                egui::Stroke::new(1.0, current_theme.accent_glow.to_color());
            vmut.widgets.hovered.fg_stroke =
                egui::Stroke::new(1.0, current_theme.text_primary.to_color());
            vmut.widgets.hovered.weak_bg_fill = current_theme.bg_hover.to_color();

            // Selected
            vmut.widgets.active.bg_fill = current_theme.item_selected.to_color();
            vmut.widgets.active.bg_stroke = egui::Stroke::new(1.0, current_theme.accent.to_color());
            vmut.widgets.active.fg_stroke =
                egui::Stroke::new(1.0, current_theme.text_primary.to_color());
            vmut.widgets.active.weak_bg_fill = current_theme.item_selected.to_color();

            // ComboBox opened
            vmut.widgets.open.bg_fill = current_theme.bg_container.to_color();
            vmut.widgets.open.bg_stroke = egui::Stroke::new(1.0, border);
            vmut.widgets.open.fg_stroke = egui::Stroke::new(1.0, text_s);
            vmut.widgets.open.weak_bg_fill = current_theme.bg_container.to_color();

            vmut.selection.bg_fill = current_theme.rubberband.to_color();
            vmut.selection.stroke = egui::Stroke::new(1.0, current_theme.accent.to_color());

            vmut.extreme_bg_color = current_theme.bg_container.to_color();

            vmut.window_fill = current_theme.bg_panel.to_color();
            vmut.window_stroke = egui::Stroke::new(1.0, border);

            vmut.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, border);

            vmut.widgets.noninteractive.fg_stroke =
                egui::Stroke::new(1.0, current_theme.text_muted.to_color());

            vmut.widgets.noninteractive.weak_bg_fill = current_theme.bg_main.to_color();

            vmut.panel_fill = current_theme.bg_main.to_color();
            vmut.faint_bg_color = current_theme.bg_container.to_color();
            vmut.override_text_color = Some(current_theme.text_primary.to_color());
        });
    }
}

impl eframe::App for BlazeApp {
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        //limpiar keyboard_state
        with_keyboard_state(|k| k.clear());

        if let Some(ref receiver) = self.dnd {
            receiver.process_event(self.state.cwd.clone(), self.state.active_id);
        } else {
            // Por ahora x11 solo detecta dropeo de files pero no de bytes, por lo que esta condición no se cumplirá al soltar imágenes o texto plano
            if !ctx.input(|i| i.raw.dropped_files.is_empty()) {
                let dropped_files = ctx.input(|i| i.raw.dropped_files.clone());

                process_event_x11(self.state.cwd.clone(), self.state.active_id, &dropped_files);
            }
        }

        //simulador de evento ctrl + (c, v, x)
        for event in &raw_input.events {
            if let egui::Event::Key { key, modifiers, .. } = event {
                if *key == egui::Key::V && modifiers.ctrl {
                    if let Some(text) = with_text_clipboard(|c| c.paste()) {
                        with_keyboard_state(|k| {
                            k.set_action(
                                KeyboardAction::Paste(text.clone()),
                                ctx.cumulative_frame_nr(),
                            )
                        });
                        raw_input.events.push(egui::Event::Paste(text));
                    }
                    break;
                }

                if *key == egui::Key::C && modifiers.ctrl {
                    tracing::debug!("Ctrl + C");
                    with_keyboard_state(|k| {
                        k.set_action(KeyboardAction::Copy, ctx.cumulative_frame_nr())
                    });
                    break;
                }

                if *key == egui::Key::X && modifiers.ctrl {
                    tracing::debug!("Ctrl + X");
                    with_keyboard_state(|k| {
                        k.set_action(KeyboardAction::Cut, ctx.cumulative_frame_nr())
                    });
                    break;
                }
            }
        }
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        self.set_custom_visuals(ui);

        self.set_up_custom_font(ui);

        with_configs(|c| match c.tick() {
            Ok(_) => {}
            Err(e) => error!("Ha ocurrido un error de guardado: {e}."),
        });

        ui.options_mut(|opt| {
            opt.reduce_texture_memory = true;
        });

        self.state.process_messages();

        self.ui_state.dialog_manager.render_area(ui);
        self.ui_state.process_events();

        let files = self.state.get_active_files();
        connect_ui_components_callback(ui, &files, &mut self.state, &mut self.ui_state);

        if self.state.is_loading || self.state.active_tasks > 0 {
            ui.request_repaint();
        } else {
            ui.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }

    fn on_exit(&mut self) {
        with_configs(|c| match c.force_save() {
            Ok(_) => {}
            Err(e) => error!("Ha ocurrido un error de guardado: {e}."),
        });
        self.state.save_caches(true);
    }
}
