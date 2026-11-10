// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2026 SOUNDGYAL 99 contributors.
#pragma once
#include <array>
#include <cstdint>
#include <cstdio>
#include "plaits/dsp/voice.h"
namespace soundgyal
{
struct Patch
{
    int engine = 8;
    float harmonics = .5f, timbre = .5f, morph = .5f, aux = 0, decay = .5f, colour = .5f;
    float release = .12f, gain = .5f;
    bool valid() const noexcept;
};
// Fixed-rate milestone. Own on a single audio thread. No heap operations in next().
class Engine
{
    alignas(16) std::array<uint8_t, 65536> memory_{};
    plaits::Voice voice_{};
    std::array<plaits::Voice::Frame, 24> frames_{};
    Patch patch_{};
    std::array<bool, 128> held_{};
    std::array<float, 128> velocities_{};
    std::array<uint64_t, 128> age_{};
    uint64_t clock_ = 0;
    int cursor_ = 24, note_ = 60, trigger_ = 0;
    float velocity_ = 0, envelope_ = 0, bend_ = 0;
    bool gate_ = false;

  public:
    Engine();
    Engine(const Engine &) = delete;
    Engine &operator=(const Engine &) = delete;
    bool patch(const Patch &) noexcept;
    void noteOn(int note, float velocity) noexcept;
    void noteOff(int note) noexcept;
    void allNotesOff() noexcept;
    void bend(float semitones) noexcept;
    std::array<float, 2> next() noexcept;
};
inline constexpr const char *engines[24] = {"Virtual analog VCF",
                                            "Phase distortion",
                                            "Six-op FM 1",
                                            "Six-op FM 2",
                                            "Six-op FM 3",
                                            "Wave terrain",
                                            "String machine",
                                            "Chiptune",
                                            "Virtual analog",
                                            "Waveshaping",
                                            "Two-op FM",
                                            "Granular formant",
                                            "Additive",
                                            "Wavetable",
                                            "Chord",
                                            "Speech",
                                            "Swarm",
                                            "Filtered noise",
                                            "Particle noise",
                                            "String",
                                            "Modal",
                                            "Bass drum",
                                            "Snare drum",
                                            "Hi-hat"};
} // namespace soundgyal
