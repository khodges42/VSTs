// SPDX-License-Identifier: GPL-3.0-or-later
#pragma once
#include "parameters.h"
#include <cstdint>
#include <vector>
namespace dubmist
{
using Stereo = std::array<float, 2>;
struct Transport
{
    bool valid = false, playing = false, sampleValid = false, barValid = false;
    double bpm = 140, ppq = 0, barStart = 0, beatsPerBar = 4;
    int64_t sample = 0;
};
class BeatRepeater
{
    std::vector<Stereo> history_;
    double rate_ = 48000, anchorBeat_ = 0, anchorFrame_ = 0, nextBeat_ = 0, bpm_ = 140,
           division_ = .25;
    double freeBeat_ = 0, expectedBeat_ = 0, barStart_ = 0, beatsPerBar_ = 4;
    int64_t frame_ = 0, write_ = 0, available_ = 0, captureEnd_ = 0, captureLength_ = 0, start_ = 0;
    int64_t nextTick_ = 0, lastCapture_ = -1, lastBoundary_ = -1, expectedSample_ = 0;
    int pool_ = 0, cycle_ = 0, seam_ = 0, fadeLength_ = 96;
    uint32_t random_ = 98;
    double playhead_ = 0;
    int64_t elapsed_ = 0;
    float gain_ = 0, mix_ = .5f;
    Stereo lastWet_{}, seamFrom_{};
    bool haveClock_ = false, clockRunning_ = false, wasPlaying_ = false, lastSampleValid_ = false;
    bool captured_ = false, play_ = false, lastClear_ = false, lastTrigger_ = false,
         retrigger_ = false;
    int lastSync_ = -1, lastDivision_ = -1;
    void invalidate() noexcept;
    void schedule() noexcept;
    bool activate(const Settings &) noexcept;
    void cycle(const Settings &, bool restart) noexcept;
    Stereo read(double, bool) const noexcept;

  public:
    void prepare(double, const Settings &);
    void beginBlock(const Transport &, int frames, const Settings &) noexcept;
    Stereo next(Stereo, const Settings &) noexcept;
    int64_t lastCaptureFrame() const noexcept
    {
        return lastCapture_;
    }
    int64_t lastBoundaryFrame() const noexcept
    {
        return lastBoundary_;
    }
    int64_t capturedLength() const noexcept
    {
        return captureLength_;
    }
    bool active() const noexcept
    {
        return captured_ && play_;
    }
    bool clockAvailable() const noexcept
    {
        return clockRunning_;
    }
};
} // namespace dubmist
