// SPDX-License-Identifier: GPL-3.0-or-later
use crate::patch::{Envelope, FilterMode, LfoShape, Wave};

/// 33-tap Hamming-windowed half-band FIR; 16 internal samples = 8 host frames.
pub struct Decimator {
    history: [[f32; 33]; 2],
    cursor: usize,
}
impl Default for Decimator {
    fn default() -> Self {
        Self {
            history: [[0.; 33]; 2],
            cursor: 0,
        }
    }
}
impl Decimator {
    pub fn process(&mut self, input: [[f32; 2]; 2]) -> [f32; 2] {
        const SIDE: [f32; 8] = [
            -0.0018878784,
            0.0038624783,
            -0.008242467,
            0.015947115,
            -0.02867656,
            0.05071856,
            -0.09801591,
            0.31594175,
        ];
        let mut out = [0.; 2];
        for (sub, frame) in input.iter().enumerate() {
            for ch in 0..2 {
                self.history[ch][self.cursor] = frame[ch];
                if sub == 0 {
                    let mut x = self.history[ch][(self.cursor + 33 - 16) % 33] * 0.5007058;
                    for (i, c) in SIDE.iter().enumerate() {
                        let a = i * 2 + 1;
                        let b = 32 - a;
                        x += (self.history[ch][(self.cursor + 33 - a) % 33]
                            + self.history[ch][(self.cursor + 33 - b) % 33])
                            * c;
                    }
                    out[ch] = x;
                }
            }
            self.cursor = (self.cursor + 1) % 33;
        }
        out
    }
}

#[derive(Clone, Copy, Default)]
pub struct Adsr {
    pub value: f32,
    stage: u8,
    release_step: f32,
}
impl Adsr {
    pub fn trigger(&mut self) {
        self.stage = 1;
    }
    pub fn release(&mut self) {
        if self.stage != 0 && self.stage != 4 {
            self.stage = 4;
            self.release_step = 0.;
        }
    }
    pub fn held(&self) -> bool {
        self.stage > 0 && self.stage < 4
    }
    pub fn finished(&self) -> bool {
        self.stage == 0
    }
    pub fn next(&mut self, e: Envelope, sr: f32, decay: f32) -> f32 {
        match self.stage {
            1 => {
                self.value += 1. / (e.attack * sr);
                if self.value >= 1. {
                    self.value = 1.;
                    self.stage = 2;
                }
            }
            2 => {
                self.value +=
                    (e.sustain - self.value) * (1. - (-5. / (e.decay * decay * sr)).exp());
                if (self.value - e.sustain).abs() < 0.0001 {
                    self.value = e.sustain;
                    self.stage = 3;
                }
            }
            3 => self.value = e.sustain,
            4 => {
                if self.release_step == 0. {
                    self.release_step = self.value / (e.release * sr);
                }
                self.value -= self.release_step;
                if self.value <= 0.000001 {
                    self.value = 0.;
                    self.stage = 0;
                }
            }
            _ => {}
        }
        self.value
    }
}
/// Topology-preserving state variable filter; bounded Q for aggressive modulation.
#[derive(Clone, Copy, Default)]
pub struct Filter {
    ic1: f32,
    ic2: f32,
}
impl Filter {
    pub fn process(&mut self, x: f32, g: f32, res: f32, mode: FilterMode) -> f32 {
        let k = 2. - res * 1.9;
        let a1 = 1. / (1. + g * (g + k));
        let v1 = a1 * (self.ic1 + g * (x - self.ic2));
        let v2 = self.ic2 + g * v1;
        self.ic1 = 2. * v1 - self.ic1;
        self.ic2 = 2. * v2 - self.ic2;
        // Flush tiny filter memories rather than depend on host FTZ/DAZ state.
        if self.ic1.abs() < 1e-20 {
            self.ic1 = 0.;
        }
        if self.ic2.abs() < 1e-20 {
            self.ic2 = 0.;
        }
        match mode {
            FilterMode::Low12 | FilterMode::Low24 => v2,
            FilterMode::High12 => x - k * v1 - v2,
            FilterMode::Band12 => v1 * k,
        }
    }
}
fn blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        t + t - t * t - 1.
    } else if t > 1. - dt {
        let t = (t - 1.) / dt;
        t * t + t + t + 1.
    } else {
        0.
    }
}
pub fn oscillator(p: f32, dt: f32, pw: f32, w: Wave, triangle: &mut f32, noise: &mut u32) -> f32 {
    match w {
        Wave::Saw => 2. * p - 1. - blep(p, dt),
        Wave::Pulse => {
            let base = if p < pw { 1. } else { -1. };
            base + blep(p, dt) - blep((p - pw).rem_euclid(1.), dt)
        }
        Wave::Triangle => {
            let square = if p < 0.5 { 1. } else { -1. };
            let square = square + blep(p, dt) - blep((p + 0.5).fract(), dt);
            *triangle = (*triangle + 4. * dt * square) * (1. - dt * 0.05);
            *triangle
        }
        Wave::Sine => (p * std::f32::consts::TAU).sin(),
        Wave::Noise => {
            *noise ^= *noise << 13;
            *noise ^= *noise >> 17;
            *noise ^= *noise << 5;
            *noise as f32 / u32::MAX as f32 * 2. - 1.
        }
    }
}
pub fn lfo(p: f32, shape: LfoShape) -> f32 {
    match shape {
        LfoShape::Sine => (p * std::f32::consts::TAU).sin(),
        LfoShape::Triangle => 1. - 4. * (p - 0.5).abs(),
        LfoShape::Saw => p * 2. - 1.,
        LfoShape::Square => {
            if p < 0.5 {
                1.
            } else {
                -1.
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn halfband_has_unity_dc_and_eight_frame_latency() {
        let mut d = Decimator::default();
        let mut impulse = Vec::new();
        for i in 0..40 {
            impulse.push(
                d.process(if i == 0 {
                    [[1.; 2], [0.; 2]]
                } else {
                    [[0.; 2]; 2]
                })[0],
            );
        }
        let peak = impulse
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
            .unwrap()
            .0;
        assert_eq!(peak, 8);
        for _ in 0..100 {
            d.process([[1.; 2]; 2]);
        }
        assert!((d.process([[1.; 2]; 2])[0] - 1.).abs() < 1e-5);
    }
    #[test]
    fn polyblep_reduces_folded_saw_energy() {
        let size = 1024;
        let bin = 157;
        let dt = bin as f32 / size as f32;
        let mut naive = Vec::new();
        let mut corrected = Vec::new();
        let mut triangle = 0.;
        let mut noise = 1;
        for i in 0..size {
            let p = (i as f32 * dt).fract();
            naive.push(p * 2. - 1.);
            corrected.push(oscillator(p, dt, 0.5, Wave::Saw, &mut triangle, &mut noise));
        }
        let alias_energy = |v: &[f32]| -> f64 {
            (1..size / 2)
                .filter(|i| ![bin, bin * 2, bin * 3].contains(i))
                .map(|k| {
                    let mut re = 0.;
                    let mut im = 0.;
                    for (i, x) in v.iter().enumerate() {
                        let angle = std::f64::consts::TAU * k as f64 * i as f64 / size as f64;
                        re += *x as f64 * angle.cos();
                        im += *x as f64 * angle.sin();
                    }
                    re * re + im * im
                })
                .sum()
        };
        assert!(alias_energy(&corrected) < alias_energy(&naive) * 0.2);
    }
}
