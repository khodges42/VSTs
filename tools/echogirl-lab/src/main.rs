// SPDX-License-Identifier: GPL-3.0-or-later
mod native;
use echogirl_core::{
    Config, Engine, Transport, factory,
    preset::{Route, Target},
};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use vst3_host::{
    Vst3Host,
    audio::{AudioBuffers, write_wav},
    midi::{MidiChannel, MidiController},
};

// (block, frame offset, parameter slot, normalized value); slot 8 is a MIDI X CC.
const GESTURES: [(usize, usize, usize, f32); 14] = [
    (30, 37, 6, 0.),
    (30, 37, 2, 1.),
    (48, 91, 2, 0.),
    (100, 91, 0, 0.85),
    (130, 75, 1, 0.95),
    (180, 37, 8, 0.2),
    (250, 17, 3, 1.),
    (400, 42, 5, 1.),
    (410, 91, 5, 0.),
    (420, 33, 3, 0.),
    (560, 0, 6, 1.),
    (650, 5, 4, 1.),
    (652, 127, 4, 0.),
    (700, 30, 0, 0.5),
];
fn gesture(c: &mut Config, slot: usize, value: f32) {
    match slot {
        0 | 8 => c.x = value,
        1 => c.y = value,
        2 => {
            c.throw = value >= 0.5;
            if c.throw {
                c.feed = false;
            }
        }
        3 => c.hold = value >= 0.5,
        4 => {
            c.kill = value >= 0.5;
            if c.kill {
                c.hold = false;
                c.throw = false;
            }
        }
        5 => c.dub = value >= 0.5,
        6 => c.feed = value >= 0.5,
        _ => unreachable!(),
    }
}

fn component(c: Config) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut params = serde_json::Map::new();
    let value = serde_json::to_value(c)?;
    for (key, v) in value.as_object().unwrap() {
        if ["mappings", "midi_cc"].contains(&key.as_str()) {
            continue;
        }
        let kind = if v.is_boolean() {
            "bool"
        } else if v.as_number().unwrap().is_f64() {
            "f32"
        } else {
            "i32"
        };
        params.insert(key.replace('_', "-"), serde_json::json!({kind:v}));
    }
    for (i, m) in c.mappings.iter().enumerate() {
        for (key, value) in [
            ("route-on", serde_json::json!({"bool":m.enabled})),
            (
                "route-source",
                serde_json::json!({"i32":if m.source==echogirl_core::preset::Source::X{0}else{1}}),
            ),
            (
                "route-target",
                serde_json::json!({"i32":Target::ALL.iter().position(|t|*t==m.target).unwrap_or(0)}),
            ),
            ("route-depth", serde_json::json!({"f32":m.amount})),
            ("route-curve", serde_json::json!({"f32":m.curve})),
        ] {
            params.insert(format!("{key}_{}", i + 1), value);
        }
    }
    for (i, cc) in c.midi_cc.iter().enumerate() {
        params.insert(format!("cc_{}", i + 1), serde_json::json!({"i32":cc}));
    }
    Ok(serde_json::to_vec(
        &serde_json::json!({"version":env!("CARGO_PKG_VERSION"),"params":params,"fields":{"schema":"1"}}),
    )?)
}
fn input(frame: usize, sr: usize) -> [f32; 2] {
    let t = frame as f32 / sr as f32;
    if t >= 4.5 {
        return [0.; 2];
    }
    let beat = (t * 2.).fract();
    let env = (-beat * 45.).exp();
    let mut n = (frame as u32)
        .wrapping_mul(747796405)
        .wrapping_add(2891336453);
    n ^= n >> 16;
    let noise = n as f32 / u32::MAX as f32 * 2. - 1.;
    let chord = [130.8128, 155.5635, 195.9977]
        .iter()
        .map(|f| (t * std::f32::consts::TAU * f).sin())
        .sum::<f32>()
        * 0.07
        * (-beat * 8.).exp();
    [noise * env * 0.22 + chord, noise * env * 0.18 + chord * 0.8]
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("render") => render(
            args.get(2)
                .map(PathBuf::from)
                .unwrap_or_else(|| "target/echogirl-demos".into()),
        ),
        Some("profile") => profile(),
        Some("smoke") | Some("editor") => host(&args),
        _ => {
            println!(
                "echogirl-lab render [OUTPUT] | profile | smoke BUNDLE [OUTPUT] | editor BUNDLE [SECONDS [WIDTH HEIGHT]]"
            );
            Ok(())
        }
    }
}
fn host(args: &[String]) -> Result<(), Box<dyn Error>> {
    let _ole = native::Ole::new()?;
    let mut host = Vst3Host::builder()
        .sample_rate(48000.)
        .block_size(256)
        .build()?;
    let mut plugin = host.load_plugin(args.get(2).ok_or("Missing VST3 bundle")?)?;
    if plugin.info().name != "ECHOGIRL" {
        return Err("Select the ECHOGIRL plugin explicitly".into());
    }
    if args[1] == "editor" {
        let secs: u64 = args.get(3).map(|s| s.parse()).transpose()?.unwrap_or(30);
        if secs > 600 {
            return Err("Preview must be <=600 seconds".into());
        }
        if let (Some(w), Some(h)) = (args.get(4), args.get(5)) {
            let w: u32 = w.parse()?;
            let h: u32 = h.parse()?;
            if !(820..=1920).contains(&w) || !(700..=1440).contains(&h) {
                return Err("Unsupported preview size".into());
            }
            plugin.load_state(&serde_json::to_vec(&serde_json::json!({"version":"0.1.0","params":{},"fields":{"editor":serde_json::to_string(&serde_json::json!({"size":[w,h]}))?}}))?)?;
        }
        let p = std::sync::Arc::new(std::sync::Mutex::new(plugin));
        let mut window = vst3_host::window::PluginWindow::new(p);
        window.open()?;
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(secs) && !window.closed_by_user() {
            native::pump();
            window.service_platform_events()?;
            std::thread::sleep(Duration::from_millis(8));
        }
        window.close();
        return Ok(());
    }
    let out = args
        .get(3)
        .map(PathBuf::from)
        .unwrap_or_else(|| "target/echogirl-host".into());
    fs::create_dir_all(&out)?;
    let buses = plugin.audio_bus_layout()?;
    if buses.inputs.len() != 1
        || buses.outputs.len() != 1
        || buses.inputs[0].channel_count != 2
        || buses.outputs[0].channel_count != 2
    {
        return Err("Expected stereo effect input/output".into());
    }
    let mut report = Vec::new();
    let parameters = plugin.get_parameters()?;
    let mut ids = [0; 7];
    for (i, name) in ["X", "Y", "THROW", "HOLD", "KILL", "DUB", "FEED"]
        .iter()
        .enumerate()
    {
        ids[i] = parameters
            .iter()
            .find(|p| p.name == *name)
            .ok_or("Missing performance parameter")?
            .id;
    }
    for scenario in 0..3 {
        let mut c = if scenario == 0 {
            Config {
                character: 0,
                sync: false,
                time_ms: 100.,
                right_ms: 150.,
                link: false,
                feedback: 0.45,
                drive: 0.,
                mix: 0.4,
                low_cut: 0.,
                high_cut: 20000.,
                mappings: [Route::default(); 8],
                ..Config::default()
            }
        } else if scenario == 1 {
            factory(2).config
        } else {
            factory(0).config
        };
        c.release_buttons();
        plugin.load_state(&component(c)?)?;
        let saved = plugin.save_state()?;
        fs::write(out.join(format!("scenario-{scenario}.state")), &saved)?;
        for pass in 0..2 {
            plugin.load_state(&saved)?;
            plugin.set_tempo(120.)?;
            plugin.set_playing(true)?;
            plugin.start_processing()?;
            let mut engine = Engine::new(48000., c);
            let mut reference_config = c;
            let mut buffers = AudioBuffers::new(2, 2, 256, 48000.);
            let mut audio = vec![Vec::new(), Vec::new()];
            let mut error = 0_f32;
            let mut first_mismatch = None;
            let mut peak = 0_f32;
            for block in 0..750 {
                let bpm = if scenario == 2 && block >= 350 {
                    178.
                } else {
                    120.
                };
                let playing = !(scenario == 2 && (450..475).contains(&block));
                plugin.set_tempo(bpm)?;
                plugin.set_playing(playing)?;
                if scenario == 2 {
                    for (_, offset, slot, value) in GESTURES.iter().filter(|g| g.0 == block) {
                        if *slot == 8 {
                            plugin.send_midi_controller_at(
                                0,
                                MidiChannel::Ch1,
                                MidiController::Midi1(16),
                                *value as f64,
                                *offset as i32,
                            )?;
                        } else {
                            plugin.set_parameter_at(ids[*slot], *value as f64, *offset as i32)?;
                        }
                    }
                }
                buffers.clear();
                for i in 0..256 {
                    let x = input(block * 256 + i, 48000);
                    buffers.inputs[0][i] = x[0];
                    buffers.inputs[1][i] = x[1];
                }
                plugin.set_transport_position(vst3_host::plugin::TransportPosition {
                    samples: (block * 256) as i64,
                    quarter_notes: (block * 256) as f64 / 24000.,
                })?;
                plugin.process_audio(&mut buffers)?;
                engine.set_transport(Transport {
                    bpm: bpm as f32,
                    playing,
                    beat: Some((block * 256) as f64 / 24000.),
                });
                for i in 0..256 {
                    if scenario == 2 {
                        for (_, _, slot, value) in
                            GESTURES.iter().filter(|g| g.0 == block && g.1 == i)
                        {
                            gesture(&mut reference_config, *slot, *value);
                            engine.set_config(reference_config);
                        }
                    }
                    let expected = engine.process(input(block * 256 + i, 48000));
                    for (ch, value) in expected.iter().enumerate() {
                        let actual = buffers.outputs[ch][i];
                        if !actual.is_finite() || actual.abs() > 1.0001 {
                            return Err("Unsafe VST3 output".into());
                        }
                        error = error.max((actual - value).abs());
                        if (actual - value).abs() > 2e-5 && first_mismatch.is_none() {
                            first_mismatch = Some((block * 256 + i, ch, actual, *value));
                        }
                        peak = peak.max(actual.abs());
                        audio[ch].push(actual);
                    }
                }
            }
            plugin.stop_processing()?;
            if error > 2e-5 || peak < 0.001 {
                return Err(format!("VST3/core mismatch: scenario={scenario}, pass={pass}, error={error}, peak={peak}, first={first_mismatch:?}").into());
            }
            if plugin.latency_samples() != 0 {
                return Err("Unexpected dry-path latency".into());
            }
            write_wav(
                out.join(format!("scenario-{scenario}-pass-{pass}.wav")),
                &audio,
                48000,
            )?;
            report.push(serde_json::json!({"scenario":scenario,"pass":pass,"max_error":error,"peak":peak,"latency":plugin.latency_samples(),"frames":audio[0].len()}));
        }
    }
    fs::write(
        out.join("report.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    println!("ECHOGIRL VST3 stereo audio/state reference smoke passed");
    Ok(())
}
fn render(out: PathBuf) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(&out)?;
    let bank = Path::new("factory/echogirl");
    fs::create_dir_all(bank)?;
    let mut report = Vec::new();
    for index in 0..10 {
        let p = factory(index);
        p.validate()?;
        let stem = format!(
            "{:02}-{}",
            index + 1,
            p.name.to_lowercase().replace(' ', "-")
        );
        fs::write(bank.join(format!("{stem}.echogirl.json")), p.to_json()?)?;
        let mut e = Engine::new(48000., p.config);
        e.set_transport(Transport {
            bpm: 120.,
            ..Transport::default()
        });
        e.reset();
        let mut audio = vec![Vec::new(), Vec::new()];
        let mut peak = 0_f32;
        let mut energy = 0_f64;
        for frame in 0..384000 {
            let t = frame as f32 / 48000.;
            if frame % 64 == 0 {
                let mut c = p.config;
                c.x = 0.5 + 0.45 * (t * 0.7).sin();
                c.y = (0.1 + t * 0.14).min(1.);
                c.throw = (0.45..0.6).contains(&t)
                    || (1.45..1.6).contains(&t)
                    || (2.95..3.1).contains(&t);
                c.hold = (4. ..5.5).contains(&t);
                c.kill = (6.5..6.7).contains(&t);
                c.dub = (3.5..4.).contains(&t);
                e.set_config(c);
            }
            let y = e.process(input(frame, 48000));
            for ch in 0..2 {
                peak = peak.max(y[ch].abs());
                energy += (y[ch] as f64).powi(2);
                audio[ch].push(y[ch]);
            }
        }
        if peak < 0.001 {
            return Err("Silent factory render".into());
        }
        write_wav(out.join(format!("{stem}.wav")), &audio, 48000)?;
        report.push(serde_json::json!({"preset":p.name,"peak":peak,"rms":(energy/768000.).sqrt(),"bpm":120,"sample_rate":48000}));
        println!("{} peak={peak:.3}", p.name);
    }
    fs::write(
        out.join("manifest.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    Ok(())
}
fn profile() -> Result<(), Box<dyn Error>> {
    let mut report = Vec::new();
    for character in 0..4 {
        let c = Config {
            character,
            feedback: 1.08,
            y: 1.,
            wow: 1.,
            spring: 0.65,
            ..Config::default()
        };
        let mut e = Engine::new(48000., c);
        for _ in 0..4096 {
            std::hint::black_box(e.process([0.2, -0.1]));
        }
        let mut times = Vec::with_capacity(375);
        for _ in 0..375 {
            let start = Instant::now();
            for _ in 0..256 {
                std::hint::black_box(e.process([0.2, -0.1]));
            }
            times.push(start.elapsed().as_secs_f64() * 1e6);
        }
        times.sort_by(f64::total_cmp);
        let mean = times.iter().sum::<f64>() / 375.;
        println!(
            "character {character}: {:.2}% of one core, p99 {:.1} us, max {:.1} us",
            mean / 5333.333 * 100.,
            times[371],
            times[374]
        );
        report.push(serde_json::json!({"character":character,"memory_bytes":e.memory_bytes(),"mean_core_percent":mean/5333.333*100.,"p99_us":times[371],"max_us":times[374],"blocks":375,"block_size":256,"rate":48000,"environment":"Uncontrolled shared desktop load; offline engine timing, not audio-device qualification."}));
    }
    fs::create_dir_all("target/echogirl-validation")?;
    fs::write(
        "target/echogirl-validation/profile.json",
        serde_json::to_string_pretty(&report)?,
    )?;
    Ok(())
}
