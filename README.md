# VOIDMUTT VSTs

**BY VOIDMUTT**

Independent Rust plugins, with host-independent DSP and their own VST3 wrappers.
DAWJ is an eventual integration target, not a runtime dependency.

**[RiffCalc 2000 Pro](https://github.com/khodges42/RiffCalc)** (formerly PACKRAT)
is now an independent MIDI workstation repository. It is no longer built or
packaged by this VST collection.

**VIXEN/SPASM 0.1.1:** fixes the reproduced VIXEN chorus-wrap idle crash and
removes SPASM's character icon. See the [crash evidence and replacement guide](docs/idle-crash-fix.md).

- **[SPASM](docs/spasm.md)**: rolling-buffer rhythmic destruction, repeat/reverse/
  stop/dive pads, X/Y performance, editable mappings and ten user-extensible presets.
- **[8 INCH STEAK / EIS](docs/eis.md)**: three-band upward/downward dynamics,
  immediate TENDERIZE intensity, BUTCHER controls and original Sweaty Ferguson skin.
- **[ECHOGIRL](docs/echogirl.md)**: stereo dub delay, feedback performance,
  THROW/HOLD/KILL/DUB, ten presets and editable X/Y mappings.
- **VIXEN**: programmable rave synthesizer, described below.

Build SPASM with
`powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bundle.ps1 -Plugin SPASM`.
Load it in a **mixer effect slot**. See its [architecture](docs/spasm-architecture.md)
and [validation record](docs/spasm-validation.md).

Build EIS with
`powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bundle.ps1 -Plugin EIS`.
Its host name is **8 INCH STEAK**; load it in a mixer effect slot. See its
[architecture](docs/eis-architecture.md) and [measured validation](docs/eis-validation.md).

Build ECHOGIRL with
`powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bundle.ps1 -Plugin ECHOGIRL`.
It loads in an **effect slot** on a mixer channel. See its
[validation record](docs/echogirl-validation.md) and
[architecture](docs/echogirl-architecture.md).

## VIXEN

A programmable Rust VST3 instrument with a performance-first X/Y surface. Ten
original synthesis recipes span hoovers, supersaws, Reese, acid, reverse bass,
chord stabs, screeches, plucks, offbeat bass and hardcore leads. Load a sound,
play MIDI, and move the pad. Open **EDIT** to make it yours.

This repository is a home for independent VOIDMUTT instruments. VIXEN has no DAWJ
dependency and does not change DAWJ playback. The current build is an early
Windows x64 instrument, with automated DSP and independent host validation;
listening review and live-show qualification remain open.

![VIXEN's actual native performance interface at its compact size](docs/images/vixen-performance.png)

## Build

Requires Windows x64, Rustup, Python 3.10+, Visual Studio Desktop C++ Build Tools and Windows
SDK. Rust 1.93.0 is pinned. The first fetch needs network access (including GitHub).

From this repository's root:

```powershell
cargo fetch --locked
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bundle.ps1
```

The process-local PowerShell flag lets the checked-in script run without changing
machine execution policy. The plugin is
`target/bundled/VIXEN.vst3/Contents/x86_64-win/VIXEN.vst3`. Point your DAW's VST3
scanner at `target/bundled`, or manually copy the **whole `VIXEN.vst3` directory**
to your user VST3 location. Nothing is automatically installed.

In your DAW, create an instrument track, load **VIXEN**, arm MIDI input and play.
The plugin itself does not open an audio device. It accepts notes, velocity,
pitch bend, wheel and sustain, and outputs stereo float32 audio. The X/Y pad and
six performance controls are automatable. Defaults: **CC16 = X, CC17 = Y**.

## FL Studio automation

Move a performance knob, then use **Tools > Last tweaked > Create automation clip**.
For separate X/Y clips, use the wrapper menu's **Browse parameters** and right-click
X or Y in the Browser. The plugins publish automation parameters; their custom
interfaces do not currently open FL's native right-click automation menu.
See the [workflow, findings and native-menu implementation scope](docs/fl-studio-automation.md).

## Play and program

- Browse with the patch selector and `<` / `>`. Double-click the X/Y pad to center
  it; drag knobs and double-click to reset. Hover for control help.
- **EDIT** edits a draft. **APPLY PATCH** makes it audible, ending old
  notes with a short de-click. **DUPLICATE** creates a named draft; factory source
  recipes always stay intact. **RESTORE FACTORY** reloads the selected recipe.
- **SAVE AS NEW USER PATCH** writes an original portable JSON patch into
  `%APPDATA%/VOIDMUTT/VIXEN/Patches/<category>`. Rename before saving a new version;
  existing files are never overwritten. Category controls its bank folder.
- **REFRESH BANK** and **LOAD USER PATCH** browse saved files. **IMPORT / EXPORT**
  use the full file path you enter. Save/export include the draft and current
  performance settings; unsent draft edits are not part of a host session save.
- Host sessions store the applied synth, editable mappings, controls, MIDI X/Y
  position and window size. Output stays global while browsing patches.

The interface uses grey panels, green indicators and square controls. Patch
descriptions sit above the full-width X/Y pad.

## Render and verify

```powershell
cargo fmt --all -- --check
cargo check --workspace --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
python scripts/dependencies.py
cargo run --release -p vixen-render --locked --offline -- --out target/demos --bank factory/vixen
cargo run --release -p vixen-host --locked --offline -- smoke target/bundled/VIXEN.vst3
```

The renderer writes twenty eight-second WAVs (default and moving X/Y for every
patch) and a measured `manifest.json`. No audio device is opened. The development
host smoke renders **the compiled VST3**, checks MIDI sample offsets and restores
an original user patch twice against the independent engine sample-for-sample.
`vixen-host editor target/bundled/VIXEN.vst3 60` opens a silent editor
inspection window; it is not a standalone live audio application.

`vixen-render --profile target/validation/core-profile.json` runs an optional
offline CPU/memory trial. Use a quiet machine for useful timing; this never
certifies device deadlines. [EDIT mode capture](docs/images/vixen-edit.png).

Run [pluginval](https://github.com/Tracktion/pluginval) independently:

```powershell
pluginval.exe --strictness-level 5 --random-seed 42 --validate target/bundled/VIXEN.vst3
```

See [architecture and DAWJ boundary](docs/architecture.md),
[patch format](docs/patch-format.md), [sound design research](docs/sound-design.md),
[validation and limitations](docs/validation.md) and [licensing](docs/licensing.md).
Original code and factory recipes are GPL-3.0-or-later; upstream code/fonts retain
their own terms. This is an unsigned development build.
