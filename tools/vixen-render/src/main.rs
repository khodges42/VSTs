// SPDX-License-Identifier: GPL-3.0-or-later
use std::{error::Error, fs, io::Write, path::PathBuf, time::Instant};
use vixen_core::{Controls, Engine, Event, factory};

fn wav(path: &std::path::Path, audio: &[[f32; 2]], rate: u32) -> std::io::Result<()> {
    let mut f = std::io::BufWriter::new(fs::File::create(path)?);
    let bytes = audio.len() as u32 * 4;
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + bytes).to_le_bytes())?;
    f.write_all(b"WAVEfmt ")?;
    f.write_all(&16_u32.to_le_bytes())?;
    f.write_all(&1_u16.to_le_bytes())?;
    f.write_all(&2_u16.to_le_bytes())?;
    f.write_all(&rate.to_le_bytes())?;
    f.write_all(&(rate * 4).to_le_bytes())?;
    f.write_all(&4_u16.to_le_bytes())?;
    f.write_all(&16_u16.to_le_bytes())?;
    f.write_all(b"data")?;
    f.write_all(&bytes.to_le_bytes())?;
    for sample in audio.iter().flatten() {
        f.write_all(&((sample.clamp(-1., 1.) * 32767.).round() as i16).to_le_bytes())?;
    }
    f.flush()
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let mut out = PathBuf::from("target/demos");
    let mut bank = None;
    let mut rate = 48000_u32;
    let mut profile = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => out = PathBuf::from(args.next().ok_or("--out needs a path")?),
            "--bank" => bank = Some(PathBuf::from(args.next().ok_or("--bank needs a path")?)),
            "--rate" => rate = args.next().ok_or("--rate needs Hz")?.parse()?,
            "--profile" => {
                profile = Some(PathBuf::from(
                    args.next().ok_or("--profile needs an output JSON path")?,
                ))
            }
            "--help" => {
                println!(
                    "vixen-render [--out target/demos] [--bank factory/vixen] [--rate 48000] [--profile target/profile.json]\nWrites ten default and ten moving-X/Y stereo WAVs and a measured manifest. --profile runs only a synthetic offline CPU trial. Never opens an audio device."
                );
                return Ok(());
            }
            _ => return Err(format!("Unknown argument {arg}").into()),
        }
    }
    if !(8000..=192000).contains(&rate) {
        return Err("Sample rate must be 8000..192000".into());
    }
    if let Some(path) = profile {
        return profile_engine(path, rate);
    }
    fs::create_dir_all(&out)?;
    if let Some(bank) = &bank {
        fs::create_dir_all(bank)?;
    }
    let mut report = Vec::new();
    for index in 0..10 {
        let patch = factory::patch(index);
        patch.validate()?;
        let stem = format!(
            "{:02}-{}",
            index + 1,
            patch.name.to_lowercase().replace(' ', "-")
        );
        if let Some(bank) = &bank {
            fs::write(bank.join(format!("{stem}.vixen.json")), patch.to_json()?)?;
        }
        for sweep in [false, true] {
            let mut engine = Engine::new(rate as f32, patch.synth);
            engine.set_controls(patch.controls);
            let frames = rate as usize * 8;
            let mut audio = Vec::with_capacity(frames);
            let step = (rate as f32 * 60. / 145. / 2.).round() as usize;
            let mut notes = Vec::new();
            let mut peak = 0_f32;
            let mut sum = 0_f64;
            let mut delta = 0_f32;
            let mut last = [0.; 2];
            for frame in 0..frames {
                if frame % step == 0 {
                    for note in notes.drain(..) {
                        engine.event(Event::NoteOff {
                            note,
                            channel: 0,
                            id: None,
                        });
                    }
                    if frame < rate as usize * 6 {
                        let pattern = [0, 0, 7, 3, 0, 10, 7, 3, 0, 12, 7, 3, 10, 7, 3, 0];
                        let root = if [2, 3, 4, 8].contains(&index) {
                            48
                        } else {
                            60
                        };
                        let note = root + pattern[(frame / step) % pattern.len()];
                        let chord: &[u8] = if index == 1 { &[0, 3, 7] } else { &[0] };
                        for interval in chord {
                            let note = note + interval;
                            engine.event(Event::NoteOn {
                                note,
                                channel: 0,
                                velocity: if (frame / step).is_multiple_of(4) {
                                    1.
                                } else {
                                    0.7
                                },
                                id: None,
                            });
                            notes.push(note);
                        }
                    }
                }
                if frame % 64 == 0 {
                    let t = frame as f32 / rate as f32;
                    let c = if sweep {
                        Controls {
                            x: 0.5 + 0.48 * (t * 0.87).sin(),
                            y: 0.5 + 0.48 * (t * 0.61 - 0.8).sin(),
                            ..patch.controls
                        }
                    } else {
                        patch.controls
                    };
                    engine.set_controls(c);
                }
                let x = engine.next(145.);
                for ch in 0..2 {
                    if !x[ch].is_finite() {
                        return Err("Nonfinite render".into());
                    }
                    peak = peak.max(x[ch].abs());
                    sum += (x[ch] as f64).powi(2);
                    delta = delta.max((x[ch] - last[ch]).abs());
                }
                last = x;
                audio.push(x);
            }
            let filename = format!("{stem}-{}.wav", if sweep { "xy" } else { "default" });
            wav(&out.join(&filename), &audio, rate)?;
            let rms = (sum / (frames * 2) as f64).sqrt();
            if peak < 0.001 {
                return Err(format!("Silent patch {}", patch.name).into());
            }
            report.push(serde_json::json!({"file":filename,"patch":patch.name,"sample_rate":rate,"frames":frames,"bpm":145,"xy_sweep":sweep,"peak":peak,"rms":rms,"max_sample_delta":delta}));
            println!("{filename}: peak={peak:.4} rms={rms:.4}");
        }
    }
    fs::write(
        out.join("manifest.json"),
        serde_json::to_string_pretty(&report)?,
    )?;
    Ok(())
}

fn profile_engine(path: PathBuf, rate: u32) -> Result<(), Box<dyn Error>> {
    let mut report = Vec::new();
    for voices in [1, 4, 12, 32] {
        for worst in [false, true] {
            let mut s = factory::patch(1).synth;
            s.voices = voices;
            if worst {
                for o in &mut s.oscillators {
                    o.unison = 7;
                    o.level = 0.5;
                }
                s.phase_mod = 3.;
                s.resonance = 0.95;
                s.drive = 1.;
            }
            let mut e = Engine::new(rate as f32, s);
            for i in 0..voices {
                e.event(Event::NoteOn {
                    note: 36 + i,
                    channel: 0,
                    velocity: 0.9,
                    id: Some(i as i32),
                });
            }
            for _ in 0..4096 {
                std::hint::black_box(e.next(145.));
            }
            let block = 256;
            let count = rate as usize * 2 / block;
            let mut us = Vec::with_capacity(count);
            let mut total = 0.;
            for i in 0..count {
                e.set_controls(Controls {
                    x: (i % 100) as f32 / 100.,
                    y: 0.8,
                    ..Controls::default()
                });
                let start = Instant::now();
                for _ in 0..block {
                    std::hint::black_box(e.next(145.));
                }
                let time = start.elapsed().as_secs_f64() * 1e6;
                total += time;
                us.push(time);
            }
            us.sort_by(f64::total_cmp);
            let budget = block as f64 / rate as f64 * 1e6;
            let realtime_fraction = total / (count as f64 * budget);
            report.push(serde_json::json!({"voices":voices,"worst_case_stack":worst,"sample_rate":rate,"block":block,"blocks":count,"memory_bytes":e.memory_bytes(),"p50_us":us[count/2],"p99_us":us[count*99/100],"max_us":us[count-1],"block_budget_us":budget,"mean_fraction_of_one_core":realtime_fraction,"environment":"Uncontrolled shared desktop load. Offline core trial, not a device deadline or MIDI-to-DAC latency measurement."}));
            println!(
                "voices={voices} worst={worst}: mean {:.1}% of one core; p99={:.1} us, max={:.1} us",
                realtime_fraction * 100.,
                us[count * 99 / 100],
                us[count - 1]
            );
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(&report)?)?;
    Ok(())
}
