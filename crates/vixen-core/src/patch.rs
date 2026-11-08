// SPDX-License-Identifier: GPL-3.0-or-later
use serde::{Deserialize, Serialize};

pub const PATCH_VERSION: u32 = 1;
pub const MAX_PATCH_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Performance {
    pub x: f32,
    pub y: f32,
    pub cutoff: f32,
    pub resonance: f32,
    pub drive: f32,
    pub decay: f32,
    pub glide: f32,
    pub output: f32,
}
impl Default for Performance {
    fn default() -> Self {
        Self {
            x: 0.5,
            y: 0.5,
            cutoff: 0.,
            resonance: 0.,
            drive: 0.,
            decay: 0.,
            glide: 0.,
            output: 0.7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Wave {
    Saw,
    Pulse,
    Triangle,
    Sine,
    Noise,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterMode {
    Low12,
    Low24,
    High12,
    Band12,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceMode {
    Poly,
    Mono,
    Legato,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LfoShape {
    Sine,
    Triangle,
    Saw,
    Square,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    Cutoff,
    Resonance,
    Drive,
    Detune,
    Width,
    Chorus,
    FilterEnv,
    PitchEnv,
    Pwm,
    LfoFilter,
    PhaseMod,
    Decay,
    Delay,
    Reverb,
}
impl Target {
    pub const ALL: [Self; 14] = [
        Self::Cutoff,
        Self::Resonance,
        Self::Drive,
        Self::Detune,
        Self::Width,
        Self::Chorus,
        Self::FilterEnv,
        Self::PitchEnv,
        Self::Pwm,
        Self::LfoFilter,
        Self::PhaseMod,
        Self::Decay,
        Self::Delay,
        Self::Reverb,
    ];
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Source {
    X,
    Y,
    Wheel,
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub enabled: bool,
    pub source: Source,
    pub target: Target,
    pub amount: f32,
    pub curve: f32,
}
impl Default for Mapping {
    fn default() -> Self {
        Self {
            enabled: false,
            source: Source::X,
            target: Target::Cutoff,
            amount: 0.,
            curve: 1.,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}
impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack: 0.005,
            decay: 0.25,
            sustain: 0.7,
            release: 0.12,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Oscillator {
    pub wave: Wave,
    pub level: f32,
    pub semitones: f32,
    pub unison: u8,
    pub detune: f32,
    pub width: f32,
    pub pulse_width: f32,
    pub pwm: f32,
}
impl Default for Oscillator {
    fn default() -> Self {
        Self {
            wave: Wave::Saw,
            level: 0.7,
            semitones: 0.,
            unison: 1,
            detune: 0.12,
            width: 0.7,
            pulse_width: 0.5,
            pwm: 0.,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lfo {
    pub shape: LfoShape,
    pub rate: f32,
    pub sync: bool,
    pub beats: f32,
    pub pitch: f32,
    pub filter: f32,
    pub pwm: f32,
}
impl Default for Lfo {
    fn default() -> Self {
        Self {
            shape: LfoShape::Sine,
            rate: 0.7,
            sync: false,
            beats: 1.,
            pitch: 0.,
            filter: 0.,
            pwm: 0.,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Effects {
    pub chorus: f32,
    pub chorus_rate: f32,
    pub delay: f32,
    pub delay_beats: f32,
    pub feedback: f32,
    pub reverb: f32,
}
impl Default for Effects {
    fn default() -> Self {
        Self {
            chorus: 0.,
            chorus_rate: 0.4,
            delay: 0.,
            delay_beats: 0.75,
            feedback: 0.32,
            reverb: 0.08,
        }
    }
}
/// All audio-facing patch data is Copy: no ownership reclamation on the callback.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Synth {
    pub oscillators: [Oscillator; 3],
    pub amp: Envelope,
    pub filter_env: Envelope,
    pub pitch_env: Envelope,
    pub pitch_amount: f32,
    pub filter_mode: FilterMode,
    pub cutoff: f32,
    pub resonance: f32,
    pub filter_amount: f32,
    pub keytrack: f32,
    pub drive: f32,
    pub phase_mod: f32,
    pub velocity: f32,
    pub accent: f32,
    pub lfos: [Lfo; 2],
    pub effects: Effects,
    pub voice_mode: VoiceMode,
    pub voices: u8,
    pub glide: f32,
    pub bend_range: f32,
    pub gain: f32,
    pub mappings: [Mapping; 12],
    pub x_cc: u8,
    pub y_cc: u8,
}
impl Default for Synth {
    fn default() -> Self {
        let mut oscillators = [Oscillator::default(); 3];
        oscillators[1].level = 0.;
        oscillators[2].level = 0.;
        Self {
            oscillators,
            amp: Envelope::default(),
            filter_env: Envelope {
                sustain: 0.,
                ..Envelope::default()
            },
            pitch_env: Envelope {
                attack: 0.001,
                decay: 0.13,
                sustain: 0.,
                release: 0.08,
            },
            pitch_amount: 0.,
            filter_mode: FilterMode::Low24,
            cutoff: 1800.,
            resonance: 0.18,
            filter_amount: 3.,
            keytrack: 0.4,
            drive: 0.1,
            phase_mod: 0.,
            velocity: 0.65,
            accent: 0.,
            lfos: [Lfo::default(); 2],
            effects: Effects::default(),
            voice_mode: VoiceMode::Poly,
            voices: 12,
            glide: 0.,
            bend_range: 2.,
            gain: 0.45,
            mappings: [Mapping::default(); 12],
            x_cc: 16,
            y_cc: 17,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Patch {
    pub version: u32,
    pub name: String,
    pub author: String,
    pub category: String,
    pub description: String,
    pub performance: String,
    pub x_label: String,
    pub y_label: String,
    pub synth: Synth,
    pub controls: Performance,
}
fn range(v: f32, min: f32, max: f32, label: &str) -> Result<(), String> {
    if v.is_finite() && (min..=max).contains(&v) {
        Ok(())
    } else {
        Err(format!("Invalid {label}: {v}"))
    }
}
impl Patch {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != PATCH_VERSION {
            return Err(format!(
                "Unsupported patch version {} (expected {PATCH_VERSION})",
                self.version
            ));
        }
        for (s, max) in [
            (&self.name, 80),
            (&self.author, 80),
            (&self.category, 80),
            (&self.x_label, 40),
            (&self.y_label, 40),
        ] {
            if s.len() > max || s.chars().any(char::is_control) {
                return Err("Metadata is too long or contains control characters".into());
            }
        }
        for s in [&self.description, &self.performance] {
            if s.len() > 1024
                || s.chars()
                    .any(|c| c.is_control() && !['\n', '\t'].contains(&c))
            {
                return Err(
                    "Patch notes are too long or contain unsupported control characters".into(),
                );
            }
        }
        if self.name.trim().is_empty() {
            return Err("Patch needs a name".into());
        }
        let c = self.controls;
        for (v, min, max, n) in [
            (c.x, 0., 1., "X"),
            (c.y, 0., 1., "Y"),
            (c.cutoff, -1., 1., "brightness"),
            (c.resonance, -1., 1., "performance resonance"),
            (c.drive, -1., 1., "performance drive"),
            (c.decay, -1., 1., "character"),
            (c.glide, 0., 2., "performance glide"),
            (c.output, 0., 1., "output"),
        ] {
            range(v, min, max, n)?;
        }
        let s = &self.synth;
        for o in s.oscillators {
            range(o.level, 0., 1., "oscillator level")?;
            range(o.semitones, -36., 36., "tuning")?;
            if !(1..=7).contains(&o.unison) {
                return Err("Unison must be 1..7".into());
            }
            range(o.detune, 0., 1., "detune")?;
            range(o.width, 0., 1., "width")?;
            range(o.pulse_width, 0.05, 0.95, "pulse width")?;
            range(o.pwm, 0., 0.45, "PWM")?;
        }
        for e in [s.amp, s.filter_env, s.pitch_env] {
            range(e.attack, 0.001, 10., "attack")?;
            range(e.decay, 0.001, 10., "decay")?;
            range(e.sustain, 0., 1., "sustain")?;
            range(e.release, 0.002, 10., "release")?;
        }
        for (v, min, max, name) in [
            (s.cutoff, 25., 20000., "cutoff"),
            (s.resonance, 0., 0.95, "resonance"),
            (s.filter_amount, -8., 8., "filter envelope"),
            (s.pitch_amount, -36., 36., "pitch envelope"),
            (s.keytrack, 0., 1., "keytrack"),
            (s.drive, 0., 1., "drive"),
            (s.phase_mod, 0., 3., "phase modulation"),
            (s.velocity, 0., 1., "velocity"),
            (s.accent, 0., 1., "accent"),
            (s.glide, 0., 2., "glide"),
            (s.bend_range, 0., 24., "pitch bend"),
            (s.gain, 0., 1., "gain"),
        ] {
            range(v, min, max, name)?;
        }
        if !(1..=32).contains(&s.voices) {
            return Err("Voices must be 1..32".into());
        }
        for l in s.lfos {
            for (v, min, max, n) in [
                (l.rate, 0.01, 30., "LFO rate"),
                (l.beats, 0.0625, 16., "LFO beats"),
                (l.pitch, 0., 2., "LFO pitch"),
                (l.filter, 0., 4., "LFO filter"),
                (l.pwm, 0., 0.45, "LFO PWM"),
            ] {
                range(v, min, max, n)?;
            }
        }
        let e = s.effects;
        for (v, min, max, n) in [
            (e.chorus, 0., 1., "chorus"),
            (e.chorus_rate, 0.05, 4., "chorus rate"),
            (e.delay, 0., 0.7, "delay"),
            (e.delay_beats, 0.0625, 2., "delay beats"),
            (e.feedback, 0., 0.75, "feedback"),
            (e.reverb, 0., 0.6, "reverb"),
        ] {
            range(v, min, max, n)?;
        }
        for m in s.mappings {
            range(m.amount, -4., 4., "mapping amount")?;
            range(m.curve, 0.2, 5., "mapping curve")?;
        }
        if s.x_cc > 119
            || s.y_cc > 119
            || s.x_cc == s.y_cc
            || [1, 64].contains(&s.x_cc)
            || [1, 64].contains(&s.y_cc)
        {
            return Err("X/Y CCs must be distinct 0..119, excluding wheel 1 and sustain 64".into());
        }
        Ok(())
    }
    pub fn from_json(json: &str) -> Result<Self, String> {
        if json.len() > MAX_PATCH_BYTES {
            return Err("Patch exceeds 64 KiB".into());
        }
        let patch: Self = serde_json::from_str(json).map_err(|e| e.to_string())?;
        patch.validate()?;
        Ok(patch)
    }
    pub fn to_json(&self) -> Result<String, String> {
        self.validate()?;
        serde_json::to_string_pretty(self).map_err(|e| e.to_string())
    }
}
