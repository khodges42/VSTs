# ECHOGIRL validation and limitations

Windows x64, Rust 1.93.0, 2026-11-05. These results cover the development VST3,
not a live-show release. No physical audio device, ASIO latency, mixed-DPI or
musician listening approval is claimed. DAWJ production audio remains unchanged.

## Recorded checks

- 36 workspace Rust tests pass: 14 delay-engine tests, three ECHOGIRL wrapper
  tests and the 19 existing VIXEN tests. Fmt, workspace check and Clippy with
  warnings denied pass. The dependency/license inventory matches all 113 locked
  external Windows packages; ECHOGIRL introduces no new external dependency.
- Delay impulse timing at 8/44.1/48/96/192 kHz, independent channels, all dotted/
  triplet divisions, and the twelve-second buffer boundary pass. A discovered
  f32 read-position wrap bug was fixed with f64 position arithmetic and bounded
  indices. Free-time frame conversion also avoids fractional rounding at exact
  integer-millisecond test positions.
- Linear dry routing, wet-only input trim, bypass unity, ping-pong channel order,
  THROW capture/gating, HOLD regeneration and KILL including diffusion clearing
  pass. CLEAN time transitions preserve a constant signal without a discontinuity.
  Character switching preserves the old tail's output level; drive makeup runs
  before its crossfade, with KILL afterward so switching cannot revive wet audio.
- Rapid time/mode/X/Y/tempo/HOLD changes remain finite and bounded. Six-second
  autonomous feedback trials at 8/48/192 kHz sustain an AC seed at feedback 1.08
  while rejecting initial DC. Nonfinite audio input is sanitized. Stop/start
  context preserves history; explicit reset produces repeatable output.
- All ten presets round-trip with complete equality. Invalid version, ranges,
  NaN, excessive size and duplicate CCs are rejected. Render/reset/configuration
  changes through all ten presets count **zero allocations and deallocations**.
- Actual NIH parameter field restore counts **zero allocations** for valid
  fixed-size metadata/MIDI fields. State filtering releases transient buttons
  without allocation. Parameters are flat because the framework's nested-array
  field helper allocated suffix strings. This test is narrower than whole-host
  concurrent state loading; JSON parsing and framework lifecycle happen outside
  the audio processor and are not declared generally real-time-safe.

The independent `echogirl-lab smoke` loads the **compiled ECHOGIRL VST3** using
vst3-host 0.11.0, confirms stereo input/output, saves state and restores it twice
while stopped for each of three scenarios. All six four-second renders match
the independent core **sample-for-sample: maximum error 0** in the recorded run,
at 48 kHz with 256-frame blocks. Initialized reported latency is zero.

Scenarios include unequal CLEAN millisecond times, Tape Drift, and a performance
sequence with sample-offset X/Y, THROW, HOLD, DUB, KILL, resumed FEED, MIDI CC16,
tempo 120->178 BPM and repeated transport context changes. These validate real
VST3 automation/controller delivery, rather than just direct engine setters.
Outputs, saved opaque state streams and JSON evidence are under
`target/echogirl-host`. This tool never opens an audio device or calls DAWJ.

[pluginval 1.0.4](https://github.com/Tracktion/pluginval) **passes strictness 5**,
seed 42, with GUI enabled. Cold/warm discovery, state, automation, editor creation
during processing and stereo bus negotiation pass. Audio/automation matrix:
44.1/48/96 kHz and 64/128/256/512/1024-frame blocks, 32-frame automation sub-blocks.
Its optional Steinberg validator stage was skipped because that separate binary
was not supplied. The initial info probe reports no tail before processing;
NIH's KeepAlive maps to VST3 infinite tail after processing. Actual DAW suspension
and offline-render tail policies still require conventional-host testing.

Native visual inspection covers PERFORM and EDIT at 820x700 and the default
1020x760. Checked-in screenshots are the actual client area. Performance controls
remain visible at minimum size; the editor scrolls. This does not qualify every
host's focus, DPI or resize handling. A native Duplicate -> Apply -> Save as new
gesture trial produced a version-1 user file with the expected name and released
buttons in a temporary workspace bank; the real user preset folder was untouched.

## Offline performance trial

Four characters, feedback 1.08, Y=1, maximum wow and spring; 4096 warm-up samples,
375 blocks of 256 samples at 48 kHz. Uncontrolled shared-desktop load, wall-clock
engine timing only. The block budget is 5333 us.

| Character | Mean / one core | P99 block | Maximum block |
| --- | --- | --- | --- |
| CLEAN | 0.81% | 52.4 us | 255.3 us |
| TAPE | 0.66% | 47.6 us | 89.0 us |
| DIRTY | 0.53% | 36.2 us | 209.1 us |
| DUB | 0.53% | 35.9 us | 221.4 us |

Owned core/preallocated storage is approximately **13.9 MB at 48 kHz**,
including generation tags for constant-time clearing. It scales with sample
rate, not block size or preset count. These numbers exclude the GUI, framework,
allocator and OS working set, and do not bound future callback stalls.

## Reproduce

```powershell
cargo fmt --all -- --check
cargo check --workspace --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
python scripts/dependencies.py
cargo build --release -p echogirl-lab --locked --offline
target/release/echogirl-lab.exe render
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bundle.ps1 -Plugin ECHOGIRL
target/release/echogirl-lab.exe smoke target/bundled/ECHOGIRL.vst3
target/release/echogirl-lab.exe profile
target/release/echogirl-lab.exe editor target/bundled/ECHOGIRL.vst3 60 820 700
target/validation/pluginval/pluginval.exe --strictness-level 5 --random-seed 42 --output-dir target/echogirl-validation --output-filename pluginval-level5.txt --validate target/bundled/ECHOGIRL.vst3
```

Download pluginval separately from upstream; it is not part of the bundle.
Renders export the ten factory JSON files, ten eight-second stereo WAVs and a
measured manifest. Peak range in the recorded gesture render is 0.194–0.279;
there is no hidden normalization. Generated evidence lives in ignored `target/`.

## Remaining gates and deliberate limits

- Musician listening/tuning in real dub, jungle, garage and techno tracks. Signal
  output, distinct recipes and stress tests do not establish excellent sound.
- Conventional DAW sessions, particularly FL Studio: insert/send operation,
  physical CC routing, automation recording, stop/seek/loop policies, preset
  browsing under live input, save/close/reopen, bypass and offline-render tails.
- Longer high-feedback/device stress, quiet-machine profiling, target buffer
  sizes, focus/gesture interruption, mixed DPI and independent Windows machines.
- Stereo float32 Windows x64 only. Mono, double-precision, macOS and Linux are
  not qualified. No sidechain, multichannel, physical spring model, oversampled
  nonlinear path or bit-perfect audio freeze. Spring is a synthetic diffuser.
- User rename saves a new named file and preserves the old one. No file picker,
  filesystem overwrite, arbitrary skin loader or backward schema migration yet.
  User files validate before apply. Host scalar updates may span host-scheduled
  sub-blocks; applying a preset is not a transactional frozen-buffer snapshot.
- No production DAWJ integration. Its host lifecycle/state/latency gates and
  explicit user review must be satisfied before changing production playback.

See [the guide](echogirl.md) for exact input-gating, state and mapping semantics.
