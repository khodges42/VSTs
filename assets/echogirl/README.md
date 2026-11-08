# Optional ECHOGIRL artwork

The current native skin is entirely code-drawn: amber LCD accents, violet
indicators, dark plastic panels and concentric knob rims. No character image or
borrowed skin is included. The controls do not depend on artwork.

This directory is the separate asset slot for a future original SFW adult
cyberpunk character or alternate skin. Keep any transparent PNG and its license,
creator and source/provenance record together here. Future artwork should occupy
an optional decorative header region; do not bake labels, controls or the pad
into it. The editor palette and drawing live in `plugins/echogirl/src/editor.rs`.
Adding an asset requires explicit code integration; no runtime file load or
image decoding is permitted on the audio thread.
