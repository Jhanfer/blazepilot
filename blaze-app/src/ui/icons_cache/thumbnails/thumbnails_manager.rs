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

use std::{path::PathBuf, sync::Arc, time::UNIX_EPOCH};
use ffmpeg_sidecar::{download::auto_download, paths::ffmpeg_path};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::sync::{RwLock, Semaphore};
use tracing::{error};
use uuid::Uuid;
use crate::{core::system::{cache::cache_manager::CacheManager, clipboard::TOKIO_RUNTIME}, utils::channel_pool::{ NotifyingSender, UiEvent, with_channel_pool}};
use lru::LruCache;
use std::num::NonZeroUsize;


const DEFAULT_THUMB_CACHE_CAPACITY: usize = 400;


#[derive(Debug, Error)]
pub enum ThumbError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error en trhead Tokio: {0}")]
    ThreadError(#[from] tokio::task::JoinError),

    #[error("Error cargando imagen")]
    ImageError,

    #[error("Error procesando SVG")]
    SvgError,

    #[error("Error generando thumbnail de video")]
    VideoError,

    #[error("Slice error")]
    SliceError(#[from] std::array::TryFromSliceError),

    #[error("Directorio de caché de miniaturas no existe")]
    ThumbsDirDoesNotExist,
}


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
    pub thumb_map: Arc<RwLock<LruCache<PathBuf, Thumbnail>>>,
    pub semaphore: Arc<Semaphore>,
}

impl ThumbnailManager {
    pub fn new() -> Self {
        let manager = Self::with_capacity(DEFAULT_THUMB_CACHE_CAPACITY);

        TOKIO_RUNTIME.spawn(async {
            if let Err(e) = ThumbnailManager::cleanup_orphans().await {
                error!("cleanup failed: {}", e);
            }
        });

        manager
    }

    fn with_capacity(cap: usize) -> Self {
        let def_cap: NonZeroUsize = match NonZeroUsize::new(400) {
            Some(n) => n,
            None => unreachable!(),
        };
        let cap = NonZeroUsize::new(cap)
            .unwrap_or(def_cap);

        Self {
            thumb_map: Arc::new(RwLock::new(LruCache::new(cap))),
            semaphore: Arc::new(Semaphore::new(4)), 
        }
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



    fn is_cache_valid(cache_path: &PathBuf, current_mtime: u64) -> bool {
        if !cache_path.exists() {
            return false;
        }

        let meta_path = cache_path.with_extension("meta");
        let Ok(content) = std::fs::read_to_string(&meta_path) else {
            return false;
        };

        let mut lines = content.lines();
        let cached_mtime: u64 = lines.next()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or_else(|| 0);

        cached_mtime == current_mtime
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

                    if !Self::is_image(&path) && 
                        !Self::is_svg(&path) && 
                        !Self::is_video(&path) {
                        continue;
                    }

                    let thumb_map = self.thumb_map.clone();
                    let sender_clone = sender.clone();
                    let tab_id = sender.tab_id;
                    let sem = self.semaphore.clone();

                    let current_mtime = Self::get_real_mtime(&path);
                    let cache_path = Self::cache_path_for(&path);

                    if Self::is_cache_valid(&cache_path, current_mtime) {
                        let thumb_map = thumb_map.clone();
                        TOKIO_RUNTIME.spawn(async move {
                            let Ok(_permit) = sem
                                .acquire_owned()
                                .await 
                            else {
                                return;
                            };

                            //lee la imagen en cache
                            if let Ok(thumb) = Self::load_from_cache(&cache_path).await {
                                thumb_map.write().await.put(path.clone(), thumb);
                                sender_clone.send_ui_event(UiEvent::ThumbnailReady {
                                    full_path: path,
                                    tab_id,
                                }).ok();
                            }

                        });
                    } else {
                        TOKIO_RUNTIME.spawn(async move {
                            let Ok(_permit) = sem
                                .acquire_owned()
                                .await 
                            else {
                                return;
                            };

                            //genera el thumnail dependiendo del tipo
                            let thumb = if Self::is_image(&path) {
                                Self::generate_image_thumb(&path).await
                            } else if Self::is_svg(&path) {
                                Self::generate_svg_thumb(&path).await
                            } else {
                                Self::generate_video_thumb(&path).await
                            };

                            if let Ok(thumb) = thumb {
                                // Guardar en cache
                                if let Err(e) = Self::save_to_cache(
                                    &cache_path,
                                    &thumb,
                                    current_mtime,
                                    &path
                                ).await {
                                    let err = format!("Error en el caché de miniaturas: {}", e);
                                    error!(err);
                                    sender_clone.send_ui_event(UiEvent::ShowError(
                                        err
                                    )).ok();
                                }

                                thumb_map.write().await.put(path.clone(), thumb);
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

    async fn load_from_cache(cache_path: &PathBuf) -> Result<Thumbnail, ThumbError>  {
        let bytes = tokio::fs::read(cache_path)
            .await
            .map_err(ThumbError::Io)?;

        if bytes.len() < 8 {
            return Err(ThumbError::ImageError);
        }

        let w = u32::from_le_bytes(
            bytes[0..4]
                .try_into()
                .map_err(|e| ThumbError::SliceError(e))?
        );
        let h = u32::from_le_bytes(
            bytes[4..8]
                .try_into()
                .map_err(|e| ThumbError::SliceError(e))?
        );

        if w == 0 || h == 0 || bytes.len() < 8 + (w as usize * h as usize * 4) {
            return Err(ThumbError::ImageError);
        }

        let pixels = bytes[8..].to_vec();

        Ok(
            Thumbnail {
                pixels: Arc::new(pixels),
                width: w,
                height: h,
            }
        )
    }


    async fn save_to_cache(cache_path: &PathBuf, thumb: &Thumbnail, mtime: u64, original_path: &PathBuf) -> Result<(), ThumbError> {
        if let Some(parent) = cache_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(ThumbError::Io)?;
        }

        let pixels = thumb.pixels.clone();
        let (w, h) = (thumb.width, thumb.height);
        let cache_path_clone = cache_path.clone();

        //guardar binario
        tokio::task::spawn_blocking(move || -> Result<(), ThumbError> {
            let mut buf = Vec::with_capacity(8 + pixels.len());
            buf.extend_from_slice(&w.to_le_bytes());
            buf.extend_from_slice(&h.to_le_bytes());
            buf.extend_from_slice(&pixels);
            std::fs::write(&cache_path_clone, buf)
                .map_err(ThumbError::Io)?;
            
            Ok(())
        })
        .await
        .map_err(|e| ThumbError::ThreadError(e))??;

        // Guardar meta
        let meta_path = cache_path.with_extension("meta");
        let meta_content = format!("{}\n{}", mtime, original_path.to_string_lossy());
        tokio::fs::write(&meta_path, meta_content)
            .await
            .map_err(ThumbError::Io)?;

        Ok(())
    }


    async fn generate_image_thumb(path: &PathBuf) -> Result<Thumbnail, ThumbError> {
        let path = path.clone();
        tokio::task::spawn_blocking(move || -> Result<Thumbnail, ThumbError> {
            let img = image::open(&path)
                .map_err(|_| ThumbError::ImageError)?;
            let resized = img.thumbnail(64, 64);
            let rgba = resized.to_rgba8();
            let (w, h) = rgba.dimensions();

            Ok(Thumbnail {
                pixels: Arc::new(rgba.into_raw()),
                width: w,
                height: h,
            })
        })
        .await
        .map_err(|e| ThumbError::ThreadError(e))?
    }

    async fn generate_svg_thumb(path: &PathBuf) -> Result<Thumbnail, ThumbError> {
        let path = path.clone();
        let out = tokio::task::spawn_blocking(
            move || {
                let data = std::fs::read(&path)
                    .map_err(ThumbError::Io)?;

                let opt = resvg::usvg::Options::default();

                let tree = resvg::usvg::Tree::from_data(&data, &opt)
                    .map_err(|_| ThumbError::ImageError)?;

                let mut pixmap = resvg::tiny_skia::Pixmap::new(64, 64)
                    .ok_or(ThumbError::SvgError)?;
                
                let transform = resvg::tiny_skia::Transform::from_scale(
                    64.0 / tree.size().width(),
                    64.0 / tree.size().height(),
                );
                resvg::render(&tree, transform, &mut pixmap.as_mut());

                Ok(Thumbnail {
                    pixels: Arc::new(pixmap.data().to_vec()),
                    width: 64,
                    height: 64,
                })
            }
        )
        .await
        .map_err(|e| ThumbError::ThreadError(e))?;

        out
    }


    async fn run_ffmpeg(args: &[&str]) -> Result<Vec<u8>, ThumbError> {
        let output = tokio::process::Command::new(ffmpeg_path())
            .args(args)
            .output()
            .await
            .map_err(ThumbError::Io)?;

        if !output.status.success() || output.stdout.is_empty() {
            return Err(ThumbError::VideoError);
        }

        Ok(output.stdout)
    }

    async fn generate_video_thumb(path: &PathBuf) -> Result<Thumbnail, ThumbError> {
        let path_str = path.to_string_lossy().to_string();

        auto_download().map_err(|_| ThumbError::VideoError)?;

        let output = match Self::run_ffmpeg(&[
                "-ss", "00:00:01",
                "-i", &path_str,
                "-vframes", "1",
                "-f", "image2pipe",
                "-vcodec", "png",
                "-"
            ])
            .await
        {
            Ok(out) => out,
            Err(_) => {
                let fallback = Self::run_ffmpeg(&[
                    "-i", &path_str,
                    "-vframes", "1",
                    "-f", "image2pipe",
                    "-vcodec", "png",
                    "-"
                ]).await?;
                fallback
            }
        };

        let img = image::load_from_memory(&output)
            .map_err(|_| ThumbError::VideoError)?;
        let resized = img.thumbnail(64, 64);
        let rgba = resized.to_rgba8();
        let (w, h) = rgba.dimensions();

        Ok(Thumbnail {
            pixels: Arc::new(rgba.into_raw()),
            width: w,
            height: h,
        })
    }


    pub async fn cleanup_orphans() -> Result<(), ThumbError> {
        let dir = Self::thumb_cache_dir();
        if !dir.exists() { return Err(ThumbError::ThumbsDirDoesNotExist); }

        let mut entries = tokio::fs::read_dir(&dir)
            .await
            .map_err(ThumbError::Io)?;

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
                tokio::fs::remove_file(&bin_path)
                    .await
                    .map_err(ThumbError::Io)?;
                tokio::fs::remove_file(&meta_path)
                    .await
                    .map_err(ThumbError::Io)?;
            }
        }

        Ok(())
    }

}



#[test]
fn test_path_hash_is_consistent() {
    let path = PathBuf::from("/home/test/image.png");

    let h1 = ThumbnailManager::path_hash(&path);
    let h2 = ThumbnailManager::path_hash(&path);

    assert_eq!(h1, h2);
}

#[test]
fn test_cache_path_generation() {
    let path = PathBuf::from("/a/b/c.png");

    let cache = ThumbnailManager::cache_path_for(&path);

    assert!(cache.to_string_lossy().contains("thumbs"));
    assert!(cache.extension().unwrap() == "bin");
}


#[test]
fn test_file_type_detection() {
    assert!(ThumbnailManager::is_image(&PathBuf::from("a.png")));
    assert!(ThumbnailManager::is_svg(&PathBuf::from("a.svg")));
    assert!(ThumbnailManager::is_video(&PathBuf::from("a.mp4")));
}

#[test]
fn test_cache_invalid_when_missing() {
    let cache = PathBuf::from("/fake/path.bin");

    let valid = ThumbnailManager::is_cache_valid(&cache, 123);

    assert!(!valid);
}