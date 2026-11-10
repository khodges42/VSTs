# Dependency and licensing record

New integration and presets are GPL-3.0-or-later, Copyright 2026 Dubmist contributors.
Original Clouds/stmlib DSP, resources and portable ShyFFT are MIT, Emilie Gillet.
Original per-file notices, including FFT historical attribution, remain intact.
Clouds source and exact upstream/shipped hashes are recorded in UPSTREAM.md.

JUCE 7.0.12, revision 4f43011b96eb0636104cb3e433894cda98243626, is used under
its GPLv3 option. Its bundled older VST3 SDK uses GPLv3, not the current SDK's MIT
grant. Permissive JUCE core/events/audio modules retain ISC notices. Bundled zlib,
libpng, IJG JPEG, FLAC and Ogg/Vorbis retain their zlib/libpng/IJG/BSD-style grants.
Applicable framework, SDK and codec notices are under notices/JUCE. Full exact
framework sources are provisioned into target/dubmist-deps/JUCE; complete-source
archives include them and a verified file manifest. Windows SDK/runtime are
system build dependencies. No commercial JUCE license or proprietary SDK grant
is selected, and no telemetry or commercial splash is enabled.

Monsoon is a hardware/control-layout reference. No circuit files, CAD, panel
artwork, hardware LED design or firmware modifications are copied. Kammerl's MIT player (Julius Kammerl 2017) is now retained in vendor/kammerl for
provenance, and its tape-speed formulas are adapted in src/kammerl_curves.h with
the full MIT notice. The new host-rate scheduler and history adapter are GPLv3.
The full firmware player is not compiled wholesale. Parasites' STM32 MIT source
and Audible Instruments' GPLv3-or-later port remain inspected references only. Reference
port artwork has separate permission-based terms and is excluded. Their exact
revisions and findings are documented in investigation.md and research-revisions.json.
Future Kammerl/Parasites adaptations need their own retained notices and audit.

No font file, artwork, recording, sample pack, Mutable Instruments product identity
or operating-system asset is copied. Demonstrations include GPL procedural signals and the Frequency 303 CC0 SK-5
recording; its source notes and CC0 dedication are retained in tests/fixtures
and the audio archive. No copyrighted commercial break was imported. Pluginval and clang-format are development
tools, not linked runtime dependencies. Binary redistribution must provide the
complete corresponding source under the selected GPL terms, including the exact
framework source; the local source archive supplies it.
