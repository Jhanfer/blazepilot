use egui_winit::winit::event_loop::ControlFlow;
use egui_winit::winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};
use std::{path::Path, sync::Arc, time::Duration};
use tracing::{debug, error, warn};

use crate::{
    app::{BlazeApp, BlazeAppBuilder},
    core::system::clipboard::global_clipboard::TOKIO_RUNTIME,
    window_backend::{AppWindow, Renderer, RendererConfig, repaint_signal::RepaintSignal},
};

pub struct BlazeEventLoop {
    cached_primitives: Vec<egui::ClippedPrimitive>,
    egui_context: egui::Context,
    egui_state: Option<egui_winit::State>,
    render_config: RendererConfig,
    initial_path: Option<Arc<std::path::Path>>,
    blaze_app: Option<BlazeApp>,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    repaint_signal: RepaintSignal,
}

impl BlazeEventLoop {
    pub fn new(render_config: RendererConfig, initial_path: Option<Arc<Path>>) -> Self {
        Self {
            cached_primitives: Vec::new(),
            window: None,
            renderer: None,
            blaze_app: None,
            render_config,
            initial_path,
            egui_state: None,
            egui_context: egui::Context::default(),
            repaint_signal: RepaintSignal::new(),
        }
    }

    pub fn init(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        let window = AppWindow::create(event_loop, &self.render_config)?;

        let display_ptr = raw_display_handle(&window);

        let blazeapp = BlazeAppBuilder::default()
            .with_start_path(self.initial_path.clone())
            .build(display_ptr);

        let renderer =
            TOKIO_RUNTIME.block_on(Renderer::new(window.clone(), self.render_config.clone()))?;

        let egui_state = egui_winit::State::new(
            self.egui_context.clone(),
            egui::ViewportId::default(),
            &window,
            None,
            None,
            None,
        );

        let pixels_per_point = window.scale_factor() as f32;
        self.egui_context.set_pixels_per_point(pixels_per_point);

        window.request_redraw();

        self.window = Some(window);
        self.renderer = Some(renderer);
        self.blaze_app = Some(blazeapp);
        self.egui_state = Some(egui_state);

        Ok(())
    }
}

impl ApplicationHandler for BlazeEventLoop {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(e) = self.init(event_loop) {
            error!("Error inicializando: {e:?}");
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };
        let Some(blazeapp) = self.blaze_app.as_mut() else {
            return;
        };
        let Some(egui_state) = self.egui_state.as_mut() else {
            return;
        };

        let egui_response = egui_state.on_window_event(window, &event);

        if !egui_response.consumed && event == WindowEvent::CloseRequested {
            event_loop.exit();
        }

        match event {
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let size = window.inner_size();
                renderer.resize(size.width, size.height, scale_factor as f32);
                self.egui_context.set_pixels_per_point(scale_factor as f32);
            }

            WindowEvent::Resized(size) => {
                let ppp = window.scale_factor() as f32;
                renderer.resize(size.width, size.height, ppp);
                window.request_redraw();
            }

            WindowEvent::RedrawRequested => {
                let mut raw_input = egui_state.take_egui_input(window);

                raw_input.max_texture_side = Some(renderer.max_texture_side());

                blazeapp.raw_input_hook(&self.egui_context, &mut raw_input);

                let full_output = self.egui_context.run_ui(raw_input, |ctx| {
                    blazeapp.ui(ctx);
                });

                egui_state.handle_platform_output(window, full_output.platform_output);

                self.cached_primitives = self
                    .egui_context
                    .tessellate(full_output.shapes, full_output.pixels_per_point);

                renderer.render(&self.cached_primitives, &full_output.textures_delta);

                let sleep_time = match self.repaint_signal.take() {
                    Some(0) => Duration::from_millis(8),
                    Some(ms) => Duration::from_millis(ms),
                    None => Duration::from_millis(8),
                };

                window.request_redraw();
                std::thread::sleep(sleep_time);
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Wait);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        let Some(blazeapp) = self.blaze_app.as_mut() else {
            return;
        };
        blazeapp.on_exit();
    }
}

fn raw_display_handle(window: &Window) -> Option<*mut std::ffi::c_void> {
    #[cfg(target_os = "linux")]
    {
        use egui_winit::winit::raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
        match window.display_handle() {
            Ok(handle) => match handle.as_raw() {
                RawDisplayHandle::Wayland(h) => {
                    debug!("Backend Wayland Ok");
                    Some(h.display.as_ptr())
                }
                other => {
                    warn!("El backend es: {:?}", other);
                    None
                }
            },
            Err(e) => {
                error!("display_handle() ha fallado: {e}");
                None
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

impl Drop for BlazeEventLoop {
    fn drop(&mut self) {
        drop(self.renderer.take());
        drop(self.egui_state.take());
        drop(self.window.take());
        debug!("BlazeEventLoop destruido correctamente");
    }
}
