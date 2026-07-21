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

use egui::{TextureHandle, Ui};
use parking_lot::Mutex;
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tracing::{debug, warn};

use crate::ui::media_player::{
    audio_player::AudioPlayer,
    clock::{ClockState, PlaybackClock},
    video_player::VideoPlayer,
};

pub struct MediaPlayer {
    pub video_player: VideoPlayer,
    pub audio_player: AudioPlayer,
    pub clock: Arc<Mutex<PlaybackClock>>,
    seek_epoch: Arc<AtomicU64>,
    audio_enabled: bool,
    video_enabled: bool,
    media_path: Option<Arc<Path>>,
}

impl MediaPlayer {
    pub fn init() -> Self {
        let clock = Arc::new(Mutex::new(PlaybackClock::new()));
        let seek_epoch = Arc::new(AtomicU64::new(0));

        Self {
            video_player: VideoPlayer::init(clock.clone(), seek_epoch.clone()),
            audio_player: AudioPlayer::init(clock.clone(), seek_epoch.clone()),
            clock,
            seek_epoch,
            audio_enabled: false,
            video_enabled: false,
            media_path: None,
        }
    }

    pub fn load_path(&mut self, path: Arc<Path>) {
        if !path.exists() {
            self.media_path = None;
        } else {
            self.media_path = Some(path);
        }
    }

    pub fn is_playing(&self) -> bool {
        self.clock.lock().is_playing()
    }

    pub fn toggle_pause(&mut self) {
        if self.clock.lock().is_playing() {
            self.pause();
        } else if self.clock.lock().is_paused() {
            self.resume();
        } else {
            self.play_video_audio();
        }
    }

    pub fn pause(&mut self) {
        if self.audio_enabled {
            match self.audio_player.pause() {
                Ok(_) => {}
                Err(e) => {
                    warn!("Error hacer pause audio: {e}")
                }
            }
        }
        if self.video_enabled {
            self.video_player.pause();
        }
    }

    pub fn resume(&mut self) {
        if self.audio_enabled {
            match self.audio_player.resume() {
                Ok(_) => {}
                Err(e) => {
                    warn!("Error hacer resume audio: {e}")
                }
            }
        }
        if self.video_enabled {
            self.video_player.resume();
        }
    }

    pub fn stop(&mut self) {
        debug!("Llamado stop de mediaplayer");
        if !self.clock.lock().is_playing() {
            return;
        }

        self.media_path = None;
        if self.audio_enabled {
            match self.audio_player.stop() {
                Ok(_) => {}
                Err(e) => warn!("Error en stop audio: {e}"),
            }
        }
        if self.video_enabled {
            self.video_player.stop();
        }

        self.audio_enabled = false;
        self.video_enabled = false;
        self.stop_players();
        self.clock.lock().reset();
    }

    pub fn stop_players(&mut self) {
        debug!("StopPlayers");
        if !self.clock.lock().is_playing() {
            return;
        }
        self.seek_epoch.store(0, Ordering::SeqCst);
    }

    pub fn play_audio(&mut self) {
        self.stop();
        if let Some(path) = self.media_path.as_deref() {
            self.audio_player.load_path(path.into());

            match self.audio_player.play() {
                Ok(_) => {}
                Err(e) => warn!("Error hacer play audio: {e}"),
            }
            self.audio_enabled = true;

            self.clock.lock().play();
        }
    }

    //tarea: habilitar para imagenes animadas o gifs
    #[allow(unused)]
    pub fn play_video(&mut self) {
        self.stop();

        if let Some(path) = self.media_path.as_deref() {
            self.video_player.load_path(path.into());
            match self.video_player.play() {
                Ok(_) => {}
                Err(e) => warn!("Error hacer play video: {e}"),
            }
            self.video_enabled = true;

            self.clock.lock().play();
        }
    }

    fn reset(&mut self) {
        if self.audio_enabled {
            self.audio_player.reset();
        }
        if self.video_enabled {
            self.video_player.reset();
        }
        self.clock.lock().reset();
        self.seek(0.0);
    }

    pub fn play_video_audio(&mut self) {
        debug!("Estamos llamando a pva");
        if self.clock.lock().is_playing() {
            debug!("Está reproduciendo, paramos");
            self.stop();
        }

        if self.clock.lock().is_ended() {
            debug!("reset en mp");
            self.reset();
        }

        let mut is_err = false;
        if let Some(path) = self.media_path.as_deref() {
            self.audio_player.load_path(path.into());

            match self.audio_player.play() {
                Ok(_) => {
                    debug!("Iniciado audio");
                }
                Err(e) => {
                    is_err = true;
                    warn!("Error hacer play audio: {e}")
                }
            }
            self.audio_enabled = true;

            self.video_player.load_path(path.into());
            match self.video_player.play() {
                Ok(_) => {}
                Err(e) => {
                    is_err = true;
                    warn!("Error hacer play video: {e}")
                }
            }
            self.video_enabled = true;

            std::thread::sleep(Duration::from_millis(50));

            self.clock.lock().play();
        }

        if is_err {
            debug!("Estamos llamando stop por err");
            self.stop();
        }
    }

    pub fn seek(&mut self, secs: f32) {
        self.clock.lock().seek_to(secs);
        self.seek_epoch.fetch_add(1, Ordering::AcqRel);
        if self.audio_enabled {
            self.audio_player.seek(secs);
        }
        if self.video_enabled {
            self.video_player.seek(secs);
        }
    }

    pub fn volume(&mut self, new_volume: f32) {
        self.audio_player.set_volume(new_volume);
    }

    pub fn seek_5s_forward(&mut self) {
        let add = 5.0;
        let current = self.clock.lock().elapsed();

        let clamped = if let Some(dur) = self.clock.lock().media_duration {
            (current + add).clamp(current, dur)
        } else {
            current + add
        };

        self.seek(clamped);
    }

    pub fn seek_5s_back(&mut self) {
        let sus = 5.0;
        let current = self.clock.lock().elapsed();

        let clamped = if let Some(dur) = self.clock.lock().media_duration {
            (current - sus).clamp(0.0, dur)
        } else {
            current - sus
        };

        self.seek(clamped);
    }

    pub fn update<C>(&mut self, ui: &mut Ui, callback: &mut C)
    where
        C: FnMut(&TextureHandle, &mut Ui, (u32, u32)),
    {
        {
            let clock_guard = self.clock.clone();
            let mut clock = clock_guard.lock();
            let mut ended = false;
            if let ClockState::Playing(start) = clock.state {
                let pos = clock.accumulated + start.elapsed().as_secs_f32();
                let duration = clock.media_duration.unwrap_or(0.0);
                if pos >= duration {
                    clock.state = ClockState::Ended;
                    ended = true;
                }
            }
            drop(clock);

            if ended {
                self.stop_players();
            }
        }

        if self.video_enabled {
            self.video_player
                .update(ui, &mut |tex, ui, (w, h)| callback(tex, ui, (w, h)));
        }
    }
}
