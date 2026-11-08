// SPDX-License-Identifier: GPL-3.0-or-later
use echogirl_core::{
    Config, Engine, Preset, Transport, factory,
    preset::{DIVISIONS, Route},
};
fn clean() -> Config {
    Config {
        character: 0,
        sync: false,
        time_ms: 100.,
        right_ms: 150.,
        feedback: 0.,
        drive: 0.,
        mix: 1.,
        low_cut: 0.,
        high_cut: 20000.,
        resonance: 0.,
        mappings: [Route::default(); 8],
        ..Config::default()
    }
}
fn samples(e: &mut Engine, n: usize) -> Vec<[f32; 2]> {
    (0..n).map(|_| e.process([0.; 2])).collect()
}
#[test]
fn impulses_have_exact_time_and_separate_channels() {
    for sr in [8000., 44100., 48000., 96000., 192000.] {
        let mut c = clean();
        c.link = false;
        let mut e = Engine::new(sr, c);
        e.process([0.5, 0.3]);
        let a = samples(&mut e, (sr * 0.2) as usize);
        let l = (sr * 0.1).round() as usize - 1;
        let r = (sr * 0.15).round() as usize - 1;
        assert!(a[..l].iter().all(|s| s[0] == 0.));
        assert!((a[l][0] - 0.5).abs() < 1e-5, "{sr}");
        assert!(a[..r].iter().all(|s| s[1] == 0.));
        assert!((a[r][1] - 0.3).abs() < 1e-5);
    }
}
#[test]
fn tempo_divisions_dotted_triplet_and_long_buffer_boundary() {
    for (index, (_, beats)) in DIVISIONS.iter().enumerate() {
        let c = Config {
            sync: true,
            left_div: index as u8,
            ..clean()
        };
        let mut e = Engine::new(8000., c);
        e.set_transport(Transport {
            bpm: 120.,
            ..Transport::default()
        });
        e.reset();
        e.process([0.5, 0.]);
        let n = (8000. * 0.5 * beats).round() as usize;
        let a = samples(&mut e, n + 2);
        assert!(
            a[n.saturating_sub(2)..=n].iter().any(|s| s[0].abs() > 0.1),
            "division {index}"
        );
    }
    let c = Config {
        time_ms: 12000.,
        ..clean()
    };
    let mut e = Engine::new(8000., c);
    e.process([0.4, 0.]);
    let a = samples(&mut e, 192001);
    assert!((a[95999][0] - 0.4).abs() < 1e-5);
    assert!(a.iter().all(|s| s.iter().all(|x| x.is_finite())));
}
#[test]
fn dry_and_bypass_are_unity_and_input_trim_only_changes_echo_feed() {
    let mut c = clean();
    c.mix = 0.;
    c.input_db = 12.;
    let mut e = Engine::new(48000., c);
    for i in 0..1000 {
        let x = [(i as f32 * 0.1).sin() * 0.4, -0.3];
        assert_eq!(e.process(x), x);
    }
    c.bypass = true;
    c.output_db = -20.;
    e.set_config(c);
    for _ in 0..12000 {
        e.process([0.; 2]);
    }
    assert!((e.process([0.4, -0.2])[0] - 0.4).abs() < 1e-5);
}
#[test]
fn ping_pong_alternates_repeat_channels() {
    let mut c = clean();
    c.ping_pong = true;
    c.feedback = 0.7;
    let mut e = Engine::new(8000., c);
    e.process([0.8, 0.8]);
    let a = samples(&mut e, 2000);
    assert!(a[799][0] > 0.79 && a[799][1] == 0.);
    assert!(a[1599][1] > 0.5 && a[1599][0].abs() < 0.001);
}
#[test]
fn hold_sustains_then_kill_clears_including_spring() {
    let mut c = clean();
    c.feedback = 0.6;
    c.spring = 0.4;
    c.time_ms = 20.;
    let mut e = Engine::new(8000., c);
    e.process([0.8, 0.3]);
    samples(&mut e, 160);
    c.hold = true;
    e.set_config(c);
    let held = samples(&mut e, 16000);
    assert!(held[8000..].iter().flatten().any(|x| x.abs() > 0.001));
    c.kill = true;
    e.set_config(c);
    let killed = samples(&mut e, 1000);
    assert!(killed[200..].iter().flatten().all(|x| *x == 0.));
    c.kill = false;
    c.hold = false;
    e.set_config(c);
    assert!(samples(&mut e, 8000).iter().flatten().all(|x| *x == 0.));
}
#[test]
fn throw_captures_then_closes_input_without_muting_dry() {
    let mut c = clean();
    c.mix = 0.;
    let mut e = Engine::new(8000., c);
    c.throw = true;
    e.set_config(c);
    for _ in 0..100 {
        assert_eq!(e.process([0.2, 0.1]), [0.2, 0.1]);
    }
    c.throw = false;
    e.set_config(c);
    assert!(e.throw_mode());
    for _ in 0..8000 {
        assert_eq!(e.process([0.2, 0.1]), [0.2, 0.1]);
    }
    c.mix = 1.;
    e.set_config(c);
    for _ in 0..8000 {
        e.process([0.3, 0.3]);
    }
    assert!(e.process([0.3, 0.3]).iter().all(|x| x.abs() < 1e-5));
    c.feed = false;
    e.set_config(c);
    c.feed = true;
    e.set_config(c);
    assert!(!e.throw_mode());
}
#[test]
fn rapid_xy_tempo_hold_mode_and_time_changes_are_bounded() {
    let mut c = factory(7).config;
    c.feedback = 1.08;
    let mut e = Engine::new(8000., c);
    for i in 0..80000 {
        if i % 31 == 0 {
            c.x = (i % 101) as f32 / 100.;
            c.y = (i % 97) as f32 / 96.;
            c.hold = i % 701 < 350;
            c.character = ((i / 1000) % 4) as u8;
            c.time_ms = 10. + (i % 1000) as f32;
            c.sync = i % 2000 < 1000;
            e.set_config(c);
            e.set_transport(Transport {
                bpm: if i % 2 == 0 { 20. } else { 400. },
                playing: i % 4000 < 2000,
                beat: None,
            });
        }
        let a = e.process([if i < 1000 { 0.3 } else { 0. }, -0.1]);
        assert!(a.iter().all(|x| x.is_finite() && x.abs() <= 1.00001));
    }
}
#[test]
fn moving_clean_head_does_not_discontinue_constant_echo() {
    let mut c = clean();
    c.time_ms = 20.;
    let mut e = Engine::new(48000., c);
    for _ in 0..48000 {
        e.process([0.2, 0.2]);
    }
    let mut prior = e.process([0.2, 0.2]);
    c.time_ms = 300.;
    e.set_config(c);
    for _ in 0..24000 {
        let a = e.process([0.2, 0.2]);
        assert!((a[0] - prior[0]).abs() < 0.01);
        prior = a;
    }
}
#[test]
fn context_stop_start_keeps_echo_history_but_reset_is_deterministic() {
    let c = clean();
    let mut e = Engine::new(8000., c);
    e.process([0.5, 0.]);
    e.set_transport(Transport {
        playing: false,
        ..Transport::default()
    });
    let a = samples(&mut e, 1000);
    assert!((a[799][0] - 0.5).abs() < 1e-5);
    e.set_transport(Transport::default());
    e.reset();
    e.process([0.5, 0.]);
    assert_eq!(a, samples(&mut e, 1000));
}
#[test]
fn presets_validate_round_trip_and_reject_unknown_or_dangerous_data() {
    for i in 0..10 {
        let p = factory(i);
        assert_eq!(Preset::from_json(&p.to_json().unwrap()).unwrap(), p);
    }
    let mut p = factory(0);
    p.version = 2;
    assert!(p.validate().is_err());
    p.version = 1;
    p.config.feedback = f32::NAN;
    assert!(p.validate().is_err());
    p.config = clean();
    p.config.midi_cc[1] = p.config.midi_cc[0];
    assert!(p.validate().is_err());
    assert!(Preset::from_json(&"x".repeat(32769)).is_err());
}
#[test]
fn rendering_and_resets_do_not_allocate_or_free() {
    let mut e = Engine::new(48000., clean());
    let bank: Vec<_> = (0..10).map(|i| factory(i).config).collect();
    let info = allocation_counter::measure(|| {
        for c in bank.iter() {
            e.set_config(*c);
            e.reset();
            for _ in 0..2048 {
                std::hint::black_box(e.process([0.2, -0.1]));
            }
        }
    });
    assert_eq!(info.count_total, 0, "{info:?}");
    assert_eq!(info.count_current, 0);
}
#[test]
fn nonfinite_input_is_sanitized() {
    let mut e = Engine::new(8000., clean());
    for _ in 0..10000 {
        assert!(
            e.process([f32::NAN, f32::INFINITY])
                .iter()
                .all(|x| x.is_finite())
        );
    }
}

#[test]
fn autonomous_feedback_stays_bounded_and_rejects_dc() {
    for rate in [8000, 48000, 192000] {
        let mut c = clean();
        c.time_ms = 10.;
        c.feedback = 1.08;
        let mut e = Engine::new(rate as f32, c);
        let mut dc = 0_f64;
        let mut power = 0_f64;
        for i in 0..rate * 6 {
            let seed = if i < rate / 10 {
                0.2 + 0.2 * (std::f32::consts::TAU * 100. * i as f32 / rate as f32).sin()
            } else {
                0.
            };
            let a = e.process([seed, -seed]);
            assert!(a.iter().all(|x| x.is_finite() && x.abs() <= 1.));
            if i >= rate * 5 {
                dc += a[0] as f64;
                power += (a[0] as f64).powi(2);
            }
        }
        assert!(dc.abs() / (rate as f64) < 0.01);
        assert!(
            power / (rate as f64) > 0.001,
            "Feedback lost its seed at {rate} Hz"
        );
    }
}

#[test]
fn character_crossfade_preserves_level_and_cannot_revive_killed_audio() {
    let mut c = clean();
    c.time_ms = 10.;
    let mut e = Engine::new(8000., c);
    for _ in 0..8000 {
        e.process([0.2, 0.2]);
    }
    let before = e.process([0.2, 0.2]);
    c.character = 2;
    e.set_config(c);
    assert!((e.process([0.2, 0.2])[0] - before[0]).abs() < 1e-5);
    c.kill = true;
    e.set_config(c);
    for _ in 0..100 {
        e.process([0.2, 0.2]);
    }
    for i in 0..1000 {
        c.character = (i % 4) as u8;
        e.set_config(c);
        assert_eq!(e.process([0.2, 0.2]), [0.; 2]);
    }
}
