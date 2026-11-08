// SPDX-License-Identifier: GPL-3.0-or-later
mod edit;
mod editor;
mod fields;
mod library;
mod params;
mod shared;
use nih_plug::prelude::*;
use nih_plug::wrapper::state::ParamValue;
use nih_plug_egui::create_egui_editor;
use params::SpasmParams;
use shared::{Shared, WaveWriter};
use spasm_core::{Engine, Event, Transport};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU8, Ordering},
};
pub struct Spasm {
    params: Arc<SpasmParams>,
    engine: Engine,
    shared: Arc<Shared>,
    wave: WaveWriter,
    gui_held: Arc<[AtomicBool; 8]>,
    gui_wake: Arc<AtomicU8>,
    gui_released: [bool; 8],
    last_gates: [bool; 8],
    last_axes: [f32; 2],
    revision: bool,
}
impl Default for Spasm {
    fn default() -> Self {
        let p = Arc::new(SpasmParams::default());
        Self {
            engine: Engine::new(48000., p.config()),
            last_axes: [p.x.value(), p.y.value()],
            revision: p.revision.value(),
            params: p,
            shared: Arc::new(Shared::default()),
            wave: WaveWriter::default(),
            gui_held: Arc::new(std::array::from_fn(|_| AtomicBool::new(false))),
            gui_wake: Arc::new(AtomicU8::new(0)),
            gui_released: [false; 8],
            last_gates: [false; 8],
        }
    }
}
impl Plugin for Spasm {
    const NAME: &'static str = "SPASM";
    const VENDOR: &'static str = "VOIDMUTT";
    const URL: &'static str = "https://github.com/khodges42/VSTs";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];
    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    type SysExMessage = ();
    type BackgroundTask = ();
    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }
    fn editor(&mut self, _: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let p = self.params.clone();
        let shared = self.shared.clone();
        let held = self.gui_held.clone();
        let wake = self.gui_wake.clone();
        create_egui_editor(
            p.editor.clone(),
            editor::UiState::default(),
            editor::setup,
            move |ctx, s, state| editor::draw(ctx, s, state, &p, &shared, &held, &wake),
        )
    }
    fn filter_state(state: &mut PluginState) {
        for i in 1..=8 {
            let id = match i {
                1 => "pad-1",
                2 => "pad-2",
                3 => "pad-3",
                4 => "pad-4",
                5 => "pad-5",
                6 => "pad-6",
                7 => "pad-7",
                _ => "pad-8",
            };
            if let Some(p) = state.params.get_mut(id) {
                *p = ParamValue::Bool(false);
            }
        }
    }
    fn initialize(
        &mut self,
        _: &AudioIOLayout,
        c: &BufferConfig,
        ctx: &mut impl InitContext<Self>,
    ) -> bool {
        if !(8000. ..=192000.).contains(&c.sample_rate)
            || self.params.schema.load(Ordering::Relaxed) != 1
        {
            return false;
        }
        if self.engine.sample_rate() != c.sample_rate {
            self.engine = Engine::new(c.sample_rate, self.params.config());
        }
        self.revision = self.params.revision.value();
        self.last_axes = [self.params.x.value(), self.params.y.value()];
        self.last_gates = [false; 8];
        self.gui_released = [false; 8];
        ctx.set_latency_samples(0);
        true
    }
    fn reset(&mut self) {
        self.engine.set_config(self.params.config());
        self.engine.reset();
        self.last_gates = [false; 8];
        self.gui_released = [false; 8];
        self.revision = self.params.revision.value();
    }
    fn process(
        &mut self,
        buffer: &mut Buffer,
        _: &mut AuxiliaryBuffers,
        ctx: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let axes = [self.params.x.value(), self.params.y.value()];
        for (i, v) in axes.iter().enumerate() {
            if *v != self.last_axes[i] {
                self.params.midi.set_value(i, -1.);
            }
        }
        self.last_axes = axes;
        self.engine.set_config(self.params.config());
        if self.params.revision.value() != self.revision {
            self.engine.reset_performance(false);
            self.engine.event(Event::AllNotesOff);
            self.revision = self.params.revision.value();
        }
        let mut gates = self.params.gates();
        let wake = self.gui_wake.swap(0, Ordering::Relaxed);
        for (i, v) in gates.iter_mut().enumerate() {
            if *v != self.last_gates[i] || wake & (1 << i) != 0 {
                self.gui_released[i] = false;
            }
            self.last_gates[i] = *v;
            if !self.params.editor.is_open() && self.gui_held[i].swap(false, Ordering::Relaxed) {
                self.gui_released[i] = true;
            }
            if self.gui_released[i] {
                *v = false;
            }
        }
        self.engine.set_gates(gates);
        let t = ctx.transport();
        self.engine.set_transport(Transport {
            bpm: t.tempo,
            beat: t.pos_beats(),
            sample: t.pos_samples(),
            playing: Some(t.playing),
        });
        let mut event = ctx.next_event();
        let rate = self.engine.sample_rate();
        let history = self.params.history_seconds.value();
        for (i, mut frame) in buffer.iter_samples().enumerate() {
            while let Some(e) = event {
                if e.timing() > i as u32 {
                    break;
                }
                match e {
                    NoteEvent::NoteOn {
                        note,
                        channel,
                        velocity,
                        ..
                    } => self.engine.event(Event::Note {
                        note,
                        channel,
                        on: true,
                        velocity,
                    }),
                    NoteEvent::NoteOff { note, channel, .. } => self.engine.event(Event::Note {
                        note,
                        channel,
                        on: false,
                        velocity: 0.,
                    }),
                    NoteEvent::Choke { .. } => self.engine.event(Event::AllNotesOff),
                    NoteEvent::MidiCC {
                        cc, value, channel, ..
                    } => {
                        let c = self.params.base();
                        if c.midi_channel != 0 && c.midi_channel != channel + 1 {
                            // The selected input channel applies to notes and CCs.
                        } else if let Some(axis) = c.midi_cc.iter().position(|n| *n == cc) {
                            self.params.midi.set_value(axis, value.clamp(0., 1.));
                            self.engine.set_config(self.params.config());
                        } else if cc == 120 || cc == 123 {
                            self.engine.event(Event::AllNotesOff);
                        }
                    }
                    _ => {}
                }
                event = ctx.next_event();
            }
            let x = [*frame.get_mut(0).unwrap(), *frame.get_mut(1).unwrap()];
            let y = self.engine.process(x);
            self.wave
                .push(&self.shared, x, rate, history, self.engine.status().epoch);
            *frame.get_mut(0).unwrap() = y[0];
            *frame.get_mut(1).unwrap() = y[1];
        }
        let status = self.engine.status();
        self.shared.publish(status);
        if status.engaged != 0 || status.pending != 0 {
            ProcessStatus::KeepAlive
        } else {
            ProcessStatus::Tail((rate * 0.03) as u32)
        }
    }
}
impl Vst3Plugin for Spasm {
    const VST3_CLASS_ID: [u8; 16] = *b"VOIDMUTTSPAS0001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Modulation,
        Vst3SubCategory::Distortion,
    ];
}
nih_export_vst3!(Spasm);
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_ids_and_fixed_state_restore() {
        let p = SpasmParams::default();
        assert_eq!(p.config(), spasm_core::factory(0).config);
        let ids: Vec<_> = p.param_map().into_iter().map(|v| v.0).collect();
        let distinct: std::collections::BTreeSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), distinct.len());
        p.name.set_text("USER FRACTURE");
        p.midi.set_value(0, 0.73);
        p.edit.store(true, Ordering::Relaxed);
        let fields = p.serialize_fields();
        let q = SpasmParams::default();
        let info = allocation_counter::measure(|| q.deserialize_fields(&fields));
        assert_eq!(info.count_total, 0, "{info:?}");
        assert_eq!(info.count_current, 0);
        assert_eq!(q.name.get(), "USER FRACTURE");
        assert_eq!(q.config().x, 0.73);
        assert!(q.edit.load(Ordering::Relaxed));
    }
    #[test]
    fn atomic_publication_is_bounded_without_heap_work() {
        let s = Shared::default();
        let mut w = WaveWriter::default();
        let info = allocation_counter::measure(|| {
            for _ in 0..2000 {
                w.push(&s, [0.2, -0.1], 8000., 1., 0);
            }
            s.publish(spasm_core::Status::default());
        });
        assert_eq!(info.count_total, 0);
        assert_eq!(info.count_current, 0);
        assert!(s.valid.load(Ordering::Relaxed) > 0);
    }
    #[test]
    fn state_filter_releases_pads_and_view_switch_preserves_audio() {
        let p = SpasmParams::default();
        let before = p.config();
        p.edit.store(true, Ordering::Relaxed);
        assert_eq!(p.config(), before);
        p.edit.store(false, Ordering::Relaxed);
        assert_eq!(p.config(), before);
        let mut state = PluginState {
            version: "0.1.0".into(),
            params: std::collections::BTreeMap::new(),
            fields: std::collections::BTreeMap::new(),
        };
        for i in 1..=8 {
            state
                .params
                .insert(format!("pad-{i}"), ParamValue::Bool(true));
        }
        let info = allocation_counter::measure(|| Spasm::filter_state(&mut state));
        assert_eq!(info.count_total, 0);
        assert_eq!(info.count_current, 0);
        assert!(
            state
                .params
                .values()
                .all(|v| matches!(v, ParamValue::Bool(false)))
        );
    }
}
