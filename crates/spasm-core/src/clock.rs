// SPDX-License-Identifier: GPL-3.0-or-later
#[derive(Clone, Copy, Debug, Default)]
pub struct Transport {
    pub bpm: Option<f64>,
    pub beat: Option<f64>,
    pub sample: Option<i64>,
    pub playing: Option<bool>,
}
#[derive(Clone, Copy, Debug)]
pub struct Clock {
    pub beat: f64,
    pub bpm: f64,
    pub host: bool,
    pub playing: bool,
    sample: Option<i64>,
    processed: u64,
    initialized: bool,
    sr: f64,
}
impl Clock {
    /// Stable decisions at a mathematically exact musical boundary, independent
    /// of how often the host re-anchors its floating-point position.
    pub fn step(&self, division: f64) -> i64 {
        ((self.beat + 1e-9) / division).floor() as i64
    }
    pub fn new(sr: f32) -> Self {
        Self {
            beat: 0.,
            bpm: 120.,
            host: false,
            playing: false,
            sample: None,
            processed: 0,
            initialized: false,
            sr: sr as f64,
        }
    }
    /// Returns (timeline discontinuity, transport start, transport stop).
    pub fn set(&mut self, t: Transport) -> (bool, bool, bool) {
        if let Some(bpm) = t.bpm.filter(|v| v.is_finite() && *v > 0.) {
            self.bpm = bpm.clamp(20., 400.);
        }
        let playing = t.playing.unwrap_or(false);
        let start = self.initialized && playing && !self.playing;
        let stop = self.initialized && !playing && self.playing;
        let beat = t.beat.filter(|v| v.is_finite());
        let sample_jump = if let (Some(a), Some(b)) = (t.sample, self.sample) {
            if playing {
                a.abs_diff(b.saturating_add(self.processed.min(i64::MAX as u64) as i64)) > 2
            } else {
                a.abs_diff(b) > 2
            }
        } else {
            false
        };
        let beat_jump =
            playing && self.host && beat.is_some_and(|b| (b - self.beat).abs() > 1. / 128.);
        let jump = self.initialized && (sample_jump || beat_jump) && !start && !stop;
        self.host = playing && beat.is_some();
        if self.host {
            self.beat = beat.unwrap();
        }
        self.playing = playing;
        self.sample = t.sample;
        self.processed = 0;
        self.initialized = true;
        (jump, start, stop)
    }
    pub fn tick(&mut self) {
        self.beat += self.bpm / (60. * self.sr);
        self.processed = self.processed.saturating_add(1);
    }
    pub fn boundary(&self, division: f64) -> f64 {
        if division <= 0. {
            self.beat
        } else {
            ((self.beat - 1e-9) / division).ceil() * division
        }
    }
}
