// SPDX-License-Identifier: GPL-3.0-or-later
//! Host-independent, bounded synthesizer. Construct/deserialize outside process().
pub mod dsp;
pub mod factory;
pub mod patch;
pub use dsp::{Controls, Engine, Event};
pub use patch::{Patch, Synth};
