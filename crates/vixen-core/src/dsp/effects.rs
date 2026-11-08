// SPDX-License-Identifier: GPL-3.0-or-later
use crate::patch::Effects;

struct Line {
    data: Vec<f32>,
    tags: Vec<u64>,
    cursor: usize,
    epoch: u64,
}
impl Line {
    fn new(len: usize) -> Self {
        Self {
            data: vec![0.; len],
            tags: vec![0; len],
            cursor: 0,
            epoch: 1,
        }
    }
    fn reset(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.cursor = 0;
    }
    fn at(&self, i: usize) -> f32 {
        if self.tags[i] == self.epoch {
            self.data[i]
        } else {
            0.
        }
    }
    fn read(&self, delay: f32) -> f32 {
        // A small negative f32 position can rem_euclid() to exactly `len`
        // through rounding. The chorus continues reading when no notes play, so
        // this otherwise rare wrap can abort a host after a long idle period.
        let p = (self.cursor as f64 - delay.clamp(1., self.data.len() as f32 - 2.) as f64)
            .rem_euclid(self.data.len() as f64);
        let i = (p as usize) % self.data.len();
        let a = self.at(i);
        a + (self.at((i + 1) % self.data.len()) - a) * p.fract() as f32
    }
    fn write(&mut self, x: f32) {
        self.data[self.cursor] = if x.abs() < 1e-20 { 0. } else { x };
        self.tags[self.cursor] = self.epoch;
        self.cursor = (self.cursor + 1) % self.data.len();
    }
}
pub struct EffectsProcessor {
    sr: f32,
    chorus: [Line; 2],
    delay: [Line; 2],
    comb: [Line; 4],
    phase: f32,
    delay_time: f32,
    dc_x: [f32; 2],
    dc_y: [f32; 2],
    dc_pole: f32,
    time_slew: f32,
}
impl EffectsProcessor {
    pub fn heap_bytes(&self) -> usize {
        self.chorus
            .iter()
            .chain(self.delay.iter())
            .chain(self.comb.iter())
            .map(|l| {
                l.data.capacity() * std::mem::size_of::<f32>()
                    + l.tags.capacity() * std::mem::size_of::<u64>()
            })
            .sum()
    }
    pub fn new(sr: f32) -> Self {
        Self {
            sr,
            chorus: std::array::from_fn(|_| Line::new((sr * 0.05) as usize + 8)),
            delay: std::array::from_fn(|_| Line::new((sr * 6.) as usize + 8)),
            comb: std::array::from_fn(|i| {
                Line::new((sr * [0.0297, 0.0371, 0.0411, 0.0437][i]) as usize + 8)
            }),
            phase: 0.,
            delay_time: sr * 0.375,
            dc_x: [0.; 2],
            dc_y: [0.; 2],
            dc_pole: (-std::f32::consts::TAU * 10. / sr).exp(),
            time_slew: 1. - (-1. / (sr * 0.2)).exp(),
        }
    }
    pub fn reset(&mut self) {
        for line in self
            .chorus
            .iter_mut()
            .chain(self.delay.iter_mut())
            .chain(self.comb.iter_mut())
        {
            line.reset();
        }
        self.dc_x = [0.; 2];
        self.dc_y = [0.; 2];
        self.phase = 0.;
        self.delay_time = self.sr * 0.375;
    }
    pub fn process(
        &mut self,
        mut x: [f32; 2],
        e: &Effects,
        chorus: f32,
        delay: f32,
        reverb: f32,
        bpm: f32,
    ) -> [f32; 2] {
        self.phase = (self.phase + e.chorus_rate / self.sr).fract();
        for (ch, sample) in x.iter_mut().enumerate() {
            let t = (0.012 + 0.004 * (self.phase * std::f32::consts::TAU + ch as f32 * 1.7).sin())
                * self.sr;
            let wet = self.chorus[ch].read(t);
            self.chorus[ch].write(*sample);
            *sample = (*sample + wet * chorus * 0.8) / (1. + chorus * 0.3);
        }
        // Slewed read position avoids discontinuities on tempo changes (audible tape glide).
        self.delay_time += (self.sr * 60. / bpm * e.delay_beats - self.delay_time) * self.time_slew;
        let dl = [
            self.delay[0].read(self.delay_time),
            self.delay[1].read(self.delay_time * 1.003),
        ];
        for ch in 0..2 {
            self.delay[ch].write((x[ch] + dl[1 - ch] * e.feedback).tanh());
            x[ch] += dl[ch] * delay;
        }
        let mono = (x[0] + x[1]) * 0.5;
        let mut room = [0.; 2];
        for (i, line) in self.comb.iter_mut().enumerate() {
            let z = line.read(line.data.len() as f32 - 3.);
            line.write((mono + z * 0.72).tanh());
            room[i % 2] += z * 0.35;
        }
        for ch in 0..2 {
            x[ch] += room[ch] * reverb;
            let y = x[ch] - self.dc_x[ch] + self.dc_pole * self.dc_y[ch];
            self.dc_x[ch] = x[ch];
            self.dc_y[ch] = if y.abs() < 1e-20 { 0. } else { y };
            x[ch] = y;
        }
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fractional_read_just_before_wrap_stays_inside_the_line() {
        // FL64's 2026-11-06 dump: effects.rs:24, index == len == 2213.
        // f32 rem_euclid rounds this small negative offset up to the full length.
        let mut line = Line::new(2213);
        for i in 0..2213 {
            line.write(i as f32 / 2213.);
        }
        line.cursor = 353;
        let delay = f32::from_bits(353_f32.to_bits() + 1);
        let value = line.read(delay);
        assert!(value.is_finite());
        // Just before index zero, interpolation approaches the first sample.
        assert!(value.abs() < 0.001, "{value}");
    }

    #[test]
    fn fractional_wrap_interpolates_both_sides_at_supported_line_sizes() {
        for sr in [8000., 44100., 48000., 96000., 192000.] {
            for seconds in [0.0297, 0.05, 6.] {
                let n = (sr * seconds) as usize + 8;
                let mut line = Line::new(n);
                for (index, value) in [(0, 0.25), (1, 0.75), (n - 1, -0.75)] {
                    line.data[index] = value;
                    line.tags[index] = line.epoch;
                }
                for cursor in [1, 353.min(n - 3), n / 2, n - 4] {
                    line.cursor = cursor;
                    for offset in -4_i32..=4 {
                        let delay = f32::from_bits(
                            (cursor as f32)
                                .to_bits()
                                .checked_add_signed(offset)
                                .unwrap(),
                        );
                        let delta = cursor as f64 - delay.clamp(1., n as f32 - 2.) as f64;
                        let expected = 0.25 + delta * if delta < 0. { 1. } else { 0.5 };
                        let y = line.read(delay);
                        assert!(
                            (y as f64 - expected).abs() < 1e-6,
                            "sr={sr}, n={n}, cursor={cursor}, delay={delay}, y={y}, expected={expected}"
                        );
                    }
                }
            }
        }
    }
}
