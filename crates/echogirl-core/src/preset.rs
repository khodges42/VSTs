// SPDX-License-Identifier: GPL-3.0-or-later
use serde::{Deserialize, Serialize};
pub const VERSION: u32 = 1;
pub const MAX_BYTES: usize = 32 * 1024;
pub const DIVISIONS: [(&str, f32); 13] = [
    ("1/16T", 1. / 6.),
    ("1/16", 0.25),
    ("1/16D", 0.375),
    ("1/8T", 1. / 3.),
    ("1/8", 0.5),
    ("1/8D", 0.75),
    ("1/4T", 2. / 3.),
    ("1/4", 1.),
    ("1/4D", 1.5),
    ("1/2T", 4. / 3.),
    ("1/2", 2.),
    ("1/2D", 3.),
    ("1/1", 4.),
];
pub const CHARACTERS: [&str; 4] = ["CLEAN", "TAPE", "DIRTY", "DUB"];
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Source {
    X,
    Y,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Target {
    Filter,
    Feedback,
    Drive,
    Mix,
    Modulation,
    Spring,
    Spread,
    Resonance,
}
impl Target {
    pub const ALL: [Self; 8] = [
        Self::Filter,
        Self::Feedback,
        Self::Drive,
        Self::Mix,
        Self::Modulation,
        Self::Spring,
        Self::Spread,
        Self::Resonance,
    ];
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub enabled: bool,
    pub source: Source,
    pub target: Target,
    pub amount: f32,
    pub curve: f32,
}
impl Default for Route {
    fn default() -> Self {
        Self {
            enabled: false,
            source: Source::X,
            target: Target::Filter,
            amount: 0.,
            curve: 1.,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub character: u8,
    pub sync: bool,
    pub time_ms: f32,
    pub right_ms: f32,
    pub left_div: u8,
    pub right_div: u8,
    pub link: bool,
    pub ping_pong: bool,
    pub feedback: f32,
    pub filter: f32,
    pub drive: f32,
    pub mix: f32,
    pub input_db: f32,
    pub output_db: f32,
    pub low_cut: f32,
    pub high_cut: f32,
    pub resonance: f32,
    pub wow: f32,
    pub mod_rate: f32,
    pub mod_sync: bool,
    pub mod_beats: f32,
    pub spring: f32,
    pub x: f32,
    pub y: f32,
    pub feed: bool,
    pub throw: bool,
    pub hold: bool,
    pub kill: bool,
    pub dub: bool,
    pub bypass: bool,
    pub dub_feedback: f32,
    pub dub_drive: f32,
    pub dub_filter: f32,
    pub mappings: [Route; 8],
    pub midi_cc: [u8; 8],
}
impl Default for Config {
    fn default() -> Self {
        let mut mappings = [Route::default(); 8];
        mappings[0] = Route {
            enabled: true,
            source: Source::X,
            target: Target::Filter,
            amount: 0.5,
            curve: 1.3,
        };
        mappings[1] = Route {
            enabled: true,
            source: Source::Y,
            target: Target::Feedback,
            amount: 0.8,
            curve: 1.8,
        };
        mappings[2] = Route {
            enabled: true,
            source: Source::Y,
            target: Target::Drive,
            amount: 0.7,
            curve: 3.,
        };
        mappings[3] = Route {
            enabled: true,
            source: Source::Y,
            target: Target::Resonance,
            amount: 0.22,
            curve: 3.,
        };
        Self {
            character: 3,
            sync: true,
            time_ms: 375.,
            right_ms: 500.,
            left_div: 5,
            right_div: 7,
            link: true,
            ping_pong: false,
            feedback: 0.24,
            filter: 0.5,
            drive: 0.1,
            mix: 0.32,
            input_db: 0.,
            output_db: 0.,
            low_cut: 35.,
            high_cut: 16000.,
            resonance: 0.12,
            wow: 0.,
            mod_rate: 0.4,
            mod_sync: false,
            mod_beats: 4.,
            spring: 0.,
            x: 0.5,
            y: 0.45,
            feed: true,
            throw: false,
            hold: false,
            kill: false,
            dub: false,
            bypass: false,
            dub_feedback: 0.12,
            dub_drive: 0.25,
            dub_filter: -0.12,
            mappings,
            midi_cc: [16, 17, 18, 19, 20, 21, 22, 23],
        }
    }
}
impl Config {
    pub fn release_buttons(&mut self) {
        self.throw = false;
        self.hold = false;
        self.kill = false;
        self.dub = false;
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.character > 3
            || self.left_div as usize >= DIVISIONS.len()
            || self.right_div as usize >= DIVISIONS.len()
        {
            return Err("Unknown character or time division".into());
        }
        for (v, min, max, name) in [
            (self.time_ms, 10., 12000., "left time"),
            (self.right_ms, 10., 12000., "right time"),
            (self.feedback, 0., 1.08, "feedback"),
            (self.filter, 0., 1., "filter"),
            (self.drive, 0., 1., "drive"),
            (self.mix, 0., 1., "mix"),
            (self.input_db, -24., 12., "input gain"),
            (self.output_db, -24., 6., "output gain"),
            (self.low_cut, 0., 3000., "low cut"),
            (self.high_cut, 500., 20000., "high cut"),
            (self.resonance, 0., 0.85, "resonance"),
            (self.wow, 0., 1., "wow"),
            (self.mod_rate, 0.05, 8., "mod rate"),
            (self.mod_beats, 0.125, 16., "mod cycle"),
            (self.spring, 0., 0.65, "spring"),
            (self.x, 0., 1., "X"),
            (self.y, 0., 1., "Y"),
            (self.dub_feedback, 0., 0.4, "DUB feedback"),
            (self.dub_drive, 0., 0.7, "DUB drive"),
            (self.dub_filter, -0.5, 0.5, "DUB filter"),
        ] {
            if !v.is_finite() || !(min..=max).contains(&v) {
                return Err(format!("Invalid {name}: {v}"));
            }
        }
        for m in self.mappings {
            if !m.amount.is_finite()
                || !(-2. ..=2.).contains(&m.amount)
                || !m.curve.is_finite()
                || !(0.2..=5.).contains(&m.curve)
            {
                return Err("Invalid performance route".into());
            }
        }
        for (i, cc) in self.midi_cc.iter().enumerate() {
            if *cc > 119 || self.midi_cc[..i].contains(cc) {
                return Err("MIDI CCs must be distinct 0..119".into());
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
            return Err(format!(
                "Unsupported ECHOGIRL preset version {}",
                self.version
            ));
        }
        if self.name.trim().is_empty()
            || self.name.len() > 32
            || self.name.chars().any(char::is_control)
        {
            return Err("Name must be 1..32 UTF-8 bytes, without controls".into());
        }
        if self.description.len() > 1024
            || self
                .description
                .chars()
                .any(|c| c.is_control() && c != '\n')
        {
            return Err("Invalid description".into());
        }
        self.config.validate()
    }
    pub fn from_json(text: &str) -> Result<Self, String> {
        if text.len() > MAX_BYTES {
            return Err("Preset exceeds 32 KiB".into());
        }
        let p: Self = serde_json::from_str(text).map_err(|e| e.to_string())?;
        p.validate()?;
        Ok(p)
    }
    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}
pub const NAMES: [&str; 10] = [
    "Classic Dub Echo",
    "Springy Dub Space",
    "Tape Drift",
    "Jungle Snare Throw",
    "Garage Shuffle",
    "Deep Dubstep Space",
    "Dub Techno Wash",
    "Feedback Screamer",
    "Dirty Digital Echo",
    "Endless Drone",
];
pub fn factory(index: usize) -> Preset {
    let i = index.min(9);
    let mut c = Config::default();
    let description = match i {
        0 => {
            c.character = 3;
            c.left_div = 7;
            c.high_cut = 7500.;
            c.low_cut = 90.;
            "Warm quarter-note dub repeats. X opens the tone then removes bass; Y builds regeneration and grit."
        }
        1 => {
            c.left_div = 5;
            c.right_div = 7;
            c.link = false;
            c.ping_pong = true;
            c.spring = 0.38;
            c.high_cut = 9000.;
            c.resonance = 0.3;
            c.feedback = 0.3;
            "Alternating dotted-eighth and quarter-note echoes with a metallic spring-inspired tail. DUB adds drive and space."
        }
        2 => {
            c.character = 1;
            c.sync = false;
            c.time_ms = 420.;
            c.right_ms = 438.;
            c.link = false;
            c.wow = 0.35;
            c.high_cut = 6500.;
            c.feedback = 0.28;
            "Unequal tape heads with gentle wow. Move TIME during HOLD for pitch dives; X changes the repeat bandwidth."
        }
        3 => {
            c.character = 2;
            c.left_div = 5;
            c.right_div = 7;
            c.link = false;
            c.ping_pong = true;
            c.feed = false;
            c.mix = 0.5;
            c.high_cut = 6500.;
            c.low_cut = 160.;
            c.feedback = 0.33;
            c.drive = 0.3;
            "Gated snare throws into dotted-eighth/quarter ping-pong. Hold THROW over a hit, then ride Y."
        }
        4 => {
            c.character = 0;
            c.left_div = 3;
            c.right_div = 5;
            c.link = false;
            c.ping_pong = true;
            c.feedback = 0.15;
            c.mix = 0.28;
            c.low_cut = 180.;
            c.high_cut = 12000.;
            "Triplet-eighth against dotted-eighth creates a shuffled stereo answer. Keep Y low for short garage accents."
        }
        5 => {
            c.character = 3;
            c.left_div = 8;
            c.right_div = 10;
            c.link = false;
            c.feed = false;
            c.mix = 0.4;
            c.low_cut = 80.;
            c.high_cut = 2800.;
            c.spring = 0.2;
            c.feedback = 0.35;
            "Sparse, dark dotted-quarter/half-note throws. X thins the echoes while leaving the dry sub intact."
        }
        6 => {
            c.character = 1;
            c.left_div = 5;
            c.right_div = 8;
            c.link = false;
            c.wow = 0.18;
            c.spring = 0.32;
            c.feedback = 0.42;
            c.high_cut = 4800.;
            c.mix = 0.38;
            "Drifting dotted echoes and a diffuse metallic wash for chords. HOLD sustains a captured chord; X reshapes its output."
        }
        7 => {
            c.character = 3;
            c.left_div = 2;
            c.right_div = 4;
            c.link = false;
            c.resonance = 0.48;
            c.feedback = 0.45;
            c.drive = 0.32;
            c.low_cut = 300.;
            c.high_cut = 9500.;
            c.mix = 0.3;
            c.y = 0.62;
            c.dub_feedback = 0.24;
            "Short resonant feedback with a narrow usable edge. Upper Y reaches regeneration. KILL fades and clears the echo."
        }
        8 => {
            c.character = 2;
            c.sync = false;
            c.time_ms = 187.;
            c.right_ms = 263.;
            c.link = false;
            c.drive = 0.6;
            c.feedback = 0.3;
            c.high_cut = 4200.;
            c.low_cut = 120.;
            c.mix = 0.36;
            "Two unequal digital times, restricted bandwidth and hard coloration. DUB pushes the repeat loop without muting dry."
        }
        _ => {
            c.character = 1;
            c.left_div = 10;
            c.feedback = 0.48;
            c.wow = 0.1;
            c.spring = 0.25;
            c.mix = 0.5;
            c.y = 0.6;
            c.high_cut = 6500.;
            "Long evolving echoes. Capture audio with HOLD for a drone; change TIME for tape pitch motion, then release or KILL."
        }
    };
    Preset {
        version: VERSION,
        name: NAMES[i].into(),
        description: description.into(),
        config: c,
    }
}
