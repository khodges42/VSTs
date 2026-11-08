# Validation and remaining qualification

The [VIXEN 0.1.1 FL Studio idle-crash follow-up](idle-crash-fix.md) supersedes
the original build for effect-buffer wrap safety. The historical measurements
below remain the initial 0.1.0 checkpoint.

Recorded on Windows x64 build 26200, Rust 1.93.0, 2026-11-05. Functional checks and
offline CPU trials ran under uncontrolled shared desktop load. No audio device,
physical MIDI input, ASIO latency test or live-show qualification is claimed.
Generated WAVs, timing JSON, native captures and validator logs live in ignored
`target/`. Public captures in `docs/images/` show the actual plugin, not a mockup.

## Automated results

All 19 Rust tests passed; fmt, workspace check, Clippy with warnings denied, and
the exact dependency inventory check passed. The 113-package Windows inventory
includes development dependencies and retains upstream license texts.

| Check | Result and scope |
| --- | --- |
| Ten patch round-trips | Full synth/metadata/control equality, version rejection, strict fields, range/size/nonfinite validation |
| Deterministic synthesis | Identical renders from original and JSON-restored patches |
| Bank sound/gesture checks | Every factory recipe produces signal, distinct sampled audio, and audible-energy differences under X/Y |
| MIDI identities | Duplicate notes, optional note IDs, sustain and channel isolation, mono/legato held-note return |
| Musical timing | Pitch, low-bass level and envelope durations at 8/44.1/48/96/192 kHz; bend octave and velocity response |
| Edge behavior | 32 voices, steals, extreme XY/cutoff/tempo, release-all, silent replacement and deterministic reset |
| Audio allocations | Zero counted allocations/deallocations through render, MIDI, reset and ten patch switches |
| Antialiasing primitives | PolyBLEP reduces folded saw energy; half-band has unity DC gain and 8-frame group delay |
| Session fields | Actual NIH Params field serialization restores an edited user synth, edited route and latched MIDI X/Y |
| Publication | Eight-entry queue remains bounded and delivers the latest applied patch |
| Invalid host patch | Last valid synth retained, explicit error; does not promise rollback of other host parameters |

The independent `vixen-host smoke` loads the **built VST3**, imports an original
edited Reese through VIXEN's known component-state format, saves it, and restores
it twice while stopped. Both four-second VST3 renders agree **sample-for-sample**
with the core reference (maximum error 0 in the recorded run). The note starts at
block 1, offset 37; preceding samples are silent. Automated X restores to 0.8.
The initialized plugin reports 8 samples of latency. The harness never opens an
audio device or calls any DAWJ code.

[pluginval 1.0.4](https://github.com/Tracktion/pluginval) **passed strictness level
5**, seed 42, with GUI tests enabled: cold/warm discovery, native editor,
editor creation during processing, state, automation, and stereo bus negotiation.
Its matrix covered 44.1/48/96 kHz and 64/128/256/512/1024-frame blocks. Its optional
Steinberg validator stage was skipped because that separate binary was not
supplied. The pre-initialization info probe logs zero latency; the initialized
actual-audio harness verifies the correct 8-frame reported value.

Native visual inspection exercised PERFORM and EDIT at 840x660 and PERFORM at
1040x740. Both compact captures are checked in. This is not mixed-DPI, focus,
screen-reader or every-DAW resize qualification.

## CPU and memory trial

`vixen-render --profile target/validation/core-profile.json` measures 375 blocks
of 256 frames per case at 48 kHz, after 4096 warm-up frames. Synth notes remain
held and X moves. Timing includes wall-clock preemption; the mean is expressed
as a fraction of one core's available real-time budget. Worst-stack cases enable
three seven-member oscillators, PM, resonance and drive. These are synthetic
offline engine measurements, not a driver or callback deadline test.

| Voices | Hypersaw mean / one core | Worst stack mean / one core | Worst-stack maximum block |
| --- | --- | --- | --- |
| 1 | 2.0% | 2.7% | 203 us |
| 4 | 5.5% | 8.0% | 525 us |
| 12 | 15.0% | 23.2% | 1558 us |
| 32 | 39.8% | 62.0% | 4278 us |

The block budget is 5333 us. These short shared-load results do not bound future
stalls. Default polyphony is 12; bass recipes use mono/legato. Owned core storage
and preallocated sample/tag arrays total approximately 7.1 MB at 48 kHz, excluding
allocator overhead, GUI/framework and OS working set. Memory scales with sample
rate; patch changes do not allocate additional delay memory. Repeat timing on a
quiet target machine before choosing live voice limits or device buffers.

## Reproduce

Use the README build/check commands. The bundled instrument is
`target/bundled/VIXEN.vst3`. Run:

```powershell
target/release/vixen-host.exe smoke target/bundled/VIXEN.vst3 target/host-smoke
target/release/vixen-host.exe editor target/bundled/VIXEN.vst3 60 840 660
target/release/vixen-render.exe --out target/demos --bank factory/vixen
target/release/vixen-render.exe --profile target/validation/core-profile.json
target/validation/pluginval/pluginval.exe --strictness-level 5 --random-seed 42 --output-dir target/validation --output-filename vixen-level5.txt --validate target/bundled/VIXEN.vst3
```

Download pluginval separately from its upstream release; it is a test tool and
not shipped inside the instrument. The renderer writes twenty eight-second
stereo WAVs and a level manifest. Default-patch peaks in the review render are
approximately 0.21–0.48; the renderer does not normalize them behind the scenes.
File hashes/audio phase may change with intentional engine changes; rerender for
the current source before listening comparisons.

## Remaining work before a release or DAWJ integration

- Listening review of all ten recipes in real tracks, mono/stereo, register and
  velocity extremes. Automated distinctness is not proof of excellent sound.
- A musician's conventional-DAW session: rapid notes, overlapping/repeated notes,
  long holds, sustain, wheel/bend, CC assignment, XY automation recording, patch
  browsing, host save/close/reopen and user file sharing.
- Live device/driver performance, physical MIDI-to-output latency, longer stress,
  target-machine voice limits, DPI/focus and host-native resize behavior.
- Non-Windows packaging, advisories, release source/notices review and signing.
- Explicit DAWJ integration review against its worker and scheduling contracts.

Known design limits: live **host state restoration** may allocate in NIH-plug;
use stopped/quiescent restoration. Internal browsing/APPLY uses a real-time-safe
copy path. Preset changes intentionally end old notes/tails. MIDI CC axis movement
is saved/displayed but is not emitted as host automation recording. LFO sync is
tempo-based, not beat-position phase locking. Extreme high-note phase modulation
and saturation can still alias. Reverb is a compact synthetic room. Factory sounds
are interpretations, not Alpha Juno/JP-8000/TB-303 emulations. There is no MPE,
phaser, oscillator hard sync, AU/CLAP export or standalone device application.
