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
use tracing_subscriber::{EnvFilter, fmt};
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
            trash_manager::manager::init_trash_backend,
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

    if std::env::var("BLAZE_IS_CHILD").is_ok() {
        let present_mode = parse_present_mode_from_env();
        let with_trasnparency = parse_transparency_from_env();
        let backend = parse_backend_from_env();
        let initial_path = parse_initial_path();
        let _ = init_dir_trash().map_err(|e| warn!("Error inicializando: {}", e));

        let config = RunConfigs::wgpu_present(backend, present_mode, with_trasnparency);
        if let Err(e) = run_application(config, initial_path) {
            error!("Fallo al arrancar: {}", e);
            std::process::exit(1);
        }
        return;
    }

    let _ = init_dir_trash().map_err(|e| warn!("Ha ocurrido un error inicializando: {}", e));

    if let Err(e) = try_run_with_retries() {
        error!("Todos los intentos han fallado: {}", e);
        std::process::exit(1);
    }
}

fn try_run_with_retries() -> anyhow::Result<()> {
    let exe = std::env::current_exe()?;
    let args: Vec<String> = std::env::args().skip(1).collect();

    let retry_delay = std::env::var("BLAZE_RETRY_DELAY")
        .unwrap_or_else(|_| "500".to_string())
        .parse()
        .unwrap_or(500u64);

    let backend = with_configs(|c| c.get_display_backend());

    let configs = [
        (backend.clone(), "Immediate", true),
        (backend.clone(), "Immediate", false),
        (backend.clone(), "Fifo", true),
        (backend, "Fifo", false),
        (DisplayBackend::Auto, "Fifo", true),
        (DisplayBackend::Auto, "Fifo", false),
    ];

    for (attempt, (backend, present_mode, with_transparency)) in configs.iter().enumerate() {
        info!(
            "Intento {}/{}: Backend={:?}, PresentMode={}, Transparencias={}",
            attempt + 1,
            configs.len(),
            backend,
            present_mode,
            with_transparency,
        );

        let mut cmd = std::process::Command::new(&exe);
        cmd.args(&args)
            .env("BLAZE_PRESENT_MODE", present_mode)
            .env("BLAZE_BACKEND", format!("{:?}", backend))
            .env("BALZE_TRANSPARENCY", format!("{:?}", with_transparency))
            .env("BLAZE_IS_CHILD", "1");

        let status = cmd.status()?;

        if status.success() {
            info!("Intento {} completado correctamente.", attempt + 1);
            return Ok(());
        }

        warn!(
            "Intento {} terminó con código: {:?}",
            attempt + 1,
            status.code()
        );

        if attempt < configs.len() - 1 {
            let delay = retry_delay * (attempt as u64 + 1);
            info!("Esperando {}ms antes del siguiente intento...", delay);
            std::thread::sleep(Duration::from_millis(delay));
        }
    }

    Err(anyhow::anyhow!(
        "Todos los intentos fallaron. Instala drivers Vulkan o ejecuta con LIBGL_ALWAYS_SOFTWARE=1"
    ))
}

fn parse_transparency_from_env() -> bool {
    match std::env::var("BALZE_TRANSPARENCY").as_deref() {
        Ok(transp) => match transp.to_lowercase().as_ref() {
            "true" => true,
            "false" => false,
            _ => false,
        },
        _ => false,
    }
}

fn parse_present_mode_from_env() -> eframe::wgpu::PresentMode {
    match std::env::var("BLAZE_PRESENT_MODE").as_deref() {
        Ok("Immediate") => eframe::wgpu::PresentMode::Immediate,
        _ => eframe::wgpu::PresentMode::Fifo,
    }
}

fn parse_backend_from_env() -> DisplayBackend {
    match std::env::var("BLAZE_BACKEND").as_deref() {
        Ok("X11") => DisplayBackend::X11,
        Ok("Wayland") => DisplayBackend::Wayland,
        _ => DisplayBackend::Auto,
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

#[derive(Clone, Debug)]
struct RunConfigs {
    backend: DisplayBackend,
    vsync: bool,
    multisampling: u16,
    power_preference: eframe::wgpu::PowerPreference,
    present_mode: eframe::wgpu::PresentMode,
    transparency: bool,
}

impl RunConfigs {
    fn wgpu_present(
        backend: DisplayBackend,
        present_mode: eframe::wgpu::PresentMode,
        transparency: bool,
    ) -> Self {
        Self {
            backend,
            present_mode,
            vsync: matches!(present_mode, eframe::wgpu::PresentMode::Fifo),
            multisampling: 0,
            power_preference: eframe::wgpu::PowerPreference::LowPower,
            transparency,
        }
    }
}

fn create_native_options(configs: &RunConfigs) -> NativeOptions {
    let viewport = egui::ViewportBuilder::default()
        .with_inner_size([1280.0, 720.0])
        .with_min_inner_size([800.0, 500.0])
        .with_title("BlazePilot")
        .with_decorations(true)
        .with_transparent(configs.transparency)
        .with_resizable(true)
        .with_maximized(false)
        .with_fullscreen(false);
    NativeOptions {
        viewport,
        renderer: eframe::Renderer::Wgpu,
        hardware_acceleration: HardwareAcceleration::Preferred,
        vsync: configs.vsync,
        multisampling: configs.multisampling,
        depth_buffer: 0,
        stencil_buffer: 0,
        dithering: false,
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            present_mode: configs.present_mode,
            desired_maximum_frame_latency: Some(1),
            wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(
                eframe::egui_wgpu::WgpuSetupCreateNew {
                    power_preference: configs.power_preference,
                    device_descriptor: Arc::new(|adapter| {
                        let limits = adapter.limits();
                        eframe::wgpu::DeviceDescriptor {
                            label: Some("BlazePilot Device"),
                            required_limits: limits,
                            required_features: eframe::wgpu::Features::empty(),
                            memory_hints: eframe::wgpu::MemoryHints::MemoryUsage,
                            experimental_features: eframe::wgpu::ExperimentalFeatures::disabled(),
                            trace: eframe::wgpu::Trace::Off,
                        }
                    }),
                    ..eframe::egui_wgpu::WgpuSetupCreateNew::without_display_handle()
                },
            ),
            ..Default::default()
        },
        ..Default::default()
    }
}
