// SPDX-License-Identifier: GPL-3.0-or-later
use crate::BANDS;
use serde::{Deserialize, Serialize};
pub const VERSION: u32 = 1;
pub const MAX_BYTES: usize = 32768;
pub const NAMES: [&str; 8] = [
    "Classic OTT",
    "Hyper",
    "Reese",
    "Drum Smash",
    "Air Lift",
    "Bass Violence",
    "Glue",
    "Destroy",
];
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Band {
    pub up_threshold: f32,
    pub up_ratio: f32,
    pub down_threshold: f32,
    pub down_ratio: f32,
    pub attack_ms: f32,
    pub release_ms: f32,
    pub makeup_db: f32,
    pub knee_db: f32,
    pub up_limit_db: f32,
    pub drive: f32,
    pub sat_enabled: bool,
    pub link: f32,
    pub mix: f32,
    pub bypass: bool,
    pub solo: bool,
}
impl Default for Band {
    fn default() -> Self {
        Self {
            up_threshold: -42.,
            up_ratio: 3.,
            down_threshold: -20.,
            down_ratio: 8.,
            attack_ms: 4.,
            release_ms: 100.,
            makeup_db: 4.,
            knee_db: 6.,
            up_limit_db: 18.,
            drive: 0.,
            sat_enabled: true,
            link: 1.,
            mix: 1.,
            bypass: false,
            solo: false,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub crossovers: [f32; 2],
    pub bands: [Band; BANDS],
    pub amount: f32,
    pub time: f32,
    pub tone_db: [f32; BANDS],
    pub drive: f32,
    pub juice: f32,
    pub punch: f32,
    pub mix: f32,
    pub output_db: f32,
    pub floor_db: f32,
    pub auto_gain: bool,
    pub safety: bool,
    pub bypass: bool,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            crossovers: [120., 2500.],
            bands: [Band::default(); BANDS],
            amount: 0.65,
            time: 1.,
            tone_db: [0.; BANDS],
            drive: 0.,
            juice: 1.,
            punch: 0.15,
            mix: 1.,
            output_db: -3.,
            floor_db: -84.,
            auto_gain: false,
            safety: true,
            bypass: false,
        }
    }
}
fn range(v: f32, min: f32, max: f32, name: &str) -> Result<(), String> {
    if v.is_finite() && (min..=max).contains(&v) {
        Ok(())
    } else {
        Err(format!("Invalid {name}: {v}"))
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        range(self.crossovers[0], 40., 1000., "low crossover")?;
        range(self.crossovers[1], 600., 16000., "high crossover")?;
        if self.crossovers[1] < 1.5 * self.crossovers[0] {
            return Err("Crossovers require at least a 1.5:1 separation".into());
        }
        for (v, min, max, name) in [
            (self.amount, 0., 1.5, "amount"),
            (self.time, 0.1, 4., "time"),
            (self.drive, 0., 1., "drive"),
            (self.juice, 0., 1.5, "juice"),
            (self.punch, 0., 1., "punch"),
            (self.mix, 0., 1., "mix"),
            (self.output_db, -24., 12., "output"),
            (self.floor_db, -110., -50., "noise floor"),
        ] {
            range(v, min, max, name)?;
        }
        for t in self.tone_db {
            range(t, -12., 12., "tone gain")?;
        }
        for b in self.bands {
            for (v, min, max, name) in [
                (b.up_threshold, -72., -6., "upward threshold"),
                (b.up_ratio, 1., 12., "upward ratio"),
                (b.down_threshold, -48., 0., "downward threshold"),
                (b.down_ratio, 1., 40., "downward ratio"),
                (b.attack_ms, 0.1, 150., "attack"),
                (b.release_ms, 10., 1000., "release"),
                (b.makeup_db, -12., 18., "makeup"),
                (b.knee_db, 0., 18., "knee"),
                (b.up_limit_db, 0., 24., "upward limit"),
                (b.drive, 0., 1., "band drive"),
                (b.link, 0., 1., "stereo link"),
                (b.mix, 0., 1., "band mix"),
            ] {
                range(v, min, max, name)?;
            }
            if b.up_threshold > b.down_threshold {
                return Err("Upward threshold must not exceed downward threshold".into());
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preset {
    pub version: u32,
    pub name: String,
    pub description: String,
    pub config: Config,
}
impl Preset {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != VERSION {
            return Err("Unsupported EIS preset version".into());
        }
        if self.name.trim().is_empty()
            || self.name.len() > 32
            || self.name.chars().any(char::is_control)
        {
            return Err("Preset name must be 1–32 UTF-8 bytes, without control characters".into());
        }
        if self.description.len() > 1024
            || self
                .description
                .chars()
                .any(|c| c.is_control() && c != '\n')
        {
            return Err("Invalid preset notes (1024-byte limit)".into());
        }
        self.config.validate()
    }
    pub fn from_json(s: &str) -> Result<Self, String> {
        if s.len() > MAX_BYTES {
            return Err("Preset exceeds 32 KiB".into());
        }
        let p: Self = serde_json::from_str(s).map_err(|e| e.to_string())?;
        p.validate()?;
        Ok(p)
    }
    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}
pub fn factory(index: usize) -> Preset {
    let i = index.min(7);
    let mut c = Config::default();
    c.bands[0].attack_ms = 18.;
    c.bands[0].release_ms = 160.;
    c.bands[1].attack_ms = 3.;
    c.bands[2].attack_ms = 1.;
    c.bands[2].release_ms = 70.;
    let note = match i {
        0 => {
            "Three-band upward/downward density. TENDERIZE scales movement without rewriting the band setup."
        }
        1 => {
            c.amount = 1.1;
            c.bands[1].down_ratio = 20.;
            c.bands[1].down_threshold = -26.;
            c.bands[2].up_threshold = -36.;
            c.bands[2].makeup_db = 6.;
            c.time = 0.7;
            c.output_db = -14.;
            "Fast, dense supersaw/lead treatment with lifted detail and restrained output. Not a mastering default."
        }
        2 => {
            c.crossovers = [100., 1800.];
            c.bands[0].up_ratio = 1.;
            c.bands[0].down_ratio = 4.;
            c.bands[0].attack_ms = 28.;
            c.bands[0].makeup_db = 1.;
            c.bands[1].up_threshold = -36.;
            c.bands[1].down_threshold = -24.;
            c.bands[1].down_ratio = 16.;
            c.bands[2].up_limit_db = 6.;
            c.tone_db = [1., 2., -3.];
            c.punch = 0.3;
            "Protect the sub, animate the Reese midrange and keep the top restrained."
        }
        3 => {
            c.bands[0].attack_ms = 5.;
            c.bands[1].attack_ms = 0.7;
            c.bands[2].attack_ms = 0.4;
            for b in &mut c.bands {
                b.release_ms = 45.;
                b.down_threshold = -24.;
                b.up_limit_db = 12.;
            }
            c.punch = 0.55;
            c.output_db = -5.;
            "Quick percussion compression with PUNCH preserving leading edges; lift ghost notes with JUICE."
        }
        4 => {
            c.amount = 0.5;
            c.crossovers = [150., 4000.];
            c.bands[0].up_ratio = 1.;
            c.bands[0].down_ratio = 2.;
            c.bands[0].makeup_db = 0.;
            c.bands[1].up_limit_db = 6.;
            c.bands[1].down_ratio = 2.;
            c.bands[1].makeup_db = 0.;
            c.bands[2].up_threshold = -32.;
            c.bands[2].up_ratio = 6.;
            c.bands[2].down_ratio = 3.;
            c.bands[2].makeup_db = 3.;
            c.tone_db[2] = 2.;
            c.output_db = -6.;
            "Lift quiet high-band detail while leaving the low bands relatively gentle."
        }
        5 => {
            c.crossovers = [90., 2200.];
            c.amount = 1.;
            c.bands[0].up_ratio = 1.;
            c.bands[0].makeup_db = 0.;
            c.bands[0].down_ratio = 3.;
            c.bands[0].attack_ms = 35.;
            c.bands[1].up_ratio = 8.;
            c.bands[1].down_ratio = 25.;
            c.bands[1].down_threshold = -28.;
            c.bands[2].up_limit_db = 10.;
            c.tone_db[1] = 3.;
            c.output_db = -6.;
            "Dense, aggressive mid/high movement with a slower, upward-free sub band."
        }
        6 => {
            c.amount = 0.35;
            c.punch = 0.45;
            for b in &mut c.bands {
                b.up_ratio = 1.4;
                b.up_limit_db = 4.;
                b.down_ratio = 2.;
                b.down_threshold = -12.;
                b.attack_ms = 25.;
                b.release_ms = 220.;
                b.makeup_db = 0.;
            }
            c.output_db = 0.;
            "Restrained bus dynamics with slow recovery. Level-match and judge it in the track."
        }
        _ => {
            c.amount = 1.3;
            c.time = 0.35;
            c.juice = 1.3;
            c.punch = 0.;
            c.drive = 0.35;
            for b in &mut c.bands {
                b.down_threshold = -30.;
                b.down_ratio = 40.;
                b.up_threshold = -34.;
                b.up_ratio = 10.;
                b.up_limit_db = 24.;
                b.release_ms = 35.;
                b.makeup_db = 8.;
            }
            c.bands[1].drive = 0.2;
            c.output_db = -9.;
            "Deliberately excessive density and oversampled saturation. Turn down before adding more steak."
        }
    };
    Preset {
        version: VERSION,
        name: NAMES[i].into(),
        description: note.into(),
        config: c,
    }
}
