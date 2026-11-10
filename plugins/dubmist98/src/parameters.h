// SPDX-License-Identifier: GPL-3.0-or-later
#pragma once
#include <array>
#include <cmath>
#include <cstddef>
namespace dubmist
{
enum Param : size_t
{
    Mode,
    Position,
    Size,
    Density,
    Texture,
    Pitch,
    Mix,
    Spread,
    Feedback,
    Reverb,
    Freeze,
    Quality,
    InputGain,
    OutputGain,
    Bypass,
    Clear,
    Trigger,
    Sync,
    Division,
    Repeat,
    Reverse,
    Alternate,
    Quantize,
    Probability,
    Slice,
    Step,
    TapePitch,
    TapeRamp,
    FreeTempo,
    RestartBar,
    RhythmEnabled,
    Count
};
struct Spec
{
    const char *id;
    const char *name;
    float min, max, initial;
    bool discrete = false;
};
inline constexpr std::array<Spec, Count> specs{
    {{"mode", "Processing mode", 0, 3, 0, true},
     {"position", "Grain position", 0, 1, .5f},
     {"size", "Grain size", 0, 1, .5f},
     {"density", "Grain density", 0, 1, .7f},
     {"texture", "Grain texture", 0, 1, .5f},
     {"pitch", "Pitch semitones", -48, 48, 0},
     {"mix", "Wet / dry", 0, 1, .5f},
     {"spread", "Stereo spread", 0, 1, .5f},
     {"feedback", "Feedback", 0, 1, 0},
     {"reverb", "Reverb", 0, 1, 0},
     {"freeze", "Freeze", 0, 1, 0, true},
     {"quality", "Audio quality", 0, 3, 0, true},
     {"input_gain", "Input gain dB", -48, 12, 0},
     {"output_gain", "Output gain dB", -48, 6, 0},
     {"bypass", "Bypass", 0, 1, 0, true},
     {"clear", "Clear buffer", 0, 1, 0, true},
     {"trigger", "Trigger grain / retrigger", 0, 1, 0, true},
     {"sync", "Host sync", 0, 1, 1, true},
     {"division", "Repeat division", 0, 9, 2, true},
     {"repeat", "Repeat hold", 0, 1, 0, true},
     {"reverse", "Reverse slices", 0, 1, 0, true},
     {"alternate", "Alternate direction", 0, 1, 0, true},
     {"quantize", "Quantize repeat start", 0, 1, 1, true},
     {"probability", "Repeat probability", 0, 1, 1},
     {"slice", "History slice", 0, 7, 0, true},
     {"step", "Slice step", 0, 7, 0, true},
     {"tape_pitch", "Tape pitch semitones", -24, 12, 0},
     {"tape_ramp", "Tape speed ramp", -1, 1, 0},
     {"free_tempo", "Free tempo BPM", 40, 240, 140},
     {"restart_bar", "Restart on bar", 0, 1, 0, true},
     {"rhythm_enabled", "Rhythmic slice mode", 0, 1, 0, true}}};
inline constexpr std::array<const char *, 5> modes{
    {"Granular texture", "Pitch / time stretch", "Looping delay", "Spectral", "Rhythmic slices"}};
inline constexpr size_t legacyCount = 17;
inline constexpr std::array<double, 10> divisionBeats{
    {1, .5, .25, .125, .0625, 2. / 3, 1. / 3, 1. / 6, 1. / 12, 1. / 24}};
inline constexpr std::array<const char *, 10> divisionNames{
    {"1/4", "1/8", "1/16", "1/32", "1/64", "1/4 triplet", "1/8 triplet", "1/16 triplet",
     "1/32 triplet", "1/64 triplet"}};
struct Settings
{
    std::array<float, Count> values{};
    Settings()
    {
        for (size_t i = 0; i < Count; ++i)
            values[i] = specs[i].initial;
    }
    float operator[](size_t i) const noexcept
    {
        return values[i];
    }
    float &operator[](size_t i) noexcept
    {
        return values[i];
    }
    bool valid() const noexcept
    {
        for (size_t i = 0; i < Count; ++i)
            if (!std::isfinite(values[i]) || values[i] < specs[i].min || values[i] > specs[i].max ||
                (specs[i].discrete && values[i] != std::round(values[i])))
                return false;
        return true;
    }
};
struct Preset
{
    const char *name;
    Settings settings;
};
inline std::array<Preset, 12> factory()
{
    std::array<Preset, 12> bank{};
    const char *names[] = {"Classic Cloud",  "Jungle Grain Scatter", "Breakcore Micrograin",
                           "Dark 140 Mist",  "Vocal Freeze Ready",   "Dub Loop",
                           "Spectral Glass", "Stretch Bed",          "Vintage Mono"};
    for (size_t i = 0; i < 9; ++i)
        bank[i].name = names[i];
    bank[1].settings[Size] = .23f;
    bank[1].settings[Density] = .88f;
    bank[1].settings[Spread] = .7f;
    bank[1].settings[Mix] = .4f;
    bank[2].settings[Size] = .05f;
    bank[2].settings[Density] = .96f;
    bank[2].settings[Pitch] = 12;
    bank[2].settings[Texture] = .15f;
    bank[2].settings[Mix] = .65f;
    bank[3].settings[Size] = .85f;
    bank[3].settings[Pitch] = -12;
    bank[3].settings[Feedback] = .45f;
    bank[3].settings[Reverb] = .55f;
    bank[3].settings[Mix] = .55f;
    bank[4].settings[Position] = .7f;
    bank[4].settings[Size] = .7f;
    bank[4].settings[Mix] = 1;
    bank[4].settings[Spread] = .8f;
    bank[4].settings[Reverb] = .25f;
    bank[5].settings[Mode] = 2;
    bank[5].settings[Feedback] = .65f;
    bank[5].settings[Mix] = .4f;
    bank[5].settings[Texture] = .38f;
    bank[6].settings[Mode] = 3;
    bank[6].settings[Mix] = .8f;
    bank[6].settings[Texture] = .7f;
    bank[6].settings[Reverb] = .3f;
    bank[7].settings[Mode] = 1;
    bank[7].settings[Pitch] = -7;
    bank[7].settings[Size] = .7f;
    bank[7].settings[Mix] = .7f;
    bank[7].settings[Spread] = .8f;
    bank[8].settings[Quality] = 3;
    bank[8].settings[Mix] = .7f;
    bank[8].settings[Texture] = .8f;
    bank[9].name = "175 Break Roll";
    bank[10].name = "140 Reverse Fill";
    bank[11].name = "Triplet Ping-Pong";
    for (size_t i = 9; i < 12; ++i)
    {
        bank[i].settings[RhythmEnabled] = 1;
        bank[i].settings[Sync] = 0;
        bank[i].settings[Repeat] = 1;
        bank[i].settings[Mix] = .8f;
    }
    bank[9].settings[FreeTempo] = 175;
    bank[9].settings[Division] = 2;
    bank[10].settings[FreeTempo] = 140;
    bank[10].settings[Division] = 1;
    bank[10].settings[Reverse] = 1;
    bank[10].settings[TapeRamp] = -.5f;
    bank[11].settings[FreeTempo] = 135;
    bank[11].settings[Division] = 7;
    bank[11].settings[Alternate] = 1;
    bank[11].settings[Step] = 1;
    return bank;
}
} // namespace dubmist
