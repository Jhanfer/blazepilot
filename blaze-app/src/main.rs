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

use egui_winit::winit::event_loop::EventLoop;
use std::{path::Path, sync::Arc, time::Duration};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{EnvFilter, fmt};
mod app;
mod core;
mod platform;
mod ui;
mod utils;
mod window_backend;

#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;

use crate::{
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
    utils::mimalloc_fn::set_mi_option,
    window_backend::{BlazeEventLoop, PresentMode, RendererConfig},
};

use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn init_dir_trash() -> Result<(), Box<dyn std::error::Error>> {
    KnownDirsManager::init();
    init_trash_backend()?;
    Ok(())
}

fn main() {
    //setea la opción de mimalloc para liberar memoria
    unsafe {
        set_mi_option();
    }
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .init();

    if std::env::var("BLAZE_IS_CHILD").is_ok() {
        unsafe {
            set_mi_option();
        }
        let present_mode = parse_present_mode_from_env();
        let with_trasnparency = parse_transparency_from_env();
        let backend = parse_backend_from_env();
        let initial_path = parse_initial_path();
        let _ = init_dir_trash().map_err(|e| warn!("Error inicializando: {}", e));

        let config = RendererConfig::wgpu_present(backend, present_mode, with_trasnparency);
        if let Err(e) = run_application(config, initial_path) {
            error!("Fallo al arrancar: {}", e);
            std::process::exit(1);
        }
        return;
    }

    let _ = init_dir_trash().map_err(|e| warn!("Ha ocurrido un error inicializando: {}", e));

    let _ = ffmpeg_next::init().map_err(|e| warn!("Ha ocurrido un error inicializando: {}", e));

    ffmpeg_next::log::set_level(ffmpeg_next::log::Level::Error);

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
        (backend.clone(), PresentMode::Immediate, true),
        (backend.clone(), PresentMode::Immediate, false),
        (backend.clone(), PresentMode::Fifo, true),
        (backend, PresentMode::Fifo, false),
        (DisplayBackend::Auto, PresentMode::Fifo, true),
        (DisplayBackend::Auto, PresentMode::Fifo, false),
    ];

    for (attempt, (backend, present_mode, with_transparency)) in configs.iter().enumerate() {
        debug!(
            "Intento {}/{}: Backend={:?}, PresentMode={:?}, Transparencias={}",
            attempt + 1,
            configs.len(),
            backend,
            present_mode,
            with_transparency,
        );

        let mut cmd = std::process::Command::new(&exe);
        cmd.args(&args)
            .env("BLAZE_PRESENT_MODE", format!("{:?}", present_mode))
            .env("BLAZE_BACKEND", format!("{:?}", backend))
            .env("BALZE_TRANSPARENCY", format!("{:?}", with_transparency))
            .env("BLAZE_IS_CHILD", "1")
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());

        let status = cmd.status()?;

        if status.success() {
            info!("Intento {} completado correctamente.", attempt + 1);
            return Ok(());
        }

        #[cfg(unix)]
        if let Some(signal) = status.signal() {
            info!("Proceso terminado por señal {} (cierre normal)", signal);
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

fn parse_present_mode_from_env() -> egui_wgpu::wgpu::PresentMode {
    match std::env::var("BLAZE_PRESENT_MODE").as_deref() {
        Ok("Immediate") => egui_wgpu::wgpu::PresentMode::Immediate,
        _ => egui_wgpu::wgpu::PresentMode::Fifo,
    }
}

fn parse_backend_from_env() -> DisplayBackend {
    match std::env::var("BLAZE_BACKEND").as_deref() {
        Ok("X11") => DisplayBackend::X11,
        Ok("Wayland") => DisplayBackend::Wayland,
        _ => DisplayBackend::Auto,
    }
}

fn run_application(config: RendererConfig, initial_path: Option<Arc<Path>>) -> anyhow::Result<()> {
    let mut event_loop_builder = EventLoop::with_user_event();
    #[cfg(target_os = "linux")]
    {
        use egui_winit::winit::platform::wayland::EventLoopBuilderExtWayland;
        use egui_winit::winit::platform::x11::EventLoopBuilderExtX11;

        match config.renderer {
            DisplayBackend::X11 => {
                event_loop_builder.with_x11();
            }
            DisplayBackend::Wayland => {
                use crate::{
                    core::system::clipboard_text::text_clipboard::with_text_clipboard,
                    platform::wayland::clipboard_wayland::WaylandClipboard,
                };
                with_text_clipboard(|c| c.init(WaylandClipboard::new()));
                event_loop_builder.with_wayland();
            }
            _ => {}
        }
    }

    let event_loop = event_loop_builder
        .build()
        .map_err(|e| anyhow::anyhow!("Error creando event loop: {e}"))?;

    let mut app = BlazeEventLoop::new(config, initial_path);

    event_loop
        .run_app(&mut app)
        .map_err(|e| anyhow::anyhow!("Error en event loop: {e}"))
}
