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

use std::{collections::HashMap, path::PathBuf, sync::Arc, time::UNIX_EPOCH};
use sha2::{Digest, Sha256};
use tokio::sync::{RwLock, Semaphore};
use uuid::Uuid;
use crate::{core::system::{cache::cache_manager::CacheManager, clipboard::TOKIO_RUNTIME}, utils::channel_pool::{ NotifyingSender, UiEvent, with_channel_pool}};




pub enum ThumbnailMessages {
    RequestThumb(PathBuf),
}

#[derive(Clone)]
pub struct Thumbnail {
    pub pixels: Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}


pub struct ThumbnailManager {
    pub thumb_map: Arc<RwLock<HashMap<PathBuf, Thumbnail>>>,
    pub semaphore: Arc<Semaphore>,
}

impl ThumbnailManager {
    pub fn new() -> Self {
        let manager = Self {
            thumb_map: Arc::new(RwLock::new(HashMap::new())), 
            semaphore: Arc::new(Semaphore::new(4)), 
        };

        TOKIO_RUNTIME.spawn(async {
            ThumbnailManager::cleanup_orphans().await;
        });

        manager
    }

    fn thumb_cache_dir() -> PathBuf {
        CacheManager::global().cache_dir.join("thumbs")
    }

    fn path_hash(path: &PathBuf) -> String {
        let mut hasher = Sha256::new();
        hasher.update(path.to_string_lossy().as_bytes());
        hasher.finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }

    fn cache_path_for(path: &PathBuf) -> PathBuf {
        Self::thumb_cache_dir().join(format!("{}.bin", Self::path_hash(path)))
    }

    fn get_real_mtime(path: &PathBuf) -> u64 {
        std::fs::metadata(path)
            .and_then(|m| m.modified())
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
            .unwrap_or(0)
    }



    fn is_image(path: &PathBuf) -> bool {
        matches!(
            path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()).as_deref(),
            Some("png" | "jpg" | "jpeg" | "webp" | "gif")
        )
    }

    fn is_svg(path: &PathBuf) -> bool {
        path.extension().and_then(|e| e.to_str())
            .map(|e| e.to_lowercase()) .as_deref() == Some("svg")
    }

    fn is_video(path: &PathBuf) -> bool {
        matches!(
            path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase()).as_deref(),
            Some("mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv")
        )
    }




    pub fn process_messages(&self, active_id: Uuid, sender: NotifyingSender) {
        let messages: Vec<ThumbnailMessages> = with_channel_pool(|pool| {
            let mut msgs = Vec::new();
            pool.process_thumbnail_events(active_id, |msg| {
                msgs.push(msg);
                true
            });
            msgs
        });

        for msg in messages {
            match msg {
                ThumbnailMessages::RequestThumb(path) => {
                    if path.starts_with(Self::thumb_cache_dir()) {
                        continue;
                    }

                    if !Self::is_image(&path) && !Self::is_svg(&path) && !Self::is_video(&path) {
                        continue;
                    }

                    if !Self::is_image(&path) && 
                        !Self::is_svg(&path) && 
                        !Self::is_video(&path) {
                        continue;
                    }

                    let current_mtime = Self::get_real_mtime(&path);
                    let cache_path = Self::cache_path_for(&path);

                    let cache_valid = if cache_path.exists() {
                        //guardando el mtime en un archivo llamado meta
                        let meta_path = cache_path.with_extension("meta");
                        std::fs::read_to_string(&meta_path)
                            .ok()
                            .and_then(|s| s.lines().next()?.trim().parse::<u64>().ok())
                            .map(|cached_mtime| cached_mtime == current_mtime)
                            .unwrap_or(false)
                    } else {
                        false
                    };

                    let thumb_map = self.thumb_map.clone();
                    let sender_clone = sender.clone();
                    let tab_id = sender.tab_id;
                    let sem = self.semaphore.clone();

                    if cache_valid {
                        TOKIO_RUNTIME.spawn(async move {
                            let _permit = sem.acquire_owned().await.unwrap();
                            //lee la imagen en cache
                            if let Ok(bytes) = tokio::fs::read(&cache_path).await {

                                if bytes.len() > 8 {
                                    //carga la imagen y genera el thumnail
                                    let w = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
                                    let h = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
                                    let pixels = bytes[8..].to_vec();

                                    let thumb = Thumbnail {
                                        pixels: Arc::new(pixels),
                                        width: w,
                                        height: h,
                                    };

                                    //lo guarda
                                    if let Ok(mut map) = thumb_map.try_write() {
                                        map.insert(path.clone(), thumb);
                                    }

                                    //envia el mensaje al state
                                    sender_clone.send_ui_event(UiEvent::ThumbnailReady {
                                        full_path: path,
                                        tab_id,
                                    }).ok();
                                }
                            }
                        });
                    } else {
                        TOKIO_RUNTIME.spawn(async move {
                            let _permit = sem.acquire_owned().await.unwrap();

                            //genera el thumnail dependiendo del tipo
                            let thumb = if Self::is_image(&path) {
                                Self::generate_image_thumb(&path).await
                            } else if Self::is_svg(&path) {
                                Self::generate_svg_thumb(&path).await
                            } else {
                                Self::generate_video_thumb(&path).await
                            };

                            if let Some(thumb) = thumb {
                                // Guardar en disco
                                if let Some(parent) = cache_path.parent() {
                                    tokio::fs::create_dir_all(parent).await.ok();
                                }

                                // Guardar binario
                                let pixels = thumb.pixels.clone();
                                let (w, h) = (thumb.width, thumb.height);
                                let cache_path_clone = cache_path.clone();
                                tokio::task::spawn_blocking(move || {
                                    let mut buf = Vec::with_capacity(8 + pixels.len());
                                    buf.extend_from_slice(&w.to_le_bytes());
                                    buf.extend_from_slice(&h.to_le_bytes());
                                    buf.extend_from_slice(&pixels);
                                    std::fs::write(&cache_path_clone, buf).ok()
                                }).await.ok();

                                // Guardar mtime en .meta
                                let meta_path = cache_path.with_extension("meta");

                                // al guardar el thumbnail
                                let meta_content = format!("{}\n{}", current_mtime, path.to_string_lossy());
                                tokio::fs::write(&meta_path, meta_content).await.ok();

                                if let Ok(mut map) = thumb_map.try_write() {
                                    map.insert(path.clone(), thumb);
                                }
                                sender_clone.send_ui_event(UiEvent::ThumbnailReady {
                                    full_path: path,
                                    tab_id,
                                }).ok();
                            }
                        });
                    }
                },
            }
        }
    }


    async fn generate_image_thumb(path: &PathBuf) -> Option<Thumbnail> {
        let path = path.clone();
        tokio::task::spawn_blocking(move || {
            let img = image::open(&path).ok()?;
            let resized = img.thumbnail(64, 64);
            let rgba = resized.to_rgba8();
            let (w, h) = rgba.dimensions();

            Some(Thumbnail {
                pixels: Arc::new(rgba.into_raw()),
                width: w,
                height: h,
            })
        }).await.ok()?
    }

    async fn generate_svg_thumb(path: &PathBuf) -> Option<Thumbnail> {
        let path = path.clone();
        tokio::task::spawn_blocking(move || {
            let data = std::fs::read(&path).ok()?;
            let opt = resvg::usvg::Options::default();
            let tree = resvg::usvg::Tree::from_data(&data, &opt).ok()?;
            let mut pixmap = resvg::tiny_skia::Pixmap::new(64, 64)?;
            let transform = resvg::tiny_skia::Transform::from_scale(
                64.0 / tree.size().width(),
                64.0 / tree.size().height(),
            );
            resvg::render(&tree, transform, &mut pixmap.as_mut());

            Some(Thumbnail {
                pixels: Arc::new(pixmap.data().to_vec()),
                width: 64,
                height: 64,
            })
        }).await.ok()?
    }

    async fn generate_video_thumb(path: &PathBuf) -> Option<Thumbnail> {
        let path_str = path.to_string_lossy().to_string();

        let output = tokio::process::Command::new("ffmpeg")
            .args([
                "-ss", "00:00:01",
                "-i", &path_str,
                "-vframes", "1",
                "-f", "image2pipe",
                "-vcodec", "png",
                "-"
            ])
            .output()
            .await
            .ok()?;

        if !output.status.success() || output.stdout.is_empty() {
            let output = tokio::process::Command::new("ffmpeg")
                .args([
                    "-i", &path_str,
                    "-vframes", "1",
                    "-f", "image2pipe",
                    "-vcodec", "png",
                    "-"
                ])
                .output()
                .await
                .ok()?;

            if output.stdout.is_empty() {
                return None;
            }

            let img = image::load_from_memory(&output.stdout).ok()?;
            let resized = img.thumbnail(64, 64);
            let rgba = resized.to_rgba8();
            let (w, h) = rgba.dimensions();
            return Some(Thumbnail {
                pixels: Arc::new(rgba.into_raw()),
                width: w,
                height: h,
            });
        }

        let img = image::load_from_memory(&output.stdout).ok()?;
        let resized = img.thumbnail(64, 64);
        let rgba = resized.to_rgba8();
        let (w, h) = rgba.dimensions();
        return Some(Thumbnail {
            pixels: Arc::new(rgba.into_raw()),
            width: w,
            height: h,
        });
    }

    pub async fn cleanup_orphans() {
        let dir = Self::thumb_cache_dir();
        if !dir.exists() { return; }

        let Ok(mut entries) = tokio::fs::read_dir(&dir).await else { return };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let meta_path = entry.path();
            if meta_path.extension().and_then(|e| e.to_str()) != Some("meta") {
                continue;
            }

            // esto lee "mtime\n/path/original"
            let Ok(content) = tokio::fs::read_to_string(&meta_path).await else { continue };
            let mut lines = content.lines();
            let Some(_) = lines.next() else { continue };
            let Some(orig_str) = lines.next() else { continue };

            let orig_path = PathBuf::from(orig_str);
            if !orig_path.exists() {
                // borra el bin y el meta
                let bin_path = meta_path.with_extension("bin");
                tokio::fs::remove_file(&bin_path).await.ok();
                tokio::fs::remove_file(&meta_path).await.ok();
            }
        }
    }

}