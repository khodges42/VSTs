# Dependency and license record

New integration and factory recipes: GPL-3.0-or-later, Copyright 2026 SOUNDGYAL
contributors. The publisher identity is Voidmutt Enterprises. This does not
replace upstream ownership. Original MIT notices remain in vendor files.

| Component | Exact identity | Selected terms |
| --- | --- | --- |
| Plaits | pichenettes/eurorack f4cfa7697095a28d292bff758beb51deb49b5a86 | MIT, Emilie Gillet |
| stmlib | e3bd7c9cc00e4364166f9905c0509b6ffd0535ec | MIT, Emilie Gillet |
| JUCE | 7.0.12, 4f43011b96eb0636104cb3e433894cda98243626 | GPLv3 option; core/audio-basics/events/devices include ISC grants |
| VST3 SDK bundled by JUCE 7 | Exact files in pinned JUCE tree | GPLv3 option, Steinberg |
| zlib | Exact source bundled by pinned JUCE | zlib license |
| libpng | Exact source bundled by pinned JUCE | libpng license |
| IJG JPEG | 6b, bundled JUCE source | IJG grant/conditions |
| FLAC | Exact bundled JUCE source | BSD-style license in Flac Licence.txt |
| Ogg/Vorbis | Exact bundled JUCE source | BSD-style notices in Ogg Vorbis Licence.txt |
| Windows platform libraries | Windows SDK / MSVC runtime | System build dependencies, not vendored source |

JUCE's entire exact source is fetched by scripts/bootstrap.ps1; the revision is
verified. Applicable module headers, framework license, SDK licenses and bundled
codec notices are preserved under notices/JUCE. Full corresponding dependency
source is in VSTs/target/soundgyal-deps/JUCE after bootstrap. A binary redistribution
must provide corresponding source under the selected GPL terms, including that
exact framework dependency, rather than only an inner VST3 DLL.

JUCE's bundled older SDK is used under GPLv3, **not described as MIT**. Steinberg's
current SDK is MIT-licensed, but replacing the SDK underneath JUCE 7 has not been
validated. The requested compatible-framework route is used here. No proprietary
SDK agreement is selected, and no fake certification or endorsement is claimed.

The desktop vendor patch records: MSVC-compatible unused resource table, omission
of GNU inline hints, initialized FM scratch, thread-local/per-instance PRNG
handling, and a separate optional LPG bypass. TEST uses portable upstream math and
null embedded user-data lookup; six-op banks still load upstream factory data.
No synthesis algorithm is replaced with an approximation. Modified source newline
normalization is reflected in shipped SHA-256 hashes. Upstream original byte
hashes are retained separately.

No JUCE commercial splash or telemetry is enabled. No font file, artwork, sample
recording, Mutable Instruments identity artwork or OS asset is copied. Native
controls and vector drawing establish the interface. Pluginval 1.0.4 and
clang-format 23.1.5 are development tools under ignored target directories, not
linked/shipped runtime dependencies. The Rust workspace dependency graph is
unchanged; its separate 113-package inventory still passes.
