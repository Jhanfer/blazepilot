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





use std::collections::HashMap;
use egui::{ColorImage, Context, TextureHandle, vec2};
use resvg::usvg::Options;

pub struct IconCache {
    cache: HashMap<String, TextureHandle>,
}

impl IconCache {
    pub fn new() -> Self {
        Self { cache: HashMap::new() }
    }

    pub fn get_or_load(&mut self, ctx: &Context, name: &str, svg_bytes: &[u8], tint: egui::Color32) -> &TextureHandle {
        self.cache.entry(name.to_string()).or_insert_with(|| {
            let image = rasterize_svg(svg_bytes, 16, 16, tint);
            ctx.load_texture(name, image, egui::TextureOptions::LINEAR)
        })
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
        .map(|p| {
            egui::Color32::from_rgba_unmultiplied(
                tint.r(),
                tint.g(),
                tint.b(),
                p.alpha(),
            )
    }).collect();

    ColorImage {
        size: [width as usize, height as usize],
        pixels,
        source_size: vec2(width as f32, height as f32),
    }
}