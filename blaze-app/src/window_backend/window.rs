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

use crate::window_backend::RendererConfig;
use egui_winit::winit::{
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes},
};
use std::sync::Arc;

/// Representación de la ventana de la app
///
/// Sirve como wrapper para centralizar la configuración
/// e inicialización de la ventana sobre `Winit`
pub struct AppWindow;

impl AppWindow {
    /// Crea e inicializa la nueva ventana
    ///
    /// Aplica las dimensiones iniciales y mínimas por defecto
    ///
    /// # Parámetros:
    /// - `event_loop`: es el buvle de eventos activo de winit requerido para instanciar la ventana
    /// - `config`: configuración del renderizador
    ///
    /// # Errores:
    /// Devuelve un error si el sistema operativo o el backend de la ventana no pueden crear la ventana
    ///
    /// # Retorno:
    /// Devuelve un [`Arc<Window>`] para permitir compartir la referencia de la ventana de manera segura entre el contexto de renderizado y el bucle de eventos
    pub fn create(
        event_loop: &ActiveEventLoop,
        config: &RendererConfig,
    ) -> anyhow::Result<Arc<Window>> {
        // se define las dimensiones lógicas para mantener proporciones consistentes
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
