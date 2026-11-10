// SPDX-License-Identifier: GPL-3.0-or-later
#include "beat_repeat.h"
#include "kammerl_curves.h"
#include <algorithm>
#include <cmath>
#include <stdexcept>
namespace dubmist
{
void BeatRepeater::prepare(double rate, const Settings &s)
{
    if (!std::isfinite(rate) || rate < 32000 || rate > 192000)
        throw std::invalid_argument("Invalid rhythm rate");
    rate_ = rate;
    history_.assign(size_t(rate * 8), Stereo{});
    frame_ = write_ = available_ = 0;
    captured_ = play_ = false;
    freeBeat_ = 0;
    haveClock_ = false;
    lastSync_ = -1;
    lastDivision_ = -1;
    random_ = 98;
    gain_ = 0;
    mix_ = s[Mix];
    fadeLength_ = std::max(1, int(rate * .002));
    lastClear_ = lastTrigger_ = retrigger_ = false;
    lastWet_ = seamFrom_ = {};
    lastCapture_ = lastBoundary_ = -1;
}
void BeatRepeater::invalidate() noexcept
{
    available_ = 0;
    captured_ = play_ = false;
    retrigger_ = false;
    pool_ = 0;
}
void BeatRepeater::schedule() noexcept
{
    nextTick_ = int64_t(
        std::floor(anchorFrame_ + (nextBeat_ - anchorBeat_) * 60 * rate_ / bpm_ + .5000001));
}
void BeatRepeater::beginBlock(const Transport &t, int count, const Settings &s) noexcept
{
    bool sync = s[Sync] > .5f;
    int division = int(s[Division]);
    bool changed = lastSync_ != int(sync) || lastDivision_ != division;
    division_ = divisionBeats[size_t(division)];
    lastSync_ = int(sync);
    lastDivision_ = division;
    bool valid = sync ? (t.valid && std::isfinite(t.ppq) && std::isfinite(t.bpm) && t.bpm >= 20 &&
                         t.bpm <= 400)
                      : true;
    bool running = valid && (!sync || t.playing);
    double beat = sync ? t.ppq : freeBeat_;
    double bpm = sync ? t.bpm : s[FreeTempo];
    bool discontinuity = changed || !haveClock_ || running != wasPlaying_;
    if (sync && haveClock_ && running && wasPlaying_)
    {
        if (t.sampleValid && lastSampleValid_ && t.sample != expectedSample_)
            discontinuity = true;
        if (std::abs(beat - expectedBeat_) > 2 * bpm / (60 * rate_))
            discontinuity = true;
    }
    if (discontinuity || !running)
        invalidate();
    bpm_ = valid ? bpm : 140;
    anchorBeat_ = valid ? beat : 0;
    anchorFrame_ = double(frame_);
    barStart_ = t.barValid ? t.barStart : 0;
    beatsPerBar_ = std::isfinite(t.beatsPerBar) && t.beatsPerBar > 0 ? t.beatsPerBar : 4;
    clockRunning_ = running;
    if (discontinuity && running)
        nextBeat_ = std::ceil((beat - 1.e-9) / division_) * division_;
    if (running)
        schedule();
    expectedBeat_ = beat + (running ? count * bpm_ / (60 * rate_) : 0);
    expectedSample_ = t.sample + count;
    lastSampleValid_ = sync && t.sampleValid;
    wasPlaying_ = running;
    haveClock_ = valid;
}
bool BeatRepeater::activate(const Settings &s) noexcept
{
    int64_t length = std::clamp(int64_t(std::floor(division_ * 60 * rate_ / bpm_ + .5)), int64_t(8),
                                int64_t(history_.size()));
    if (available_ < length)
        return false;
    captureLength_ = length;
    captureEnd_ = write_;
    pool_ = int(std::min(int64_t(8), available_ / length));
    captured_ = true;
    cycle_ = -1;
    lastCapture_ = frame_;
    cycle(s, true);
    return true;
}
void BeatRepeater::cycle(const Settings &s, bool restart) noexcept
{
    cycle_ = restart ? 0 : cycle_ + 1;
    int slot = (int(s[Slice]) + cycle_ * int(s[Step])) % pool_;
    start_ = captureEnd_ - (slot + 1) * captureLength_;
    playhead_ = 0;
    elapsed_ = 0;
    random_ = random_ * 1664525u + 1013904223u;
    play_ = double(random_) / 4294967296. < s[Probability];
    seam_ = fadeLength_;
    seamFrom_ = lastWet_;
}
Stereo BeatRepeater::read(double position, bool reverse) const noexcept
{
    // Double-precision bounded addressing avoids the embedded fixed-point index
    // overflow hazards at desktop rates and long-running absolute sample counters.
    auto at = [&](int64_t i)
    {
        i = (i % captureLength_ + captureLength_) % captureLength_;
        int64_t absolute = start_ + i;
        return history_[size_t(absolute % int64_t(history_.size()))];
    };
    double p = reverse ? double(captureLength_ - 1) - position : position;
    auto i = int64_t(std::floor(p));
    float f = float(p - std::floor(p));
    auto a = at(i - 1), b = at(i), c = at(i + 1), d = at(i + 2);
    Stereo y{};
    for (int ch = 0; ch < 2; ++ch)
    {
        float c1 = .5f * (c[ch] - a[ch]);
        float c2 = a[ch] - 2.5f * b[ch] + 2 * c[ch] - .5f * d[ch];
        float c3 = .5f * (d[ch] - a[ch]) + 1.5f * (b[ch] - c[ch]);
        y[ch] = ((c3 * f + c2) * f + c1) * f + b[ch];
    }
    return y;
}
Stereo BeatRepeater::next(Stereo x, const Settings &s) noexcept
{
    bool clear = s[Clear] > .5f;
    if (clear && !lastClear_)
        invalidate();
    lastClear_ = clear;
    bool trigger = s[Trigger] > .5f;
    if (trigger && !lastTrigger_)
        retrigger_ = true;
    lastTrigger_ = trigger;
    bool tick = false;
    double tickBeat = nextBeat_;
    if (clockRunning_ && frame_ >= nextTick_)
    {
        tick = true;
        lastBoundary_ = frame_;
        nextBeat_ += division_;
        schedule();
    }
    bool requested =
        s[RhythmEnabled] > .5f && (s[Repeat] > .5f || s[Freeze] > .5f) && clockRunning_;
    if (!requested)
    {
        captured_ = false;
        play_ = false;
    }
    bool mayStart = s[Quantize] < .5f || tick;
    if (requested && !captured_ && mayStart)
        activate(s);
    else if (requested && captured_ && (tick || (retrigger_ && mayStart)))
    {
        double bar = (tickBeat - barStart_) / beatsPerBar_;
        bool restart =
            retrigger_ || (s[RestartBar] > .5f && tick && std::abs(bar - std::round(bar)) < 1.e-7);
        cycle(s, restart);
    }
    if (mayStart)
        retrigger_ = false;
    if (!captured_)
    {
        auto &dest = history_[size_t(write_ % int64_t(history_.size()))];
        for (int c = 0; c < 2; ++c)
            dest[c] = std::isfinite(x[c]) ? std::clamp(x[c], -4.f, 4.f) : 0;
        ++write_;
        available_ = std::min(available_ + 1, int64_t(history_.size()));
    }
    Stereo wet = lastWet_;
    if (captured_)
    {
        bool reverse = (s[Reverse] > .5f) != (s[Alternate] > .5f && (cycle_ & 1));
        wet = read(playhead_, reverse);
        if (seam_ > 0)
        {
            float a = float(fadeLength_ - seam_ + 1) / fadeLength_;
            for (int c = 0; c < 2; ++c)
                wet[c] = seamFrom_[c] * (1 - a) + wet[c] * a;
            --seam_;
        }
        double progress = std::clamp(double(elapsed_) / (division_ * 60 * rate_ / bpm_), 0., 1.);
        double speed = kammerlTapeStep(std::exp2(s[TapePitch] / 12.), s[TapeRamp], progress);
        playhead_ += std::clamp(speed, .125, 4.);
        ++elapsed_;
        if (playhead_ >= captureLength_)
        {
            playhead_ = std::fmod(playhead_, double(captureLength_));
            seam_ = fadeLength_;
            seamFrom_ = wet;
        }
        lastWet_ = wet;
    }
    float target = requested && play_ ? 1.f : 0.f;
    gain_ += std::clamp(target - gain_, -1.f / fadeLength_, 1.f / fadeLength_);
    if (!captured_ && gain_ == 0)
        lastWet_ = {};
    mix_ += (s[Mix] - mix_) * .002f;
    if (std::abs(s[Mix] - mix_) < .0001f)
        mix_ = s[Mix];
    float amount = gain_ * mix_;
    Stereo y{};
    for (int c = 0; c < 2; ++c)
        y[c] = x[c] * (1 - amount) + wet[c] * amount;
    if (s[Sync] < .5f)
        freeBeat_ += s[FreeTempo] / (60 * rate_);
    ++frame_;
    return y;
}
} // namespace dubmist
