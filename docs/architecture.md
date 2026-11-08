# VIXEN architecture and DAWJ integration boundary

## Repository inspection and decision

Reviewed the sibling DAWJ checkout's contributor rules, README, development and
licensing guides, engine contract, and the VST3 discovery, worker, continuity and
split experiments. The split experiment's `THREADING.md` and
`PRODUCTION-PROPOSAL.md` are particularly relevant. They identify incomplete
native lifecycle, editor/state concurrency, note-identity and latency gates.
The sample renderer is sample-specific; there is no established production
instrument interface to plug into.

VIXEN therefore lives entirely in this separate repository. No dependency points
into DAWJ; no files, scheduling, session format, latency defaults or playback path
there change. VIXEN does not adopt the experimental worker as a plugin runtime.

## Small independent workspace

| Layer | Responsibility | Audio-facing ownership |
| --- | --- | --- |
| `crates/vixen-core/patch.rs` | Versioned editable schema, validation, portable JSON | `Synth` and performance controls are fixed-size `Copy` values |
| `crates/vixen-core/factory.rs` | Ten original recipes and descriptions | Built off-thread; generated JSON is portable |
| `crates/vixen-core/dsp/` | Voices, envelopes, oscillators, filters, mapping, effects | One engine exclusively owned by processor |
| `plugins/vixen/` | VST3 buses, MIDI, automation, host state, native editor | NIH-plug wrapper; parameter atomics and bounded queues |
| `tools/vixen-render/` | Deterministic core WAV renders and level metrics | Offline only; no audio device |
| `tools/vixen-host/` | Actual VST3 render/state and silent editor inspection | Development harness; serialized lifecycle, never production callback |

NIH-plug is pinned to commit `de421011f41a6d10fc8c7a6084e4f4dee0143683`.
It provides the VST3 COM boundary, MIDI-to-parameter adaptation, stable parameter
IDs, native window embedding and state envelope. The native GUI uses egui 0.31.1
through pinned baseview adapters. All transitive revisions/registry checksums are
in Cargo.lock. No SDK checkout or proprietary binary is needed to build.

The native interface uses grey panels, green indicators and square controls.
The performance view gives the full width to the X/Y pad, with patch descriptions
and playing notes alongside the functional controls. EDIT exposes the underlying
patch. Branding is limited to VIXEN and BY VOIDMUTT.

## Signal and control paths

Each of up to 32 voices has three oscillators, each with 1–7 unison members,
stereo spread, tuning, pulse width/PWM, and optional serial phase modulation.
PolyBLEP corrects saw/pulse discontinuities; triangle integrates a corrected
square. Voices run oscillators, filters and saturation at 2x host rate. A
33-tap half-band FIR decimates the stereo sum and adds **8 host frames**, reported
through the VST3 latency API. Chorus, tempo delay, a small four-comb room and DC
removal follow at host rate. A soft output ceiling protects the final bus.
This is practical antialiasing, not a claim of alias-free extreme PM/distortion.
Effect-line read coordinates use double precision and bounded integer wrap;
the earlier single-precision wrap could round to an out-of-bounds index while
idle. See the [FL Studio crash investigation and regression](idle-crash-fix.md).

ADSR amplitude/filter/pitch envelopes advance in seconds at the negotiated rate.
Two LFOs have sine, triangle, saw and square shapes, free Hz or tempo-based cycle
lengths. Sync follows tempo, not absolute transport phase. Pitch bend is
per-channel; velocity controls amplitude and configurable high-velocity filter
accent. Wheel routes are patch-editable global macros (maximum channel wheel).

X/Y and six performance targets are smoothed over approximately 8 ms. Mapping
targets are recomputed every 16 frames and interpolated per sample; resonant
filter coefficients are likewise interpolated. LFOs and pitch/amp envelopes
continue at sample rate. Important controls have stable, sample-accurate host
automation IDs; the wrapper splits blocks at automation offsets.

Polyphony steals the quietest active voice, oldest on ties, with a 3 ms residual
fade. Free voices win first. Mono uses last-note priority and envelope retrigger;
legato preserves envelopes while notes overlap. A fixed 128-entry held-note table
supports return to previous held notes and per-channel sustain. Note IDs are
honored when available; unidentified repeated pitches release oldest first.
There is no MPE, per-note expression matrix or unlimited held-note history.

## Real-time contract

`Engine::next`, MIDI handling, controls, reset and patch switch do not allocate,
lock, parse, access files, log, call a device API or drop heap-owned data. FX memory
is allocated and touched on construction/rate change. Delay lines carry generation
tags, so reset/switch clears history logically without traversing large buffers.
The counter test measures allocations and deallocations through the safe test
allocator API; synthesis code forbids handwritten unsafe Rust.

The UI owns strings, drafts, file browsing and serialization. It validates patches
then publishes copied `Synth` values through an eight-entry lock-free queue.
The processor takes at most eight entries at a block boundary, using the newest.
Publishers serialize with each other through a mutex; the audio consumer never
touches that mutex. Replaced queue values contain no heap allocations. Audition
events use a separate bounded 64-entry queue; host MIDI is handled at host offsets.

On patch replacement or APPLY, old held notes and effect history end deliberately.
The last output sample decays over 5 ms; new MIDI after the boundary starts the new
patch. Old held notes are not replayed. This is a short de-click transition, not
transparent sound morphing or a preserved old-preset reverb tail. An editor close
releases its audition notes without releasing unrelated host notes.

## State boundary and risks

Host state includes ordinary performance parameters, the applied patch document,
editor dimensions and latched MIDI X/Y. GUI edits notify the host via a dedicated
non-automatable patch-change parameter. Only applied drafts enter host state.
Patch files reject unknown fields/versions, invalid/nonfinite values, oversized
metadata and files above 64 KiB. Invalid host patch documents retain the last
valid synth and surface an error; the host may already have applied its other
parameters, so rejection is not a transactional rollback of the whole host state.

**Live host-driven state restoration is not qualified as real-time safe.** The
pinned NIH-plug wrapper can deserialize persistent fields and call initialize/reset
at a processing boundary. That work may allocate or use lifecycle locks. Same-rate
VIXEN reinitialization avoids reallocating FX, but cannot make the framework's
state parser real-time safe. Restore while stopped/quiescent. VIXEN's own patch
browser/APPLY uses the bounded copy path and never calls this restoration path.
Saving a state reads the control-side patch mutex; it does not query or gate DSP.

Other release risks: native DPI/focus and resize behavior across DAWs; high-note
PM aliasing; final listening/level calibration; 32-voice worst-case CPU; host MIDI
CC routing differences; platform packaging. Linux/macOS are not qualified.
See validation.md for measured checks and remaining manual work.

## Future DAWJ proposal, requiring explicit review

Keep DAWJ's single scheduler/TempoMap authoritative. Qualify VIXEN first in an
independent conventional host. If the approved supervised-worker direction is
adopted, feed generation/frame-tagged prepared stereo blocks into existing part
DSP, deck mix, crossfader, cue and master. Preserve sample playback. MIDI/XY
admission must respect the worker's committed render frontier and explicit added
latency; do not backdate notes or automation. Account for VIXEN's 8-frame latency
alongside worker/device latency. Retire notes, DSP, UI and native objects in the
reviewed order. No callback mutex or generic plugin graph is proposed here.
