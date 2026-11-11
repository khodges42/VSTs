# Windows install packages

The current local builds are staged in **[windows-x64/](windows-x64/README.md)**.
Generated binaries are not committed to Git; a fresh checkout must build and stage
first. Staging does not publish anything to [GitHub Releases](https://github.com/khodges42/VSTs/releases).

| Directory | Contents |
| --- | --- |
| `windows-x64/packages/` | Versioned install ZIPs for all six plugins |
| `windows-x64/VST3/` | Complete, ready-to-copy `.vst3` folders |
| `windows-x64/source/` | Complete SOUNDGYAL and Dubmist source archives |
| `windows-x64/examples/` | Audio auditions; no plugin installation required |
| `windows-x64/SHA256.json` | Checksums for staged archives |
| `windows-x64/current.json` | Versions, package paths and binary checksums |

Close the DAW and copy entire bundles from `VST3/` to a folder it scans, commonly
`C:\Program Files\Common Files\VST3`. Rescan, then reopen the plugin. Updating?
Replace the complete bundle, not just its inner file. Keep a copy of the old build
if you need to revert. SOUNDGYAL/Dubmist remain previews; consult their validation notes.

Optional initial-install helper (may need an elevated PowerShell for Program Files):

```powershell
# Preview the copies; nothing is installed:
powershell -NoProfile -ExecutionPolicy Bypass -File releases/install.ps1 -Destination 'C:\Program Files\Common Files\VST3' -WhatIf
# Install one plugin:
powershell -NoProfile -ExecutionPolicy Bypass -File releases/install.ps1 -Plugin dubmist98 -Destination 'C:\Program Files\Common Files\VST3'
```

The helper verifies the staged binary checksum and refuses to overwrite an existing
bundle. Use Explorer to replace existing bundles after closing the DAW.

To regenerate the layout from existing builds and packages:

```powershell
python scripts/stage_releases.py
```

Build prerequisites and commands: [build/host guide](../docs/build-and-host-guide.md),
[SOUNDGYAL](../plugins/soundgyal99/README.md), [Dubmist](../plugins/dubmist98/README.md).
Rust packages include their dependency notices; their source is in this repository.
Complete C++ source archives include the pinned framework. This staging check
verifies packaging and Windows x64 format, not listening quality or every host.
