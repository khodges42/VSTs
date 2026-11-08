// SPDX-License-Identifier: GPL-3.0-or-later
use crate::preset::{Config, DIVISIONS, Source, Target};
use std::f32::consts::{PI, TAU};
pub const MAX_SECONDS: f32 = 12.;
#[derive(Clone, Copy, Debug)]
pub struct Transport {
    pub bpm: f32,
    pub playing: bool,
    pub beat: Option<f64>,
}
impl Default for Transport {
    fn default() -> Self {
        Self {
            bpm: 120.,
            playing: true,
            beat: None,
        }
    }
}
fn finite(x: f32, default: f32) -> f32 {
    if x.is_finite() { x } else { default }
}
fn flush(x: f32) -> f32 {
    if x.abs() < 1e-20 { 0. } else { x }
}
fn ceiling(x: f32, knee: f32, limit: f32) -> f32 {
    let a = x.abs();
    if a <= knee {
        x
    } else {
        x.signum() * (knee + (limit - knee) * (a - knee) / ((limit - knee) + (a - knee)))
    }
}
struct Line {
    data: Vec<f32>,
    tags: Vec<u64>,
    epoch: u64,
    cursor: usize,
}
impl Line {
    fn new(n: usize) -> Self {
        Self {
            data: vec![0.; n],
            tags: vec![0; n],
            epoch: 1,
            cursor: 0,
        }
    }
    fn clear(&mut self) {
        self.epoch = self.epoch.wrapping_add(1);
        self.cursor = 0;
    }
    fn at(&self, i: usize) -> f32 {
        if self.tags[i] == self.epoch {
            self.data[i]
        } else {
            0.
        }
    }
    fn read(&self, time: f32) -> f32 {
        let p = (self.cursor as f64 - time.clamp(1., self.data.len() as f32 - 3.) as f64)
            .rem_euclid(self.data.len() as f64);
        let i = (p as usize) % self.data.len();
        let a = self.at(i);
        a + (self.at((i + 1) % self.data.len()) - a) * p.fract() as f32
    }
    fn write(&mut self, x: f32) {
        self.data[self.cursor] = flush(x);
        self.tags[self.cursor] = self.epoch;
        self.cursor = (self.cursor + 1) % self.data.len();
    }
    fn bytes(&self) -> usize {
        self.data.capacity() * 4 + self.tags.capacity() * 8
    }
}
#[derive(Clone, Copy)]
struct Head {
    time: f32,
    to: f32,
    phase: f32,
    tape: bool,
}
impl Head {
    fn new(time: f32) -> Self {
        Self {
            time,
            to: time,
            phase: 1.,
            tape: false,
        }
    }
    fn read(&mut self, line: &Line, target: f32, tape: bool, sr: f32) -> f32 {
        if tape != self.tape {
            self.time = self.to;
            self.phase = 1.;
            self.tape = tape;
        }
        if tape {
            self.time +=
                ((target - self.time) * (1. - (-1. / (sr * 0.08)).exp())).clamp(-1.5, 0.98);
            self.to = self.time;
            return line.read(self.time);
        }
        if self.phase >= 1. && (target - self.time).abs() > 0.5 {
            self.to = target;
            self.phase = 0.;
        }
        if self.phase < 1. {
            let w = 0.5 - 0.5 * (self.phase * PI).cos();
            let x = line.read(self.time) * (1. - w) + line.read(self.to) * w;
            self.phase = (self.phase + 1. / (sr * 0.02)).min(1.);
            if self.phase >= 1. {
                self.time = self.to;
            }
            x
        } else {
            line.read(self.time)
        }
    }
}
#[derive(Clone, Copy, Default)]
struct Tone {
    lp_blend: f32,
    hp_x: f32,
    hp_y: f32,
    ic1: f32,
    ic2: f32,
    dc_x: f32,
    dc_y: f32,
}
#[derive(Clone, Copy)]
struct ToneCoefficients {
    hp: f32,
    g: f32,
    q: f32,
    lp_on: bool,
    hp_on: bool,
    dc: f32,
    smooth: f32,
}
impl Tone {
    fn process(&mut self, x: f32, coefficients: ToneCoefficients) -> f32 {
        let ToneCoefficients {
            hp,
            g,
            q,
            lp_on,
            hp_on,
            dc,
            smooth,
        } = coefficients;
        let h = x - self.hp_x + hp * self.hp_y;
        self.hp_x = x;
        self.hp_y = flush(h);
        let x = if hp_on { h } else { x };
        let k = 2. - q * 1.9;
        let v1 = (self.ic1 + g * (x - self.ic2)) / (1. + g * (g + k));
        let v2 = self.ic2 + g * v1;
        self.ic1 = flush(2. * v1 - self.ic1);
        self.ic2 = flush(2. * v2 - self.ic2);
        self.lp_blend += ((if lp_on { 1. } else { 0. }) - self.lp_blend) * smooth;
        let y = v2 * self.lp_blend + x * (1. - self.lp_blend);
        let z = y - self.dc_x + dc * self.dc_y;
        self.dc_x = y;
        self.dc_y = flush(z);
        z
    }
}
struct Spring {
    lines: [Line; 6],
    damp: [f32; 2],
    coefficient: f32,
}
impl Spring {
    fn new(sr: f32) -> Self {
        Self {
            lines: std::array::from_fn(|i| {
                Line::new((sr * [0.0067, 0.0113, 0.0179, 0.0239, 0.0311, 0.0377][i]) as usize + 8)
            }),
            damp: [0.; 2],
            coefficient: 1. - (-TAU * 3000. / sr).exp(),
        }
    }
    fn clear(&mut self) {
        for l in &mut self.lines {
            l.clear();
        }
        self.damp = [0.; 2];
    }
    fn process(&mut self, input: [f32; 2]) -> [f32; 2] {
        let mut out = [0.; 2];
        for ch in 0..2 {
            let mut x = input[ch];
            for j in 0..2 {
                let line = &mut self.lines[ch * 2 + j];
                let z = line.read(line.data.len() as f32 - 3.);
                let y = z - x * 0.62;
                line.write(x + y * 0.62);
                x = y;
            }
            let line = &mut self.lines[4 + ch];
            let z = line.read(line.data.len() as f32 - 3.);
            self.damp[ch] = flush(self.damp[ch] + self.coefficient * (z - self.damp[ch]));
            line.write(ceiling(x + self.damp[ch] * 0.78, 1., 2.));
            out[ch] = z * 0.7;
        }
        out
    }
    fn bytes(&self) -> usize {
        self.lines.iter().map(Line::bytes).sum()
    }
}
#[derive(Clone, Copy)]
struct Effective {
    feedback: f32,
    filter: f32,
    drive: f32,
    mix: f32,
    input: f32,
    output: f32,
    res: f32,
    wow: f32,
    spring: f32,
    spread: f32,
    ping: f32,
    hold: f32,
    inject: f32,
    bypass: f32,
}
impl Effective {
    fn new(c: Config, armed: bool) -> Self {
        let mut e = Self {
            feedback: c.feedback,
            filter: c.filter,
            drive: c.drive,
            mix: c.mix,
            input: 10_f32.powf(c.input_db / 20.),
            output: 10_f32.powf(c.output_db / 20.),
            res: c.resonance,
            wow: c.wow,
            spring: c.spring,
            spread: 0.,
            ping: if c.ping_pong { 1. } else { 0. },
            hold: if c.hold && !c.kill && !c.bypass {
                1.
            } else {
                0.
            },
            inject: if ((c.feed && !armed) || c.throw) && !c.hold && !c.kill && !c.bypass {
                1.
            } else {
                0.
            },
            bypass: if c.bypass { 1. } else { 0. },
        };
        for m in c.mappings {
            if !m.enabled {
                continue;
            }
            let a = match m.source {
                Source::X => c.x * 2. - 1.,
                Source::Y => c.y,
            };
            let v = a.signum() * a.abs().powf(m.curve) * m.amount;
            match m.target {
                Target::Filter => e.filter += v,
                Target::Feedback => e.feedback += v,
                Target::Drive => e.drive += v,
                Target::Mix => e.mix += v,
                Target::Modulation => e.wow += v,
                Target::Spring => e.spring += v,
                Target::Spread => e.spread += v,
                Target::Resonance => e.res += v,
            }
        }
        if c.dub {
            e.feedback += c.dub_feedback;
            e.drive += c.dub_drive;
            e.filter += c.dub_filter;
            e.spring += 0.15;
        }
        e.feedback = e.feedback.clamp(0., 1.08);
        e.filter = e.filter.clamp(0., 1.);
        e.drive = e.drive.clamp(0., 1.);
        e.mix = e.mix.clamp(0., 1.);
        e.res = e.res.clamp(0., 0.85);
        e.wow = e.wow.clamp(0., 1.);
        e.spring = e.spring.clamp(0., 0.65);
        e.spread = e.spread.clamp(-0.5, 0.5);
        e
    }
    fn approach(&mut self, t: Self, k: f32) {
        macro_rules! s{($($f:ident),*)=>{$(self.$f+=(t.$f-self.$f)*k;)*};}
        s!(
            feedback, filter, drive, mix, input, output, res, wow, spring, spread, ping, hold,
            inject, bypass
        );
    }
}
/// Target feedback including performance mappings; smoothing runs in the engine.
pub fn performance_feedback(c: Config) -> f32 {
    if c.kill || c.bypass {
        0.
    } else if c.hold {
        1.
    } else {
        Effective::new(c, false).feedback
    }
}
/// Prepared, host-independent stereo delay. Pass validated configurations.
pub struct Engine {
    sr: f32,
    cfg: Config,
    transport: Transport,
    lines: [Line; 2],
    heads: [Head; 2],
    tones: [Tone; 2],
    spring: Spring,
    effective: Effective,
    target: Effective,
    smooth: f32,
    phase: f32,
    flutter: f32,
    tick: u8,
    hp: f32,
    g: f32,
    hp_on: bool,
    lp_on: bool,
    dc: f32,
    armed: bool,
    last_throw: bool,
    last_feed: bool,
    last_kill: bool,
    kill_count: u32,
    kill_gain: f32,
    kill_start: f32,
    bypass_cleared: bool,
    mode: u8,
    mode_fade: f32,
    mode_tail: [f32; 2],
    last_wet: [f32; 2],
    time: [f32; 2],
}
impl Engine {
    pub fn new(sr: f32, cfg: Config) -> Self {
        let sr = finite(sr, 48000.).clamp(8000., 192000.);
        let e = Effective::new(cfg, false);
        let mut s = Self {
            sr,
            cfg,
            transport: Transport::default(),
            lines: std::array::from_fn(|_| Line::new((sr * MAX_SECONDS) as usize + 16)),
            heads: [Head::new(1.); 2],
            tones: [Tone::default(); 2],
            spring: Spring::new(sr),
            effective: e,
            target: e,
            smooth: 1. - (-1. / (sr * 0.008)).exp(),
            phase: 0.,
            flutter: 0.,
            tick: 0,
            hp: 0.,
            g: 0.,
            hp_on: false,
            lp_on: false,
            dc: (-TAU * 8. / sr).exp(),
            armed: false,
            last_throw: false,
            last_feed: cfg.feed,
            last_kill: false,
            kill_count: 0,
            kill_gain: 1.,
            kill_start: 1.,
            bypass_cleared: false,
            mode: cfg.character,
            mode_fade: 0.,
            mode_tail: [0.; 2],
            last_wet: [0.; 2],
            time: [0.; 2],
        };
        s.reset();
        s
    }
    pub fn sample_rate(&self) -> f32 {
        self.sr
    }
    pub fn memory_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.lines.iter().map(Line::bytes).sum::<usize>()
            + self.spring.bytes()
    }
    fn times(&self) -> [f32; 2] {
        let c = self.cfg;
        let bpm = finite(self.transport.bpm, 120.).clamp(20., 400.);
        let l = if c.sync {
            self.sr * 60. / bpm * DIVISIONS[(c.left_div as usize).min(12)].1
        } else {
            (self.sr as f64 * c.time_ms as f64 / 1000.) as f32
        };
        let r = if c.link {
            l
        } else if c.sync {
            self.sr * 60. / bpm * DIVISIONS[(c.right_div as usize).min(12)].1
        } else {
            (self.sr as f64 * c.right_ms as f64 / 1000.) as f32
        };
        [
            l.clamp(2., self.sr * MAX_SECONDS),
            r.clamp(2., self.sr * MAX_SECONDS),
        ]
    }
    fn clear_history(&mut self) {
        for l in &mut self.lines {
            l.clear();
        }
        self.spring.clear();
        self.tones = [Tone::default(); 2];
    }
    pub fn reset(&mut self) {
        self.clear_history();
        self.armed = false;
        self.last_throw = false;
        self.last_feed = self.cfg.feed;
        self.last_kill = false;
        self.kill_count = 0;
        self.kill_gain = 1.;
        self.kill_start = 1.;
        self.phase = 0.;
        self.flutter = 0.;
        self.tick = 0;
        self.mode_fade = 0.;
        self.mode = self.cfg.character;
        self.last_wet = [0.; 2];
        self.bypass_cleared = false;
        self.effective = Effective::new(self.cfg, false);
        self.target = self.effective;
        self.time = self.times();
        self.heads = self.time.map(Head::new);
        self.coefficients();
    }
    pub fn set_config(&mut self, c: Config) {
        if c.throw && !self.last_throw {
            self.armed = true;
        }
        if c.feed && !self.last_feed {
            self.armed = false;
        }
        if c.kill && !self.last_kill {
            self.kill_start = self.kill_gain;
            self.kill_count = (self.sr * 0.005).round() as u32;
        }
        if c.character != self.mode {
            self.mode_tail = self.last_wet;
            self.mode_fade = 1.;
            self.mode = c.character;
        }
        self.last_throw = c.throw;
        self.last_feed = c.feed;
        self.last_kill = c.kill;
        self.cfg = c;
        self.target = Effective::new(c, self.armed);
    }
    pub fn set_transport(&mut self, t: Transport) {
        self.transport = Transport {
            bpm: finite(t.bpm, 120.).clamp(20., 400.),
            ..t
        };
        if self.cfg.mod_sync
            && let Some(beat) = t.beat.filter(|b| b.is_finite())
        {
            self.phase = (beat / self.cfg.mod_beats as f64).rem_euclid(1.) as f32;
        }
    }
    fn coefficients(&mut self) {
        let e = self.effective;
        let c = self.cfg;
        let base_lp = match c.character {
            1 => 11000.,
            2 => 6000.,
            3 => 14000.,
            _ => 20000.,
        };
        let lp = c.high_cut.min(base_lp) * 2_f32.powf(-(0.5 - e.filter).max(0.) * 12.);
        let hp = c.low_cut.max(if e.filter > 0.5 {
            10. * 2_f32.powf((e.filter - 0.5) * 16.)
        } else {
            0.
        });
        self.hp = (-TAU * hp.min(self.sr * 0.35) / self.sr).exp();
        self.g = (PI * lp.clamp(50., self.sr * 0.4) / self.sr).tan();
        self.hp_on = hp > 0.01;
        self.lp_on = lp < 19990.;
    }
    pub fn effective_feedback(&self) -> f32 {
        self.effective.feedback
    }
    pub fn throw_mode(&self) -> bool {
        self.armed || !self.cfg.feed
    }
    pub fn process(&mut self, input: [f32; 2]) -> [f32; 2] {
        let input = input.map(|x| finite(x, 0.).clamp(-32., 32.));
        self.effective.approach(self.target, self.smooth);
        let e = self.effective;
        if self.tick == 0 {
            self.coefficients();
            self.time = self.times();
        }
        let hz = if self.cfg.mod_sync {
            self.transport.bpm / 60. / self.cfg.mod_beats
        } else {
            self.cfg.mod_rate
        };
        if !self.cfg.mod_sync || self.transport.playing {
            self.phase = (self.phase + hz / self.sr).fract();
        }
        self.flutter = (self.flutter + 6.73 / self.sr).fract();
        let mut raw = [0.; 2];
        for (ch, r) in raw.iter_mut().enumerate() {
            let m = ((self.phase * TAU + ch as f32 * 1.7).sin() * 0.003
                + (self.flutter * TAU).sin() * 0.0003)
                * e.wow
                * self.sr;
            let time = (self.time[ch] * (if ch == 1 { 1. + e.spread } else { 1. }) + m)
                .clamp(2., self.sr * MAX_SECONDS);
            *r = self.heads[ch].read(&self.lines[ch], time, self.cfg.character == 1, self.sr);
        }
        let mut wet = [0.; 2];
        let intrinsic = match self.cfg.character {
            1 => 0.1,
            2 => 0.3,
            3 => 0.16,
            _ => 0.,
        };
        let drive = (e.drive + intrinsic).min(1.);
        let gain = 1. + drive * 14.;
        for ch in 0..2 {
            let x = self.tones[ch].process(
                raw[ch],
                ToneCoefficients {
                    hp: self.hp,
                    g: self.g,
                    q: e.res,
                    lp_on: self.lp_on,
                    hp_on: self.hp_on,
                    dc: self.dc,
                    smooth: self.smooth,
                },
            );
            wet[ch] = if drive < 0.00001 {
                x
            } else {
                (x * gain).tanh() / gain
            };
        }
        if self.cfg.kill {
            if self.kill_count > 0 {
                self.kill_gain = self.kill_start * self.kill_count as f32 / (self.sr * 0.005);
                self.kill_count -= 1;
            } else {
                if self.kill_gain != 0. {
                    self.clear_history();
                }
                self.kill_gain = 0.;
            }
        } else {
            self.kill_gain = (self.kill_gain + 1. / (self.sr * 0.005)).min(1.);
        }
        if self.cfg.bypass && e.bypass > 0.999 && !self.bypass_cleared {
            self.clear_history();
            self.bypass_cleared = true;
        }
        if !self.cfg.bypass {
            self.bypass_cleared = false;
        }
        let input_wet = [
            input[0] * (1. - e.ping) + (input[0] + input[1]) * 0.5 * e.ping,
            input[1] * (1. - e.ping),
        ];
        for ch in 0..2 {
            let normal = wet[ch] * e.feedback;
            let loop_sample = normal * (1. - e.hold) + raw[ch] * e.hold;
            let other = wet[1 - ch] * e.feedback * (1. - e.hold) + raw[1 - ch] * e.hold;
            let feedback = loop_sample * (1. - e.ping) + other * e.ping;
            let write =
                (input_wet[ch] * e.input * e.inject + feedback * (1. - e.bypass)) * self.kill_gain;
            self.lines[ch].write(ceiling(write, 1., 2.));
        }
        if (self.cfg.kill && self.kill_count == 0) || (self.cfg.bypass && self.bypass_cleared) {
            wet = [0.; 2];
        }
        let spring = self.spring.process(wet);
        for ch in 0..2 {
            wet[ch] = (wet[ch] * (1. - e.spring) + spring[ch] * e.spring)
                * (1. + drive * 4.)
                * (1. - self.mode_fade)
                + self.mode_tail[ch] * self.mode_fade;
        }
        self.mode_fade = (self.mode_fade - 1. / (self.sr * 0.005)).max(0.);
        // Character tails share the output makeup domain; never apply it twice.
        // Keep KILL downstream of this crossfade so a mode change cannot revive it.
        self.last_wet = wet;
        for value in &mut wet {
            *value *= self.kill_gain;
        }
        let mut out = [0.; 2];
        for ch in 0..2 {
            let mixed = (input[ch] * (1. - e.mix) + wet[ch] * e.mix) * e.output;
            let safe = ceiling(mixed, 0.95, 1.);
            out[ch] = finite(safe * (1. - e.bypass) + input[ch] * e.bypass, 0.);
        }
        self.tick = (self.tick + 1) % 16;
        out
    }
}
