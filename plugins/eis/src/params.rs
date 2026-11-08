// SPDX-License-Identifier: GPL-3.0-or-later
use crate::fields::TextField;
use eis_core::{Band, Config, Preset, factory};
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};
#[derive(Clone, Copy)]
pub struct BandRef<'a> {
    pub up_threshold: &'a FloatParam,
    pub up_ratio: &'a FloatParam,
    pub down_threshold: &'a FloatParam,
    pub down_ratio: &'a FloatParam,
    pub attack_ms: &'a FloatParam,
    pub release_ms: &'a FloatParam,
    pub makeup_db: &'a FloatParam,
    pub knee_db: &'a FloatParam,
    pub up_limit_db: &'a FloatParam,
    pub drive: &'a FloatParam,
    pub link: &'a FloatParam,
    pub mix: &'a FloatParam,
    pub sat_enabled: &'a BoolParam,
    pub bypass: &'a BoolParam,
    pub solo: &'a BoolParam,
}
impl BandRef<'_> {
    pub fn value(&self) -> Band {
        Band {
            up_threshold: self.up_threshold.value(),
            up_ratio: self.up_ratio.value(),
            down_threshold: self.down_threshold.value(),
            down_ratio: self.down_ratio.value(),
            attack_ms: self.attack_ms.value(),
            release_ms: self.release_ms.value(),
            makeup_db: self.makeup_db.value(),
            knee_db: self.knee_db.value(),
            up_limit_db: self.up_limit_db.value(),
            drive: self.drive.value(),
            link: self.link.value(),
            mix: self.mix.value(),
            sat_enabled: self.sat_enabled.value(),
            bypass: self.bypass.value(),
            solo: self.solo.value(),
        }
    }
}
#[derive(Params)]
pub struct EisParams {
    #[persist = "editor"]
    pub editor: Arc<EguiState>,
    #[persist = "preset-name"]
    pub name: TextField,
    #[persist = "preset-notes"]
    pub notes: TextField,
    #[persist = "schema"]
    pub schema: Arc<AtomicU32>,
    #[id = "preset"]
    pub preset: IntParam,
    #[id = "revision"]
    pub revision: BoolParam,
    #[persist = "butcher"]
    pub butcher: Arc<AtomicBool>,
    #[id = "amount"]
    pub amount: FloatParam,
    #[id = "time"]
    pub time: FloatParam,
    #[id = "low"]
    pub low: FloatParam,
    #[id = "mid"]
    pub mid: FloatParam,
    #[id = "high"]
    pub high: FloatParam,
    #[id = "drive"]
    pub drive: FloatParam,
    #[id = "juice"]
    pub juice: FloatParam,
    #[id = "punch"]
    pub punch: FloatParam,
    #[id = "mix"]
    pub mix: FloatParam,
    #[id = "output-db"]
    pub output_db: FloatParam,
    #[id = "floor-db"]
    pub floor_db: FloatParam,
    #[id = "cross-low"]
    pub cross_low: FloatParam,
    #[id = "cross-high"]
    pub cross_high: FloatParam,
    #[id = "auto-gain"]
    pub auto_gain: BoolParam,
    #[id = "safety"]
    pub safety: BoolParam,
    #[id = "bypass"]
    pub bypass: BoolParam,
    // Flat stable IDs avoid allocations from nested-array persistence helpers.
    #[id = "b1-up-threshold"]
    b1_up_threshold: FloatParam,
    #[id = "b1-up-ratio"]
    b1_up_ratio: FloatParam,
    #[id = "b1-down-threshold"]
    b1_down_threshold: FloatParam,
    #[id = "b1-down-ratio"]
    b1_down_ratio: FloatParam,
    #[id = "b1-attack-ms"]
    b1_attack_ms: FloatParam,
    #[id = "b1-release-ms"]
    b1_release_ms: FloatParam,
    #[id = "b1-makeup-db"]
    b1_makeup_db: FloatParam,
    #[id = "b1-knee-db"]
    b1_knee_db: FloatParam,
    #[id = "b1-up-limit-db"]
    b1_up_limit_db: FloatParam,
    #[id = "b1-drive"]
    b1_drive: FloatParam,
    #[id = "b1-link"]
    b1_link: FloatParam,
    #[id = "b1-mix"]
    b1_mix: FloatParam,
    #[id = "b1-sat-enabled"]
    b1_sat_enabled: BoolParam,
    #[id = "b1-bypass"]
    b1_bypass: BoolParam,
    #[id = "b1-solo"]
    b1_solo: BoolParam,
    #[id = "b2-up-threshold"]
    b2_up_threshold: FloatParam,
    #[id = "b2-up-ratio"]
    b2_up_ratio: FloatParam,
    #[id = "b2-down-threshold"]
    b2_down_threshold: FloatParam,
    #[id = "b2-down-ratio"]
    b2_down_ratio: FloatParam,
    #[id = "b2-attack-ms"]
    b2_attack_ms: FloatParam,
    #[id = "b2-release-ms"]
    b2_release_ms: FloatParam,
    #[id = "b2-makeup-db"]
    b2_makeup_db: FloatParam,
    #[id = "b2-knee-db"]
    b2_knee_db: FloatParam,
    #[id = "b2-up-limit-db"]
    b2_up_limit_db: FloatParam,
    #[id = "b2-drive"]
    b2_drive: FloatParam,
    #[id = "b2-link"]
    b2_link: FloatParam,
    #[id = "b2-mix"]
    b2_mix: FloatParam,
    #[id = "b2-sat-enabled"]
    b2_sat_enabled: BoolParam,
    #[id = "b2-bypass"]
    b2_bypass: BoolParam,
    #[id = "b2-solo"]
    b2_solo: BoolParam,
    #[id = "b3-up-threshold"]
    b3_up_threshold: FloatParam,
    #[id = "b3-up-ratio"]
    b3_up_ratio: FloatParam,
    #[id = "b3-down-threshold"]
    b3_down_threshold: FloatParam,
    #[id = "b3-down-ratio"]
    b3_down_ratio: FloatParam,
    #[id = "b3-attack-ms"]
    b3_attack_ms: FloatParam,
    #[id = "b3-release-ms"]
    b3_release_ms: FloatParam,
    #[id = "b3-makeup-db"]
    b3_makeup_db: FloatParam,
    #[id = "b3-knee-db"]
    b3_knee_db: FloatParam,
    #[id = "b3-up-limit-db"]
    b3_up_limit_db: FloatParam,
    #[id = "b3-drive"]
    b3_drive: FloatParam,
    #[id = "b3-link"]
    b3_link: FloatParam,
    #[id = "b3-mix"]
    b3_mix: FloatParam,
    #[id = "b3-sat-enabled"]
    b3_sat_enabled: BoolParam,
    #[id = "b3-bypass"]
    b3_bypass: BoolParam,
    #[id = "b3-solo"]
    b3_solo: BoolParam,
}
fn float(name: &str, value: f32, min: f32, max: f32, unit: &'static str) -> FloatParam {
    FloatParam::new(name, value, FloatRange::Linear { min, max })
        .with_value_to_string(Arc::new(move |v| match unit {
            "db" => format!("{v:.1} dB"),
            "ms" => format!("{v:.1} ms"),
            "hz" => format!("{v:.0} Hz"),
            "ratio" => format!("{v:.1}:1"),
            _ => format!("{:.0}%", v * 100.),
        }))
        .with_string_to_value(Arc::new(move |text| {
            let text = text.trim();
            let number = text
                .trim_end_matches("dB")
                .trim_end_matches("ms")
                .trim_end_matches("Hz")
                .trim_end_matches(":1")
                .trim_end_matches('%')
                .trim()
                .parse::<f32>()
                .ok()?;
            Some(if unit == "percent" {
                number / 100.
            } else {
                number
            })
        }))
}
impl Default for EisParams {
    fn default() -> Self {
        let p = factory(0);
        let c = p.config;
        Self {
            editor: EguiState::from_size(1020, 800),
            name: TextField::new(&p.name),
            notes: TextField::new(&p.description),
            schema: Arc::new(AtomicU32::new(1)),
            preset: IntParam::new("Preset identity", 0, IntRange::Linear { min: 0, max: 8 })
                .hide()
                .non_automatable(),
            revision: BoolParam::new("Preset metadata revision", false)
                .hide()
                .non_automatable(),
            butcher: Arc::new(AtomicBool::new(false)),
            amount: float("TENDERIZE", c.amount, 0.0, 1.5, "percent"),
            time: float("TIME", c.time, 0.1, 4.0, "percent"),
            low: float("LOW / FAT", c.tone_db[0], -12.0, 12.0, "db"),
            mid: float("MID", c.tone_db[1], -12.0, 12.0, "db"),
            high: float("HIGH", c.tone_db[2], -12.0, 12.0, "db"),
            drive: float("SIZZLE / DRIVE", c.drive, 0.0, 1.0, "percent"),
            juice: float("JUICE", c.juice, 0.0, 1.5, "percent"),
            punch: float("PUNCH", c.punch, 0.0, 1.0, "percent"),
            mix: float("MIX", c.mix, 0.0, 1.0, "percent"),
            output_db: float("OUTPUT", c.output_db, -24.0, 12.0, "db"),
            floor_db: float("Upward noise floor", c.floor_db, -110.0, -50.0, "db"),
            cross_low: float("Low crossover", c.crossovers[0], 40.0, 1000.0, "hz"),
            cross_high: float("High crossover", c.crossovers[1], 600.0, 16000.0, "hz"),
            auto_gain: BoolParam::new("Level match", c.auto_gain),
            safety: BoolParam::new("Safety", c.safety),
            bypass: BoolParam::new("Bypass", c.bypass).make_bypass(),
            b1_up_threshold: float(
                "Low Up threshold",
                c.bands[0].up_threshold,
                -72.0,
                -6.0,
                "db",
            ),
            b1_up_ratio: float("Low Up ratio", c.bands[0].up_ratio, 1.0, 12.0, "ratio"),
            b1_down_threshold: float(
                "Low Down threshold",
                c.bands[0].down_threshold,
                -48.0,
                0.0,
                "db",
            ),
            b1_down_ratio: float("Low Down ratio", c.bands[0].down_ratio, 1.0, 40.0, "ratio"),
            b1_attack_ms: float("Low Attack", c.bands[0].attack_ms, 0.1, 150.0, "ms"),
            b1_release_ms: float("Low Release", c.bands[0].release_ms, 10.0, 1000.0, "ms"),
            b1_makeup_db: float("Low Makeup", c.bands[0].makeup_db, -12.0, 18.0, "db"),
            b1_knee_db: float("Low Knee", c.bands[0].knee_db, 0.0, 18.0, "db"),
            b1_up_limit_db: float(
                "Low Max upward gain",
                c.bands[0].up_limit_db,
                0.0,
                24.0,
                "db",
            ),
            b1_drive: float("Low Band drive", c.bands[0].drive, 0.0, 1.0, "percent"),
            b1_link: float("Low Stereo link", c.bands[0].link, 0.0, 1.0, "percent"),
            b1_mix: float("Low Band mix", c.bands[0].mix, 0.0, 1.0, "percent"),
            b1_sat_enabled: BoolParam::new("Low Saturation", c.bands[0].sat_enabled),
            b1_bypass: BoolParam::new("Low Band bypass", c.bands[0].bypass),
            b1_solo: BoolParam::new("Low Solo", c.bands[0].solo),
            b2_up_threshold: float(
                "Mid Up threshold",
                c.bands[1].up_threshold,
                -72.0,
                -6.0,
                "db",
            ),
            b2_up_ratio: float("Mid Up ratio", c.bands[1].up_ratio, 1.0, 12.0, "ratio"),
            b2_down_threshold: float(
                "Mid Down threshold",
                c.bands[1].down_threshold,
                -48.0,
                0.0,
                "db",
            ),
            b2_down_ratio: float("Mid Down ratio", c.bands[1].down_ratio, 1.0, 40.0, "ratio"),
            b2_attack_ms: float("Mid Attack", c.bands[1].attack_ms, 0.1, 150.0, "ms"),
            b2_release_ms: float("Mid Release", c.bands[1].release_ms, 10.0, 1000.0, "ms"),
            b2_makeup_db: float("Mid Makeup", c.bands[1].makeup_db, -12.0, 18.0, "db"),
            b2_knee_db: float("Mid Knee", c.bands[1].knee_db, 0.0, 18.0, "db"),
            b2_up_limit_db: float(
                "Mid Max upward gain",
                c.bands[1].up_limit_db,
                0.0,
                24.0,
                "db",
            ),
            b2_drive: float("Mid Band drive", c.bands[1].drive, 0.0, 1.0, "percent"),
            b2_link: float("Mid Stereo link", c.bands[1].link, 0.0, 1.0, "percent"),
            b2_mix: float("Mid Band mix", c.bands[1].mix, 0.0, 1.0, "percent"),
            b2_sat_enabled: BoolParam::new("Mid Saturation", c.bands[1].sat_enabled),
            b2_bypass: BoolParam::new("Mid Band bypass", c.bands[1].bypass),
            b2_solo: BoolParam::new("Mid Solo", c.bands[1].solo),
            b3_up_threshold: float(
                "High Up threshold",
                c.bands[2].up_threshold,
                -72.0,
                -6.0,
                "db",
            ),
            b3_up_ratio: float("High Up ratio", c.bands[2].up_ratio, 1.0, 12.0, "ratio"),
            b3_down_threshold: float(
                "High Down threshold",
                c.bands[2].down_threshold,
                -48.0,
                0.0,
                "db",
            ),
            b3_down_ratio: float("High Down ratio", c.bands[2].down_ratio, 1.0, 40.0, "ratio"),
            b3_attack_ms: float("High Attack", c.bands[2].attack_ms, 0.1, 150.0, "ms"),
            b3_release_ms: float("High Release", c.bands[2].release_ms, 10.0, 1000.0, "ms"),
            b3_makeup_db: float("High Makeup", c.bands[2].makeup_db, -12.0, 18.0, "db"),
            b3_knee_db: float("High Knee", c.bands[2].knee_db, 0.0, 18.0, "db"),
            b3_up_limit_db: float(
                "High Max upward gain",
                c.bands[2].up_limit_db,
                0.0,
                24.0,
                "db",
            ),
            b3_drive: float("High Band drive", c.bands[2].drive, 0.0, 1.0, "percent"),
            b3_link: float("High Stereo link", c.bands[2].link, 0.0, 1.0, "percent"),
            b3_mix: float("High Band mix", c.bands[2].mix, 0.0, 1.0, "percent"),
            b3_sat_enabled: BoolParam::new("High Saturation", c.bands[2].sat_enabled),
            b3_bypass: BoolParam::new("High Band bypass", c.bands[2].bypass),
            b3_solo: BoolParam::new("High Solo", c.bands[2].solo),
        }
    }
}
impl EisParams {
    pub fn bands(&self) -> [BandRef<'_>; 3] {
        [
            BandRef {
                up_threshold: &self.b1_up_threshold,
                up_ratio: &self.b1_up_ratio,
                down_threshold: &self.b1_down_threshold,
                down_ratio: &self.b1_down_ratio,
                attack_ms: &self.b1_attack_ms,
                release_ms: &self.b1_release_ms,
                makeup_db: &self.b1_makeup_db,
                knee_db: &self.b1_knee_db,
                up_limit_db: &self.b1_up_limit_db,
                drive: &self.b1_drive,
                link: &self.b1_link,
                mix: &self.b1_mix,
                sat_enabled: &self.b1_sat_enabled,
                bypass: &self.b1_bypass,
                solo: &self.b1_solo,
            },
            BandRef {
                up_threshold: &self.b2_up_threshold,
                up_ratio: &self.b2_up_ratio,
                down_threshold: &self.b2_down_threshold,
                down_ratio: &self.b2_down_ratio,
                attack_ms: &self.b2_attack_ms,
                release_ms: &self.b2_release_ms,
                makeup_db: &self.b2_makeup_db,
                knee_db: &self.b2_knee_db,
                up_limit_db: &self.b2_up_limit_db,
                drive: &self.b2_drive,
                link: &self.b2_link,
                mix: &self.b2_mix,
                sat_enabled: &self.b2_sat_enabled,
                bypass: &self.b2_bypass,
                solo: &self.b2_solo,
            },
            BandRef {
                up_threshold: &self.b3_up_threshold,
                up_ratio: &self.b3_up_ratio,
                down_threshold: &self.b3_down_threshold,
                down_ratio: &self.b3_down_ratio,
                attack_ms: &self.b3_attack_ms,
                release_ms: &self.b3_release_ms,
                makeup_db: &self.b3_makeup_db,
                knee_db: &self.b3_knee_db,
                up_limit_db: &self.b3_up_limit_db,
                drive: &self.b3_drive,
                link: &self.b3_link,
                mix: &self.b3_mix,
                sat_enabled: &self.b3_sat_enabled,
                bypass: &self.b3_bypass,
                solo: &self.b3_solo,
            },
        ]
    }
    pub fn config(&self) -> Config {
        Config {
            bands: self.bands().map(|b| b.value()),
            crossovers: [self.cross_low.value(), self.cross_high.value()],
            tone_db: [self.low.value(), self.mid.value(), self.high.value()],
            amount: self.amount.value(),
            time: self.time.value(),
            drive: self.drive.value(),
            juice: self.juice.value(),
            punch: self.punch.value(),
            mix: self.mix.value(),
            output_db: self.output_db.value(),
            floor_db: self.floor_db.value(),
            auto_gain: self.auto_gain.value(),
            safety: self.safety.value(),
            bypass: self.bypass.value(),
        }
    }
    pub fn document(&self) -> Preset {
        Preset {
            version: 1,
            name: self.name.get(),
            description: self.notes.get(),
            config: self.config(),
        }
    }
    pub fn apply(&self, p: &Preset, index: i32, s: &ParamSetter) {
        let c = p.config;
        self.name.set_text(&p.name);
        self.notes.set_text(&p.description);
        s.begin_set_parameter(&self.amount);
        s.set_parameter(&self.amount, c.amount);
        s.end_set_parameter(&self.amount);
        s.begin_set_parameter(&self.time);
        s.set_parameter(&self.time, c.time);
        s.end_set_parameter(&self.time);
        s.begin_set_parameter(&self.low);
        s.set_parameter(&self.low, c.tone_db[0]);
        s.end_set_parameter(&self.low);
        s.begin_set_parameter(&self.mid);
        s.set_parameter(&self.mid, c.tone_db[1]);
        s.end_set_parameter(&self.mid);
        s.begin_set_parameter(&self.high);
        s.set_parameter(&self.high, c.tone_db[2]);
        s.end_set_parameter(&self.high);
        s.begin_set_parameter(&self.drive);
        s.set_parameter(&self.drive, c.drive);
        s.end_set_parameter(&self.drive);
        s.begin_set_parameter(&self.juice);
        s.set_parameter(&self.juice, c.juice);
        s.end_set_parameter(&self.juice);
        s.begin_set_parameter(&self.punch);
        s.set_parameter(&self.punch, c.punch);
        s.end_set_parameter(&self.punch);
        s.begin_set_parameter(&self.mix);
        s.set_parameter(&self.mix, c.mix);
        s.end_set_parameter(&self.mix);
        s.begin_set_parameter(&self.output_db);
        s.set_parameter(&self.output_db, c.output_db);
        s.end_set_parameter(&self.output_db);
        s.begin_set_parameter(&self.floor_db);
        s.set_parameter(&self.floor_db, c.floor_db);
        s.end_set_parameter(&self.floor_db);
        s.begin_set_parameter(&self.cross_low);
        s.set_parameter(&self.cross_low, c.crossovers[0]);
        s.end_set_parameter(&self.cross_low);
        s.begin_set_parameter(&self.cross_high);
        s.set_parameter(&self.cross_high, c.crossovers[1]);
        s.end_set_parameter(&self.cross_high);
        s.begin_set_parameter(&self.auto_gain);
        s.set_parameter(&self.auto_gain, c.auto_gain);
        s.end_set_parameter(&self.auto_gain);
        s.begin_set_parameter(&self.safety);
        s.set_parameter(&self.safety, c.safety);
        s.end_set_parameter(&self.safety);
        s.begin_set_parameter(&self.bypass);
        s.set_parameter(&self.bypass, c.bypass);
        s.end_set_parameter(&self.bypass);
        for (i, r) in self.bands().iter().enumerate() {
            let b = c.bands[i];
            s.begin_set_parameter(r.up_threshold);
            s.set_parameter(r.up_threshold, b.up_threshold);
            s.end_set_parameter(r.up_threshold);
            s.begin_set_parameter(r.up_ratio);
            s.set_parameter(r.up_ratio, b.up_ratio);
            s.end_set_parameter(r.up_ratio);
            s.begin_set_parameter(r.down_threshold);
            s.set_parameter(r.down_threshold, b.down_threshold);
            s.end_set_parameter(r.down_threshold);
            s.begin_set_parameter(r.down_ratio);
            s.set_parameter(r.down_ratio, b.down_ratio);
            s.end_set_parameter(r.down_ratio);
            s.begin_set_parameter(r.attack_ms);
            s.set_parameter(r.attack_ms, b.attack_ms);
            s.end_set_parameter(r.attack_ms);
            s.begin_set_parameter(r.release_ms);
            s.set_parameter(r.release_ms, b.release_ms);
            s.end_set_parameter(r.release_ms);
            s.begin_set_parameter(r.makeup_db);
            s.set_parameter(r.makeup_db, b.makeup_db);
            s.end_set_parameter(r.makeup_db);
            s.begin_set_parameter(r.knee_db);
            s.set_parameter(r.knee_db, b.knee_db);
            s.end_set_parameter(r.knee_db);
            s.begin_set_parameter(r.up_limit_db);
            s.set_parameter(r.up_limit_db, b.up_limit_db);
            s.end_set_parameter(r.up_limit_db);
            s.begin_set_parameter(r.drive);
            s.set_parameter(r.drive, b.drive);
            s.end_set_parameter(r.drive);
            s.begin_set_parameter(r.link);
            s.set_parameter(r.link, b.link);
            s.end_set_parameter(r.link);
            s.begin_set_parameter(r.mix);
            s.set_parameter(r.mix, b.mix);
            s.end_set_parameter(r.mix);
            s.begin_set_parameter(r.sat_enabled);
            s.set_parameter(r.sat_enabled, b.sat_enabled);
            s.end_set_parameter(r.sat_enabled);
            s.begin_set_parameter(r.bypass);
            s.set_parameter(r.bypass, b.bypass);
            s.end_set_parameter(r.bypass);
            s.begin_set_parameter(r.solo);
            s.set_parameter(r.solo, b.solo);
            s.end_set_parameter(r.solo);
        }
        s.begin_set_parameter(&self.preset);
        s.set_parameter(&self.preset, index);
        s.end_set_parameter(&self.preset);
        self.schema.store(1, Ordering::Relaxed);
        s.begin_set_parameter(&self.revision);
        s.set_parameter(&self.revision, !self.revision.value());
        s.end_set_parameter(&self.revision);
    }
}
