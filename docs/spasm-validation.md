# SPASM validation record

Development checkpoint: 2026-11-06, Windows x64, Rust 1.93.0, pinned locked
dependencies. No physical audio device, FL Studio session or DAWJ playback path
was used. Original source is GPL-3.0-or-later; bundled upstream notices are retained.

## Checkpoints

| Phase | Implemented and exercised | Result |
| --- | --- | --- |
| 1 / vertical slice | Prepared capture banks, synced Repeat, clean idle, stereo VST3 | Independent compiled-plugin audio/state comparison passed before GUI expansion |
| 2 / performance engine | Stutter, Reverse, Gate, quantization, priority, momentary/latch/reset | Core timing/boundary/event tests and actual VST3 automation/MIDI passed |
| 3 / destruction | Actual speed-dependent Stop/Dive, sample hold/bit reduction, low-pass/drive | Speed endpoints, gate/crush transfer, silence, safety and audio renders passed |
| 4 / performance GUI | X/Y, pads, live waveform/readhead, state indicators, resizable skin | Actual 920×760, 1040×820 and 1040×1000 EDIT views inspected; pluginval GUI tests passed |
| 5 / authoring | Ten factory recipes, user files, live EDIT, eight routes, MIDI assignments, persistent state | Portable/file/fixed-field round-trips and restored compiled-plugin reference runs passed |
| 6 / hardening | Allocation checks, partition independence, host/rate/automation tests, CPU/memory | Automated checkpoint passed; device, listening and conventional-DAW manual gates remain open |

Each implementation checkpoint retained a working release build. This records
engineering evidence, not production or live-show readiness.

## Automated checks

`cargo fmt --all -- --check`, workspace check, Clippy with `-D warnings`, locked
offline workspace tests, and `python scripts/dependencies.py` passed. **87 tests**
across the workspace: **21 SPASM core + 4 SPASM wrapper/editor**, plus the 62
existing product tests. The inventory still matches **113 external packages**;
SPASM adds no external dependencies.

SPASM tests include:

- Circular wrap, stale/future/invalid read rejection and linear interpolation.
  A stereo impulse recorded after a complete bank wrap repeats at exactly
  frames **499, 1499, 2499**, with a 1000-frame window at 8 kHz/120 BPM/1/16.
  A new incoming impulse does not leak into the fully wet locked phrase.
- Zero-latency exact idle/unity bypass and clean RESET. Captured input can remain
  frozen beyond eight seconds while new input continues recording.
- Reverse sample ordering, sample-rate-aware subdivisions (8, 44.1, 48, 96 kHz),
  mid-block triggers, cancellation before quantization, and tempo-retimed actions.
- Explicit Stop speed reaching zero, Dive reaching quarter speed at −24 semitones,
  and curved macro pitch settling at half/double speed with parameter smoothing.
- Gate pattern response, four-bit quantization transfer, finite repeat completion,
  numerical safety under rapid presets/macros/triggers and silence in every module.
- Balanced overlapping MIDI note sources, independent parameter gates, latch,
  priority, transport jumps/start/stop, missing/invalid tempo and history fallback.
- Identical offline output with block partitions **1, 17, 64, 127, 256, 1024**,
  including a tempo change and CHAOS. This caught and fixed a one-sample floating
  point grid-boundary error; Gate and CHAOS use the same stable step calculation.
- Allocation-counter checks during prepared processing, repeated reset, capture
  changes, fixed metadata restoration, state filtering and atomic publication.
  No allocations or heap deallocations were observed in those scopes. Framework
  JSON parsing/lifecycle and GUI/file work are outside that assertion.
- Strict portable-state validation, ten factory round-trips, create-new file
  protection, oversized-file rejection, unique parameter IDs and view changes
  preserving the audio configuration.

RESET on a silent capture versus constant 0.5 live input is tested with two
concurrent raised-cosine transitions at 8 kHz/default 5 ms; the adjacent-step
bound is below 0.04 (observed approximately 0.0255). This does not establish a
universal psychoacoustic click threshold for arbitrary incoming audio.

## Independent VST3 host

The separate `vst3-host` development host loads **the actual bundled binary**,
verifies stereo buses and zero reported processing latency, saves its opaque
state and restores it twice for each of eight scenarios. Total: **16 four-second
stereo renders at 48 kHz**, compared sample-for-sample with the independent engine.
Every run has **maximum error 0.0** and measurable processed output versus dry.

Scenarios exercise different factory configurations, an edited user Stutter,
sample-offset pad/X/Y host automation, actual VST3 note-on/off, actual CC16,
MIDI axis state restoration, overlapping pads, RESET, 138→173 BPM changes,
sample-position discontinuity and repeated transport stop/start with stationary
stopped positions. MIDI routes are verified in this host, not assumed from the
parameter implementation. Host fixture JSON is explicit for the pinned NIH schema;
saved opaque state is never patched.

Evidence: `target/spasm-host/report.json`, `.state` files and 16 WAVs.
The host uses a serialized Windows OLE/message-pump development boundary; no
claim is made about arbitrary concurrent state/lifecycle entry from other hosts.

## pluginval

pluginval v1.0.4, strictness **5**, seed **42**: **SUCCESS**.
Stereo processing and automation matrix: **44.1, 48, 96 kHz × 64, 128, 256,
512, 1024 frames**; automation uses 32-frame subblocks. Includes cold/warm loading,
state, native editor, editor while processing, editor automation and bus layouts.
Reported initial latency and tail are zero; actual processing latency also tests
as zero. Active/latching captures keep processing alive through the wrapper.

The separate Steinberg SDK validator was **skipped** because no validator path
was supplied. Windows AU validation is not applicable. Evidence:
`target/spasm-validation/pluginval.txt`; no tests were manually disabled.

## Renders and performance

`spasm-lab render` generates **18 ten-second 48 kHz stereo WAVs**: an original
138-BPM procedural input, ten preset performances and seven focused module
performances. Triggers, X/Y changes and reset times are recorded in the manifest.
No proprietary Amen sample or other recording is included. Each render is
non-silent and differs measurably from its input. This is reproducible audio for
review, not a completed listening judgment.

`spasm-lab profile` prepares/fills the full eight-second history and times 750
256-frame blocks with rapid triggers, fresh capture changes and X/Y changes.
Input generation is outside the measured interval. Shared-desktop **core-only**
offline trial, not a driver deadline or whole-plugin GUI CPU qualification:

| Sample rate | Engine bytes | Mean of one core | p99 / maximum block |
| --- | --- | --- | --- |
| 48 kHz | 9,217,264 | 0.48% | 35.8 / 42.6 µs |
| 192 kHz | 36,865,264 | 2.20% | 44.7 / 48.0 µs |

Evidence: `target/spasm-validation/profile.json`. Prepared bank sizes do not grow
inside processing; changing logical history uses the same fixed capacity.

## Native GUI and remaining gates

The screenshots in `docs/images/spasm-*.png` capture the actual compiled native
editor. Performance captures show real generated audio, a real MIDI-triggered
repeat and automated X/Y, discarded offline. EDIT opens from its persisted view
field. The character icon was removed in the idle-crash follow-up; the header
now uses branding and warning stripes only.
A quantization-row overlap and right-edge value spacing were corrected after
native inspection. This does not qualify every physical pad gesture, pointer
capture/focus interaction, touch device or cross-machine DPI setting.

Still required before production qualification:

- Listening with actual jungle/breakbeat material and levels appropriate to the
  intended channel, then musical tuning where needed.
- A physical audio interface, sustained live load and rapid human pad/automation
  use in FL Studio or another conventional DAW; verify MIDI effect-port routing.
- Editor close/focus, host bypass and session restoration in those DAWs, and
  target display/DPI coverage. Arbitrary concurrent lifecycle remains unqualified.
- Other operating-system bundles and the separate Steinberg validator if releasing
  those targets. No DAWJ integration is authorized or performed.

The build and hashes are recorded in `target/spasm-validation/build.json`.
See [the guide and limitations](spasm.md) and
[pre-implementation architecture/risks](spasm-architecture.md).

The [FL Studio idle-crash follow-up](idle-crash-fix.md) documents the VIXEN cause,
SPASM reader audit and new 30-minute offline family soaks.
