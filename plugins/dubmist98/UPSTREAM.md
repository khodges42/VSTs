Clouds: https://github.com/pichenettes/eurorack
Revision: f4cfa7697095a28d292bff758beb51deb49b5a86
stmlib revision: e3bd7c9cc00e4364166f9905c0509b6ffd0535ec
JUCE 7.0.12 revision: 4f43011b96eb0636104cb3e433894cda98243626

Clouds DSP and resources are unmodified. stmlib changes omit GNU inline hints
and isolate the PRNG with thread_local storage. All original MIT notices remain.
See docs/desktop.patch, upstream-sha256.json and vendor-sha256.json.
Other inspected source revisions: docs/research-revisions.json.

Kammerl reference: https://github.com/jkammerl/eurorack
Revision: 3dae7807f928304247005ad6ccd9b206a08129c5
Original player files are preserved in vendor/kammerl (MIT, Julius Kammerl).
Tape interpolation is adapted in src/kammerl_curves.h; host scheduling and
history ownership are new GPLv3 code. See docs/rhythm-milestone.md.
