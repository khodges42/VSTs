# ECHOGIRL architecture and engineering gates

## Inspection and reuse

Reviewed VSTs/AGENTS.md, the VIXEN core/wrapper/editor/state design, locked
dependencies, build/notices tooling and independent host harness. Also reviewed
DAWJ's engine contract and the worker/continuity/split VST experiments, including
THREADING.md and PRODUCTION-PROPOSAL.md. Those experiments remain conditional
engineering evidence, not a production audio interface. DAWJ stays unchanged.

Reuse the pinned NIH-plug VST3 wrapper, egui/baseview embedding, scalar automation,
bounded/prepared-memory policies, license inventory and offline native-host test
pattern. Keep VIXEN's plugin and DSP unchanged. Its compact synth effects are not
a suitable dub-feedback engine and will not become a shared production dependency.

## Minimal implementation

- `crates/echogirl-core`: validated portable presets, fixed-size performance
  mappings, one stereo delay engine, filters, character shaping and a small
  optional spring-inspired diffusion network. No GUI, host or device dependency.
- `plugins/echogirl`: stereo-in/stereo-out VST3, host controls, MIDI assignment,
  persistence and amber/violet native performance UI with a separate editor.
- `tools/echogirl-lab`: deterministic demonstrations, profiling and actual VST3
  effect/audio/state inspection. No physical audio device or DAWJ dependency.
- Parameterize the bundle script while retaining VIXEN as its default.

One pair of fractional delay lines serves CLEAN, TAPE, DIRTY and DUB. Twelve
seconds of prepared memory covers whole notes at 20 BPM. CLEAN/DIRTY/DUB time
changes crossfade read heads; TAPE slews a moving head for intentional pitch bends.
No resizing, heap ownership transfer, file access or mutex belongs in processing.
Same-rate state reinitialization must reuse buffers; rate changes prepare fresh
memory during host lifecycle, ending previous echoes.

Signal order: wet input trim/gate -> delay write; delay read -> high-pass ->
resonant low-pass -> 8 Hz DC rejection -> character saturation -> feedback
routing/gain -> bounded delay write. Saturation normalizes its small-signal loop
gain; output-only drive makeup retains character without multiplying feedback.
A compact spring-inspired stage is parallel to the wet output, outside the
regeneration loop. Linear wet/dry mixing preserves unity at MIX=0 and removes dry
at MIX=1. Master trim follows, with a transparent-below-threshold safety ceiling.
The dry path has zero added latency. No lookahead or delayed dry compensation.

## Performance semantics

FEED continuously injects input. Pressing THROW selects gated-input mode, opens
the echo feed while pressed, then closes it on release. FEED explicitly returns
to continuous input. The dry branch never follows these gates. HOLD closes new
input and blends to unity raw-buffer regeneration, keeping coloration audible
on output. Time remains playable during HOLD; resampling/filtering means HOLD is
not an archival bit-perfect freeze. KILL fades wet/input/feedback over 5 ms, clears
history using generation tags, and stays silent while held. DUB is a momentary
preset-defined boost to feedback, drive and tone. KILL has highest priority.

Preset browsing releases HOLD/THROW/KILL/DUB and smoothly transforms existing
echoes rather than resurrecting stale saved buffers. Host session restoration
stores settings, not echo recordings, and starts with empty history. Transport
play/stop context alone preserves echoes/HOLD while callbacks continue; an explicit
host reset clears history. Rate changes also clear history.

FILTER is a base position. X curves offset it from dark low-pass through open
center toward high-pass. FEEDBACK/DRIVE/MIX are bases; editable Y routes add
curved amounts, including a late-travel drive/resonance increase. The engine
clamps the combined result, not the stored bases, so automation has defined
additive semantics. MIDI overrides latch until the corresponding host value or
GUI gesture changes. Hosts must route CCs to the effect for MIDI mapping to work.

Host state uses scalar parameters and fixed-size atomic metadata, avoiding
VIXEN's persistent string/mutex patch field. Valid field restoration and same-rate
initialization require allocation tests. The framework still owns VST3 lifecycle
synchronization; arbitrary concurrent host state replacement is a separate gate,
not something to silently qualify from a successful offline restore. Parameters
are explicitly flat: the framework's nested-array persistence helper allocated
16 suffix strings per field restore. Flat declarations retain stable IDs while
the valid restore test now measures zero allocations, without a framework fork.

For recorded THROW performances, record FEED=off before THROW=on and FEED=on
when continuous input should resume. The GUI writes this FEED transition itself.
Re-sending an unchanged host value does not clear a latched MIDI override; an
actual value transition or GUI gesture does. This is deliberate value-based
arbitration, not last-event timestamp priority.

## Focused spikes / acceptance evidence

1. Impulse timing, fixed-capacity boundaries, stereo routing and unity dry path.
2. Zero-allocation render/reset, maximum feedback and hostile automation; NaN/DC/
   denormal checks, read-head transitions, HOLD/THROW/KILL and transport cases.
3. Actual independently hosted VST3, stereo input processing, stopped state
   round-trips and pluginval with editor/automation tests.
4. Cheap spring-inspired diffusion: judge demonstration renders and document its
   synthetic character. A physical dispersive spring model is a future project.
5. Offline CPU/memory trial; listening and conventional-DAW live qualification
   remain explicit manual gates. Do not call offline wall-clock timing DAC latency.

Implementation and measured evidence are recorded in
[the user guide](echogirl.md) and [validation results](echogirl-validation.md).
No DAWJ integration or production audio modifications were made. A future
integration proposal should consume the independent VST3, preserve the host's
bus/latency/state-thread contracts and pass its existing lifecycle gates before
any production playback change. Sharing the core is possible, but would require
an explicit adapter and review rather than linking to host spike internals.

References: Steinberg's [processing/bypass/state FAQ](https://steinbergmedia.github.io/vst3_dev_portal/pages/FAQ/Processing.html)
and [process context](https://steinbergmedia.github.io/vst3_doc/vstinterfaces/structSteinberg_1_1Vst_1_1ProcessContext.html).
Spring modeling research uses dispersive/allpass networks; a small voiced
diffuser is not a verified physical emulation ([DAFx spring study](https://www.dafx.de/paper-archive/2011/Papers/39_e.pdf)).
