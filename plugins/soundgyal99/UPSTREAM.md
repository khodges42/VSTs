Plaits: https://github.com/pichenettes/eurorack
Revision: f4cfa7697095a28d292bff758beb51deb49b5a86
stmlib: e3bd7c9cc00e4364166f9905c0509b6ffd0535ec

Original MIT notices remain in each source. Desktop changes are recorded in
docs/desktop.patch: MSVC inline/resource compatibility, initialized FM scratch,
thread-local PRNG and an explicit optional LPG bypass. The wrapper saves/restores
PRNG state per instrument. Changes do not replace synthesis algorithms.
Original byte hashes are upstream-sha256.json; shipped hashes are vendor-sha256.json.
Only DSP, resources, desktop user-data stub and stmlib helpers are included.
TEST selects upstream portable math and disables embedded flash access.
No firmware, hardware design, branding, or artwork is used.
