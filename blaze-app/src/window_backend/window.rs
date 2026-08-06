use std::sync::Arc;

use crate::window_backend::RendererConfig;
use egui_winit::winit::{
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};

pub struct AppWindow;

impl AppWindow {
    pub fn create(
        event_loop: &ActiveEventLoop,
        config: &RendererConfig,
    ) -> anyhow::Result<Arc<Window>> {
        let attributes = WindowAttributes::default()
            .with_inner_size(egui_winit::winit::dpi::LogicalSize::new(1280.0, 720.0))
            .with_min_inner_size(egui_winit::winit::dpi::LogicalSize::new(800.0, 500.0))
            .with_title("BlazePilot")
            .with_decorations(true)
            .with_transparent(config.transparency)
            .with_resizable(true)
            .with_maximized(false);

        let window = event_loop
            .create_window(attributes)
            .map_err(|e| anyhow::anyhow!("Error creando ventana: {e}"))?;

        Ok(Arc::new(window))
    }
}
