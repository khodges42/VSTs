# License and dependency record

Original VIXEN, ECHOGIRL, EIS and SPASM code and procedural recipes are
**GPL-3.0-or-later**. The repository's original LICENSE is preserved. No commercial
instrument binaries, sampled factory presets, anime images or borrowed skins are
included. Research references describe ideas; no source/art was copied from them.

RiffCalc (formerly PACKRAT) is maintained in its independent repository, with
its own C++ source, GPLv3 application and dependency inventory. It is not part
of this repository's Rust build or dependency graph.

The exact locked Windows graph, including development tools, is in
`dependencies-windows.json`. `python scripts/dependencies.py` compares its exact
name/version/source/license and retained-text list against filtered locked Cargo
metadata and fails on unknown expressions or missing license texts.

NIH-plug/derive/egui are ISC; baseview, egui-baseview and the development host are
permissive. The pinned `vst3-sys` declares **GPLv3**, with its full upstream license
text retained by the collector. Original project source offers GPLv3-or-later; use
the GPLv3 option for a combined VST3 binary containing those bindings. No claim is
made that upstream GPLv3 code has been relicensed to a later-license grant.

Common MIT/Apache alternatives select Apache-2.0; combined AND obligations remain
combined. The egui default-font package additionally requires OFL-1.1 and
Ubuntu-font-1.0. Its original `OFL.txt`, `UFL.txt`, Hack/Bitstream Vera and emoji
font notices are collected. Fonts are unmodified. Unicode-3.0 and BSL-1.0 notices
are retained where declared. Windows system/OpenGL libraries are prerequisites,
not redistributed proprietary runtime packages.

Some registry archives omit workspace license files. Exact-commit supplements
are checked in under `notices/`, with provenance in `notices/sources.json`; Git
workspace subcrates retain their nearest inherited notice. Supplements can be
reproduced with `python scripts/fetch_notices.py` (network required). The normal
build uses checked-in supplements and cached sources, without fetching legal text.

The bundle script checks the inventory and collects upstream notices under
`Contents/Resources/third-party`. Include corresponding source, pinned build
instructions, lockfile, these notices, LICENSE and NOTICE with any distribution.
Retain compiler/runtime notices as applicable. A development DLL and successful
metadata check are not a signed release, legal certification or advisory audit.
Recheck non-Windows graphs/native dependencies before releasing other platforms.
