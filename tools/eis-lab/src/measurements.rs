// SPDX-License-Identifier: GPL-3.0-or-later
use crate::Result;
use eis_core::{
    Band, Engine,
    crossover::Crossover,
    dynamics::{Dynamics, transfer},
    factory, gain,
    oversample::Saturator,
};
use std::f32::consts::TAU;
use std::{fmt::Write, fs};
fn fft_power(x: &[f32]) -> Vec<f64> {
    let n = x.len();
    let bits = n.trailing_zeros();
    let mut re = vec![0_f64; n];
    let mut im = vec![0_f64; n];
    for (i, x) in x.iter().enumerate() {
        re[i.reverse_bits() >> (usize::BITS - bits)] = *x as f64;
    }
    let mut size = 2;
    while size <= n {
        let a = -std::f64::consts::TAU / size as f64;
        let (wr, wi) = (a.cos(), a.sin());
        for base in (0..n).step_by(size) {
            let (mut r, mut m) = (1., 0.);
            for j in 0..size / 2 {
                let a = base + j;
                let b = a + size / 2;
                let tr = r * re[b] - m * im[b];
                let ti = r * im[b] + m * re[b];
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
                let nr = r * wr - m * wi;
                m = r * wi + m * wr;
                r = nr;
            }
        }
        size *= 2;
    }
    (0..n / 2).map(|i| re[i] * re[i] + im[i] * im[i]).collect()
}
pub fn run() -> Result {
    let out = std::path::Path::new("target/eis-validation");
    fs::create_dir_all(out)?;
    let mut response = String::from("rate,hz,low_db,mid_db,high_db,sum_db\n");
    let mut max_error = 0_f64;
    for sr in [8000, 44100, 48000, 96000, 192000] {
        for f in [
            30., 40., 60., 100., 120., 180., 400., 1000., 2000., 2500., 3500., 6000., 10000.,
            16000.,
        ] {
            if f >= sr as f32 * 0.4 {
                continue;
            }
            let mut cross = Crossover::new(sr as f32, &[120., 2500_f32.min(sr as f32 * 0.4)]);
            let mut p = [0_f64; 4];
            let mut reference = 0_f64;
            for i in 0..sr {
                let x = (TAU * f * i as f32 / sr as f32).sin() * 0.1;
                let b = cross.process([x, x]);
                if i > sr / 2 {
                    for j in 0..3 {
                        p[j] += (b[j][0] as f64).powi(2);
                    }
                    p[3] += (b[0][0] + b[1][0] + b[2][0]).powi(2) as f64;
                    reference += (x as f64).powi(2);
                }
            }
            let d = p.map(|v| 10. * (v / reference).max(1e-18).log10());
            max_error = max_error.max(d[3].abs());
            writeln!(response, "{sr},{f},{},{},{},{}", d[0], d[1], d[2], d[3])?;
        }
    }
    fs::write(out.join("frequency-response.csv"), response)?;
    let mut curves = String::from("preset,band,input_db,up_db,down_db,output_db\n");
    for preset in 0..8 {
        let p = factory(preset);
        for i in 0..3 {
            for level in -120..=6 {
                let t = transfer(
                    level as f32,
                    p.config.bands[i],
                    p.config.amount,
                    p.config.juice,
                    p.config.floor_db,
                );
                writeln!(
                    curves,
                    "{preset},{i},{level},{},{},{}",
                    t[0],
                    t[1],
                    level as f32 + t[0] + t[1] + p.config.bands[i].makeup_db + p.config.tone_db[i]
                )?;
            }
        }
    }
    fs::write(out.join("transfer-curves.csv"), curves)?;
    let mut measured =
        String::from("preset,band,input_db,target_up,target_down,measured_up,measured_down\n");
    let mut gain_error = 0_f32;
    for index in 0..8 {
        let p = factory(index);
        for (band, b) in p.config.bands.iter().enumerate() {
            for level in [-60., -40., -20., 0.] {
                let mut d = Dynamics::default();
                let x = gain(level);
                for _ in 0..16000 {
                    d.process(
                        [x, x],
                        *b,
                        p.config.amount,
                        p.config.juice,
                        p.config.time,
                        p.config.punch,
                        p.config.floor_db,
                        8000.,
                    );
                }
                let target = transfer(
                    level,
                    *b,
                    p.config.amount,
                    p.config.juice,
                    p.config.floor_db,
                );
                gain_error = gain_error
                    .max((d.up[0] - target[0]).abs())
                    .max((d.down[0] - target[1]).abs());
                writeln!(
                    measured,
                    "{index},{band},{level},{},{},{},{}",
                    target[0], target[1], d.up[0], d.down[0]
                )?;
            }
        }
    }
    fs::write(out.join("measured-transfer.csv"), measured)?;
    let b = Band {
        up_ratio: 1.,
        down_threshold: -30.,
        down_ratio: 20.,
        attack_ms: 5.,
        makeup_db: 0.,
        ..Band::default()
    };
    let mut a = Dynamics::default();
    let mut punch = Dynamics::default();
    let mut envelopes = String::from("frame,input,plain_gain_db,punch_gain_db\n");
    let mut at_5ms = [0.; 2];
    for i in 0..24000 {
        let x = if (1000..13000).contains(&i) {
            0.8
        } else {
            0.001
        };
        let ga = a.process([x, x], b, 1., 1., 1., 0., -84., 48000.)[0];
        let gp = punch.process([x, x], b, 1., 1., 1., 1., -84., 48000.)[0];
        if i == 1240 {
            at_5ms = [ga, gp];
        }
        writeln!(envelopes, "{i},{x},{ga},{gp}")?;
    }
    fs::write(out.join("transients.csv"), envelopes)?;
    let mut noise = String::from("input_dbfs,output_dbfs,lift_db\n");
    for level in [-120., -100., -90., -84., -78., -72., -60., -48., -36., -18.] {
        let mut c = factory(0).config;
        c.output_db = 0.;
        c.safety = false;
        for b in &mut c.bands {
            b.makeup_db = 0.;
        }
        let mut e = Engine::new(8000., c);
        let mut energy = 0_f64;
        let mut source = 0_f64;
        let mut seed = 0x12345678_u32;
        for i in 0..16000 {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let x = (seed as f32 / u32::MAX as f32 * 2. - 1.) * gain(level);
            let y = e.process([x, x]);
            if i >= 8000 {
                energy += (y[0] as f64).powi(2);
                source += (x as f64).powi(2);
            }
        }
        let input = 10. * (source / 8000.).log10();
        let output = 10. * (energy / 8000.).log10();
        writeln!(noise, "{input},{output},{}", output - input)?;
    }
    fs::write(out.join("noise-floor.csv"), noise)?;
    let n = 4096;
    let bin = 817;
    let drive = 0.65;
    let g = 1. + 24. * drive * drive;
    let mut sat = Saturator::default();
    let mut one = Vec::new();
    let mut four = Vec::new();
    for i in 0..n * 2 {
        let x = (std::f64::consts::TAU * bin as f64 * i as f64 / n as f64).sin() as f32 * 0.4;
        let y = sat.process([x, x], drive)[0];
        if i >= n {
            one.push((x * g).tanh() * g.sqrt() / g);
            four.push(y);
        }
    }
    let alias = |x: &[f32]| {
        let p = fft_power(x);
        let unwanted = p
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != bin && *i > 0)
            .map(|(_, x)| x)
            .sum::<f64>();
        10. * (unwanted / p[bin]).log10()
    };
    let alias1 = alias(&one);
    let alias4 = alias(&four);
    signals(out)?;
    let report = serde_json::json!({"max_crossover_error_db":max_error,"crossover_acceptance_db":0.035,"maximum_measured_transfer_error_db":gain_error,"transient_gain_after_5ms_db":at_5ms,"saturation_test":{"rate":48000,"frequency_hz":bin as f64*48000./n as f64,"amplitude":0.4,"drive":drive,"one_x_unwanted_dbc":alias1,"four_x_unwanted_dbc":alias4,"improvement_db":alias1-alias4},"comparison_scope":"Independent mathematical reference and 1x local shaper; no proprietary binary or preset used. Demo manifest includes offline BS.1770-style LUFS estimates, not meter certification."});
    fs::write(
        out.join("measurements.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if max_error > 0.035 {
        return Err("Crossover response gate failed".into());
    }
    if gain_error > 0.05 {
        return Err("Measured dynamics transfer gate failed".into());
    }
    if alias1 - alias4 < 20. || alias4 > -40. {
        return Err("Saturation antialias gate failed; keep saturation unqualified".into());
    }
    if at_5ms[1] <= at_5ms[0] + 5. {
        return Err("PUNCH transient gate failed".into());
    }
    Ok(())
}

fn signals(root: &std::path::Path) -> Result {
    let out = root.join("signals");
    fs::create_dir_all(&out)?;
    for scenario in 0..5 {
        let mut e = Engine::new(48000., factory(0).config);
        let mut source = vec![Vec::new(), Vec::new()];
        let mut audio = vec![Vec::new(), Vec::new()];
        let mut seed = 0x12345678_u32;
        let mut pink = [0_f32; 12];
        let mut phase = 0_f64;
        for i in 0..144000 {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let noise = seed as f32 / u32::MAX as f32 * 2. - 1.;
            let x = match scenario {
                0 => {
                    if i == 0 {
                        0.5
                    } else {
                        0.
                    }
                }
                1 => noise * 0.15,
                2 => {
                    let row = ((i + 1) as u32).trailing_zeros() as usize;
                    if row < 12 {
                        pink[row] = noise;
                    }
                    (pink.iter().sum::<f32>() + noise) * 0.025
                }
                3 => {
                    let f = 20. * (21600_f64 / 20.).powf(i as f64 / 144000.);
                    phase += std::f64::consts::TAU * f / 48000.;
                    phase.sin() as f32 * 0.1
                }
                _ => {
                    let amplitude = match i / 24000 {
                        0 => 0.003,
                        1 | 2 => 0.5,
                        3 | 4 => 0.03,
                        _ => 0.,
                    };
                    (TAU * 1000. * i as f32 / 48000.).sin() * amplitude
                }
            };
            let input = [x, x * 0.25];
            let y = e.process(input);
            for ch in 0..2 {
                source[ch].push(input[ch]);
                audio[ch].push(y[ch]);
            }
        }
        let stem = [
            "impulse",
            "white-noise",
            "pink-noise",
            "log-sweep",
            "amplitude-steps",
        ][scenario];
        vst3_host::audio::write_wav(out.join(format!("{stem}-input.wav")), &source, 48000)?;
        vst3_host::audio::write_wav(out.join(format!("{stem}-processed.wav")), &audio, 48000)?;
    }
    Ok(())
}
