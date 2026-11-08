// SPDX-License-Identifier: GPL-3.0-or-later
pub mod crossover;
pub mod dynamics;
pub mod engine;
pub mod oversample;
pub mod preset;
pub use engine::{Engine, Meter};
pub use preset::{Band, Config, Preset, factory};
pub const BANDS: usize = 3;
pub const MAX_BANDS: usize = 4;
pub const LATENCY: usize = 24;
pub fn db(x: f32) -> f32 {
    20. * x.abs().max(1e-9).log10()
}
pub fn gain(db: f32) -> f32 {
    10_f32.powf(db / 20.)
}
pub(crate) fn flush(x: f32) -> f32 {
    if x.abs() < 1e-20 { 0. } else { x }
}
pub(crate) fn sanitize(x: f32) -> f32 {
    if x.is_finite() {
        flush(x.clamp(-32., 32.))
    } else {
        0.
    }
}
