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



use std::sync::Arc;
use eframe::HardwareAcceleration;
use tracing::warn;
use tracing_subscriber::{fmt, EnvFilter};
mod app;
mod core;
mod ui;
mod utils;
use mimalloc::MiMalloc;

#[cfg(target_os = "linux")]
use winit::platform::x11::EventLoopBuilderExtX11;
#[cfg(target_os = "linux")]
use winit::platform::wayland::EventLoopBuilderExtWayland;


use crate::{app::BlazeAppBuilder, core::{bootstrap::configs::config_manager::with_configs, system::{knowndirs::knowndirs_manager::KnownDirsManager, trash_manager::trash_manager::init_trash_backend}},utils::initial_path_handler::parse_initial_path};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;


fn init_dir_trash() -> Result<(), Box<dyn std::error::Error>> {
    KnownDirsManager::init();
    init_trash_backend()?;
    Ok(())
}


fn main() {

    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .init();

    let initial_path = parse_initial_path();

    let _ = init_dir_trash()
        .map_err(|e| warn!("Ha ocurrido un error inicializando: {}", e));

    let backend = with_configs(|c| {
        c.get_display_backend()
    });


    let mut options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 500.0])
            .with_title("BlazePilot")
            .with_decorations(true)
            .with_transparent(true)
            .with_resizable(true)
            .with_maximized(false)
            .with_fullscreen(false),
        renderer: eframe::Renderer::Wgpu,
        hardware_acceleration: HardwareAcceleration::Preferred,
        vsync: false,
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        dithering: false,

        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            present_mode: eframe::wgpu::PresentMode::Fifo,
            desired_maximum_frame_latency: Some(1),

            wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(eframe::egui_wgpu::WgpuSetupCreateNew {
                power_preference: eframe::wgpu::PowerPreference::LowPower,
                device_descriptor: Arc::new(|adapter| eframe::wgpu::DeviceDescriptor {
                    label: Some("BlazePilot Device"),
                    required_limits: adapter.limits(),
                    required_features: eframe::wgpu::Features::empty(),
                    memory_hints: eframe::wgpu::MemoryHints::MemoryUsage,
                    experimental_features: eframe::wgpu::ExperimentalFeatures::disabled(),
                    trace: eframe::wgpu::Trace::Off,
                }),

                ..eframe::egui_wgpu::WgpuSetupCreateNew::without_display_handle()
            }),

            ..Default::default()
        },

        ..Default::default()
    };

    #[cfg(target_os = "linux")]
    {
        options.event_loop_builder = Some(Box::new(move |builder| {
            use crate::core::bootstrap::configs::platform::linux::conf_structs::DisplayBackend;
            match backend {
                DisplayBackend::X11 => builder.with_x11(),
                DisplayBackend::Wayland => builder.with_wayland(),
                _ => builder,
            };
        }));
    }

    let blazeapp = BlazeAppBuilder::default()
        .with_start_path(initial_path)
        .build();

    eframe::run_native(
        "BlazePilot",
        options,
        Box::new(|_cc| {
            Ok(Box::new(blazeapp))
        }),
    ).unwrap();
}