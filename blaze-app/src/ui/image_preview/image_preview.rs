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




use std::path::PathBuf;
use crossbeam_channel::{Receiver, bounded};
use egui::{ColorImage, TextureHandle, TextureOptions, Ui, Vec2};
use crate::core::system::clipboard::TOKIO_RUNTIME;


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
        let current_index = all_images.iter()
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
        if self.image_paths.len() <= 1 {return;}

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
        if self.image_paths.len() <= 1 {return;}

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
        Self::poll_rx(&mut self.current_rx, &mut self.current_texture, &mut self.loading, "preview_current", ui);

        Self::poll_rx_silent(&mut self.prev_rx, &mut self.prev_texture, "preview_prev", ui);

        Self::poll_rx_silent(&mut self.next_rx, &mut self.next_texture, "preview_next", ui);
    }


    fn poll_rx(rx: &mut Option<Receiver<Option<ColorImage>>>, texture: &mut Option<TextureHandle>, loading: &mut bool, name: &str, ui: &mut Ui) {
        if let Some(r) = rx {
            if let Ok(Some(img)) = r.try_recv() {
                *texture = Some(ui.load_texture(name, img, TextureOptions::LINEAR));
                *loading = false;
                *rx = None;
            }
        }
    }


    fn poll_rx_silent(rx: &mut Option<Receiver<Option<ColorImage>>>, texture: &mut Option<TextureHandle>, name: &str, ui: &mut Ui) {
        if let Some(r) = rx {
            if let Ok(Some(img)) = r.try_recv() {
                *texture = Some(ui.load_texture(name, img, TextureOptions::LINEAR));
                *rx = None;
            }
        }
    }

    fn img_handler(path: PathBuf) -> Option<ColorImage> {
        let img = image::open(&path).ok()?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        Some(
            ColorImage::from_rgba_unmultiplied(
                [w as usize, h as usize],
                &rgba.into_raw(),
            )
        )
    }
    
    fn svg_handler(path: PathBuf) -> Option<ColorImage> {
        let data = std::fs::read(&path).ok()?;
        let opt = resvg::usvg::Options::default();
        let tree = resvg::usvg::Tree::from_data(&data, &opt).ok()?;
        let size = tree.size();

        const MAX_SIZE: u32 = 1024;
        
        let scale = (MAX_SIZE as f32 / size.width())
            .min(MAX_SIZE as f32 / size.height());

        let target_width = (size.width() * scale).ceil() as u32;
        let target_height = (size.height() * scale).ceil() as u32;

        let mut pixmap = resvg::tiny_skia::Pixmap::new(target_width, target_height)?;

        let transform = resvg::tiny_skia::Transform::from_scale(scale, scale);
        
        resvg::render(&tree, transform, &mut pixmap.as_mut());

        let image_data = pixmap.take();

        Some(
            ColorImage::from_rgba_unmultiplied(
                [target_width as usize, target_height as usize],
                &image_data,
            )
        )
    }

    fn spawn_load(path: Option<PathBuf>, ui: &mut Ui,) -> Option<Receiver<Option<ColorImage>>> {
        let path = path?;
        let ui_clone = ui.clone();
        let (tx, rx) = bounded(1);
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        TOKIO_RUNTIME.spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                let res = match ext.as_str() {
                    "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp" | "tiff" => {
                        Self::img_handler(path)
                    },

                    "svg" => Self::svg_handler(path),

                    _ => {None},
                };

                res
            })
            .await
            .ok()
            .flatten();

            tx.send(result).ok();
            ui_clone.request_repaint();
            Some(())
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