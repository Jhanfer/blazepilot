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

use crossbeam_channel::{Receiver, Sender};
use egui::{ColorImage, TextureHandle, TextureOptions, Ui};
use ffmpeg_next::{
    Packet,
    format::{Pixel, input},
    media::Type,
    software::scaling::{context::Context as ScalingContext, flag::Flags},
    util::frame::video::Video as VideoFrame,
};
use parking_lot::Mutex;
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};
use tracing::{debug, error, info, warn};

use crate::ui::media_player::{
    clock::PlaybackClock,
    error::{VideoError, VideoResult},
};

struct DebugReceiver<T> {
    inner: Receiver<T>,
    name: String,
}

impl<T> DebugReceiver<T> {
    fn new(inner: Receiver<T>, name: &str) -> Self {
        debug!("Receiver {} creado", name);
        Self {
            inner,
            name: name.to_string(),
        }
    }
}

impl<T> Drop for DebugReceiver<T> {
    fn drop(&mut self) {
        debug!("⚠️ Receiver {} DROPEADO ⚠️", self.name);
    }
}

impl<T> std::ops::Deref for DebugReceiver<T> {
    type Target = Receiver<T>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug)]
pub enum VideoDecoderCommand {
    Pause,
    Resume,
    SeekTo(f32),
    Stop,
}

pub struct OutputVideoFrame {
    pub data: Arc<[u8]>,
    pub width: u32,
    pub height: u32,
    pub timestamp: f32,
    pub epoch: u64,
}

#[derive(Default)]
pub struct VideoPlayer {
    pub streamer: Option<VideoStreamer>,
    frame_rx: Option<DebugReceiver<OutputVideoFrame>>,
    clock: Arc<Mutex<PlaybackClock>>,
    texture: Option<TextureHandle>,
    pub(crate) pending_frame: Option<OutputVideoFrame>,
    video_path: Option<Arc<Path>>,
    generation: Arc<AtomicU64>,
    seek_epoch: Arc<AtomicU64>,
    video_width: u32,
    video_height: u32,
    last_texture_name: Option<Box<str>>,
}

impl VideoPlayer {
    pub fn init(clock: Arc<Mutex<PlaybackClock>>, seek_epoch: Arc<AtomicU64>) -> Self {
        Self {
            streamer: None,
            frame_rx: None,
            clock,
            texture: None,
            pending_frame: None,
            video_path: None,
            generation: Arc::new(AtomicU64::new(0)),
            seek_epoch,
            video_width: 0,
            video_height: 0,
            last_texture_name: None,
        }
    }

    pub fn load_path(&mut self, video_path: Arc<Path>) {
        if let Ok(ictx) = ffmpeg_next::format::input(&video_path) {
            let duration = if ictx.duration() > 0 {
                ictx.duration() as f32 / ffmpeg_next::ffi::AV_TIME_BASE as f32
            } else {
                f32::INFINITY
            };
            self.clock.lock().set_duration(duration);
        }

        if !video_path.exists() {
            self.video_path = None;
        }

        self.video_path = Some(video_path);
    }

    fn is_playing(&self) -> bool {
        self.clock.lock().is_playing()
    }

    pub fn play(&mut self) -> VideoResult<()> {
        debug!("video_path: {:?}", self.video_path);
        if self.streamer.is_some() {
            self.resume();
            return Ok(());
        }

        if let Some(old_streamer) = self.streamer.take() {
            old_streamer.send(VideoDecoderCommand::Stop);
            std::thread::sleep(Duration::from_millis(50));
        }

        if let Some(rx) = self.frame_rx.take() {
            drop(rx);
        }

        if self.is_playing() {
            self.stop();
            std::thread::sleep(Duration::from_millis(20));
        }

        let generation = self.generation.clone();
        let my_generation = generation.fetch_add(1, Ordering::AcqRel) + 1;

        if let Some(rx) = &self.frame_rx {
            while rx.try_recv().is_ok() {
                debug!("Se vacia canal video");
            }
        }

        debug!("video_path: {:?}", self.video_path);
        if let Some(path) = self.video_path.clone() {
            let (frame_tx, frame_rx) = crossbeam_channel::bounded::<OutputVideoFrame>(8);

            debug!("Se llama spawn video");
            self.streamer = Some(VideoStreamer::spawn(
                path,
                &frame_tx,
                generation,
                my_generation,
                self.seek_epoch.clone(),
            ));

            self.frame_rx = Some(DebugReceiver::new(frame_rx, "VideoPlayer"));
        }

        Ok(())
    }

    pub fn reset(&mut self) {
        self.texture = None;
        self.streamer = None;
        self.pending_frame = None;
        self.video_width = 0;
        self.video_height = 0;
        self.seek(0.0);
    }

    pub fn stop(&mut self) {
        info!("Llamado stop de VideoPlayer");
        self.generation.fetch_add(1, Ordering::SeqCst);

        if let Some(streamer) = self.streamer.take() {
            streamer.send(VideoDecoderCommand::Stop);
        }

        // self.video_path = None;
        self.streamer = None;
        self.video_width = 0;
        self.video_height = 0;
        self.texture = None;
        self.pending_frame = None;

        if let Some(rx) = self.frame_rx.take() {
            info!("Llamado dropeo de video rx");
            drop(rx);
        }
    }

    pub fn pause(&mut self) {
        self.clock.lock().pause();
        if let Some(s) = &self.streamer {
            s.send(VideoDecoderCommand::Pause);
        }
    }

    pub fn resume(&mut self) {
        if self.clock.lock().is_paused() {
            self.clock.lock().play();
        }
        if let Some(s) = &self.streamer {
            s.send(VideoDecoderCommand::Resume);
        }
    }

    pub fn seek(&mut self, secs: f32) {
        self.seek_epoch.fetch_add(1, Ordering::AcqRel);
        if let Some(s) = &self.streamer {
            s.send(VideoDecoderCommand::SeekTo(secs));
        }
        self.pending_frame = None;
    }

    pub fn update<C>(&mut self, ui: &mut Ui, callback: &mut C)
    where
        C: FnMut(&TextureHandle, &mut Ui, (u32, u32)),
    {
        let elapsed = self.clock.lock().elapsed();

        if self.pending_frame.is_none()
            && let Some(rx) = self.frame_rx.as_ref()
        {
            let current_epoch = self.seek_epoch.load(Ordering::Acquire);

            while let Ok(frame) = rx.try_recv() {
                if frame.epoch != current_epoch {
                    continue;
                }

                self.video_width = frame.width;
                self.video_height = frame.height;

                if frame.timestamp < elapsed - 0.5 {
                    continue;
                }

                if frame.timestamp > elapsed {
                    self.pending_frame = Some(frame);
                    break;
                }

                self.pending_frame = Some(frame);
            }

            ui.request_repaint_after(Duration::from_millis(1));
        }

        let mut frame_updated = false;

        if let Some(frame) = self.pending_frame.take() {
            if frame.timestamp <= elapsed {
                let color_image = ColorImage::from_rgba_premultiplied(
                    [frame.width as usize, frame.height as usize],
                    &frame.data,
                );

                let texture_name = format!("frame_{}", frame.timestamp);
                self.texture = None;
                if let Some(old_name) = &self.last_texture_name {
                    ui.forget_image(old_name);
                    self.last_texture_name = Some(texture_name.into());
                }

                self.texture = Some(ui.load_texture("frame", color_image, TextureOptions::LINEAR));

                self.pending_frame = None;
                frame_updated = true;
            } else {
                let wait = Duration::from_secs_f32((frame.timestamp - elapsed).max(0.0));
                self.pending_frame = Some(frame);
                ui.request_repaint_after(wait);
            }
        }

        if let Some(texture) = &self.texture {
            callback(texture, ui, (self.video_width, self.video_height));
        }

        if frame_updated {
            ui.request_repaint_after(Duration::from_millis(1));
        }
    }
}

pub struct VideoStreamer {
    thread: Option<JoinHandle<()>>,
    command_tx: Sender<VideoDecoderCommand>,
}

impl VideoStreamer {
    pub fn spawn(
        path: Arc<Path>,
        frame_tx: &Sender<OutputVideoFrame>,
        generation: Arc<AtomicU64>,
        my_generation: u64,
        seek_epoch: Arc<AtomicU64>,
    ) -> Self {
        let (command_tx, command_rx) = crossbeam_channel::unbounded();

        let frame_tx = frame_tx.clone();

        let thread = std::thread::spawn(move || {
            if generation.load(Ordering::Acquire) != my_generation {
                debug!("Video generation no es igual");
                return;
            }

            debug!("Canal {:?}", frame_tx.capacity());

            match Self::decode_loop(
                &path,
                &frame_tx,
                &command_rx,
                generation,
                my_generation,
                seek_epoch,
            ) {
                Ok(s) => {
                    warn!("Se ha salido de decode_loop. {s}");
                }
                Err(e) => {
                    warn!("Ha ocurrido un error decodificando el video: {e}.");
                }
            }
        });

        Self {
            thread: Some(thread),
            command_tx,
        }
    }

    pub fn send(&self, cmd: VideoDecoderCommand) {
        let _ = self.command_tx.send(cmd);
    }

    fn decode_loop(
        path: &Path,
        tx: &Sender<OutputVideoFrame>,
        commands: &Receiver<VideoDecoderCommand>,
        generation: Arc<AtomicU64>,
        my_generation: u64,
        seek_epoch: Arc<AtomicU64>,
    ) -> VideoResult<String> {
        debug!("Se llama decode loop video");

        if generation.load(Ordering::Acquire) != my_generation {
            debug!("Video generation no es igual, saliendo");
            return Ok("Video generation no es igual, saliendo".into());
        } else {
            debug!("Video generation OK, iniciando decodificación");
        }

        let mut ictx = input(path).map_err(VideoError::FfmpegError)?;

        let video_stream = ictx
            .streams()
            .best(Type::Video)
            .ok_or(ffmpeg_next::Error::StreamNotFound)?;

        let video_stream_index = video_stream.index();
        let time_base = video_stream.time_base();

        let context_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(video_stream.parameters())
                .map_err(VideoError::FfmpegError)?;

        let mut decoder = context_decoder
            .decoder()
            .video()
            .map_err(VideoError::FfmpegError)?;

        let mut scaler = ScalingContext::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGBA,
            decoder.width(),
            decoder.height(),
            Flags::FAST_BILINEAR,
        )
        .map_err(VideoError::FfmpegError)?;

        let width = decoder.width();
        let height = decoder.height();

        let mut paused = false;
        let mut did_seek = false;
        let mut seek_target_secs: f32 = 0.0;
        let mut frame_buffer: Vec<u8> = Vec::new();

        let mut packet = Packet::empty();

        loop {
            if generation.load(Ordering::Acquire) != my_generation {
                return Ok("Video generation no es igual, saliendo".into());
            }

            match packet.read(&mut ictx) {
                Ok(()) => {
                    let stream = packet.stream();

                    while let Ok(cmd) = commands.try_recv() {
                        match cmd {
                            VideoDecoderCommand::Pause => paused = true,
                            VideoDecoderCommand::Resume => paused = false,
                            VideoDecoderCommand::SeekTo(secs) => {
                                let timestamp =
                                    (secs * ffmpeg_next::ffi::AV_TIME_BASE as f32) as i64;
                                ictx.seek(timestamp, ..timestamp)
                                    .map_err(VideoError::FfmpegError)?;
                                decoder.flush();

                                seek_target_secs = secs;
                                did_seek = true;
                                break;
                            }
                            VideoDecoderCommand::Stop => return Ok("Stop".into()),
                        }
                    }

                    if paused {
                        match commands.recv() {
                            Ok(VideoDecoderCommand::Resume) => paused = false,
                            Ok(VideoDecoderCommand::SeekTo(secs)) => {
                                let timestamp =
                                    (secs * ffmpeg_next::ffi::AV_TIME_BASE as f32) as i64;
                                ictx.seek(timestamp, ..timestamp)
                                    .map_err(VideoError::FfmpegError)?;
                                decoder.flush();
                                seek_target_secs = secs;
                                did_seek = true;
                            }
                            Ok(VideoDecoderCommand::Stop) | Err(_) => return Ok("Stop".into()),
                            _ => {}
                        }
                        continue;
                    }

                    if stream != video_stream_index {
                        continue;
                    }

                    decoder
                        .send_packet(&packet)
                        .map_err(VideoError::FfmpegError)?;

                    let mut decoded = VideoFrame::empty();

                    while decoder.receive_frame(&mut decoded).is_ok() {
                        if generation.load(Ordering::Acquire) != my_generation {
                            return Ok("Video generation no es igual, saliendo".into());
                        }

                        let mut rgba_frame = VideoFrame::empty();
                        scaler.run(&decoded, &mut rgba_frame)?;

                        let pts = decoded.pts().unwrap_or(0);
                        let timestamp = pts as f32 * time_base.numerator() as f32
                            / time_base.denominator() as f32;

                        if did_seek {
                            packet = Packet::empty();

                            if timestamp < seek_target_secs {
                                continue;
                            }
                            did_seek = false;
                        }

                        let stride = rgba_frame.stride(0);
                        let raw = rgba_frame.data(0);
                        let row_bytes = width as usize * 4;

                        frame_buffer.clear();
                        frame_buffer.reserve(row_bytes * height as usize);
                        for row in 0..height as usize {
                            let start = row * stride;
                            frame_buffer.extend_from_slice(&raw[start..start + row_bytes]);
                        }

                        let frame = OutputVideoFrame {
                            data: std::mem::take(&mut frame_buffer).into(),
                            width,
                            height,
                            timestamp,
                            epoch: seek_epoch.load(Ordering::Acquire),
                        };

                        match tx.send(frame) {
                            Ok(()) => {}
                            Err(e) => {
                                error!("Error al enviar frame: {e}");
                                return Ok("Error en el send".into());
                            }
                        }
                    }
                }
                Err(ffmpeg_next::Error::Eof) => loop {
                    match commands.recv() {
                        Ok(VideoDecoderCommand::SeekTo(secs)) => {
                            let timestamp = (secs * ffmpeg_next::ffi::AV_TIME_BASE as f32) as i64;
                            ictx.seek(timestamp, ..timestamp)
                                .map_err(VideoError::FfmpegError)?;
                            decoder.flush();

                            seek_target_secs = secs;
                            did_seek = true;
                            seek_epoch.fetch_add(1, Ordering::Release);
                            break;
                        }
                        Ok(VideoDecoderCommand::Pause) => paused = true,
                        Ok(VideoDecoderCommand::Resume) => paused = false,
                        Ok(VideoDecoderCommand::Stop) | Err(_) => return Ok("Stop".into()),
                    }
                },
                Err(e) => return Err(VideoError::FfmpegError(e)),
            }
        }
    }
}

impl Drop for VideoStreamer {
    fn drop(&mut self) {
        self.send(VideoDecoderCommand::Stop);

        if let Some(handle) = self.thread.take() {
            drop(handle);
        }
    }
}
