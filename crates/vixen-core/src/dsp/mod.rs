// SPDX-License-Identifier: GPL-3.0-or-later
mod effects;
mod primitives;
use crate::patch::*;
use effects::EffectsProcessor;
use primitives::{Adsr, Decimator, Filter, lfo, oscillator};

pub const MAX_VOICES: usize = 32;
pub const LATENCY_SAMPLES: u32 = 8;
pub use crate::patch::Performance as Controls;
#[derive(Debug, Clone, Copy)]
pub enum Event {
    NoteOn {
        note: u8,
        channel: u8,
        velocity: f32,
        id: Option<i32>,
    },
    NoteOff {
        note: u8,
        channel: u8,
        id: Option<i32>,
    },
    Bend {
        channel: u8,
        value: f32,
    },
    Cc {
        channel: u8,
        cc: u8,
        value: f32,
    },
    AllOff,
}
#[derive(Clone, Copy)]
struct Held {
    active: bool,
    released: bool,
    note: u8,
    channel: u8,
    velocity: f32,
    id: Option<i32>,
    age: u64,
}
impl Default for Held {
    fn default() -> Self {
        Self {
            active: false,
            released: false,
            note: 0,
            channel: 0,
            velocity: 0.,
            id: None,
            age: 0,
        }
    }
}
#[derive(Clone, Copy)]
struct Voice {
    active: bool,
    note: u8,
    channel: u8,
    id: Option<i32>,
    age: u64,
    velocity: f32,
    phases: [[f32; 7]; 3],
    triangle: [[f32; 7]; 3],
    amp: Adsr,
    filter_env: Adsr,
    pitch_env: Adsr,
    filters: [[Filter; 2]; 2],
    pitch: f32,
    noise: u32,
    cutoff_g: f32,
    cutoff_target: f32,
    previous: [f32; 2],
    residual: [f32; 2],
    residual_gain: f32,
}
impl Default for Voice {
    fn default() -> Self {
        Self {
            active: false,
            note: 0,
            channel: 0,
            id: None,
            age: 0,
            velocity: 0.,
            phases: [[0.; 7]; 3],
            triangle: [[0.; 7]; 3],
            amp: Adsr::default(),
            filter_env: Adsr::default(),
            pitch_env: Adsr::default(),
            filters: [[Filter::default(); 2]; 2],
            pitch: 60.,
            noise: 1,
            cutoff_g: 0.1,
            cutoff_target: 0.1,
            previous: [0.; 2],
            residual: [0.; 2],
            residual_gain: 0.,
        }
    }
}
impl Voice {
    fn start(&mut self, h: Held, legato: bool, glide: bool) {
        let prior = self.previous;
        if !legato {
            let old_pitch = self.pitch;
            let was_active = self.active;
            *self = Self::default();
            self.residual = prior;
            self.residual_gain = 1.;
            self.pitch = if glide && was_active {
                old_pitch
            } else {
                h.note as f32
            };
            self.amp.trigger();
            self.filter_env.trigger();
            self.pitch_env.trigger();
            self.noise = (h.age as u32)
                .wrapping_mul(747796405)
                .wrapping_add(2891336453);
            for o in 0..3 {
                for u in 0..7 {
                    self.phases[o][u] = ((o * 19 + u * 37) as f32 * 0.0137).fract();
                    self.triangle[o][u] = 1. - 4. * (self.phases[o][u] - 0.5).abs();
                }
            }
        }
        self.active = true;
        self.note = h.note;
        self.channel = h.channel;
        self.id = h.id;
        self.age = h.age;
        self.velocity = h.velocity;
    }
    fn release(&mut self) {
        self.amp.release();
        self.filter_env.release();
        self.pitch_env.release();
    }
}
#[derive(Clone, Copy)]
struct Effective {
    cutoff: f32,
    resonance: f32,
    drive: f32,
    detune: f32,
    width: f32,
    chorus: f32,
    filter_env: f32,
    pitch_env: f32,
    pwm: f32,
    lfo_filter: f32,
    phase_mod: f32,
    decay: f32,
    delay: f32,
    reverb: f32,
}
#[derive(Clone, Copy)]
struct OscCache {
    ratio: [f32; 7],
    pan: [[f32; 2]; 7],
}
impl OscCache {
    fn new(o: Oscillator, e: Effective) -> Self {
        let mut cache = Self {
            ratio: [1.; 7],
            pan: [[0.; 2]; 7],
        };
        let detune = (o.detune + e.detune).clamp(0., 1.);
        let width = (o.width + e.width).clamp(0., 1.);
        let count = o.unison as usize;
        let gain = o.level / (count as f32).sqrt();
        for u in 0..count {
            let pos = if count == 1 {
                0.
            } else {
                2. * u as f32 / (count - 1) as f32 - 1.
            };
            cache.ratio[u] = 2_f32.powf((o.semitones + pos * detune * 0.65) / 12.);
            let pan = pos * width;
            cache.pan[u] = [
                gain * (0.5 * (1. - pan)).sqrt(),
                gain * (0.5 * (1. + pan)).sqrt(),
            ];
        }
        cache
    }
    fn approach(&mut self, target: Self) {
        for u in 0..7 {
            self.ratio[u] += (target.ratio[u] - self.ratio[u]) * 0.125;
            for ch in 0..2 {
                self.pan[u][ch] += (target.pan[u][ch] - self.pan[u][ch]) * 0.125;
            }
        }
    }
}
impl Effective {
    fn approach(&mut self, target: Self) {
        macro_rules! step {($($f:ident),*)=>{$(self.$f+=(target.$f-self.$f)*0.125;)*};}
        step!(
            cutoff, resonance, drive, detune, width, chorus, filter_env, pitch_env, pwm,
            lfo_filter, phase_mod, decay, delay, reverb
        );
    }
    fn new(s: &Synth, c: Controls, wheel: f32) -> Self {
        let mut e = Self {
            cutoff: s.cutoff * 2_f32.powf(c.cutoff * 4.),
            resonance: s.resonance + c.resonance * 0.5,
            drive: s.drive + c.drive,
            detune: 0.,
            width: 0.,
            chorus: s.effects.chorus,
            filter_env: s.filter_amount,
            pitch_env: s.pitch_amount,
            pwm: 0.,
            lfo_filter: 0.,
            phase_mod: s.phase_mod,
            decay: 2_f32.powf(c.decay * 2.),
            delay: s.effects.delay,
            reverb: s.effects.reverb,
        };
        for m in s.mappings {
            if !m.enabled {
                continue;
            }
            let a = match m.source {
                Source::X => c.x - 0.5,
                Source::Y => c.y - 0.5,
                Source::Wheel => wheel,
            };
            let v = if m.source == Source::Wheel {
                a.powf(m.curve) * m.amount
            } else {
                a.signum() * (a.abs() * 2.).powf(m.curve) * 0.5 * m.amount
            };
            match m.target {
                Target::Cutoff => e.cutoff *= 2_f32.powf(v * 4.),
                Target::Resonance => e.resonance += v * 0.4,
                Target::Drive => e.drive += v,
                Target::Detune => e.detune += v,
                Target::Width => e.width += v,
                Target::Chorus => e.chorus += v,
                Target::FilterEnv => e.filter_env += v * 4.,
                Target::PitchEnv => e.pitch_env += v * 12.,
                Target::Pwm => e.pwm += v * 0.4,
                Target::LfoFilter => e.lfo_filter += v * 3.,
                Target::PhaseMod => e.phase_mod += v,
                Target::Decay => e.decay *= 2_f32.powf(v * 2.),
                Target::Delay => e.delay += v * 0.5,
                Target::Reverb => e.reverb += v * 0.4,
            }
        }
        e.cutoff = e.cutoff.clamp(25., 20000.);
        e.resonance = e.resonance.clamp(0., 0.95);
        e.drive = e.drive.clamp(0., 1.);
        e.chorus = e.chorus.clamp(0., 1.);
        e.phase_mod = e.phase_mod.clamp(0., 3.);
        e.delay = e.delay.clamp(0., 0.7);
        e.reverb = e.reverb.clamp(0., 0.6);
        e
    }
}
/// Owns voices and preallocated FX memory. No host, device, GUI, locks or I/O.
pub struct Engine {
    sample_rate: f32,
    synth: Synth,
    voices: [Voice; MAX_VOICES],
    held: [Held; 128],
    age: u64,
    sustain: [bool; 16],
    bend: [f32; 16],
    wheel: [f32; 16],
    controls: Controls,
    target: Controls,
    smooth: f32,
    lfophase: [f32; 2],
    lfoval: [f32; 2],
    effective: Effective,
    effective_target: Effective,
    osc_cache: [OscCache; 3],
    osc_target: [OscCache; 3],
    tick: u8,
    effects: EffectsProcessor,
    decimator: Decimator,
    last: [f32; 2],
    switch_tail: [f32; 2],
    switch_gain: f32,
    cc_xy: [Option<f32>; 2],
}
fn finite(v: f32, fallback: f32) -> f32 {
    if v.is_finite() { v } else { fallback }
}
impl Engine {
    pub fn new(sample_rate: f32, synth: Synth) -> Self {
        let sample_rate = finite(sample_rate, 48000.).clamp(8000., 192000.);
        let controls = Controls::default();
        Self {
            sample_rate,
            synth,
            voices: [Voice::default(); MAX_VOICES],
            held: [Held::default(); 128],
            age: 0,
            sustain: [false; 16],
            bend: [0.; 16],
            wheel: [0.; 16],
            controls,
            target: controls,
            smooth: 1. - (-1. / (sample_rate * 0.008)).exp(),
            lfophase: [0.; 2],
            lfoval: [0.; 2],
            effective: Effective::new(&synth, controls, 0.),
            effective_target: Effective::new(&synth, controls, 0.),
            osc_cache: synth
                .oscillators
                .map(|o| OscCache::new(o, Effective::new(&synth, controls, 0.))),
            osc_target: synth
                .oscillators
                .map(|o| OscCache::new(o, Effective::new(&synth, controls, 0.))),
            tick: 0,
            effects: EffectsProcessor::new(sample_rate),
            decimator: Decimator::default(),
            last: [0.; 2],
            switch_tail: [0.; 2],
            switch_gain: 0.,
            cc_xy: [None; 2],
        }
    }
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
    /// Owned sample/tag memory plus engine storage, excluding allocator overhead.
    pub fn memory_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.effects.heap_bytes()
    }
    pub fn synth(&self) -> &Synth {
        &self.synth
    }
    pub fn active_voices(&self) -> usize {
        self.voices.iter().filter(|v| v.active).count()
    }
    pub fn xy(&self) -> [f32; 2] {
        [
            self.cc_xy[0].unwrap_or(self.target.x),
            self.cc_xy[1].unwrap_or(self.target.y),
        ]
    }
    pub fn reset(&mut self) {
        self.voices.fill(Voice::default());
        self.held.fill(Held::default());
        self.age = 0;
        self.sustain.fill(false);
        self.bend.fill(0.);
        self.wheel.fill(0.);
        self.lfophase = [0.; 2];
        self.effects.reset();
        self.decimator = Decimator::default();
        self.last = [0.; 2];
        self.switch_gain = 0.;
        self.cc_xy = [None; 2];
        self.controls = self.target;
        self.tick = 0;
        self.effective = Effective::new(&self.synth, self.controls, 0.);
        self.effective_target = self.effective;
        self.osc_cache = self
            .synth
            .oscillators
            .map(|o| OscCache::new(o, self.effective));
        self.osc_target = self.osc_cache;
    }
    /// End old notes at the boundary, preserve a 5 ms decaying residual, reset FX
    /// using generation tags (constant work). No held notes are resurrected.
    pub fn switch_patch(&mut self, synth: Synth) {
        let last = self.last;
        self.reset();
        self.synth = synth;
        self.effective = Effective::new(&synth, self.controls, 0.);
        self.effective_target = self.effective;
        self.osc_cache = synth.oscillators.map(|o| OscCache::new(o, self.effective));
        self.osc_target = self.osc_cache;
        self.switch_tail = last;
        self.switch_gain = 1.;
        self.tick = 0;
    }
    pub fn set_controls(&mut self, c: Controls) {
        // A changed host value takes over its axis from MIDI. CC remains latched otherwise.
        if c.x != self.target.x && self.cc_xy[0] != Some(c.x) {
            self.cc_xy[0] = None;
        }
        if c.y != self.target.y && self.cc_xy[1] != Some(c.y) {
            self.cc_xy[1] = None;
        }
        self.target = Controls {
            x: finite(c.x, 0.5).clamp(0., 1.),
            y: finite(c.y, 0.5).clamp(0., 1.),
            cutoff: finite(c.cutoff, 0.).clamp(-1., 1.),
            resonance: finite(c.resonance, 0.).clamp(-1., 1.),
            drive: finite(c.drive, 0.).clamp(-1., 1.),
            decay: finite(c.decay, 0.).clamp(-1., 1.),
            glide: finite(c.glide, 0.).clamp(0., 2.),
            output: finite(c.output, 0.7).clamp(0., 1.),
        };
    }
    fn latest_held(&self) -> Option<Held> {
        self.held
            .iter()
            .filter(|h| h.active)
            .max_by_key(|h| h.age)
            .copied()
    }
    fn reconcile_mono(&mut self) {
        if let Some(h) = self.latest_held() {
            if self.voices[0].age != h.age {
                self.voices[0].start(
                    h,
                    self.synth.voice_mode == VoiceMode::Legato && self.voices[0].active,
                    true,
                );
            }
        } else {
            self.voices[0].release();
        }
    }
    pub fn event(&mut self, event: Event) {
        match event {
            Event::NoteOn {
                note,
                channel,
                velocity,
                id,
            } if note < 128 && channel < 16 => {
                if velocity <= 0. {
                    self.event(Event::NoteOff { note, channel, id });
                    return;
                }
                self.age = self.age.wrapping_add(1);
                let h = Held {
                    active: true,
                    released: false,
                    note,
                    channel,
                    velocity: finite(velocity, 0.).clamp(0., 1.),
                    id,
                    age: self.age,
                };
                let index = self.held.iter().position(|h| !h.active).unwrap_or_else(|| {
                    self.held
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, h)| h.age)
                        .map(|(i, _)| i)
                        .unwrap_or(0)
                });
                self.held[index] = h;
                if self.synth.voice_mode != VoiceMode::Poly {
                    let legato = self.synth.voice_mode == VoiceMode::Legato
                        && self.voices[0].active
                        && self.voices[0].amp.held();
                    self.voices[0].start(
                        h,
                        legato,
                        self.synth.glide > 0. || self.target.glide > 0.,
                    );
                } else {
                    let count = self.synth.voices as usize;
                    let i = self.voices[..count]
                        .iter()
                        .position(|v| !v.active)
                        .unwrap_or_else(|| {
                            self.voices[..count]
                                .iter()
                                .enumerate()
                                .min_by(|(_, a), (_, b)| {
                                    a.amp.value.total_cmp(&b.amp.value).then(a.age.cmp(&b.age))
                                })
                                .map(|(i, _)| i)
                                .unwrap_or(0)
                        });
                    self.voices[i].start(h, false, false);
                }
            }
            Event::NoteOff { note, channel, id } if channel < 16 => {
                let index = self
                    .held
                    .iter()
                    .enumerate()
                    .filter(|(_, h)| {
                        h.active
                            && !h.released
                            && h.channel == channel
                            && h.note == note
                            && (id.is_none() || h.id == id)
                    })
                    .min_by_key(|(_, h)| h.age)
                    .map(|(i, _)| i);
                if let Some(i) = index {
                    let h = self.held[i];
                    self.held[i].released = true;
                    if !self.sustain[channel as usize] {
                        self.held[i].active = false;
                        for v in &mut self.voices {
                            if self.synth.voice_mode == VoiceMode::Poly
                                && v.active
                                && v.age == h.age
                            {
                                v.release();
                            }
                        }
                    }
                }
                if self.synth.voice_mode != VoiceMode::Poly {
                    self.reconcile_mono();
                }
            }
            Event::Bend { channel, value } if channel < 16 => {
                self.bend[channel as usize] = finite(value, 0.).clamp(-1., 1.)
            }
            Event::Cc { channel, cc, value } if channel < 16 => {
                let value = finite(value, 0.).clamp(0., 1.);
                match cc {
                    1 => self.wheel[channel as usize] = value,
                    64 => {
                        self.sustain[channel as usize] = value >= 0.5;
                        if value < 0.5 {
                            for h in &mut self.held {
                                if h.active && h.released && h.channel == channel {
                                    h.active = false;
                                    for v in &mut self.voices {
                                        if self.synth.voice_mode == VoiceMode::Poly
                                            && v.age == h.age
                                        {
                                            v.release();
                                        }
                                    }
                                }
                            }
                            if self.synth.voice_mode != VoiceMode::Poly {
                                self.reconcile_mono();
                            }
                        }
                    }
                    120 | 123 => {
                        for h in &mut self.held {
                            if h.channel == channel {
                                h.active = false;
                            }
                        }
                        for v in &mut self.voices {
                            if v.channel == channel {
                                if cc == 120 {
                                    v.active = false;
                                } else {
                                    v.release();
                                }
                            }
                        }
                        self.sustain[channel as usize] = false;
                    }
                    _ if cc == self.synth.x_cc => {
                        self.cc_xy[0] = Some(value);
                    }
                    _ if cc == self.synth.y_cc => {
                        self.cc_xy[1] = Some(value);
                    }
                    _ => {}
                }
            }
            Event::AllOff => {
                for v in &mut self.voices {
                    v.release();
                }
                self.held.fill(Held::default());
                self.sustain.fill(false);
            }
            _ => {}
        }
    }
    /// Produce one stereo sample. `bpm` is the host tempo, 120 if unavailable.
    pub fn next(&mut self, bpm: f32) -> [f32; 2] {
        let bpm = finite(bpm, 120.).clamp(20., 400.);
        let mut target = self.target;
        if let Some(x) = self.cc_xy[0] {
            target.x = x;
        }
        if let Some(y) = self.cc_xy[1] {
            target.y = y;
        }
        macro_rules! smooth {($($f:ident),*)=>{$(self.controls.$f+=(target.$f-self.controls.$f)*self.smooth;)*};}
        smooth!(x, y, cutoff, resonance, drive, decay, glide, output);
        let s = &self.synth;
        if self.tick == 0 {
            self.effective_target = Effective::new(
                s,
                self.controls,
                self.wheel.iter().copied().fold(0., f32::max),
            );
            self.osc_target = s
                .oscillators
                .map(|o| OscCache::new(o, self.effective_target));
        }
        self.effective.approach(self.effective_target);
        for i in 0..3 {
            self.osc_cache[i].approach(self.osc_target[i]);
        }
        let e = self.effective;
        for i in 0..2 {
            let l = s.lfos[i];
            let hz = if l.sync { bpm / 60. / l.beats } else { l.rate };
            self.lfophase[i] = (self.lfophase[i] + hz / self.sample_rate).fract();
            self.lfoval[i] = lfo(self.lfophase[i], l.shape);
        }
        let pitch_lfo = self.lfoval[0] * s.lfos[0].pitch + self.lfoval[1] * s.lfos[1].pitch;
        let filter_lfo =
            self.lfoval[0] * (s.lfos[0].filter + e.lfo_filter) + self.lfoval[1] * s.lfos[1].filter;
        let pwm_lfo = self.lfoval[0] * s.lfos[0].pwm + self.lfoval[1] * s.lfos[1].pwm;
        let mut oversampled = [[0.; 2]; 2];
        let sr = self.sample_rate * 2.;
        let glide = (s.glide + self.controls.glide).min(2.);
        let glide_step = if glide > 0.0001 {
            1. - (-1. / (glide * self.sample_rate)).exp()
        } else {
            1.
        };
        for v in &mut self.voices {
            if !v.active {
                continue;
            }
            let amp = v.amp.next(s.amp, self.sample_rate, e.decay);
            if v.amp.finished() {
                v.active = false;
                continue;
            }
            let env = v.filter_env.next(s.filter_env, self.sample_rate, e.decay);
            let penv = v.pitch_env.next(s.pitch_env, self.sample_rate, 1.);
            v.pitch += (v.note as f32 - v.pitch) * glide_step;
            let pitch = v.pitch
                + self.bend[v.channel as usize] * s.bend_range
                + pitch_lfo
                + penv * e.pitch_env;
            let base_freq = 440. * 2_f32.powf((pitch - 69.) / 12.);
            if self.tick == 0 {
                let accent = if v.velocity > 0.8 {
                    s.accent * (v.velocity - 0.8) * 5.
                } else {
                    0.
                };
                let hz = e.cutoff
                    * 2_f32.powf(
                        env * e.filter_env
                            + filter_lfo
                            + (v.note as f32 - 60.) * s.keytrack / 12.
                            + accent,
                    );
                v.cutoff_target =
                    (std::f32::consts::PI * hz.clamp(20., self.sample_rate * 0.4) / sr).tan();
            }
            v.cutoff_g += (v.cutoff_target - v.cutoff_g) * 0.125;
            let vel = 1. - s.velocity + s.velocity * v.velocity;
            for (sub, output) in oversampled.iter_mut().enumerate() {
                let mut raw = [0.; 2];
                let mut modulator = 0.;
                for (oi, o) in s.oscillators.iter().enumerate() {
                    if o.level <= 0. {
                        continue;
                    }
                    let count = o.unison as usize;
                    let cache = self.osc_cache[oi];
                    let mut mono = 0.;
                    for u in 0..count {
                        let hz = base_freq * cache.ratio[u];
                        let dt = (hz / sr).clamp(0.000001, 0.45);
                        let phase = &mut v.phases[oi][u];
                        let pw = (o.pulse_width + self.lfoval[0] * o.pwm + pwm_lfo + e.pwm)
                            .clamp(0.05, 0.95);
                        let p = (*phase + modulator * e.phase_mod * 0.1).rem_euclid(1.);
                        let wave =
                            oscillator(p, dt, pw, o.wave, &mut v.triangle[oi][u], &mut v.noise);
                        *phase = (*phase + dt).fract();
                        raw[0] += wave * cache.pan[u][0];
                        raw[1] += wave * cache.pan[u][1];
                        mono += wave / count as f32;
                    }
                    modulator = mono;
                }
                for ch in 0..2 {
                    let mut x =
                        v.filters[ch][0].process(raw[ch], v.cutoff_g, e.resonance, s.filter_mode);
                    if s.filter_mode == FilterMode::Low24 {
                        x = v.filters[ch][1].process(x, v.cutoff_g, e.resonance, FilterMode::Low12);
                    }
                    let gain = 1. + e.drive * 14.;
                    x = (x * gain).tanh() / (1. + e.drive * 2.).sqrt();
                    let sample = x * amp * vel * 0.28 + v.residual[ch] * v.residual_gain;
                    output[ch] += sample;
                    if sub == 1 {
                        v.previous[ch] = sample;
                    }
                }
            }
            v.residual_gain = (v.residual_gain - 1. / (self.sample_rate * 0.003)).max(0.);
        }
        let out = self.decimator.process(oversampled);
        let mut out = self
            .effects
            .process(out, &s.effects, e.chorus, e.delay, e.reverb, bpm);
        for (ch, x) in out.iter_mut().enumerate() {
            *x = ((*x * s.gain * self.controls.output * 1.8).tanh()
                + self.switch_tail[ch] * self.switch_gain)
                .clamp(-1., 1.);
            if !x.is_finite() {
                *x = 0.;
            }
        }
        self.switch_gain = (self.switch_gain - 1. / (self.sample_rate * 0.005)).max(0.);
        self.last = out;
        self.tick = (self.tick + 1) % 16;
        out
    }
}
