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



use eframe::HardwareAcceleration;
use tracing_subscriber::{fmt, EnvFilter};
mod app;
mod core;
mod ui;
mod utils;
use app::BlazeApp;
use mimalloc::MiMalloc;

use crate::{core::{blaze_state::BlazeCoreState, configs::config_state::with_configs, system::clipboard::TOKIO_RUNTIME}, ui::blaze_ui_state::BlazeUiState};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() {

    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .init();


    with_configs(|cfg| cfg.load_or_init_cofigs().unwrap());


    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("BlazePilot")
            .with_decorations(true)
            .with_transparent(true),
        multisampling: 4,
        renderer: eframe::Renderer::Wgpu,
        hardware_acceleration: HardwareAcceleration::Required,
        ..Default::default()
    };

    let state = TOKIO_RUNTIME.block_on(BlazeCoreState::new());
    let ui_state = BlazeUiState::new();

    let blazeapp = BlazeApp {
        state,
        ui_state,
    };

    eframe::run_native(
        "BlazePilot", 
        options, 
        Box::new(|_cc| Ok(Box::new(blazeapp))),
    ).unwrap();
}
