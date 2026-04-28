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
use eframe::Frame;
use egui::{FontData, FontDefinitions, FontFamily, Ui};
use tracing::{debug};
use crate::core::blaze_state::BlazeCoreState;
use crate::ui::blaze_ui_state::BlazeUiState;
use crate::ui::modules::ui_callback::connect_ui_components_callback;



pub struct BlazeApp {
    pub state: BlazeCoreState, //motor, archivos, mover
    pub ui_state: BlazeUiState, //visuales (qué item está hovereado, etc)
}


impl BlazeApp {
    pub fn set_up_custom_font(&self, ui: &mut Ui) {
        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "NotoSans".to_owned(),
            FontData::from_static(include_bytes!("./ui/assets/noto/NotoSans-Regular.ttf")).into()
        );

        fonts.families.entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "NotoSans".to_owned());
            
        fonts.families.entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "NotoSans".to_owned());
        
        ui.set_fonts(fonts);
    }
}


impl eframe::App for BlazeApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {

        self.set_up_custom_font(ui);

        //Dropeo de archivos
        if !ui.ctx().input(|i| i.raw.dropped_files.is_empty()) {
            debug!("Se dropea el objeto");
            let dropped_files: Vec<PathBuf> = ui.input(|i| i.raw.dropped_files.clone())
                .iter()
                .map(|d| d.path.clone().unwrap_or_default())
                .collect();

            let cwd = self.state.cwd.clone();
            self.state.move_files(dropped_files, cwd);

            ui.input_mut(|i| i.raw.dropped_files.clear());
        }


        self.state.process_messages();
        
        self.ui_state.dialog_manager.render_area(ui);
        self.ui_state.process_events();

        let mut files = self.state.active_files();
        if self.state.needs_sort {
            files = self.state.sort_indices(&mut files);
        }

        connect_ui_components_callback(ui, &files, &mut self.state, &mut self.ui_state);


        if self.state.is_loading || self.state.active_tasks > 0 {
            ui.request_repaint();
        } else {
            ui.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }

    fn on_exit(&mut self) {
        self.state.save_caches(true);
    }
}