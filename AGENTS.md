# VSTs engineering

- Independent plugins live in `plugins/`; host-independent DSP in `crates/`.
  Read `README.md`, `docs/architecture.md` and `docs/patch-format.md` first.
- Keep DAWJ unchanged. Integration into its production audio path needs explicit
  user review; its experimental host is not a production dependency.
- No handwritten unsafe code, locks, filesystem access, JSON, allocation or heap
  destruction in synth processing. The development host has a small documented
  Windows OLE/message-pump boundary and never runs on an audio callback.
- Pin Rust, commit Cargo.lock, use `--locked`. Run fmt, workspace check/Clippy/tests,
  dependency inventory, offline renders and independent VST3 validation.
- Review licensing before changing dependencies. Preserve license texts and asset
  provenance. Never add proprietary samples or copied anime artwork.
- Preserve user changes. Put scratch work in ignored `.dev-notes/codex/`; generated
  audio, captures, timing and host logs in ignored `target/`.
- Do not publish, install system-wide, push or integrate without user authorization.
