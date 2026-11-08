// SPDX-License-Identifier: GPL-3.0-or-later
mod editor;
mod fields;
mod params;
use echogirl_core::{Config, Engine, Transport};
use nih_plug::prelude::*;
use nih_plug::wrapper::state::ParamValue;
use nih_plug_egui::create_egui_editor;
use params::EchoParams;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

pub struct Echogirl {
    params: Arc<EchoParams>,
    engine: Engine,
    last_base: [f32; 8],
    meter: Arc<AtomicU32>,
    ui_gates: Arc<[AtomicBool; 3]>,
}
fn base_values(c: Config) -> [f32; 8] {
    [
        c.x,
        c.y,
        c.throw as u8 as f32,
        c.hold as u8 as f32,
        c.kill as u8 as f32,
        c.dub as u8 as f32,
        c.feed as u8 as f32,
        c.bypass as u8 as f32,
    ]
}
impl Default for Echogirl {
    fn default() -> Self {
        let p = Arc::new(EchoParams::default());
        Self {
            last_base: base_values(p.base()),
            engine: Engine::new(48000., p.config()),
            params: p,
            meter: Arc::new(AtomicU32::new(0)),
            ui_gates: Arc::new(std::array::from_fn(|_| AtomicBool::new(false))),
        }
    }
}
impl Plugin for Echogirl {
    const NAME: &'static str = "ECHOGIRL";
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
        let meter = self.meter.clone();
        let gates = self.ui_gates.clone();
        create_egui_editor(
            p.editor.clone(),
            editor::UiState::default(),
            editor::setup,
            move |ctx, setter, state| editor::draw(ctx, setter, state, &p, &meter, &gates),
        )
    }
    fn filter_state(state: &mut PluginState) {
        for id in ["throw", "hold", "kill", "dub"] {
            if let Some(value) = state.params.get_mut(id) {
                *value = ParamValue::Bool(false);
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
        self.last_base = base_values(self.params.base());
        ctx.set_latency_samples(0);
        true
    }
    fn reset(&mut self) {
        self.engine.set_config(self.params.config());
        self.engine.reset();
    }
    fn process(
        &mut self,
        buffer: &mut Buffer,
        _: &mut AuxiliaryBuffers,
        ctx: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let bases = base_values(self.params.base());
        for (i, v) in bases.iter().enumerate() {
            if *v != self.last_base[i] {
                self.params.midi.set_value(i, -1.);
            }
        }
        if bases[2] >= 0.5 && self.last_base[2] < 0.5 {
            self.params.midi.set_value(6, 0.);
        }
        if bases[4] >= 0.5 && self.last_base[4] < 0.5 {
            self.params.midi.set_value(3, 0.);
            self.params.midi.set_value(2, 0.);
        }
        self.last_base = bases;
        if !self.params.editor.is_open() {
            for (i, index) in [2, 4, 5].iter().enumerate() {
                if self.ui_gates[i].swap(false, Ordering::Relaxed) {
                    self.params.midi.set_value(*index, 0.);
                }
            }
        }
        self.engine.set_config(self.params.config());
        let t = ctx.transport();
        self.engine.set_transport(Transport {
            bpm: t.tempo.unwrap_or(120.) as f32,
            playing: t.playing,
            beat: t.pos_beats(),
        });
        let mut event = ctx.next_event();
        let mut peak = 0_f32;
        for (i, mut frame) in buffer.iter_samples().enumerate() {
            while let Some(e) = event {
                if e.timing() > i as u32 {
                    break;
                }
                if let NoteEvent::MidiCC { cc, value, .. } = e {
                    let c = self.params.base();
                    if let Some(index) = c.midi_cc.iter().position(|n| *n == cc) {
                        self.params.midi.set_value(index, value.clamp(0., 1.));
                        if index == 2 && value >= 0.5 {
                            self.params.midi.set_value(6, 0.);
                        }
                        if index == 4 && value >= 0.5 {
                            self.params.midi.set_value(3, 0.);
                            self.params.midi.set_value(2, 0.);
                        }
                        self.engine.set_config(self.params.config());
                    }
                }
                event = ctx.next_event();
            }
            let input = [*frame.get_mut(0).unwrap(), *frame.get_mut(1).unwrap()];
            let output = self.engine.process(input);
            *frame.get_mut(0).unwrap() = output[0];
            *frame.get_mut(1).unwrap() = output[1];
            peak = peak.max(output[0].abs()).max(output[1].abs());
        }
        self.meter.store(peak.to_bits(), Ordering::Relaxed);
        ProcessStatus::KeepAlive
    }
}
impl Vst3Plugin for Echogirl {
    const VST3_CLASS_ID: [u8; 16] = *b"VOIDMUTTECHO0001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Delay];
}
nih_export_vst3!(Echogirl);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_is_the_factory_sound_and_parameter_ids_are_unique() {
        let p = EchoParams::default();
        assert_eq!(p.config(), echogirl_core::factory(0).config);
        let ids: Vec<_> = p.param_map().into_iter().map(|v| v.0).collect();
        let unique: std::collections::BTreeSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len());
        assert!(ids.iter().any(|id| id == "route-depth_8"));
    }
    #[test]
    fn valid_fixed_state_restoration_allocates_nothing() {
        let a = EchoParams::default();
        a.name.set_text("CUSTOM TAPE");
        a.notes.set_text("User notes\nSecond line");
        a.midi.set_value(0, 0.76);
        a.midi.set_value(1, 0.32);
        a.midi.set_value(2, 1.);
        a.midi.set_value(6, 0.);
        let fields = a.serialize_fields();
        let b = EchoParams::default();
        let info = allocation_counter::measure(|| b.deserialize_fields(&fields));
        assert_eq!(info.count_total, 0, "{info:?}");
        assert_eq!(info.count_current, 0);
        assert_eq!(b.name.get(), "CUSTOM TAPE");
        assert_eq!(b.notes.get(), "User notes\nSecond line");
        assert_eq!(b.midi.read()[0], 0.76);
        assert_eq!(b.midi.read()[2], -1.);
        assert_eq!(b.midi.read()[6], 0.);
    }
    #[test]
    fn state_filter_releases_performance_buttons_without_allocating() {
        let mut state = PluginState {
            version: "0.1.0".into(),
            params: std::collections::BTreeMap::new(),
            fields: std::collections::BTreeMap::new(),
        };
        for id in ["throw", "hold", "kill", "dub"] {
            state.params.insert(id.into(), ParamValue::Bool(true));
        }
        let info = allocation_counter::measure(|| Echogirl::filter_state(&mut state));
        assert_eq!(info.count_total, 0);
        for v in state.params.values() {
            assert!(matches!(v, ParamValue::Bool(false)));
        }
    }
}
