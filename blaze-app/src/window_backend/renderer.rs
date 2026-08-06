use crate::core::bootstrap::configs::platform::linux::conf_structs::DisplayBackend;
use egui_wgpu::wgpu::{
    CompositeAlphaMode, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
    InstanceDescriptor, LoadOp, Operations, PresentMode as WgpuPresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface,
    SurfaceConfiguration, TextureUsages,
};
use egui_winit::winit::window::Window;
use std::{sync::Arc, time::Duration};
use tracing::debug;

const MAX_PRIMITIVES_PER_FRAME: usize = 512;

#[derive(Debug, Clone)]
pub struct RendererConfig {
    pub renderer: DisplayBackend,
    pub power_preference: egui_wgpu::wgpu::PowerPreference,
    pub present_mode: egui_wgpu::wgpu::PresentMode,
    pub transparency: bool,
}

impl RendererConfig {
    pub fn wgpu_present(
        renderer: DisplayBackend,
        present_mode: egui_wgpu::wgpu::PresentMode,
        transparency: bool,
    ) -> Self {
        Self {
            renderer,
            present_mode,
            power_preference: egui_wgpu::wgpu::PowerPreference::LowPower,
            transparency,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PresentMode {
    Immediate,
    Fifo,
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

pub struct Renderer {
    surface: Surface<'static>,
    pending_free: Vec<egui::TextureId>,
    device: egui_wgpu::wgpu::Device,
    queue: Queue,
    config: SurfaceConfiguration,
    egui_renderer: egui_wgpu::Renderer,
    screen_descriptor: egui_wgpu::ScreenDescriptor,
}

impl Renderer {
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
                required_limits: adapter.limits(),
                memory_hints: egui_wgpu::wgpu::MemoryHints::Performance,
                trace: Default::default(),
                experimental_features: ExperimentalFeatures::disabled(),
            })
            .await
            .map_err(|e| anyhow::anyhow!("Ha ocurrido un error al pedir el divice: {e}"))?;

        let caps = surface.get_capabilities(&adapter);

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
            present_mode: render_config.present_mode,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let egui_renderer =
            egui_wgpu::Renderer::new(&device, format, egui_wgpu::RendererOptions::default());

        Ok(Self {
            surface,
            pending_free: Vec::new(),
            device,
            queue,
            config,
            egui_renderer,
            screen_descriptor,
        })
    }

    pub fn max_texture_side(&self) -> usize {
        self.device.limits().max_texture_dimension_2d as usize
    }

    pub fn resize(&mut self, width: u32, height: u32, pixels_per_point: f32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;

        self.surface.configure(&self.device, &self.config);

        self.screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [width, height],
            pixels_per_point,
        };
    }

    pub fn render(
        &mut self,
        primitives: &[egui::ClippedPrimitive],
        textures_delta: &egui::TexturesDelta,
    ) {
        for id in self.pending_free.drain(..) {
            self.egui_renderer.free_texture(&id);
        }

        let primitives = &primitives[..primitives.len().min(MAX_PRIMITIVES_PER_FRAME)];

        let frame = match self.surface.get_current_texture() {
            egui_wgpu::wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            egui_wgpu::wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
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
            let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("RenderPass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(egui_wgpu::wgpu::Color {
                            r: 0.12,
                            g: 0.08,
                            b: 0.20,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });

            let mut render_pass = render_pass.forget_lifetime();

            self.egui_renderer
                .render(&mut render_pass, primitives, &self.screen_descriptor);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();

        self.pending_free
            .extend(textures_delta.free.iter().copied());
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        debug!("Renderer drop ha empzado");
        let _ = self.device.poll(egui_wgpu::wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_millis(500)),
        });
        debug!("Renderer drop ha terminado");
    }
}
