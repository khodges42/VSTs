// SPDX-License-Identifier: GPL-3.0-or-later
mod analysis;
mod editor;
mod fields;
mod params;
use analysis::{Shared, Writer};
use eis_core::{Engine, LATENCY};
use nih_plug::prelude::*;
use nih_plug_egui::create_egui_editor;
use params::EisParams;
use std::sync::{Arc, atomic::Ordering};
pub struct Eis {
    params: Arc<EisParams>,
    engine: Engine,
    shared: Arc<Shared>,
    writer: Writer,
}
impl Default for Eis {
    fn default() -> Self {
        let p = Arc::new(EisParams::default());
        Self {
            engine: Engine::new(48000., p.config()),
            params: p,
            shared: Arc::new(Shared::default()),
            writer: Writer::default(),
        }
    }
}
impl Plugin for Eis {
    const NAME: &'static str = "8 INCH STEAK";
    const VENDOR: &'static str = "Sweaty Ferguson Audio Systems";
    const URL: &'static str = "https://github.com/khodges42/VSTs";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;
    type SysExMessage = ();
    type BackgroundTask = ();
    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }
    fn editor(&mut self, _: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let p = self.params.clone();
        let shared = self.shared.clone();
        create_egui_editor(
            p.editor.clone(),
            editor::UiState::default(),
            editor::setup,
            move |ctx, s, state| editor::draw(ctx, s, state, &p, &shared),
        )
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
        self.shared
            .rate
            .store(c.sample_rate.to_bits(), Ordering::Relaxed);
        ctx.set_latency_samples(LATENCY as u32);
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
        _: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        self.engine.set_config(self.params.config());
        let analyze = self.params.editor.is_open();
        for mut frame in buffer.iter_samples() {
            let x = [*frame.get_mut(0).unwrap(), *frame.get_mut(1).unwrap()];
            if analyze {
                self.writer.push(&self.shared, x);
            }
            let y = self.engine.process(x);
            *frame.get_mut(0).unwrap() = y[0];
            *frame.get_mut(1).unwrap() = y[1];
        }
        self.shared.publish(self.engine.meter());
        ProcessStatus::Tail((self.engine.sample_rate() * 0.5) as u32 + LATENCY as u32)
    }
}
impl Vst3Plugin for Eis {
    const VST3_CLASS_ID: [u8; 16] = *b"SWEATYEISFX00001";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}
nih_export_vst3!(Eis);
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_and_parameter_ids_are_stable() {
        let p = EisParams::default();
        assert_eq!(p.config(), eis_core::factory(0).config);
        let ids: Vec<_> = p.param_map().into_iter().map(|v| v.0).collect();
        let unique: std::collections::BTreeSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len());
        assert!(ids.iter().any(|id| id == "b3-up-ratio"));
        let before = p.config();
        let fields = p.serialize_fields();
        let q = EisParams::default();
        q.deserialize_fields(&fields);
        assert_eq!(q.config(), before);
    }
    #[test]
    fn fixed_fields_restore_without_heap_work() {
        let p = EisParams::default();
        p.name.set_text("USER STEAK");
        p.notes.set_text("Original band settings");
        p.butcher.store(true, Ordering::Relaxed);
        let fields = p.serialize_fields();
        let q = EisParams::default();
        let info = allocation_counter::measure(|| q.deserialize_fields(&fields));
        assert_eq!(info.count_total, 0, "{info:?}");
        assert_eq!(info.count_current, 0);
        assert_eq!(q.name.get(), "USER STEAK");
        assert!(q.butcher.load(Ordering::Relaxed));
    }
    #[test]
    fn gui_mode_changes_cannot_change_audio_configuration() {
        let p = EisParams::default();
        let c = p.config();
        let mut a = Engine::new(8000., c);
        let mut b = Engine::new(8000., c);
        for i in 0..4096 {
            if i % 63 == 0 {
                p.butcher.store(i % 2 == 0, Ordering::Relaxed);
                a.set_config(p.config());
            }
            let x = [(i as f32 * 0.17).sin() * 0.2, 0.1];
            assert_eq!(a.process(x), b.process(x));
        }
        assert_eq!(p.config(), c);
    }
    #[test]
    fn analysis_detects_antiphase_and_publishes_without_allocation() {
        let s = Shared::default();
        let mut w = Writer::default();
        let info = allocation_counter::measure(|| {
            for i in 0..1024 {
                let x = (std::f32::consts::TAU * 32. * i as f32 / 1024.).sin() * 0.5;
                w.push(&s, [x, -x]);
            }
            s.publish(eis_core::Meter::default());
        });
        assert_eq!(info.count_total, 0);
        let (_, x) = s.snapshot().unwrap();
        let bins = analysis::spectrum(x);
        assert!((bins[32] + 6.0206).abs() < 0.03);
        assert!(bins[40] < -60.);
    }
}
