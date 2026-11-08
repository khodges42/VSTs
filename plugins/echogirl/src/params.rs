// SPDX-License-Identifier: GPL-3.0-or-later
use crate::fields::{MidiState, TextField};
use echogirl_core::{
    Config, Preset,
    preset::{Route, Source, Target},
};
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};
#[derive(Clone, Copy)]
struct RouteRef<'a> {
    enabled: &'a BoolParam,
    source: &'a IntParam,
    target: &'a IntParam,
    amount: &'a FloatParam,
    curve: &'a FloatParam,
}
impl RouteRef<'_> {
    fn value(&self) -> Route {
        Route {
            enabled: self.enabled.value(),
            source: if self.source.value() == 0 {
                Source::X
            } else {
                Source::Y
            },
            target: Target::ALL[self.target.value() as usize],
            amount: self.amount.value(),
            curve: self.curve.value(),
        }
    }
}
#[derive(Params)]
pub struct EchoParams {
    #[persist = "editor"]
    pub editor: Arc<EguiState>,
    #[persist = "preset-name"]
    pub name: TextField,
    #[persist = "preset-notes"]
    pub notes: TextField,
    #[persist = "midi"]
    pub midi: MidiState,
    #[persist = "schema"]
    pub schema: Arc<AtomicU32>,
    #[id = "preset"]
    pub preset: IntParam,
    #[id = "preset-revision"]
    pub revision: BoolParam,
    #[id = "time-ms"]
    pub time_ms: FloatParam,
    #[id = "right-ms"]
    pub right_ms: FloatParam,
    #[id = "feedback"]
    pub feedback: FloatParam,
    #[id = "filter"]
    pub filter: FloatParam,
    #[id = "drive"]
    pub drive: FloatParam,
    #[id = "mix"]
    pub mix: FloatParam,
    #[id = "input-db"]
    pub input_db: FloatParam,
    #[id = "output-db"]
    pub output_db: FloatParam,
    #[id = "low-cut"]
    pub low_cut: FloatParam,
    #[id = "high-cut"]
    pub high_cut: FloatParam,
    #[id = "resonance"]
    pub resonance: FloatParam,
    #[id = "wow"]
    pub wow: FloatParam,
    #[id = "mod-rate"]
    pub mod_rate: FloatParam,
    #[id = "mod-beats"]
    pub mod_beats: FloatParam,
    #[id = "spring"]
    pub spring: FloatParam,
    #[id = "x"]
    pub x: FloatParam,
    #[id = "y"]
    pub y: FloatParam,
    #[id = "dub-feedback"]
    pub dub_feedback: FloatParam,
    #[id = "dub-drive"]
    pub dub_drive: FloatParam,
    #[id = "dub-filter"]
    pub dub_filter: FloatParam,
    #[id = "character"]
    pub character: IntParam,
    #[id = "left-div"]
    pub left_div: IntParam,
    #[id = "right-div"]
    pub right_div: IntParam,
    #[id = "sync"]
    pub sync: BoolParam,
    #[id = "link"]
    pub link: BoolParam,
    #[id = "ping-pong"]
    pub ping_pong: BoolParam,
    #[id = "mod-sync"]
    pub mod_sync: BoolParam,
    #[id = "feed"]
    pub feed: BoolParam,
    #[id = "throw"]
    pub throw: BoolParam,
    #[id = "hold"]
    pub hold: BoolParam,
    #[id = "kill"]
    pub kill: BoolParam,
    #[id = "dub"]
    pub dub: BoolParam,
    #[id = "bypass"]
    pub bypass: BoolParam,
    // Scalar IDs avoid heap-allocated nested-array suffixes during state restoration.
    #[id = "route-on_1"]
    r1_enabled: BoolParam,
    #[id = "route-source_1"]
    r1_source: IntParam,
    #[id = "route-target_1"]
    r1_target: IntParam,
    #[id = "route-depth_1"]
    r1_amount: FloatParam,
    #[id = "route-curve_1"]
    r1_curve: FloatParam,
    #[id = "route-on_2"]
    r2_enabled: BoolParam,
    #[id = "route-source_2"]
    r2_source: IntParam,
    #[id = "route-target_2"]
    r2_target: IntParam,
    #[id = "route-depth_2"]
    r2_amount: FloatParam,
    #[id = "route-curve_2"]
    r2_curve: FloatParam,
    #[id = "route-on_3"]
    r3_enabled: BoolParam,
    #[id = "route-source_3"]
    r3_source: IntParam,
    #[id = "route-target_3"]
    r3_target: IntParam,
    #[id = "route-depth_3"]
    r3_amount: FloatParam,
    #[id = "route-curve_3"]
    r3_curve: FloatParam,
    #[id = "route-on_4"]
    r4_enabled: BoolParam,
    #[id = "route-source_4"]
    r4_source: IntParam,
    #[id = "route-target_4"]
    r4_target: IntParam,
    #[id = "route-depth_4"]
    r4_amount: FloatParam,
    #[id = "route-curve_4"]
    r4_curve: FloatParam,
    #[id = "route-on_5"]
    r5_enabled: BoolParam,
    #[id = "route-source_5"]
    r5_source: IntParam,
    #[id = "route-target_5"]
    r5_target: IntParam,
    #[id = "route-depth_5"]
    r5_amount: FloatParam,
    #[id = "route-curve_5"]
    r5_curve: FloatParam,
    #[id = "route-on_6"]
    r6_enabled: BoolParam,
    #[id = "route-source_6"]
    r6_source: IntParam,
    #[id = "route-target_6"]
    r6_target: IntParam,
    #[id = "route-depth_6"]
    r6_amount: FloatParam,
    #[id = "route-curve_6"]
    r6_curve: FloatParam,
    #[id = "route-on_7"]
    r7_enabled: BoolParam,
    #[id = "route-source_7"]
    r7_source: IntParam,
    #[id = "route-target_7"]
    r7_target: IntParam,
    #[id = "route-depth_7"]
    r7_amount: FloatParam,
    #[id = "route-curve_7"]
    r7_curve: FloatParam,
    #[id = "route-on_8"]
    r8_enabled: BoolParam,
    #[id = "route-source_8"]
    r8_source: IntParam,
    #[id = "route-target_8"]
    r8_target: IntParam,
    #[id = "route-depth_8"]
    r8_amount: FloatParam,
    #[id = "route-curve_8"]
    r8_curve: FloatParam,
    #[id = "cc_1"]
    cc1: IntParam,
    #[id = "cc_2"]
    cc2: IntParam,
    #[id = "cc_3"]
    cc3: IntParam,
    #[id = "cc_4"]
    cc4: IntParam,
    #[id = "cc_5"]
    cc5: IntParam,
    #[id = "cc_6"]
    cc6: IntParam,
    #[id = "cc_7"]
    cc7: IntParam,
    #[id = "cc_8"]
    cc8: IntParam,
}
fn float(name: &str, v: f32, min: f32, max: f32) -> FloatParam {
    let p = FloatParam::new(name, v, FloatRange::Linear { min, max });
    match name {
        "TIME" | "Right time" => p
            .with_unit(" ms")
            .with_value_to_string(formatters::v2s_f32_rounded(0)),
        "Input" | "OUTPUT" => p
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),
        "FEEDBACK" | "FILTER" | "DRIVE" | "MIX" => p
            .with_unit(" %")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),
        _ => p,
    }
}
impl Default for EchoParams {
    fn default() -> Self {
        let p = echogirl_core::factory(0);
        let c = p.config;
        Self {
            editor: EguiState::from_size(1020, 760),
            name: TextField::new(&p.name),
            notes: TextField::new(&p.description),
            midi: MidiState::default(),
            schema: Arc::new(AtomicU32::new(1)),
            preset: IntParam::new("Preset identity", 0, IntRange::Linear { min: 0, max: 10 })
                .non_automatable()
                .hide(),
            r1_enabled: BoolParam::new("Route 1 enabled", c.mappings[0].enabled),
            r1_source: IntParam::new(
                "Route 1 source",
                if c.mappings[0].source == Source::X {
                    0
                } else {
                    1
                },
                IntRange::Linear { min: 0, max: 1 },
            ),
            r1_target: IntParam::new(
                "Route 1 target",
                Target::ALL
                    .iter()
                    .position(|t| *t == c.mappings[0].target)
                    .unwrap_or(0) as i32,
                IntRange::Linear { min: 0, max: 7 },
            ),
            r1_amount: float("Route 1 depth", c.mappings[0].amount, -2., 2.),
            r1_curve: float("Route 1 curve", c.mappings[0].curve, 0.2, 5.),
            r2_enabled: BoolParam::new("Route 2 enabled", c.mappings[1].enabled),
            r2_source: IntParam::new(
                "Route 2 source",
                if c.mappings[1].source == Source::X {
                    0
                } else {
                    1
                },
                IntRange::Linear { min: 0, max: 1 },
            ),
            r2_target: IntParam::new(
                "Route 2 target",
                Target::ALL
                    .iter()
                    .position(|t| *t == c.mappings[1].target)
                    .unwrap_or(0) as i32,
                IntRange::Linear { min: 0, max: 7 },
            ),
            r2_amount: float("Route 2 depth", c.mappings[1].amount, -2., 2.),
            r2_curve: float("Route 2 curve", c.mappings[1].curve, 0.2, 5.),
            r3_enabled: BoolParam::new("Route 3 enabled", c.mappings[2].enabled),
            r3_source: IntParam::new(
                "Route 3 source",
                if c.mappings[2].source == Source::X {
                    0
                } else {
                    1
                },
                IntRange::Linear { min: 0, max: 1 },
            ),
            r3_target: IntParam::new(
                "Route 3 target",
                Target::ALL
                    .iter()
                    .position(|t| *t == c.mappings[2].target)
                    .unwrap_or(0) as i32,
                IntRange::Linear { min: 0, max: 7 },
            ),
            r3_amount: float("Route 3 depth", c.mappings[2].amount, -2., 2.),
            r3_curve: float("Route 3 curve", c.mappings[2].curve, 0.2, 5.),
            r4_enabled: BoolParam::new("Route 4 enabled", c.mappings[3].enabled),
            r4_source: IntParam::new(
                "Route 4 source",
                if c.mappings[3].source == Source::X {
                    0
                } else {
                    1
                },
                IntRange::Linear { min: 0, max: 1 },
            ),
            r4_target: IntParam::new(
                "Route 4 target",
                Target::ALL
                    .iter()
                    .position(|t| *t == c.mappings[3].target)
                    .unwrap_or(0) as i32,
                IntRange::Linear { min: 0, max: 7 },
            ),
            r4_amount: float("Route 4 depth", c.mappings[3].amount, -2., 2.),
            r4_curve: float("Route 4 curve", c.mappings[3].curve, 0.2, 5.),
            r5_enabled: BoolParam::new("Route 5 enabled", c.mappings[4].enabled),
            r5_source: IntParam::new(
                "Route 5 source",
                if c.mappings[4].source == Source::X {
                    0
                } else {
                    1
                },
                IntRange::Linear { min: 0, max: 1 },
            ),
            r5_target: IntParam::new(
                "Route 5 target",
                Target::ALL
                    .iter()
                    .position(|t| *t == c.mappings[4].target)
                    .unwrap_or(0) as i32,
                IntRange::Linear { min: 0, max: 7 },
            ),
            r5_amount: float("Route 5 depth", c.mappings[4].amount, -2., 2.),
            r5_curve: float("Route 5 curve", c.mappings[4].curve, 0.2, 5.),
            r6_enabled: BoolParam::new("Route 6 enabled", c.mappings[5].enabled),
            r6_source: IntParam::new(
                "Route 6 source",
                if c.mappings[5].source == Source::X {
                    0
                } else {
                    1
                },
                IntRange::Linear { min: 0, max: 1 },
            ),
            r6_target: IntParam::new(
                "Route 6 target",
                Target::ALL
                    .iter()
                    .position(|t| *t == c.mappings[5].target)
                    .unwrap_or(0) as i32,
                IntRange::Linear { min: 0, max: 7 },
            ),
            r6_amount: float("Route 6 depth", c.mappings[5].amount, -2., 2.),
            r6_curve: float("Route 6 curve", c.mappings[5].curve, 0.2, 5.),
            r7_enabled: BoolParam::new("Route 7 enabled", c.mappings[6].enabled),
            r7_source: IntParam::new(
                "Route 7 source",
                if c.mappings[6].source == Source::X {
                    0
                } else {
                    1
                },
                IntRange::Linear { min: 0, max: 1 },
            ),
            r7_target: IntParam::new(
                "Route 7 target",
                Target::ALL
                    .iter()
                    .position(|t| *t == c.mappings[6].target)
                    .unwrap_or(0) as i32,
                IntRange::Linear { min: 0, max: 7 },
            ),
            r7_amount: float("Route 7 depth", c.mappings[6].amount, -2., 2.),
            r7_curve: float("Route 7 curve", c.mappings[6].curve, 0.2, 5.),
            r8_enabled: BoolParam::new("Route 8 enabled", c.mappings[7].enabled),
            r8_source: IntParam::new(
                "Route 8 source",
                if c.mappings[7].source == Source::X {
                    0
                } else {
                    1
                },
                IntRange::Linear { min: 0, max: 1 },
            ),
            r8_target: IntParam::new(
                "Route 8 target",
                Target::ALL
                    .iter()
                    .position(|t| *t == c.mappings[7].target)
                    .unwrap_or(0) as i32,
                IntRange::Linear { min: 0, max: 7 },
            ),
            r8_amount: float("Route 8 depth", c.mappings[7].amount, -2., 2.),
            r8_curve: float("Route 8 curve", c.mappings[7].curve, 0.2, 5.),
            cc1: IntParam::new(
                "MIDI CC 1",
                c.midi_cc[0] as i32,
                IntRange::Linear { min: 0, max: 119 },
            ),
            cc2: IntParam::new(
                "MIDI CC 2",
                c.midi_cc[1] as i32,
                IntRange::Linear { min: 0, max: 119 },
            ),
            cc3: IntParam::new(
                "MIDI CC 3",
                c.midi_cc[2] as i32,
                IntRange::Linear { min: 0, max: 119 },
            ),
            cc4: IntParam::new(
                "MIDI CC 4",
                c.midi_cc[3] as i32,
                IntRange::Linear { min: 0, max: 119 },
            ),
            cc5: IntParam::new(
                "MIDI CC 5",
                c.midi_cc[4] as i32,
                IntRange::Linear { min: 0, max: 119 },
            ),
            cc6: IntParam::new(
                "MIDI CC 6",
                c.midi_cc[5] as i32,
                IntRange::Linear { min: 0, max: 119 },
            ),
            cc7: IntParam::new(
                "MIDI CC 7",
                c.midi_cc[6] as i32,
                IntRange::Linear { min: 0, max: 119 },
            ),
            cc8: IntParam::new(
                "MIDI CC 8",
                c.midi_cc[7] as i32,
                IntRange::Linear { min: 0, max: 119 },
            ),
            revision: BoolParam::new("Preset document revision", false)
                .non_automatable()
                .hide(),
            time_ms: float("TIME", c.time_ms, 10.0, 12000.0),
            right_ms: float("Right time", c.right_ms, 10.0, 12000.0),
            feedback: float("FEEDBACK", c.feedback, 0.0, 1.08),
            filter: float("FILTER", c.filter, 0.0, 1.0),
            drive: float("DRIVE", c.drive, 0.0, 1.0),
            mix: float("MIX", c.mix, 0.0, 1.0),
            input_db: float("Input", c.input_db, -24.0, 12.0),
            output_db: float("OUTPUT", c.output_db, -24.0, 6.0),
            low_cut: float("Feedback high-pass", c.low_cut, 0.0, 3000.0),
            high_cut: float("Feedback low-pass", c.high_cut, 500.0, 20000.0),
            resonance: float("Resonance", c.resonance, 0.0, 0.85),
            wow: float("Wow / modulation", c.wow, 0.0, 1.0),
            mod_rate: float("Modulation rate", c.mod_rate, 0.05, 8.0),
            mod_beats: float("Modulation quarter notes", c.mod_beats, 0.125, 16.0),
            spring: float("Spring", c.spring, 0.0, 0.65),
            x: float("X", c.x, 0.0, 1.0),
            y: float("Y", c.y, 0.0, 1.0),
            dub_feedback: float("DUB feedback", c.dub_feedback, 0.0, 0.4),
            dub_drive: float("DUB drive", c.dub_drive, 0.0, 0.7),
            dub_filter: float("DUB filter", c.dub_filter, -0.5, 0.5),
            character: IntParam::new(
                "Character",
                c.character as i32,
                IntRange::Linear { min: 0, max: 3 },
            ),
            left_div: IntParam::new(
                "Left division",
                c.left_div as i32,
                IntRange::Linear { min: 0, max: 12 },
            )
            .with_value_to_string(Arc::new(|i| {
                echogirl_core::preset::DIVISIONS[i as usize].0.into()
            })),
            right_div: IntParam::new(
                "Right division",
                c.right_div as i32,
                IntRange::Linear { min: 0, max: 12 },
            )
            .with_value_to_string(Arc::new(|i| {
                echogirl_core::preset::DIVISIONS[i as usize].0.into()
            })),
            sync: BoolParam::new("Tempo sync", c.sync),
            link: BoolParam::new("Link times", c.link),
            ping_pong: BoolParam::new("Ping-pong", c.ping_pong),
            mod_sync: BoolParam::new("Modulation sync", c.mod_sync),
            feed: BoolParam::new("FEED", c.feed),
            throw: BoolParam::new("THROW", c.throw),
            hold: BoolParam::new("HOLD", c.hold),
            kill: BoolParam::new("KILL", c.kill),
            dub: BoolParam::new("DUB", c.dub),
            bypass: BoolParam::new("Bypass", c.bypass).make_bypass(),
        }
    }
}
impl EchoParams {
    fn route_refs(&self) -> [RouteRef<'_>; 8] {
        [
            RouteRef {
                enabled: &self.r1_enabled,
                source: &self.r1_source,
                target: &self.r1_target,
                amount: &self.r1_amount,
                curve: &self.r1_curve,
            },
            RouteRef {
                enabled: &self.r2_enabled,
                source: &self.r2_source,
                target: &self.r2_target,
                amount: &self.r2_amount,
                curve: &self.r2_curve,
            },
            RouteRef {
                enabled: &self.r3_enabled,
                source: &self.r3_source,
                target: &self.r3_target,
                amount: &self.r3_amount,
                curve: &self.r3_curve,
            },
            RouteRef {
                enabled: &self.r4_enabled,
                source: &self.r4_source,
                target: &self.r4_target,
                amount: &self.r4_amount,
                curve: &self.r4_curve,
            },
            RouteRef {
                enabled: &self.r5_enabled,
                source: &self.r5_source,
                target: &self.r5_target,
                amount: &self.r5_amount,
                curve: &self.r5_curve,
            },
            RouteRef {
                enabled: &self.r6_enabled,
                source: &self.r6_source,
                target: &self.r6_target,
                amount: &self.r6_amount,
                curve: &self.r6_curve,
            },
            RouteRef {
                enabled: &self.r7_enabled,
                source: &self.r7_source,
                target: &self.r7_target,
                amount: &self.r7_amount,
                curve: &self.r7_curve,
            },
            RouteRef {
                enabled: &self.r8_enabled,
                source: &self.r8_source,
                target: &self.r8_target,
                amount: &self.r8_amount,
                curve: &self.r8_curve,
            },
        ]
    }
    fn cc_refs(&self) -> [&IntParam; 8] {
        [
            &self.cc1, &self.cc2, &self.cc3, &self.cc4, &self.cc5, &self.cc6, &self.cc7, &self.cc8,
        ]
    }

    pub fn base(&self) -> Config {
        Config {
            time_ms: self.time_ms.value(),
            right_ms: self.right_ms.value(),
            feedback: self.feedback.value(),
            filter: self.filter.value(),
            drive: self.drive.value(),
            mix: self.mix.value(),
            input_db: self.input_db.value(),
            output_db: self.output_db.value(),
            low_cut: self.low_cut.value(),
            high_cut: self.high_cut.value(),
            resonance: self.resonance.value(),
            wow: self.wow.value(),
            mod_rate: self.mod_rate.value(),
            mod_beats: self.mod_beats.value(),
            spring: self.spring.value(),
            x: self.x.value(),
            y: self.y.value(),
            dub_feedback: self.dub_feedback.value(),
            dub_drive: self.dub_drive.value(),
            dub_filter: self.dub_filter.value(),
            sync: self.sync.value(),
            link: self.link.value(),
            ping_pong: self.ping_pong.value(),
            mod_sync: self.mod_sync.value(),
            feed: self.feed.value(),
            throw: self.throw.value(),
            hold: self.hold.value(),
            kill: self.kill.value(),
            dub: self.dub.value(),
            bypass: self.bypass.value(),
            character: self.character.value() as u8,
            left_div: self.left_div.value() as u8,
            right_div: self.right_div.value() as u8,
            mappings: self.route_refs().map(|r| r.value()),
            midi_cc: self.cc_refs().map(|p| p.value() as u8),
        }
    }
    pub fn config(&self) -> Config {
        let mut c = self.base();
        let a = self.midi.read();
        if a[0] >= 0. {
            c.x = a[0];
        }
        if a[1] >= 0. {
            c.y = a[1];
        }
        for (i, b) in [
            &mut c.throw,
            &mut c.hold,
            &mut c.kill,
            &mut c.dub,
            &mut c.feed,
            &mut c.bypass,
        ]
        .into_iter()
        .enumerate()
        {
            if a[i + 2] >= 0. {
                *b = a[i + 2] >= 0.5;
            }
        }
        c
    }
    pub fn preset_document(&self) -> Preset {
        let mut config = self.config();
        config.release_buttons();
        Preset {
            version: 1,
            name: self.name.get(),
            description: self.notes.get(),
            config,
        }
    }
    pub fn apply(&self, p: &Preset, index: i32, s: &ParamSetter) {
        self.midi.clear();
        self.name.set_text(&p.name);
        self.notes.set_text(&p.description);
        let mut c = p.config;
        c.release_buttons();
        s.begin_set_parameter(&self.time_ms);
        s.set_parameter(&self.time_ms, c.time_ms);
        s.end_set_parameter(&self.time_ms);
        s.begin_set_parameter(&self.right_ms);
        s.set_parameter(&self.right_ms, c.right_ms);
        s.end_set_parameter(&self.right_ms);
        s.begin_set_parameter(&self.feedback);
        s.set_parameter(&self.feedback, c.feedback);
        s.end_set_parameter(&self.feedback);
        s.begin_set_parameter(&self.filter);
        s.set_parameter(&self.filter, c.filter);
        s.end_set_parameter(&self.filter);
        s.begin_set_parameter(&self.drive);
        s.set_parameter(&self.drive, c.drive);
        s.end_set_parameter(&self.drive);
        s.begin_set_parameter(&self.mix);
        s.set_parameter(&self.mix, c.mix);
        s.end_set_parameter(&self.mix);
        s.begin_set_parameter(&self.input_db);
        s.set_parameter(&self.input_db, c.input_db);
        s.end_set_parameter(&self.input_db);
        s.begin_set_parameter(&self.output_db);
        s.set_parameter(&self.output_db, c.output_db);
        s.end_set_parameter(&self.output_db);
        s.begin_set_parameter(&self.low_cut);
        s.set_parameter(&self.low_cut, c.low_cut);
        s.end_set_parameter(&self.low_cut);
        s.begin_set_parameter(&self.high_cut);
        s.set_parameter(&self.high_cut, c.high_cut);
        s.end_set_parameter(&self.high_cut);
        s.begin_set_parameter(&self.resonance);
        s.set_parameter(&self.resonance, c.resonance);
        s.end_set_parameter(&self.resonance);
        s.begin_set_parameter(&self.wow);
        s.set_parameter(&self.wow, c.wow);
        s.end_set_parameter(&self.wow);
        s.begin_set_parameter(&self.mod_rate);
        s.set_parameter(&self.mod_rate, c.mod_rate);
        s.end_set_parameter(&self.mod_rate);
        s.begin_set_parameter(&self.mod_beats);
        s.set_parameter(&self.mod_beats, c.mod_beats);
        s.end_set_parameter(&self.mod_beats);
        s.begin_set_parameter(&self.spring);
        s.set_parameter(&self.spring, c.spring);
        s.end_set_parameter(&self.spring);
        s.begin_set_parameter(&self.x);
        s.set_parameter(&self.x, c.x);
        s.end_set_parameter(&self.x);
        s.begin_set_parameter(&self.y);
        s.set_parameter(&self.y, c.y);
        s.end_set_parameter(&self.y);
        s.begin_set_parameter(&self.dub_feedback);
        s.set_parameter(&self.dub_feedback, c.dub_feedback);
        s.end_set_parameter(&self.dub_feedback);
        s.begin_set_parameter(&self.dub_drive);
        s.set_parameter(&self.dub_drive, c.dub_drive);
        s.end_set_parameter(&self.dub_drive);
        s.begin_set_parameter(&self.dub_filter);
        s.set_parameter(&self.dub_filter, c.dub_filter);
        s.end_set_parameter(&self.dub_filter);
        s.begin_set_parameter(&self.sync);
        s.set_parameter(&self.sync, c.sync);
        s.end_set_parameter(&self.sync);
        s.begin_set_parameter(&self.link);
        s.set_parameter(&self.link, c.link);
        s.end_set_parameter(&self.link);
        s.begin_set_parameter(&self.ping_pong);
        s.set_parameter(&self.ping_pong, c.ping_pong);
        s.end_set_parameter(&self.ping_pong);
        s.begin_set_parameter(&self.mod_sync);
        s.set_parameter(&self.mod_sync, c.mod_sync);
        s.end_set_parameter(&self.mod_sync);
        s.begin_set_parameter(&self.feed);
        s.set_parameter(&self.feed, c.feed);
        s.end_set_parameter(&self.feed);
        s.begin_set_parameter(&self.throw);
        s.set_parameter(&self.throw, c.throw);
        s.end_set_parameter(&self.throw);
        s.begin_set_parameter(&self.hold);
        s.set_parameter(&self.hold, c.hold);
        s.end_set_parameter(&self.hold);
        s.begin_set_parameter(&self.kill);
        s.set_parameter(&self.kill, c.kill);
        s.end_set_parameter(&self.kill);
        s.begin_set_parameter(&self.dub);
        s.set_parameter(&self.dub, c.dub);
        s.end_set_parameter(&self.dub);
        s.begin_set_parameter(&self.bypass);
        s.set_parameter(&self.bypass, c.bypass);
        s.end_set_parameter(&self.bypass);
        s.begin_set_parameter(&self.character);
        s.set_parameter(&self.character, c.character as i32);
        s.end_set_parameter(&self.character);
        s.begin_set_parameter(&self.left_div);
        s.set_parameter(&self.left_div, c.left_div as i32);
        s.end_set_parameter(&self.left_div);
        s.begin_set_parameter(&self.right_div);
        s.set_parameter(&self.right_div, c.right_div as i32);
        s.end_set_parameter(&self.right_div);
        for (i, m) in c.mappings.iter().enumerate() {
            let r = self.route_refs()[i];
            s.begin_set_parameter(r.enabled);
            s.set_parameter(r.enabled, m.enabled);
            s.end_set_parameter(r.enabled);
            s.begin_set_parameter(r.source);
            s.set_parameter(r.source, if m.source == Source::X { 0 } else { 1 });
            s.end_set_parameter(r.source);
            s.begin_set_parameter(r.target);
            s.set_parameter(
                r.target,
                Target::ALL.iter().position(|t| *t == m.target).unwrap_or(0) as i32,
            );
            s.end_set_parameter(r.target);
            s.begin_set_parameter(r.amount);
            s.set_parameter(r.amount, m.amount);
            s.end_set_parameter(r.amount);
            s.begin_set_parameter(r.curve);
            s.set_parameter(r.curve, m.curve);
            s.end_set_parameter(r.curve);
        }
        for (i, cc) in c.midi_cc.iter().enumerate() {
            let p = self.cc_refs()[i];
            s.begin_set_parameter(p);
            s.set_parameter(p, *cc as i32);
            s.end_set_parameter(p);
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
