# EIS architecture and implementation gates

## Inspection and boundary

Reviewed AGENTS, README, VIXEN architecture/patch conventions, ECHOGIRL's
fixed-field state and flat parameter declarations, pinned build/license tooling,
offline tests, independent VST3 host and native editor lifecycle. Reviewed DAWJ's
split-spike THREADING and PRODUCTION-PROPOSAL gates. DAWJ is unchanged; no plugin
or DSP dependency points into it. VIXEN and ECHOGIRL retain their implementations.

Reuse Rust 1.93.0, pinned NIH-plug/egui/baseview, serde, the license collector and
vst3-host development harness. No new external DSP framework is needed. Original
source is GPL-3.0-or-later; the combined VST3 uses the GPLv3 option because the
locked upstream bindings declare GPLv3. No proprietary implementation or assets
are used. The steak icon will be original code-drawn pixels, separate from controls.

## Minimal modules

- `crates/eis-core`: versioned presets, macro/base configuration, LR4 network,
  reusable band dynamics, oversampled saturation and fixed-storage processor.
- `plugins/eis`: stereo effect, flat automation parameters and fixed-size metadata,
  independent analysis/meter publication, silver/red native Easy and BUTCHER UI.
- `tools/eis-lab`: timing/transfer/transient/noise measurements, reproducible audio,
  profiling and actual VST3 audio/state/automation comparison. No audio device.

Three active bands; the crossover and band processing storage accommodate four.
The v1 portable schema/GUI exposes three. A fourth band requires a schema/UI
extension, not a replacement dynamics algorithm or crossover design.

## DSP decisions to prove

Use cascaded second-order Butterworth sections for 24 dB/oct LR4 splits. Split
low first, then split the remainder at the high crossover. Allpass-compensate
earlier bands for later splits. The sum has unity magnitude and allpass phase;
do not claim a linear-phase or sample-identical recombination. MIX uses the
same phase-aligned unprocessed band sum. Explicit bypass uses delayed raw input.

Separate upward and downward soft-knee dB gain laws, peak detection, attack/
release smoothing, boost/reduction caps and a soft upward noise-floor gate.
Stereo linking blends independent envelopes toward their shared maximum.
Independent band bypass removes dynamics, makeup and saturation, preserving
the crossover. Solo excludes other bands with fades. Nonfinite input is sanitized.

TENDERIZE scales gain movement in dB, TIME scales base attack/release, JUICE scales
upward movement and PUNCH lengthens downward gain attack. LOW/MID/HIGH are output
trims. None rewrites base settings. Switching the UI does not touch parameters.
Intensity shortcuts set only TENDERIZE. Limits apply to effective values, including
a 0.99 gain-law strength cap so >100% intensity does not reverse the static slope.

Milestone 4 saturation will use local FIR oversampling, verified against 1x
aliasing measurements before enabling presets. Reserve a constant 24-frame
delay for a 4x, two-stage 33-tap halfband path, aligned across dry, band bypass
and explicit plugin bypass; verify the actual impulse position. This is filter
latency, not lookahead. If the antialiasing test fails, leave saturation disabled
and document it rather than qualify an unmeasured nonlinear path.

Optional level matching measures slow wet/input energy and uses bounded, slowly
changing compensation on wet only, before MIX and final trim. No double preset
compensation or claim of perfect perceived loudness. Freeze on silence and solo.
Output safety is separate and explicit. Keep normal processing deterministic
across callback sizes and avoid allocations, heap drops, locks, files and UI work.

Input spectrum snapshots use fixed atomic storage; FFT and all rendering happen
on the GUI thread. Gain/input/output meters publish bounded scalar atomics.
State contains configuration, not detector/FIR history. Restore while quiescent;
the framework's arbitrary concurrent host state lifecycle remains unqualified.
BUTCHER is a persisted atomic GUI flag rather than an audio parameter: the view
responds immediately even when a host has suspended audio. It does not enter the
engine configuration. A metadata revision gesture notifies the host of changes.

## Milestones and required evidence

1. Core LR4, explicit upward/downward laws, envelopes, smoothing. Measure flatness,
   transfer, channel consistency, noise floor, silence and allocation behavior.
2. Build stereo VST3, automatable macros/bands, state, Easy UI/meters. Independently
   load/render/restore the actual binary and run pluginval.
3. BUTCHER, editable crossovers/bands, input FFT/curve, user presets. Test mode
   invariance, portable/host state and inspect the actual native editor.
4. Verify saturation, slow gain compensation, PUNCH, eight distinct presets,
   demos and CPU/memory. Run the full independent-host and pluginval matrix again.

All four implementation milestones are complete within the measured development
scope; results and remaining release gates are recorded in `eis-validation.md`.
Conventional-DAW listening, physical device deadlines and production DAWJ
integration are separate gates. Never infer those from offline success.

## Research and reference behavior

Ableton documents separate upper/lower thresholds, upward/downward dynamics,
global ratio intensity and proportional time scaling in its
[Multiband Dynamics reference](https://www.ableton.com/en/manual/live-audio-effect-reference/).
EIS implements those broad concepts independently; no exact preset ratios,
internal detector law or sample-identical emulation is asserted.
[RaneNote 160](https://www.ranecommercial.com/legacy/note160.html) and
[Linkwitz's crossover discussion](https://www.linkwitzlab.com/crossovers.htm)
describe LR alignments and allpass phase/recombination. Equations are implemented
locally, not copied as source code. Xfer's [freeware page](https://xferrecords.com/freeware)
was inspected as a product reference; freeware distribution supplies no basis
for treating proprietary source/assets as reusable. No such code is incorporated.

Also evaluated LSP's [GOTT manual](https://lsp-plug.in/?page=manuals&section=gott_compressor_stereo)
and inspected its [project license declaration](https://github.com/lsp-plugins/lsp-plugins-gott-compressor/blob/master/project.mk)
and COPYING.LESSER. The declaration offers LGPLv3-or-later, which makes it a
credible open-source reference candidate. Adopting its C++ framework/build graph
is unnecessary for this small Rust processor; no LSP code/assets are included.
Its richer sidechain/crossover/lookahead scope is deliberately not the EIS MVP.
