// SPDX-License-Identifier: GPL-3.0-or-later
use eis_core::{
    Band, Config, Engine, LATENCY, Preset,
    crossover::Crossover,
    db,
    dynamics::{Dynamics, transfer},
    factory, gain,
    oversample::Saturator,
};
use std::f32::consts::TAU;
fn neutral() -> Config {
    let mut c = Config {
        amount: 0.,
        output_db: 0.,
        safety: false,
        ..Config::default()
    };
    for b in &mut c.bands {
        b.makeup_db = 0.;
        b.up_ratio = 1.;
        b.down_ratio = 1.;
    }
    c
}
fn response(sr: usize, f: f32, four: bool) -> f32 {
    let mut cross = Crossover::new(
        sr as f32,
        if four {
            &[120., 800., 4000.]
        } else {
            &[120., 2500.]
        },
    );
    let mut power = 0_f64;
    let mut reference = 0_f64;
    for i in 0..sr {
        let x = (TAU * f * i as f32 / sr as f32).sin() * 0.1;
        let b = cross.process([x, x]);
        let y: f32 = b.iter().map(|b| b[0]).sum();
        if i > sr / 2 {
            power += (y as f64).powi(2);
            reference += (x as f64).powi(2);
        }
    }
    10. * (power / reference).log10() as f32
}
#[test]
fn three_and_four_band_recombination_are_flat() {
    for sr in [8000, 44100, 48000, 96000, 192000] {
        for f in [40., 120., 600., 2500., 8000., 16000.] {
            if f < sr as f32 * 0.4 {
                for four in [false, true] {
                    let d = response(sr, f, four);
                    assert!(d.abs() < 0.035, "sr={sr} f={f} four={four} delta={d}");
                }
            }
        }
    }
}
#[test]
fn lr4_is_minus_six_db_at_crossover_and_rolls_off() {
    let sr = 48000;
    let mut cross = Crossover::new(sr as f32, &[1000.]);
    let mut power = [0_f64; 2];
    for i in 0..sr {
        let x = (TAU * 1000. * i as f32 / sr as f32).sin();
        let b = cross.process([x, x]);
        if i > sr / 2 {
            for ch in 0..2 {
                power[ch] += (b[ch][0] as f64).powi(2);
            }
        }
    }
    for p in power {
        let d = 10. * (p / ((sr / 2 - 1) as f64 * 0.5)).log10();
        assert!((d + 6.0206).abs() < 0.02, "{d}");
    }
    assert!(response(48000, 4000., false).abs() < 0.02);
}
#[test]
fn upward_and_downward_transfer_are_real_and_capped() {
    let b = Band {
        knee_db: 0.,
        up_threshold: -40.,
        up_ratio: 4.,
        down_threshold: -20.,
        down_ratio: 4.,
        ..Band::default()
    };
    assert_eq!(transfer(-60., b, 1., 1., -100.), [15., 0.]);
    assert_eq!(transfer(-30., b, 1., 1., -100.), [0., 0.]);
    assert_eq!(transfer(0., b, 1., 1., -100.), [0., -15.]);
    assert_eq!(transfer(-120., b, 1., 1., -84.), [0., 0.]);
    assert_eq!(transfer(-60., b, 0., 1., -100.), [0., 0.]);
    let mut b = b;
    b.up_limit_db = 9.;
    assert_eq!(transfer(-60., b, 1.5, 1.5, -100.)[0], 9.);
    for level in -120..12 {
        let t = transfer(level as f32, b, 1., 1., -84.);
        assert!(t[0] <= 9. && t[1] >= -48.);
    }
}
#[test]
fn punch_changes_attack_without_makeup_and_link_preserves_image() {
    let b = Band {
        up_ratio: 1.,
        down_threshold: -30.,
        down_ratio: 20.,
        attack_ms: 5.,
        makeup_db: 0.,
        ..Band::default()
    };
    let mut fast = Dynamics::default();
    let mut punch = Dynamics::default();
    for _ in 0..240 {
        fast.process([0.8, 0.2], b, 1., 1., 1., 0., -84., 48000.);
        punch.process([0.8, 0.2], b, 1., 1., 1., 1., -84., 48000.);
    }
    assert!(punch.down[0] > fast.down[0] + 5.);
    assert_eq!(fast.down[0], fast.down[1]);
    for _ in 0..48000 {
        fast.process([0.8, 0.2], b, 1., 1., 1., 0., -84., 48000.);
        punch.process([0.8, 0.2], b, 1., 1., 1., 1., -84., 48000.);
    }
    assert!((fast.down[0] - punch.down[0]).abs() < 0.01);
}
#[test]
fn oversampling_has_exact_latency_and_unity_dc() {
    let mut sat = Saturator::default();
    let mut result = Vec::new();
    for i in 0..100 {
        result.push(sat.process(if i == 0 { [1.; 2] } else { [0.; 2] }, 0.)[0]);
    }
    let peak = result
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.abs().total_cmp(&b.1.abs()))
        .unwrap()
        .0;
    assert_eq!(peak, LATENCY);
    for _ in 0..200 {
        sat.process([0.2, -0.1], 0.);
    }
    let y = sat.process([0.2, -0.1], 0.);
    assert!((y[0] - 0.2).abs() < 1e-5 && (y[1] + 0.1).abs() < 1e-5);
}
#[test]
fn mix_zero_is_phase_aligned_and_true_bypass_is_delayed_raw() {
    let mut a = Engine::new(48000., neutral());
    let mut c = neutral();
    c.mix = 0.;
    let mut b = Engine::new(48000., c);
    for i in 0..4096 {
        let x = [(i as f32 * 0.71).sin() * 0.2, (i as f32 * 0.1).sin() * 0.1];
        let ya = a.process(x);
        let yb = b.process(x);
        assert!((ya[0] - yb[0]).abs() < 1e-6);
    }
    let mut c = neutral();
    c.bypass = true;
    c.output_db = -20.;
    let mut e = Engine::new(48000., c);
    for i in 0..100 {
        assert_eq!(
            e.process(if i == 0 { [1., -0.5] } else { [0.; 2] }),
            if i == LATENCY { [1., -0.5] } else { [0.; 2] }
        );
    }
}
#[test]
fn near_silence_has_bounded_lift_and_exact_silence_stays_zero() {
    let mut c = factory(7).config;
    c.safety = false;
    c.output_db = 0.;
    let mut e = Engine::new(8000., c);
    for _ in 0..4000 {
        assert_eq!(e.process([0.; 2]), [0.; 2]);
    }
    let mut max = 0_f32;
    for i in 0..8000 {
        let x = (i as f32 * 0.3).sin() * 1e-7;
        let y = e.process([x, -x]);
        max = max.max(y[0].abs());
    }
    assert!(max < 1e-5, "Noise-floor lift {max}");
}
#[test]
fn preset_validation_roundtrip_and_non_destructive_macros() {
    let mut previous = None;
    for i in 0..8 {
        let p = factory(i);
        p.validate().unwrap();
        assert_eq!(p, Preset::from_json(&p.to_json().unwrap()).unwrap());
        if let Some(c) = previous {
            assert_ne!(p.config.bands, c);
        }
        previous = Some(p.config.bands);
    }
    let mut p = factory(0);
    let bands = p.config.bands;
    p.config.amount = 1.3;
    p.config.time = 0.3;
    p.config.juice = 0.5;
    p.config.punch = 0.8;
    assert_eq!(p.config.bands, bands);
    p.version = 2;
    assert!(p.validate().is_err());
    p.version = 1;
    p.config.bands[1].attack_ms = f32::NAN;
    assert!(p.validate().is_err());
    assert!(Preset::from_json(&" ".repeat(32769)).is_err());
}
#[test]
fn buffers_resets_and_automation_are_deterministic_and_allocate_nothing() {
    let c = factory(0).config;
    let mut a = Engine::new(8000., c);
    let mut b = Engine::new(8000., c);
    let bank: Vec<_> = (0..8).map(|i| factory(i).config).collect();
    let info = allocation_counter::measure(|| {
        for i in 0..8000 {
            if i % 1000 == 0 {
                a.set_config(bank[i / 1000]);
                b.set_config(bank[i / 1000]);
            }
            if i == 4000 {
                a.reset();
                b.reset();
            }
            let x = [(i as f32 * 0.17).sin() * 0.2, -0.1];
            assert_eq!(a.process(x), b.process(x));
        }
    });
    assert_eq!(info.count_total, 0, "{info:?}");
    assert_eq!(info.count_current, 0);
}
#[test]
fn extreme_automation_and_hostile_audio_remain_finite() {
    let mut c = factory(7).config;
    let mut e = Engine::new(8000., c);
    for i in 0..20000 {
        if i % 73 == 0 {
            c.amount = if i % 2 == 0 { 0. } else { 1.5 };
            c.crossovers = [40. + (i % 960) as f32, 600. + (i % 15000) as f32];
            c.drive = (i % 101) as f32 / 100.;
            c.bands[(i / 73) % 3].bypass = i % 2 == 0;
            c.auto_gain = i % 4 == 0;
            c.bypass = i % 6 == 0;
            e.set_config(c);
        }
        let x = if i % 199 == 0 {
            [f32::NAN, f32::INFINITY]
        } else {
            [32., -32.]
        };
        let y = e.process(x);
        assert!(y.iter().all(|v| v.is_finite() && v.abs() <= 32.));
    }
}
#[test]
fn smoothing_limits_output_trim_steps() {
    let mut c = neutral();
    c.bypass = true;
    let mut e = Engine::new(8000., c);
    for _ in 0..1000 {
        e.process([0.1, 0.1]);
    }
    c.bypass = false;
    c.output_db = -12.;
    c.mix = 0.;
    e.set_config(c);
    let mut before = 0.1;
    for _ in 0..3000 {
        let y = e.process([0.1, 0.1])[0];
        assert!((y - before).abs() < 0.01);
        before = y;
    }
    assert!(db(gain(-12.)) + 12. < 1e-5);
}

#[test]
fn extreme_macros_keep_static_transfer_monotonic() {
    for i in 0..8 {
        let c = factory(i).config;
        for b in c.bands {
            let mut prior = -200.;
            for step in 0..1260 {
                let level = -120. + step as f32 * 0.1;
                let g = transfer(level, b, 1.5, 1.5, c.floor_db);
                let out = level + g[0] + g[1];
                assert!(
                    out >= prior - 0.0001,
                    "Reversed transfer at {level}: {out} < {prior}"
                );
                prior = out;
            }
        }
    }
}
#[test]
fn band_bypass_removes_processing_and_solo_overrides_global_dry() {
    let reference = neutral();
    let mut c = factory(7).config;
    c.output_db = 0.;
    c.safety = false;
    c.tone_db = [6., -3., 4.];
    for b in &mut c.bands {
        b.bypass = true;
    }
    let mut a = Engine::new(8000., c);
    let mut b = Engine::new(8000., reference);
    for i in 0..2000 {
        let x = [(i as f32 * 0.17).sin() * 0.2, 0.1];
        let y = a.process(x);
        let r = b.process(x);
        assert!((y[0] - r[0]).abs() < 1e-6);
    }
    let mut c = neutral();
    c.bands[0].solo = true;
    c.mix = 0.;
    let mut a = Engine::new(8000., c);
    c.mix = 1.;
    let mut b = Engine::new(8000., c);
    for i in 0..1000 {
        let x = [(i as f32 * 0.9).sin() * 0.2, 0.];
        assert_eq!(a.process(x), b.process(x));
    }
}
#[test]
fn compensation_is_bounded_slow_and_not_applied_to_dry() {
    let mut c = neutral();
    c.auto_gain = true;
    for b in &mut c.bands {
        b.makeup_db = 12.;
    }
    let mut e = Engine::new(8000., c);
    let mut prior = 0.;
    for i in 0..48000 {
        let x = (i as f32 * 0.2).sin() * 0.1;
        e.process([x, x]);
        let m = e.meter();
        assert!(m.compensation_db >= -12.0001 && m.compensation_db <= 6.0001);
        assert!((m.compensation_db - prior).abs() <= 2.01 / 8000.);
        prior = m.compensation_db;
    }
    assert!(prior < -9.);
    c.mix = 0.;
    let mut a = Engine::new(8000., c);
    let mut b = Engine::new(8000., neutral());
    for i in 0..8000 {
        let x = [(i as f32 * 0.2).sin() * 0.1, 0.];
        assert_eq!(a.process(x), b.process(x));
    }
}
#[test]
fn explicit_chunk_sizes_produce_identical_samples() {
    let c = factory(3).config;
    let input: Vec<_> = (0..4096)
        .map(|i| {
            [
                (i as f32 * 0.17).sin() * 0.2,
                (i as f32 * 0.31).sin() * 0.05,
            ]
        })
        .collect();
    let mut e = Engine::new(48000., c);
    let reference: Vec<_> = input.iter().map(|x| e.process(*x)).collect();
    for size in [1, 31, 64, 257, 1024] {
        let mut e = Engine::new(48000., c);
        let mut output = Vec::new();
        for chunk in input.chunks(size) {
            e.set_config(c);
            output.extend(chunk.iter().map(|x| e.process(*x)));
        }
        assert_eq!(output, reference);
    }
}

#[test]
fn linked_engine_preserves_stereo_ratio_without_saturation() {
    let mut c = factory(0).config;
    c.safety = false;
    c.output_db = 0.;
    let mut e = Engine::new(48000., c);
    for i in 0..4096 {
        let x = (i as f32 * 0.17).sin() * 0.4;
        let y = e.process([x, x * 0.25]);
        assert!((y[1] - y[0] * 0.25).abs() < 1e-6);
    }
}
#[test]
fn attack_and_release_times_follow_sample_rate() {
    let b = Band {
        up_ratio: 1.,
        attack_ms: 10.,
        release_ms: 100.,
        down_threshold: -30.,
        ..Band::default()
    };
    let mut reference = None;
    for sr in [8000, 44100, 48000, 96000, 192000] {
        let mut d = Dynamics::default();
        let mut values = [0.; 2];
        for i in 0..sr / 2 {
            let x = if i < sr / 4 { 0.8 } else { 0.001 };
            d.process([x, x], b, 1., 1., 1., 0., -84., sr as f32);
            if i == sr / 100 {
                values[0] = d.down[0];
            }
            if i == sr / 4 + sr / 10 {
                values[1] = d.down[0];
            }
        }
        if let Some(r) = reference {
            let r: [f32; 2] = r;
            assert!((values[0] - r[0]).abs() < 0.2);
            assert!((values[1] - r[1]).abs() < 0.2);
        } else {
            reference = Some(values);
        }
    }
}
#[test]
fn safety_is_transparent_below_knee_and_tails_end_in_exact_silence() {
    let c = neutral();
    let mut a = Engine::new(8000., c);
    let mut c = c;
    c.safety = true;
    let mut b = Engine::new(8000., c);
    for i in 0..1000 {
        let x = [(i as f32 * 0.17).sin() * 0.2, 0.1];
        assert_eq!(a.process(x), b.process(x));
    }
    let mut e = Engine::new(8000., factory(7).config);
    e.process([0.5, -0.2]);
    let mut last = [1.; 2];
    for _ in 0..48000 {
        last = e.process([0.; 2]);
    }
    assert_eq!(last, [0.; 2]);
    for _ in 0..1000 {
        let y = e.process([32., -32.]);
        assert!(y.iter().all(|v| v.abs() <= 1.));
    }
}
