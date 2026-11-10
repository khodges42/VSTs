// SPDX-License-Identifier: GPL-3.0-or-later
#pragma once
#include <array>
#include <cmath>
#include <cstddef>
namespace soundgyal
{
struct Spec
{
    const char *id;
    const char *name;
    float min, max, initial;
    bool discrete = false;
};
enum Param : size_t
{
    Model,
    Harmonics,
    Timbre,
    Morph,
    Aux,
    Decay,
    Colour,
    Lpg,
    Mode,
    Glide,
    BendRange,
    Layer,
    Detune,
    Width,
    Sub,
    SubShape,
    SubOctave,
    FilterMode,
    Cutoff,
    Resonance,
    Keytrack,
    FilterEnv,
    DriveMode,
    Drive,
    Compensation,
    Attack,
    AmpDecay,
    Sustain,
    Release,
    ModAttack,
    ModDecay,
    ModSustain,
    ModRelease,
    Lfo1Rate,
    Lfo1Sync,
    Lfo1Shape,
    Lfo2Rate,
    Lfo2Sync,
    Lfo2Shape,
    LowMono,
    Output,
    RouteBase
};
inline constexpr std::array<Spec, RouteBase> baseSpecs{
    {{"engine", "Plaits engine", 0, 23, 8, true},
     {"harmonics", "Harmonics", 0, 1, .5f},
     {"timbre", "Timbre", 0, 1, .5f},
     {"morph", "Morph", 0, 1, .5f},
     {"aux", "MAIN / AUX", 0, 1, 0},
     {"decay", "Plaits decay", 0, 1, .5f},
     {"colour", "LPG colour", 0, 1, .5f},
     {"lpg", "LPG enabled", 0, 1, 1, true},
     {"voice_mode", "Voice mode", 0, 2, 0, true},
     {"glide", "Glide seconds", 0, 2, 0},
     {"bend_range", "Pitch bend semitones", 0, 24, 2},
     {"layer", "Detuned layer blend", 0, 1, 0},
     {"detune", "Layer detune cents", 0, 60, 12},
     {"width", "Layer stereo width", 0, 1, 0},
     {"sub", "Clean sub level", 0, 1, 0},
     {"sub_shape", "Sub waveform", 0, 1, 0, true},
     {"sub_octave", "Sub octave", -2, 0, -1, true},
     {"filter_mode", "Filter mode", 0, 3, 0, true},
     {"cutoff", "Filter cutoff Hz", 25, 18000, 12000},
     {"resonance", "Filter resonance", 0, .95f, .1f},
     {"keytrack", "Filter key tracking", 0, 1, 0},
     {"filter_env", "Filter envelope octaves", -6, 6, 0},
     {"drive_mode", "Drive mode", 0, 3, 0, true},
     {"drive", "Drive amount", 0, 1, 0},
     {"compensation", "Drive compensation", 0, 1, 1},
     {"attack", "Amp attack seconds", .001f, 5, .005f},
     {"amp_decay", "Amp decay seconds", .005f, 5, .25f},
     {"sustain", "Amp sustain", 0, 1, 1},
     {"release", "Amp release seconds", .005f, 8, .12f},
     {"mod_attack", "Mod attack seconds", .001f, 5, .005f},
     {"mod_decay", "Mod decay seconds", .005f, 5, .3f},
     {"mod_sustain", "Mod sustain", 0, 1, 0},
     {"mod_release", "Mod release seconds", .005f, 8, .15f},
     {"lfo1_rate", "LFO 1 Hz / beats", .0625f, 16, 1},
     {"lfo1_sync", "LFO 1 tempo sync", 0, 1, 1, true},
     {"lfo1_shape", "LFO 1 shape", 0, 3, 0, true},
     {"lfo2_rate", "LFO 2 Hz / beats", .0625f, 16, 2},
     {"lfo2_sync", "LFO 2 tempo sync", 0, 1, 1, true},
     {"lfo2_shape", "LFO 2 shape", 0, 3, 1, true},
     {"low_mono", "Mono bass below 120 Hz", 0, 1, 1, true},
     {"output", "Output gain dB", -48, 6, -9}}};
inline constexpr size_t routeCount = 8, paramCount = RouteBase + routeCount * 3;
inline constexpr std::array<const char *, 8> sourceNames{
    {"Off", "LFO 1", "LFO 2", "Mod envelope", "Velocity", "Mod wheel", "Key", "Amp envelope"}};
inline constexpr std::array<const char *, 11> targetNames{
    {"Harmonics", "Timbre", "Morph", "Cutoff (octaves)", "Drive", "Amplitude", "Pitch (12 st)",
     "Layer blend", "Detune", "Width", "Sub level"}};
struct Settings
{
    std::array<float, paramCount> value{};
    Settings()
    {
        for (size_t i = 0; i < RouteBase; ++i)
            value[i] = baseSpecs[i].initial;
    }
    float operator[](size_t i) const noexcept
    {
        return value[i];
    }
    float &operator[](size_t i) noexcept
    {
        return value[i];
    }
    bool valid() const noexcept
    {
        for (size_t i = 0; i < paramCount; ++i)
        {
            float lo = 0, hi = 1;
            bool discrete = false;
            if (i < RouteBase)
            {
                lo = baseSpecs[i].min;
                hi = baseSpecs[i].max;
                discrete = baseSpecs[i].discrete;
            }
            else
            {
                auto part = (i - RouteBase) % 3;
                lo = part == 2 ? -1.f : 0.f;
                hi = part == 0 ? 7.f : part == 1 ? 10.f : 1.f;
                discrete = part != 2;
            }
            if (!std::isfinite(value[i]) || value[i] < lo || value[i] > hi ||
                (discrete && value[i] != std::round(value[i])))
                return false;
        }
        return true;
    }
};
} // namespace soundgyal
