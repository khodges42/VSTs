# Preset revision 0.3: Hoover correction and ten new configurations

## What was wrong with Jungle Hoover?

The 0.2 recipe selected engine 16 (Swarm), HARMONICS .6, TIMBRE .65 and MORPH .3.
The original Swarm implementation uses those controls for pitch dispersion,
grain density and grain length. A second detuned swarm compounded the texture.
There was no routed pitch envelope. This was a poor sound-design choice for a
Hoover, rather than evidence that Plaits should be replaced or its Swarm engine
changed. The original engine is preserved, including its unusual grain behavior.

The corrected recipe uses original virtual analog engine 8, near-unison internal
tuning, a modest detuned layer, two slow waveform-modulation routes and a short
positive pitch envelope. Its sub is restrained. This is a Plaits interpretation,
not an Alpha Juno circuit/waveform/chorus emulation. No chorus was added.

[Modor's own Hoover recipe](https://www.modormusic.com/hoover.html) describes the
combination of PWM saw/pulse, sub and chorus, with pitch-envelope movement often
added. This informed the tonal/movement goals; no patches, samples or artwork
were copied. The Plaits VA controls are not identical to the original instrument.

Saved 0.2 sessions remain unchanged: their complete parameter state still selects
Swarm. Choose Jungle Hoover again from the factory browser to load the correction.
The legacy XML is retained as a regression fixture, not silently migrated.

## Ten additions

| Preset | Category | Engine / distinguishing design | Suggested MIDI notes |
| --- | --- | --- | --- |
| Dred Reverse Bass | UK GARAGE | VA; slow-opening filter envelope, solid sub, short glide | 33-48 |
| FM Donk | UK GARAGE | Two-op FM; fast index-envelope pluck, zero amp sustain | 36-55 |
| Liquid Glass Keys | DRUM & BASS | Additive; eight-voice keys, harmonic envelope and long release | 48-76 |
| Formant Roller | DRUM & BASS | Granular formant; two-rate vowel/shape movement, restrained saturation | 30-48 |
| Classic Wobble | DUBSTEP | VA; eighth-note-cycle triangle cutoff movement over a central sub | 28-43 |
| Riddim Square | DUBSTEP | Phase distortion; square amplitude modulation plus shape movement | 28-48 |
| Vowel Growl | DUBSTEP | Granular formant; opposing formant/shape cycles and ratio envelope | 28-48 |
| Metallic Yoi | DUBSTEP | Two-op FM; downward index envelope, feedback movement, mild wavefold | 30-50 |
| Deep Dub Pluck | DUBSTEP | VA VCF; low cutoff, dark filter transient and long subby pluck | 28-43 |
| Rave Fifth Stab | JUNGLE | Chord; bright short layered stab without an obligatory sub | 48-72 |

Preset names describe sound-design targets, not reproductions of named artists
or commercial patches. Try 140 BPM for dubstep, 132 for garage and 170 for jungle
and DnB. LFO sync follows the DAW. Wheel increases cutoff on every factory patch.

## Research references

- [Native Instruments' UK garage guide](https://www.native-instruments.com/blogs/blog/uk-garage-music)
  discusses the reverse-feeling Dred bass produced with an opening low-pass filter.
  Dred Reverse Bass implements that broad envelope principle with Plaits VA.
- [Native Instruments' LFO guide](https://www.native-instruments.com/blogs/blog/what-is-an-lfo)
  identifies LFO movement as part of dubstep bass design. Classic Wobble uses
  cutoff movement, whereas Riddim Square uses amplitude/shape movement.
- [Native Instruments' oscillator documentation](https://docs.native-instruments.com/online-guides/massive-xl-user-guide/en/wavetable-oscillators.html)
  describes moving oscillator shape and formant timbres. The two formant presets
  adapt these general movement goals to Plaits grainlet/formant synthesis; they
  do not import NI wavetables or claim to implement that synth.
- [Ableton's Operator reference](https://www.ableton.com/en/manual/live-instrument-reference/#operator)
  explains FM timbral complexity and envelope modulation. FM Donk and Metallic
  Yoi use the original Plaits two-operator engine with different index/envelope
  articulation. They are original recipes, not converted proprietary presets.

These sources inform familiar genre archetypes; no popularity ranking is claimed.

## Audition and validation

The offline suite creates before/after held Hoover examples at MIDI note 60.
It removes the sub in separate diagnostic renders and measures periodicity near
the played note for MIDI 48, 60 and 72. That guards against a noisy patch passing
only because its clean sub supplies the fundamental. It also checks that the
Hoover's movement routes change audio. This is an objective regression check,
not a claim that correlation alone establishes a convincing Hoover.

All 35 presets are independently imported and rendered, with complete-state and
sample-identical reproduction tests. New patches also have three-second held-note
renders so their modulation is audible beyond short staccato probes. The audition
archive contains those examples, the fixed Hoover and the old Hoover comparison.
Human listening and a reference-record comparison have not been performed here.

Measured played-period correlation (sub disabled; after the initial pitch sweep):

| MIDI note | Old Swarm recipe | Corrected VA recipe |
| --- | --- | --- |
| 48 | 0.183 | 0.998 |
| 60 | 0.284 | 1.000 |
| 72 | 0.332 | 0.998 |

The threshold test requires correlation above 0.6 and improvement of at least
0.15 over the legacy recipe at every tested pitch. Movement is checked separately.
These figures describe periodicity, not perceptual similarity to an Alpha Juno.
