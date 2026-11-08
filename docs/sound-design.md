# Factory bank: synthesis and performance

These are original synthesized recipes. No sample recordings or copied commercial
factory patches are used. Archetype names express intended musical roles, not
verified instrument emulations. Automated distinctness/level tests do not replace
a musician's listening review; the bank is ready for that review, not declared
sonically finished by a test suite.

## Research informing the recipes

Roland describes the JP-8000's supersaw as seven detuned saws in its
[original product description](https://www.roland.com/uk/products/jp-8000/).
VIXEN uses a seven-member stack with its own detune law, phase initialization,
filter and panning. It does not reproduce the JP oscillator algorithm.

Roland's [TB-303 manual](https://cdn.roland.com/assets/media/pdf/TB-303_OM.pdf)
documents cutoff/resonance/envelope/decay, accent and slide. Its
[sequencer guide](https://articles.roland.com/mastering-the-tb-303-sequencer-in-roland-cloud/)
explains filter/volume accents. VIXEN interprets those performance ideas with
overlapping MIDI notes, glide, velocity-driven amplitude and high-velocity filter
accent. Its SVF is not a transistor-ladder or 303 circuit model.

Native Instruments' [Reese synthesis tutorial](https://www.native-instruments.com/blogs/blog/reese-bass)
demonstrates oppositely detuned oscillators and processing. VIXEN uses beating saws,
slow filter motion, a quieter pulse pair, and a centered sine sub. The sub stays
mono because its unison count is one, even while X/Y spreads the other stacks.

For hoover, reverse bass, stabs, screech, pluck and hardcore lead, the table below
records VIXEN's own engineering interpretation of the requested behaviors. It
does not assert a definitive historical recipe. Hoover uses conventional pulse
PWM plus saws; the Alpha Juno's special PWM-saw waveform is not implemented.
Reverse bass is an envelope-based swell rather than reversed recorded audio.
The chord stab is synthesized as oscillator intervals rather than lifted from a
copyrighted record. Aggressive leads use bounded serial PM and saturation.

## Ten recipes

| Patch | Underlying sound and useful starting performance | X | Y |
| --- | --- | --- | --- |
| VIXEN HOOVER | Saw/PWM stacks, sub pulse, falling pitch, chorus; hold octaves | BRIGHTNESS: cutoff, resonance, pitch sweep | VIOLENCE: chorus, detune, drive |
| HYPERSAW | Seven saws, octave shimmer, sine foundation; open minor chords | WIDTH: detune, spread, top end | INTENSITY: cutoff, drive, filter motion |
| REESE MACHINE | Beating saws/pulses, low sine, slow filter; low legato runs | MOVEMENT: cutoff, LFO depth, resonance | DIRT: width, beating, drive |
| ACID 303 | Resonant saw/pulse, envelope snap, velocity accent; overlap for slides | SQUELCH: cutoff and resonance | ACID: envelope, drive, shorter decay |
| REVERSE BASS | Rising amplitude, inverted filter envelope, recovering low pitch; offbeats | RISE: decay and pitch recovery | BITE: cutoff and drive |
| RAVE STAB | Root/minor-third/fifth stack with short envelopes; one-key chord stabs | PUNCH: cutoff and saturation | SPACE: decay, echo and room |
| SCREECH | Sine-modulated saw/pulse, resonant band-pass, saturation; short high phrases | METAL: PM index and pulse-width bias | TEAR: formant sweep, motion, drive |
| TRANCE PLUCK | Fast saw/triangle, tiny noise attack, dotted-eighth delay; sixteenths | GLASS: cutoff and envelope depth | AFTERGLOW: delay, room and decay |
| OFFBEAT BASS | Asymmetric pulse/saw with low sine, tight dry envelope; eighth-note offbeats | KNOCK: filter envelope and cutoff | WEIGHT: body length and drive |
| GABBER LEAD | Octave pulse/saw, noise, pitch snap, PWM, saturation; octaves or held top line | PANIC: pitch snap and brightness | DAMAGE: phase modulation, PWM, drive |

All factory patches also map wheel to brightness and filter motion. Each ships
its own description, playing notes, axis labels and editable route definitions.

## Listening review and reproducibility

Run `vixen-render` to write `target/demos/01-vixen-hoover-default.wav` through
`10-gabber-lead-xy.wav`. Each file is 8 seconds, stereo 16-bit PCM at the selected
rate, with a fixed 145 BPM note pattern and two seconds for release. X/Y versions
follow deterministic sine trajectories; default versions keep the center.
Hypersaw uses chords; bass archetypes use lower notes. Output gain is not secretly
normalized by the renderer: the manifest reports actual peak/RMS/sample delta.

Review quiet/strong velocities, low/high registers, stereo and mono fold-down,
the entire pad, retriggers and contextual mixes with a kick/break. Prioritize
hoover swirl, stable Reese sub, acid slide/accent, reverse-bass rise and distinct
hardcore lead attacks. Adjust factory recipe gain/behavior based on that review,
then regenerate the portable bank and demonstrations. Avoid claiming exact
emulation or final loudness matching without evidence.
