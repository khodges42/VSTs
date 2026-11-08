// SPDX-License-Identifier: GPL-3.0-or-later
use vixen_core::{Controls, Engine, Event, Patch, Synth, factory, patch::*};
fn on(note: u8, id: i32) -> Event {
    Event::NoteOn {
        note,
        channel: 0,
        velocity: 0.9,
        id: Some(id),
    }
}
fn off(note: u8, id: i32) -> Event {
    Event::NoteOff {
        note,
        channel: 0,
        id: Some(id),
    }
}
fn render(e: &mut Engine, n: usize) -> Vec<[f32; 2]> {
    (0..n).map(|_| e.next(145.)).collect()
}
fn energy(audio: &[[f32; 2]]) -> f64 {
    audio
        .iter()
        .flatten()
        .map(|v| (*v as f64).powi(2))
        .sum::<f64>()
        / (audio.len() * 2) as f64
}
fn clean_sine() -> Synth {
    let mut s = Synth::default();
    s.oscillators[0].wave = Wave::Sine;
    s.cutoff = 18000.;
    s.filter_amount = 0.;
    s.keytrack = 0.;
    s.amp.sustain = 1.;
    s.effects.reverb = 0.;
    s
}
#[test]
fn factory_patches_round_trip_and_reject_untrusted_input() {
    let mut notes = factory::patch(0);
    notes.description = "Line one\nLine two".into();
    assert_eq!(Patch::from_json(&notes.to_json().unwrap()).unwrap(), notes);
    for i in 0..10 {
        let p = factory::patch(i);
        p.validate().unwrap();
        assert_eq!(Patch::from_json(&p.to_json().unwrap()).unwrap(), p);
    }
    let mut p = factory::patch(0);
    p.version = 99;
    assert!(p.validate().is_err());
    p.version = 1;
    p.synth.oscillators[0].unison = 255;
    assert!(p.validate().is_err());
    p.synth.oscillators[0].unison = 1;
    p.synth.cutoff = f32::NAN;
    assert!(p.validate().is_err());
    assert!(Patch::from_json(&" ".repeat(MAX_PATCH_BYTES + 1)).is_err());
    let json = factory::patch(0).to_json().unwrap();
    assert!(Patch::from_json(&json.replace("\"version\": 1", "\"version\": 2")).is_err());
    assert!(
        Patch::from_json(&json.replace("\"version\": 1", "\"mystery\": 0, \"version\": 1"))
            .is_err()
    );
    let mut p = factory::patch(0);
    p.synth.y_cc = p.synth.x_cc;
    assert!(p.validate().is_err());
}
#[test]
fn deterministic_render_after_serialization() {
    for i in 0..10 {
        let p = factory::patch(i);
        let restored = Patch::from_json(&p.to_json().unwrap()).unwrap();
        let mut a = Engine::new(48000., p.synth);
        let mut b = Engine::new(48000., restored.synth);
        a.event(on(60, 1));
        b.event(on(60, 1));
        for _ in 0..4096 {
            assert_eq!(a.next(145.), b.next(145.));
        }
    }
}
#[test]
fn ten_distinct_audible_patches_and_xy_has_audible_effect() {
    let mut fingerprints = Vec::new();
    for i in 0..10 {
        let p = factory::patch(i);
        let mut a = Engine::new(48000., p.synth);
        a.event(on(60, 1));
        let center = render(&mut a, 16000);
        assert!(energy(&center) > 1e-6, "{} is inaudible", p.name);
        let fingerprint: Vec<_> = center.iter().step_by(83).copied().collect();
        assert!(
            !fingerprints.contains(&fingerprint),
            "Duplicate sound {}",
            p.name
        );
        fingerprints.push(fingerprint);
        let mut b = Engine::new(48000., p.synth);
        b.set_controls(Controls {
            x: 0.95,
            y: 0.95,
            ..Controls::default()
        });
        b.event(on(60, 1));
        let corner = render(&mut b, 16000);
        let difference: f64 = center
            .iter()
            .zip(&corner)
            .map(|(a, b)| ((a[0] - b[0]) as f64).powi(2))
            .sum();
        assert!(difference > 0.01, "{} has inert XY", p.name);
    }
}
#[test]
fn voice_ids_sustain_and_channel_isolation() {
    let mut s = clean_sine();
    s.amp.release = 0.01;
    let mut e = Engine::new(48000., s);
    e.event(on(60, 1));
    e.event(on(60, 2));
    render(&mut e, 1024);
    e.event(off(60, 1));
    render(&mut e, 1024);
    assert_eq!(e.active_voices(), 1);
    e.event(Event::Cc {
        channel: 0,
        cc: 64,
        value: 1.,
    });
    e.event(off(60, 2));
    render(&mut e, 1024);
    assert_eq!(e.active_voices(), 1);
    e.event(Event::Cc {
        channel: 1,
        cc: 64,
        value: 0.,
    });
    render(&mut e, 1024);
    assert_eq!(e.active_voices(), 1);
    e.event(Event::Cc {
        channel: 0,
        cc: 64,
        value: 0.,
    });
    render(&mut e, 1024);
    assert_eq!(e.active_voices(), 0);
}
#[test]
fn repeated_pitch_without_ids_releases_oldest_one_at_a_time() {
    let mut s = clean_sine();
    s.amp.release = 0.01;
    let mut e = Engine::new(48000., s);
    for _ in 0..3 {
        e.event(Event::NoteOn {
            note: 60,
            channel: 0,
            velocity: 1.,
            id: None,
        });
    }
    render(&mut e, 1024);
    e.event(Event::NoteOff {
        note: 60,
        channel: 0,
        id: None,
    });
    render(&mut e, 1024);
    assert_eq!(e.active_voices(), 2);
    e.event(Event::AllOff);
    render(&mut e, 1024);
    assert_eq!(e.active_voices(), 0);
}
#[test]
fn legato_last_note_priority_returns_to_held_note_without_releasing_envelope() {
    let mut s = clean_sine();
    s.voice_mode = VoiceMode::Legato;
    s.amp.release = 0.01;
    let mut e = Engine::new(48000., s);
    e.event(on(60, 1));
    render(&mut e, 1024);
    e.event(on(67, 2));
    render(&mut e, 1024);
    e.event(off(67, 2));
    let held = render(&mut e, 8000);
    assert_eq!(e.active_voices(), 1);
    assert!(energy(&held[4000..]) > 1e-5);
    e.event(off(60, 1));
    render(&mut e, 1024);
    assert_eq!(e.active_voices(), 0);
}
#[test]
fn sample_rate_independent_pitch_and_envelope_times() {
    for rate in [8000., 44100., 48000., 96000., 192000.] {
        let mut s = clean_sine();
        s.amp.attack = 0.01;
        s.amp.release = 0.02;
        let mut e = Engine::new(rate, s);
        e.event(on(69, 1));
        render(&mut e, (rate * 0.1) as usize);
        let a = render(&mut e, (rate * 0.25) as usize);
        let crosses = a
            .windows(2)
            .filter(|w| w[0][0] <= 0. && w[1][0] > 0.)
            .count();
        assert!(
            (crosses as f32 - 110.).abs() <= 1.,
            "{rate}: {crosses} crossings"
        );
        e.event(off(69, 1));
        render(&mut e, (rate * 0.025) as usize);
        assert_eq!(e.active_voices(), 0);
    }
}
#[test]
fn pitch_bend_and_velocity_are_musical_controls() {
    let mut s = clean_sine();
    s.bend_range = 12.;
    s.velocity = 1.;
    let mut a = Engine::new(48000., s);
    a.event(on(69, 1));
    a.event(Event::Bend {
        channel: 0,
        value: 1.,
    });
    render(&mut a, 4800);
    let bent = render(&mut a, 12000);
    let crossings = bent
        .windows(2)
        .filter(|w| w[0][0] <= 0. && w[1][0] > 0.)
        .count();
    assert!((crossings as i32 - 220).abs() <= 1);
    let mut soft = Engine::new(48000., s);
    soft.event(Event::NoteOn {
        note: 69,
        channel: 0,
        velocity: 0.1,
        id: None,
    });
    assert!(energy(&render(&mut soft, 12000)) < energy(&bent) * 0.1);
}
#[test]
fn midi_xy_latches_then_host_value_takes_over() {
    let mut e = Engine::new(48000., factory::patch(0).synth);
    e.event(Event::Cc {
        channel: 0,
        cc: 16,
        value: 0.9,
    });
    e.set_controls(Controls::default());
    assert_eq!(e.xy()[0], 0.9);
    e.set_controls(Controls {
        x: 0.1,
        ..Controls::default()
    });
    assert_eq!(e.xy()[0], 0.1);
}
#[test]
fn maximum_polyphony_and_extreme_automation_stay_finite() {
    let mut s = factory::patch(1).synth;
    s.voices = 32;
    s.resonance = 0.95;
    s.drive = 1.;
    s.phase_mod = 3.;
    let mut e = Engine::new(8000., s);
    for note in 20..100 {
        e.event(on(note, note as i32));
    }
    assert_eq!(e.active_voices(), 32);
    for i in 0..16000 {
        if i % 17 == 0 {
            e.set_controls(Controls {
                x: (i % 31) as f32 / 30.,
                y: (i % 29) as f32 / 28.,
                cutoff: if i % 2 == 0 { 1. } else { -1. },
                ..Controls::default()
            });
        }
        let a = e.next(if i % 200 < 100 { 20. } else { 400. });
        assert!(a.iter().all(|x| x.is_finite() && x.abs() <= 1.));
    }
    e.event(Event::AllOff);
    render(&mut e, 12000);
    assert_eq!(e.active_voices(), 0);
}
#[test]
fn patch_switch_releases_notes_and_drops_old_effect_history() {
    let mut e = Engine::new(48000., factory::patch(1).synth);
    e.event(on(60, 1));
    render(&mut e, 24000);
    let mut silent = clean_sine();
    for o in &mut silent.oscillators {
        o.level = 0.;
    }
    e.switch_patch(silent);
    assert_eq!(e.active_voices(), 0);
    let a = render(&mut e, 24000);
    assert!(a[1000..].iter().flatten().all(|x| *x == 0.));
}
#[test]
fn render_events_and_patch_switches_allocate_nothing() {
    let patches: Vec<_> = (0..10).map(|i| factory::patch(i).synth).collect();
    let mut e = Engine::new(48000., patches[0]);
    let info = allocation_counter::measure(|| {
        for (index, s) in patches.iter().enumerate() {
            e.switch_patch(*s);
            for note in 48..80 {
                e.event(on(note, note as i32));
            }
            e.set_controls(Controls {
                x: index as f32 / 10.,
                y: 0.8,
                ..Controls::default()
            });
            for _ in 0..256 {
                std::hint::black_box(e.next(145.));
            }
            e.event(Event::AllOff);
            e.reset();
        }
    });
    assert_eq!(info.count_total, 0, "Audio path allocated: {info:?}");
    assert_eq!(info.count_current, 0);
}

#[test]
fn reset_repeats_audio_and_patch_boundary_preserves_the_last_sample() {
    let mut e = Engine::new(48000., factory::patch(0).synth);
    e.event(on(60, 1));
    let a = render(&mut e, 12000);
    e.reset();
    e.event(on(60, 1));
    assert_eq!(a, render(&mut e, 12000));
    let last = e.next(145.);
    e.switch_patch(factory::patch(1).synth);
    assert_eq!(last, e.next(145.));
}

#[test]
fn low_bass_level_is_consistent_across_sample_rates() {
    let mut levels = Vec::new();
    for rate in [8000., 44100., 48000., 96000., 192000.] {
        let mut e = Engine::new(rate, clean_sine());
        e.event(on(24, 1));
        render(&mut e, (rate * 0.5) as usize);
        levels.push(energy(&render(&mut e, (rate * 0.5) as usize)));
    }
    let min = levels.iter().copied().fold(f64::INFINITY, f64::min);
    let max = levels.iter().copied().fold(0., f64::max);
    assert!(
        max / min < 1.05,
        "Sample-rate-dependent bass level: {levels:?}"
    );
}
