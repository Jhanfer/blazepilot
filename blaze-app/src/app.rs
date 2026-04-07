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




use crate::core::blaze_state::BlazeCoreState;
use crate::ui::blaze_ui_state::BlazeUiState;
use crate::ui::modules::ui_callback::connect_ui_components_callback;


pub struct BlazeApp {
    pub state: BlazeCoreState, //motor, archivos, mover
    pub ui_state: BlazeUiState, //visuales (qué item está hovereado, etc)
}

impl eframe::App for BlazeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {        
        self.state.process_messages();
        
        self.ui_state.dialog_manager.render_area(ctx);
        self.ui_state.process_events();

        let files = self.state.active_files();
        connect_ui_components_callback(ctx, &files, &mut self.state, &mut self.ui_state);


        if self.state.is_loading || self.state.active_tasks > 0 {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
    }
}