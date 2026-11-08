// SPDX-License-Identifier: GPL-3.0-or-later
mod native;
use spasm_core::{Effect, Engine, Event, Preset, Transport, factory};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use vst3_host::{
    Vst3Host,
    audio::{AudioBuffers, write_wav},
    midi::{MidiChannel, MidiController, MidiEvent},
};
type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;
fn text_words(s: &str) -> [[u32; 8]; 32] {
    let mut b = [0u8; 1024];
    let n = s.len().min(1024);
    b[..n].copy_from_slice(&s.as_bytes()[..n]);
    std::array::from_fn(|i| {
        std::array::from_fn(|j| {
            let p = (i * 8 + j) * 4;
            u32::from_le_bytes([b[p], b[p + 1], b[p + 2], b[p + 3]])
        })
    })
}
fn component(p: &Preset) -> Result<Vec<u8>> {
    let mut values = serde_json::Map::new();
    let ints = [
        "repeat_div",
        "stutter_div",
        "reverse_div",
        "gate_div",
        "chaos_div",
        "quantize",
        "repeat_count",
        "gate_pattern",
        "latch_mask",
        "chaos_mask",
        "seed",
        "midi_channel",
    ];
    for (k, v) in serde_json::to_value(p.config)?.as_object().unwrap() {
        if v.is_array() {
            continue;
        }
        let ty = if v.is_boolean() {
            "bool"
        } else if ints.contains(&k.as_str()) {
            "i32"
        } else {
            "f32"
        };
        values.insert(k.replace('_', "-"), serde_json::json!({ty:v}));
    }
    for (i, v) in p.config.midi_notes.iter().enumerate() {
        values.insert(format!("note-{}", i + 1), serde_json::json!({"i32":v}));
    }
    for (i, v) in p.config.midi_cc.iter().enumerate() {
        values.insert(format!("cc-{}", i + 1), serde_json::json!({"i32":v}));
    }
    for (i, v) in p.config.priority.iter().enumerate() {
        values.insert(format!("priority-{}", i + 1), serde_json::json!({"i32":v}));
    }
    for (i, v) in p.config.pad_assign.iter().enumerate() {
        values.insert(
            format!("assign-{}", i + 1),
            serde_json::json!({"i32":v.id()}),
        );
    }
    for (i, r) in p.config.routes.iter().enumerate() {
        for (k, v) in serde_json::to_value(r)?.as_object().unwrap() {
            let (ty, v) = match k.as_str() {
                "enabled" => ("bool", v.clone()),
                "source" => ("i32", serde_json::json!(r.source as usize)),
                "target" => ("i32", serde_json::json!(r.target as usize)),
                _ => ("f32", v.clone()),
            };
            values.insert(format!("r{}-{}", i + 1, k), serde_json::json!({ty:v}));
        }
    }
    Ok(serde_json::to_vec(
        &serde_json::json!({"version":env!("CARGO_PKG_VERSION"),"params":values,"fields":{"schema":"1","midi-xy":"[-1.0,-1.0]","preset-name":serde_json::to_string(&text_words(&p.name))?,"preset-notes":serde_json::to_string(&text_words(&p.description))?}}),
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
                .unwrap_or_else(|| "target/spasm-demos".into()),
        ),
        Some("profile") => profile(),
        Some("smoke") | Some("editor") => host(&a),
        _ => {
            println!(
                "spasm-lab render [OUT] | profile | smoke BUNDLE [OUT] | editor BUNDLE [SECONDS WIDTH HEIGHT edit]"
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
    let mut plugin = host.load_plugin(a.get(2).ok_or("Missing bundle")?)?;
    if plugin.info().name != "SPASM" {
        return Err("Select SPASM explicitly".into());
    }
    if a[1] == "editor" {
        let seconds: u64 = a.get(3).map(|s| s.parse()).transpose()?.unwrap_or(30);
        if seconds > 600 {
            return Err("Preview maximum is 600 seconds".into());
        }
        if let (Some(w), Some(h)) = (a.get(4), a.get(5)) {
            let w: u32 = w.parse()?;
            let h: u32 = h.parse()?;
            if !(920..=1920).contains(&w) || !(760..=1440).contains(&h) {
                return Err("Preview size outside supported bounds".into());
            }
            plugin.load_state(&serde_json::to_vec(&serde_json::json!({"version":"0.1.0","params":{},"fields":{"edit":(a.get(6).map(String::as_str)==Some("edit")).to_string(),"editor":serde_json::to_string(&serde_json::json!({"size":[w,h]}))?}}))?)?;
        }
        let plugin = std::sync::Arc::new(std::sync::Mutex::new(plugin));
        let mut window = vst3_host::window::PluginWindow::new(plugin.clone());
        window.open()?;
        // Real plugin processing, discarded offline: visible meters/spectrum, no device.
        plugin.lock().unwrap().start_processing()?;
        let demo = a.get(7).map(String::as_str) == Some("demo");
        let demo_ids = plugin.lock().unwrap().get_parameters()?;
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
                if demo && frame < 48000 && frame + 256 >= 48000 {
                    let mut p = plugin.lock().unwrap();
                    p.send_midi_event_at(
                        MidiEvent::NoteOn {
                            channel: MidiChannel::Ch1,
                            note: 36,
                            velocity: 110,
                        },
                        0,
                    )?;
                    for (name, value) in [("X", 0.58), ("Y", 0.37)] {
                        if let Some(id) = demo_ids.iter().find(|p| p.name == name) {
                            p.set_parameter_at(id.id, value, 0)?;
                        }
                    }
                }
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
        fs::create_dir_all("target/spasm-validation")?;
        fs::write(
            "target/spasm-validation/editor-gestures.json",
            serde_json::to_string_pretty(&edits)?,
        )?;
        window.close();
        return Ok(());
    }

    let out = a
        .get(3)
        .map(PathBuf::from)
        .unwrap_or_else(|| "target/spasm-host".into());
    fs::create_dir_all(&out)?;
    let buses = plugin.audio_bus_layout()?;
    if buses.inputs.len() != 1
        || buses.outputs.len() != 1
        || buses.inputs[0].channel_count != 2
        || buses.outputs[0].channel_count != 2
    {
        return Err("Expected stereo effect".into());
    }
    let params = plugin.get_parameters()?;
    let id = |n: &str| -> Result<u32> {
        Ok(params
            .iter()
            .find(|p| p.name == n)
            .ok_or("Missing parameter")?
            .id)
    };
    let pads: Vec<_> = spasm_core::patch::PAD_NAMES
        .iter()
        .map(|n| id(n))
        .collect::<Result<_>>()?;
    let xid = id("X")?;
    let yid = id("Y")?;
    let mut report = Vec::new();
    for scenario in 0..8 {
        let mut p = factory(if scenario == 0 { 0 } else { scenario + 1 });
        if scenario == 2 {
            p.name = "USER CHOP".into();
            p.config.quantize = 0;
            p.config.stutter_ms = 23.;
            p.config.pad_assign[0] = Effect::Stutter;
        }
        let mut fixture: serde_json::Value = serde_json::from_slice(&component(&p)?)?;
        if scenario == 7 {
            fixture["fields"]["midi-xy"] = serde_json::json!("[0.77,0.29]");
        }
        plugin.load_state(&serde_json::to_vec(&fixture)?)?;
        let state = plugin.save_state()?;
        fs::write(out.join(format!("case-{scenario}.state")), &state)?;
        for pass in 0..2 {
            plugin.load_state(&state)?;
            plugin.start_processing()?;
            let mut c = p.config;
            if scenario == 7 {
                c.x = 0.77;
                c.y = 0.29;
            }
            let mut e = Engine::new(48000., c);
            let mut buffers = AudioBuffers::new(2, 2, 256, 48000.);
            let mut gates = [false; 8];
            let mut beat = 0.;
            let mut transport_sample = 0;
            let mut error = 0f32;
            let mut first = None;
            let mut audio = vec![Vec::new(), Vec::new()];
            let mut changed = 0f64;
            for block in 0..750 {
                let bpm = if block < 400 { 138. } else { 173. };
                let sample = block * 256;
                let playing =
                    scenario != 6 || !((200..240).contains(&block) || (430..470).contains(&block));
                let pos = if scenario == 5 && block >= 500 {
                    sample + 48000
                } else if scenario == 6 {
                    transport_sample
                } else {
                    sample
                };
                plugin.set_tempo(bpm)?;
                plugin.set_playing(playing)?;
                plugin.set_transport_position(vst3_host::plugin::TransportPosition {
                    samples: pos as i64,
                    quarter_notes: beat,
                })?;
                e.set_transport(Transport {
                    bpm: Some(bpm),
                    beat: Some(beat),
                    sample: Some(pos as i64),
                    playing: Some(playing),
                });
                // Actual VST3 automation and MIDI carry within-block offsets.
                for (b, off, pad, down) in [
                    (100, 37, 0, true),
                    (220, 91, 1, true),
                    (280, 43, 1, false),
                    (330, 19, 0, false),
                    (350, 99, 2, true),
                    (420, 47, 2, false),
                    (450, 23, 3, true),
                    (520, 61, 3, false),
                    (550, 7, 4, true),
                    (590, 73, 5, true),
                    (630, 13, 4, false),
                    (650, 29, 5, false),
                    (675, 81, 6, true),
                    (710, 57, 7, true),
                    (720, 11, 7, false),
                ] {
                    if block == b {
                        if scenario == 1 || scenario == 3 {
                            plugin.send_midi_event_at(
                                if down {
                                    MidiEvent::NoteOn {
                                        channel: MidiChannel::Ch1,
                                        note: c.midi_notes[pad],
                                        velocity: 110,
                                    }
                                } else {
                                    MidiEvent::NoteOff {
                                        channel: MidiChannel::Ch1,
                                        note: c.midi_notes[pad],
                                        velocity: 0,
                                    }
                                },
                                off,
                            )?;
                        } else {
                            plugin.set_parameter_at(pads[pad], if down { 1. } else { 0. }, off)?;
                        }
                    }
                }
                if block == 180 {
                    if scenario == 3 {
                        plugin.send_midi_controller_at(
                            0,
                            MidiChannel::Ch1,
                            MidiController::Midi1(c.midi_cc[0] as u16),
                            94. / 127.,
                            53,
                        )?;
                    } else {
                        plugin.set_parameter_at(xid, 0.7, 53)?;
                    }
                }
                if block == 260 {
                    plugin.set_parameter_at(yid, 0.6, 67)?;
                }
                for i in 0..256 {
                    let x = input(sample + i, 48000);
                    buffers.inputs[0][i] = x[0];
                    buffers.inputs[1][i] = x[1];
                }
                plugin.process_audio(&mut buffers)?;
                for i in 0..256 {
                    for (b, off, pad, down) in [
                        (100, 37, 0, true),
                        (220, 91, 1, true),
                        (280, 43, 1, false),
                        (330, 19, 0, false),
                        (350, 99, 2, true),
                        (420, 47, 2, false),
                        (450, 23, 3, true),
                        (520, 61, 3, false),
                        (550, 7, 4, true),
                        (590, 73, 5, true),
                        (630, 13, 4, false),
                        (650, 29, 5, false),
                        (675, 81, 6, true),
                        (710, 57, 7, true),
                        (720, 11, 7, false),
                    ] {
                        if block == b && i == off as usize {
                            if scenario == 1 || scenario == 3 {
                                e.event(Event::Note {
                                    note: c.midi_notes[pad],
                                    channel: 0,
                                    on: down,
                                    velocity: 110. / 127.,
                                });
                            } else {
                                gates[pad] = down;
                                e.set_gates(gates);
                            }
                        }
                    }
                    if block == 180 && i == 53 {
                        c.x = if scenario == 3 { 94. / 127. } else { 0.7 };
                        e.set_config(c);
                    }
                    if block == 260 && i == 67 {
                        c.y = 0.6;
                        e.set_config(c);
                    }
                    let expected = e.process(input(sample + i, 48000));
                    for ch in 0..2 {
                        let actual = buffers.outputs[ch][i];
                        if !actual.is_finite() || actual.abs() > 32. {
                            return Err("Invalid output".into());
                        }
                        let delta = (actual - expected[ch]).abs();
                        error = error.max(delta);
                        if delta > 2e-5 && first.is_none() {
                            first = Some(sample + i);
                        }
                        changed += (actual - input(sample + i, 48000)[ch]).abs() as f64;
                        audio[ch].push(actual);
                    }
                }
                if playing {
                    beat += 256. * bpm / (60. * 48000.);
                    transport_sample += 256;
                }
            }
            plugin.stop_processing()?;
            if error > 2e-5 {
                return Err(format!(
                    "Host/core mismatch case {scenario} pass {pass}: {error}, first {first:?}"
                )
                .into());
            }
            if changed < 1. {
                return Err("No actual processed audio".into());
            }
            if plugin.latency_samples() != 0 {
                return Err("Expected zero latency".into());
            }
            write_wav(
                out.join(format!("case-{scenario}-pass-{pass}.wav")),
                &audio,
                48000,
            )?;
            report.push(serde_json::json!({"case":scenario,"pass":pass,"frames":audio[0].len(),"maximum_error":error,"latency":plugin.latency_samples(),"difference_from_dry":changed}));
        }
    }
    fs::write(
        out.join("report.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    println!("SPASM actual stereo/MIDI/CC/automation/state reference comparison passed");
    Ok(())
}
fn render(out: PathBuf) -> Result {
    fs::create_dir_all(&out)?;
    fs::create_dir_all("factory/spasm")?;
    let frames = 480000;
    let dry: Vec<Vec<f32>> = (0..2)
        .map(|ch| (0..frames).map(|i| input(i, 48000)[ch]).collect())
        .collect();
    write_wav(out.join("00-original-input.wav"), &dry, 48000)?;
    let mut manifest = Vec::new();
    for index in 0..17 {
        let mut p = factory(index.min(9));
        if index >= 10 {
            p = factory(0);
            p.name = Effect::ALL[index - 9].name().into();
            p.config.quantize = 0;
            p.config.pad_assign[0] = Effect::ALL[index - 9];
        }
        p.validate()?;
        let stem = format!(
            "{:02}-{}",
            index + 1,
            p.name.to_lowercase().replace(' ', "-")
        );
        if index < 10 {
            fs::write(
                Path::new("factory/spasm").join(format!("{stem}.spasm.json")),
                p.to_json()?,
            )?;
        }
        let mut e = Engine::new(48000., p.config);
        e.set_transport(Transport {
            bpm: Some(138.),
            ..Default::default()
        });
        let mut audio = vec![Vec::new(), Vec::new()];
        let mut peak = 0f32;
        let mut delta = 0f64;
        let mut c = p.config;
        for frame in 0..frames {
            if frame == 48000 {
                e.event(Event::Pad {
                    pad: 0,
                    down: true,
                    velocity: 1.,
                });
            }
            if frame == 144000 {
                c.x = 0.85;
                c.y = 0.5;
                e.set_config(c);
            }
            if frame == 240000 {
                e.event(Event::Pad {
                    pad: 1,
                    down: true,
                    velocity: 1.,
                });
            }
            if frame == 288000 {
                e.event(Event::Pad {
                    pad: 1,
                    down: false,
                    velocity: 0.,
                });
                e.event(Event::Pad {
                    pad: 3,
                    down: true,
                    velocity: 1.,
                });
            }
            if frame == 336000 {
                e.event(Event::Reset);
            }
            if frame == 384000 {
                e.event(Event::Pad {
                    pad: 0,
                    down: false,
                    velocity: 0.,
                });
                e.event(Event::Pad {
                    pad: 6,
                    down: true,
                    velocity: 1.,
                });
            }
            if frame == 432000 {
                e.event(Event::Reset);
            }
            let x = input(frame, 48000);
            let y = e.process(x);
            for ch in 0..2 {
                audio[ch].push(y[ch]);
                peak = peak.max(y[ch].abs());
                delta += (y[ch] - x[ch]).abs() as f64;
            }
        }
        if peak < 0.001 || delta < 1. {
            return Err("Unusable silent/unchanged demo".into());
        }
        write_wav(out.join(format!("{stem}.wav")), &audio, 48000)?;
        manifest.push(serde_json::json!({"name":p.name,"peak":peak,"difference_from_dry":delta,"frames":frames,"rate":48000,"events":"1s press first pad;3s XY;5s reverse;6s dive;7s reset;8s chaos;9s reset"}));
    }
    fs::write(
        out.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    Ok(())
}
fn profile() -> Result {
    let mut report = Vec::new();
    for sr in [48000, 192000] {
        let mut c = factory(6).config;
        c.history_seconds = 8.;
        c.quantize = 0;
        c.capture_locked = false;
        let mut e = Engine::new(sr as f32, c);
        for i in 0..sr * 8 {
            std::hint::black_box(e.process(input(i, sr)));
        }
        let mut times = Vec::new();
        for block in 0..750 {
            let source: [_; 256] = std::array::from_fn(|i| input(block * 256 + i, sr));
            let start = Instant::now();
            e.event(Event::Pad {
                pad: (block % 7) as u8,
                down: block % 2 == 0,
                velocity: 1.,
            });
            if block % 19 == 0 {
                c.x = (block % 100) as f32 / 100.;
                c.y = 0.95;
                e.set_config(c);
            }
            for x in source {
                std::hint::black_box(e.process(x));
            }
            times.push(start.elapsed().as_secs_f64() * 1e6);
        }
        times.sort_by(f64::total_cmp);
        let mean = times.iter().sum::<f64>() / 750.;
        report.push(serde_json::json!({"rate":sr,"mean_core_percent":mean/(256e6/sr as f64)*100.,"p99_us":times[742],"maximum_us":times[749],"engine_bytes":e.memory_bytes(),"block":256,"blocks":750,"environment":"Shared desktop offline; device deadlines not qualified"}));
    }
    fs::create_dir_all("target/spasm-validation")?;
    fs::write(
        "target/spasm-validation/profile.json",
        serde_json::to_string_pretty(&report)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
