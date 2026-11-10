# Rhythmic slices — milestone 3a, preview 0.2

This adds a bounded, usable rhythm processor while preserving the four original
Clouds modes. It is a desktop adaptation of Kammerl repeat concepts and tape-speed
curves, **not a drop-in port of the complete Kammerl firmware**.

## Operation

Select **Rhythmic slices** in the processing-mode selector. The editor opens the
Rhythm property sheet. Feed audio, then press **REPEAT / HOLD**. With Host Sync
and Quantize Start enabled, capture begins on the next selected musical boundary.
At least one complete division of history must be available first. Freeze also
requests a held slice in this mode. Release fades back to live audio immediately;
quantization applies to capture/retrigger starts, not releases.

Host Sync needs host BPM, PPQ and playing state. Without them, synced processing
stays dry and the status indicates missing clock. Disable Host Sync for the Free
Tempo control; free-running repeats intentionally continue regardless of host
transport. The three new factory patches use free tempo so offline auditions work.
Enable Host Sync when you want them to follow a DAW.

Divisions are 1/4, 1/8, 1/16, 1/32 and 1/64 notes, plus their triplets. They are
quarter-note beat durations of 1, .5, .25, .125, .0625 and 2/3, 1/3, 1/6, 1/12,
1/24. Boundaries are computed from the host timeline and rounded to the nearest
sample; rounded periods are never repeatedly accumulated. A captured division's
length is rounded once to whole host frames, so fractional periods may differ
from adjacent boundary spacing by one frame.

An eight-second stereo history is preallocated at prepare time (up to about
12.3 MB at 192 kHz). A hold snapshots the available history logically and stops
writing it. There is no large capture copy or allocation on a clock boundary.
This differs from firmware that continually captures new slices: repeat holds
the selected snapshot until released, cleared or invalidated by transport.

History Slice selects one of up to eight retained divisions. Slice Step advances
through that pool on clock edges. Selection wraps within the divisions actually
available; it never reads unrecorded memory. Reverse reverses sample order;
Alternate flips direction on successive clock cycles. Repeat Probability chooses
whether each cycle presents the held slice or live input. Zero means dry input;
it does not repeatedly refresh the held snapshot.

Retrigger restarts the selected snapshot immediately or on the next boundary,
according to Quantize Start. Restart On Bar resets slice stepping and direction
at bar boundaries, using host bar position/time signature when supplied. Clear
invalidates history and fades out; an armed hold can capture again after enough
fresh input becomes available.

Tape Pitch and Tape Speed Ramp change **pitch and duration together**. Ramp
interpolates playback speed over a clock cycle, following Kammerl's increasing
and decreasing speed formulas. This is not pitch-preserving time stretch.
Hermite interpolation and short transitions are used; high-speed playback is
not a fully bandlimited resampling system and can alias at extreme settings.

Mix, gains and bypass remain shared. Mix is linear in the rhythmic mode, whereas
Clouds retains its original curve. Spread, feedback, reverb, quality and the
original macro controls apply to Clouds modes only. They do not secretly alter
the slice processor. Bass protection is still absent.

## Host behavior and latency

Each block supplies tempo/PPQ and optional sample/bar positions to a scheduler
which advances at host-sample resolution. Tempo changes recompute future clock
boundaries without changing the held recording's sample rate. Synchronized stop,
start, seek, clock-source and division changes discard the prior capture/history
and fade back to live audio. New input must prime the next capture. Very small
PPQ discrepancies below two host samples are tolerated as clock rounding.

The original Clouds conversion/processing latency is retained as a common plugin
latency. Rhythmic output receives the same reported integer delay; bypass is
aligned to it. Two-millisecond transitions reduce activation, release, retrigger,
direction and loop-boundary discontinuities. Tests inspect samples after those
deliberate transitions, rather than claiming a discontinuous hard switch.

Parameter automation still reaches the wrapper at host block boundaries. Once
the wrapper receives an arm action, its quantized onset is scheduled at a host
sample boundary. This does not make incoming automation sample-accurate within
the host block. Sub-block button pulses can coalesce.

## Source and licensing boundary

Original Clouds and stmlib remain unchanged from preview 0.1. The inspected
Kammerl revision is `3dae7807f928304247005ad6ccd9b206a08129c5`. Original
`kammerl_player.h/.cc` are preserved in `vendor/kammerl` for provenance. They are
not compiled wholesale against incompatible Clouds parameter structures.

`src/kammerl_curves.h` adapts the increasing/decreasing tape interpolation from
Julius Kammerl's MIT-licensed player and preserves his complete notice. The new
host-rate scheduler, stereo history ownership, transitions and transport adapter
are GPLv3 integration code. Embedded fixed-point addresses and clock-by-block
assumptions are replaced because they cannot give the requested desktop timing
or safely address long host-rate buffers. No generic granular replacement is
introduced: the original granular path is still the original Clouds processor.

## State compatibility

All seventeen old IDs **and their ranges** are unchanged. In particular, the old
four-mode parameter still has range 0–3; extending its normalized range would
change old automation. A new boolean `rhythm_enabled` selects the extra mode.
The GUI combines those two controls into its five-entry mode menu. Hosts can
automate the new boolean independently. Fourteen new IDs are appended, giving
31 parameters total.

Schema 2 saves the complete configuration. Schema 1 explicitly migrates the old
17 fields and supplies new defaults, with rhythm disabled. Unknown schemas,
missing/duplicate fields and invalid values reject without mutation. Clear and
Trigger still save as zero. Captured audio, clock phase and held playback are
not serialized: reopening a project rebuilds history from new live audio.

## Executed validation

- 480 deterministic renders cover four host rates, 140/175 BPM, ten divisions,
  forward/reverse and one/127/512-frame partitions. Onsets match rounded host
  grid samples, slice order is checked, and outputs match across partitions.
- Separate tests cover stop/seek invalidation without stale-audio return,
  probability zero, long holds, tempo changes, exact stepped/alternating sample
  order, tape pitch, tape curves, retrigger and bar restart.
- Recorded CC0 Casio SK-5 drum material from Frequency 303 is retimed into
  sixteen-beat fixtures at 140 and 175 BPM. Forward/reverse samples are checked
  through the processor with a supplied host playhead, including reported latency.
  It is a recorded drum-machine loop, not an acoustic Amen break.
- The independent native host sends tempo/PPQ through the **actual VST3** and
  verifies repeated stereo sample order at the compensated output position.
- Existing original-Clouds reference, freeze, quality/rate, state and allocation
  checks remain in place. The realtime harness additionally exercises rhythm
  without observed C++ new/new[] allocations.

Audio examples include SK5 input/repeat/reverse WAVs for both tempi. Source and
CC0 dedication are retained under `tests/fixtures` and in the audio archive.
No human listening comparison or commercial-DAW validation has occurred.
