// SPDX-License-Identifier: GPL-3.0-or-later
#pragma once
#include "parameters.h"
#include "beat_repeat.h"
#include "clouds/dsp/granular_processor.h"
namespace dubmist
{
using Frame = std::array<float, 2>;
class CloudsCore
{
    alignas(16) std::array<uint8_t, 118784> large_{};
    alignas(16) std::array<uint8_t, 65408> small_{};
    clouds::GranularProcessor processor_;
    std::array<clouds::ShortFrame, 32> input_{}, output_{};
    Settings target_, smooth_;
    int cursor_ = 0, mode_ = 0, quality_ = 0;
    float fade_ = 1;
    bool changing_ = false, lastClear_ = false, lastTrigger_ = false, pendingClear_ = false;
    uint32_t random_ = 33;
    void initialize() noexcept;
    void render() noexcept;

  public:
    CloudsCore();
    bool settings(const Settings &) noexcept;
    Frame next(Frame) noexcept;
    bool frozen() const noexcept
    {
        return processor_.frozen();
    }
};
class SincBank
{
    std::array<std::array<float, 64>, 256> taps_{};

  public:
    void prepare(double ratio);
    Frame read(const std::array<Frame, 128> &history, double position) const noexcept;
};
class DesktopEngine
{
    CloudsCore core_;
    BeatRepeater repeat_;
    std::array<Frame, 2048> repeatDelay_{};
    float rhythmBlend_ = 0;
    SincBank down_, up_;
    std::array<Frame, 128> input_{}, output_{};
    std::array<Frame, 2048> dry_{};
    Settings target_, smooth_;
    double rate_ = 48000;
    int64_t host_ = -1, internal_ = -1;
    int latency_ = 128;
    float bypass_ = 0;

  public:
    void prepare(double, const Settings &);
    bool settings(const Settings &s) noexcept;
    Frame next(Frame) noexcept;
    void beginBlock(const Transport &t, int count) noexcept
    {
        repeat_.beginBlock(t, count, target_);
    }
    const BeatRepeater &rhythm() const noexcept
    {
        return repeat_;
    }
    int latency() const noexcept
    {
        return latency_;
    }
};
} // namespace dubmist
