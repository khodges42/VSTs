// SPDX-License-Identifier: GPL-3.0-or-later
use serde::{Deserialize, Serialize};
pub const VERSION: u32 = 1;
pub const MAX_BYTES: usize = 32768;
pub const PAD_NAMES: [&str; 8] = [
    "REPEAT", "REVERSE", "STOP", "DIVE", "GATE", "CRUSH", "CHAOS", "RESET",
];
pub const DIVISIONS: [(&str, f64); 7] = [
    ("1/2", 2.),
    ("1/4", 1.),
    ("1/8", 0.5),
    ("1/16", 0.25),
    ("1/32", 0.125),
    ("1/64", 0.0625),
    ("1/128", 0.03125),
];
pub const QUANTIZE: [(&str, f64); 5] = [
    ("OFF", 0.),
    ("1/4", 1.),
    ("1/8", 0.5),
    ("1/16", 0.25),
    ("1/32", 0.125),
];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    #[default]
    Live,
    Repeat,
    Stutter,
    Reverse,
    Stop,
    Dive,
    Gate,
    Crush,
    Chaos,
}
impl Effect {
    pub const ALL: [Self; 9] = [
        Self::Live,
        Self::Repeat,
        Self::Stutter,
        Self::Reverse,
        Self::Stop,
        Self::Dive,
        Self::Gate,
        Self::Crush,
        Self::Chaos,
    ];
    pub const PRIMARY: [Self; 5] = [
        Self::Repeat,
        Self::Stutter,
        Self::Reverse,
        Self::Stop,
        Self::Dive,
    ];
    pub fn id(self) -> usize {
        Self::ALL.iter().position(|e| *e == self).unwrap_or(0)
    }
    pub fn name(self) -> &'static str {
        [
            "LIVE", "REPEAT", "STUTTER", "REVERSE", "STOP", "DIVE", "GATE", "CRUSH", "CHAOS",
        ][self.id()]
    }
    pub fn bit(self) -> u16 {
        if self.id() == 0 {
            0
        } else {
            1 << (self.id() - 1)
        }
    }
    pub fn captured(self) -> bool {
        Self::PRIMARY.contains(&self)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Source {
    X,
    Y,
    Velocity,
}
impl Source {
    pub const ALL: [Self; 3] = [Self::X, Self::Y, Self::Velocity];
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    Rate,
    Fragment,
    Pitch,
    StopTime,
    GateDepth,
    Bits,
    Reduction,
    Filter,
    Drive,
    Wander,
    Acceleration,
    Mix,
}
impl Target {
    pub const ALL: [Self; 12] = [
        Self::Rate,
        Self::Fragment,
        Self::Pitch,
        Self::StopTime,
        Self::GateDepth,
        Self::Bits,
        Self::Reduction,
        Self::Filter,
        Self::Drive,
        Self::Wander,
        Self::Acceleration,
        Self::Mix,
    ];
    pub fn range(self) -> (f32, f32) {
        match self {
            Self::Rate => (-6., 6.),
            Self::Fragment => (-250., 250.),
            Self::Pitch => (-48., 12.),
            Self::StopTime => (-4000., 4000.),
            Self::Bits => (-12., 12.),
            Self::Reduction => (-63., 63.),
            Self::Filter => (-6., 6.),
            _ => (-1., 1.),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Rate => "Rate / divisions",
            Self::Fragment => "Fragment / ms",
            Self::Pitch => "Pitch / semitones",
            Self::StopTime => "Stop / ms",
            Self::GateDepth => "Gate depth",
            Self::Bits => "Bit depth",
            Self::Reduction => "Sample hold",
            Self::Filter => "Filter / octaves",
            Self::Drive => "Drive",
            Self::Wander => "Pitch wander",
            Self::Acceleration => "Acceleration",
            Self::Mix => "Wet mix",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub enabled: bool,
    pub source: Source,
    pub target: Target,
    pub start: f32,
    pub end: f32,
    pub curve: f32,
}
impl Default for Route {
    fn default() -> Self {
        Self {
            enabled: false,
            source: Source::X,
            target: Target::Rate,
            start: 0.,
            end: 0.,
            curve: 1.,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub history_seconds: f32,
    pub capture_locked: bool,
    pub repeat_div: u8,
    pub stutter_div: u8,
    pub stutter_ms: f32,
    pub stutter_reverse: bool,
    pub repeat_count: u8,
    pub acceleration: f32,
    pub reverse_div: u8,
    pub stop_ms: f32,
    pub stop_sync: bool,
    pub stop_beats: f32,
    pub stop_curve: f32,
    pub dive_ms: f32,
    pub dive_semitones: f32,
    pub pitch_semitones: f32,
    pub gate_div: u8,
    pub gate_pattern: u16,
    pub gate_depth: f32,
    pub gate_attack_ms: f32,
    pub gate_release_ms: f32,
    pub bits: f32,
    pub reduction: f32,
    pub filter_hz: f32,
    pub filter_enabled: bool,
    pub drive: f32,
    pub wander: f32,
    pub mix: f32,
    pub fx_output_db: f32,
    pub crossfade_ms: f32,
    pub edge_ms: f32,
    pub quantize: u8,
    pub latch: bool,
    pub latch_mask: u8,
    pub reset_clear: bool,
    pub auto_damage: bool,
    pub chaos_div: u8,
    pub chaos_mask: u16,
    pub seed: u16,
    pub priority: [u8; 5],
    pub pad_assign: [Effect; 7],
    pub midi_notes: [u8; 8],
    pub midi_channel: u8,
    pub midi_cc: [u8; 2],
    pub x: f32,
    pub y: f32,
    pub routes: [Route; 8],
    pub bypass: bool,
}
impl Default for Config {
    fn default() -> Self {
        let mut routes = [Route::default(); 8];
        for (i, (source, target, end, curve)) in [
            (Source::X, Target::Rate, 3., 1.3),
            (Source::X, Target::Fragment, -30., 1.),
            (Source::Y, Target::Drive, 0.65, 2.),
            (Source::Y, Target::Bits, -8., 1.5),
            (Source::Y, Target::Reduction, 15., 2.),
            (Source::Y, Target::Filter, -2., 1.2),
            (Source::Y, Target::Wander, 0.12, 2.),
        ]
        .into_iter()
        .enumerate()
        {
            routes[i] = Route {
                enabled: true,
                source,
                target,
                start: 0.,
                end,
                curve,
            };
        }
        Self {
            history_seconds: 4.,
            capture_locked: true,
            repeat_div: 3,
            stutter_div: 4,
            stutter_ms: 40.,
            stutter_reverse: false,
            repeat_count: 0,
            acceleration: 0.2,
            reverse_div: 2,
            stop_ms: 700.,
            stop_sync: true,
            stop_beats: 1.,
            stop_curve: 1.7,
            dive_ms: 700.,
            dive_semitones: -24.,
            pitch_semitones: 0.,
            gate_div: 3,
            gate_pattern: 0b1110_1010_1110_1010,
            gate_depth: 1.,
            gate_attack_ms: 1.,
            gate_release_ms: 4.,
            bits: 14.,
            reduction: 1.,
            filter_hz: 18000.,
            filter_enabled: false,
            drive: 0.,
            wander: 0.,
            mix: 1.,
            fx_output_db: 0.,
            crossfade_ms: 5.,
            edge_ms: 1.,
            quantize: 3,
            latch: false,
            latch_mask: 0,
            reset_clear: false,
            auto_damage: true,
            chaos_div: 3,
            chaos_mask: Effect::Repeat.bit() | Effect::Reverse.bit() | Effect::Gate.bit(),
            seed: 1701,
            priority: [4, 3, 2, 0, 1],
            pad_assign: [
                Effect::Repeat,
                Effect::Reverse,
                Effect::Stop,
                Effect::Dive,
                Effect::Gate,
                Effect::Crush,
                Effect::Chaos,
            ],
            midi_notes: [36, 37, 38, 39, 40, 41, 42, 43],
            midi_channel: 0,
            midi_cc: [16, 17],
            x: 0.,
            y: 0.,
            routes,
            bypass: false,
        }
    }
}
fn range(x: f32, lo: f32, hi: f32, name: &str) -> Result<(), String> {
    if x.is_finite() && (lo..=hi).contains(&x) {
        Ok(())
    } else {
        Err(format!("Invalid {name}: {x}"))
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), String> {
        for (x, lo, hi, n) in [
            (self.history_seconds, 0.25, 8., "history"),
            (self.stutter_ms, 2., 250., "fragment"),
            (self.acceleration, 0., 1., "acceleration"),
            (self.stop_ms, 50., 4000., "stop time"),
            (self.stop_beats, 0.125, 4., "stop beats"),
            (self.stop_curve, 0.3, 4., "stop curve"),
            (self.dive_ms, 50., 4000., "dive time"),
            (self.dive_semitones, -48., 0., "dive pitch"),
            (self.pitch_semitones, -48., 12., "pitch"),
            (self.gate_depth, 0., 1., "gate depth"),
            (self.gate_attack_ms, 0.2, 25., "gate attack"),
            (self.gate_release_ms, 0.2, 50., "gate release"),
            (self.bits, 4., 16., "bits"),
            (self.reduction, 1., 64., "sample hold"),
            (self.filter_hz, 100., 20000., "filter"),
            (self.drive, 0., 1., "drive"),
            (self.wander, 0., 1., "wander"),
            (self.mix, 0., 1., "mix"),
            (self.fx_output_db, -12., 6., "wet output"),
            (self.crossfade_ms, 2., 25., "crossfade"),
            (self.edge_ms, 0.1, 10., "edge taper"),
            (self.x, 0., 1., "X"),
            (self.y, 0., 1., "Y"),
        ] {
            range(x, lo, hi, n)?;
        }
        for d in [
            self.repeat_div,
            self.stutter_div,
            self.reverse_div,
            self.gate_div,
            self.chaos_div,
        ] {
            if d as usize >= DIVISIONS.len() {
                return Err("Unknown note division".into());
            }
        }
        if self.quantize > 4
            || self.repeat_count > 64
            || self.latch_mask > 127
            || self.chaos_mask > 127
            || self.seed == 0
            || self.midi_channel > 16
        {
            return Err("Invalid performance setting".into());
        }
        for (i, p) in self.priority.iter().enumerate() {
            if *p > 4 || self.priority[..i].contains(p) {
                return Err("Priority ranks must be unique 0–4".into());
            }
        }
        for (i, n) in self.midi_notes.iter().enumerate() {
            if *n > 127 || self.midi_notes[..i].contains(n) {
                return Err("Pad MIDI notes must be unique 0–127".into());
            }
        }
        if self.midi_cc[0] > 119 || self.midi_cc[1] > 119 || self.midi_cc[0] == self.midi_cc[1] {
            return Err("X/Y CCs must be distinct 0–119".into());
        }
        for r in self.routes {
            let (lo, hi) = r.target.range();
            range(r.start, lo, hi, "route start")?;
            range(r.end, lo, hi, "route end")?;
            range(r.curve, 0.2, 5., "route curve")?;
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
            return Err("Unsupported SPASM preset version".into());
        }
        if self.name.trim().is_empty()
            || self.name.len() > 32
            || self.name.chars().any(char::is_control)
        {
            return Err("Name requires 1–32 UTF-8 bytes without control characters".into());
        }
        if self.description.len() > 1024
            || self
                .description
                .chars()
                .any(|c| c.is_control() && c != '\n')
        {
            return Err("Invalid notes (1024-byte limit)".into());
        }
        self.config.validate()
    }
    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
    pub fn from_json(s: &str) -> Result<Self, String> {
        if s.len() > MAX_BYTES {
            return Err("Preset exceeds 32 KiB".into());
        }
        let p: Self = serde_json::from_str(s).map_err(|e| e.to_string())?;
        p.validate()?;
        Ok(p)
    }
}
pub const NAMES: [&str; 10] = [
    "Amen Murder",
    "Jungle Roll",
    "Drill Stutter",
    "Tape Collapse",
    "Reverse Madness",
    "Digital Shrapnel",
    "Breakcore Meltdown",
    "Half-Time Crush",
    "Glitch Scratch",
    "Emergency Rave Damage",
];
pub fn factory(index: usize) -> Preset {
    let i = index.min(9);
    let mut c = Config::default();
    let description = match i {
        0 => {
            "Lock a break, fracture the repeat with X and add grit with Y. REVERSE/DIVE keep the same captured phrase."
        }
        1 => {
            c.repeat_div = 2;
            c.stutter_ms = 65.;
            c.acceleration = 0.35;
            c.gate_pattern = 0b1110_1111_1010_1111;
            c.chaos_mask = Effect::Repeat.bit() | Effect::Stutter.bit() | Effect::Gate.bit();
            "Rolling eighth-note cuts and syncopated gates; X tightens the roll, Y adds restrained digital dirt."
        }
        2 => {
            c.pad_assign[0] = Effect::Stutter;
            c.stutter_ms = 12.;
            c.stutter_div = 4;
            c.acceleration = 0.8;
            c.repeat_count = 0;
            c.routes[1].end = -10.;
            c.bits = 12.;
            c.chaos_mask = Effect::Stutter.bit() | Effect::Gate.bit();
            "REPEAT becomes a drill stutter. X tightens tiny fragments; Y breaks up their texture."
        }
        3 => {
            c.stop_beats = 2.;
            c.stop_curve = 2.4;
            c.dive_ms = 1400.;
            c.dive_semitones = -36.;
            c.filter_enabled = true;
            c.filter_hz = 12000.;
            c.chaos_mask = Effect::Stop.bit();
            c.routes[0].enabled = false;
            c.routes[1].target = Target::StopTime;
            c.routes[1].end = -500.;
            "STOP slows captured audio to silence; DIVE collapses pitch. X shortens the stop, Y darkens and saturates."
        }
        4 => {
            c.reverse_div = 1;
            c.stutter_reverse = true;
            c.repeat_div = 2;
            c.chaos_mask = Effect::Reverse.bit() | Effect::Stutter.bit();
            c.routes[6].end = 0.3;
            "Backward quarter-note phrases with reverse stutter. X slices time; Y introduces warped pitch and grime."
        }
        5 => {
            c.bits = 9.;
            c.reduction = 4.;
            c.drive = 0.12;
            c.filter_enabled = true;
            c.filter_hz = 12000.;
            c.chaos_mask = Effect::Repeat.bit() | Effect::Reverse.bit() | Effect::Crush.bit();
            "CRUSH fires coarse digital shards; repeat/reverse supply the rhythm. X fragments, Y reduces resolution further."
        }
        6 => {
            c.repeat_div = 4;
            c.stutter_div = 5;
            c.stutter_ms = 14.;
            c.acceleration = 0.9;
            c.chaos_div = 3;
            c.chaos_mask = 127;
            c.seed = 6203;
            c.routes[1].end = -12.;
            c.routes[6].end = 0.5;
            "CHAOS chooses seeded rhythmic cuts. X accelerates fracture; Y pushes pitch wander, saturation and bit damage."
        }
        7 => {
            c.repeat_div = 0;
            c.reverse_div = 0;
            c.pitch_semitones = -12.;
            c.dive_semitones = -12.;
            c.bits = 10.;
            c.reduction = 3.;
            c.quantize = 1;
            c.chaos_mask = Effect::Repeat.bit() | Effect::Dive.bit() | Effect::Crush.bit();
            c.routes[0].end = 1.;
            "Half-speed captured playback stretches pitch and duration together. X adds cuts, Y adds weighty digital crunch."
        }
        8 => {
            c.pad_assign[0] = Effect::Stutter;
            c.stutter_ms = 90.;
            c.stutter_div = 3;
            c.stutter_reverse = true;
            c.quantize = 0;
            c.chaos_mask = Effect::Stutter.bit() | Effect::Reverse.bit();
            c.routes[0].target = Target::Pitch;
            c.routes[0].start = -12.;
            c.routes[0].end = 12.;
            c.routes[1].end = -80.;
            "Immediate scratch-like resampling on a locked capture. X sweeps pitch/fragment size; reverse flips the phrase."
        }
        _ => {
            c.quantize = 0;
            c.repeat_div = 4;
            c.stutter_div = 5;
            c.stutter_ms = 20.;
            c.acceleration = 0.65;
            c.chaos_div = 4;
            c.chaos_mask = 127;
            c.seed = 9013;
            c.drive = 0.1;
            c.bits = 12.;
            c.reset_clear = true;
            c.routes[1].end = -18.;
            "Immediate aggressive cuts and controlled damage. X shreds time; Y wrecks timbre. RESET also clears history after its fade."
        }
    };
    Preset {
        version: VERSION,
        name: NAMES[i].into(),
        description: description.into(),
        config: c,
    }
}
