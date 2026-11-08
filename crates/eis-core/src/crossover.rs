// SPDX-License-Identifier: GPL-3.0-or-later
//! LR4 serial splits, with allpass compensation for all later splits.
use crate::{MAX_BANDS, flush};
use std::f32::consts::{PI, SQRT_2};
#[derive(Clone, Copy, Default)]
struct Svf {
    s1: f32,
    s2: f32,
}
impl Svf {
    fn process(&mut self, x: f32, g: f32) -> [f32; 3] {
        let b = (self.s1 + g * (x - self.s2)) / (1. + g * (g + SQRT_2));
        let l = self.s2 + g * b;
        self.s1 = flush(2. * b - self.s1);
        self.s2 = flush(2. * l - self.s2);
        [l, x - SQRT_2 * b - l, x - 2. * SQRT_2 * b]
    }
}
#[derive(Clone, Copy, Default)]
struct Split {
    first: Svf,
    low: Svf,
    high: Svf,
}
impl Split {
    fn process(&mut self, x: f32, g: f32) -> [f32; 2] {
        let a = self.first.process(x, g);
        [self.low.process(a[0], g)[0], self.high.process(a[1], g)[1]]
    }
}
pub struct Crossover {
    splits: [[Split; MAX_BANDS - 1]; 2],
    phase: [[[Svf; MAX_BANDS - 1]; MAX_BANDS]; 2],
    g: [f32; MAX_BANDS - 1],
    target: [f32; MAX_BANDS - 1],
    count: usize,
    k: f32,
    sr: f32,
}
impl Crossover {
    pub fn new(sr: f32, frequencies: &[f32]) -> Self {
        assert!((1..MAX_BANDS).contains(&frequencies.len()));
        let mut s = Self {
            splits: [[Split::default(); MAX_BANDS - 1]; 2],
            phase: [[[Svf::default(); MAX_BANDS - 1]; MAX_BANDS]; 2],
            g: [0.; MAX_BANDS - 1],
            target: [0.; MAX_BANDS - 1],
            count: frequencies.len() + 1,
            k: 1. - (-1. / (sr * 0.02)).exp(),
            sr,
        };
        s.set_frequencies(frequencies);
        s.g = s.target;
        s
    }
    pub fn set_frequencies(&mut self, f: &[f32]) {
        for (i, v) in f.iter().take(self.count - 1).enumerate() {
            self.target[i] = (PI * v.clamp(20., self.sr * 0.4) / self.sr).tan();
        }
    }
    pub fn reset(&mut self) {
        self.splits = [[Split::default(); MAX_BANDS - 1]; 2];
        self.phase = [[[Svf::default(); MAX_BANDS - 1]; MAX_BANDS]; 2];
        self.g = self.target;
    }
    pub fn process(&mut self, x: [f32; 2]) -> [[f32; 2]; MAX_BANDS] {
        for i in 0..self.count - 1 {
            self.g[i] += (self.target[i] - self.g[i]) * self.k;
        }
        let mut out = [[0.; 2]; MAX_BANDS];
        for ch in 0..2 {
            let mut rest = x[ch];
            for (i, band) in out.iter_mut().enumerate().take(self.count - 1) {
                let split = self.splits[ch][i].process(rest, self.g[i]);
                band[ch] = split[0];
                rest = split[1];
            }
            out[self.count - 1][ch] = rest;
            for (i, band) in out.iter_mut().enumerate().take(self.count - 1) {
                for j in i + 1..self.count - 1 {
                    band[ch] = self.phase[ch][i][j].process(band[ch], self.g[j])[2];
                }
            }
        }
        out
    }
}
pub fn effective_crossovers(c: [f32; 2], sr: f32) -> [f32; 2] {
    let low = c[0].clamp(40., 1000.);
    [low, c[1].clamp(low * 1.5, sr.mul_add(0.4, 0.).min(16000.))]
}
