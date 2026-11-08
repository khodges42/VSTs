// SPDX-License-Identifier: GPL-3.0-or-later
use spasm_core::{
    Config, Effect, Engine, Event, Preset, Transport,
    buffer::{Bank, Slice},
    factory,
    patch::Route,
};
fn basic() -> Config {
    Config {
        quantize: 0,
        auto_damage: false,
        filter_enabled: false,
        routes: [Route::default(); 8],
        edge_ms: 0.1,
        ..Config::default()
    }
}
fn pad(e: &mut Engine, pad: u8, down: bool) {
    e.event(Event::Pad {
        pad,
        down,
        velocity: 1.,
    });
}
fn fill(e: &mut Engine, frames: usize) {
    for i in 0..frames {
        e.process([(i as f32 * 0.13).sin() * 0.2, (i as f32 * 0.07).sin() * 0.1]);
    }
}
#[test]
fn block_partitioning_and_deterministic_renders_match_with_tempo_and_triggers() {
    fn render(block_size: usize) -> Vec<[f32; 2]> {
        let mut c = factory(6).config;
        c.capture_locked = false;
        let mut e = Engine::new(8000., c);
        let mut output = Vec::new();
        let mut start = 0;
        while start < 32000 {
            let n = block_size.min(32000 - start).min(if start < 16000 {
                16000 - start
            } else {
                32000 - start
            });
            let bpm = if start < 16000 { 120. } else { 173. };
            let beat = if start < 16000 {
                start as f64 / 4000.
            } else {
                4. + (start - 16000) as f64 * 173. / 480000.
            };
            e.set_config(c);
            e.set_transport(Transport {
                bpm: Some(bpm),
                beat: Some(beat),
                sample: Some(start as i64),
                playing: Some(true),
            });
            for i in start..start + n {
                match i {
                    1737 => pad(&mut e, 0, true),
                    4551 => pad(&mut e, 1, true),
                    6713 => pad(&mut e, 1, false),
                    8119 => {
                        c.x = 0.9;
                        c.y = 0.7;
                        e.set_config(c);
                    }
                    12345 => pad(&mut e, 6, true),
                    19321 => pad(&mut e, 3, true),
                    21231 => e.event(Event::Reset),
                    23013 => pad(&mut e, 0, false),
                    25057 => {
                        pad(&mut e, 6, false);
                        pad(&mut e, 0, true);
                    }
                    29013 => pad(&mut e, 0, false),
                    _ => {}
                }
                output.push(e.process([
                    (i as f32 * 0.13).sin() * 0.3,
                    (i as f32 * 0.071).sin() * 0.2,
                ]));
            }
            start += n;
        }
        output
    }
    let reference = render(1);
    for n in [17, 64, 127, 256, 1024] {
        let actual = render(n);
        if let Some(i) = actual.iter().zip(&reference).position(|(a, b)| a != b) {
            panic!(
                "block {n}, first {i}, actual {:?}, expected {:?}",
                actual[i], reference[i]
            );
        }
    }
}
#[test]
fn gate_pattern_crush_transfer_and_clean_release_are_audible_and_bounded() {
    let mut c = basic();
    c.gate_pattern = 1;
    c.gate_div = 3;
    c.bits = 4.;
    c.reduction = 8.;
    c.crossfade_ms = 2.;
    let mut e = Engine::new(8000., c);
    pad(&mut e, 4, true);
    for _ in 0..100 {
        e.process([0.5, 0.25]);
    }
    assert!(e.process([0.5, 0.25])[0] > 0.49);
    for _ in 0..1500 {
        e.process([0.5, 0.25]);
    }
    assert!(e.process([0.5, 0.25])[0] < 1e-5);
    e.event(Event::Reset);
    for _ in 0..100 {
        e.process([0.137, 0.137]);
    }
    pad(&mut e, 5, true);
    for _ in 0..100 {
        e.process([0.137, 0.137]);
    }
    assert!((e.process([0.137, 0.137])[0] - 0.125).abs() < 1e-6);
    pad(&mut e, 5, false);
    for _ in 0..100 {
        e.process([0.137, 0.137]);
    }
    assert_eq!(e.process([0.137, 0.137]), [0.137; 2]);
}
#[test]
fn silence_all_modules_and_cold_chaos_never_leak_uninitialized_audio() {
    for effect in Effect::ALL {
        let mut c = basic();
        c.pad_assign[0] = effect;
        c.chaos_mask = 127;
        c.drive = 1.;
        c.bits = 4.;
        c.reduction = 64.;
        c.wander = 1.;
        let mut e = Engine::new(8000., c);
        pad(&mut e, 0, true);
        for _ in 0..10000 {
            assert_eq!(e.process([0.; 2]), [0.; 2]);
        }
    }
    let mut c = basic();
    c.pad_assign[0] = Effect::Chaos;
    c.chaos_mask = Effect::Reverse.bit();
    let mut e = Engine::new(8000., c);
    pad(&mut e, 0, true);
    for _ in 0..31 {
        assert_eq!(e.process([0.2, -0.2]), [0.2, -0.2]);
    }
    assert_ne!(e.status().pending, 0);
    e.process([0.2, -0.2]);
    assert_eq!(e.status().pending, 0);
}
#[test]
fn reset_history_clear_and_finite_repeat_require_a_fresh_press() {
    let mut c = basic();
    c.reset_clear = true;
    c.repeat_count = 2;
    c.repeat_div = 4;
    let mut e = Engine::new(8000., c);
    fill(&mut e, 8000);
    pad(&mut e, 0, true);
    fill(&mut e, 1200);
    assert_eq!(e.status().engaged, 0);
    fill(&mut e, 100);
    assert_eq!(e.status().effect, Effect::Live);
    pad(&mut e, 0, false);
    pad(&mut e, 0, true);
    fill(&mut e, 100);
    assert_eq!(e.status().effect, Effect::Repeat);
    let epoch = e.status().epoch;
    e.event(Event::Reset);
    fill(&mut e, 100);
    assert!(e.status().epoch > epoch);
    assert!(e.status().history_seconds < 0.02);
    assert_eq!(e.status().pending, 0);
    assert_eq!(e.status().latched, 0);
}
#[test]
fn mapped_pitch_smooths_to_musical_endpoints_and_reset_crossfades_cleanly() {
    use spasm_core::patch::{Source, Target};
    let mut c = basic();
    c.routes[0] = Route {
        enabled: true,
        source: Source::X,
        target: Target::Pitch,
        start: -12.,
        end: 12.,
        curve: 2.,
    };
    let mut e = Engine::new(8000., c);
    fill(&mut e, 8000);
    pad(&mut e, 0, true);
    fill(&mut e, 500);
    assert!((e.status().speed - 0.5).abs() < 0.001);
    c.x = 1.;
    e.set_config(c);
    e.process([0.; 2]);
    assert!(e.status().speed < 0.52);
    fill(&mut e, 2000);
    assert!((e.status().speed - 2.).abs() < 0.001);
    // With no captured signal and a constant live source, RESET is an actual fade,
    // followed by exact unity, rather than a discontinuous switch.
    let mut e = Engine::new(8000., basic());
    for _ in 0..8000 {
        e.process([0.; 2]);
    }
    pad(&mut e, 0, true);
    for _ in 0..100 {
        e.process([0.5; 2]);
    }
    e.event(Event::Reset);
    let mut last = 0f32;
    let mut max_step = 0f32;
    for _ in 0..100 {
        let y = e.process([0.5; 2])[0];
        max_step = max_step.max((y - last).abs());
        last = y;
    }
    assert!(max_step < 0.04, "{max_step}");
    assert_eq!(last, 0.5);
}
#[test]
fn missing_invalid_tempo_uses_continuous_free_clock_and_zero_memory_growth() {
    let mut e = Engine::new(8000., basic());
    let bytes = e.memory_bytes();
    e.set_transport(Transport {
        bpm: Some(f64::NAN),
        ..Default::default()
    });
    fill(&mut e, 4000);
    assert!(!e.status().host_sync);
    assert!((e.status().beat - 1.).abs() < 0.001);
    assert_eq!(e.status().bpm, 120.);
    e.set_transport(Transport {
        bpm: Some(200.),
        ..Default::default()
    });
    fill(&mut e, 2400);
    assert!((e.status().beat - 2.).abs() < 0.001);
    e.set_transport(Transport {
        bpm: Some(-1.),
        beat: Some(f64::INFINITY),
        ..Default::default()
    });
    fill(&mut e, 2400);
    assert_eq!(e.status().bpm, 200.);
    assert!((e.status().beat - 3.).abs() < 0.001);
    assert_eq!(e.memory_bytes(), bytes);
}
#[test]
fn bank_wrap_interpolation_and_invalid_reads_are_bounded() {
    let mut b = Bank::new(32);
    for i in 0..100 {
        b.write([i as f32, -(i as f32)]);
    }
    let s = b.slice(0, 32, 100);
    assert_eq!(s.start, 68);
    assert_eq!(b.read(s, 0.), [68., -68.]);
    assert_eq!(b.read(s, 31.), [99., -99.]);
    assert_eq!(b.read(s, 1.5), [69.5, -69.5]);
    assert_eq!(
        b.read(
            Slice {
                start: u64::MAX,
                len: 32,
                ..s
            },
            0.
        ),
        [0.; 2]
    );
    assert_eq!(b.read(s, f64::NAN), [0.; 2]);
    b.reset();
    assert_eq!(b.read(s, 1.), [0.; 2]);
}
#[test]
fn idle_and_bypass_have_exact_unity_and_zero_latency() {
    for bypass in [false, true] {
        let mut c = basic();
        c.bypass = bypass;
        c.y = 1.;
        c.fx_output_db = -12.;
        let mut e = Engine::new(8000., c);
        for i in 0..1000 {
            let x = [(i as f32 * 0.31).sin() * 2., -0.5];
            assert_eq!(e.process(x), x);
        }
    }
}
#[test]
fn captured_repeat_is_periodic_while_new_input_is_recorded() {
    let mut e = Engine::new(8000., basic());
    fill(&mut e, 2000);
    pad(&mut e, 0, true);
    let a: Vec<_> = (0..2200).map(|_| e.process([0.; 2])).collect();
    for i in 100..950 {
        assert!((a[i][0] - a[i + 1000][0]).abs() < 1e-6);
    }
    assert!(a[100..950].iter().any(|x| x[0].abs() > 0.05));
    assert_eq!(e.status().effect, Effect::Repeat);
    assert!((e.status().repeat_seconds - 0.125).abs() < 1e-6);
    assert!(e.status().history_seconds > 0.2);
}
#[test]
fn wrapped_history_repeats_a_stereo_impulse_at_exact_offsets_without_new_input_leakage() {
    let mut c = basic();
    c.history_seconds = 8.;
    c.repeat_div = 3;
    let mut e = Engine::new(8000., c);
    // More than one complete prepared bank, then capture its most recent window.
    for frame in 0..70000 {
        e.process(if frame == 69500 { [0.4, 0.1] } else { [0.; 2] });
    }
    pad(&mut e, 0, true);
    let mut onsets = Vec::new();
    for frame in 0..3000 {
        let y = e.process(if frame == 1003 { [0.8, -0.4] } else { [0.; 2] });
        if y[0] > 0.2 {
            onsets.push(frame);
            assert_eq!(y, [0.4, 0.1]);
        } else {
            assert_eq!(y, [0.; 2]);
        }
    }
    assert_eq!(onsets, [499, 1499, 2499]);
}
#[test]
fn reverse_reads_only_the_captured_past() {
    let mut e = Engine::new(8000., basic());
    for i in 0..2000 {
        e.process([i as f32 * 0.0001, 0.]);
    }
    pad(&mut e, 1, true);
    for i in 0..800 {
        let y = e.process([0.; 2]);
        if i > 50 {
            assert!(
                (y[0] - (2000 - i) as f32 * 0.0001).abs() < 2e-6,
                "{i}: {}",
                y[0]
            );
        }
    }
    assert!(e.status().reverse);
}
#[test]
fn musical_divisions_and_mid_block_quantization_are_exact() {
    for sr in [8000, 44100, 48000, 96000] {
        let mut c = basic();
        c.quantize = 3;
        let mut e = Engine::new(sr as f32, c);
        fill(&mut e, 2000);
        e.set_transport(Transport {
            bpm: Some(120.),
            beat: Some(0.),
            sample: Some(0),
            playing: Some(true),
        });
        let due = (sr as f64 / 8.).ceil() as usize;
        for frame in 0..=due {
            if frame == 37 {
                pad(&mut e, 0, true);
            }
            e.process([0.1, 0.1]);
            if frame < due {
                assert_eq!(e.status().engaged, 0);
            } else {
                assert_eq!(e.status().engaged & 1, 1);
            }
        }
    }
}
#[test]
fn pending_release_cancels_and_tempo_changes_retime_in_beats() {
    let mut c = basic();
    c.quantize = 1;
    let mut e = Engine::new(8000., c);
    fill(&mut e, 2000);
    e.set_transport(Transport {
        bpm: Some(120.),
        beat: Some(0.),
        sample: Some(0),
        playing: Some(true),
    });
    pad(&mut e, 0, true);
    e.process([0.; 2]);
    pad(&mut e, 0, false);
    fill(&mut e, 5000);
    assert_eq!(e.status().engaged, 0);
    e.reset();
    fill(&mut e, 2000);
    e.set_transport(Transport {
        bpm: Some(120.),
        beat: Some(0.),
        sample: Some(0),
        playing: Some(true),
    });
    for i in 0..=2500 {
        if i == 37 {
            pad(&mut e, 0, true);
        }
        if i == 1000 {
            e.set_transport(Transport {
                bpm: Some(240.),
                beat: Some(0.25),
                sample: Some(1000),
                playing: Some(true),
            });
        }
        e.process([0.1, 0.1]);
        if i < 2500 {
            assert_eq!(e.status().engaged, 0);
        } else {
            assert_eq!(e.status().engaged & 1, 1);
        }
    }
}
#[test]
fn latch_priority_release_and_emergency_reset_are_unambiguous() {
    let mut c = basic();
    c.latch = true;
    let mut e = Engine::new(8000., c);
    fill(&mut e, 3000);
    pad(&mut e, 0, true);
    fill(&mut e, 100);
    pad(&mut e, 0, false);
    fill(&mut e, 100);
    assert_eq!(e.status().effect, Effect::Repeat);
    assert_eq!(e.status().latched & 1, 1);
    pad(&mut e, 1, true);
    fill(&mut e, 100);
    assert_eq!(e.status().effect, Effect::Reverse);
    pad(&mut e, 2, true);
    fill(&mut e, 100);
    assert_eq!(e.status().effect, Effect::Stop);
    e.event(Event::Reset);
    for _ in 0..200 {
        e.process([0.25, -0.1]);
    }
    assert_eq!(e.process([0.25, -0.1]), [0.25, -0.1]);
    assert_eq!(e.status().engaged, 0);
    assert_eq!(e.status().pending, 0);
    fill(&mut e, 100);
    assert_eq!(e.status().engaged, 0);
    pad(&mut e, 0, false);
    pad(&mut e, 0, true);
    fill(&mut e, 100);
    assert_eq!(e.status().effect, Effect::Repeat);
}
#[test]
fn cold_capture_waits_without_reading_uninitialized_audio() {
    let mut e = Engine::new(8000., basic());
    pad(&mut e, 0, true);
    for i in 0..31 {
        let x = [i as f32 * 0.001, 0.];
        assert_eq!(e.process(x), x);
        assert_eq!(e.status().engaged, 0);
    }
    e.process([0.1, 0.]);
    assert_eq!(e.status().engaged & 1, 1);
    assert!(e.status().short_history);
}
#[test]
fn frozen_capture_survives_more_than_a_full_bank_of_new_audio() {
    let mut e = Engine::new(8000., basic());
    fill(&mut e, 2000);
    pad(&mut e, 0, true);
    fill(&mut e, 100);
    let mut power = 0_f64;
    for i in 0..72000 {
        let y = e.process([0.; 2]);
        if i > 64000 {
            power += (y[0] as f64).powi(2);
        }
    }
    assert!(power > 1.);
}
#[test]
fn midi_sources_balance_overlaps_and_host_gates_are_independent() {
    let c = basic();
    let mut e = Engine::new(8000., c);
    fill(&mut e, 3000);
    for _ in 0..2 {
        e.event(Event::Note {
            note: 36,
            channel: 0,
            on: true,
            velocity: 0.8,
        });
    }
    fill(&mut e, 100);
    e.event(Event::Note {
        note: 36,
        channel: 0,
        on: false,
        velocity: 0.,
    });
    fill(&mut e, 100);
    assert_eq!(e.status().effect, Effect::Repeat);
    pad(&mut e, 0, true);
    e.event(Event::Note {
        note: 36,
        channel: 0,
        on: false,
        velocity: 0.,
    });
    fill(&mut e, 100);
    assert_eq!(e.status().effect, Effect::Repeat);
    pad(&mut e, 0, false);
    fill(&mut e, 100);
    assert_eq!(e.status().effect, Effect::Live);
}
#[test]
fn seek_cancels_old_performance_and_stopped_positions_do_not_reset_repeatedly() {
    let mut e = Engine::new(8000., basic());
    e.set_transport(Transport {
        bpm: Some(120.),
        beat: Some(0.),
        sample: Some(0),
        playing: Some(true),
    });
    fill(&mut e, 3000);
    pad(&mut e, 0, true);
    fill(&mut e, 100);
    e.set_transport(Transport {
        bpm: Some(120.),
        beat: Some(16.),
        sample: Some(64000),
        playing: Some(true),
    });
    fill(&mut e, 100);
    assert_eq!(e.status().engaged, 0);
    assert!(e.status().history_seconds < 0.02);
    e.set_transport(Transport {
        bpm: Some(120.),
        beat: Some(16.),
        sample: Some(64000),
        playing: Some(false),
    });
    for _ in 0..10 {
        fill(&mut e, 256);
        e.set_transport(Transport {
            bpm: Some(120.),
            beat: Some(16.),
            sample: Some(64000),
            playing: Some(false),
        });
    }
    assert!(e.status().history_seconds > 0.25);
    assert!(!e.status().host_sync);
}
#[test]
fn stop_and_dive_change_actual_playback_speed() {
    let mut c = basic();
    c.stop_sync = false;
    c.stop_ms = 200.;
    c.stop_curve = 1.;
    c.dive_ms = 200.;
    let mut e = Engine::new(8000., c);
    fill(&mut e, 8000);
    pad(&mut e, 2, true);
    fill(&mut e, 400);
    let a = e.status().speed;
    fill(&mut e, 800);
    let b = e.status().speed;
    assert!(a > 0.7 && b < 0.3);
    fill(&mut e, 800);
    assert_eq!(e.status().speed, 0.);
    pad(&mut e, 2, false);
    fill(&mut e, 100);
    pad(&mut e, 3, true);
    fill(&mut e, 1800);
    assert!((e.status().speed - 0.25).abs() < 0.001);
}
#[test]
fn rapid_changes_reset_and_capture_switches_allocate_nothing() {
    let mut e = Engine::new(8000., basic());
    let bank: Vec<_> = (0..10).map(|i| factory(i).config).collect();
    fill(&mut e, 2000);
    let info = allocation_counter::measure(|| {
        for i in 0..10000 {
            if i % 127 == 0 {
                let mut c = bank[(i / 127) % 10];
                c.quantize = 0;
                c.capture_locked = i % 2 == 0;
                c.x = (i % 101) as f32 / 100.;
                c.y = (i % 97) as f32 / 96.;
                e.set_config(c);
                pad(&mut e, ((i / 127) % 7) as u8, true);
            }
            if i % 127 == 63 {
                e.event(Event::Reset);
            }
            if i == 5000 {
                e.reset();
            }
            let y = e.process([if i % 199 == 0 { f32::NAN } else { 0.2 }, -0.1]);
            assert!(y.iter().all(|v| v.is_finite() && v.abs() <= 1.));
        }
    });
    assert_eq!(info.count_total, 0, "{info:?}");
    assert_eq!(info.count_current, 0);
}
#[test]
fn presets_roundtrip_and_reject_unsafe_or_unknown_data() {
    for i in 0..10 {
        let p = factory(i);
        p.validate().unwrap();
        assert_eq!(Preset::from_json(&p.to_json().unwrap()).unwrap(), p);
    }
    let mut p = factory(0);
    p.version = 2;
    assert!(p.validate().is_err());
    p.version = 1;
    p.config.midi_notes[0] = p.config.midi_notes[1];
    assert!(p.validate().is_err());
    p.config = basic();
    p.config.routes[0].end = f32::NAN;
    assert!(p.validate().is_err());
    assert!(Preset::from_json(&" ".repeat(32769)).is_err());
}
