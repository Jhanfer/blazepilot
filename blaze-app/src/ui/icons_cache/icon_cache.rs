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

use egui::{ColorImage, TextureHandle, TextureOptions, Ui, vec2};
use lru::LruCache;
use resvg::usvg::Options;
use std::num::NonZeroUsize;

pub struct IconCache {
    cache: LruCache<String, TextureHandle>,
}

impl IconCache {
    pub fn new(max_entries: usize) -> Self {
        let def_cap = match NonZeroUsize::new(500) {
            Some(cap) => cap,
            None => unreachable!(),
        };
        let cap = NonZeroUsize::new(max_entries).unwrap_or(def_cap);

        Self {
            cache: LruCache::new(cap),
        }
    }

    pub fn get_or_load(
        &mut self,
        ui: &mut Ui,
        name: &str,
        svg_bytes: &[u8],
        tint: egui::Color32,
        icon_size: egui::Vec2,
    ) -> TextureHandle {
        let tint_key = format!("{:02X}{:02X}{:02X}", tint.r(), tint.g(), tint.b());
        let pixels_per_point = ui.pixels_per_point();
        let w = (icon_size.x * pixels_per_point).round() as u32;
        let h = (icon_size.y * pixels_per_point).round() as u32;
        let scale_key = (pixels_per_point * 10.0).round() as u32;
        let full_key = format!("{}-{}-{}x{}-{}ppp", name, tint_key, w, h, scale_key);

        if let Some(texture) = self.cache.get(&full_key) {
            return texture.clone();
        }

        let image = rasterize_svg(svg_bytes, w, h, tint);
        let texture = ui.load_texture(name, image, TextureOptions::LINEAR);

        self.cache.push(full_key, texture.clone());

        texture
    }
}

fn rasterize_svg(svg_bytes: &[u8], width: u32, height: u32, tint: egui::Color32) -> ColorImage {
    let opt = Options::default();
    let tree = resvg::usvg::Tree::from_data(svg_bytes, &opt).unwrap();

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).unwrap();

    let transform = resvg::tiny_skia::Transform::from_scale(
        width as f32 / tree.size().width(),
        height as f32 / tree.size().height(),
    );

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let pixels: Vec<egui::Color32> = pixmap
        .pixels()
        .iter()
        .map(|p| egui::Color32::from_rgba_unmultiplied(tint.r(), tint.g(), tint.b(), p.alpha()))
        .collect();

    ColorImage {
        size: [width as usize, height as usize],
        pixels,
        source_size: vec2(width as f32, height as f32),
    }
}
