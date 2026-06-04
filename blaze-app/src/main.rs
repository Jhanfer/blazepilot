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

use eframe::{HardwareAcceleration, NativeOptions};
use std::{path::Path, sync::Arc, time::Duration};
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, EnvFilter};
mod app;
mod core;
mod ui;
mod utils;
use mimalloc::MiMalloc;

#[cfg(target_os = "linux")]
use winit::platform::wayland::EventLoopBuilderExtWayland;
#[cfg(target_os = "linux")]
use winit::platform::x11::EventLoopBuilderExtX11;

use crate::{
    app::BlazeAppBuilder,
    core::{
        bootstrap::configs::{
            config_manager::with_configs, platform::linux::conf_structs::DisplayBackend,
        },
        system::{
            knowndirs::knowndirs_manager::KnownDirsManager,
            trash_manager::trash_manager::init_trash_backend,
        },
    },
    utils::initial_path_handler::parse_initial_path,
};

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

    let _ = init_dir_trash().map_err(|e| warn!("Ha ocurrido un error inicializando: {}", e));

    if let Err(e) = try_run_with_retries(initial_path) {
        error!("Todos los intentos han fallado: {}", e);
        std::process::exit(1);
    }
}

fn try_run_with_retries(initial_path: Option<Arc<Path>>) -> anyhow::Result<()> {
    let mut last_error;

    let max_tries = std::env::var("BLAZE_RETRIES")
        .unwrap_or_else(|_| "3".to_string())
        .parse()
        .unwrap_or(3);

    let retry_delay = std::env::var("BALAZE_RETRY_DELAY")
        .unwrap_or_else(|_| "500".to_string())
        .parse()
        .unwrap_or(500);

    let backend = with_configs(|c| c.get_display_backend());

    let configs = vec![
        RunConfigs::default_with_backend(backend),
        RunConfigs::default_with_backend(DisplayBackend::Wayland),
        RunConfigs::default_with_backend(DisplayBackend::X11),
        RunConfigs::fallback_config(),
    ];

    for (attempt, config) in configs.iter().enumerate() {
        info!(
            "Intento {}: iniciando con backend {:?}",
            attempt + 1,
            config.backend
        );

        match run_application(config.clone(), initial_path.clone()) {
            Ok(_) => return Ok(()),
            Err(e) => {
                last_error = Some(e);
                warn!(
                    "Intento {} ha fallado: {:?}: esperando antes de reintentar...",
                    attempt + 1,
                    last_error
                );

                if attempt < max_tries - 1 {
                    std::thread::sleep(Duration::from_millis(retry_delay * (attempt + 1) as u64));
                }
            }
        }
    }

    warn!("Intentando configuración predeterminada...");
    if let Err(e) = run_application(RunConfigs::default_config(), initial_path) {
        Err(anyhow::anyhow!("Fallo catastrófico: {}", e))
    } else {
        Ok(())
    }
}

fn run_application(config: RunConfigs, initial_path: Option<Arc<Path>>) -> anyhow::Result<()> {
    let mut options = create_native_options(&config);

    #[cfg(target_os = "linux")]
    {
        let backend = config.backend.clone();
        options.event_loop_builder = Some(Box::new(move |builder| {
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
        Box::new(|_cc| Ok(Box::new(blazeapp))),
    )
    .map_err(|e| anyhow::anyhow!("Error al ejecutar: {}", e))
}

#[derive(Clone)]
struct RunConfigs {
    backend: DisplayBackend,
    vsync: bool,
    multisampling: u16,
    power_preference: eframe::wgpu::PowerPreference,
}

impl RunConfigs {
    fn default_with_backend(backend: DisplayBackend) -> Self {
        Self {
            backend,
            vsync: false,
            multisampling: 0,
            power_preference: eframe::wgpu::PowerPreference::LowPower,
        }
    }

    fn fallback_config() -> Self {
        Self {
            backend: DisplayBackend::Auto,
            vsync: true,
            multisampling: 0,
            power_preference: eframe::wgpu::PowerPreference::LowPower,
        }
    }

    fn default_config() -> Self {
        Self {
            backend: DisplayBackend::Auto,
            vsync: true,
            multisampling: 0,
            power_preference: eframe::wgpu::PowerPreference::None,
        }
    }
}

fn create_native_options(configs: &RunConfigs) -> NativeOptions {
    NativeOptions {
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
        vsync: configs.vsync,
        multisampling: configs.multisampling,
        depth_buffer: 0,
        stencil_buffer: 0,
        dithering: false,

        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            present_mode: if configs.vsync {
                eframe::wgpu::PresentMode::Fifo
            } else {
                eframe::wgpu::PresentMode::Immediate
            },
            desired_maximum_frame_latency: Some(1),

            wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(
                eframe::egui_wgpu::WgpuSetupCreateNew {
                    power_preference: configs.power_preference,
                    device_descriptor: Arc::new(|adapter| eframe::wgpu::DeviceDescriptor {
                        label: Some("BlazePilot Device"),
                        required_limits: adapter.limits(),
                        required_features: eframe::wgpu::Features::empty(),
                        memory_hints: eframe::wgpu::MemoryHints::MemoryUsage,
                        experimental_features: eframe::wgpu::ExperimentalFeatures::disabled(),
                        trace: eframe::wgpu::Trace::Off,
                    }),

                    ..eframe::egui_wgpu::WgpuSetupCreateNew::without_display_handle()
                },
            ),

            ..Default::default()
        },

        ..Default::default()
    }
}
