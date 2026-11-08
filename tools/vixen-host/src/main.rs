// SPDX-License-Identifier: GPL-3.0-or-later
//! Offline development harness, never an audio callback or DAWJ dependency.
use std::{
    error::Error,
    fs,
    path::PathBuf,
    time::{Duration, Instant},
};
use vst3_host::{
    Vst3Host,
    audio::{AudioBuffers, write_wav},
    midi::{MidiChannel, MidiEvent},
};

// Unsafe code is limited to the development host's OLE apartment and Windows
// message pump. The plugin and DSP core forbid handwritten unsafe code.
#[cfg(windows)]
#[link(name = "ole32")]
unsafe extern "system" {
    fn OleInitialize(p: *mut std::ffi::c_void) -> i32;
    fn OleUninitialize();
}
struct Ole;
impl Ole {
    fn new() -> Result<Self, Box<dyn Error>> {
        #[cfg(windows)]
        if unsafe { OleInitialize(std::ptr::null_mut()) } < 0 {
            return Err("Cannot initialize OLE STA".into());
        }
        Ok(Self)
    }
}
impl Drop for Ole {
    fn drop(&mut self) {
        #[cfg(windows)]
        unsafe {
            OleUninitialize();
        }
    }
}
fn pump() {
    #[cfg(windows)]
    // All messages are owned by this UI thread; no plugin lock is held while
    // dispatching callbacks, and MSG storage outlives each native call.
    unsafe {
        use winapi::um::winuser::*;
        let mut msg = std::mem::zeroed();
        for _ in 0..1024 {
            if PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) == 0 {
                break;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() < 3 {
        return Err(
            "vixen-host smoke BUNDLE [OUTPUT] | idle BUNDLE OUTPUT [SECONDS RATE] | editor BUNDLE [SECONDS [WIDTH HEIGHT]]".into(),
        );
    }
    let _ole = Ole::new()?;
    let rate = if args[1] == "idle" {
        args.get(5)
            .map(|s| s.parse::<u32>())
            .transpose()?
            .unwrap_or(44100)
    } else {
        48000
    };
    if !(8000..=192000).contains(&rate) {
        return Err("Unsupported rate".into());
    }
    let mut host = Vst3Host::builder()
        .sample_rate(rate as f64)
        .block_size(256)
        .build()?;
    let mut plugin = host.load_plugin(&args[2])?;
    if plugin.info().name != "VIXEN"
        && !(args[1] == "idle" && ["ECHOGIRL", "SPASM"].contains(&plugin.info().name.as_str()))
    {
        return Err("Harness expects VIXEN, refusing an unrelated plugin".into());
    }
    if args[1] == "idle" {
        let seconds = args
            .get(4)
            .map(|s| s.parse::<u32>())
            .transpose()?
            .unwrap_or(1800);
        if !(30..=7200).contains(&seconds) {
            return Err("Idle duration must be 30..7200 seconds".into());
        }
        let out = args
            .get(3)
            .map(PathBuf::from)
            .unwrap_or_else(|| "target/idle-crash/soak".into());
        fs::create_dir_all(&out)?;
        let name = plugin.info().name.clone();
        let inputs = if name == "VIXEN" { 0 } else { 2 };
        let gate = plugin
            .get_parameters()?
            .into_iter()
            .find(|p| p.name == if name == "SPASM" { "REPEAT" } else { "THROW" })
            .map(|p| p.id);
        let mut buffers = AudioBuffers::new(inputs, 2, 256, rate as f64);
        plugin.set_tempo(120.)?;
        plugin.start_processing()?;
        let blocks = (rate as u64 * seconds as u64).div_ceil(256);
        let start = Instant::now();
        let mut peak = 0_f32;
        let mut idle_peak = 0_f32;
        let mut last_minute = 0;
        for block in 0..blocks {
            if block == 1 && inputs == 0 {
                plugin.send_midi_event_at(
                    MidiEvent::NoteOn {
                        channel: MidiChannel::Ch1,
                        note: 60,
                        velocity: 100,
                    },
                    37,
                )?;
            }
            if block == rate as u64 / 256 * 2 && inputs == 0 {
                plugin.send_midi_event_at(
                    MidiEvent::NoteOff {
                        channel: MidiChannel::Ch1,
                        note: 60,
                        velocity: 0,
                    },
                    91,
                )?;
            }
            buffers.clear();
            if let Some(gate) = gate {
                if block == rate as u64 / 256 {
                    plugin.set_parameter_at(gate, 1., 37)?;
                }
                if block == rate as u64 / 256 * 2 {
                    plugin.set_parameter_at(gate, 0., 91)?;
                }
            }
            if block * 256 < rate as u64 * 2 {
                for channel in &mut buffers.inputs {
                    for (i, x) in channel.iter_mut().enumerate() {
                        *x = (std::f64::consts::TAU * 220. * (block * 256 + i as u64) as f64
                            / rate as f64)
                            .sin() as f32
                            * 0.2;
                    }
                }
            }
            plugin.process_audio(&mut buffers)?;
            for ch in &buffers.outputs {
                for y in ch {
                    if !y.is_finite() || y.abs() > 1.01 {
                        return Err("Nonfinite or runaway output during idle soak".into());
                    }
                    peak = peak.max(y.abs());
                    if block * 256 > rate as u64 * 30 {
                        idle_peak = idle_peak.max(y.abs());
                    }
                }
            }
            let minute = block * 256 / (rate as u64 * 60);
            if minute > last_minute {
                println!("Processed {minute} audio minutes at {rate} Hz");
                last_minute = minute;
            }
        }
        plugin.stop_processing()?;
        if peak < 0.001 || idle_peak > 1e-4 {
            return Err(format!("Unexpected sounding/idle levels: {peak}/{idle_peak}").into());
        }
        let report = serde_json::json!({"product":name,"input_channels":inputs,"rate":rate,"requested_audio_seconds":seconds,"frames":blocks*256,"block":256,"press_offset":37,"release_offset":91,"peak":peak,"idle_peak_after_30_seconds":idle_peak,"elapsed_seconds":start.elapsed().as_secs_f64(),"latency":plugin.latency_samples(),"host":"Independent VST3 host; offline simulated audio time, no device or GUI"});
        fs::write(
            out.join(format!("idle-{rate}.json")),
            serde_json::to_string_pretty(&report)?,
        )?;
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }
    if args[1] == "editor" {
        let seconds: u64 = args.get(3).map(|s| s.parse()).transpose()?.unwrap_or(30);
        if seconds > 3600 {
            return Err("Seconds must be <= 3600".into());
        }
        if let (Some(width), Some(height)) = (args.get(4), args.get(5)) {
            let width: u32 = width.parse()?;
            let height: u32 = height.parse()?;
            if !(840..=1920).contains(&width) || !(660..=1440).contains(&height) {
                return Err("Preview size must be 840..1920 x 660..1440".into());
            }
            // VIXEN's pinned NIH component-state schema, accepted as a legacy raw
            // component stream by this offline host. Never rewrite opaque blobs.
            let state = serde_json::json!({"version":env!("CARGO_PKG_VERSION"),"params":{},"fields":{"editor":serde_json::to_string(&serde_json::json!({"size":[width,height]}))?}});
            plugin.load_state(&serde_json::to_vec(&state)?)?;
        }
        let plugin = std::sync::Arc::new(std::sync::Mutex::new(plugin));
        let mut window = vst3_host::window::PluginWindow::new(plugin.clone());
        window.open()?;
        let start = Instant::now();
        println!("VIXEN editor inspection, no audio device. Closing after {seconds}s.");
        while start.elapsed() < Duration::from_secs(seconds) && !window.closed_by_user() {
            pump();
            window.service_platform_events()?;
            std::thread::sleep(Duration::from_millis(10));
        }
        window.close();
        return Ok(());
    }
    if args[1] != "smoke" {
        return Err("Unknown command".into());
    }
    let output = args
        .get(3)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/host-smoke"));
    fs::create_dir_all(&output)?;
    let layout = plugin.audio_bus_layout()?;
    if !layout.inputs.is_empty()
        || layout.outputs.len() != 1
        || layout.outputs[0].channel_count != 2
    {
        return Err("Expected no audio input and one stereo output".into());
    }
    let params = plugin.get_parameters()?;
    let x = params
        .iter()
        .find(|p| p.name == "X")
        .ok_or("Missing X automation parameter")?
        .id;
    plugin.set_parameter(x, 0.8)?;
    let mut user = vixen_core::factory::patch(2);
    user.name = "SMOKE USER REESE".into();
    user.synth.oscillators[0].detune = 0.71;
    user.synth.mappings[0].amount = -0.8;
    user.validate()?;
    let fixture = serde_json::json!({"version":env!("CARGO_PKG_VERSION"),"params":{"xy-x":{"f32":0.8}},"fields":{"patch-v1":user.to_json()?}});
    plugin.load_state(&serde_json::to_vec(&fixture)?)?;
    let state = plugin.save_state()?;
    fs::write(output.join("saved.state"), &state)?;
    let mut buffers = AudioBuffers::new(0, 2, 256, 48000.);
    let mut report = Vec::new();
    for pass in 0..2 {
        // Restore only while stopped. Framework state parsing/lifecycle may allocate.
        plugin.load_state(&state)?;
        plugin.set_tempo(145.)?;
        plugin.start_processing()?;
        let mut audio = vec![Vec::new(), Vec::new()];
        let mut reference = vixen_core::Engine::new(48000., user.synth);
        reference.set_controls(vixen_core::Controls {
            x: 0.8,
            ..vixen_core::Controls::default()
        });
        reference.reset();
        let mut max_error = 0_f32;
        for block in 0..750 {
            if block == 1 {
                plugin.send_midi_event_at(
                    MidiEvent::NoteOn {
                        channel: MidiChannel::Ch1,
                        note: 60,
                        velocity: 110,
                    },
                    37,
                )?;
            }
            if block == 187 {
                plugin.send_midi_event_at(
                    MidiEvent::NoteOff {
                        channel: MidiChannel::Ch1,
                        note: 60,
                        velocity: 0,
                    },
                    91,
                )?;
            }
            buffers.clear();
            plugin.process_audio(&mut buffers)?;
            for i in 0..256 {
                let frame = block * 256 + i;
                if frame == 293 {
                    reference.event(vixen_core::Event::NoteOn {
                        note: 60,
                        channel: 0,
                        velocity: 110. / 127.,
                        id: None,
                    });
                }
                if frame == 187 * 256 + 91 {
                    reference.event(vixen_core::Event::NoteOff {
                        note: 60,
                        channel: 0,
                        id: None,
                    });
                }
                let expected = reference.next(145.);
                for (ch, value) in expected.iter().enumerate() {
                    max_error = max_error.max((buffers.outputs[ch][i] - value).abs());
                }
            }
            for (dest, src) in audio.iter_mut().zip(&buffers.outputs) {
                if src.iter().any(|x| !x.is_finite() || x.abs() > 1.) {
                    return Err("Invalid plugin audio".into());
                }
                dest.extend_from_slice(src);
            }
        }
        plugin.stop_processing()?;
        if audio.iter().flat_map(|c| &c[..293]).any(|x| *x != 0.) {
            return Err("Note sounded before its sample offset".into());
        }
        let peak = audio
            .iter()
            .flatten()
            .copied()
            .map(f32::abs)
            .fold(0., f32::max);
        if peak < 0.001 {
            return Err("Plugin produced no note audio".into());
        }
        if max_error > 1e-5 {
            return Err(format!(
                "VST3 user patch/state/audio differs from the reference by {max_error}"
            )
            .into());
        }
        let returned = plugin
            .get_parameters()?
            .into_iter()
            .find(|p| p.id == x)
            .ok_or("Missing restored X")?
            .value;
        if (returned - 0.8).abs() > 1e-5 {
            return Err("X failed state restoration".into());
        }
        write_wav(output.join(format!("vst3-pass-{pass}.wav")), &audio, 48000)?;
        report.push(serde_json::json!({"pass":pass,"peak":peak,"x":returned,"frames":audio[0].len(),"sample_offset":37,"reference_max_error":max_error,"user_patch":user.name,"latency_samples":plugin.latency_samples()}));
    }
    fs::write(
        output.join("report.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    println!(
        "VST3 note-offset/audio/state smoke passed: {}",
        output.display()
    );
    Ok(())
}
