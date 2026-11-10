# Source investigation and architecture decisions

The initial milestone 1?2 findings below are retained as an audit record. Preview
0.2 adds the documented host-rate adaptation in [rhythm-milestone.md](rhythm-milestone.md);
statements below about unimplemented rhythm describe the earlier preview.

## Inspected sources

Exact reference revisions are in `research-revisions.json`.

- [Original Clouds](https://github.com/pichenettes/eurorack), revision
  `f4cfa7697095a28d292bff758beb51deb49b5a86`: original MIT DSP, resources, granular,
  WSOLA stretch, looping-delay, spectral and quality modes. stmlib is pinned to
  `e3bd7c9cc00e4364166f9905c0509b6ffd0535ec`.
- [Monsoon](https://github.com/jakplugg/Monsoon): Jak Plugg's README describes
  separate controls/CV for the four blend functions. Slider LED circuitry displays
  CV, not a new DSP or software waveform monitor. Its hardware is not imported.
  This product uses the control-organization idea, not circuit/panel artwork.
- [Kammerl](https://github.com/jkammerl/eurorack): `kammerl_player.h` carries
  Julius Kammerl's 2017 MIT grant. It contains an eight-slice pool, clock-driven
  capture, probability and fixed/reverse/increasing/decreasing/scratch pitch
  modes, with size quantization in `kammerl_player.cc`. Its clock and hardware
  parameter mapping require a dedicated desktop scheduler. This milestone does
  not include or advertise the algorithm. Pitch-speed modes are tape-like, not
  automatically pitch-preserving time stretch.
- [Parasites](https://github.com/mqtthiqs/parasites): repository and original
  granular processor notices inspected. STM32 software declares MIT, with
  hardware separately CC-BY-SA and AVR projects separately GPL3. Extra firmware
  processing is not adopted; the source boundary stays original Clouds.
- [Audible Instruments](https://github.com/VCVRack/AudibleInstruments): inspected
  `src/Clouds.cpp` and license. Its GPLv3-or-later desktop port uses input/output
  conversion to 32 kHz, original memory sizes and explicit zeroing before Init.
  Its panel graphics have a separate permission-based grant and are not reused.
  No VCV or Rack runtime code is linked into Dubmist.

JUCE 7.0.12 is used under its GPLv3 option, as in the C++ sibling instrument.
Its bundled older VST3 SDK uses GPLv3, not the current SDK's MIT grant. The exact
framework source and license texts are preserved/provisioned independently.
No DAWJ or sibling plugin is a runtime/build dependency.

## Important adaptation risks

Clouds assumes 32 kHz, 32-frame processing, 118784 bytes of large RAM and 65408
bytes of small RAM. These sizes are preserved. Stereo 16-bit sample storage is
approximately one second; quality choices trade channel count, bit resolution
and rate for recording duration. There is no untested enlarged buffer control.

Prepare is part of DSP scheduling: it performs bounded correlator work for WSOLA
and buffering for spectral processing. Dubmist calls Prepare at each internal
quantum before Process. The portable original stmlib ShyFFT is used; no FFT or
granular algorithm is replaced. TEST disables embedded debug-pin hardware only
and selects portable stmlib math.

Both directions use precomputed 256-phase, 64-tap windowed-sinc conversion.
Original 32 kHz constants and embedded coloration remain intact. Input conversion
is causal, delayed 32 host frames. Core collection delays 32 internal frames;
output conversion adds 32 internal frames. Reported latency is the rounded-up
sum: ceil(32 + 64 * hostRate / 32000). Artistic grain/WSOLA/spectral delays are
not compensated out of the sound. The integer dry bypass aligns to the reported
latency; wet conversion can differ from that rounding by less than one host frame.

Memory and kernels allocate in prepareToPlay, never processing. Audio processing
uses bounded arrays, cached parameter atomics and one bounded configuration
snapshot attempt. Mode/quality/clear changes fade out over ten internal quanta,
reinitialize bounded storage, then fade in. This is a bounded reset, not a
lock-free background buffer swap or seamless tail-preserving mode morph.

Clouds firmware relies on zero-initialized static RAM. The wrapper explicitly
zeros its non-polymorphic processor and storage before Init, as the inspected
desktop reference does. No object with host/framework ownership is zeroed.
stmlib's PRNG is thread-local; each core saves/restores its own state per quantum.
Original feedback limiting/high-pass filtering, freeze interpolation, dry/wet
curve, grain windows and reverb remain in the original processor.

Freeze stops live buffer writes; it does not serialize captured audio. Projects
store parameters only. A fresh instance restored frozen has an empty buffer until
unfrozen and fed audio. Clear/Trigger are edge actions and save as zero. Freeze
is a latched parameter; hosts can automate momentary edges. Grain Trigger and
Clear are momentary GUI buttons. Actions occur on internal quantum boundaries,
not sample-aligned host beat boundaries. No transport-dependent behavior exists.

## Remaining milestones

Milestone 3 needs an explicit host sample-time/PPQ scheduler, clocked Kammerl slice
integration, seek/stop/tempo policies, tape-speed labeling and real licensed drum
break timing tests. Triggering a Clouds grain is not tempo-accurate slicing.

Milestone 4 needs a complementary phase-conscious crossover, delayed low-band
path, bypass transitions and measurements of summed magnitude/phase and sub
preservation. A high-pass before Clouds alone will not meet that requirement.
No bass-protection or beat-repeat control is exposed in this preview.
