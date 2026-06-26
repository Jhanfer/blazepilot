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
use crate::core::system::knowndirs::knowndirs_manager::KnownDirsManager;
use crate::ui::blaze_ui_state::BlazeUiState;
use crate::ui::modules::ui_callback::connect_ui_components_callback;
use crate::ui::themes::platform::structs::ToColor;
use crate::ui::themes::theme_manager::with_theme;
use eframe::Frame;
use egui::{FontData, FontDefinitions, FontFamily, Ui};
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, error};

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
    pub fn build(self) -> BlazeApp {
        let state = TOKIO_RUNTIME.block_on(
            BlazeCoreBuilder::default()
                .with_start_path(self.start_path)
                .build(),
        );
        let ui_state = BlazeUiState::default();

        BlazeApp { state, ui_state }
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

        //Dropeo de archivos
        if !ui.ctx().input(|i| i.raw.dropped_files.is_empty()) {
            debug!("Se dropea el objeto");
            let dropped_files: Vec<Arc<Path>> = ui
                .input(|i| i.raw.dropped_files.clone())
                .iter()
                .map(|d| {
                    let path_buf = &d.path.clone().unwrap_or_default();
                    let path: &Path = path_buf.as_ref();
                    Arc::from(path)
                })
                .collect();

            let dest = self.state.cwd.clone();

            self.state.move_files(dropped_files, dest);

            ui.input_mut(|i| i.raw.dropped_files.clear());
        }

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
