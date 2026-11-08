// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    BANDS, Band, Config, MAX_BANDS,
    crossover::{Crossover, effective_crossovers},
    db,
    dynamics::Dynamics,
    flush, gain,
    oversample::{Delay, Saturator},
    sanitize,
};
#[derive(Clone, Copy, Debug, Default)]
pub struct Meter {
    pub input: f32,
    pub output: f32,
    pub up: [f32; BANDS],
    pub down: [f32; BANDS],
    pub band_input: [f32; BANDS],
    pub band_output: [f32; BANDS],
    pub compensation_db: f32,
    pub crossovers: [f32; 2],
    pub safety_active: bool,
}
#[derive(Clone, Copy)]
struct Effective {
    amount: f32,
    time: f32,
    tone: [f32; BANDS],
    drive: f32,
    juice: f32,
    punch: f32,
    mix: f32,
    output: f32,
    floor: f32,
    bypass: f32,
    safety: f32,
    solo_active: f32,
    bands: [Band; BANDS],
    enabled: [f32; BANDS],
    solo: [f32; BANDS],
    sat: [f32; BANDS],
}
impl Effective {
    fn new(c: Config) -> Self {
        let any = c.bands.iter().any(|b| b.solo);
        Self {
            amount: c.amount,
            time: c.time,
            tone: c.tone_db,
            drive: c.drive,
            juice: c.juice,
            punch: c.punch,
            mix: c.mix,
            output: c.output_db,
            floor: c.floor_db,
            bypass: c.bypass as u8 as f32,
            safety: c.safety as u8 as f32,
            solo_active: any as u8 as f32,
            bands: c.bands,
            enabled: c.bands.map(|b| (!b.bypass) as u8 as f32),
            solo: c.bands.map(|b| (!any || b.solo) as u8 as f32),
            sat: c.bands.map(|b| b.sat_enabled as u8 as f32),
        }
    }
    fn approach(&mut self, t: Self, k: f32) {
        macro_rules! s{($($f:ident),*)=>{$(self.$f=flush(self.$f+(t.$f-self.$f)*k);)*};}
        s!(
            amount,
            time,
            drive,
            juice,
            punch,
            mix,
            output,
            floor,
            bypass,
            safety,
            solo_active
        );
        for i in 0..BANDS {
            self.tone[i] = flush(self.tone[i] + (t.tone[i] - self.tone[i]) * k);
            self.enabled[i] = flush(self.enabled[i] + (t.enabled[i] - self.enabled[i]) * k);
            self.solo[i] = flush(self.solo[i] + (t.solo[i] - self.solo[i]) * k);
            self.sat[i] = flush(self.sat[i] + (t.sat[i] - self.sat[i]) * k);
            let a = &mut self.bands[i];
            let b = t.bands[i];
            macro_rules! f{($($f:ident),*)=>{$(a.$f=flush(a.$f+(b.$f-a.$f)*k);)*};}
            f!(
                up_threshold,
                up_ratio,
                down_threshold,
                down_ratio,
                attack_ms,
                release_ms,
                makeup_db,
                knee_db,
                up_limit_db,
                drive,
                link,
                mix
            );
        }
    }
}
/// Prepared fixed-storage engine. Configuration validation belongs outside audio processing.
pub struct Engine {
    sr: f32,
    cfg: Config,
    split: Crossover,
    bands: [Dynamics; MAX_BANDS],
    sat: [Saturator; MAX_BANDS],
    band_dry: [Delay; MAX_BANDS],
    processed_dry: [Delay; MAX_BANDS],
    raw: Delay,
    aligned: Delay,
    effective: Effective,
    target: Effective,
    k: f32,
    auto_db: f32,
    auto_applied_db: f32,
    dry_energy: f32,
    wet_energy: f32,
    age: u64,
    peak: [f32; 2],
    band_peak: [[f32; 2]; BANDS],
    meter: Meter,
    safety_hold: u32,
}
impl Engine {
    pub fn new(sr: f32, c: Config) -> Self {
        let sr = if sr.is_finite() {
            sr.clamp(8000., 192000.)
        } else {
            48000.
        };
        let e = Effective::new(c);
        Self {
            sr,
            cfg: c,
            split: Crossover::new(sr, &effective_crossovers(c.crossovers, sr)),
            bands: [Dynamics::default(); MAX_BANDS],
            sat: [Saturator::default(); MAX_BANDS],
            band_dry: [Delay::default(); MAX_BANDS],
            processed_dry: [Delay::default(); MAX_BANDS],
            raw: Delay::default(),
            aligned: Delay::default(),
            effective: e,
            target: e,
            k: 1. - (-1. / (sr * 0.008)).exp(),
            auto_db: 0.,
            auto_applied_db: 0.,
            dry_energy: 0.,
            wet_energy: 0.,
            age: 0,
            peak: [0.; 2],
            band_peak: [[0.; 2]; BANDS],
            meter: Meter::default(),
            safety_hold: 0,
        }
    }
    pub fn set_config(&mut self, c: Config) {
        if c.crossovers != self.cfg.crossovers {
            self.split
                .set_frequencies(&effective_crossovers(c.crossovers, self.sr));
        }
        self.cfg = c;
        self.target = Effective::new(c);
    }
    pub fn sample_rate(&self) -> f32 {
        self.sr
    }
    pub fn config(&self) -> Config {
        self.cfg
    }
    pub fn meter(&self) -> Meter {
        self.meter
    }
    pub fn memory_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
    pub fn reset(&mut self) {
        *self = Self::new(self.sr, self.cfg);
    }
    pub fn process(&mut self, input: [f32; 2]) -> [f32; 2] {
        let x = input.map(sanitize);
        self.effective.approach(self.target, self.k);
        let e = self.effective;
        let raw = self.raw.process(x);
        let split = self.split.process(x);
        let aligned = self.aligned.process([
            split[..BANDS].iter().map(|b| b[0]).sum(),
            split[..BANDS].iter().map(|b| b[1]).sum(),
        ]);
        let mut wet = [0.; 2];
        let fall = (-1. / (self.sr * 0.25)).exp();
        for (i, band) in split.iter().enumerate().take(BANDS) {
            let b = e.bands[i];
            let movement = self.bands[i].process(
                *band, b, e.amount, e.juice, e.time, e.punch, e.floor, self.sr,
            );
            let processed = std::array::from_fn(|ch| band[ch] * gain(movement[ch] + b.makeup_db));
            let drive = (b.drive + e.drive).min(1.);
            let sat = self.sat[i].process(processed, drive);
            let delayed = self.band_dry[i].process(*band);
            let processed_delay = self.processed_dry[i].process(processed);
            let sat_mix = (drive * 12.).clamp(0., 1.) * e.sat[i];
            let mut band_out = [0.; 2];
            for ch in 0..2 {
                let color = processed_delay[ch] * (1. - sat_mix) + sat[ch] * sat_mix;
                let effect = delayed[ch] * (1. - b.mix) + color * b.mix;
                band_out[ch] = (delayed[ch] * (1. - e.enabled[i])
                    + effect * gain(e.tone[i]) * e.enabled[i])
                    * e.solo[i];
                wet[ch] += band_out[ch];
            }
            self.band_peak[i][0] =
                flush(self.band_peak[i][0] * fall).max(band[0].abs().max(band[1].abs()));
            self.band_peak[i][1] =
                flush(self.band_peak[i][1] * fall).max(band_out[0].abs().max(band_out[1].abs()));
            self.meter.up[i] = self.bands[i].up[0].max(self.bands[i].up[1]) * e.enabled[i] * b.mix;
            self.meter.down[i] =
                self.bands[i].down[0].min(self.bands[i].down[1]) * e.enabled[i] * b.mix;
            self.meter.band_input[i] = db(self.band_peak[i][0]);
            self.meter.band_output[i] = db(self.band_peak[i][1]);
        }
        let energy_k = 1. - (-1. / self.sr).exp();
        self.dry_energy = flush(
            self.dry_energy
                + ((aligned[0] * aligned[0] + aligned[1] * aligned[1]) * 0.5 - self.dry_energy)
                    * energy_k,
        );
        self.wet_energy = flush(
            self.wet_energy
                + ((wet[0] * wet[0] + wet[1] * wet[1]) * 0.5 - self.wet_energy) * energy_k,
        );
        self.age = self.age.saturating_add(1);
        if self.cfg.auto_gain
            && !self.cfg.bands.iter().any(|b| b.solo)
            && self.age > self.sr as u64 / 3
            && self.dry_energy > 1e-8
            && self.wet_energy > 1e-10
        {
            let target = (10. * (self.dry_energy / self.wet_energy).log10()).clamp(-12., 6.);
            self.auto_db += (target - self.auto_db).clamp(-2. / self.sr, 2. / self.sr);
        }
        let compensation_target = if self.cfg.auto_gain { self.auto_db } else { 0. };
        self.auto_applied_db +=
            (compensation_target - self.auto_applied_db).clamp(-2. / self.sr, 2. / self.sr);
        let compensation = gain(self.auto_applied_db);
        let output = gain(e.output);
        let mix = e.mix + (1. - e.mix) * e.solo_active;
        let mut y = [0.; 2];
        let mut limited = false;
        for ch in 0..2 {
            let v = (aligned[ch] * (1. - mix) + wet[ch] * compensation * mix) * output;
            let a = v.abs();
            let safe = if a > 0.98 {
                limited = true;
                v.signum() * (0.98 + 0.02 * (a - 0.98) / (0.02 + a - 0.98))
            } else {
                v
            };
            let v = v * (1. - e.safety) + safe * e.safety;
            y[ch] = sanitize(v * (1. - e.bypass) + raw[ch] * e.bypass);
        }
        self.peak[0] = flush(self.peak[0] * fall).max(x[0].abs().max(x[1].abs()));
        self.peak[1] = flush(self.peak[1] * fall).max(y[0].abs().max(y[1].abs()));
        self.meter.input = db(self.peak[0]);
        self.meter.output = db(self.peak[1]);
        self.meter.compensation_db = self.auto_applied_db;
        self.safety_hold = self.safety_hold.saturating_sub(1);
        if limited && self.cfg.safety {
            self.safety_hold = (self.sr * 0.25) as u32;
        }
        self.meter.safety_active = self.safety_hold > 0;
        self.meter.crossovers = effective_crossovers(self.cfg.crossovers, self.sr);
        y
    }
}
