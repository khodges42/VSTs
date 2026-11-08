// SPDX-License-Identifier: GPL-3.0-or-later
use crate::fields::{MidiAxes, TextField};
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use spasm_core::{
    Config, Effect, Preset, factory,
    patch::{DIVISIONS, PAD_NAMES, QUANTIZE, Route, Source, Target},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};
#[derive(Clone, Copy)]
pub struct RouteRef<'a> {
    pub enabled: &'a BoolParam,
    pub source: &'a IntParam,
    pub target: &'a IntParam,
    pub start: &'a FloatParam,
    pub end: &'a FloatParam,
    pub curve: &'a FloatParam,
}
impl RouteRef<'_> {
    fn value(&self) -> Route {
        Route {
            enabled: self.enabled.value(),
            source: Source::ALL[self.source.value() as usize],
            target: Target::ALL[self.target.value() as usize],
            start: self.start.value(),
            end: self.end.value(),
            curve: self.curve.value(),
        }
    }
}
#[derive(Params)]
pub struct SpasmParams {
    #[persist = "editor"]
    pub editor: Arc<EguiState>,
    #[persist = "edit"]
    pub edit: Arc<AtomicBool>,
    #[persist = "preset-name"]
    pub name: TextField,
    #[persist = "preset-notes"]
    pub notes: TextField,
    #[persist = "schema"]
    pub schema: Arc<AtomicU32>,
    #[persist = "midi-xy"]
    pub midi: MidiAxes,
    #[id = "preset"]
    pub preset: IntParam,
    #[id = "revision"]
    pub revision: BoolParam,
    #[id = "metadata"]
    pub metadata: BoolParam,
    #[id = "history-seconds"]
    pub history_seconds: FloatParam,
    #[id = "stutter-ms"]
    pub stutter_ms: FloatParam,
    #[id = "acceleration"]
    pub acceleration: FloatParam,
    #[id = "stop-ms"]
    pub stop_ms: FloatParam,
    #[id = "stop-beats"]
    pub stop_beats: FloatParam,
    #[id = "stop-curve"]
    pub stop_curve: FloatParam,
    #[id = "dive-ms"]
    pub dive_ms: FloatParam,
    #[id = "dive-semitones"]
    pub dive_semitones: FloatParam,
    #[id = "pitch-semitones"]
    pub pitch_semitones: FloatParam,
    #[id = "gate-depth"]
    pub gate_depth: FloatParam,
    #[id = "gate-attack-ms"]
    pub gate_attack_ms: FloatParam,
    #[id = "gate-release-ms"]
    pub gate_release_ms: FloatParam,
    #[id = "bits"]
    pub bits: FloatParam,
    #[id = "reduction"]
    pub reduction: FloatParam,
    #[id = "filter-hz"]
    pub filter_hz: FloatParam,
    #[id = "drive"]
    pub drive: FloatParam,
    #[id = "wander"]
    pub wander: FloatParam,
    #[id = "mix"]
    pub mix: FloatParam,
    #[id = "fx-output-db"]
    pub fx_output_db: FloatParam,
    #[id = "crossfade-ms"]
    pub crossfade_ms: FloatParam,
    #[id = "edge-ms"]
    pub edge_ms: FloatParam,
    #[id = "x"]
    pub x: FloatParam,
    #[id = "y"]
    pub y: FloatParam,
    #[id = "capture-locked"]
    pub capture_locked: BoolParam,
    #[id = "stutter-reverse"]
    pub stutter_reverse: BoolParam,
    #[id = "stop-sync"]
    pub stop_sync: BoolParam,
    #[id = "filter-enabled"]
    pub filter_enabled: BoolParam,
    #[id = "latch"]
    pub latch: BoolParam,
    #[id = "reset-clear"]
    pub reset_clear: BoolParam,
    #[id = "auto-damage"]
    pub auto_damage: BoolParam,
    #[id = "bypass"]
    pub bypass: BoolParam,
    #[id = "repeat-div"]
    pub repeat_div: IntParam,
    #[id = "stutter-div"]
    pub stutter_div: IntParam,
    #[id = "reverse-div"]
    pub reverse_div: IntParam,
    #[id = "gate-div"]
    pub gate_div: IntParam,
    #[id = "chaos-div"]
    pub chaos_div: IntParam,
    #[id = "quantize"]
    pub quantize: IntParam,
    #[id = "repeat-count"]
    pub repeat_count: IntParam,
    #[id = "gate-pattern"]
    pub gate_pattern: IntParam,
    #[id = "latch-mask"]
    pub latch_mask: IntParam,
    #[id = "chaos-mask"]
    pub chaos_mask: IntParam,
    #[id = "seed"]
    pub seed: IntParam,
    #[id = "midi-channel"]
    pub midi_channel: IntParam,
    #[id = "pad-1"]
    p0: BoolParam,
    #[id = "note-1"]
    n0: IntParam,
    #[id = "pad-2"]
    p1: BoolParam,
    #[id = "note-2"]
    n1: IntParam,
    #[id = "pad-3"]
    p2: BoolParam,
    #[id = "note-3"]
    n2: IntParam,
    #[id = "pad-4"]
    p3: BoolParam,
    #[id = "note-4"]
    n3: IntParam,
    #[id = "pad-5"]
    p4: BoolParam,
    #[id = "note-5"]
    n4: IntParam,
    #[id = "pad-6"]
    p5: BoolParam,
    #[id = "note-6"]
    n5: IntParam,
    #[id = "pad-7"]
    p6: BoolParam,
    #[id = "note-7"]
    n6: IntParam,
    #[id = "pad-8"]
    p7: BoolParam,
    #[id = "note-8"]
    n7: IntParam,
    #[id = "assign-1"]
    a0: IntParam,
    #[id = "assign-2"]
    a1: IntParam,
    #[id = "assign-3"]
    a2: IntParam,
    #[id = "assign-4"]
    a3: IntParam,
    #[id = "assign-5"]
    a4: IntParam,
    #[id = "assign-6"]
    a5: IntParam,
    #[id = "assign-7"]
    a6: IntParam,
    #[id = "priority-1"]
    priority0: IntParam,
    #[id = "priority-2"]
    priority1: IntParam,
    #[id = "priority-3"]
    priority2: IntParam,
    #[id = "priority-4"]
    priority3: IntParam,
    #[id = "priority-5"]
    priority4: IntParam,
    #[id = "cc-1"]
    cc0: IntParam,
    #[id = "cc-2"]
    cc1: IntParam,
    #[id = "r1-enabled"]
    r0_enabled: BoolParam,
    #[id = "r1-source"]
    r0_source: IntParam,
    #[id = "r1-target"]
    r0_target: IntParam,
    #[id = "r1-start"]
    r0_start: FloatParam,
    #[id = "r1-end"]
    r0_end: FloatParam,
    #[id = "r1-curve"]
    r0_curve: FloatParam,
    #[id = "r2-enabled"]
    r1_enabled: BoolParam,
    #[id = "r2-source"]
    r1_source: IntParam,
    #[id = "r2-target"]
    r1_target: IntParam,
    #[id = "r2-start"]
    r1_start: FloatParam,
    #[id = "r2-end"]
    r1_end: FloatParam,
    #[id = "r2-curve"]
    r1_curve: FloatParam,
    #[id = "r3-enabled"]
    r2_enabled: BoolParam,
    #[id = "r3-source"]
    r2_source: IntParam,
    #[id = "r3-target"]
    r2_target: IntParam,
    #[id = "r3-start"]
    r2_start: FloatParam,
    #[id = "r3-end"]
    r2_end: FloatParam,
    #[id = "r3-curve"]
    r2_curve: FloatParam,
    #[id = "r4-enabled"]
    r3_enabled: BoolParam,
    #[id = "r4-source"]
    r3_source: IntParam,
    #[id = "r4-target"]
    r3_target: IntParam,
    #[id = "r4-start"]
    r3_start: FloatParam,
    #[id = "r4-end"]
    r3_end: FloatParam,
    #[id = "r4-curve"]
    r3_curve: FloatParam,
    #[id = "r5-enabled"]
    r4_enabled: BoolParam,
    #[id = "r5-source"]
    r4_source: IntParam,
    #[id = "r5-target"]
    r4_target: IntParam,
    #[id = "r5-start"]
    r4_start: FloatParam,
    #[id = "r5-end"]
    r4_end: FloatParam,
    #[id = "r5-curve"]
    r4_curve: FloatParam,
    #[id = "r6-enabled"]
    r5_enabled: BoolParam,
    #[id = "r6-source"]
    r5_source: IntParam,
    #[id = "r6-target"]
    r5_target: IntParam,
    #[id = "r6-start"]
    r5_start: FloatParam,
    #[id = "r6-end"]
    r5_end: FloatParam,
    #[id = "r6-curve"]
    r5_curve: FloatParam,
    #[id = "r7-enabled"]
    r6_enabled: BoolParam,
    #[id = "r7-source"]
    r6_source: IntParam,
    #[id = "r7-target"]
    r6_target: IntParam,
    #[id = "r7-start"]
    r6_start: FloatParam,
    #[id = "r7-end"]
    r6_end: FloatParam,
    #[id = "r7-curve"]
    r6_curve: FloatParam,
    #[id = "r8-enabled"]
    r7_enabled: BoolParam,
    #[id = "r8-source"]
    r7_source: IntParam,
    #[id = "r8-target"]
    r7_target: IntParam,
    #[id = "r8-start"]
    r7_start: FloatParam,
    #[id = "r8-end"]
    r7_end: FloatParam,
    #[id = "r8-curve"]
    r7_curve: FloatParam,
}
fn float(name: &str, v: f32, min: f32, max: f32, unit: &'static str) -> FloatParam {
    FloatParam::new(name, v, FloatRange::Linear { min, max })
        .with_value_to_string(Arc::new(move |x| {
            if unit == "%" {
                format!("{:.0}%", x * 100.)
            } else {
                format!("{x:.2} {unit}")
            }
        }))
        .with_string_to_value(Arc::new(move |s| {
            let n = s.trim().trim_end_matches(unit).trim().parse::<f32>().ok()?;
            Some(if unit == "%" { n / 100. } else { n })
        }))
}
fn int(name: &str, v: i32, min: i32, max: i32) -> IntParam {
    IntParam::new(name, v, IntRange::Linear { min, max })
}
impl Default for SpasmParams {
    fn default() -> Self {
        let p = factory(0);
        let c = p.config;
        Self {
            editor: EguiState::from_size(1040, 820),
            edit: Arc::new(AtomicBool::new(false)),
            name: TextField::new(&p.name),
            notes: TextField::new(&p.description),
            schema: Arc::new(AtomicU32::new(1)),
            midi: MidiAxes::default(),
            preset: int("Preset identity", 0, 0, 10).non_automatable().hide(),
            revision: BoolParam::new("Preset applied", false)
                .non_automatable()
                .hide(),
            metadata: BoolParam::new("Metadata revision", false)
                .non_automatable()
                .hide(),
            history_seconds: float("History", c.history_seconds, 0.25, 8.0, "s"),
            stutter_ms: float("Fragment", c.stutter_ms, 2.0, 250.0, "ms"),
            acceleration: float("Acceleration", c.acceleration, 0.0, 1.0, "%"),
            stop_ms: float("Stop time", c.stop_ms, 50.0, 4000.0, "ms"),
            stop_beats: float("Stop beats", c.stop_beats, 0.125, 4.0, "beats"),
            stop_curve: float("Stop curve", c.stop_curve, 0.3, 4.0, ""),
            dive_ms: float("Dive time", c.dive_ms, 50.0, 4000.0, "ms"),
            dive_semitones: float("Dive depth", c.dive_semitones, -48.0, 0.0, "st"),
            pitch_semitones: float("Pitch", c.pitch_semitones, -48.0, 12.0, "st"),
            gate_depth: float("Gate depth", c.gate_depth, 0.0, 1.0, "%"),
            gate_attack_ms: float("Gate attack", c.gate_attack_ms, 0.2, 25.0, "ms"),
            gate_release_ms: float("Gate release", c.gate_release_ms, 0.2, 50.0, "ms"),
            bits: float("Bit depth", c.bits, 4.0, 16.0, "bits"),
            reduction: float("Sample hold", c.reduction, 1.0, 64.0, "x"),
            filter_hz: float("Filter", c.filter_hz, 100.0, 20000.0, "Hz"),
            drive: float("Drive", c.drive, 0.0, 1.0, "%"),
            wander: float("Pitch wander", c.wander, 0.0, 1.0, "%"),
            mix: float("Mix", c.mix, 0.0, 1.0, "%"),
            fx_output_db: float("FX output", c.fx_output_db, -12.0, 6.0, "dB"),
            crossfade_ms: float("Release crossfade", c.crossfade_ms, 2.0, 25.0, "ms"),
            edge_ms: float("Window edge", c.edge_ms, 0.1, 10.0, "ms"),
            x: float("X", c.x, 0.0, 1.0, "%"),
            y: float("Y", c.y, 0.0, 1.0, "%"),
            capture_locked: BoolParam::new("Lock active capture", c.capture_locked),
            stutter_reverse: BoolParam::new("Reverse stutter", c.stutter_reverse),
            stop_sync: BoolParam::new("Tempo stop", c.stop_sync),
            filter_enabled: BoolParam::new("Wet filter", c.filter_enabled),
            latch: BoolParam::new("LATCH", c.latch),
            reset_clear: BoolParam::new("RESET clears history", c.reset_clear),
            auto_damage: BoolParam::new("Damage overlay", c.auto_damage),
            bypass: BoolParam::new("Bypass", c.bypass).make_bypass(),
            repeat_div: int("Repeat division", c.repeat_div as i32, 0, 6)
                .with_value_to_string(Arc::new(|i| DIVISIONS[i as usize].0.into())),
            stutter_div: int("Stutter division", c.stutter_div as i32, 0, 6)
                .with_value_to_string(Arc::new(|i| DIVISIONS[i as usize].0.into())),
            reverse_div: int("Reverse division", c.reverse_div as i32, 0, 6)
                .with_value_to_string(Arc::new(|i| DIVISIONS[i as usize].0.into())),
            gate_div: int("Gate division", c.gate_div as i32, 0, 6)
                .with_value_to_string(Arc::new(|i| DIVISIONS[i as usize].0.into())),
            chaos_div: int("Chaos division", c.chaos_div as i32, 0, 6)
                .with_value_to_string(Arc::new(|i| DIVISIONS[i as usize].0.into())),
            quantize: int("Quantization", c.quantize as i32, 0, 4)
                .with_value_to_string(Arc::new(|i| QUANTIZE[i as usize].0.into())),
            repeat_count: int("Repeat count", c.repeat_count as i32, 0, 64),
            gate_pattern: int("Gate pattern", c.gate_pattern as i32, 0, 65535),
            latch_mask: int("Latched pads", c.latch_mask as i32, 0, 127),
            chaos_mask: int("Chaos modules", c.chaos_mask as i32, 0, 127),
            seed: int("Chaos seed", c.seed as i32, 1, 65535),
            midi_channel: int("MIDI channel", c.midi_channel as i32, 0, 16),
            p0: BoolParam::new(PAD_NAMES[0], false),
            n0: int("Pad 1 MIDI note", c.midi_notes[0] as i32, 0, 127),
            p1: BoolParam::new(PAD_NAMES[1], false),
            n1: int("Pad 2 MIDI note", c.midi_notes[1] as i32, 0, 127),
            p2: BoolParam::new(PAD_NAMES[2], false),
            n2: int("Pad 3 MIDI note", c.midi_notes[2] as i32, 0, 127),
            p3: BoolParam::new(PAD_NAMES[3], false),
            n3: int("Pad 4 MIDI note", c.midi_notes[3] as i32, 0, 127),
            p4: BoolParam::new(PAD_NAMES[4], false),
            n4: int("Pad 5 MIDI note", c.midi_notes[4] as i32, 0, 127),
            p5: BoolParam::new(PAD_NAMES[5], false),
            n5: int("Pad 6 MIDI note", c.midi_notes[5] as i32, 0, 127),
            p6: BoolParam::new(PAD_NAMES[6], false),
            n6: int("Pad 7 MIDI note", c.midi_notes[6] as i32, 0, 127),
            p7: BoolParam::new(PAD_NAMES[7], false),
            n7: int("Pad 8 MIDI note", c.midi_notes[7] as i32, 0, 127),
            a0: int("Pad 1 module", c.pad_assign[0].id() as i32, 0, 8)
                .with_value_to_string(Arc::new(|i| Effect::ALL[i as usize].name().into())),
            a1: int("Pad 2 module", c.pad_assign[1].id() as i32, 0, 8)
                .with_value_to_string(Arc::new(|i| Effect::ALL[i as usize].name().into())),
            a2: int("Pad 3 module", c.pad_assign[2].id() as i32, 0, 8)
                .with_value_to_string(Arc::new(|i| Effect::ALL[i as usize].name().into())),
            a3: int("Pad 4 module", c.pad_assign[3].id() as i32, 0, 8)
                .with_value_to_string(Arc::new(|i| Effect::ALL[i as usize].name().into())),
            a4: int("Pad 5 module", c.pad_assign[4].id() as i32, 0, 8)
                .with_value_to_string(Arc::new(|i| Effect::ALL[i as usize].name().into())),
            a5: int("Pad 6 module", c.pad_assign[5].id() as i32, 0, 8)
                .with_value_to_string(Arc::new(|i| Effect::ALL[i as usize].name().into())),
            a6: int("Pad 7 module", c.pad_assign[6].id() as i32, 0, 8)
                .with_value_to_string(Arc::new(|i| Effect::ALL[i as usize].name().into())),
            priority0: int("Repeat priority", c.priority[0] as i32, 0, 4),
            priority1: int("Stutter priority", c.priority[1] as i32, 0, 4),
            priority2: int("Reverse priority", c.priority[2] as i32, 0, 4),
            priority3: int("Stop priority", c.priority[3] as i32, 0, 4),
            priority4: int("Dive priority", c.priority[4] as i32, 0, 4),
            cc0: int("MIDI X CC", c.midi_cc[0] as i32, 0, 119),
            cc1: int("MIDI Y CC", c.midi_cc[1] as i32, 0, 119),
            r0_enabled: BoolParam::new("Route 1 enabled", c.routes[0].enabled),
            r0_source: int(
                "Route 1 source",
                Source::ALL
                    .iter()
                    .position(|v| *v == c.routes[0].source)
                    .unwrap() as i32,
                0,
                2,
            ),
            r0_target: int(
                "Route 1 target",
                Target::ALL
                    .iter()
                    .position(|v| *v == c.routes[0].target)
                    .unwrap() as i32,
                0,
                11,
            ),
            r0_start: float("Route 1 start", c.routes[0].start, -4000., 4000., ""),
            r0_end: float("Route 1 end", c.routes[0].end, -4000., 4000., ""),
            r0_curve: float("Route 1 curve", c.routes[0].curve, 0.2, 5., ""),
            r1_enabled: BoolParam::new("Route 2 enabled", c.routes[1].enabled),
            r1_source: int(
                "Route 2 source",
                Source::ALL
                    .iter()
                    .position(|v| *v == c.routes[1].source)
                    .unwrap() as i32,
                0,
                2,
            ),
            r1_target: int(
                "Route 2 target",
                Target::ALL
                    .iter()
                    .position(|v| *v == c.routes[1].target)
                    .unwrap() as i32,
                0,
                11,
            ),
            r1_start: float("Route 2 start", c.routes[1].start, -4000., 4000., ""),
            r1_end: float("Route 2 end", c.routes[1].end, -4000., 4000., ""),
            r1_curve: float("Route 2 curve", c.routes[1].curve, 0.2, 5., ""),
            r2_enabled: BoolParam::new("Route 3 enabled", c.routes[2].enabled),
            r2_source: int(
                "Route 3 source",
                Source::ALL
                    .iter()
                    .position(|v| *v == c.routes[2].source)
                    .unwrap() as i32,
                0,
                2,
            ),
            r2_target: int(
                "Route 3 target",
                Target::ALL
                    .iter()
                    .position(|v| *v == c.routes[2].target)
                    .unwrap() as i32,
                0,
                11,
            ),
            r2_start: float("Route 3 start", c.routes[2].start, -4000., 4000., ""),
            r2_end: float("Route 3 end", c.routes[2].end, -4000., 4000., ""),
            r2_curve: float("Route 3 curve", c.routes[2].curve, 0.2, 5., ""),
            r3_enabled: BoolParam::new("Route 4 enabled", c.routes[3].enabled),
            r3_source: int(
                "Route 4 source",
                Source::ALL
                    .iter()
                    .position(|v| *v == c.routes[3].source)
                    .unwrap() as i32,
                0,
                2,
            ),
            r3_target: int(
                "Route 4 target",
                Target::ALL
                    .iter()
                    .position(|v| *v == c.routes[3].target)
                    .unwrap() as i32,
                0,
                11,
            ),
            r3_start: float("Route 4 start", c.routes[3].start, -4000., 4000., ""),
            r3_end: float("Route 4 end", c.routes[3].end, -4000., 4000., ""),
            r3_curve: float("Route 4 curve", c.routes[3].curve, 0.2, 5., ""),
            r4_enabled: BoolParam::new("Route 5 enabled", c.routes[4].enabled),
            r4_source: int(
                "Route 5 source",
                Source::ALL
                    .iter()
                    .position(|v| *v == c.routes[4].source)
                    .unwrap() as i32,
                0,
                2,
            ),
            r4_target: int(
                "Route 5 target",
                Target::ALL
                    .iter()
                    .position(|v| *v == c.routes[4].target)
                    .unwrap() as i32,
                0,
                11,
            ),
            r4_start: float("Route 5 start", c.routes[4].start, -4000., 4000., ""),
            r4_end: float("Route 5 end", c.routes[4].end, -4000., 4000., ""),
            r4_curve: float("Route 5 curve", c.routes[4].curve, 0.2, 5., ""),
            r5_enabled: BoolParam::new("Route 6 enabled", c.routes[5].enabled),
            r5_source: int(
                "Route 6 source",
                Source::ALL
                    .iter()
                    .position(|v| *v == c.routes[5].source)
                    .unwrap() as i32,
                0,
                2,
            ),
            r5_target: int(
                "Route 6 target",
                Target::ALL
                    .iter()
                    .position(|v| *v == c.routes[5].target)
                    .unwrap() as i32,
                0,
                11,
            ),
            r5_start: float("Route 6 start", c.routes[5].start, -4000., 4000., ""),
            r5_end: float("Route 6 end", c.routes[5].end, -4000., 4000., ""),
            r5_curve: float("Route 6 curve", c.routes[5].curve, 0.2, 5., ""),
            r6_enabled: BoolParam::new("Route 7 enabled", c.routes[6].enabled),
            r6_source: int(
                "Route 7 source",
                Source::ALL
                    .iter()
                    .position(|v| *v == c.routes[6].source)
                    .unwrap() as i32,
                0,
                2,
            ),
            r6_target: int(
                "Route 7 target",
                Target::ALL
                    .iter()
                    .position(|v| *v == c.routes[6].target)
                    .unwrap() as i32,
                0,
                11,
            ),
            r6_start: float("Route 7 start", c.routes[6].start, -4000., 4000., ""),
            r6_end: float("Route 7 end", c.routes[6].end, -4000., 4000., ""),
            r6_curve: float("Route 7 curve", c.routes[6].curve, 0.2, 5., ""),
            r7_enabled: BoolParam::new("Route 8 enabled", c.routes[7].enabled),
            r7_source: int(
                "Route 8 source",
                Source::ALL
                    .iter()
                    .position(|v| *v == c.routes[7].source)
                    .unwrap() as i32,
                0,
                2,
            ),
            r7_target: int(
                "Route 8 target",
                Target::ALL
                    .iter()
                    .position(|v| *v == c.routes[7].target)
                    .unwrap() as i32,
                0,
                11,
            ),
            r7_start: float("Route 8 start", c.routes[7].start, -4000., 4000., ""),
            r7_end: float("Route 8 end", c.routes[7].end, -4000., 4000., ""),
            r7_curve: float("Route 8 curve", c.routes[7].curve, 0.2, 5., ""),
        }
    }
}
impl SpasmParams {
    pub fn pads(&self) -> [&BoolParam; 8] {
        [
            &self.p0, &self.p1, &self.p2, &self.p3, &self.p4, &self.p5, &self.p6, &self.p7,
        ]
    }
    pub fn notes(&self) -> [&IntParam; 8] {
        [
            &self.n0, &self.n1, &self.n2, &self.n3, &self.n4, &self.n5, &self.n6, &self.n7,
        ]
    }
    pub fn assignments(&self) -> [&IntParam; 7] {
        [
            &self.a0, &self.a1, &self.a2, &self.a3, &self.a4, &self.a5, &self.a6,
        ]
    }
    pub fn priorities(&self) -> [&IntParam; 5] {
        [
            &self.priority0,
            &self.priority1,
            &self.priority2,
            &self.priority3,
            &self.priority4,
        ]
    }
    pub fn ccs(&self) -> [&IntParam; 2] {
        [&self.cc0, &self.cc1]
    }
    pub fn routes(&self) -> [RouteRef<'_>; 8] {
        [
            RouteRef {
                enabled: &self.r0_enabled,
                source: &self.r0_source,
                target: &self.r0_target,
                start: &self.r0_start,
                end: &self.r0_end,
                curve: &self.r0_curve,
            },
            RouteRef {
                enabled: &self.r1_enabled,
                source: &self.r1_source,
                target: &self.r1_target,
                start: &self.r1_start,
                end: &self.r1_end,
                curve: &self.r1_curve,
            },
            RouteRef {
                enabled: &self.r2_enabled,
                source: &self.r2_source,
                target: &self.r2_target,
                start: &self.r2_start,
                end: &self.r2_end,
                curve: &self.r2_curve,
            },
            RouteRef {
                enabled: &self.r3_enabled,
                source: &self.r3_source,
                target: &self.r3_target,
                start: &self.r3_start,
                end: &self.r3_end,
                curve: &self.r3_curve,
            },
            RouteRef {
                enabled: &self.r4_enabled,
                source: &self.r4_source,
                target: &self.r4_target,
                start: &self.r4_start,
                end: &self.r4_end,
                curve: &self.r4_curve,
            },
            RouteRef {
                enabled: &self.r5_enabled,
                source: &self.r5_source,
                target: &self.r5_target,
                start: &self.r5_start,
                end: &self.r5_end,
                curve: &self.r5_curve,
            },
            RouteRef {
                enabled: &self.r6_enabled,
                source: &self.r6_source,
                target: &self.r6_target,
                start: &self.r6_start,
                end: &self.r6_end,
                curve: &self.r6_curve,
            },
            RouteRef {
                enabled: &self.r7_enabled,
                source: &self.r7_source,
                target: &self.r7_target,
                start: &self.r7_start,
                end: &self.r7_end,
                curve: &self.r7_curve,
            },
        ]
    }
    pub fn gates(&self) -> [bool; 8] {
        self.pads().map(|p| p.value())
    }
    pub fn base(&self) -> Config {
        Config {
            history_seconds: self.history_seconds.value(),
            stutter_ms: self.stutter_ms.value(),
            acceleration: self.acceleration.value(),
            stop_ms: self.stop_ms.value(),
            stop_beats: self.stop_beats.value(),
            stop_curve: self.stop_curve.value(),
            dive_ms: self.dive_ms.value(),
            dive_semitones: self.dive_semitones.value(),
            pitch_semitones: self.pitch_semitones.value(),
            gate_depth: self.gate_depth.value(),
            gate_attack_ms: self.gate_attack_ms.value(),
            gate_release_ms: self.gate_release_ms.value(),
            bits: self.bits.value(),
            reduction: self.reduction.value(),
            filter_hz: self.filter_hz.value(),
            drive: self.drive.value(),
            wander: self.wander.value(),
            mix: self.mix.value(),
            fx_output_db: self.fx_output_db.value(),
            crossfade_ms: self.crossfade_ms.value(),
            edge_ms: self.edge_ms.value(),
            x: self.x.value(),
            y: self.y.value(),
            capture_locked: self.capture_locked.value(),
            stutter_reverse: self.stutter_reverse.value(),
            stop_sync: self.stop_sync.value(),
            filter_enabled: self.filter_enabled.value(),
            latch: self.latch.value(),
            reset_clear: self.reset_clear.value(),
            auto_damage: self.auto_damage.value(),
            bypass: self.bypass.value(),
            repeat_div: self.repeat_div.value() as u8,
            stutter_div: self.stutter_div.value() as u8,
            reverse_div: self.reverse_div.value() as u8,
            gate_div: self.gate_div.value() as u8,
            chaos_div: self.chaos_div.value() as u8,
            quantize: self.quantize.value() as u8,
            repeat_count: self.repeat_count.value() as u8,
            gate_pattern: self.gate_pattern.value() as u16,
            latch_mask: self.latch_mask.value() as u8,
            chaos_mask: self.chaos_mask.value() as u16,
            seed: self.seed.value() as u16,
            midi_channel: self.midi_channel.value() as u8,
            routes: self.routes().map(|r| r.value()),
            pad_assign: self.assignments().map(|p| Effect::ALL[p.value() as usize]),
            midi_notes: self.notes().map(|p| p.value() as u8),
            priority: self.priorities().map(|p| p.value() as u8),
            midi_cc: self.ccs().map(|p| p.value() as u8),
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
        c
    }
    pub fn document(&self) -> Preset {
        Preset {
            version: 1,
            name: self.name.get(),
            description: self.notes.get(),
            config: self.config(),
        }
    }
    pub fn metadata_changed(&self, s: &ParamSetter) {
        s.begin_set_parameter(&self.metadata);
        s.set_parameter(&self.metadata, !self.metadata.value());
        s.end_set_parameter(&self.metadata);
    }
    pub fn set_metadata(&self, name: &str, notes: &str, s: &ParamSetter) {
        self.name.set_text(name);
        self.notes.set_text(notes);
        self.metadata_changed(s);
    }
    pub fn apply(&self, p: &Preset, index: i32, s: &ParamSetter) {
        let c = p.config;
        self.midi.clear();
        self.set_metadata(&p.name, &p.description, s);
        for p in self.pads() {
            s.begin_set_parameter(p);
            s.set_parameter(p, false);
            s.end_set_parameter(p);
        }
        s.begin_set_parameter(&self.history_seconds);
        s.set_parameter(&self.history_seconds, c.history_seconds);
        s.end_set_parameter(&self.history_seconds);
        s.begin_set_parameter(&self.stutter_ms);
        s.set_parameter(&self.stutter_ms, c.stutter_ms);
        s.end_set_parameter(&self.stutter_ms);
        s.begin_set_parameter(&self.acceleration);
        s.set_parameter(&self.acceleration, c.acceleration);
        s.end_set_parameter(&self.acceleration);
        s.begin_set_parameter(&self.stop_ms);
        s.set_parameter(&self.stop_ms, c.stop_ms);
        s.end_set_parameter(&self.stop_ms);
        s.begin_set_parameter(&self.stop_beats);
        s.set_parameter(&self.stop_beats, c.stop_beats);
        s.end_set_parameter(&self.stop_beats);
        s.begin_set_parameter(&self.stop_curve);
        s.set_parameter(&self.stop_curve, c.stop_curve);
        s.end_set_parameter(&self.stop_curve);
        s.begin_set_parameter(&self.dive_ms);
        s.set_parameter(&self.dive_ms, c.dive_ms);
        s.end_set_parameter(&self.dive_ms);
        s.begin_set_parameter(&self.dive_semitones);
        s.set_parameter(&self.dive_semitones, c.dive_semitones);
        s.end_set_parameter(&self.dive_semitones);
        s.begin_set_parameter(&self.pitch_semitones);
        s.set_parameter(&self.pitch_semitones, c.pitch_semitones);
        s.end_set_parameter(&self.pitch_semitones);
        s.begin_set_parameter(&self.gate_depth);
        s.set_parameter(&self.gate_depth, c.gate_depth);
        s.end_set_parameter(&self.gate_depth);
        s.begin_set_parameter(&self.gate_attack_ms);
        s.set_parameter(&self.gate_attack_ms, c.gate_attack_ms);
        s.end_set_parameter(&self.gate_attack_ms);
        s.begin_set_parameter(&self.gate_release_ms);
        s.set_parameter(&self.gate_release_ms, c.gate_release_ms);
        s.end_set_parameter(&self.gate_release_ms);
        s.begin_set_parameter(&self.bits);
        s.set_parameter(&self.bits, c.bits);
        s.end_set_parameter(&self.bits);
        s.begin_set_parameter(&self.reduction);
        s.set_parameter(&self.reduction, c.reduction);
        s.end_set_parameter(&self.reduction);
        s.begin_set_parameter(&self.filter_hz);
        s.set_parameter(&self.filter_hz, c.filter_hz);
        s.end_set_parameter(&self.filter_hz);
        s.begin_set_parameter(&self.drive);
        s.set_parameter(&self.drive, c.drive);
        s.end_set_parameter(&self.drive);
        s.begin_set_parameter(&self.wander);
        s.set_parameter(&self.wander, c.wander);
        s.end_set_parameter(&self.wander);
        s.begin_set_parameter(&self.mix);
        s.set_parameter(&self.mix, c.mix);
        s.end_set_parameter(&self.mix);
        s.begin_set_parameter(&self.fx_output_db);
        s.set_parameter(&self.fx_output_db, c.fx_output_db);
        s.end_set_parameter(&self.fx_output_db);
        s.begin_set_parameter(&self.crossfade_ms);
        s.set_parameter(&self.crossfade_ms, c.crossfade_ms);
        s.end_set_parameter(&self.crossfade_ms);
        s.begin_set_parameter(&self.edge_ms);
        s.set_parameter(&self.edge_ms, c.edge_ms);
        s.end_set_parameter(&self.edge_ms);
        s.begin_set_parameter(&self.x);
        s.set_parameter(&self.x, c.x);
        s.end_set_parameter(&self.x);
        s.begin_set_parameter(&self.y);
        s.set_parameter(&self.y, c.y);
        s.end_set_parameter(&self.y);
        s.begin_set_parameter(&self.capture_locked);
        s.set_parameter(&self.capture_locked, c.capture_locked);
        s.end_set_parameter(&self.capture_locked);
        s.begin_set_parameter(&self.stutter_reverse);
        s.set_parameter(&self.stutter_reverse, c.stutter_reverse);
        s.end_set_parameter(&self.stutter_reverse);
        s.begin_set_parameter(&self.stop_sync);
        s.set_parameter(&self.stop_sync, c.stop_sync);
        s.end_set_parameter(&self.stop_sync);
        s.begin_set_parameter(&self.filter_enabled);
        s.set_parameter(&self.filter_enabled, c.filter_enabled);
        s.end_set_parameter(&self.filter_enabled);
        s.begin_set_parameter(&self.latch);
        s.set_parameter(&self.latch, c.latch);
        s.end_set_parameter(&self.latch);
        s.begin_set_parameter(&self.reset_clear);
        s.set_parameter(&self.reset_clear, c.reset_clear);
        s.end_set_parameter(&self.reset_clear);
        s.begin_set_parameter(&self.auto_damage);
        s.set_parameter(&self.auto_damage, c.auto_damage);
        s.end_set_parameter(&self.auto_damage);
        s.begin_set_parameter(&self.bypass);
        s.set_parameter(&self.bypass, c.bypass);
        s.end_set_parameter(&self.bypass);
        s.begin_set_parameter(&self.repeat_div);
        s.set_parameter(&self.repeat_div, c.repeat_div as i32);
        s.end_set_parameter(&self.repeat_div);
        s.begin_set_parameter(&self.stutter_div);
        s.set_parameter(&self.stutter_div, c.stutter_div as i32);
        s.end_set_parameter(&self.stutter_div);
        s.begin_set_parameter(&self.reverse_div);
        s.set_parameter(&self.reverse_div, c.reverse_div as i32);
        s.end_set_parameter(&self.reverse_div);
        s.begin_set_parameter(&self.gate_div);
        s.set_parameter(&self.gate_div, c.gate_div as i32);
        s.end_set_parameter(&self.gate_div);
        s.begin_set_parameter(&self.chaos_div);
        s.set_parameter(&self.chaos_div, c.chaos_div as i32);
        s.end_set_parameter(&self.chaos_div);
        s.begin_set_parameter(&self.quantize);
        s.set_parameter(&self.quantize, c.quantize as i32);
        s.end_set_parameter(&self.quantize);
        s.begin_set_parameter(&self.repeat_count);
        s.set_parameter(&self.repeat_count, c.repeat_count as i32);
        s.end_set_parameter(&self.repeat_count);
        s.begin_set_parameter(&self.gate_pattern);
        s.set_parameter(&self.gate_pattern, c.gate_pattern as i32);
        s.end_set_parameter(&self.gate_pattern);
        s.begin_set_parameter(&self.latch_mask);
        s.set_parameter(&self.latch_mask, c.latch_mask as i32);
        s.end_set_parameter(&self.latch_mask);
        s.begin_set_parameter(&self.chaos_mask);
        s.set_parameter(&self.chaos_mask, c.chaos_mask as i32);
        s.end_set_parameter(&self.chaos_mask);
        s.begin_set_parameter(&self.seed);
        s.set_parameter(&self.seed, c.seed as i32);
        s.end_set_parameter(&self.seed);
        s.begin_set_parameter(&self.midi_channel);
        s.set_parameter(&self.midi_channel, c.midi_channel as i32);
        s.end_set_parameter(&self.midi_channel);
        for (i, p) in self.notes().iter().enumerate() {
            s.begin_set_parameter(*p);
            s.set_parameter(*p, c.midi_notes[i] as i32);
            s.end_set_parameter(*p);
        }
        for (i, p) in self.assignments().iter().enumerate() {
            s.begin_set_parameter(*p);
            s.set_parameter(*p, c.pad_assign[i].id() as i32);
            s.end_set_parameter(*p);
        }
        for (i, p) in self.priorities().iter().enumerate() {
            s.begin_set_parameter(*p);
            s.set_parameter(*p, c.priority[i] as i32);
            s.end_set_parameter(*p);
        }
        for (i, p) in self.ccs().iter().enumerate() {
            s.begin_set_parameter(*p);
            s.set_parameter(*p, c.midi_cc[i] as i32);
            s.end_set_parameter(*p);
        }
        for (i, r) in self.routes().iter().enumerate() {
            let v = c.routes[i];
            s.begin_set_parameter(r.enabled);
            s.set_parameter(r.enabled, v.enabled);
            s.end_set_parameter(r.enabled);
            s.begin_set_parameter(r.source);
            s.set_parameter(
                r.source,
                Source::ALL.iter().position(|x| *x == v.source).unwrap() as i32,
            );
            s.end_set_parameter(r.source);
            s.begin_set_parameter(r.target);
            s.set_parameter(
                r.target,
                Target::ALL.iter().position(|x| *x == v.target).unwrap() as i32,
            );
            s.end_set_parameter(r.target);
            s.begin_set_parameter(r.start);
            s.set_parameter(r.start, v.start);
            s.end_set_parameter(r.start);
            s.begin_set_parameter(r.end);
            s.set_parameter(r.end, v.end);
            s.end_set_parameter(r.end);
            s.begin_set_parameter(r.curve);
            s.set_parameter(r.curve, v.curve);
            s.end_set_parameter(r.curve);
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
