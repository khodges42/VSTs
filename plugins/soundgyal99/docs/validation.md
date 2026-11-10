# Validation record - Windows x64 preview 0.3

Validated November 10, 2026 with MSVC 14.50 and JUCE 7.0.12. This records executed
checks rather than an audio-quality certification.

## Executed checks

- Release VST3 and both native test applications build successfully.
- CTest: all three suites pass: original_engines, plugin_state_factory_rates,
  realtime_synth.
- Original-engine probes: all 24 slots produce nontrivial finite/bounded audio,
  MAIN/AUX adapter is mono-compatible, voices release to silence, invalid engine
  and NaN patches reject, and caller buffer partitions preserve output.
- All 35 authored presets export/import into separate processors, restore every
  parameter exactly and reproduce sample-identical stereo audio. Two-second
  48 kHz/24-bit WAV examples and portable patches are generated.
- Rates 32, 44.1, 48, 88.2, 96 and 192 kHz pass. Deterministic sub patch renders
  match at 1-frame versus 127-frame host partitions.
- Eight-voice stress, stealing, extreme resonance/drive/bend, matrix audio effect,
  sustain pedal lifecycle, all-notes-off and release silence pass.
- Core render stress observes zero C++ new/new[] allocations. This is not an
  exhaustive CRT/driver allocation tracer or proof of every host callback.
- Native host loads the actual compiled VST3: discovery, 65 parameters, MIDI note
  offsets, finite stereo audio, release silence, state restored into a second
  VST3 instance, native editor attachment and capture pass.
- Independent pluginval 1.0.4 passes strictness **5 and 10**, random seed 99, with
  GUI tests enabled. Level 10 includes state restoration, background state,
  parameter thread safety and fuzzing. Rates 44.1/48/96 kHz and buffer sizes
  64/128/256/512/1024 are exercised by the validator. Steinberg's separate SDK
  validator was not supplied; pluginval explicitly skips that optional test.
- Existing Rust workspace fmt, locked offline check, Clippy with -D warnings,
  workspace tests and dependency inventory pass. No sibling source was changed.

## Preset revision checks

The 0.3 bank contains 35 patches. The legacy Hoover XML still restores its original
Swarm settings; factory reselection loads the corrected VA recipe. Sub-disabled
Hoover correlation improves from 0.183/0.284/0.332 to 0.998/1.000/0.998 at MIDI
48/60/72, with separately verified active movement routes. Before/after examples
and ten held-note auditions are generated. See presets-0.3.md for researched
sound-design references and the limits of these objective checks. No vendor DSP,
parameter IDs or schema changes were required.

## Reproduce

From VSTs, run scripts/build-vst3.ps1 as documented in README. This invokes CTest
and the actual VST3 host. Run external pluginval separately if available:

```powershell
target/pluginval/pluginval.exe --strictness-level 10 --random-seed 99 --output-dir target/soundgyal-cmake/pluginval-final --validate "target/soundgyal-cmake/Soundgyal99_artefacts/Release/VST3/SOUNDGYAL 99.vst3"
```

Full logs stay under target/soundgyal-cmake, including Testing/Temporary/LastTest.log
and pluginval-final/*.txt. WAVs and actual UI captures are in validation/. The
README images are copied from those actual captures, including the native VST3
window; they are not mockups. Development captures contain no unrelated windows.

## Known limitations and remaining release work

- No reference listening comparison, sustained human sound-design session, FL
  Studio test, DAWJ test or high-DPI/multi-monitor interaction review has occurred.
  The presets are technically verified, but their musical quality still needs
  listening review. No claim of polished commercial preset quality is made.
- Host MIDI sample offsets are honored, but original 24-frame DSP quanta and
  trigger delay remain. Sub-quantum note/retrigger bursts can coalesce. Host
  automation reaches the adapter at block boundaries and smooths internally.
- No MPE/VST3 note-ID support, aftertouch, MIDI program selection, custom Plaits
  SysEx banks, phase-policy switch or separate DRUM editor layout. Percussion
  engines and presets remain fully accessible. No step sequencer is intended.
- The 120 Hz mono-bass option is a gentle side high-pass, not a steep split.
  Detuned fundamentals can beat. The central sub remains independent.
- Distortion uses two interpolated evaluations, not a high-order oversampling
  system. Extreme hard clipping/wavefolding can alias. A dedicated antialiasing
  and subjective listening pass remains worthwhile.
- No output limiter or automatic loudness matching; loud stacked patches can
  exceed full scale. Gain compensation is explicit. No delay/chorus/reverb rack.
- State restores configuration, not phase, held notes or running LFO position.
  Changing engine/mode releases previous held notes. No schema migration exists
  beyond schema 1; unknown future schemas reject without mutation.
- Fixed logical editor size, host DPI scaling, no manual zoom or waveform scope.
- Supported rates are 32-192 kHz and output is stereo float. Unsupported rates
  mute. Only Windows x64 is built/tested here.
- CPU stress passed comfortably in this environment, but no standardized timing
  study or long multi-instance DAW soak has occurred. Do those before labeling a
  release production-ready.
