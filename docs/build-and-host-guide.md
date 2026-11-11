# Build and host reference

Commands and file paths below are relative to the VSTs repository root.
For install packages, start with [the release directory](../releases/README.md).

## Use in FL Studio

### Add the plugins

1. Obtain or build the Windows x64 bundles. Keep each **complete `.vst3` folder**;
   do not extract only the file inside `Contents/x86_64-win`.
2. Copy the bundles to a VST3 installation directory, for example
   `C:\Program Files\Common Files\VST3` (Windows may request administrator access).
3. In FL Studio, open **Options > File settings > Manage plugins**. Enable
   **Verify plugins**, then select **Find installed plugins**. For an update,
   enable rescanning previously verified plugins as needed.
4. Load **VIXEN** or **SOUNDGYAL 99** from the Channel Rack's **+ / More plugins** as an instrument.
   Load **ECHOGIRL**, **8 INCH STEAK**, **SPASM** or **Dubmist 98** in a **Mixer effect slot**.
   EIS appears under its full host name, **8 INCH STEAK**.

No build script installs into system folders. Image-Line's
[installation guide](https://www.image-line.com/fl-studio-learning/fl-studio-online-manual/html/basics_externalplugins.htm)
explains scanning, classification and favorites. If an update seems unchanged,
check the loaded file path and reopen the instance after replacing the bundle.

### Automate a performance

Move a performance knob, then choose **Tools > Last tweaked > Create automation
clip**. For separate X/Y clips, choose **Browse parameters** from the plugin
wrapper menu and right-click **X** or **Y** in FL's Browser.

The plugins expose automation, but their custom interfaces do not currently open
FL Studio's native right-click parameter menu. Some EDIT controls modify draft
patch data rather than individual host parameters. See the
[FL Studio automation guide](../docs/fl-studio-automation.md) for the supported
workflow, limitations and scope of native-menu support.

## Build from source

Requirements: **Windows x64**, Rustup, **Visual Studio Desktop C++ Build Tools**
with Windows SDK, and **Python 3.10+**. The repository pins **Rust 1.93.0**.
The initial dependency fetch requires network access, including GitHub; subsequent
bundle builds use the locked dependency graph offline.

From the repository root:

```powershell
cargo fetch --locked
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bundle.ps1 -Plugin VIXEN
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bundle.ps1 -Plugin ECHOGIRL
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bundle.ps1 -Plugin EIS
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/bundle.ps1 -Plugin SPASM
```

Each command produces its bundle under `target/bundled/`, including factory files
and collected dependency notices. The PowerShell flag applies to that invocation;
it does not change the machine's execution policy. Use `-DebugBuild` for Debug.

```text
target/bundled/
  VIXEN.vst3/
  ECHOGIRL.vst3/
  EIS.vst3/
  SPASM.vst3/
```

Each bundle contains `Contents/x86_64-win/<bundle-name>.vst3` and accompanying
resources. Keep that structure intact when copying or packaging it.
These are host plugins, not standalone live-audio executables.

## Patches, presets and sessions

Factory recipes are starting points you can inspect and modify. Use each
plugin's editor and save/import/export controls to retain original user versions.
Rust plugins use portable JSON; SOUNDGYAL and Dubmist use versioned XML.
Host sessions preserve the
applied state. Apply pending draft edits before expecting them in playback or a
saved session. VIXEN preserves global output while browsing patches.

The operator guides document each product's exact save, switching, latch and
release behavior. Avoid substituting one plugin's preset format for another.
Do not assume that every editor field is independently automatable.

## Engineering & documentation

VIXEN, ECHOGIRL, EIS and SPASM use Rust cores and egui. SOUNDGYAL and Dubmist
use C++/JUCE; their build and state-format guides are linked from their READMEs. DAWJ is an eventual integration target, not a runtime dependency.
Processing is designed around bounded state and prepared buffers; GUI/filesystem
work stays outside the DSP path. Framework lifecycle limitations, including state
restoration boundaries, are documented rather than treated as production guarantees.

| Product | Architecture | Validation |
|---|---|---|
| VIXEN | [Engine and integration boundary](../docs/architecture.md) | [Tests and limitations](../docs/validation.md) |
| ECHOGIRL | [Delay architecture](../docs/echogirl-architecture.md) | [Validation record](../docs/echogirl-validation.md) |
| EIS | [Dynamics architecture](../docs/eis-architecture.md) | [Measurements and validation](../docs/eis-validation.md) |
| SPASM | [Performance architecture](../docs/spasm-architecture.md) | [Validation record](../docs/spasm-validation.md) |

Run the workspace checks after fetching dependencies:

```powershell
cargo fmt --all -- --check
cargo check --workspace --locked --offline
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
python scripts/dependencies.py
```

Offline renderers and independent native hosts are included for reproducible
checks. For example, after building VIXEN:

```powershell
cargo run --release -p vixen-render --locked --offline -- --out target/demos --bank factory/vixen
cargo run --release -p vixen-host --locked --offline -- smoke target/bundled/VIXEN.vst3
```

The renderer creates default and moving-X/Y examples for each factory patch. The
host loads the actual VST3 and checks note offsets, state and reference audio.
It does not open an audio device. A development editor inspection window is not
a standalone performance host. Run the external
[pluginval](https://github.com/Tracktion/pluginval) tool separately when validating
host behavior. Automated tests do not establish subjective sound quality or every
FL Studio interaction.

## Contributing

Useful reports include the plugin version, operating system and DAW version,
loaded binary path, preset, repeatable steps and expected behavior. Include a
portable user patch when it reproduces the problem. Keep DSP changes separate
from interface work, preserve stable parameter/state identifiers, and run the
checks appropriate to the change. Retain source and asset licensing notices.

