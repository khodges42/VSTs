# Validation record - milestone 3a preview 0.2

Windows x64, MSVC 14.50, JUCE 7.0.12. This records executed checks, not a sound
quality certification. Generated logs and material stay under `target/dubmist-cmake`.

## Automated checks

- Original Clouds comparison: native-rate granular output matches the directly
  instantiated upstream processor sample-for-sample, accounting for the wrapper's
  one-quantum collection delay. No replacement granular algorithm is involved.
- Freeze/live capture: two identical captures remain equivalent after receiving
  different input while frozen. Captured signal persists; unfreezing and feeding
  silence replaces the old material and returns output to silence.
- All four original modes × all four quality options × 44.1/48/88.2/96 kHz pass
  finite, bounded, non-silent stereo processing tests with 135 BPM synthetic input.
- Twelve factory patches export/import independently, restore exact parameters and
  generate sample-identical renders in separate processors.
- Host partitions of one versus 257 frames produce identical output. Bypass dry
  samples match their original stereo source at reported latency across the four
  common host rates. Mono input negotiation is tested with invalid data in the
  unused input channel to catch accidental reads.
- Thirty-second bounded-memory/high-feedback stress includes twelve seconds of
  uninterrupted buffer wraparound, then freeze, mode and quality changes. Clear
  and processing perform zero observed C++ new/new[] allocations. This is not an
  exhaustive CRT/driver/host allocation tracer.
- Versioned state restores into another processor; unknown schema rejects without
  changing the active configuration. Large captured buffers are not serialized.
- The native development host loads the **actual VST3**, verifies host-clock repeat
  sample alignment, and processes stereo input,
  discovers parameters, restores state into another VST3 instance, attaches the
  native editor and captures its window. It never opens an audio device.
- Independent pluginval 1.0.4 strictness 10, seed 98, passes with GUI tests enabled:
  state, automation, editor-during-processing, parameter-thread-safety and fuzzing.
  Its optional separate Steinberg validator is not provided and is explicitly skipped.

## Audio artifacts

Twelve three-second 48 kHz stereo/24-bit WAVs demonstrate real processing of the
same deterministic synthetic drum-like source at 175 BPM. The source includes
transients, low-frequency kick-like decay, noise and different pitched left/right
components. Additional automated cases use 135 BPM. These synthetic examples do not establish acoustic-break behavior or listening
quality. Separate SK5 recordings and timing assertions now validate the rhythm path.

Actual editor captures are included in `docs/images`; they are not mockups.

## Remaining work and limits

Milestone 3a is now covered by the dedicated rhythm timing suite and actual
VST3 host-clock checks. See rhythm-milestone.md for the executed 480-render timing
matrix, tape/stepping/retrigger tests, recorded CC0 material, and exact source
adaptation boundaries. The six SK5 input/repeat/reverse examples use recorded
drum-machine material; the earlier synthetic examples remain clearly labelled.

Remaining rhythm work includes tempo-quantized original Clouds grains/freeze,
a fuller continuously refreshed Kammerl implementation if desired, acoustic
break/reference listening and broader host/transport soak. Automation is still
received at block boundaries; scheduled repeat onsets then use host samples.

Milestone 4: build and measure complementary crossover/bass preservation with
matched latency, summed response and phase; test crossover smoothing and bypass.
No bass protection is currently supplied, and no high-pass-only approximation
is advertised as such.

Before a production release: human listening/reference comparison, DAWJ and
conventional DAW testing, extended multi-instance soak, high-DPI/multi-monitor
interaction, and more exhaustive allocation/timing instrumentation. No listening
or DAW compatibility claim is made here.

Further technical limits: original fixed recording memory; no live-audio state
persistence; original mode-specific grain/WSOLA/spectral delays; automation at
host block boundaries (and 32-frame internal quantization for Clouds); short
automation pulses can coalesce. The rhythm scheduler itself runs at host rate. Mode/quality/clear reset with a fade rather than
preserving old effect tails. Bypass is a delayed dry transition, not zero latency.
Gain can exceed full scale at aggressive settings; input into Clouds is bounded
to its original int16 range. No external limiter hides clipping. Host rates outside
32–192 kHz mute safely. Only Windows x64 is built here.
