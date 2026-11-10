// SPDX-License-Identifier: GPL-3.0-or-later
#pragma once
#include "engine.h"
#include "parameters.h"
namespace soundgyal
{
struct Envelope
{
    float level = 0, releaseStep = 0;
    int stage = 0;
    void on(bool legato) noexcept
    {
        if (!legato || !stage)
            stage = 1;
    }
    void off(float seconds) noexcept
    {
        releaseStep = level / (seconds * 48000);
        stage = 4;
    }
    float next(float attack, float decay, float sustain) noexcept;
};
struct PlaitsUnit
{
    alignas(16) std::array<uint8_t, 65536> ram{};
    plaits::Voice dsp{};
    std::array<plaits::Voice::Frame, 24> frames{};
    int cursor = 24;
    PlaitsUnit();
    float next(const Settings &, float note, float velocity, bool gate, bool retrigger,
               const std::array<float, 11> &mod) noexcept;
};
struct SynthVoice
{
    PlaitsUnit main, layer;
    Envelope amp, env;
    int note = 60, channel = 0;
    bool key = false, active = false, retrigger = false;
    uint64_t age = 0;
    float velocity = 0, pitch = 60, phase = 0, ic1[2]{}, ic2[2]{}, lowSide = 0;
    std::array<float, 2> last{}, residual{};

    float drivePrevious[2]{};
};
class Synth
{
    std::array<SynthVoice, 8> voices_{};
    Settings target_, smooth_;
    std::array<std::array<unsigned, 128>, 16> held_{};
    std::array<std::array<uint64_t, 128>, 16> noteAge_{};
    std::array<std::array<float, 128>, 16> velocity_{};
    std::array<float, 16> bends_{}, wheels_{};
    std::array<bool, 16> sustain_{};
    float dcInput_[2]{}, dcOutput_[2]{};
    double lfo_[2]{};
    float tempo_ = 140;
    uint64_t clock_ = 0;
    uint32_t random_ = 33;
    void start(SynthVoice &, int, int, float, bool) noexcept;

  public:
    bool settings(const Settings &s) noexcept;
    void tempo(float bpm) noexcept;
    void noteOn(int channel, int note, float velocity) noexcept;
    void noteOff(int channel, int note) noexcept;
    void controller(int channel, int cc, float value) noexcept;
    void pitchBend(int channel, float normalized) noexcept;
    void reset() noexcept;
    std::array<float, 2> next() noexcept;
    int activeVoices() const noexcept;
};
// Causal windowed-sinc sample-rate conversion, precomputed off the audio thread.
class DesktopEngine
{
    Synth synth_;
    std::array<std::array<float, 64>, 256> kernel_{};
    std::array<std::array<float, 2>, 128> history_{};
    double position_ = 0, ratio_ = 1;
    int64_t generated_ = -1;

  public:
    void prepare(double rate);
    Synth &synth() noexcept
    {
        return synth_;
    }
    std::array<float, 2> next() noexcept;
    int latency(double rate) const noexcept
    {
        return int(std::ceil(32 * rate / 48000));
    }
};
} // namespace soundgyal
