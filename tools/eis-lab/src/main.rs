// SPDX-License-Identifier: GPL-3.0-or-later
mod loudness;
mod measurements;
mod native;
use eis_core::{Engine, LATENCY, Preset, factory};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use vst3_host::{
    Vst3Host,
    audio::{AudioBuffers, write_wav},
};
type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;
fn text_words(text: &str) -> [[u32; 8]; 32] {
    let mut b = [0_u8; 1024];
    let n = text.len().min(1024);
    b[..n].copy_from_slice(&text.as_bytes()[..n]);
    std::array::from_fn(|i| {
        std::array::from_fn(|j| {
            let p = (i * 8 + j) * 4;
            u32::from_le_bytes([b[p], b[p + 1], b[p + 2], b[p + 3]])
        })
    })
}
fn component(p: &Preset, butcher: bool) -> Result<Vec<u8>> {
    let c = p.config;
    let mut params = serde_json::Map::new();
    for (key, v) in serde_json::to_value(c)?.as_object().unwrap() {
        if v.is_array() {
            continue;
        }
        params.insert(
            key.replace('_', "-"),
            if v.is_boolean() {
                serde_json::json!({"bool":v})
            } else {
                serde_json::json!({"f32":v})
            },
        );
    }
    for (i, key) in ["low", "mid", "high"].iter().enumerate() {
        params.insert((*key).into(), serde_json::json!({"f32":c.tone_db[i]}));
    }
    for (i, key) in ["cross-low", "cross-high"].iter().enumerate() {
        params.insert((*key).into(), serde_json::json!({"f32":c.crossovers[i]}));
    }
    for (i, b) in c.bands.iter().enumerate() {
        for (key, v) in serde_json::to_value(b)?.as_object().unwrap() {
            params.insert(
                format!("b{}-{}", i + 1, key.replace('_', "-")),
                if v.is_boolean() {
                    serde_json::json!({"bool":v})
                } else {
                    serde_json::json!({"f32":v})
                },
            );
        }
    }
    // Explicit fixture for this pinned NIH schema. Saved opaque streams are never edited.
    Ok(serde_json::to_vec(
        &serde_json::json!({"version":env!("CARGO_PKG_VERSION"),"params":params,"fields":{"schema":"1","butcher":butcher.to_string(),"preset-name":serde_json::to_string(&text_words(&p.name))?,"preset-notes":serde_json::to_string(&text_words(&p.description))?}}),
    )?)
}
/// Original synthesized percussion + moving bass/saw/chord material, no samples.
pub fn input(frame: usize, sr: usize) -> [f32; 2] {
    let t = frame as f32 / sr as f32;
    let beat = (t * 2.3).fract();
    let block = (t * 2.3) as i32;
    let mut n = (frame as u32)
        .wrapping_mul(747796405)
        .wrapping_add(2891336453);
    n ^= n >> 16;
    let noise = n as f32 / u32::MAX as f32 * 2. - 1.;
    let kick = (std::f32::consts::TAU * (45. * t + 10. * (1. - (-beat * 25.).exp()))).sin()
        * (-beat * 22.).exp()
        * 0.35;
    let snare = if block % 2 == 1 {
        noise * (-beat * 28.).exp() * 0.2
    } else {
        0.
    };
    let phase = (t * 98.).fract();
    let saw = (phase * 2. - 1.) * 0.04 * (0.25 + 0.75 * (-beat * 8.).exp());
    let chord = (std::f32::consts::TAU * 196. * t).sin() * 0.025
        + (std::f32::consts::TAU * 233.08 * t).sin() * 0.02;
    [
        kick + snare + saw + chord,
        kick + snare * 0.75 - saw * 0.7 + chord * 0.9,
    ]
}
fn main() -> Result {
    let a: Vec<_> = std::env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("render") => render(
            a.get(2)
                .map(PathBuf::from)
                .unwrap_or_else(|| "target/eis-demos".into()),
        ),
        Some("measure") => measurements::run(),
        Some("profile") => profile(),
        Some("smoke") | Some("editor") => host(&a),
        _ => {
            println!(
                "eis-lab render [OUT] | measure | profile | smoke BUNDLE [OUT] | editor BUNDLE [SECONDS [WIDTH HEIGHT]]"
            );
            Ok(())
        }
    }
}
fn host(a: &[String]) -> Result {
    let _ole = native::Ole::new()?;
    let mut host = Vst3Host::builder()
        .sample_rate(48000.)
        .block_size(256)
        .build()?;
    let mut plugin = host.load_plugin(a.get(2).ok_or("Missing VST3 bundle")?)?;
    if plugin.info().name != "8 INCH STEAK" {
        return Err("Select the EIS plugin explicitly".into());
    }
    if a[1] == "editor" {
        let seconds: u64 = a.get(3).map(|s| s.parse()).transpose()?.unwrap_or(30);
        if seconds > 600 {
            return Err("Preview maximum is 600 seconds".into());
        }
        if let (Some(w), Some(h)) = (a.get(4), a.get(5)) {
            let w: u32 = w.parse()?;
            let h: u32 = h.parse()?;
            if !(900..=1920).contains(&w) || !(760..=1440).contains(&h) {
                return Err("Preview size outside supported bounds".into());
            }
            plugin.load_state(&serde_json::to_vec(&serde_json::json!({"version":"0.1.0","params":{},"fields":{"butcher":(a.get(6).map(String::as_str)==Some("butcher")).to_string(),"editor":serde_json::to_string(&serde_json::json!({"size":[w,h]}))?}}))?)?;
        }
        let plugin = std::sync::Arc::new(std::sync::Mutex::new(plugin));
        let mut window = vst3_host::window::PluginWindow::new(plugin.clone());
        window.open()?;
        // Real plugin processing, discarded offline: visible meters/spectrum, no device.
        plugin.lock().unwrap().start_processing()?;
        let mut buffers = AudioBuffers::new(2, 2, 256, 48000.);
        let mut frame = 0;
        let mut edits = Vec::new();
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(seconds) && !window.closed_by_user() {
            native::pump();
            window.service_platform_events()?;
            {
                let mut p = plugin.lock().unwrap();
                p.service_host_requests()?;
                edits.extend(
                    p.take_parameter_edits()
                        .into_iter()
                        .map(|edit| format!("{edit:?}")),
                );
                let _ = p.get_parameter_changes();
            }
            let due = (start.elapsed().as_secs_f64() * 48000.) as usize;
            while frame + 256 <= due {
                for i in 0..256 {
                    let x = input(frame + i, 48000);
                    buffers.inputs[0][i] = x[0];
                    buffers.inputs[1][i] = x[1];
                }
                plugin.lock().unwrap().process_audio(&mut buffers)?;
                frame += 256;
            }
            std::thread::sleep(Duration::from_millis(8));
        }
        plugin.lock().unwrap().stop_processing()?;
        fs::create_dir_all("target/eis-validation")?;
        fs::write(
            "target/eis-validation/editor-gestures.json",
            serde_json::to_string_pretty(&edits)?,
        )?;
        window.close();
        return Ok(());
    }
    let out = a
        .get(3)
        .map(PathBuf::from)
        .unwrap_or_else(|| "target/eis-host".into());
    fs::create_dir_all(&out)?;
    let buses = plugin.audio_bus_layout()?;
    if buses.inputs.len() != 1
        || buses.outputs.len() != 1
        || buses.inputs[0].channel_count != 2
        || buses.outputs[0].channel_count != 2
    {
        return Err("Expected a stereo effect".into());
    }
    let parameters = plugin.get_parameters()?;
    let id = |name: &str| -> Result<u32> {
        Ok(parameters
            .iter()
            .find(|p| p.name == name)
            .ok_or("Missing parameter")?
            .id)
    };
    let amount = id("TENDERIZE")?;
    let mix = id("MIX")?;
    let bypass = id("Bypass")?;
    let mut report = Vec::new();
    for scenario in 0..4 {
        let mut p = factory(if scenario == 0 || scenario == 3 {
            0
        } else if scenario == 1 {
            2
        } else {
            7
        });
        if scenario == 1 {
            p.name = "SMOKE USER REESE".into();
            p.config.bands[1].up_ratio = 5.1;
            p.config.bands[1].attack_ms = 7.;
            p.config.tone_db[1] = 1.4;
            p.config.auto_gain = true;
        }
        plugin.load_state(&component(&p, scenario == 1 || scenario == 3)?)?;
        let state = plugin.save_state()?;
        fs::write(out.join(format!("scenario-{scenario}.state")), &state)?;
        for pass in 0..2 {
            plugin.load_state(&state)?;
            plugin.start_processing()?;
            let mut e = Engine::new(48000., p.config);
            let mut c = p.config;
            let mut buffers = AudioBuffers::new(2, 2, 256, 48000.);
            let mut audio = vec![Vec::new(), Vec::new()];
            let mut error = 0_f32;
            let mut first = None;
            for block in 0..375 {
                if scenario == 2 {
                    for (b, offset, key, value) in [
                        (30, 37, amount, 0.2),
                        (100, 91, mix, 0.45),
                        (220, 73, bypass, 1.),
                        (240, 99, bypass, 0.),
                    ] {
                        if block == b {
                            plugin.set_parameter_at(key, value, offset)?;
                        }
                    }
                }
                for i in 0..256 {
                    let x = input(block * 256 + i, 48000);
                    buffers.inputs[0][i] = x[0];
                    buffers.inputs[1][i] = x[1];
                }
                plugin.process_audio(&mut buffers)?;
                for i in 0..256 {
                    if scenario == 2 {
                        let mut changed = true;
                        match (block, i) {
                            (30, 37) => c.amount = 0.3,
                            (100, 91) => c.mix = 0.45,
                            (220, 73) => c.bypass = true,
                            (240, 99) => c.bypass = false,
                            _ => changed = false,
                        }
                        if changed {
                            e.set_config(c);
                        }
                    }
                    let expected = e.process(input(block * 256 + i, 48000));
                    for (ch, value) in expected.iter().enumerate() {
                        let actual = buffers.outputs[ch][i];
                        if !actual.is_finite() || actual.abs() > 32. {
                            return Err("Invalid VST3 output".into());
                        }
                        let delta = (actual - value).abs();
                        error = error.max(delta);
                        if delta > 2e-5 && first.is_none() {
                            first = Some(block * 256 + i);
                        }
                        audio[ch].push(actual);
                    }
                }
            }
            plugin.stop_processing()?;
            if error > 2e-5 {
                return Err(format!(
                    "Host/core mismatch scenario {scenario} pass {pass}: {error}, first {first:?}"
                )
                .into());
            }
            if plugin.latency_samples() != LATENCY as u32 {
                return Err("Incorrect VST3 latency".into());
            }
            write_wav(
                out.join(format!("scenario-{scenario}-pass-{pass}.wav")),
                &audio,
                48000,
            )?;
            report.push(serde_json::json!({"scenario":scenario,"pass":pass,"frames":audio[0].len(),"maximum_error":error,"latency":plugin.latency_samples()}));
        }
    }
    fs::write(
        out.join("report.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    println!("EIS VST3 stereo/state/automation reference smoke passed");
    Ok(())
}
fn render(out: PathBuf) -> Result {
    fs::create_dir_all(&out)?;
    fs::create_dir_all("factory/eis")?;
    let mut manifest = Vec::new();
    let mut dry = vec![Vec::new(), Vec::new()];
    for frame in 0..384000 {
        let x = input(frame, 48000);
        for ch in 0..2 {
            dry[ch].push(x[ch]);
        }
    }
    write_wav(out.join("00-original-input.wav"), &dry, 48000)?;
    let input_lufs = loudness::integrated(&dry).ok_or("Source loudness could not be measured")?;
    for index in 0..8 {
        let p = factory(index);
        p.validate()?;
        let stem = format!(
            "{:02}-{}",
            index + 1,
            p.name.to_lowercase().replace(' ', "-")
        );
        fs::write(
            Path::new("factory/eis").join(format!("{stem}.eis.json")),
            p.to_json()?,
        )?;
        let mut e = Engine::new(48000., p.config);
        let mut unprotected = Engine::new(
            48000.,
            eis_core::Config {
                safety: false,
                ..p.config
            },
        );
        let mut audio = vec![Vec::new(), Vec::new()];
        let mut peak = 0_f32;
        let mut pre_safety_peak = 0_f32;
        let mut energy = 0_f64;
        let mut reduction = [0_f32; 3];
        let mut lift = [0_f32; 3];
        for frame in 0..384000 {
            let y = e.process(input(frame, 48000));
            for value in unprotected.process(input(frame, 48000)) {
                pre_safety_peak = pre_safety_peak.max(value.abs());
            }
            for ch in 0..2 {
                audio[ch].push(y[ch]);
                peak = peak.max(y[ch].abs());
                energy += (y[ch] as f64).powi(2);
            }
            let m = e.meter();
            for i in 0..3 {
                reduction[i] = reduction[i].min(m.down[i]);
                lift[i] = lift[i].max(m.up[i]);
            }
        }
        if peak < 0.001 {
            return Err("Silent preset".into());
        }
        write_wav(out.join(format!("{stem}.wav")), &audio, 48000)?;
        let output_lufs =
            loudness::integrated(&audio).ok_or("Output loudness could not be measured")?;
        let matching_db = (input_lufs - output_lufs)
            .min(0.)
            .min(20. * (0.98 / peak as f64).log10());
        let target_lufs = output_lufs + matching_db;
        let matched_input_db = target_lufs - input_lufs;
        let matched: Vec<Vec<f32>> = audio
            .iter()
            .map(|ch| {
                ch.iter()
                    .map(|v| *v * eis_core::gain(matching_db as f32))
                    .collect()
            })
            .collect();
        write_wav(
            out.join(format!("{stem}-matched-output.wav")),
            &matched,
            48000,
        )?;
        let matched_input: Vec<Vec<f32>> = dry
            .iter()
            .map(|ch| {
                ch.iter()
                    .map(|v| *v * eis_core::gain(matched_input_db as f32))
                    .collect()
            })
            .collect();
        write_wav(
            out.join(format!("{stem}-matched-input.wav")),
            &matched_input,
            48000,
        )?;
        manifest.push(serde_json::json!({"preset":p.name,"peak":peak,"pre_safety_peak":pre_safety_peak,"rms_dbfs":10.*(energy/768000.).log10(),"input_lufs_estimate":input_lufs,"output_lufs_estimate":output_lufs,"loudness_delta_lu":output_lufs-input_lufs,"matched_pair_target_lufs":target_lufs,"matched_input_gain_db":matched_input_db,"matched_output_gain_db":matching_db,"maximum_upward_db":lift,"maximum_downward_db":reduction,"rate":48000,"frames":384000}));
        println!(
            "{} peak {:.3}, before safety {:.3}",
            p.name, peak, pre_safety_peak
        );
    }
    fs::write(
        out.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    Ok(())
}
fn profile() -> Result {
    let mut report = Vec::new();
    for scenario in 0..3 {
        let mut c = factory(if scenario == 0 { 0 } else { 7 }).config;
        if scenario == 2 {
            c.auto_gain = true;
            for b in &mut c.bands {
                b.solo = false;
                b.drive = 1.;
                b.link = 0.;
            }
            c.drive = 1.;
        }
        let mut e = Engine::new(48000., c);
        for i in 0..4096 {
            std::hint::black_box(e.process(input(i, 48000)));
        }
        let mut times = Vec::with_capacity(375);
        for block in 0..375 {
            let source: [_; 256] = std::array::from_fn(|i| input(block * 256 + i, 48000));
            let start = Instant::now();
            for x in source {
                std::hint::black_box(e.process(x));
            }
            times.push(start.elapsed().as_secs_f64() * 1e6);
        }
        times.sort_by(f64::total_cmp);
        let mean = times.iter().sum::<f64>() / 375.;
        report.push(serde_json::json!({"scenario":scenario,"mean_core_percent":mean/5333.333*100.,"p99_us":times[371],"maximum_us":times[374],"engine_bytes":e.memory_bytes(),"rate":48000,"block":256,"blocks":375,"environment":"Shared desktop, offline timing only; no driver deadlines qualified."}));
        println!(
            "case {scenario}: {:.2}% one core, p99 {:.1} us, max {:.1} us",
            mean / 5333.333 * 100.,
            times[371],
            times[374]
        );
    }
    fs::create_dir_all("target/eis-validation")?;
    fs::write(
        "target/eis-validation/profile.json",
        serde_json::to_string_pretty(&report)?,
    )?;
    Ok(())
}
