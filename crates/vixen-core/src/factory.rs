// SPDX-License-Identifier: GPL-3.0-or-later
//! Original subtractive/phase-modulation recipes, not hardware emulations.
use crate::patch::*;
pub const NAMES: [&str; 10] = [
    "VIXEN HOOVER",
    "HYPERSAW",
    "REESE MACHINE",
    "ACID 303",
    "REVERSE BASS",
    "RAVE STAB",
    "SCREECH",
    "TRANCE PLUCK",
    "OFFBEAT BASS",
    "GABBER LEAD",
];
fn osc(wave: Wave, level: f32, semitones: f32, unison: u8, detune: f32, width: f32) -> Oscillator {
    Oscillator {
        wave,
        level,
        semitones,
        unison,
        detune,
        width,
        ..Oscillator::default()
    }
}
fn env(attack: f32, decay: f32, sustain: f32, release: f32) -> Envelope {
    Envelope {
        attack,
        decay,
        sustain,
        release,
    }
}
fn map(source: Source, target: Target, amount: f32) -> Mapping {
    Mapping {
        enabled: true,
        source,
        target,
        amount,
        curve: 1.,
    }
}
pub fn patch(index: usize) -> Patch {
    let index = index.min(9);
    let mut s = Synth::default();
    let (category, description, performance, x, y, routes) = match index {
        0 => {
            s.oscillators = [
                osc(Wave::Saw, 0.55, 0., 3, 0.2, 0.6),
                osc(Wave::Pulse, 0.55, 0., 3, 0.13, 0.8),
                osc(Wave::Pulse, 0.25, -12., 1, 0., 0.),
            ];
            s.oscillators[1].pwm = 0.32;
            s.oscillators[2].pulse_width = 0.25;
            s.pitch_amount = 12.;
            s.pitch_env = env(0.001, 0.18, 0., 0.12);
            s.amp = env(0.012, 0.3, 0.8, 0.2);
            s.cutoff = 2600.;
            s.lfos[0].rate = 0.85;
            s.lfos[1].pitch = 0.06;
            s.lfos[1].rate = 4.8;
            s.effects.chorus = 0.8;
            s.effects.reverb = 0.12;
            (
                "RAVE",
                "PWM and saw stacks, sub pulse, falling pitch and wide chorus. A synthetic hoover interpretation.",
                "Hold octaves, then drag toward the upper right for a rising wall of PWM and saturation.",
                "BRIGHTNESS",
                "VIOLENCE",
                vec![
                    map(Source::X, Target::Cutoff, 1.3),
                    map(Source::X, Target::Resonance, 0.7),
                    map(Source::X, Target::PitchEnv, 0.6),
                    map(Source::Y, Target::Chorus, 0.65),
                    map(Source::Y, Target::Detune, 0.4),
                    map(Source::Y, Target::Drive, 0.65),
                ],
            )
        }
        1 => {
            s.oscillators = [
                osc(Wave::Saw, 0.7, 0., 7, 0.44, 1.),
                osc(Wave::Saw, 0.2, 12., 3, 0.2, 0.9),
                osc(Wave::Sine, 0.16, -12., 1, 0., 0.),
            ];
            s.amp = env(0.018, 0.4, 0.85, 0.28);
            s.cutoff = 5200.;
            s.filter_amount = 1.;
            s.effects.delay = 0.2;
            s.effects.reverb = 0.22;
            s.gain = 0.36;
            (
                "TRANCE",
                "Seven detuned saws with octave shimmer and a centered sine foundation.",
                "Play open chords. Sweep width into euphoria; lift intensity for the drop.",
                "WIDTH",
                "INTENSITY",
                vec![
                    map(Source::X, Target::Detune, 0.6),
                    map(Source::X, Target::Width, 0.7),
                    map(Source::X, Target::Cutoff, 0.4),
                    map(Source::Y, Target::Cutoff, 1.),
                    map(Source::Y, Target::Drive, 0.5),
                    map(Source::Y, Target::LfoFilter, 0.1),
                ],
            )
        }
        2 => {
            s.oscillators = [
                osc(Wave::Saw, 0.7, 0., 2, 0.55, 0.35),
                osc(Wave::Pulse, 0.2, 0.07, 2, 0.17, 0.4),
                osc(Wave::Sine, 0.45, -12., 1, 0., 0.),
            ];
            s.cutoff = 850.;
            s.filter_amount = 1.;
            s.amp = env(0.02, 0.4, 0.85, 0.15);
            s.lfos[0].filter = 0.9;
            s.lfos[0].rate = 0.23;
            s.voice_mode = VoiceMode::Legato;
            s.glide = 0.09;
            s.drive = 0.32;
            s.gain = 0.38;
            (
                "JUNGLE / DNB",
                "Slowly beating saw pair and modulated pulse with a mono sub oscillator.",
                "Use low single notes and legato runs. Movement opens the growl; dirt widens the beating mids.",
                "MOVEMENT",
                "DIRT",
                vec![
                    map(Source::X, Target::Cutoff, 0.9),
                    map(Source::X, Target::LfoFilter, 0.5),
                    map(Source::X, Target::Resonance, 0.6),
                    map(Source::Y, Target::Width, 0.7),
                    map(Source::Y, Target::Detune, 0.65),
                    map(Source::Y, Target::Drive, 0.8),
                ],
            )
        }
        3 => {
            s.oscillators = [
                osc(Wave::Saw, 0.85, 0., 1, 0., 0.),
                osc(Wave::Pulse, 0.12, 0., 1, 0., 0.),
                osc(Wave::Sine, 0., 0., 1, 0., 0.),
            ];
            s.voice_mode = VoiceMode::Legato;
            s.glide = 0.065;
            s.cutoff = 420.;
            s.resonance = 0.78;
            s.filter_amount = 4.;
            s.filter_env = env(0.001, 0.19, 0., 0.08);
            s.amp = env(0.002, 0.22, 0.45, 0.06);
            s.accent = 0.8;
            s.drive = 0.3;
            s.effects.delay = 0.12;
            s.effects.reverb = 0.;
            (
                "ACID",
                "Single saw/pulse through a resonant 24 dB filter. Velocity above 80% adds filter accent.",
                "Overlap notes for slides; alternate soft and hard velocities for accents. Not a circuit model.",
                "SQUELCH",
                "ACID",
                vec![
                    map(Source::X, Target::Cutoff, 1.6),
                    map(Source::X, Target::Resonance, 0.28),
                    map(Source::Y, Target::FilterEnv, 0.9),
                    map(Source::Y, Target::Drive, 0.8),
                    map(Source::Y, Target::Decay, -0.35),
                ],
            )
        }
        4 => {
            s.oscillators = [
                osc(Wave::Saw, 0.7, -12., 2, 0.12, 0.15),
                osc(Wave::Pulse, 0.3, -12., 1, 0., 0.),
                osc(Wave::Sine, 0.25, -12., 1, 0., 0.),
            ];
            s.voice_mode = VoiceMode::Mono;
            s.cutoff = 2400.;
            s.filter_amount = -3.8;
            s.filter_env = env(0.001, 0.19, 0., 0.05);
            s.amp = env(0.075, 0.18, 0., 0.04);
            s.pitch_amount = -7.;
            s.pitch_env = env(0.001, 0.12, 0., 0.04);
            s.drive = 0.48;
            s.effects.reverb = 0.;
            (
                "HARD DANCE",
                "Slow amplitude rise against an inverted filter envelope and upward pitch recovery.",
                "Trigger short offbeats after a kick. Push rise for a longer swelling tail, then add bite.",
                "RISE",
                "BITE",
                vec![
                    map(Source::X, Target::Decay, 0.8),
                    map(Source::X, Target::PitchEnv, 0.35),
                    map(Source::Y, Target::Cutoff, 1.),
                    map(Source::Y, Target::Drive, 0.75),
                ],
            )
        }
        5 => {
            s.oscillators = [
                osc(Wave::Saw, 0.55, 0., 2, 0.13, 0.6),
                osc(Wave::Pulse, 0.4, 3., 2, 0.12, 0.6),
                osc(Wave::Saw, 0.45, 7., 2, 0.15, 0.8),
            ];
            s.amp = env(0.002, 0.17, 0., 0.09);
            s.filter_env = env(0.001, 0.12, 0., 0.08);
            s.cutoff = 1100.;
            s.filter_amount = 3.5;
            s.drive = 0.25;
            s.effects.chorus = 0.3;
            s.effects.delay = 0.17;
            s.effects.reverb = 0.2;
            (
                "OLD SCHOOL",
                "Root, minor third and fifth synthesized as a punchy chord stab. No sampled records.",
                "One key plays the chord. Short hits work over breaks; extend decay and space for transitions.",
                "PUNCH",
                "SPACE",
                vec![
                    map(Source::X, Target::Cutoff, 1.),
                    map(Source::X, Target::Drive, 0.5),
                    map(Source::Y, Target::Decay, 0.8),
                    map(Source::Y, Target::Delay, 0.55),
                    map(Source::Y, Target::Reverb, 0.6),
                ],
            )
        }
        6 => {
            s.oscillators = [
                osc(Wave::Sine, 0.35, 19., 1, 0., 0.),
                osc(Wave::Saw, 0.7, 0., 2, 0.1, 0.7),
                osc(Wave::Pulse, 0.35, 7., 1, 0., 0.),
            ];
            s.filter_mode = FilterMode::Band12;
            s.cutoff = 2200.;
            s.resonance = 0.65;
            s.phase_mod = 1.8;
            s.drive = 0.85;
            s.lfos[0].rate = 4.;
            s.lfos[0].filter = 0.8;
            s.lfos[1].rate = 0.43;
            s.lfos[1].pwm = 0.15;
            s.amp = env(0.003, 0.2, 0.7, 0.08);
            s.voice_mode = VoiceMode::Mono;
            s.glide = 0.04;
            s.effects.delay = 0.12;
            s.gain = 0.32;
            (
                "HARDSTYLE",
                "Audio-rate phase modulation into a narrow resonant band and heavy saturation.",
                "Play short high phrases. Move metal for changing sidebands; tear sweeps the resonant formant.",
                "METAL",
                "TEAR",
                vec![
                    map(Source::X, Target::PhaseMod, 2.),
                    map(Source::X, Target::Pwm, 0.6),
                    map(Source::Y, Target::Cutoff, 1.2),
                    map(Source::Y, Target::LfoFilter, 0.5),
                    map(Source::Y, Target::Drive, 0.25),
                ],
            )
        }
        7 => {
            s.oscillators = [
                osc(Wave::Saw, 0.7, 0., 5, 0.24, 0.9),
                osc(Wave::Triangle, 0.3, 12., 1, 0., 0.),
                osc(Wave::Noise, 0.025, 0., 1, 0., 0.),
            ];
            s.amp = env(0.001, 0.16, 0., 0.08);
            s.filter_env = env(0.001, 0.1, 0., 0.04);
            s.cutoff = 700.;
            s.filter_amount = 5.;
            s.effects.delay = 0.35;
            s.effects.delay_beats = 0.75;
            s.effects.feedback = 0.45;
            s.effects.reverb = 0.15;
            (
                "TRANCE",
                "Fast saw/triangle pluck with a small noise transient and dotted-eighth ping-pong echoes.",
                "Play sixteenths at the host tempo. Glass changes the transient; afterglow opens the echo field.",
                "GLASS",
                "AFTERGLOW",
                vec![
                    map(Source::X, Target::Cutoff, 0.9),
                    map(Source::X, Target::FilterEnv, 0.65),
                    map(Source::Y, Target::Delay, 0.8),
                    map(Source::Y, Target::Reverb, 0.4),
                    map(Source::Y, Target::Decay, 0.4),
                ],
            )
        }
        8 => {
            s.oscillators = [
                osc(Wave::Pulse, 0.7, -12., 1, 0., 0.),
                osc(Wave::Saw, 0.35, -12., 1, 0., 0.),
                osc(Wave::Sine, 0.35, -24., 1, 0., 0.),
            ];
            s.oscillators[0].pulse_width = 0.35;
            s.amp = env(0.002, 0.12, 0., 0.035);
            s.filter_env = env(0.001, 0.075, 0., 0.04);
            s.cutoff = 500.;
            s.filter_amount = 3.6;
            s.resonance = 0.3;
            s.drive = 0.32;
            s.voice_mode = VoiceMode::Mono;
            s.effects.reverb = 0.;
            (
                "HARD HOUSE",
                "Dry asymmetric pulse and saw over a centered sub, with a short percussive filter envelope.",
                "Sequence eighth-note offbeats. Knock adds upper bite; weight stretches the bass body.",
                "KNOCK",
                "WEIGHT",
                vec![
                    map(Source::X, Target::FilterEnv, 0.8),
                    map(Source::X, Target::Cutoff, 0.6),
                    map(Source::Y, Target::Decay, 0.65),
                    map(Source::Y, Target::Drive, 0.6),
                ],
            )
        }
        _ => {
            s.oscillators = [
                osc(Wave::Pulse, 0.7, 0., 2, 0.2, 0.5),
                osc(Wave::Saw, 0.5, 12., 2, 0.15, 0.7),
                osc(Wave::Noise, 0.045, 0., 1, 0., 0.),
            ];
            s.cutoff = 4200.;
            s.filter_amount = 2.;
            s.pitch_amount = 7.;
            s.pitch_env = env(0.001, 0.06, 0., 0.04);
            s.amp = env(0.002, 0.18, 0.6, 0.065);
            s.drive = 0.95;
            s.resonance = 0.48;
            s.lfos[0].rate = 6.;
            s.lfos[0].pwm = 0.12;
            s.effects.delay = 0.14;
            s.effects.reverb = 0.08;
            s.gain = 0.33;
            (
                "HARDCORE",
                "Octave pulse/saw attack with pitch snap, PWM, noise and dense saturation.",
                "Stab octaves or hold a screaming top line. Panic raises the pitch snap; damage adds roughness.",
                "PANIC",
                "DAMAGE",
                vec![
                    map(Source::X, Target::PitchEnv, 0.8),
                    map(Source::X, Target::Cutoff, 0.8),
                    map(Source::Y, Target::PhaseMod, 1.5),
                    map(Source::Y, Target::Pwm, 0.8),
                    map(Source::Y, Target::Drive, 0.3),
                ],
            )
        }
    };
    for (dst, route) in s.mappings.iter_mut().zip(routes) {
        *dst = route;
    }
    s.gain = [0.6, 0.48, 0.65, 0.7, 0.8, 0.85, 0.85, 0.95, 0.75, 0.7][index];
    s.mappings[10] = map(Source::Wheel, Target::Cutoff, 0.6);
    s.mappings[11] = map(Source::Wheel, Target::LfoFilter, 0.25);
    Patch {
        version: PATCH_VERSION,
        name: NAMES[index].into(),
        author: "VOIDMUTT".into(),
        category: category.into(),
        description: description.into(),
        performance: performance.into(),
        x_label: x.into(),
        y_label: y.into(),
        synth: s,
        controls: Performance::default(),
    }
}
