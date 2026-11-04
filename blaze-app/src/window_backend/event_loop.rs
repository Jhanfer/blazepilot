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

use egui::Id;
use egui_winit::winit::event::StartCause;
use egui_winit::winit::event_loop::ControlFlow;
use egui_winit::winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};
use std::time::Instant;
use std::{path::Path, sync::Arc, time::Duration};
use tracing::{debug, error, warn};

use crate::{
    app::{BlazeApp, BlazeAppBuilder},
    core::system::clipboard::global_clipboard::TOKIO_RUNTIME,
    window_backend::{AppWindow, Renderer, RendererConfig},
};

/// Controlador principal del bucle de eventos
///
/// Intergra la ventana `winit`, el motor de renderizado [`Renderer`],
/// la integración con `egui_winit` y el estado de la aplicación. Implementa
/// [`ApplicationHandler`] para responder al ciclo de vida del evento y gestionar el
/// redibujado bajo demanda o en diferido
pub struct BlazeEventLoop {
    /// Contexto global de egui
    egui_context: egui::Context,
    /// Estado de integración entre egui y winit
    egui_state: Option<egui_winit::State>,
    /// Configuración inicial del renderizador
    render_config: RendererConfig,
    /// Ruta inicial opcional que se pasa al construir [`BlazeApp`]
    initial_path: Option<Arc<std::path::Path>>,
    /// Instancia principal de la lógica de Blaze
    blaze_app: Option<BlazeApp>,
    /// Referencia compartida a la ventana gestionada por winit
    window: Option<Arc<Window>>,
    /// Backend de renderizado sobre la ventana
    renderer: Option<Renderer>,
    /// Instancia programada para solicitar el próximo redibujado
    next_repaint_at: Option<Instant>,
}

impl BlazeEventLoop {
    /// Crea una instancia no inicializada del bucle de eventos
    ///
    /// La ventana, el renderizador y el estado de egui/Blaze no se instancian
    /// hasta que `winit` emita el evento [`ApplicationHandler::resumed`] invocado en [`Self::init`]
    pub fn new(render_config: RendererConfig, initial_path: Option<Arc<Path>>) -> Self {
        Self {
            window: None,
            renderer: None,
            blaze_app: None,
            render_config,
            initial_path,
            egui_state: None,
            egui_context: egui::Context::default(),
            next_repaint_at: None,
        }
    }

    /// Inicializa la ventana, el renderizador, el estado de egui y [`BlazeApp`]
    ///
    /// Si la ventana ya ha sido creada previamente e intenta reinicializarse, la función retorna `()` inmediatamente
    ///
    /// # Errores:
    /// Devuelve un error si falla la creación de la ventana con [´AppWindow::create´]
    /// o si la instanciación asíncrona del [`Renderer`] falla en el runtime de tokio
    pub fn init(&mut self, event_loop: &ActiveEventLoop) -> anyhow::Result<()> {
        if self.window.is_some() {
            return Ok(());
        }

        let window = AppWindow::create(event_loop, &self.render_config)?;

        // callback para que egui pida repintado inmediato cuando hay cambios visuales activos
        {
            let repaint_window = window.clone();
            self.egui_context.set_request_repaint_callback(move |info| {
                if info.delay.is_zero() {
                    repaint_window.request_redraw();
                }
            });
        }

        // bloquea síncronamente el runtime tokio para crear el renderizador de la ventana
        let renderer =
            TOKIO_RUNTIME.block_on(Renderer::new(window.clone(), self.render_config.clone()))?;

        // desactiva el feathering si hay multisapling (MSAA) para evitar artefactos
        self.egui_context.tessellation_options_mut(|tess_options| {
            tess_options.feathering = renderer.msaa_samples() == 1;
        });

        let display_ptr = raw_display_handle(&window);
        let blazeapp = BlazeAppBuilder::default()
            .with_start_path(self.initial_path.clone())
            .build(display_ptr);

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
    /// Invocado cuando el bucle de eventos se reanuda o arranca
    /// Inicializa los recursos de la aplicación e interrumpe el evento en caso de fallo crítico
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Err(e) = self.init(event_loop) {
            error!("Error inicializando: {e:?}");
            event_loop.exit();
        }
    }

    /// Procesa los eventos específicos de ventana (ej: redibujado, redimensionado, interacción de periféricos)
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

        // alimenta egui con los eventos de la ventana winit
        let egui_response = egui_state.on_window_event(window, &event);

        if egui_response.repaint && !matches!(event, WindowEvent::RedrawRequested) {
            window.request_redraw();
        }

        if !egui_response.consumed && event == WindowEvent::CloseRequested {
            event_loop.exit();
        }

        match event {
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let size = window.inner_size();
                let ppp = scale_factor as f32;

                renderer.resize(size.width, size.height, ppp);
                self.egui_context.set_pixels_per_point(scale_factor as f32);

                window.request_redraw();
            }

            WindowEvent::Resized(size) => {
                let ppp = window.scale_factor() as f32;
                renderer.resize(size.width, size.height, ppp);
            }

            WindowEvent::RedrawRequested => {
                let mut raw_input = egui_state.take_egui_input(window);

                raw_input.max_texture_side = Some(renderer.max_texture_side());

                blazeapp.raw_input_hook(&self.egui_context, &mut raw_input);

                let full_output = self.egui_context.run_ui(raw_input, |ctx| {
                    ctx.data_mut(|d| {
                        d.insert_temp(Id::new("icon_cache"), blazeapp.ui_state.icon_cache.clone())
                    });
                    blazeapp.ui(ctx);
                });

                egui_state.handle_platform_output(window, full_output.platform_output);

                let primitives = self
                    .egui_context
                    .tessellate(full_output.shapes, full_output.pixels_per_point);

                window.pre_present_notify();

                renderer.render(&primitives, &full_output.textures_delta);

                // determina el retraso del repintado difrerido
                let repaint_delay = full_output
                    .viewport_output
                    .get(&egui::ViewportId::ROOT)
                    .map(|v| v.repaint_delay)
                    .unwrap_or(Duration::MAX);

                self.next_repaint_at = if repaint_delay == Duration::MAX {
                    None
                } else {
                    Some(Instant::now() + repaint_delay)
                };
            }

            _ => {}
        }
    }

    /// Configura la espera del bucle de eventos [`ControlFlow`]
    /// según si existe un redibujado programado
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        match self.next_repaint_at {
            None => event_loop.set_control_flow(ControlFlow::Wait),
            Some(at) => event_loop.set_control_flow(ControlFlow::WaitUntil(at)),
        }
    }

    /// Evalúa el despertar de temporizador del bucle para solicitar el redibujado
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if !matches!(cause, StartCause::ResumeTimeReached { .. }) {
            return;
        }

        let Some(at) = self.next_repaint_at else {
            return;
        };
        if at > Instant::now() {
            return;
        }

        self.next_repaint_at = None;
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    /// Notifica la salida de la aplicación y libera los recursos del entorno gráfico
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(blazeapp) = self.blaze_app.as_mut() {
            blazeapp.on_exit();
        };

        drop(self.renderer.take());
        drop(self.egui_state.take());
        drop(self.window.take());
    }
}

/// Extrae el puntero crudo al display nativo de la ventana en entornos Wayland de Linux
///
/// Retorna `Some(ptr)` si se está ejecutando sobre un servidor Wayland válido o `None`
/// en otros backends (ej X11)
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
