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

use crate::core::bootstrap::configs::platform::linux::conf_structs::DisplayBackend;
use egui_wgpu::wgpu::{
    CompositeAlphaMode, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
    InstanceDescriptor, LoadOp, Operations, PresentMode as WgpuPresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface,
    SurfaceCapabilities, SurfaceConfiguration, TextureUsages,
};
use egui_winit::winit::window::Window;
use std::{sync::Arc, time::Duration};
use tracing::debug;

/// Límite superior de primitivas de renderizado por fotogramas
/// para acortar el costo de dibujado
const MAX_PRIMITIVES_PER_FRAME: usize = 512;

/// Configuración para inicialización de la canalización de renderizado en `wgpu`
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// Backend de pantalla asociado al entorno de ejecución (ej Wayland)
    pub renderer: DisplayBackend,
    /// Preferencia de rendimiento de la GPU
    pub power_preference: egui_wgpu::wgpu::PowerPreference,
    /// Modo de representación para la sincronizaciópn vertical
    pub present_mode: egui_wgpu::wgpu::PresentMode,
    /// Indica si se deshabilita la transparencia
    pub transparency: bool,
    /// Cantidad de muestras para el antialiasing MSAA
    pub msaa_samples: u32,
}

impl RendererConfig {
    /// Crea una configuración predeterminada orientada a bajo consumo energético
    pub fn wgpu_present(
        renderer: DisplayBackend,
        present_mode: egui_wgpu::wgpu::PresentMode,
        transparency: bool,
        msaa_samples: u32,
    ) -> Self {
        Self {
            renderer,
            present_mode,
            power_preference: egui_wgpu::wgpu::PowerPreference::LowPower,
            transparency,
            msaa_samples,
        }
    }
}

/// Modo de representación simplificados para el control de la sincronización vertical
#[derive(Clone, Debug, PartialEq)]
pub enum PresentMode {
    /// Inmediaro, sin esperar al VSYNC
    Immediate,
    /// Cola FIFO acoplada a la tasa de refresco del monitor (VSYNC)
    Fifo,
    /// Selección automática segun capacidades de la GPU
    Auto,
}

impl From<PresentMode> for WgpuPresentMode {
    fn from(mode: PresentMode) -> Self {
        match mode {
            PresentMode::Immediate => WgpuPresentMode::Immediate,
            PresentMode::Fifo => WgpuPresentMode::Fifo,
            PresentMode::Auto => WgpuPresentMode::AutoVsync,
        }
    }
}

/// Gestor principal del estado gráfico y la canalizacion de renderizado
/// basada en `wgpu` y `egui`
///
/// Encapsula la superficie de dibujo nativa, la GPU (`Device`), la cola de comandos (`Queue`),
/// las texturas MSAA y la integración directa con el renderizador interno de [`egui_wgpu::Renderer`]
pub struct Renderer {
    /// Superficie de la ventana vinvulada a `wgpu`
    surface: Surface<'static>,
    /// Dispositivo lógico de la GPU
    device: egui_wgpu::wgpu::Device,
    /// Adaptador físico que representa la GPU seleccionada
    adapter: egui_wgpu::wgpu::Adapter,
    /// Cola de ejecución para el envio de comandos a la GPU
    queue: Queue,
    /// Configuración de formato, dimensiones y sincronización de la superficie
    config: SurfaceConfiguration,
    /// Instancia interna del renderizador de `egui` sobre `wgpu`
    egui_renderer: egui_wgpu::Renderer,
    /// Escala y dimensiones en pixeles físicos para el cálculo de interfaz
    screen_descriptor: egui_wgpu::ScreenDescriptor,
    /// Cinturón del staging para transferencias eficientes de memoria CPU/GPU
    staging: egui_wgpu::wgpu::util::StagingBelt,
    /// Vista de la textura MSAA usada como búfer intermedio
    msaa_view: Option<egui_wgpu::wgpu::TextureView>,
    /// Número de muestras MSAA activas
    msaa_samples: u32,
    /// Determina la transparencia de la app
    transparency: bool,
    /// Color de fondo para limpiar el búfer en cada fotograma
    clear_color: egui_wgpu::wgpu::Color,
}

impl Renderer {
    /// Crea e inicializa de forma asíncrona una nueva instancia de [`Renderer`]
    ///
    /// Configura la superficie de la ventana, solicita el adaptador e inicializa
    /// la canalización gráfica soportando MSAA
    ///
    /// # Parámetros:
    /// - `windowd`: referencia compartida a la ventana nativa de `winit`
    /// - `render_config`: opciones de inicialización (MSAA, transparencias, etc)
    ///
    /// # Errores:
    /// Devuelve un error si falla la creación de la superficie, la selección
    /// del adaptador o la soliocitud del dispositivo lógico de la GPU
    pub async fn new(window: Arc<Window>, render_config: RendererConfig) -> anyhow::Result<Self> {
        let instance = Instance::new(InstanceDescriptor {
            backends: egui_wgpu::wgpu::Backends::PRIMARY,
            flags: egui_wgpu::wgpu::InstanceFlags::default(),
            memory_budget_thresholds: Default::default(),
            backend_options: egui_wgpu::wgpu::BackendOptions::default(),
            display: None,
        });

        let size = window.inner_size();

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point: window.scale_factor() as f32,
        };

        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: render_config.power_preference,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|e| anyhow::anyhow!("Ha ocurrido un error creando el adaptador: {e}"))?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("BlazePilotDevice"),
                required_features: Features::empty(),
                required_limits: Default::default(),
                memory_hints: egui_wgpu::wgpu::MemoryHints::Performance,
                trace: Default::default(),
                experimental_features: ExperimentalFeatures::disabled(),
            })
            .await
            .map_err(|e| anyhow::anyhow!("Ha ocurrido un error al pedir el divice: {e}"))?;

        let caps = surface.get_capabilities(&adapter);

        // caída a modo VSYNC FIFO si la GPU no soporta el modo solicitado
        let present_mode = if caps.present_modes.contains(&render_config.present_mode) {
            render_config.present_mode
        } else {
            WgpuPresentMode::Fifo
        };

        let alpha_mode = Self::pick_fallback_alpha_mode(&caps, render_config.transparency);

        // se prioriza el formato sRGB para corregir la percepción de color en el renderizado
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // verifica si la GPU físicamente la cantidad de muestras MSAA solicitadas
        let max_samples = adapter
            .get_texture_format_features(format)
            .flags
            .sample_count_supported(render_config.msaa_samples);

        let msaa_samples = if max_samples {
            render_config.msaa_samples
        } else {
            1
        };

        let msaa_view = if msaa_samples > 1 {
            Some(Self::create_msaa_view(&device, &config, msaa_samples))
        } else {
            None
        };

        let egui_renderer = egui_wgpu::Renderer::new(
            &device,
            format,
            egui_wgpu::RendererOptions {
                msaa_samples,
                dithering: true,
                ..Default::default()
            },
        );

        // calcula un tamaño razonable para el staging belt según la resolución inicial (mínimo 1 MB)
        let belt_size = ((size.width * size.height) / 16).max(1024 * 1024) as u64;
        let staging = egui_wgpu::wgpu::util::StagingBelt::new(device.clone(), belt_size);

        let clear_color = egui_wgpu::wgpu::Color {
            r: 0.7647,
            g: 0.0,
            b: 1.0,
            a: if render_config.transparency { 0.0 } else { 1.0 },
        };

        Ok(Self {
            surface,
            device,
            adapter,
            queue,
            config,
            egui_renderer,
            screen_descriptor,
            staging,
            msaa_view,
            msaa_samples,
            transparency: render_config.transparency,
            clear_color,
        })
    }

    /// Retorna el número actrual de muestras MSAA
    pub fn msaa_samples(&self) -> u32 {
        self.msaa_samples
    }

    /// Crea el búfer de textura inmediato para el renderizado MSAA
    fn create_msaa_view(
        device: &egui_wgpu::wgpu::Device,
        config: &SurfaceConfiguration,
        samples: u32,
    ) -> egui_wgpu::wgpu::TextureView {
        let texture = device.create_texture(&egui_wgpu::wgpu::TextureDescriptor {
            label: Some("MSAA_Texture"),
            size: egui_wgpu::wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: samples,
            dimension: egui_wgpu::wgpu::TextureDimension::D2,
            format: config.format,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        texture.create_view(&Default::default())
    }

    /// Selecciona el modo de composición alfa soportado por la superficie según
    /// las preferencias de transparencia
    fn pick_fallback_alpha_mode(
        caps: &SurfaceCapabilities,
        transparency: bool,
    ) -> CompositeAlphaMode {
        if caps.alpha_modes.is_empty() {
            return CompositeAlphaMode::Opaque;
        }

        if transparency {
            *caps
                .alpha_modes
                .iter()
                .find(|m| **m == CompositeAlphaMode::PreMultiplied)
                .or_else(|| {
                    caps.alpha_modes
                        .iter()
                        .find(|m| **m == CompositeAlphaMode::PostMultiplied)
                })
                .unwrap_or(&CompositeAlphaMode::Opaque)
        } else {
            if caps.alpha_modes.contains(&CompositeAlphaMode::Opaque) {
                CompositeAlphaMode::Opaque
            } else {
                caps.alpha_modes[0]
            }
        }
    }

    /// Retorna la dimensión máxima para texturas 2D permitidas por la GPU
    pub fn max_texture_side(&self) -> usize {
        self.device.limits().max_texture_dimension_2d as usize
    }

    /// Reconfigura la superficie y recrea los recursos dependientes de la resolución
    /// tras el redimensionado
    ///
    /// Ignora las redimensiones con dimensión cero al minimizar la ventana
    pub fn resize(&mut self, width: u32, height: u32, pixels_per_point: f32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;

        self.surface.configure(&self.device, &self.config);

        // recrea el búfer MSAA
        self.msaa_view = if self.msaa_samples > 1 {
            Some(Self::create_msaa_view(
                &self.device,
                &self.config,
                self.msaa_samples,
            ))
        } else {
            None
        };

        self.screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point,
        };
    }

    /// Renderiza un fotograma actualizando las texturas,
    /// búferes de geometría de ´egui´ y envía los comandos
    /// a la GPU
    pub fn render(
        &mut self,
        primitives: &[egui::ClippedPrimitive],
        textures_delta: &egui::TexturesDelta,
    ) {
        // truncado de las primitivas para evitar sobrecargas
        let primitives = &primitives[..primitives.len().min(MAX_PRIMITIVES_PER_FRAME)];

        let frame = match self.surface.get_current_texture() {
            egui_wgpu::wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            egui_wgpu::wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
            egui_wgpu::wgpu::CurrentSurfaceTexture::Lost
            | egui_wgpu::wgpu::CurrentSurfaceTexture::Outdated => {
                // reconfigura la superficie si la textura se ha vuelto obsoleta o se perdió la asociación con la ventana
                let caps = self.surface.get_capabilities(&self.adapter);

                if !caps.alpha_modes.contains(&self.config.alpha_mode) {
                    self.config.alpha_mode =
                        Self::pick_fallback_alpha_mode(&caps, self.transparency);
                }

                if !caps.present_modes.contains(&self.config.present_mode) {
                    self.config.present_mode = egui_wgpu::wgpu::PresentMode::Fifo;
                }

                self.surface.configure(&self.device, &self.config);
                return;
            }
            egui_wgpu::wgpu::CurrentSurfaceTexture::Timeout => return,
            _ => return,
        };

        let view = frame
            .texture
            .create_view(&egui_wgpu::wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.device
                .create_command_encoder(&egui_wgpu::wgpu::CommandEncoderDescriptor {
                    label: Some("MainEncoder"),
                });

        self.staging.recall();

        // actualiza las texturas modificadas o cargadas por egui en este fotograma
        for (id, image_delta) in &textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, image_delta);
        }

        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            primitives,
            &self.screen_descriptor,
        );

        {
            // redirecciona el renderizado al búffer MSAA si está activo
            let color_attachment = if let Some(ref msaa_view) = self.msaa_view {
                RenderPassColorAttachment {
                    view: msaa_view,
                    resolve_target: Some(&view),
                    ops: Operations {
                        load: LoadOp::Clear(self.clear_color),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                }
            } else {
                RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(self.clear_color),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                }
            };

            let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("RenderPass"),
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            // olvida el lifetime del render pass permitiendo que egui encadene sus comandos internos
            let mut render_pass = render_pass.forget_lifetime();

            self.egui_renderer
                .render(&mut render_pass, primitives, &self.screen_descriptor);
        }

        self.staging.finish();

        self.queue.submit(Some(encoder.finish()));

        // libera las texturas que egui vaya descartando durante ese fotograma
        for id in textures_delta.free.iter() {
            self.egui_renderer.free_texture(id);
        }

        frame.present();
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        debug!("Renderer drop ha empzado");
        // asegura que la GPU haya terminado los trabajos pendientes antes de destruir los recursos
        let _ = self.device.poll(egui_wgpu::wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_millis(50)),
        });
        debug!("Renderer drop ha terminado");
    }
}
