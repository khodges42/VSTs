# Engineering architecture

Sibling plugins use Rust/NIH-plug. SOUNDGYAL uses C++17 to compile original Plaits
directly without handwritten Rust FFI. Its isolated CMake project is outside Cargo.
The reusable C++ core has no JUCE, VST3, GUI or host dependency. Existing plugins,
DAWJ and RiffCalc remain unchanged.

| Layer | Source | Responsibility |
| --- | --- | --- |
| Original DSP | vendor/plaits, vendor/stmlib | MIT algorithms/resources, documented desktop adaptations |
| Reference adapter | src/engine.* | Retained first-milestone mono probe |
| Parameters | src/parameters.h | Fixed-size validated settings and stable IDs |
| Synth | src/synth.* | Eight voices, two Plaits units per voice, sub, envelopes, filter, drive, matrix |
| Conversion | DesktopEngine | Precomputed 256-phase, 64-tap causal windowed-sinc SRC |
| Presets | src/factory.h, factory/ | Complete independent configurations |
| Plugin/state | plugin/Processor.* | JUCE VST3, parameter atomics, versioned XML off audio thread |
| Editor | plugin/Editor.* | Attached controls, native windows, message-thread file operations |
| Validation | tests/ | Core probes, synth stress, presets/state, actual VST3 host |

## Signal path

Each voice blends original MAIN/AUX and a detuned second Plaits instance. Width
adds a difference signal; optional one-pole side removal reduces stereo bass
around 120 Hz. It is not a brick-wall crossover. A stable TPT state-variable
filter provides LP/HP/BP or bypass. Key tracking and cutoff modulation use octaves.
Drive offers tanh, hard clip and periodic wavefolding. Two interpolated nonlinear
evaluations reduce transitions, without high-order oversampling or a claim of
alias-free distortion. The centered sub joins after filter/drive to preserve
weight. Final DC removal rejects offsets. There is no output limiter.

Original internal 48 kHz, 24-frame quanta, int16 MAIN/AUX processing, gain settings
and embedded limiter behavior remain. SRC has 32 internal frames of nominal
delay, reported rounded up to host frames. Original trigger delay/quantization
remain instrument response. Host MIDI offsets are consumed before their output
sample, but very short gates/retriggers can coalesce within internal quanta.
Automation is host-block-based in this JUCE adapter, smoothed over approximately
8 ms; discrete choices are not interpolated.

## Voices and state

Mono uses last-held priority and repeated-pitch counts. Legato preserves
envelopes across overlaps. Polyphony chooses a free voice, then quietest/oldest;
stealing fades residual output. Channels have independent bend, wheel and sustain.
The JUCE MIDI adapter does not expose VST3 note IDs/MPE; repeated polyphonic pitches
release the oldest matching key-down voice.

Engine/mode changes clear held notes and fade old output. Other edits smooth
existing voices. State saves patch configuration, not oscillator/LFO phase,
held MIDI or audio history. Host state and user files share complete versioned XML.

## Real-time boundary

Memory and SRC kernels initialize in prepareToPlay. Processing uses fixed arrays;
no allocation/deletion, XML, I/O, logging or blocking locks. Sysex/system messages
are skipped before constructing heap-backed JUCE messages. Audition notes use a
bounded SPSC queue; overflow releases audition voices to avoid stuck keys.

Raw parameter pointers are cached. Multi-parameter writes publish an odd/even
generation; audio takes one bounded snapshot attempt and retains its previous
configuration during overlap. No waiting or retry. GUI writes follow the message
thread contract; competing simultaneous host state writers are unsupported.

stmlib PRNG is thread-local, with per-instrument save/restore around rendering to
prevent thread races and cross-instance noise dependence. Vendor edits and
hashes are recorded. Allocation instrumentation covers C++ new/new[] in core
rendering, not every CRT/driver/host memory operation. Pluginval adds independent
lifecycle and automation stress, not proof for every DAW.
