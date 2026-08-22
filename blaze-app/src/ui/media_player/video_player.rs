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

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, TryRecvError};
use egui::{ColorImage, TextureHandle, TextureOptions, Ui};
use ffmpeg_next::{
    Dictionary, Packet,
    format::{Pixel, input_with_interrupt_and_dictionary},
    media::Type,
    packet::Mut,
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
        debug!("Se ha dropeado el receiver {} ", self.name);
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

struct VideoBuffer {
    data: Vec<u8>,
    tx: Sender<Vec<u8>>,
}

impl Drop for VideoBuffer {
    fn drop(&mut self) {
        let data = std::mem::take(&mut self.data);
        let _ = self.tx.send(data);
    }
}

pub struct OutputVideoFrame {
    data: VideoBuffer,
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

            drop(ictx);
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

        if let Some(rx) = self.frame_rx.take() {
            info!("Llamado dropeo de video rx");
            drop(rx);
        }

        self.seek(0.0);
    }

    pub fn stop(&mut self) {
        info!("Llamado stop de VideoPlayer");
        self.generation.fetch_add(1, Ordering::SeqCst);

        self.pending_frame = None;

        if let Some(rx) = self.frame_rx.take() {
            info!("Llamado dropeo de video rx");
            drop(rx);
        }

        if let Some(mut streamer) = self.streamer.take() {
            streamer.send(VideoDecoderCommand::Stop);
            if let Some(handle) = streamer.thread.take() {
                info!("stop() esperando join...");
                let _ = handle.join();
                info!("stop() join completado");
            }
        }

        self.video_width = 0;
        self.video_height = 0;
        self.texture = None;

        info!("stop() fin");

        unsafe {
            libmimalloc_sys::mi_collect(true);
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

            let mut latest_frame = None;

            while let Ok(frame) = rx.try_recv() {
                if frame.epoch != current_epoch {
                    drop(frame);
                    continue;
                }

                self.video_width = frame.width;
                self.video_height = frame.height;

                if frame.timestamp < elapsed - 0.5 {
                    continue;
                }

                if frame.timestamp > elapsed {
                    latest_frame = Some(frame);
                    break;
                } else {
                    self.pending_frame = Some(frame);
                    break;
                }
            }

            let lf_is_some = latest_frame.is_some();

            if lf_is_some {
                self.pending_frame = latest_frame;
            }

            if lf_is_some || self.pending_frame.is_some() {
                ui.request_repaint_after(Duration::from_millis(1));
            }
        }

        if let Some(frame) = self.pending_frame.take() {
            if frame.timestamp <= elapsed {
                let color_image = ColorImage::from_rgba_premultiplied(
                    [frame.width as usize, frame.height as usize],
                    &frame.data.data,
                );

                drop(frame);

                if let Some(texture) = &mut self.texture {
                    texture.set_partial([0, 0], color_image, TextureOptions::LINEAR);
                } else {
                    self.texture =
                        Some(ui.load_texture("frame", color_image, TextureOptions::LINEAR));
                }

                self.pending_frame = None;
            } else if self.is_playing() {
                let wait = Duration::from_secs_f32((frame.timestamp - elapsed).max(0.0));
                self.pending_frame = Some(frame);
                ui.request_repaint_after(wait);
            }
        }

        if let Some(texture) = &self.texture {
            callback(texture, ui, (self.video_width, self.video_height));
        }
    }
}

pub struct VideoStreamer {
    thread: Option<JoinHandle<()>>,
    command_tx: Sender<VideoDecoderCommand>,
}

impl VideoStreamer {
    const MAX_PACKETS_BUFF: i32 = 4;

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
            debug!("Video generation no es igual: saliendo en inicio del decode loop");
            return Ok("Video generation no es igual, saliendo".into());
        } else {
            debug!("Video generation OK, iniciando decodificación");
        }

        let gene = generation.clone();
        let my_gene = my_generation;

        let mut ops = Dictionary::new();

        ops.set("probesize", "5000000");
        ops.set("analyzeduration", "2000000");

        let mut ictx = input_with_interrupt_and_dictionary(
            path,
            move || gene.load(Ordering::Acquire) != my_gene,
            ops,
        )
        .map_err(VideoError::FfmpegError)?;

        unsafe {
            let ctx = ictx.as_mut_ptr();
            (*ctx).max_interleave_delta = 100000;
            (*ctx).flags |= 64;
        }

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

        let (buff_tx, buff_rx) = crossbeam_channel::bounded(2);
        for _ in 0..2 {
            buff_tx
                .send(Vec::with_capacity(width as usize * height as usize * 4))
                .ok();
        }

        let mut packects_in_buff = 0;

        loop {
            if generation.load(Ordering::Acquire) != my_generation {
                debug!("Video generation no es igual: saliendo en inicio del loop");
                return Ok("Video generation no es igual, saliendo".into());
            }

            while packects_in_buff >= Self::MAX_PACKETS_BUFF || tx.len() >= 4 {
                if generation.load(Ordering::Acquire) != my_generation {
                    debug!("Video generation no es igual: saliendo en inicio del loop");
                    return Ok("Video generation no es igual, saliendo".into());
                }

                match commands.try_recv() {
                    Ok(VideoDecoderCommand::Stop) => return Ok("Stop".into()),

                    Ok(VideoDecoderCommand::Pause) => paused = true,

                    Ok(VideoDecoderCommand::Resume) => paused = false,

                    Ok(VideoDecoderCommand::SeekTo(secs)) => {
                        let timestamp = (secs * ffmpeg_next::ffi::AV_TIME_BASE as f32) as i64;
                        ictx.seek(timestamp, ..timestamp)
                            .map_err(VideoError::FfmpegError)?;
                        decoder.flush();

                        seek_target_secs = secs;
                        did_seek = true;
                    }

                    Err(TryRecvError::Empty) => {}

                    Err(TryRecvError::Disconnected) => {
                        return Ok("Command channel cerrado".into());
                    }
                }

                if paused {
                    break;
                }

                std::thread::sleep(Duration::from_millis(5));
            }

            if paused {
                continue;
            }

            let mut packet = Packet::empty();

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
                        match commands.recv_timeout(Duration::from_millis(50)) {
                            Ok(VideoDecoderCommand::Resume) => paused = false,
                            Ok(VideoDecoderCommand::SeekTo(secs)) => {
                                unsafe {
                                    ffmpeg_next::ffi::av_packet_unref(packet.as_mut_ptr());
                                }
                                let timestamp =
                                    (secs * ffmpeg_next::ffi::AV_TIME_BASE as f32) as i64;
                                ictx.seek(timestamp, ..timestamp)
                                    .map_err(VideoError::FfmpegError)?;
                                decoder.flush();
                                seek_target_secs = secs;
                                did_seek = true;
                            }
                            Ok(VideoDecoderCommand::Stop) => return Ok("Stop".into()),
                            Err(RecvTimeoutError::Timeout) => {
                                if generation.load(Ordering::Acquire) != my_generation {
                                    debug!(
                                        "Video generation no es igual: saliendo en if paused del loop"
                                    );
                                    return Ok("Video generation no es igual, saliendo".into());
                                }
                            }
                            Err(RecvTimeoutError::Disconnected) => {
                                return Ok("Command channel cerrado".into());
                            }
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

                    packects_in_buff += 1;

                    let mut decoded = VideoFrame::empty();

                    while decoder.receive_frame(&mut decoded).is_ok() {
                        packects_in_buff = (packects_in_buff - 1).max(0);

                        if generation.load(Ordering::Acquire) != my_generation {
                            debug!(
                                "Video generation no es igual: saliendo en decoder.receive_frame del loop"
                            );

                            unsafe {
                                ffmpeg_next::ffi::av_frame_unref(decoded.as_mut_ptr());
                            }

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

                        let mut write_buff = loop {
                            match buff_rx.recv_timeout(Duration::from_millis(10)) {
                                Ok(buff) => break buff,
                                Err(RecvTimeoutError::Timeout) => match commands.try_recv() {
                                    Ok(VideoDecoderCommand::Stop) => return Ok("Stop".into()),
                                    Err(TryRecvError::Disconnected) => {
                                        return Ok("Command channel cerrado".into());
                                    }
                                    _ => {}
                                },

                                Err(RecvTimeoutError::Disconnected) => {
                                    return Ok("Buffer pool cerrado".into());
                                }
                            }
                        };

                        write_buff.clear();
                        for row in 0..height as usize {
                            let start = row * stride;
                            write_buff.extend_from_slice(&raw[start..start + row_bytes]);
                        }

                        let frame = OutputVideoFrame {
                            data: VideoBuffer {
                                data: write_buff,
                                tx: buff_tx.clone(),
                            },
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

                        decoded = VideoFrame::empty();
                        rgba_frame = VideoFrame::empty();
                    }
                }
                Err(ffmpeg_next::Error::Eof) => loop {
                    match commands.recv_timeout(Duration::from_millis(50)) {
                        Err(RecvTimeoutError::Timeout) => {
                            if generation.load(Ordering::Acquire) != my_generation {
                                return Ok("Command timeout".into());
                            }
                        }
                        Ok(VideoDecoderCommand::SeekTo(secs)) => {
                            let timestamp = (secs * ffmpeg_next::ffi::AV_TIME_BASE as f32) as i64;
                            ictx.seek(timestamp, ..timestamp)
                                .map_err(VideoError::FfmpegError)?;
                            decoder.flush();

                            seek_target_secs = secs;
                            did_seek = true;

                            break;
                        }
                        Ok(VideoDecoderCommand::Pause) => paused = true,
                        Ok(VideoDecoderCommand::Resume) => paused = false,
                        Ok(VideoDecoderCommand::Stop) | Err(_) => return Ok("Stop".into()),
                    }
                },

                Err(ffmpeg_next::Error::Exit) => {
                    return Ok("Interrumpido por señal".into());
                }
                Err(e) => return Err(VideoError::FfmpegError(e)),
            }
        }
    }
}

impl Drop for VideoStreamer {
    fn drop(&mut self) {
        self.send(VideoDecoderCommand::Stop);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}
