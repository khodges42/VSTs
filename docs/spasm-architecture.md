# SPASM architecture and pre-implementation risks

## Inspection and reuse

Reviewed VSTs/AGENTS, README, architecture/patch conventions, VIXEN/ECHOGIRL
wrappers, EIS's persistent GUI view and atomic visualization publication, pinned
dependencies, independent host/native editor tools and validation records. Also
reviewed DAWJ's VST3 split-spike THREADING and PRODUCTION-PROPOSAL. Its host/state/
lifecycle gates remain experimental; no DAWJ production files or formats change.

Reuse Rust 1.93.0, pinned NIH-plug/egui/baseview, serde, the license inventory and
independent vst3-host pattern. No new external dependency is planned. Original
source is GPL-3.0-or-later; use the GPLv3 option for the combined VST3 with the
locked GPLv3 bindings. Original art/procedural demos only, no sampled Amen break.

## Standalone modules

- `crates/spasm-core`: portable configuration/mappings, capture banks, musical
  clock, trigger state machine, captured-audio readers and compact coloration.
- `plugins/spasm`: stereo VST3, automation/MIDI input, fixed metadata, native
  performance/EDIT views and atomic waveform/status publication.
- `tools/spasm-lab`: deterministic signal/render tests, actual VST3 reference
  comparisons, native inspection and CPU/memory trials. No physical audio device.

## Capture strategy

Prepare three stereo float32 banks of eight seconds each at initialization/rate
change. Approximately 9.22 MB at 48 kHz or 36.87 MB at 192 kHz; no resize/copy of
a complete capture on the audio thread. User history limits 0.25–8 seconds select
a logical window inside prepared capacity. Bank validity counters protect all
reads; reset invalidates counters rather than clearing large allocations.

One bank records continuously. A trigger pins its valid recent history and
switches recording to a free bank in constant time. A second immutable reader
can finish a short transition. Retriggering during a transition retires the older
reader before bank reuse; bounded residual de-clicking handles that replacement.
Interpolated reads wrap within the captured window, never into future audio.

Default capture policy locks the snapshot while an audio effect remains active,
so Repeat -> Reverse -> Dive manipulate the same captured phrase. The alternative
RECENT ON PRESS pins fresh input on each trigger. After a bank rotation, fresh
history can be shorter than requested: shorten the capture and visibly report it.
With fewer than 32 frames, remain clean and armed; OFF executes once ready, while
quantized activation waits for the next usable grid boundary. No uninitialized
samples and no one-sample DC loops. Frozen audio can repeat indefinitely without
being overwritten; live input and the dry branch continue independently.

## Timing and events

The clock advances per sample in quarter-note beats. Valid host beat/sample
positions anchor it; absent timing uses a continuous free-running grid at the
last valid tempo, initially 120 BPM. Clamp tempo to 20–400 BPM. Quantization labels
OFF/1/4/1/8/1/16/1/32 are conventional note values (a quarter note is one beat).
Repeat additionally supports half notes and 1/64–1/128 notes. Explicitly document
that a half *beat* is an eighth *note*.

Pad presses quantize activation only; release cancels pending activation and
returns to live immediately through the release fade. Pending targets are stored
in beats, so tempo changes retime them. Events at a grid boundary execute there;
mid-block MIDI/automation events use their sample offsets. Latest pending press
for a pad wins. Repeated MIDI note-ons retrigger; balanced note counts prevent one
note-off from releasing another held note/channel.

Transport stop cancels performance/pending actions; start additionally clears
history. A timeline seek/loop jump cancels actions, returns live and logically
clears history. Held sources must release before rearming. Missing positions are
not treated as repeated jumps. Stopped/missing transport uses the free clock.

## Trigger state machine

Single primary captured-audio engine plus a limited gate/crush/filter/drive chain.
Default priority: STOP > DIVE > REVERSE > STUTTER > REPEAT. EDIT may change ranks;
ties from host automation resolve in a stable canonical order. Explicit primary
pads outrank CHAOS choices. Gate and Crush are independent overlays. CHAOS steps
through a seeded, deterministic set of preset-enabled modules on a musical grid;
it is not an unbounded random graph or feedback engine.

Momentary pads follow press/release. Latch policy toggles on a fresh press and
ignores release; visible processor state, not just GUI button values, indicates
engaged/pending/overridden actions. Global latch can supplement per-pad policies.
Preset loading releases triggers and private MIDI/latch state. Session restore
always starts clean with empty history; no captured recording is serialized.

RESET is immediate, unquantized and highest priority. It cancels all pending,
active and latched actions and suppresses held sources until release. Preserve
X/Y, patch settings and history by default; optional CLEAR HISTORY invalidates
banks once the short release finishes. Holding RESET keeps clean live-through.
GUI-owned momentary gestures release when the editor closes. RESET never latches.

## Effect definitions

- Repeat loops a recent window at a musical division. Stutter additionally uses
  an independently sized fragment, direction, count and bounded acceleration.
- Reverse reads captured history backwards; it never assumes future audio.
- Stop and Dive use variable-speed interpolation on captured audio. Stop actually
  reduces speed/pitch to zero, with a final amplitude taper to avoid stopped DC.
  Dive changes both pitch and playback duration. No independent time stretching.
- Gate follows a configurable 16-step mask, musical rate and attack/release.
- Crush combines quantization and sample holding, with optional wet-path filtering.
  Aliasing is intentional digital damage, not an anti-aliased mastering shaper.
- Filter/drive operate only on active wet processing. No feedback or signal gain
  accumulation. Linear wet/dry mixing and wet output trim leave idle input intact.

Window edges taper to zero for click control without shortening the musical loop
period. Engine changes crossfade readers; rapid replacements add bounded de-click
residuals. Rate/window changes must not create invalid indices. Filter coefficients
and continuous targets smooth; discrete rhythmic divisions remain clearly labeled.
Idle/default and completed RESET are exact clean-through for ordinary finite
input, with zero added latency. Explicit bypass cancels performance and fades
clean. Safety bounds the wet path; it must not clip the idle dry branch.

## Parameters, mappings, GUI and MIDI

Flat NIH parameters avoid nested-array restore allocations. Store scalar sound
settings, eight editable X/Y/velocity routes, pad assignments, priorities, note/CC
mapping and metadata. Each route has source, target, independent signed endpoints
and curve. Sum contributions relative to stored bases, then clamp effective values.
Changing the view never changes the engine. File import validates strict version,
size, ranges and mapping identities before applying parameter gestures.

Default MIDI notes 36–43 trigger seven effects and RESET; CC16/17 control X/Y.
Host/GUI parameter gates and MIDI gates combine without one source releasing the
other. MIDI axes latch until a corresponding host value changes or a GUI gesture
takes over. Assignments are editable. Verify actual note/CC delivery in the
independent VST3 host; conventional DAW effect-port routing remains a separate gate.

Large X/Y, eight trigger pads, visible quantization/effect/direction/repeat length,
compact rolling waveform and separate EDIT view. Black/purple, acid green/magenta,
chrome/bevels, warning stripes and CRT numerals. The character icon was removed
at the user's request; the header is decorative hardware/branding only. Waveform peaks use
bounded atomic publication; GUI never reads the processor's raw capture Vecs.

## Six checkpoints and risks

1. Rolling history, synchronized Repeat, unity idle/bypass and minimal stereo VST3;
   prove loading and audio through the independent host.
2. Stutter/Reverse/Gate, priority, quantization, release/reset; test offsets, missing
   timing, loops/jumps, incomplete history and rapid source changes.
3. Stop/Dive/Crush/coloration; measure actual speed movement, boundaries, gain and
   silence. Do not qualify resampling as high-quality independent pitch shifting.
4. Native pads/X/Y/waveform/skin; inspect compact/expanded views and avoid UI-thread
   timing in the audio scheduler.
5. Ten original recipes, portable user presets and editable mappings; verify full
   round-trips and clean host restore.
6. Allocation/stress/rate tests, independent actual-MIDI/automation/audio/state
   comparison, pluginval and CPU/memory. Record scope and manual gates.

Primary risks: available history after bank rotation; continuity during rapid
reader replacements; host position/tempo discontinuities; MIDI effect routing;
automation versus latched state; UI-close releases; CPU at extreme read rates.
Framework state parsing and arbitrary concurrent lifecycle are not blanket
real-time-safe merely because core rendering allocates nothing. Restore while
quiescent. Physical device/DPI/focus/listening qualification and any DAWJ production
integration require separate evidence/review. Keep working builds between phases.
