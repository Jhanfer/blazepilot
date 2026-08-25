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
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, TryRecvError};
use ffmpeg_next::{
    ChannelLayout, Dictionary, Packet,
    format::{Sample, input_with_interrupt_and_dictionary},
    media::Type,
    util::frame::audio::Audio as AudioFrame,
};
use parking_lot::Mutex;
use std::{
    collections::VecDeque,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
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
    pub buffer: Arc<Mutex<VecDeque<f32>>>,
    pub volume_bits: Arc<AtomicU32>,
    pub volume: f32,
    pub audio_path: Option<Arc<Path>>,
    pub generation: Arc<AtomicU64>,
    pub seek_epoch: Arc<AtomicU64>,
    pub buffer_fill_handle: Option<JoinHandle<()>>,
    pub should_run: Option<Arc<AtomicBool>>,
}

impl AudioPlayer {
    pub fn init(clock: Arc<Mutex<PlaybackClock>>, seek_epoch: Arc<AtomicU64>) -> Self {
        Self {
            streamer: None,
            audio_rx: None,
            clock,
            device_stream: None,
            buffer: Arc::new(Mutex::new(VecDeque::new())),
            volume_bits: Arc::new(AtomicU32::new(f32::to_bits(1.0))),
            volume: 1.0,
            audio_path: None,
            generation: Arc::new(AtomicU64::new(0)),
            seek_epoch,
            buffer_fill_handle: None,
            should_run: None,
        }
    }

    const MAX_BUFF_SAMPLES: usize = 48_000 * 2 * 2;

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
            if self.clock.lock().is_paused() {
                self.stop()?;
            } else {
                self.resume()?;
                return Ok(());
            }
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

    pub fn reset(&mut self) -> AudioResult<()> {
        self.streamer = None;
        self.audio_path = None;
        self.seek(0.0);
        self.buffer.lock().clear();

        if let Some(rx) = self.audio_rx.take() {
            info!("Llamado dropeo de audio rx");
            drop(rx);
        }

        Ok(())
    }

    pub fn stop(&mut self) -> AudioResult<()> {
        info!("Llamado stop de AudioPlayer");
        self.generation.fetch_add(1, Ordering::SeqCst);

        if let Some(run) = self.should_run.as_ref() {
            run.store(false, Ordering::Release);
        }

        if let Some(rx) = self.audio_rx.take() {
            tracing::info!("Llamado dropeo de audio rx");
            drop(rx);
        }

        if let Some(mut streamer) = self.streamer.take() {
            streamer.send(AudioDecoderCommand::Stop);
            if let Some(handle) = streamer.thread.take() {
                info!("stop() esperando join...");
                let _ = handle.join();
                info!("stop() join completado");
            }
        }

        if let Some(handle) = self.buffer_fill_handle.take() {
            info!("Esperando buffer_fill...");
            let _ = handle.join();
            info!("buffer_fill terminado");
        }

        if let Some(stream) = self.device_stream.take() {
            info!("Dropping device stream");
            drop(stream);
        }

        let mut buf = self.buffer.lock();
        buf.clear();
        buf.shrink_to_fit();
        self.streamer = None;
        self.audio_rx = None;
        self.device_stream = None;

        unsafe {
            libmimalloc_sys::mi_collect(true);
        }

        info!("stop() fin");

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

                    let (s1, s2) = buf.as_slices();
                    let from_s1 = len.min(s1.len());
                    data[..from_s1].copy_from_slice(&s1[..from_s1]);

                    if from_s1 < len {
                        let from_s2 = len - from_s1;
                        data[from_s1..len].copy_from_slice(&s2[..from_s2]);
                    }
                    buf.drain(..len);
                    data[len..].fill(0.0);

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
        let run = Arc::new(AtomicBool::new(true));
        let should_run = run.clone();

        self.should_run = Some(run);

        let buffer = self.buffer.clone();
        let audio_rx = self.audio_rx.take();
        let seek_epoch = self.seek_epoch.clone();

        self.buffer_fill_handle = Some(std::thread::spawn(move || {
            while should_run.load(Ordering::Relaxed) {
                if let Some(rx) = &audio_rx {
                    let current_epoch = seek_epoch.load(Ordering::Acquire);

                    let buff_len = buffer.lock().len();
                    if buff_len >= Self::MAX_BUFF_SAMPLES {
                        std::thread::sleep(Duration::from_millis(5));
                        continue;
                    }

                    let mut received_any = false;

                    while let Ok(frame) = rx.try_recv() {
                        if frame.epoch == current_epoch {
                            let mut buff = buffer.lock();
                            buff.extend(frame.data.iter());
                        }

                        received_any = true;

                        if buffer.lock().len() >= Self::MAX_BUFF_SAMPLES {
                            break;
                        }
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
    const MAX_PACKETS_BUFF: i32 = 4;

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
                error!(
                    "Error decodificando audio: {:?} | raw: {}",
                    e,
                    match &e {
                        AudioError::FfmpegError(fe) => fe.to_string(),
                        _ => "no ffmpeg".to_string(),
                    }
                );
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
    ) -> AudioResult<String> {
        if generation.load(Ordering::Acquire) != my_generation {
            return Ok("Audio generation no es igual, saliendo".into());
        } else {
            debug!("Audio generation OK, iniciando decodificación");
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
        .map_err(AudioError::FfmpegError)?;

        unsafe {
            let ctx = ictx.as_mut_ptr();
            (*ctx).max_interleave_delta = 100000;
            (*ctx).flags |= 64;
        }

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

        let source_layout =
            if decoder.channel_layout().bits() == 0 || decoder.channel_layout().is_empty() {
                match decoder.channels() {
                    1 => ChannelLayout::MONO,
                    2 => ChannelLayout::STEREO,
                    6 => ChannelLayout::_5POINT1,
                    _ => ChannelLayout::STEREO,
                }
            } else {
                decoder.channel_layout()
            };

        let mut resampler = ffmpeg_next::software::resampling::Context::get(
            decoder.format(),
            source_layout,
            decoder.rate(),
            target_format,
            target_channel_layout,
            sample_rate,
        )
        .map_err(AudioError::FfmpegError)?;

        let mut paused = false;
        let mut did_seek = false;
        let mut seek_target_secs: f32 = 0.0;

        let mut packects_in_buff = 0;

        loop {
            if generation.load(Ordering::Acquire) != my_generation {
                return Ok("Audio generation no es igual, saliendo".into());
            }

            while packects_in_buff >= Self::MAX_PACKETS_BUFF || tx.len() >= 4 {
                if generation.load(Ordering::Acquire) != my_generation {
                    return Ok("Audio generation no es igual, saliendo".into());
                }

                match commands.try_recv() {
                    Ok(AudioDecoderCommand::Stop) => return Ok("Stop".into()),

                    Ok(AudioDecoderCommand::Pause) => paused = true,

                    Ok(AudioDecoderCommand::Resume) => paused = false,

                    Ok(AudioDecoderCommand::SeekTo(secs)) => {
                        let timestamp = (secs * ffmpeg_next::ffi::AV_TIME_BASE as f32) as i64;
                        ictx.seek(timestamp, ..timestamp)
                            .map_err(AudioError::FfmpegError)?;
                        decoder.flush();

                        resampler = ffmpeg_next::software::resampling::Context::get(
                            decoder.format(),
                            source_layout,
                            decoder.rate(),
                            target_format,
                            target_channel_layout,
                            sample_rate,
                        )
                        .map_err(AudioError::FfmpegError)?;
                        seek_target_secs = secs;
                        did_seek = true;
                        seek_epoch.fetch_add(1, Ordering::Release);
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

            debug!("Leyendo paquete... packects_in_buff={}", packects_in_buff);

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
                                    source_layout,
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
                            AudioDecoderCommand::Stop => return Ok("Stop".into()),
                        }
                    }

                    if paused {
                        match commands.recv_timeout(Duration::from_millis(50)) {
                            Ok(AudioDecoderCommand::Resume) => paused = false,
                            Ok(AudioDecoderCommand::Stop) | Err(_) => return Ok("Stop".into()),
                            _ => continue,
                        }
                    }

                    decoder
                        .send_packet(&packet)
                        .map_err(AudioError::FfmpegError)?;

                    packects_in_buff += 1;

                    let mut decoded = AudioFrame::empty();

                    while decoder.receive_frame(&mut decoded).is_ok() {
                        packects_in_buff = (packects_in_buff - 1).max(0);

                        if generation.load(Ordering::Acquire) != my_generation {
                            return Ok("Audio generation no es igual, saliendo".into());
                        }

                        if decoded.channel_layout().bits() == 0 {
                            unsafe {
                                use ffmpeg_next::ffi::{
                                    AVChannelLayout, AVChannelLayout__bindgen_ty_1, AVChannelOrder,
                                };
                                let frame = &mut *decoded.as_mut_ptr();
                                frame.ch_layout = AVChannelLayout {
                                    order: AVChannelOrder::AV_CHANNEL_ORDER_NATIVE,
                                    nb_channels: decoded.channels() as i32,
                                    u: AVChannelLayout__bindgen_ty_1 {
                                        mask: ChannelLayout::STEREO.bits(),
                                    },
                                    opaque: std::ptr::null_mut(),
                                };
                            }
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

                        let frame_data: Arc<[f32]> = Arc::from(data_f32);

                        let frame = OutputAudioFrame {
                            data: frame_data,
                            channels,
                            timestamp,
                            epoch: seek_epoch.load(Ordering::Acquire),
                        };

                        debug!(
                            "Frame enviado: timestamp={:.2}s epoch={}",
                            timestamp, frame.epoch
                        );

                        if tx.send(frame).is_err() {
                            return Ok("Error al enviar el frame de audio".into());
                        }

                        decoded = AudioFrame::empty();
                        resampled = AudioFrame::empty();
                    }
                }

                Err(ffmpeg_next::Error::Eof) => loop {
                    debug!("EOF alcanzado, esperando comandos...");
                    match commands.recv_timeout(Duration::from_millis(50)) {
                        Err(RecvTimeoutError::Timeout) => {
                            if generation.load(Ordering::Acquire) != my_generation {
                                return Ok("Audio generation no es igual, saliendo".into());
                            }
                        }

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
                        Ok(AudioDecoderCommand::Stop) | Err(_) => return Ok("Stop".into()),
                    }
                },

                Err(ffmpeg_next::Error::Exit) => {
                    return Ok("Interrumpido por señal".into());
                }

                Err(e) => return Err(AudioError::FfmpegError(e)),
            }
        }
    }
}

impl Drop for AudioStreamer {
    fn drop(&mut self) {
        self.send(AudioDecoderCommand::Stop);
        info!("Se inicia drop");
        if let Some(handle) = self.thread.take() {
            info!("Antes del join");
            let _ = handle.join();
            info!("Join terminado");
        }
    }
}
