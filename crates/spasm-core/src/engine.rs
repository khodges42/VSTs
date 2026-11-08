// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    Config, Effect, MAX_SECONDS, Transport,
    buffer::{Bank, Slice},
    clean,
    clock::Clock,
    finite, flush,
    patch::{DIVISIONS, QUANTIZE, Source, Target},
};
use std::f32::consts::{PI, TAU};
#[derive(Clone, Copy, Debug)]
pub enum Event {
    Pad {
        pad: u8,
        down: bool,
        velocity: f32,
    },
    Note {
        note: u8,
        channel: u8,
        on: bool,
        velocity: f32,
    },
    Reset,
    AllNotesOff,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Status {
    pub effect: Effect,
    pub engaged: u8,
    pub pending: u8,
    pub latched: u8,
    pub history_seconds: f32,
    pub capture_seconds: f32,
    pub capture_age_seconds: f32,
    pub phase: f32,
    pub speed: f32,
    pub reverse: bool,
    pub repeat_seconds: f32,
    pub short_history: bool,
    pub host_sync: bool,
    pub bpm: f32,
    pub beat: f64,
    pub input_peak: f32,
    pub output_peak: f32,
    pub capture_serial: u32,
    pub epoch: u32,
}
#[derive(Clone, Copy, Default)]
struct Pad {
    host: bool,
    midi: [u8; 16],
    block_host: bool,
    block_midi: bool,
}
impl Pad {
    fn down(self) -> bool {
        (self.host && !self.block_host) || (self.midi.iter().any(|n| *n > 0) && !self.block_midi)
    }
}
#[derive(Clone, Copy)]
struct Effective {
    rate: f32,
    fragment: f32,
    pitch: f32,
    stop: f32,
    gate: f32,
    bits: f32,
    reduction: f32,
    filter: f32,
    filter_mix: f32,
    drive: f32,
    wander: f32,
    accel: f32,
    mix: f32,
    output: f32,
}
impl Effective {
    fn stop_seconds(self, c: Config, bpm: f64) -> f64 {
        if c.stop_sync {
            (c.stop_beats as f64 * 60. / bpm + (self.stop - c.stop_ms) as f64 * 0.001)
                .clamp(0.05, 12.)
        } else {
            self.stop as f64 * 0.001
        }
    }
    fn new(c: Config, velocity: f32) -> Self {
        let mut e = Self {
            rate: 0.,
            fragment: c.stutter_ms,
            pitch: c.pitch_semitones,
            stop: c.stop_ms,
            gate: c.gate_depth,
            bits: c.bits,
            reduction: c.reduction,
            filter: c.filter_hz,
            filter_mix: c.filter_enabled as u8 as f32,
            drive: c.drive,
            wander: c.wander,
            accel: c.acceleration,
            mix: c.mix,
            output: c.fx_output_db,
        };
        let mut octaves = 0.;
        for r in c.routes {
            if !r.enabled {
                continue;
            }
            let x = match r.source {
                Source::X => c.x,
                Source::Y => c.y,
                Source::Velocity => velocity,
            }
            .clamp(0., 1.)
            .powf(r.curve.clamp(0.2, 5.));
            let (lo, hi) = r.target.range();
            let v = (r.start + (r.end - r.start) * x).clamp(lo, hi);
            match r.target {
                Target::Rate => e.rate += v,
                Target::Fragment => e.fragment += v,
                Target::Pitch => e.pitch += v,
                Target::StopTime => e.stop += v,
                Target::GateDepth => e.gate += v,
                Target::Bits => e.bits += v,
                Target::Reduction => e.reduction += v,
                Target::Filter => octaves += v,
                Target::Drive => e.drive += v,
                Target::Wander => e.wander += v,
                Target::Acceleration => e.accel += v,
                Target::Mix => e.mix += v,
            }
        }
        e.rate = e.rate.clamp(-6., 6.);
        e.fragment = e.fragment.clamp(2., 250.);
        e.pitch = e.pitch.clamp(-48., 12.);
        e.stop = e.stop.clamp(50., 4000.);
        e.gate = e.gate.clamp(0., 1.);
        e.bits = e.bits.clamp(4., 16.);
        e.reduction = e.reduction.clamp(1., 64.);
        e.filter = (e.filter * 2_f32.powf(octaves.clamp(-6., 6.))).clamp(50., 20000.);
        e.drive = e.drive.clamp(0., 1.);
        e.wander = e.wander.clamp(0., 1.);
        e.accel = e.accel.clamp(0., 1.);
        e.mix = e.mix.clamp(0., 1.);
        e
    }
    fn approach(&mut self, t: Self, k: f32) {
        macro_rules! s{($($f:ident),*)=>{$(self.$f=flush(self.$f+(t.$f-self.$f)*k);)*};}
        s!(
            rate, fragment, pitch, stop, gate, bits, reduction, filter_mix, drive, wander, accel,
            mix, output
        );
        self.filter += (t.filter - self.filter) * k;
    }
}
#[derive(Clone, Copy)]
struct Voice {
    effect: Effect,
    slice: Slice,
    window: f64,
    phase: f64,
    age: u64,
    cycles: u32,
    speed: f64,
    done: bool,
}
impl Voice {
    fn new(effect: Effect, slice: Slice, window: f64) -> Self {
        Self {
            effect,
            slice,
            window,
            phase: 0.,
            age: 0,
            cycles: 0,
            speed: 1.,
            done: false,
        }
    }
    fn next(
        &mut self,
        banks: &[Bank; 3],
        c: Config,
        e: Effective,
        sr: f32,
        bpm: f64,
        frame: u64,
    ) -> [f32; 2] {
        let wobble =
            (std::f64::consts::TAU * 1.7 * frame as f64 / sr as f64).sin() * e.wander as f64 * 3.;
        let pitch = 2_f64.powf((e.pitch as f64 + wobble) / 12.);
        let age = self.age as f64 / sr as f64;
        let mut gain = 1.;
        let mut period = self.window;
        let mut fragment = self.window;
        self.speed = pitch;
        if self.effect == Effect::Stop {
            let duration = e.stop_seconds(c, bpm);
            let p = (age / duration).clamp(0., 1.);
            self.speed = pitch * (1. - p).powf(c.stop_curve as f64);
            gain = self.speed.min(1.).sqrt() as f32;
        }
        if self.effect == Effect::Dive {
            let p = (age / (c.dive_ms as f64 * 0.001)).clamp(0., 1.);
            self.speed = pitch * 2_f64.powf(c.dive_semitones as f64 * p * p / 12.);
        }
        if self.effect == Effect::Stutter {
            period = (self.window / (1. + e.accel as f64 * self.cycles as f64 * 0.15)).max(8.);
            fragment = (e.fragment as f64 * sr as f64 * 0.001)
                .min(period * self.speed)
                .min(self.slice.len as f64)
                .max(2.);
        }
        let p = if self.effect == Effect::Stutter {
            self.phase * self.speed
        } else {
            self.phase
        };
        let reverse =
            self.effect == Effect::Reverse || (self.effect == Effect::Stutter && c.stutter_reverse);
        let active_len = if self.effect == Effect::Stutter {
            fragment
        } else {
            self.window
        };
        let pos = if reverse { active_len - 1. - p } else { p };
        let start = self.slice.len as f64 - active_len;
        let mut y = if p >= active_len {
            [0.; 2]
        } else {
            banks[self.slice.bank].read(self.slice, start + pos)
        };
        let edge = (c.edge_ms as f64 * sr as f64 * 0.001)
            .min(active_len * 0.25)
            .max(1.);
        let taper = (p / edge).min((active_len - p) / edge).clamp(0., 1.) as f32;
        gain *= 0.5 - 0.5 * (PI * taper).cos();
        for v in &mut y {
            *v *= gain;
        }
        self.phase += if self.effect == Effect::Stutter {
            1.
        } else {
            self.speed
        };
        self.age = self.age.saturating_add(1);
        if self.phase >= period {
            let count = (self.phase / period).floor() as u32;
            self.cycles = self.cycles.saturating_add(count);
            self.phase = self.phase.rem_euclid(period);
        }
        if c.repeat_count > 0
            && self.cycles >= c.repeat_count as u32
            && matches!(
                self.effect,
                Effect::Repeat | Effect::Stutter | Effect::Reverse
            )
        {
            self.done = true;
        }
        y
    }
}
#[derive(Clone, Copy, Default)]
struct Fade {
    value: f32,
    start: f32,
    target: f32,
    position: u32,
    length: u32,
}
impl Fade {
    fn set(&mut self, active: bool, length: u32) {
        let target = active as u8 as f32;
        if target != self.target {
            self.start = self.value;
            self.target = target;
            self.position = 0;
            self.length = length.max(1);
        }
    }
    fn next(&mut self) -> f32 {
        if self.position < self.length {
            self.position += 1;
            let x = self.position as f32 / self.length as f32;
            self.value = self.start + (self.target - self.start) * (0.5 - 0.5 * (PI * x).cos());
        } else {
            self.value = self.target;
        }
        self.value
    }
}
pub struct Engine {
    sr: f32,
    cfg: Config,
    banks: [Bank; 3],
    record: usize,
    clock: Clock,
    pads: [Pad; 8],
    engaged: [bool; 7],
    pending: [Option<f64>; 7],
    fire: u8,
    voice: Option<Voice>,
    old: Option<Voice>,
    primary: Effect,
    transition: f32,
    offset: [f32; 2],
    transition_start: bool,
    last_generated: [f32; 2],
    effective: Effective,
    target: Effective,
    velocity: f32,
    k: f32,
    fade: Fade,
    clear_pending: bool,
    gate: f32,
    held: [f32; 2],
    hold_count: u32,
    filter: [f32; 2],
    filter_k: f32,
    gate_attack: f32,
    gate_release: f32,
    frame: u64,
    tick: u8,
    capture_serial: u32,
    epoch: u32,
    short: bool,
    peaks: [f32; 2],
    status: Status,
}
impl Engine {
    pub fn new(sr: f32, cfg: Config) -> Self {
        let sr = finite(sr, 48000.).clamp(8000., 192000.);
        let n = (sr * MAX_SECONDS).ceil() as usize + 4;
        let e = Effective::new(cfg, 1.);
        let mut s = Self {
            sr,
            cfg,
            banks: std::array::from_fn(|_| Bank::new(n)),
            record: 0,
            clock: Clock::new(sr),
            pads: [Pad::default(); 8],
            engaged: [false; 7],
            pending: [None; 7],
            fire: 0,
            voice: None,
            old: None,
            primary: Effect::Live,
            transition: 1.,
            offset: [0.; 2],
            transition_start: false,
            last_generated: [0.; 2],
            effective: e,
            target: e,
            velocity: 1.,
            k: -(-1. / (sr * 0.008)).exp_m1(),
            fade: Fade::default(),
            clear_pending: false,
            gate: 1.,
            held: [0.; 2],
            hold_count: 0,
            filter: [0.; 2],
            filter_k: 1.,
            gate_attack: 1.,
            gate_release: 1.,
            frame: 0,
            tick: 0,
            capture_serial: 0,
            epoch: 0,
            short: false,
            peaks: [0.; 2],
            status: Status::default(),
        };
        s.coefficients();
        s
    }
    pub fn sample_rate(&self) -> f32 {
        self.sr
    }
    pub fn memory_bytes(&self) -> usize {
        std::mem::size_of::<Self>() + self.banks.iter().map(Bank::bytes).sum::<usize>()
    }
    pub fn status(&self) -> Status {
        self.status
    }
    pub fn config(&self) -> Config {
        self.cfg
    }
    pub fn reset(&mut self) {
        for b in &mut self.banks {
            b.reset();
        }
        self.record = 0;
        self.clock = Clock::new(self.sr);
        self.pads = [Pad::default(); 8];
        self.engaged = [false; 7];
        self.pending = [None; 7];
        self.fire = 0;
        self.voice = None;
        self.old = None;
        self.primary = Effect::Live;
        self.transition = 1.;
        self.transition_start = false;
        self.offset = [0.; 2];
        self.last_generated = [0.; 2];
        self.velocity = 1.;
        self.effective = Effective::new(self.cfg, 1.);
        self.target = self.effective;
        self.fade = Fade::default();
        self.clear_pending = false;
        self.gate = 1.;
        self.held = [0.; 2];
        self.hold_count = 0;
        self.filter = [0.; 2];
        self.frame = 0;
        self.tick = 0;
        self.peaks = [0.; 2];
        self.status = Status::default();
        self.epoch = self.epoch.wrapping_add(1);
        self.coefficients();
    }
    pub fn set_config(&mut self, c: Config) {
        let remap = c.midi_notes != self.cfg.midi_notes || c.midi_channel != self.cfg.midi_channel;
        if (c.bypass && !self.cfg.bypass) || remap {
            self.reset_performance(false);
        }
        let old = self.cfg;
        self.cfg = c;
        for i in 0..7 {
            if (old.latch || old.latch_mask & (1 << i) != 0)
                && !self.latching(i)
                && !self.pads[i].down()
            {
                self.engaged[i] = false;
                self.pending[i] = None;
            }
        }
        if remap {
            for p in &mut self.pads {
                p.midi = [0; 16];
                p.block_midi = false;
            }
        }
        self.target = Effective::new(c, self.velocity);
    }
    pub fn set_transport(&mut self, t: Transport) {
        let (jump, start, stop) = self.clock.set(t);
        if jump || start || stop {
            self.reset_performance(jump || start);
        }
    }
    pub fn set_gates(&mut self, gates: [bool; 8]) {
        for (i, down) in gates.iter().enumerate() {
            self.parameter(i, *down, 1.);
        }
    }
    pub fn release_gui(&mut self, pad: usize) {
        if pad < 7 {
            let before = self.pads[pad].down();
            self.pads[pad].block_host = self.pads[pad].host;
            if before && !self.pads[pad].down() && !self.latching(pad) {
                self.engaged[pad] = false;
                self.pending[pad] = None;
            }
        }
    }
    pub fn reset_performance(&mut self, clear: bool) {
        self.engaged = [false; 7];
        self.pending = [None; 7];
        self.fire = 0;
        for p in &mut self.pads[..7] {
            p.block_host = p.host;
            p.block_midi = p.midi.iter().any(|n| *n > 0);
        }
        self.fade.set(false, self.fade_samples());
        self.clear_pending |= clear;
    }
    pub fn event(&mut self, event: Event) {
        match event {
            Event::Pad {
                pad,
                down,
                velocity,
            } => self.parameter(pad as usize, down, velocity),
            Event::Reset => self.reset_performance(self.cfg.reset_clear),
            Event::AllNotesOff => {
                for i in 0..8 {
                    let before = self.pads[i].down();
                    self.pads[i].midi = [0; 16];
                    self.pads[i].block_midi = false;
                    if before && !self.pads[i].down() && i < 7 && !self.latching(i) {
                        self.engaged[i] = false;
                        self.pending[i] = None;
                    }
                }
                self.reset_performance(false);
            }
            Event::Note {
                note,
                channel,
                on,
                velocity,
            } => {
                let on = on && velocity > 0.;
                if channel > 15
                    || (self.cfg.midi_channel != 0 && self.cfg.midi_channel != channel + 1)
                {
                    return;
                }
                if let Some(i) = self.cfg.midi_notes.iter().position(|n| *n == note) {
                    let before = self.pads[i].down();
                    let count = &mut self.pads[i].midi[channel as usize];
                    if on && velocity > 0. {
                        *count = count.saturating_add(1);
                    } else {
                        *count = count.saturating_sub(1);
                    }
                    if self.pads[i].midi.iter().all(|n| *n == 0) {
                        self.pads[i].block_midi = false;
                    }
                    let after = self.pads[i].down();
                    if i == 7 {
                        if after && !before {
                            self.reset_performance(self.cfg.reset_clear);
                        }
                    } else if after && on && !self.pads[i].block_midi {
                        self.press(i, velocity);
                    } else if before && !after && !self.latching(i) {
                        self.engaged[i] = false;
                        self.pending[i] = None;
                    }
                }
            }
        }
    }
    fn parameter(&mut self, i: usize, down: bool, velocity: f32) {
        if i > 7 || self.pads[i].host == down {
            return;
        }
        let before = self.pads[i].down();
        self.pads[i].host = down;
        if !down {
            self.pads[i].block_host = false;
        }
        let after = self.pads[i].down();
        if i == 7 {
            if after && !before {
                self.reset_performance(self.cfg.reset_clear);
            }
        } else if after && !before {
            self.press(i, velocity);
        } else if before && !after && !self.latching(i) {
            self.engaged[i] = false;
            self.pending[i] = None;
        }
    }
    fn latching(&self, i: usize) -> bool {
        self.cfg.latch || self.cfg.latch_mask & (1 << i) != 0
    }
    fn press(&mut self, i: usize, velocity: f32) {
        if self.cfg.bypass || self.pads[7].down() {
            return;
        }
        self.velocity = finite(velocity, 1.).clamp(0., 1.);
        self.target = Effective::new(self.cfg, self.velocity);
        if self.latching(i) && (self.engaged[i] || self.pending[i].is_some()) {
            self.engaged[i] = false;
            self.pending[i] = None;
            return;
        }
        self.pending[i] = Some(
            self.clock
                .boundary(QUANTIZE[self.cfg.quantize.min(4) as usize].1),
        );
    }
    fn fade_samples(&self) -> u32 {
        (self.cfg.crossfade_ms * self.sr * 0.001).round().max(1.) as u32
    }
    fn ready(&self) -> bool {
        (self.cfg.capture_locked && self.voice.is_some()) || self.banks[self.record].valid >= 32
    }
    fn activate_pending(&mut self) {
        if self.cfg.bypass || self.pads[7].down() || self.clear_pending {
            return;
        }
        for i in 0..7 {
            if self.pending[i].is_some_and(|b| self.clock.beat + 1e-9 >= b) {
                let assignment = self.cfg.pad_assign[i];
                let needs_history = assignment.captured()
                    || (assignment == Effect::Chaos && self.cfg.chaos_mask & 31 != 0);
                if needs_history && !self.ready() {
                    let q = QUANTIZE[self.cfg.quantize as usize].1;
                    if q > 0. {
                        self.pending[i] = Some(((self.clock.beat + 1e-9) / q).floor() * q + q);
                    }
                    continue;
                }
                self.engaged[i] = true;
                self.pending[i] = None;
                self.fire |= 1 << i;
            }
        }
    }
    fn choose(&self) -> (Effect, u16) {
        let mut mask = 0_u16;
        for i in 0..7 {
            if self.engaged[i] {
                mask |= self.cfg.pad_assign[i].bit();
            }
        }
        let chaos = mask & Effect::Chaos.bit() != 0;
        let mut choice = Effect::Live;
        let mut rank = usize::MAX;
        for (i, e) in Effect::PRIMARY.iter().enumerate() {
            if mask & e.bit() != 0 && (self.cfg.priority[i] as usize) < rank {
                choice = *e;
                rank = self.cfg.priority[i] as usize;
            }
        }
        if chaos {
            let mut hash = self.clock.step(DIVISIONS[self.cfg.chaos_div as usize].1) as u64
                ^ self.cfg.seed as u64;
            hash = hash.wrapping_mul(0x9e3779b97f4a7c15);
            hash ^= hash >> 29;
            let mut allowed = [Effect::Live; 5];
            let mut count = 0;
            for e in Effect::PRIMARY {
                if self.cfg.chaos_mask & e.bit() != 0 {
                    allowed[count] = e;
                    count += 1;
                }
            }
            if choice == Effect::Live && count > 0 {
                choice = allowed[hash as usize % count];
            }
            mask |= self.cfg.chaos_mask & (Effect::Gate.bit() | Effect::Crush.bit());
        }
        (choice, mask)
    }
    fn requested_window(&self, e: Effect) -> f64 {
        let index = match e {
            Effect::Reverse => self.cfg.reverse_div,
            Effect::Stutter => self.cfg.stutter_div,
            _ => self.cfg.repeat_div,
        };
        let div = (index as f32 + self.effective.rate.round()).clamp(0., 6.) as usize;
        if e == Effect::Stop {
            self.effective.stop_seconds(self.cfg, self.clock.bpm) * self.sr as f64
        } else {
            DIVISIONS[div].1 * 60. / self.clock.bpm * self.sr as f64
        }
    }
    fn window(&self, e: Effect, slice: Slice) -> f64 {
        self.requested_window(e).max(8.).min(slice.len as f64)
    }
    fn begin(&mut self, effect: Effect, recapture: bool) {
        let slice = if let Some(v) = self.voice.filter(|_| !recapture && self.cfg.capture_locked) {
            v.slice
        } else {
            let bank = self.record;
            let slice = self.banks[bank].slice(
                bank,
                (self.cfg.history_seconds * self.sr) as usize,
                self.frame,
            );
            if slice.len < 32 {
                return;
            }
            self.capture_serial = self.capture_serial.wrapping_add(1);
            slice
        };
        let prior = self.voice;
        self.old = prior;
        if slice.bank == self.record {
            self.record = (0..3)
                .find(|i| *i != slice.bank && self.old.is_none_or(|v| v.slice.bank != *i))
                .unwrap();
            self.banks[self.record].reset();
        }
        let window = self.window(effect, slice);
        self.short = window + 1. < self.requested_window(effect);
        self.voice = Some(Voice::new(effect, slice, window));
        self.primary = effect;
        self.transition = 0.;
        self.transition_start = true;
    }
    fn coefficients(&mut self) {
        self.filter_k = -(-TAU * self.effective.filter.min(self.sr * 0.4) / self.sr).exp_m1();
        self.gate_attack = -(-1. / (self.sr * self.cfg.gate_attack_ms * 0.001)).exp_m1();
        self.gate_release = -(-1. / (self.sr * self.cfg.gate_release_ms * 0.001)).exp_m1();
    }
    pub fn process(&mut self, input: [f32; 2]) -> [f32; 2] {
        let x = input.map(clean);
        self.banks[self.record].write(x);
        self.frame = self.frame.wrapping_add(1);
        self.effective.approach(self.target, self.k);
        self.activate_pending();
        let (primary, mask) = self.choose();
        let retrigger =
            (0..7).any(|i| self.fire & (1 << i) != 0 && self.cfg.pad_assign[i] == primary);
        if primary.captured() && (primary != self.primary || retrigger) {
            self.begin(primary, !self.cfg.capture_locked || self.voice.is_none());
        }
        if !primary.captured() && self.primary.captured() {
            self.old = self.voice.take();
            self.primary = Effect::Live;
            self.transition = 0.;
            self.transition_start = true;
        }
        self.fire = 0;
        if self.tick == 0 {
            self.coefficients();
            if let Some(v) = self.voice {
                let window = self.window(v.effect, v.slice);
                if (window - v.window).abs() > 0.5
                    && matches!(v.effect, Effect::Repeat | Effect::Stutter | Effect::Reverse)
                {
                    self.old = Some(v);
                    self.voice = Some(Voice::new(v.effect, v.slice, window));
                    self.transition = 0.;
                    self.transition_start = true;
                }
            }
        }
        let mut generated = self
            .voice
            .as_mut()
            .map(|v| {
                v.next(
                    &self.banks,
                    self.cfg,
                    self.effective,
                    self.sr,
                    self.clock.bpm,
                    self.frame,
                )
            })
            .unwrap_or(x);
        if self.transition < 1. {
            let old = self
                .old
                .as_mut()
                .map(|v| {
                    v.next(
                        &self.banks,
                        self.cfg,
                        self.effective,
                        self.sr,
                        self.clock.bpm,
                        self.frame,
                    )
                })
                .unwrap_or(x);
            if self.transition_start {
                for (ch, value) in old.iter().enumerate() {
                    self.offset[ch] = (self.last_generated[ch] - value).clamp(-4., 4.);
                }
                self.transition_start = false;
            }
            let a = 0.5 - 0.5 * (PI * self.transition).cos();
            for ch in 0..2 {
                generated[ch] = old[ch] * (1. - a) + generated[ch] * a + self.offset[ch] * (1. - a);
            }
            self.transition = (self.transition + 1. / self.fade_samples() as f32).min(1.);
            if self.transition >= 1. {
                self.old = None;
            }
        }
        self.last_generated = generated;
        let active = (self.engaged.iter().any(|v| *v) || self.voice.is_some())
            && !self.cfg.bypass
            && !self.pads[7].down();
        // Once a finite repeat has ended, held sources must release before rearming.
        if self.voice.is_some_and(|v| v.done) {
            self.reset_performance(false);
        }
        self.fade.set(
            active && self.engaged.iter().any(|v| *v),
            self.fade_samples(),
        );
        let amount = self.fade.next();
        let e = self.effective;
        let gate_on = mask & Effect::Gate.bit() != 0;
        let crush_on =
            mask & Effect::Crush.bit() != 0 || (self.cfg.auto_damage && primary.captured());
        let step = self
            .clock
            .step(DIVISIONS[self.cfg.gate_div as usize].1)
            .rem_euclid(16) as u16;
        let gate_target = if gate_on && self.cfg.gate_pattern & (1 << step) == 0 {
            0.
        } else {
            1.
        };
        let gk = if gate_target > self.gate {
            self.gate_attack
        } else {
            self.gate_release
        };
        self.gate = clean(self.gate + (gate_target - self.gate) * gk);
        if !crush_on || self.hold_count == 0 {
            let levels = 2_f32.powf(e.bits.round() - 1.);
            self.held = generated.map(|v| {
                if crush_on {
                    (v.clamp(-1., 1.) * levels).round() / levels
                } else {
                    v
                }
            });
            self.hold_count = if crush_on {
                e.reduction.round() as u32
            } else {
                1
            };
        }
        self.hold_count = self.hold_count.saturating_sub(1);
        let mut wet = self.held;
        let drive = 1. + e.drive * 12.;
        let output_gain = 10_f32.powf(e.output / 20.);
        for (ch, value) in wet.iter_mut().enumerate() {
            let v = *value * (1. - e.gate * (1. - self.gate));
            self.filter[ch] = clean(self.filter[ch] + (v - self.filter[ch]) * self.filter_k);
            let v = v * (1. - e.filter_mix) + self.filter[ch] * e.filter_mix;
            *value = if e.drive > 1e-6 {
                (v * drive).tanh() / drive.sqrt()
            } else {
                v
            };
            *value = (*value * output_gain).clamp(-1., 1.);
        }
        let w = amount * e.mix;
        let y = if w == 0. {
            x
        } else {
            std::array::from_fn(|ch| clean(x[ch] * (1. - w) + wet[ch] * w))
        };
        if amount == 0. {
            self.voice = None;
            self.old = None;
            self.primary = Effect::Live;
            if self.clear_pending {
                for b in &mut self.banks {
                    b.reset();
                }
                self.clear_pending = false;
                self.epoch = self.epoch.wrapping_add(1);
            }
        }
        let fall = (-1. / (self.sr * 0.25)).exp();
        self.peaks[0] = clean(self.peaks[0] * fall).max(x[0].abs().max(x[1].abs()));
        self.peaks[1] = clean(self.peaks[1] * fall).max(y[0].abs().max(y[1].abs()));
        let v = self.voice;
        let audible = if primary.captured() {
            primary
        } else if mask & Effect::Gate.bit() != 0 {
            Effect::Gate
        } else if mask & Effect::Crush.bit() != 0 {
            Effect::Crush
        } else {
            Effect::Live
        };
        self.status = Status {
            effect: if amount == 0. { Effect::Live } else { audible },
            engaged: (0..7).fold(0, |a, i| a | ((self.engaged[i] as u8) << i)),
            pending: (0..7).fold(0, |a, i| a | ((self.pending[i].is_some() as u8) << i)),
            latched: (0..7).fold(0, |a, i| {
                a | (((self.engaged[i] && self.latching(i)) as u8) << i)
            }),
            history_seconds: self.banks[self.record]
                .valid
                .min((self.cfg.history_seconds * self.sr) as usize)
                as f32
                / self.sr,
            capture_seconds: v.map(|v| v.slice.len as f32 / self.sr).unwrap_or(0.),
            capture_age_seconds: v
                .map(|v| self.frame.saturating_sub(v.slice.global_end) as f32 / self.sr)
                .unwrap_or(0.),
            phase: v.map(|v| (v.phase / v.window) as f32).unwrap_or(0.),
            speed: v.map(|v| v.speed as f32).unwrap_or(1.),
            reverse: v.is_some_and(|v| {
                v.effect == Effect::Reverse
                    || (v.effect == Effect::Stutter && self.cfg.stutter_reverse)
            }),
            repeat_seconds: v.map(|v| v.window as f32 / self.sr).unwrap_or(0.),
            short_history: self.short,
            host_sync: self.clock.host,
            bpm: self.clock.bpm as f32,
            beat: self.clock.beat,
            input_peak: self.peaks[0],
            output_peak: self.peaks[1],
            capture_serial: self.capture_serial,
            epoch: self.epoch,
        };
        self.tick = (self.tick + 1) % 16;
        self.clock.tick();
        y
    }
}
