// SPDX-License-Identifier: GPL-3.0-or-later
pub mod buffer;
pub mod clock;
pub mod engine;
pub mod patch;
pub use clock::Transport;
pub use engine::{Engine, Event, Status};
pub use patch::{Config, Effect, Preset, factory};
pub const MAX_SECONDS: f32 = 8.;
pub(crate) fn finite(x: f32, default: f32) -> f32 {
    if x.is_finite() { x } else { default }
}
pub(crate) fn flush(x: f32) -> f32 {
    if x.abs() < 1e-20 { 0. } else { x }
}
pub(crate) fn clean(x: f32) -> f32 {
    let x = finite(x, 0.).clamp(-32., 32.);
    if x.abs() < 1e-20 { 0. } else { x }
}
