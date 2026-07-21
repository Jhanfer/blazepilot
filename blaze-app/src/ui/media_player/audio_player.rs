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

use cpal::{
    Device, Stream, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use crossbeam_channel::{Receiver, Sender};
use ffmpeg_next::{
    ChannelLayout, Packet,
    format::{Sample, input},
    media::Type,
    util::frame::audio::Audio as AudioFrame,
};
use parking_lot::Mutex;
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
use tracing::{debug, error, info, warn};

use crate::ui::media_player::{
    clock::PlaybackClock,
    error::{AudioError, AudioResult},
};

pub enum AudioDecoderCommand {
    Pause,
    Resume,
    SeekTo(f32),
    Stop,
}

pub struct OutputAudioFrame {
    pub data: Arc<[f32]>,
    #[allow(unused)]
    channels: u16,
    #[allow(unused)]
    timestamp: f32,
    pub epoch: u64,
}

pub struct AudioPlayer {
    pub streamer: Option<AudioStreamer>,
    pub audio_rx: Option<Receiver<OutputAudioFrame>>,
    pub clock: Arc<Mutex<PlaybackClock>>,
    pub device_stream: Option<Stream>,
    pub buffer: Arc<Mutex<Vec<f32>>>,
    pub volume_bits: Arc<AtomicU32>,
    pub volume: f32,
    pub audio_path: Option<Arc<Path>>,
    pub generation: Arc<AtomicU64>,
    pub seek_epoch: Arc<AtomicU64>,
    pub buffer_fill_handle: Option<JoinHandle<()>>,
    pub should_run: Arc<AtomicBool>,
}

impl AudioPlayer {
    pub fn init(clock: Arc<Mutex<PlaybackClock>>, seek_epoch: Arc<AtomicU64>) -> Self {
        Self {
            streamer: None,
            audio_rx: None,
            clock,
            device_stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            volume_bits: Arc::new(AtomicU32::new(f32::to_bits(1.0))),
            volume: 1.0,
            audio_path: None,
            generation: Arc::new(AtomicU64::new(0)),
            seek_epoch,
            buffer_fill_handle: None,
            should_run: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn load_path(&mut self, path: Arc<Path>) {
        if let Ok(ictx) = ffmpeg_next::format::input(&path) {
            let duration = if ictx.duration() > 0 {
                ictx.duration() as f32 / ffmpeg_next::ffi::AV_TIME_BASE as f32
            } else {
                f32::INFINITY
            };
            self.clock.lock().set_duration(duration);
        }
        self.audio_path = Some(path);
    }

    pub fn play(&mut self) -> AudioResult<()> {
        debug!("audio_path: {:?}", self.audio_path);

        let has_active_streamer = self
            .streamer
            .as_ref()
            .map(|s| s.thread.as_ref().map(|t| !t.is_finished()).unwrap_or(false))
            .unwrap_or(false);

        if has_active_streamer {
            self.resume()?;
            return Ok(());
        }

        if self.streamer.is_some() {
            let _ = self.streamer.take();
        }
        if self.audio_rx.is_some() {
            let _ = self.audio_rx.take();
        }

        let generation = self.generation.clone();
        let my_generation = generation.fetch_add(1, Ordering::AcqRel) + 1;

        if let Some(rx) = &self.audio_rx {
            while rx.try_recv().is_ok() {
                debug!("Se vacia canal video");
            }
        }

        let host = cpal::default_host();
        let device = host.default_output_device().ok_or(AudioError::NoDevice)?;
        let config = device
            .default_output_config()
            .map_err(AudioError::CpalError)?;
        let sample_rate = config.sample_rate();

        debug!("El path de audio es: {:?}", self.audio_path);

        if let Some(path) = self.audio_path.clone() {
            let (audio_tx, audio_rx) = crossbeam_channel::bounded::<OutputAudioFrame>(4);

            debug!("iniciamos spawn");
            self.streamer = Some(AudioStreamer::spawn(
                path,
                audio_tx,
                generation,
                my_generation,
                self.seek_epoch.clone(),
                sample_rate,
            ));
            self.audio_rx = Some(audio_rx);
        }

        self.start_audio_stream(device, config)?;
        self.start_buffer_filler();

        Ok(())
    }

    pub fn reset(&mut self) {
        self.streamer = None;
        self.seek(0.0);
        self.buffer.lock().clear();
    }

    pub fn stop(&mut self) -> AudioResult<()> {
        info!("Llamado stop de AudioPlayer");
        self.generation.fetch_add(1, Ordering::SeqCst);
        self.pause()?;

        if let Some(streamer) = self.streamer.take() {
            streamer.send(AudioDecoderCommand::Stop);

            if let Some(thread) = &streamer.thread {
                let start = Instant::now();
                while !thread.is_finished() && start.elapsed() < Duration::from_millis(100) {
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
        }

        if let Some(stream) = self.device_stream.take() {
            drop(stream);
        }

        self.buffer.lock().clear();

        if let Some(rx) = self.audio_rx.take() {
            tracing::info!("Llamado dropeo de audio rx");
            drop(rx);
        }

        if let Some(handle) = self.buffer_fill_handle.take() {
            drop(handle);
        }

        self.streamer = None;
        self.audio_rx = None;
        self.device_stream = None;

        Ok(())
    }

    pub fn pause(&mut self) -> AudioResult<()> {
        self.clock.lock().pause();

        if let Some(stream) = &self.device_stream {
            stream.pause().map_err(AudioError::CpalError)?;
        }

        if let Some(s) = &self.streamer {
            s.send(AudioDecoderCommand::Pause);
        }

        Ok(())
    }

    pub fn resume(&mut self) -> AudioResult<()> {
        if !self.clock.lock().is_paused() {
            return Ok(());
        }

        self.clock.lock().play();

        if let Some(stream) = &self.device_stream {
            stream.play().map_err(AudioError::CpalError)?;
        }

        if let Some(s) = &self.streamer {
            s.send(AudioDecoderCommand::Resume);
        }

        Ok(())
    }

    pub fn seek(&mut self, secs: f32) {
        self.seek_epoch.fetch_add(1, Ordering::AcqRel);
        if let Some(s) = &self.streamer {
            s.send(AudioDecoderCommand::SeekTo(secs));
        }
        self.buffer.lock().clear();
    }

    pub fn set_volume(&mut self, new_volume: f32) {
        let clamped = new_volume.clamp(0.0, 1.0);
        self.volume = clamped;
        self.volume_bits
            .store(f32::to_bits(clamped), Ordering::Relaxed);
    }

    pub fn start_audio_stream(
        &mut self,
        device: Device,
        config: SupportedStreamConfig,
    ) -> AudioResult<()> {
        debug!("Se incia audio stream");
        let buffer = self.buffer.clone();
        let volume_bits = self.volume_bits.clone();
        let stream = device
            .build_output_stream(
                config.into(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut buf = buffer.lock();
                    let len = data.len().min(buf.len());
                    data[..len].copy_from_slice(&buf[..len]);
                    buf.drain(..len);

                    let vol = f32::from_bits(volume_bits.load(Ordering::Relaxed));
                    for sample in &mut data[..len] {
                        *sample *= vol;
                    }
                },
                |err| warn!("Error en audio: {err}"),
                Some(Duration::from_millis(500)),
            )
            .map_err(AudioError::CpalError)?;

        stream.play().map_err(AudioError::CpalError)?;
        self.device_stream = Some(stream);

        Ok(())
    }

    pub fn start_buffer_filler(&mut self) {
        debug!("Se inicia el filler de audio");
        if let Some(_old_handle) = self.buffer_fill_handle.take() {}

        let buffer = self.buffer.clone();
        let should_run = self.should_run.clone();
        let audio_rx = self.audio_rx.clone();
        let seek_epoch = self.seek_epoch.clone();

        self.buffer_fill_handle = Some(std::thread::spawn(move || {
            while should_run.load(Ordering::Relaxed) {
                if let Some(rx) = &audio_rx {
                    let current_epoch = seek_epoch.load(Ordering::Acquire);
                    let mut received_any = false;

                    while let Ok(frame) = rx.try_recv() {
                        if frame.epoch == current_epoch {
                            let mut buf = buffer.lock();
                            buf.extend_from_slice(&frame.data);
                            //break;
                        }
                        received_any = true;
                    }

                    if !received_any {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                } else {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }));
    }
}

pub struct AudioStreamer {
    thread: Option<JoinHandle<()>>,
    command_tx: Sender<AudioDecoderCommand>,
}

impl AudioStreamer {
    pub fn spawn(
        path: Arc<Path>,
        audio_tx: Sender<OutputAudioFrame>,
        generation: Arc<AtomicU64>,
        my_generation: u64,
        seek_epoch: Arc<AtomicU64>,
        sample_rate: u32,
    ) -> Self {
        let (command_tx, command_rx) = crossbeam_channel::unbounded();

        let thread = std::thread::spawn(move || {
            if let Err(e) = Self::decode_loop(
                &path,
                &audio_tx,
                &command_rx,
                generation,
                my_generation,
                seek_epoch,
                sample_rate,
            ) {
                error!("Ha ocurrido un error decodificando el audio: {e}.");
            }
        });

        Self {
            thread: Some(thread),
            command_tx,
        }
    }

    pub fn send(&self, cmd: AudioDecoderCommand) {
        let _ = self.command_tx.send(cmd);
    }

    fn decode_loop(
        path: &Path,
        tx: &Sender<OutputAudioFrame>,
        commands: &Receiver<AudioDecoderCommand>,
        generation: Arc<AtomicU64>,
        my_generation: u64,
        seek_epoch: Arc<AtomicU64>,
        sample_rate: u32,
    ) -> AudioResult<()> {
        let mut ictx = input(path).map_err(AudioError::FfmpegError)?;

        let audio_stream = ictx
            .streams()
            .best(Type::Audio)
            .ok_or(ffmpeg_next::Error::StreamNotFound)?;

        let stream_index = audio_stream.index();
        let time_base = audio_stream.time_base();

        let context_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(audio_stream.parameters())?;

        let mut decoder = context_decoder
            .decoder()
            .audio()
            .map_err(AudioError::FfmpegError)?;

        let target_format = Sample::F32(ffmpeg_next::format::sample::Type::Packed);
        let target_channel_layout = ChannelLayout::STEREO;

        let mut resampler = ffmpeg_next::software::resampling::Context::get(
            decoder.format(),
            decoder.channel_layout(),
            decoder.rate(),
            target_format,
            target_channel_layout,
            sample_rate,
        )
        .map_err(AudioError::FfmpegError)?;

        let mut paused = false;
        let mut did_seek = false;
        let mut seek_target_secs: f32 = 0.0;

        let mut packet = Packet::empty();

        loop {
            if generation.load(Ordering::Acquire) != my_generation {
                return Ok(());
            }
            match packet.read(&mut ictx) {
                Ok(()) => {
                    let stream = packet.stream();

                    if stream != stream_index {
                        continue;
                    }

                    while let Ok(cmd) = commands.try_recv() {
                        match cmd {
                            AudioDecoderCommand::Pause => paused = true,
                            AudioDecoderCommand::Resume => paused = false,
                            AudioDecoderCommand::SeekTo(secs) => {
                                let timestamp =
                                    (secs * ffmpeg_next::ffi::AV_TIME_BASE as f32) as i64;
                                ictx.seek(timestamp, ..timestamp)
                                    .map_err(AudioError::FfmpegError)?;
                                decoder.flush();

                                resampler = ffmpeg_next::software::resampling::Context::get(
                                    decoder.format(),
                                    decoder.channel_layout(),
                                    decoder.rate(),
                                    target_format,
                                    target_channel_layout,
                                    sample_rate,
                                )
                                .map_err(AudioError::FfmpegError)?;

                                seek_target_secs = secs;
                                did_seek = true;
                                seek_epoch.fetch_add(1, Ordering::Release);
                                break;
                            }
                            AudioDecoderCommand::Stop => return Ok(()),
                        }
                    }

                    if paused {
                        match commands.recv() {
                            Ok(AudioDecoderCommand::Resume) => paused = false,
                            Ok(AudioDecoderCommand::Stop) | Err(_) => return Ok(()),
                            _ => continue,
                        }
                    }

                    decoder
                        .send_packet(&packet)
                        .map_err(AudioError::FfmpegError)?;

                    let mut decoded = AudioFrame::empty();

                    while decoder.receive_frame(&mut decoded).is_ok() {
                        if generation.load(Ordering::Acquire) != my_generation {
                            return Ok(());
                        }

                        let mut resampled = AudioFrame::empty();
                        resampler
                            .run(&decoded, &mut resampled)
                            .map_err(AudioError::FfmpegError)?;

                        let pts = decoded.pts().unwrap_or(0);
                        let timestamp = pts as f32 * time_base.numerator() as f32
                            / time_base.denominator() as f32;

                        if did_seek && timestamp < seek_target_secs {
                            continue;
                        }
                        did_seek = false;

                        let data_u8 = resampled.data(0);
                        let sample_count = resampled.samples();
                        let channels = resampled.channels();

                        let data_f32 = unsafe {
                            std::slice::from_raw_parts(
                                data_u8.as_ptr() as *const f32,
                                sample_count * channels as usize,
                            )
                        };

                        let mut data = Vec::with_capacity(sample_count * channels as usize);
                        for i in 0..sample_count {
                            for ch in 0..channels as usize {
                                let idx = i * channels as usize + ch;
                                data.push(data_f32[idx]);
                            }
                        }

                        let frame = OutputAudioFrame {
                            data: Arc::from(data_f32.to_vec().into_boxed_slice()),
                            channels,
                            timestamp,
                            epoch: seek_epoch.load(Ordering::Acquire),
                        };

                        if tx.send(frame).is_err() {
                            return Ok(());
                        }
                    }
                }

                Err(ffmpeg_next::Error::Eof) => loop {
                    match commands.recv() {
                        Ok(AudioDecoderCommand::SeekTo(secs)) => {
                            let timestamp = (secs * ffmpeg_next::ffi::AV_TIME_BASE as f32) as i64;
                            ictx.seek(timestamp, ..timestamp)
                                .map_err(AudioError::FfmpegError)?;
                            decoder.flush();

                            resampler = ffmpeg_next::software::resampling::Context::get(
                                decoder.format(),
                                decoder.channel_layout(),
                                decoder.rate(),
                                target_format,
                                target_channel_layout,
                                sample_rate,
                            )
                            .map_err(AudioError::FfmpegError)?;

                            seek_target_secs = secs;
                            did_seek = true;
                            seek_epoch.fetch_add(1, Ordering::Release);
                            break;
                        }
                        Ok(AudioDecoderCommand::Pause) => paused = true,
                        Ok(AudioDecoderCommand::Resume) => paused = false,
                        Ok(AudioDecoderCommand::Stop) | Err(_) => return Ok(()),
                    }
                },

                Err(e) => return Err(AudioError::FfmpegError(e)),
            }
        }
    }
}

impl Drop for AudioStreamer {
    fn drop(&mut self) {
        self.send(AudioDecoderCommand::Stop);
        if let Some(handle) = self.thread.take() {
            drop(handle);
        }
    }
}
