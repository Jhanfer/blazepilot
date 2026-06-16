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

use crate::{
    core::system::clipboard::global_clipboard::TOKIO_RUNTIME,
    ui::{
        icons_cache::thumbnails::utils::resolve_tiff_data,
        image_preview::error::{ImagePreviewError, ImagePreviewResult},
    },
};
use crossbeam_channel::{bounded, Receiver};
use egui::{ColorImage, TextureHandle, TextureOptions, Ui, Vec2};
use std::{
    io::Read,
    path::{Path, PathBuf},
};
use tracing::warn;

pub struct ImagePreviewState {
    pub image_paths: Vec<PathBuf>,
    pub current_index: usize,
    pub prev_texture: Option<TextureHandle>,
    pub current_texture: Option<TextureHandle>,
    pub next_texture: Option<TextureHandle>,
    pub prev_rx: Option<Receiver<Option<ColorImage>>>,
    pub current_rx: Option<Receiver<Option<ColorImage>>>,
    pub next_rx: Option<Receiver<Option<ColorImage>>>,
    pub loading: bool,
    pub zoom: f32,
    pub offset: Vec2,
}

impl ImagePreviewState {
    pub fn new(clicked_path: PathBuf, all_images: Vec<PathBuf>) -> Self {
        let current_index = all_images
            .iter()
            .position(|p| p == &clicked_path)
            .unwrap_or(0);

        Self {
            image_paths: all_images,
            current_index,
            prev_texture: None,
            current_texture: None,
            next_texture: None,
            prev_rx: None,
            current_rx: None,
            next_rx: None,
            loading: true,
            zoom: 1.0,
            offset: Vec2::ZERO,
        }
    }

    pub fn cleanup(&mut self) {
        self.prev_texture = None;
        self.current_texture = None;
        self.next_texture = None;
        self.prev_rx = None;
        self.current_rx = None;
        self.next_rx = None;
        self.zoom = 1.0;
        self.offset = Vec2::ZERO;
    }

    pub fn initial_load(&mut self, ui: &mut Ui) {
        if self.image_paths.is_empty() {
            self.loading = false;
            return;
        }

        //imagen actual
        self.current_rx = Self::spawn_load(self.path_at(self.current_index), ui);

        //precargar la imagen siguiente
        if self.image_paths.len() > 1 {
            self.next_rx = Self::spawn_load(self.path_at(self.next_index()), ui);
        }

        self.loading = true;
    }

    pub fn next(&mut self, ui: &mut Ui) {
        if self.image_paths.len() <= 1 {
            return;
        }

        //rotar texturas
        self.prev_texture = self.current_texture.take();
        self.current_texture = self.next_texture.take();
        self.current_index = self.next_index();

        self.prev_rx = None;
        self.current_rx = self.next_rx.take();

        self.next_rx = Self::spawn_load(self.path_at(self.next_index()), ui);

        self.loading = self.current_texture.is_none();
    }

    pub fn prev(&mut self, ui: &mut Ui) {
        if self.image_paths.len() <= 1 {
            return;
        }

        //rotar texturas al revés
        self.next_texture = self.current_texture.take();
        self.current_texture = self.prev_texture.take();
        self.current_index = self.prev_index();

        self.next_rx = None;
        self.current_rx = self.prev_rx.take();

        self.prev_rx = Self::spawn_load(self.path_at(self.prev_index()), ui);

        self.loading = self.current_texture.is_none();
    }

    pub fn poll_loading(&mut self, ui: &mut Ui) {
        Self::poll_rx(
            &mut self.current_rx,
            &mut self.current_texture,
            &mut self.loading,
            "preview_current",
            ui,
        );

        Self::poll_rx_silent(
            &mut self.prev_rx,
            &mut self.prev_texture,
            "preview_prev",
            ui,
        );

        Self::poll_rx_silent(
            &mut self.next_rx,
            &mut self.next_texture,
            "preview_next",
            ui,
        );
    }

    fn poll_rx(
        rx: &mut Option<Receiver<Option<ColorImage>>>,
        texture: &mut Option<TextureHandle>,
        loading: &mut bool,
        name: &str,
        ui: &mut Ui,
    ) {
        if let Some(r) = rx {
            if let Ok(Some(img)) = r.try_recv() {
                *texture = Some(ui.load_texture(name, img, TextureOptions::LINEAR));
                *loading = false;
                *rx = None;
            }
        }
    }

    fn poll_rx_silent(
        rx: &mut Option<Receiver<Option<ColorImage>>>,
        texture: &mut Option<TextureHandle>,
        name: &str,
        ui: &mut Ui,
    ) {
        if let Some(r) = rx {
            if let Ok(Some(img)) = r.try_recv() {
                *texture = Some(ui.load_texture(name, img, TextureOptions::LINEAR));
                *rx = None;
            }
        }
    }

    fn img_handler(path: &Path) -> ImagePreviewResult<ColorImage> {
        let mut buffer = Vec::new();
        let mut full_file = std::fs::File::open(path).map_err(ImagePreviewError::Io)?;
        full_file
            .read_to_end(&mut buffer)
            .map_err(ImagePreviewError::Io)?;

        let (raw, w, h) = match stb_image::image::load_from_memory_with_depth(&buffer, 4, false) {
            stb_image::image::LoadResult::ImageU8(image) => {
                (image.data, image.width as u32, image.height as u32)
            }
            _ => return Err(ImagePreviewError::ImageError),
        };

        Ok(ColorImage::from_rgba_unmultiplied(
            [w as usize, h as usize],
            &raw,
        ))
    }

    fn svg_handler(path: &Path) -> ImagePreviewResult<ColorImage> {
        let data = std::fs::read(path)?;
        let opt = resvg::usvg::Options::default();
        let tree = resvg::usvg::Tree::from_data(&data, &opt)?;
        let size = tree.size();

        const MAX_SIZE: u32 = 1024;

        let scale = (MAX_SIZE as f32 / size.width()).min(MAX_SIZE as f32 / size.height());

        let target_width = (size.width() * scale).ceil() as u32;
        let target_height = (size.height() * scale).ceil() as u32;

        let mut pixmap = resvg::tiny_skia::Pixmap::new(target_width, target_height)
            .ok_or(ImagePreviewError::DimensionError)?;

        let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);

        resvg::render(&tree, transform, &mut pixmap.as_mut());

        let image_data = pixmap.take();

        Ok(ColorImage::from_rgba_unmultiplied(
            [target_width as usize, target_height as usize],
            &image_data,
        ))
    }

    fn tiff_handler(path: &Path) -> ImagePreviewResult<ColorImage> {
        let mut buffer = Vec::new();
        let mut full_file = std::fs::File::open(path).map_err(ImagePreviewError::Io)?;

        full_file
            .read_to_end(&mut buffer)
            .map_err(ImagePreviewError::Io)?;
        let cursor = std::io::Cursor::new(&buffer);

        let mut decoder =
            tiff::decoder::Decoder::new(cursor).map_err(|_| ImagePreviewError::ImageError)?;

        let (w, h) = decoder
            .dimensions()
            .map_err(|_| ImagePreviewError::ImageError)?;

        let rgba_buf = decoder
            .read_image()
            .map_err(|_| ImagePreviewError::ImageError)?;

        let rgb_data = match resolve_tiff_data(rgba_buf, w, h) {
            Ok(raw) => raw,
            Err(_) => return Err(ImagePreviewError::ImageError),
        };

        Ok(ColorImage::from_rgba_unmultiplied(
            [w as usize, h as usize],
            &rgb_data,
        ))
    }

    fn webp_handler(path: &Path) -> ImagePreviewResult<ColorImage> {
        let mut buffer = Vec::new();
        let mut full_file = std::fs::File::open(path).map_err(ImagePreviewError::Io)?;

        full_file
            .read_to_end(&mut buffer)
            .map_err(ImagePreviewError::Io)?;
        let cursor = std::io::Cursor::new(&buffer);

        let mut decoder =
            image_webp::WebPDecoder::new(cursor).map_err(|_| ImagePreviewError::ImageError)?;

        let (w, h) = decoder.dimensions();

        let has_alpha = decoder.has_alpha();
        let bytes_per_pixel = if has_alpha { 4 } else { 3 };
        let mut raw_buf = vec![0u8; (w * h * bytes_per_pixel) as usize];

        decoder
            .read_image(&mut raw_buf)
            .map_err(|_| ImagePreviewError::ImageError)?;

        let rgba_buf = if !has_alpha {
            let mut converted = Vec::with_capacity((w * h * 4) as usize);

            for chunk in raw_buf.chunks_exact(3) {
                converted.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
            converted
        } else {
            raw_buf
        };

        Ok(ColorImage::from_rgba_unmultiplied(
            [w as usize, h as usize],
            &rgba_buf,
        ))
    }

    fn avif_handler(path: &Path) -> ImagePreviewResult<ColorImage> {
        let mut buffer = Vec::new();
        let mut full_file = std::fs::File::open(path).map_err(ImagePreviewError::Io)?;

        full_file
            .read_to_end(&mut buffer)
            .map_err(ImagePreviewError::Io)?;

        let dynamic_image =
            libavif_image::read(&buffer).map_err(|_| ImagePreviewError::ImageError)?;

        let w = dynamic_image.width();
        let h = dynamic_image.height();

        let rgba_buf = dynamic_image.to_rgba8().into_raw();

        Ok(ColorImage::from_rgba_unmultiplied(
            [w as usize, h as usize],
            &rgba_buf,
        ))
    }

    fn spawn_load(path: Option<PathBuf>, ui: &mut Ui) -> Option<Receiver<Option<ColorImage>>> {
        let path = path.unwrap_or_default();
        let ui_clone = ui.clone();
        let (tx, rx) = bounded(1);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        TOKIO_RUNTIME.spawn(async move {
            let image = tokio::task::spawn_blocking(move || {
                if ext.as_str() == "svg" {
                    return Self::svg_handler(&path);
                }

                if ext.as_str() == "webp" {
                    return Self::webp_handler(&path);
                }

                if ext.as_str() == "avif" {
                    return Self::avif_handler(&path);
                }

                let is_tiff = matches!(ext.as_str(), "tif" | "tiff",);

                let is_image_ext = matches!(
                    ext.as_str(),
                    "png" | "jpg" | "jpeg" | "pbm" | "pgm" | "ppm" | "pnm" | "bmp" | "ico"
                );

                if is_tiff {
                    return Self::tiff_handler(&path);
                }

                if is_image_ext {
                    Self::img_handler(&path)
                } else {
                    Err(ImagePreviewError::UnsuportedFormat)
                }
            })
            .await;

            match image {
                Ok(Ok(color_image)) => {
                    if let Err(e) = tx.send(Some(color_image)) {
                        warn!("Error enviando la preview: {:?}", e);
                    };
                }
                Ok(Err(preview_error)) => {
                    warn!("Error del handler: {:?}", preview_error);
                }
                Err(join_err) => {
                    warn!("Panic en spawn_blocking: {join_err}");
                }
            }

            ui_clone.request_repaint();
        });

        Some(rx)
    }

    fn path_at(&self, index: usize) -> Option<PathBuf> {
        self.image_paths.get(index).cloned()
    }

    fn next_index(&self) -> usize {
        (self.current_index + 1) % self.image_paths.len()
    }

    fn prev_index(&self) -> usize {
        self.current_index
            .checked_sub(1)
            .unwrap_or(self.image_paths.len() - 1)
    }

    pub fn current_name(&self) -> &str {
        self.image_paths
            .get(self.current_index)
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("?")
    }
}
