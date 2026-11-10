# Operator reference

## Original engine controls

All 24 slots remain: VA VCF, phase distortion, three six-op FM banks, wave terrain,
string machine, chiptune, VA, waveshaping, two-op FM, granular formant, additive,
wavetable, chord, speech, swarm, filtered noise, particles, string, modal, kick,
snare and hat. Six-op uses upstream built-in banks; custom SysEx banks are absent.

Macro meanings vary by algorithm. Two-op FM uses ratio/index/feedback; six-op uses
program/brightness/envelope control. Physical models depend on excitation and
resonance. MAIN/AUX blends original outputs, often different models or signals.
Decay and LPG colour are separate from desktop envelopes. LPG On uses external
velocity/gate level for sustained bass. Off bypasses LPG but retains triggering
and model envelopes. Percussion can decay internally while its key stays held.
Amp sustain zero produces a one-shot desktop envelope. No sequencer is included.

## Bass and processing

Layer adds a second Plaits unit up to equal weighting. Detune is relative cents;
width zero is mono. Plaits phase is free-running. Only the sub phase restarts on
non-legato notes. Detuned fundamentals can beat; the independent centered sub
supports low weight. Sub octave is -2, -1 or unison relative to played/bent pitch;
level zero bypasses it. Sine is cleanest, triangle adds harmonics. Sub joins after
filter and drive deliberately.

Filter supplies bypass/LP/HP/BP, Hz cutoff, bounded resonance, tracking relative
to MIDI 60 and signed envelope octaves. Soft saturation, hard clip and wavefold
have different transfer functions. Compensation reduces gain growth without
automatic loudness matching. Low-mono gently reduces side signal around 120 Hz;
width zero is the clearest mono check. Output is dB, with red peak indication above
full scale. No automatic limiter, chorus, delay or reverb is supplied.

## Movement and MIDI

Both ADSRs use seconds and 0-1 sustain. Mono retriggers; legato preserves envelopes
across overlaps. Glide is an exponential time constant, not a linear duration.
Eight-voice polyphony glides when a voice is reused.

LFO rate is Hz with sync off. With sync on it is **quarter-note beats per cycle**:
0.25 = sixteenth, 0.5 = eighth, 1 = quarter, 4 = 4/4 bar. Intermediate values work.
Host tempo is used, or 140 BPM if unavailable. LFOs are bipolar/free-running;
they do not reset to transport position.

Eight matrix rows select a source, destination and signed depth. Sources: Off,
LFO1/2, mod envelope, velocity, wheel, key and amp envelope. Velocity/envelopes are
unipolar; key is bipolar around MIDI 60. Routes add, then safe limits apply.

| Destination | Full-depth effect |
| --- | --- |
| Harmonics / timbre / morph | Add 1 to normalized macro |
| Cutoff | Four octaves |
| Drive | Add 1 |
| Amplitude | Multiply by 1 + modulation, bounded 0-2 |
| Pitch | Twelve semitones |
| Layer / width / sub | Add 1 |
| Detune | Add 60 cents |

Off disables a row. Wheel needs a route; factory patches include wheel-to-cutoff.
Velocity always controls amplitude/accent. Bend range is semitones. CC1 is wheel,
CC64 sustain, CC120/123 release channel voices, CC121 resets channel controls.
Aftertouch, MPE, MIDI program changes and custom Plaits SysEx are unsupported.

## Automation and persistence

All 65 controls have stable identities derived by JUCE from fixed strings.
Base IDs are in src/parameters.h. Route IDs are matrix1_source, matrix1_target,
matrix1_depth through row 8. Do not rename/reuse them without migration.
Automation reaches this adapter at host-block boundaries with smoothing; MIDI
offsets are honored before internal quantum processing.

Portable files and host state use SOUNDGYAL99_PATCH XML, schema 1, containing all
65 physical values by ID. Import rejects missing/duplicate/unknown IDs, nonfinite
or out-of-range values, fractional enums, unknown versions, incomplete files and
files over 64 KiB. Invalid files leave the patch unchanged. Future schemas require
explicit migration. Factory browsing replaces everything, including output gain.
Saved state has no dependency on factory indices, external files or patch names.
Oscillator/LFO phase and held notes are not saved.

## Workstation notes

Supported host rates: 32-192 kHz, stereo float output. Unsupported rates mute
instead of throwing across the plugin boundary. JUCE/host DPI scaling handles the
fixed 1100x750 logical-pixel editor; no separate manual zoom is supplied. Drag,
Shift-fine, double-click reset and numeric entry work. File operations stay on
the GUI thread. Install the entire VST3 directory bundle, not its inner binary.
