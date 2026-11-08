# FL Studio idle crash / VIXEN 0.1.1

Investigated 2026-11-06 after a live FL Studio session crashed following a period
of idle audio processing. VIXEN and SPASM now identify as **0.1.1**; parameter IDs,
VST3 class IDs, portable-patch formats and persisted field IDs remain unchanged.
SPASM's character icon is removed, including its unused drawing module.

## Evidence and cause

Windows Application Error / WER records identify `VIXEN.vst3` as the faulting
module in three recent FL64 crashes. The latest FL Studio 21.1.1.3750 dump
contains VIXEN, SPASM and EIS; the exception is `0xc0000409`, fast-fail parameter
7. That exception alone is generic and was not treated as a diagnosis.

The installed VIXEN binary matched the retained local release DLL byte-for-byte:
SHA-256 `D83F40064472B263AA2AAC06E9DF4E2B0C61FC93A5F6B2AF23FC4AF6ACD323C4`.
Using its exact binary/PDB and original panic frames, the dump shows
`core::panicking::panic_bounds_check` from the effects-line reader. The static
panic location points to `crates/vixen-core/src/dsp/effects.rs:24` in that build,
and the captured bounds arguments are **index 2213, length 2213**. This is the
44.1 kHz chorus line (0.05 seconds + 8 frames).

The old read position was computed entirely in `f32`:

```rust
let p = (cursor as f32 - delay).rem_euclid(len as f32);
let i = p as usize;
```

For a small negative difference, adding the ring length during remainder
calculation can round to exactly `len`. That value is not a valid slice index.
For example, cursor 353 and the next representable `f32` above 353 produce this
case with a 2213-sample line. The regression initially failed with the **exact
same index/length and source location** as FL Studio.

The chorus/effects reader continues advancing with no notes held, so live playing
can sound fine before an idle read eventually reaches this floating-point edge.
Rust's checked slice access panicked; the panic reached a non-unwinding plugin
boundary, aborting the host. This evidence supports a DSP indexing failure,
rather than attributing it to GUI idling, project autosave or an FL-specific bug.

## Repair and family audit

VIXEN now computes its fractional ring coordinate in `f64`, bounds the integer
index with modulo, and converts only the interpolation fraction back to `f32`.
The fix applies to chorus, delay and reverb lines because they share this reader.
It adds no allocations, locks or changes to latency. Tests check both sides of
the seam across chorus/comb/long-delay capacities at 8, 44.1, 48, 96 and 192 kHz.

ECHOGIRL already uses double-precision read coordinates and an integer modulo
guard. SPASM uses validated capture spans, clamped double-precision positions and
integer ring wrapping. EIS has no equivalent variable fractional delay reader.
Those DSPs do not receive speculative changes for this VIXEN defect.

The original binary and PDB were preserved under ignored `target/idle-crash/`.
The original **compiled VST3**, run in the independent host with a short note
followed by silence at 44.1 kHz, reproduced the same abort between four and five
simulated audio minutes. No FL Studio process or project was touched to reproduce it.

The repaired VST3 passed **30 simulated audio minutes** at both 44.1 and 48 kHz.
ECHOGIRL and SPASM each passed 30 simulated minutes at 44.1 kHz after short
input/trigger activity followed by silence. All four runs remained finite and
returned exactly zero after the initial tail. Audio blocks: 256 frames; actual
input and trigger events used within-block offsets. These are faster-than-real-time
offline processing trials, not 30 minutes of physical-device/GUI qualification.

Reproduction / soak command (the existing independent development host accepts
these three family plugins for `idle`; normal VIXEN smoke/editor modes stay scoped):

```powershell
cargo build -p vixen-host --release --locked --offline
target/release/vixen-host.exe idle target/bundled/VIXEN.vst3 target/idle-crash/soaks/VIXEN 1800 44100
target/release/vixen-host.exe idle target/bundled/VIXEN.vst3 target/idle-crash/soaks/VIXEN 1800 48000
target/release/vixen-host.exe idle target/bundled/ECHOGIRL.vst3 target/idle-crash/soaks/ECHOGIRL 1800 44100
target/release/vixen-host.exe idle target/bundled/SPASM.vst3 target/idle-crash/soaks/SPASM 1800 44100
```

Workspace tests: **87 passed**, including the two new VIXEN regressions.
Formatting, workspace check, Clippy with warnings denied, unchanged 113-package
license inventory, compiled VIXEN/SPASM reference renders and pluginval level 5
also pass. Detailed logs, soak JSON, native captures and bundle hashes are kept
in `target/idle-crash/`. Original FL crash dumps remain local and are not included
in source, bundles or screenshots.

## Replacing the installed build

Close FL Studio before replacing loaded plugin files. Replace the **whole**
`VIXEN.vst3` and `SPASM.vst3` directories in your chosen VST3 location with the
rebuilt directories under `target/bundled`. The crash report's old VIXEN was in
`C:/Program Files/Common Files/VST3/`. Reopen FL and verify plugin version 0.1.1;
rescan the plugins if needed. The build does not install system-wide or alter FL
Studio projects/settings. Existing 0.1.0 state remains readable.

The offline reproduction and corrected boundary establish this specific fix.
A follow-up real FL Studio idle session with the replacement build remains a
manual confirmation; other unrelated host/driver failures are not ruled out.
