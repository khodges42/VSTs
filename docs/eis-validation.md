# EIS validation record

Development on Windows x64, Rust 1.93.0, 2026-11-05/06. No physical audio device,
ASIO deadline, conventional-DAW listening or live-release qualification is claimed.
Generated evidence is in ignored `target/eis-validation` and `target/eis-host`.

## Milestone 1 — core DSP: complete

Eighteen core tests pass. Three/four-band LR4 sums pass a +/-0.035 dB test gate at
8/44.1/48/96/192 kHz. The measurement sweep records maximum sum error 0.002170 dB
over its tested rates/frequencies. The crossover measures -6.02 dB at its split.
This is flat magnitude with allpass phase, not a null against the raw input.

Upward/downward transfer laws, knee/caps/noise floor, stereo linking, PUNCH attack,
silence, hostile input, smoothing, raw bypass delay and deterministic reset pass.
Allocation counting observes zero allocations/deallocations through rendering,
configuration changes and reset. The prepared oversampling path measures exactly
24 frames and unity DC. The plugin builds with the provisional Easy interface.

## Milestone 2 — functional VST3: complete

The compiled stereo EIS VST3 loads in independent vst3-host 0.11.0. Four scenarios,
each restored twice while stopped, match the independent core sample-for-sample
(maximum error zero over eight two-second renders): Classic, an edited user Reese
with active automatic level matching, Destroy with sample-offset TENDERIZE/MIX/
bypass automation, and Classic restored with the BUTCHER view. Correct initialized
latency is 24 samples. These tests use actual VST3 state and parameter calls.

Wrapper checks cover stable unique IDs/defaults, fixed metadata restore with
zero counted allocations, and allocation-free analysis publication plus a GUI FFT
that detects an antiphase stereo tone rather than cancelling it.

[pluginval 1.0.4](https://github.com/Tracktion/pluginval) passes strictness 5,
seed 42, for both the functional stage and final bundle, with GUI tests enabled.
Cold/warm load, state, editor creation during processing, automation/editor
automation and stereo buses pass. Matrix: 44.1/48/96 kHz, 64/128/256/512/1024-frame
buffers, 32-frame automation sub-blocks. Its optional Steinberg validator stage
is skipped because the separate binary was not provided. The pre-initialization
info probe prints zero latency; the initialized actual-audio host verifies 24.

## Milestone 3 — BUTCHER and persistence: complete

All bases and macros remain independently stored/automatable. A view-toggle
render test proves sample-identical audio while GUI state changes repeatedly.
The view is a persisted atomic bool, so opening BUTCHER never waits for host
audio callbacks. Restore of name/notes/view/editor fields counts zero allocations.
The final host suite also compares Classic in both saved views.

Per-band controls, constrained crossover targets, stereo input FFT and static
transfer curve are implemented. Atomic snapshot publication performs no counted
allocations; all FFT/rendering work stays on the GUI thread. Native visual
inspection covers Easy and host-restored BUTCHER at 900x760, plus BUTCHER at
1020x1000, with real offline generated input driving meters/spectrum. Advanced
content scrolls at compact sizes; controls no longer extend past the right edge.
These are actual client captures, not mockups or plugin artwork composites.

User preset read/write round-trips preserve an edited original recipe and reject
overwriting an existing file. Names, notes, duplication/rename, import/export,
refresh and user selection are implemented; factory recipes stay protected.
Physical mouse/keyboard/DPI/focus testing in conventional DAWs remains open.

## Milestone 4 — enhancements and hardening: complete

Four-times oversampled per-band saturation, slow wet-only level matching, measured
PUNCH, separate boost/reduction meters and eight distinct presets are implemented.
Preset output calibration leaves review-material peaks between 0.377 and 0.757,
with pre-safety and post-safety peaks equal at factory defaults for that material.
This does not predict headroom for arbitrary inputs or higher macro settings.

Level-match tests verify its -12/+6 dB bounds, <=2 dB/second slew and unchanged
dry path. Band bypass removes tone/processing; solo monitors wet bands despite
global dry MIX. Full-link dynamics preserve a 4:1 stereo amplitude relationship
with saturation off. Envelope timing follows sample rate. Render results are
identical under 1/31/64/257/1024-frame chunking. Extreme automation stays finite,
exact silence remains silent, tails end in zero and safety is transparent below
its knee. Macros above 100% cannot reverse the static transfer slope.

The final workspace passes **60 Rust tests**: 18 EIS core, five EIS wrapper/editor,
one offline loudness test, and all 36 existing VIXEN/ECHOGIRL tests. Fmt, workspace
check, Clippy with warnings denied and the exact license inventory pass. The
inventory still matches 113 locked external Windows packages. No DAWJ production
audio or other plugin DSP/UI implementation was modified.

## Enhancement measurement spike

At 48 kHz, a coherent 9574.21875 Hz sine, amplitude 0.4, drive 0.65, compares the
same local tanh shaper at 1x and 4x. Unwanted spectral energy falls from -9.968 to
-56.364 dBc (46.396 dB improvement). This clears the provisional >=20 dB improvement
and <-40 dBc gates for that case; it is not proof of alias-free saturation at all
frequencies/drives/rates. No proprietary processor is measured or copied.

PUNCH=1 versus 0 produces -1.975 versus -16.860 dB downward gain five milliseconds
after a level step with the test's 5 ms base attack. It changes attack behavior,
not makeup gain. Full transient, noise-floor, transfer and crossover CSVs are
written by `eis-lab measure` for inspection and plotting.

The detector/gain measurement matrix settles constant levels through every
factory band's envelope and compares against its mathematical static law.
Maximum gain error is **0.0002223 dB** in the recorded run. White-noise trials with
makeup/output neutral show about 0.0017 dB change at -124.8/-104.8/-94.8 dBFS RMS,
and 0.0030 dB at -88.8 dBFS; below-floor noise is not subjected to naive unbounded
upward gain. Floor fade and boost caps remain explicit parts of the law.

`eis-lab render` produces original procedural percussion/bass/chord input, eight
normal preset files and sixteen pairwise loudness-matched input/output files.
No gain normalization is hidden in the normal renders. Each pair uses attenuation
to a common target for that pair and preserves sample-peak headroom. The manifest
records RMS, sample peak, band movement and loudness changes; pairs have different
target levels. Review is not a musician's listening approval.

The offline stereo loudness estimator implements K-weighting at 48 kHz and
400 ms blocks with absolute/relative gates, following
[ITU-R BS.1770-4 Annex 1](https://www.itu.int/dms_pubrec/itu-r/rec/bs/R-REC-BS.1770-4-201510-S!!PDF-E.pdf).
Its silence/1 kHz level checks pass. It is a development estimate, not a certified
LUFS/true-peak meter; the plugin's real-time level matching uses a cheaper RMS
model instead. No proprietary processor output has been used as a reference.

## CPU and storage

Offline trial: 48 kHz, 256 frames, 4096 warm-up frames, 375 measured blocks.
Source blocks are prepared outside the timer. These are core DSP wall-clock
measurements under uncontrolled shared desktop load, excluding host/GUI/FFT.

| Case | Mean / one core | P99 block | Maximum block |
| --- | --- | --- | --- |
| Classic | 3.73% | 305.8 us | 339.4 us |
| Destroy | 4.26% | 258.1 us | 267.4 us |
| Maximum drive, unlinked, level matching | 4.88% | 393.9 us | 436.0 us |

The block budget is 5333 us. This short run does not bound driver callbacks or
future stalls. Engine storage is **7672 bytes**, fixed with sample rate/block size.
Analysis uses approximately another 16.5 kB of atomics. These figures exclude
parameter/framework/GUI allocation and OS working set. Saturation filters stay
prepared/running to support smooth enabling and predictable CPU.

## Reproduce and remaining gates

Use [the guide's build/test commands](eis.md). Evidence:
`target/eis-validation/{measurements.json,profile.json,eis-level5.txt*}`, response/
transfer/noise CSVs, generated `signals/`, eight restored-host WAVs and state blobs
under `target/eis-host`, and review audio/manifest under `target/eis-demos`.
Pluginval is a separately downloaded test tool, not part of the plugin bundle.

- Conventional DAW/FL Studio listening, physical controls, automation recording,
  user preset sharing, save/close/reopen, stop/seek/bypass behavior and render tails.
- Device/ASIO deadlines, longer stress, mixed DPI, keyboard/focus behavior and
  other Windows machines. Windows x64 stereo float32 is the only qualified target.
- Listening/tuning on full tracks, sub-heavy material, supersaws and percussion.
  Measurements validate defined behaviors, not universal mastering suitability.
- Saturation aliases are substantially reduced in the measured case, not proven
  inaudible for every high-frequency/high-drive input. Very fast gain modulation
  can also generate sidebands. Safety is soft clipping, not true-peak limiting.
- No external sidechain bus, M/S, four-band GUI, lookahead, linear-phase crossover,
  alternate detector type or physical/concurrent host-state qualification yet.
- Host state parsing/lifecycle may allocate outside processor work. Restore while
  quiescent; do not infer arbitrary concurrent state safety from these tests.
- No DAWJ production integration. Existing host-spike lifecycle/latency/state gates
  and explicit review remain required before any production playback change.
