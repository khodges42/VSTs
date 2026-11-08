// SPDX-License-Identifier: GPL-3.0-or-later
mod editor;
mod store;
use crossbeam_queue::ArrayQueue;
use nih_plug::prelude::*;
use nih_plug_egui::{EguiState, create_egui_editor};
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};
use store::PatchStore;
use vixen_core::{Controls, Engine, Event, factory};

#[derive(Params)]
pub struct VixenParams {
    #[persist = "editor"]
    pub editor: Arc<EguiState>,
    #[persist = "patch-v1"]
    pub patch: PatchStore,
    #[persist = "midi-x"]
    pub midi_x: Arc<AtomicF32>,
    #[persist = "midi-y"]
    pub midi_y: Arc<AtomicF32>,
    #[id = "xy-x"]
    pub x: FloatParam,
    #[id = "xy-y"]
    pub y: FloatParam,
    #[id = "brightness"]
    pub cutoff: FloatParam,
    #[id = "resonance"]
    pub resonance: FloatParam,
    #[id = "drive"]
    pub drive: FloatParam,
    #[id = "character"]
    pub decay: FloatParam,
    #[id = "glide"]
    pub glide: FloatParam,
    #[id = "output"]
    pub output: FloatParam,
    // Host dirty notification for edits to the persistent patch document.
    #[id = "patch-change"]
    pub change: BoolParam,
}
fn param(name: &str, default: f32, min: f32, max: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min, max })
}
impl Default for VixenParams {
    fn default() -> Self {
        Self {
            editor: EguiState::from_size(1040, 740),
            patch: PatchStore::default(),
            midi_x: Arc::new(AtomicF32::new(-1.)),
            midi_y: Arc::new(AtomicF32::new(-1.)),
            x: param("X", 0.5, 0., 1.),
            y: param("Y", 0.5, 0., 1.),
            cutoff: param("Brightness", 0., -1., 1.),
            resonance: param("Resonance", 0., -1., 1.),
            drive: param("Drive", 0., -1., 1.),
            decay: param("Envelope character", 0., -1., 1.),
            glide: param("Glide", 0., 0., 2.),
            output: param("Output", 0.7, 0., 1.),
            change: BoolParam::new("Patch document changed", false)
                .non_automatable()
                .hide(),
        }
    }
}
impl VixenParams {
    pub fn xy(&self) -> [f32; 2] {
        let x = self.midi_x.load(Ordering::Relaxed);
        let y = self.midi_y.load(Ordering::Relaxed);
        [
            if (0. ..=1.).contains(&x) {
                x
            } else {
                self.x.value()
            },
            if (0. ..=1.).contains(&y) {
                y
            } else {
                self.y.value()
            },
        ]
    }
    pub fn controls(&self) -> Controls {
        let [x, y] = self.xy();
        Controls {
            x,
            y,
            cutoff: self.cutoff.value(),
            resonance: self.resonance.value(),
            drive: self.drive.value(),
            decay: self.decay.value(),
            glide: self.glide.value(),
            output: self.output.value(),
        }
    }
}
pub struct Vixen {
    params: Arc<VixenParams>,
    engine: Engine,
    preview: Arc<ArrayQueue<Event>>,
    preview_notes: [bool; 128],
    host_xy: [f32; 2],
    meter: Arc<AtomicU32>,
}
impl Default for Vixen {
    fn default() -> Self {
        Self {
            params: Arc::new(VixenParams::default()),
            engine: Engine::new(48000., factory::patch(0).synth),
            preview: Arc::new(ArrayQueue::new(64)),
            preview_notes: [false; 128],
            host_xy: [0.5; 2],
            meter: Arc::new(AtomicU32::new(0)),
        }
    }
}
impl Plugin for Vixen {
    const NAME: &'static str = "VIXEN";
    const VENDOR: &'static str = "VOIDMUTT";
    const URL: &'static str = "https://github.com/khodges42/VSTs";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
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
        let params = self.params.clone();
        let preview = self.preview.clone();
        let meter = self.meter.clone();
        create_egui_editor(
            self.params.editor.clone(),
            editor::UiState::new(),
            editor::setup,
            move |ctx, setter, state| editor::draw(ctx, setter, state, &params, &preview, &meter),
        )
    }
    fn initialize(
        &mut self,
        _: &AudioIOLayout,
        config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        if !(8000. ..=192000.).contains(&config.sample_rate) {
            return false;
        }
        // A same-rate state restore does not reconstruct FX buffers.
        if self.engine.sample_rate() != config.sample_rate {
            self.engine = Engine::new(config.sample_rate, *self.engine.synth());
        }
        self.host_xy = [self.params.x.value(), self.params.y.value()];
        context.set_latency_samples(vixen_core::dsp::LATENCY_SAMPLES);
        true
    }
    fn reset(&mut self) {
        self.engine.set_controls(self.params.controls());
        self.engine.reset();
        self.preview_notes.fill(false);
        for _ in 0..64 {
            if self.preview.pop().is_none() {
                break;
            }
        }
    }
    fn process(
        &mut self,
        buffer: &mut Buffer,
        _: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Bounded patch publication; copied values only, no Arc destruction on DSP.
        let mut newest = None;
        for _ in 0..8 {
            if let Some(p) = self.params.patch.pop() {
                newest = Some(p);
            } else {
                break;
            }
        }
        if let Some(s) = newest {
            self.engine.switch_patch(s);
            self.preview_notes.fill(false);
        }
        for (i, (param, midi)) in [
            (&self.params.x, &self.params.midi_x),
            (&self.params.y, &self.params.midi_y),
        ]
        .into_iter()
        .enumerate()
        {
            let v = param.value();
            if v != self.host_xy[i] {
                midi.store(-1., Ordering::Relaxed);
                self.host_xy[i] = v;
            }
        }
        self.engine.set_controls(self.params.controls());
        for _ in 0..64 {
            let Some(e) = self.preview.pop() else {
                break;
            };
            match e {
                Event::NoteOn { note, .. } => self.preview_notes[note as usize] = true,
                Event::NoteOff { note, .. } => self.preview_notes[note as usize] = false,
                _ => {}
            }
            self.engine.event(e);
        }
        if !self.params.editor.is_open() {
            for (note, active) in self.preview_notes.iter_mut().enumerate() {
                if *active {
                    self.engine.event(Event::NoteOff {
                        note: note as u8,
                        channel: 15,
                        id: Some(-(note as i32) - 1),
                    });
                    *active = false;
                }
            }
        }
        let bpm = context.transport().tempo.unwrap_or(120.) as f32;
        let mut event = context.next_event();
        let mut peak = 0_f32;
        for (i, samples) in buffer.iter_samples().enumerate() {
            while let Some(e) = event {
                if e.timing() > i as u32 {
                    break;
                }
                match e {
                    NoteEvent::NoteOn {
                        note,
                        channel,
                        velocity,
                        voice_id,
                        ..
                    } => self.engine.event(Event::NoteOn {
                        note,
                        channel,
                        velocity,
                        id: voice_id,
                    }),
                    NoteEvent::NoteOff {
                        note,
                        channel,
                        voice_id,
                        ..
                    }
                    | NoteEvent::Choke {
                        note,
                        channel,
                        voice_id,
                        ..
                    } => self.engine.event(Event::NoteOff {
                        note,
                        channel,
                        id: voice_id,
                    }),
                    NoteEvent::MidiPitchBend { channel, value, .. } => {
                        self.engine.event(Event::Bend {
                            channel,
                            value: value * 2. - 1.,
                        })
                    }
                    NoteEvent::MidiCC {
                        channel, cc, value, ..
                    } => {
                        if cc == self.engine.synth().x_cc {
                            self.params.midi_x.store(value, Ordering::Relaxed);
                            self.engine.set_controls(self.params.controls());
                        } else if cc == self.engine.synth().y_cc {
                            self.params.midi_y.store(value, Ordering::Relaxed);
                            self.engine.set_controls(self.params.controls());
                        } else {
                            self.engine.event(Event::Cc { channel, cc, value });
                        }
                    }
                    _ => {}
                }
                event = context.next_event();
            }
            let output = self.engine.next(bpm);
            for (ch, sample) in samples.into_iter().enumerate() {
                *sample = output[ch];
                peak = peak.max(sample.abs());
            }
        }
        self.meter.store(peak.to_bits(), Ordering::Relaxed);
        ProcessStatus::KeepAlive
    }
}
impl Vst3Plugin for Vixen {
    const VST3_CLASS_ID: [u8; 16] = *b"VOIDMUTTVIXEN001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth];
}
nih_export_vst3!(Vixen);

#[cfg(test)]
mod tests {
    use super::*;
    use nih_plug::params::persist::PersistentField;
    #[test]
    fn user_patch_and_midi_xy_survive_actual_params_persistence() {
        let a = VixenParams::default();
        let mut user = factory::patch(2);
        user.name = "MY DIRTY REESE".into();
        user.synth.oscillators[0].detune = 0.71;
        user.synth.mappings[0].amount = -0.8;
        user.controls.x = 0.12;
        a.patch.publish(user.clone()).unwrap();
        a.midi_x.store(0.73, Ordering::Relaxed);
        a.midi_y.store(0.21, Ordering::Relaxed);
        let fields = a.serialize_fields();
        let b = VixenParams::default();
        b.deserialize_fields(&fields);
        assert_eq!(b.patch.read(), user);
        assert_eq!(b.patch.pop(), Some(user.synth));
        assert_eq!(b.xy(), [0.73, 0.21]);
        // Snapshotting stores the saved document, without consuming pending DSP updates.
        assert_eq!(b.serialize_fields(), fields);
    }
    #[test]
    fn invalid_host_patch_preserves_last_valid_synth_and_reports_error() {
        let a = VixenParams::default();
        let before = a.patch.read();
        let mut invalid = before.clone();
        invalid.version = 100;
        PersistentField::set(&a.patch, invalid);
        assert_eq!(a.patch.read(), before);
        assert!(a.patch.pop().is_none());
        assert!(a.patch.error().contains("Unsupported patch version"));
    }
    #[test]
    fn queue_is_bounded_and_latest_patch_wins() {
        let store = PatchStore::default();
        let mut last = factory::patch(0);
        for i in 0..100 {
            last = factory::patch(i % 10);
            last.synth.gain = i as f32 / 100.;
            store.publish(last.clone()).unwrap();
        }
        let mut newest = None;
        let mut count = 0;
        while let Some(p) = store.pop() {
            newest = Some(p);
            count += 1;
        }
        assert_eq!(count, 8);
        assert_eq!(newest, Some(last.synth));
        assert_eq!(store.read(), last);
    }
}
