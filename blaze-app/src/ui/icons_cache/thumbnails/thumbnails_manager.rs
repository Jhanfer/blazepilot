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
    core::{
        runtime::{
            bus_structs::UiEvent,
            event_bus::{Dispatcher, with_event_bus},
        },
        system::{cache::cache_manager::CacheManager, clipboard::global_clipboard::TOKIO_RUNTIME},
    },
    ui::icons_cache::thumbnails::utils::resolve_tiff_data,
};

use fast_image_resize as fr;
use ffmpeg_next::{
    Packet,
    format::{Pixel, input},
    media::Type,
    software::scaling::{context::Context as ScalingContext, flag::Flags},
    util::frame::video::Video as VideoFrame,
};
use lru::LruCache;
use parking_lot::RwLock;
use std::{
    collections::HashSet,
    hash::{DefaultHasher, Hasher},
    num::NonZeroUsize,
};
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
    time::UNIX_EPOCH,
};
use thiserror::Error;
use tokio::sync::Semaphore;
use tracing::{error, warn};
use uuid::Uuid;

const DEFAULT_THUMB_CACHE_CAPACITY: usize = 300;

#[derive(Debug, Error)]
pub enum ThumbError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Error en trhead Tokio: {0}")]
    ThreadError(#[from] tokio::task::JoinError),

    #[error("Error cargando imagen")]
    ImageError,

    #[error("Formato no soportado")]
    UnsuportedFormat,

    #[error("Error procesando SVG")]
    SvgError,

    #[error("Slice error")]
    SliceError(#[from] std::array::TryFromSliceError),

    #[error("Directorio de caché de miniaturas no existe")]
    ThumbsDirDoesNotExist,

    #[error("FfmpegError: {0}")]
    FfmpegError(#[from] ffmpeg_next::Error),
}
#[derive(Debug)]
pub enum ThumbnailMessages {
    RequestThumb(Arc<Path>),
}

#[derive(Clone)]
pub struct Thumbnail {
    pub pixels: Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

pub struct ThumbnailManager {
    pub thumb_map: Arc<RwLock<LruCache<Arc<Path>, Thumbnail>>>,
    pub semaphore: Arc<Semaphore>,
}

impl ThumbnailManager {
    pub fn new() -> Self {
        let manager = Self::with_capacity(DEFAULT_THUMB_CACHE_CAPACITY);

        TOKIO_RUNTIME.spawn(async {
            if let Err(e) = ThumbnailManager::cleanup_orphans().await {
                warn!("Limpieza ha fallado: {}", e);
            }
        });

        manager
    }

    fn with_capacity(cap: usize) -> Self {
        let def_cap: NonZeroUsize = match NonZeroUsize::new(300) {
            Some(n) => n,
            None => unreachable!(),
        };

        let cap = NonZeroUsize::new(cap).unwrap_or(def_cap);

        Self {
            thumb_map: Arc::new(RwLock::new(LruCache::new(cap))),
            semaphore: Arc::new(Semaphore::new(4)),
        }
    }

    fn thumb_cache_dir() -> PathBuf {
        CacheManager::global().cache_dir.join("thumbs")
    }

    fn path_hash(path: &Path) -> String {
        let mut hasher = DefaultHasher::new();
        hasher.write(path.as_os_str().as_encoded_bytes());
        let hash_u64 = hasher.finish();
        format!("{:02x}", hash_u64)
    }

    fn cache_path_for(path: &Path) -> PathBuf {
        Self::thumb_cache_dir().join(format!("{}.bin", Self::path_hash(path)))
    }

    fn get_real_mtime(path: &Path) -> u64 {
        std::fs::metadata(path)
            .and_then(|m| m.modified())
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
            .unwrap_or(0)
    }

    fn is_image(path: &Path) -> bool {
        matches!(
            path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .as_deref(),
            Some(
                "png"
                    | "jpg"
                    | "jpeg"
                    | "webp"
                    | "gif"
                    | "pbm"
                    | "pgm"
                    | "ppm"
                    | "pnm"
                    | "bmp"
                    | "tif"
                    | "tiff"
                    | "ico"
                    | "avif"
            )
        )
    }

    fn is_svg(path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .as_deref()
            == Some("svg")
    }

    fn is_video(path: &Path) -> bool {
        matches!(
            path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .as_deref(),
            Some("mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv")
        )
    }

    fn is_cache_valid(cache_path: &Path, current_mtime: u64) -> bool {
        if !cache_path.exists() {
            return false;
        }

        let meta_path = cache_path.with_extension("meta");
        let Ok(content) = std::fs::read_to_string(&meta_path) else {
            return false;
        };

        let mut lines = content.lines();
        let cached_mtime: u64 = lines
            .next()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);

        cached_mtime == current_mtime
    }

    pub fn process_messages(&self, active_id: Uuid, sender: Dispatcher) {
        let messages: Vec<ThumbnailMessages> = with_event_bus(|pool| {
            let mut msgs = Vec::new();
            pool.drain(active_id, |msg| {
                msgs.push(msg);
                true
            });
            msgs
        });

        let mut seen: HashSet<Arc<Path>> = HashSet::new();
        let unique_paths: Vec<Arc<Path>> = messages
            .into_iter()
            .map(|m| match m {
                ThumbnailMessages::RequestThumb(path) => path,
            })
            .filter(|p| seen.insert(Arc::clone(p)))
            .collect();

        let mut need_load = Vec::new();

        {
            let cache = self.thumb_map.read();
            for path in unique_paths {
                if cache.contains(&path) {
                    sender
                        .send(UiEvent::ThumbnailReady { full_path: path })
                        .ok();
                } else {
                    need_load.push(path);
                }
            }
        }

        for path in need_load {
            if self.thumb_map.read().contains(&path) {
                sender
                    .send(UiEvent::ThumbnailReady { full_path: path })
                    .ok();
                continue;
            }

            if path.starts_with(Self::thumb_cache_dir()) {
                continue;
            }

            if !Self::is_image(&path) && !Self::is_svg(&path) && !Self::is_video(&path) {
                continue;
            }

            let thumb_map = self.thumb_map.clone();
            let sender_clone = sender.clone();
            let sem = self.semaphore.clone();

            let current_mtime = Self::get_real_mtime(&path);
            let cache_path = Self::cache_path_for(&path);

            if Self::is_cache_valid(&cache_path, current_mtime) {
                let thumb_map = thumb_map.clone();
                TOKIO_RUNTIME.spawn(async move {
                    let Ok(_permit) = sem.acquire_owned().await else {
                        return;
                    };

                    //lee la imagen en cache
                    if let Ok(thumb) = Self::load_from_cache(&cache_path).await {
                        thumb_map.write().put(path.clone(), thumb);
                        sender_clone
                            .send(UiEvent::ThumbnailReady { full_path: path })
                            .ok();
                    }
                });
            } else {
                TOKIO_RUNTIME.spawn(async move {
                    let Ok(_permit) = sem.acquire_owned().await else {
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
                        if let Err(e) =
                            Self::save_to_cache(&cache_path, &thumb, current_mtime, &path).await
                        {
                            let err = format!("Error en el caché de miniaturas: {}", e);
                            error!(err);
                            sender_clone.send(UiEvent::ShowError(err.into())).ok();
                        }

                        thumb_map.write().put(path.clone(), thumb);
                        sender_clone
                            .send(UiEvent::ThumbnailReady { full_path: path })
                            .ok();
                    }
                });
            }
        }
    }

    async fn load_from_cache(cache_path: &PathBuf) -> Result<Thumbnail, ThumbError> {
        let bytes = tokio::fs::read(cache_path).await.map_err(ThumbError::Io)?;

        if bytes.len() < 8 {
            return Err(ThumbError::ImageError);
        }

        let w = u32::from_le_bytes(bytes[0..4].try_into().map_err(ThumbError::SliceError)?);
        let h = u32::from_le_bytes(bytes[4..8].try_into().map_err(ThumbError::SliceError)?);

        if w == 0 || h == 0 || bytes.len() < 8 + (w as usize * h as usize * 4) {
            return Err(ThumbError::ImageError);
        }

        let pixels = bytes[8..].to_vec();

        Ok(Thumbnail {
            pixels: Arc::new(pixels),
            width: w,
            height: h,
        })
    }

    async fn save_to_cache(
        cache_path: &Path,
        thumb: &Thumbnail,
        mtime: u64,
        original_path: &Path,
    ) -> Result<(), ThumbError> {
        if let Some(parent) = cache_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(ThumbError::Io)?;
        }

        let pixels = thumb.pixels.clone();
        let (w, h) = (thumb.width, thumb.height);
        let cache_path_clone = cache_path.to_path_buf();
        let meta_path = cache_path.with_extension("meta");
        let meta_content = format!("{}\n{}", mtime, original_path.to_string_lossy());

        //guardar binario
        tokio::task::spawn_blocking(move || -> Result<(), ThumbError> {
            let mut buf = Vec::with_capacity(8 + pixels.len());
            buf.extend_from_slice(&w.to_le_bytes());
            buf.extend_from_slice(&h.to_le_bytes());
            buf.extend_from_slice(&pixels);
            std::fs::write(&cache_path_clone, buf).map_err(ThumbError::Io)?;

            // Guardar meta
            std::fs::write(&meta_path, meta_content).map_err(ThumbError::Io)?;

            Ok(())
        })
        .await
        .map_err(ThumbError::ThreadError)??;

        Ok(())
    }

    async fn generate_image_thumb(path: &Path) -> Result<Thumbnail, ThumbError> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || -> Result<Thumbnail, ThumbError> {
            let mut file = std::fs::File::open(path.clone()).map_err(ThumbError::Io)?;

            let mut header = [0u8; 54];

            let n = file.read(&mut header).map_err(ThumbError::Io)?;

            let img_type =
                imagesize::image_type(&header[..n]).map_err(|_| ThumbError::UnsuportedFormat)?;

            let mut buffer = Vec::new();
            let mut full_file = std::fs::File::open(path.clone()).map_err(ThumbError::Io)?;
            full_file.read_to_end(&mut buffer).map_err(ThumbError::Io)?;

            let is_avif = path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("avif"))
                .unwrap_or(false);

            let (src_pixels, src_w, src_h) = if is_avif {
                let dynamic_image =
                    libavif_image::read(&buffer).map_err(|_| ThumbError::ImageError)?;

                let w = dynamic_image.width();
                let h = dynamic_image.height();

                let rgba_buf = dynamic_image.to_rgba8().into_raw();
                (rgba_buf, w, h)
            } else {
                match img_type {
                    imagesize::ImageType::Webp => {
                        let cursor = std::io::Cursor::new(&buffer);

                        let mut decoder = image_webp::WebPDecoder::new(cursor)
                            .map_err(|_| ThumbError::ImageError)?;

                        let (w, h) = decoder.dimensions();

                        let has_alpha = decoder.has_alpha();
                        let bytes_per_pixel = if has_alpha { 4 } else { 3 };
                        let mut raw_buf = vec![0u8; (w * h * bytes_per_pixel) as usize];

                        decoder
                            .read_image(&mut raw_buf)
                            .map_err(|_| ThumbError::ImageError)?;

                        let rgba_buf = if !has_alpha {
                            let mut converted = Vec::with_capacity((w * h * 4) as usize);

                            for chunk in raw_buf.as_chunks::<3>().0 {
                                converted.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
                            }
                            converted
                        } else {
                            raw_buf
                        };

                        (rgba_buf, w, h)
                    }

                    imagesize::ImageType::Tiff => {
                        let cursor = std::io::Cursor::new(&buffer);

                        let mut decoder = tiff::decoder::Decoder::new(cursor)
                            .map_err(|_| ThumbError::ImageError)?;

                        let (w, h) = decoder.dimensions().map_err(|_| ThumbError::ImageError)?;

                        let rgba_buf = decoder.read_image().map_err(|_| ThumbError::ImageError)?;

                        let rgb_data = resolve_tiff_data(rgba_buf, w, h)?;

                        (rgb_data, w, h)
                    }

                    _ => match stb_image::image::load_from_memory_with_depth(&buffer, 4, false) {
                        stb_image::image::LoadResult::ImageU8(image) => {
                            (image.data, image.width as u32, image.height as u32)
                        }
                        _ => return Err(ThumbError::ImageError),
                    },
                }
            };

            let src_image =
                fr::images::Image::from_vec_u8(src_w, src_h, src_pixels, fr::PixelType::U8x4)
                    .map_err(|_| ThumbError::ImageError)?;

            let mut dst_image = fr::images::Image::new(64, 64, fr::PixelType::U8x4);

            let mut resizer = fr::Resizer::new();

            resizer
                .resize(&src_image, &mut dst_image, &fr::ResizeOptions::new())
                .unwrap();

            let rgba_scaled = dst_image.into_vec();

            Ok(Thumbnail {
                pixels: Arc::new(rgba_scaled),
                width: 64,
                height: 64,
            })
        })
        .await
        .map_err(ThumbError::ThreadError)?
    }

    async fn generate_svg_thumb(path: &Path) -> Result<Thumbnail, ThumbError> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            let data = std::fs::read(&path).map_err(ThumbError::Io)?;

            let opt = resvg::usvg::Options::default();

            let tree =
                resvg::usvg::Tree::from_data(&data, &opt).map_err(|_| ThumbError::ImageError)?;

            let mut pixmap = resvg::tiny_skia::Pixmap::new(64, 64).ok_or(ThumbError::SvgError)?;

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
        })
        .await
        .map_err(ThumbError::ThreadError)?
    }

    async fn generate_video_thumb(path: &Path) -> Result<Thumbnail, ThumbError> {
        let mut ictx = input(path).map_err(ThumbError::FfmpegError)?;

        let video_stream = ictx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg_next::Error::StreamNotFound)?;

        let video_stream_index = video_stream.index();

        let context_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(video_stream.parameters())
                .map_err(ThumbError::FfmpegError)?;

        let mut decoder = context_decoder
            .decoder()
            .video()
            .map_err(ThumbError::FfmpegError)?;

        let target_width = 64;
        let target_height = 64;

        let mut scaler = ScalingContext::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGBA,
            target_width,
            target_height,
            Flags::FAST_BILINEAR,
        )
        .map_err(ThumbError::FfmpegError)?;

        let mut frame_buffer: Vec<u8> = Vec::new();
        let mut packet = Packet::empty();

        loop {
            match packet.read(&mut ictx) {
                Ok(()) => {
                    if packet.stream() != video_stream_index {
                        continue;
                    }

                    decoder
                        .send_packet(&packet)
                        .map_err(ThumbError::FfmpegError)?;

                    let mut decoded = VideoFrame::empty();

                    if decoder.receive_frame(&mut decoded).is_ok() {
                        let mut rgba_frame = VideoFrame::empty();
                        scaler
                            .run(&decoded, &mut rgba_frame)
                            .map_err(ThumbError::FfmpegError)?;

                        let stride = rgba_frame.stride(0);
                        let raw = rgba_frame.data(0);
                        let row_bytes = target_width as usize * 4;

                        frame_buffer.clear();
                        frame_buffer.reserve(row_bytes * target_height as usize);
                        for row in 0..target_height as usize {
                            let start = row * stride;
                            frame_buffer.extend_from_slice(&raw[start..start + row_bytes]);
                        }

                        return Ok(Thumbnail {
                            pixels: Arc::new(frame_buffer),
                            width: target_width,
                            height: target_height,
                        });
                    }
                }
                Err(ffmpeg_next::Error::Eof) => {
                    decoder.send_eof().map_err(ThumbError::FfmpegError)?;

                    let mut decoded = VideoFrame::empty();

                    if decoder.receive_frame(&mut decoded).is_ok() {
                        let mut rgba_frame = VideoFrame::empty();
                        scaler
                            .run(&decoded, &mut rgba_frame)
                            .map_err(ThumbError::FfmpegError)?;

                        let stride = rgba_frame.stride(0);
                        let raw = rgba_frame.data(0);
                        let row_bytes = target_width as usize * 4;

                        frame_buffer.clear();
                        frame_buffer.reserve(row_bytes * target_height as usize);
                        for row in 0..target_height as usize {
                            let start = row * stride;
                            frame_buffer.extend_from_slice(&raw[start..start + row_bytes]);
                        }

                        return Ok(Thumbnail {
                            pixels: Arc::new(frame_buffer),
                            width: target_width,
                            height: target_height,
                        });
                    }
                }
                Err(e) => return Err(ThumbError::FfmpegError(e)),
            }
        }
    }

    pub async fn cleanup_orphans() -> Result<(), ThumbError> {
        let dir = Self::thumb_cache_dir();
        if !dir.exists() {
            return Err(ThumbError::ThumbsDirDoesNotExist);
        }

        let mut entries = tokio::fs::read_dir(&dir).await.map_err(ThumbError::Io)?;

        while let Ok(Some(entry)) = entries.next_entry().await {
            let meta_path = entry.path();
            if meta_path.extension().and_then(|e| e.to_str()) != Some("meta") {
                continue;
            }

            // esto lee "mtime\n/path/original"
            let Ok(content) = tokio::fs::read_to_string(&meta_path).await else {
                continue;
            };
            let mut lines = content.lines();
            let Some(_) = lines.next() else { continue };
            let Some(orig_str) = lines.next() else {
                continue;
            };

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
