// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{Band, db, flush};
fn coefficient(sr: f32, seconds: f32) -> f32 {
    -(-1. / (sr * seconds)).exp_m1()
}
/// Soft-knee positive part. Ratios use the conventional >=1 compression form.
fn over(v: f32, k: f32) -> f32 {
    if k <= 0.0001 {
        v.max(0.)
    } else if v <= -k * 0.5 {
        0.
    } else if v >= k * 0.5 {
        v
    } else {
        (v + k * 0.5).powi(2) / (2. * k)
    }
}
pub fn transfer(level_db: f32, b: Band, amount: f32, juice: f32, floor_db: f32) -> [f32; 2] {
    let gate = ((level_db - floor_db) / 18.).clamp(0., 1.);
    let gate = gate * gate * (3. - 2. * gate);
    let up = (((1. - 1. / b.up_ratio) * amount * juice).min(0.99)
        * over(b.up_threshold.min(b.down_threshold) - level_db, b.knee_db))
    .min(b.up_limit_db)
        * gate;
    let down = (-((1. - 1. / b.down_ratio) * amount).min(0.99)
        * over(level_db - b.down_threshold, b.knee_db))
    .max(-48.);
    [up, down]
}
#[derive(Clone, Copy, Default)]
pub struct Dynamics {
    pub up: [f32; 2],
    pub down: [f32; 2],
    detector: [f32; 2],
}
impl Dynamics {
    /// Detector audio is independent of the audio path to admit a future band sidechain.
    #[allow(clippy::too_many_arguments)] // Independent band API uses explicit scalar macro controls.
    pub fn process(
        &mut self,
        detector: [f32; 2],
        b: Band,
        amount: f32,
        juice: f32,
        time: f32,
        punch: f32,
        floor: f32,
        sr: f32,
    ) -> [f32; 2] {
        let da = coefficient(sr, 0.00005);
        let dr = coefficient(sr, 0.005);
        for (ch, v) in detector.iter().enumerate() {
            let v = v.abs();
            let k = if v > self.detector[ch] { da } else { dr };
            self.detector[ch] = flush(self.detector[ch] + (v - self.detector[ch]) * k);
        }
        let peak = self.detector[0].max(self.detector[1]);
        let attack = coefficient(sr, b.attack_ms * time * 0.001);
        let punched = coefficient(sr, b.attack_ms * time * (1. + 12. * punch * punch) * 0.001);
        let release = coefficient(sr, b.release_ms * time * 0.001);
        let up_return = coefficient(sr, 0.0005);
        for ch in 0..2 {
            let level = db(self.detector[ch] * (1. - b.link) + peak * b.link);
            let t = transfer(level, b, amount, juice, floor);
            self.up[ch] = flush(
                self.up[ch]
                    + (t[0] - self.up[ch])
                        * if t[0] > self.up[ch] {
                            attack
                        } else {
                            up_return
                        },
            );
            self.down[ch] = flush(
                self.down[ch]
                    + (t[1] - self.down[ch])
                        * if t[1] < self.down[ch] {
                            punched
                        } else {
                            release
                        },
            );
        }
        [self.up[0] + self.down[0], self.up[1] + self.down[1]]
    }
}
