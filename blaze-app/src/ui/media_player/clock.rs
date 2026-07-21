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

use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ClockState {
    Playing(Instant),
    #[default]
    Paused,
    Ended,
}

pub struct PlaybackClock {
    pub state: ClockState,
    pub media_duration: Option<f32>,
    pub accumulated: f32,
}

impl Default for PlaybackClock {
    fn default() -> Self {
        Self {
            state: ClockState::Paused,
            media_duration: None,
            accumulated: 0.0,
        }
    }
}

impl PlaybackClock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_duration(&mut self, duration: f32) {
        self.media_duration = Some(duration);
    }

    pub fn elapsed(&self) -> f32 {
        match self.state {
            ClockState::Playing(start) => self.accumulated + start.elapsed().as_secs_f32(),
            ClockState::Paused => self.accumulated,
            ClockState::Ended => self.media_duration.unwrap_or(0.0),
        }
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.state, ClockState::Playing(_))
    }

    pub fn is_ended(&self) -> bool {
        matches!(self.state, ClockState::Ended)
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.state, ClockState::Paused)
    }

    pub fn reset(&mut self) {
        *self = Self::default();
        tracing::info!(
            "Reloj se supone que reseteado: {}: {:?}",
            self.accumulated,
            self.state
        );
    }

    pub fn play(&mut self) {
        if let ClockState::Paused = self.state {
            self.state = ClockState::Playing(Instant::now());
        }
    }

    pub fn pause(&mut self) {
        self.accumulated = self.elapsed();
        self.state = ClockState::Paused;
    }

    pub fn seek_to(&mut self, secs: f32) {
        if secs.is_nan() {
            return;
        }

        let clamped = secs.max(0.0);
        let clamped = if let Some(dur) = self.media_duration {
            clamped.min(dur)
        } else {
            secs
        };

        self.accumulated = clamped;
        if let ClockState::Playing(_) = self.state {
            self.state = ClockState::Playing(Instant::now());
        }
    }
}
