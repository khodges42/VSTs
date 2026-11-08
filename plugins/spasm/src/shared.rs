// SPDX-License-Identifier: GPL-3.0-or-later
use spasm_core::{Effect, Status};
use std::sync::atomic::{AtomicU32, Ordering};
pub const WAVE_SIZE: usize = 256;
pub struct Shared {
    values: [AtomicU32; 19],
    pub wave: [AtomicU32; WAVE_SIZE],
    pub cursor: AtomicU32,
    pub valid: AtomicU32,
}
impl Default for Shared {
    fn default() -> Self {
        Self {
            values: std::array::from_fn(|_| AtomicU32::new(0)),
            wave: std::array::from_fn(|_| AtomicU32::new(0)),
            cursor: AtomicU32::new(0),
            valid: AtomicU32::new(0),
        }
    }
}
impl Shared {
    pub fn publish(&self, s: Status) {
        let a = [
            s.effect.id() as f32,
            s.engaged as f32,
            s.pending as f32,
            s.latched as f32,
            s.history_seconds,
            s.capture_seconds,
            s.capture_age_seconds,
            s.phase,
            s.speed,
            s.reverse as u8 as f32,
            s.repeat_seconds,
            s.short_history as u8 as f32,
            s.host_sync as u8 as f32,
            s.bpm,
            s.beat.rem_euclid(16.) as f32,
            s.input_peak,
            s.output_peak,
            s.capture_serial as f32,
            s.epoch as f32,
        ];
        for (x, v) in self.values.iter().zip(a) {
            x.store(v.to_bits(), Ordering::Relaxed);
        }
    }
    pub fn status(&self) -> Status {
        let a = self
            .values
            .each_ref()
            .map(|v| f32::from_bits(v.load(Ordering::Relaxed)));
        Status {
            effect: Effect::ALL[(a[0] as usize).min(8)],
            engaged: a[1] as u8,
            pending: a[2] as u8,
            latched: a[3] as u8,
            history_seconds: a[4],
            capture_seconds: a[5],
            capture_age_seconds: a[6],
            phase: a[7],
            speed: a[8],
            reverse: a[9] > 0.,
            repeat_seconds: a[10],
            short_history: a[11] > 0.,
            host_sync: a[12] > 0.,
            bpm: a[13],
            beat: a[14] as f64,
            input_peak: a[15],
            output_peak: a[16],
            capture_serial: a[17] as u32,
            epoch: a[18] as u32,
        }
    }
}
#[derive(Default)]
pub struct WaveWriter {
    peak: f32,
    count: u32,
    cursor: usize,
    valid: usize,
    epoch: u32,
    history: f32,
}
impl WaveWriter {
    pub fn push(&mut self, s: &Shared, x: [f32; 2], rate: f32, history: f32, epoch: u32) {
        if epoch != self.epoch || history != self.history {
            *self = Self {
                epoch,
                history,
                ..Self::default()
            };
            s.valid.store(0, Ordering::Relaxed);
            s.cursor.store(0, Ordering::Relaxed);
        }
        let peak = x
            .iter()
            .filter(|v| v.is_finite())
            .map(|v| v.abs().min(32.))
            .fold(0_f32, f32::max);
        self.peak = self.peak.max(peak);
        self.count += 1;
        if self.count >= (rate * history / WAVE_SIZE as f32).round().max(1.) as u32 {
            s.wave[self.cursor].store(self.peak.to_bits(), Ordering::Relaxed);
            self.cursor = (self.cursor + 1) % WAVE_SIZE;
            self.valid = (self.valid + 1).min(WAVE_SIZE);
            s.cursor.store(self.cursor as u32, Ordering::Relaxed);
            s.valid.store(self.valid as u32, Ordering::Release);
            self.count = 0;
            self.peak = 0.;
        }
    }
}
