// SPDX-License-Identifier: GPL-3.0-or-later
#include "engine.h"
#include "stmlib/utils/random.h"
#include <algorithm>
#include <cstring>
#include <stdexcept>
namespace dubmist
{
void CloudsCore::initialize() noexcept
{
    // Original firmware relies on zero-initialized static RAM. Match that contract.
    std::memset(&processor_, 0, sizeof(processor_));
    large_.fill(0);
    small_.fill(0);
    processor_.Init(large_.data(), large_.size(), small_.data(), small_.size());
    processor_.set_playback_mode(static_cast<clouds::PlaybackMode>(mode_));
    processor_.set_quality(quality_);
    processor_.Prepare();
}
CloudsCore::CloudsCore()
{
    initialize();
}
bool CloudsCore::settings(const Settings &s) noexcept
{
    if (!s.valid() || s[Mode] > 3)
        return false;
    target_ = s;
    return true;
}
void CloudsCore::render() noexcept
{
    auto previous = stmlib::Random::state();
    stmlib::Random::Seed(random_);
    float previousFade = fade_;
    bool rebuild = false;
    bool clear = target_[Clear] > .5f;
    if (clear && !lastClear_)
        pendingClear_ = true;
    lastClear_ = clear;
    if (int(target_[Mode]) != mode_ || int(target_[Quality]) != quality_ || pendingClear_)
        changing_ = true;
    if (changing_)
    {
        fade_ = std::max(0.f, fade_ - .1f);
        if (fade_ == 0)
            rebuild = true;
    }
    else
        fade_ = std::min(1.f, fade_ + .1f);
    for (size_t i = 0; i < Count; ++i)
        smooth_[i] = specs[i].discrete ? target_[i] : smooth_[i] + (target_[i] - smooth_[i]) * .1f;
    processor_.Prepare();
    auto *p = processor_.mutable_parameters();
    p->position = smooth_[Position];
    p->size = smooth_[Size];
    p->density = smooth_[Density];
    p->texture = smooth_[Texture];
    p->pitch = smooth_[Pitch];
    p->dry_wet = smooth_[Mix];
    p->stereo_spread = smooth_[Spread];
    p->feedback = smooth_[Feedback];
    p->reverb = smooth_[Reverb];
    p->freeze = target_[Freeze] > .5f;
    bool trigger = target_[Trigger] > .5f;
    p->trigger = trigger && !lastTrigger_;
    p->gate = trigger;
    lastTrigger_ = trigger;
    processor_.Process(input_.data(), output_.data(), 32);
    for (size_t i = 0; i < output_.size(); ++i)
    {
        float fade = previousFade + (fade_ - previousFade) * float(i + 1) / 32;
        auto &f = output_[i];
        f.l = short(float(f.l) * fade);
        f.r = short(float(f.r) * fade);
    }
    if (rebuild)
    {
        mode_ = int(target_[Mode]);
        quality_ = int(target_[Quality]);
        initialize();
        pendingClear_ = false;
        changing_ = false;
    }
    random_ = stmlib::Random::state();
    stmlib::Random::Seed(previous);
}
Frame CloudsCore::next(Frame x) noexcept
{
    Frame y{{output_[cursor_].l / 32768.f, output_[cursor_].r / 32768.f}};
    for (int c = 0; c < 2; ++c)
    {
        float v = std::isfinite(x[c]) ? std::clamp(x[c], -1.f, .999969f) : 0;
        if (c == 0)
            input_[cursor_].l = short(v * 32768);
        else
            input_[cursor_].r = short(v * 32768);
    }
    if (++cursor_ == 32)
    {
        render();
        cursor_ = 0;
    }
    return y;
}
void SincBank::prepare(double ratio)
{
    double cutoff = std::min(1., ratio) * .94;
    constexpr double pi = 3.141592653589793;
    for (int p = 0; p < 256; ++p)
    {
        double sum = 0;
        for (int j = 0; j < 64; ++j)
        {
            double t = j - 32. + p / 256.;
            double sinc = std::abs(t) < 1.e-10 ? cutoff : std::sin(pi * t * cutoff) / (pi * t);
            double window = .5 + .5 * std::cos(pi * t / 33);
            taps_[p][j] = float(sinc * window);
            sum += taps_[p][j];
        }
        for (auto &v : taps_[p])
            v = float(v / sum);
    }
}
Frame SincBank::read(const std::array<Frame, 128> &history, double pos) const noexcept
{
    auto index = int64_t(pos);
    int phase = std::clamp(int((pos - double(index)) * 256), 0, 255);
    Frame y{};
    for (int j = 0; j < 64; ++j)
        for (int c = 0; c < 2; ++c)
            y[c] += history[size_t(index - j) & 127][c] * taps_[phase][j];
    return y;
}
void DesktopEngine::prepare(double rate, const Settings &s)
{
    if (!std::isfinite(rate) || rate < 32000 || rate > 192000 || !s.valid())
        throw std::invalid_argument("Dubmist supports 32-192 kHz and validated settings");
    rate_ = rate;
    host_ = -1;
    internal_ = -1;
    input_ = {};
    output_ = {};
    dry_ = {};
    latency_ = int(std::ceil(32 + 64 * rate / 32000));
    down_.prepare(32000 / rate);
    up_.prepare(rate / 32000);
    target_ = smooth_ = s;
    bypass_ = s[Bypass];
    repeat_.prepare(rate, s);
    repeatDelay_ = {};
    rhythmBlend_ = s[RhythmEnabled] > .5f ? 1.f : 0.f;
    auto cloudsSettings = s;
    if (cloudsSettings[RhythmEnabled] > .5f)
        cloudsSettings[Mode] = 0;
    core_.settings(cloudsSettings);
}
bool DesktopEngine::settings(const Settings &s) noexcept
{
    if (!s.valid())
        return false;
    target_ = s;
    auto cloudsSettings = s;
    if (cloudsSettings[RhythmEnabled] > .5f)
        cloudsSettings[Mode] = 0;
    core_.settings(cloudsSettings);
    return true;
}
Frame DesktopEngine::next(Frame x) noexcept
{
    ++host_;
    for (size_t i : {size_t(InputGain), size_t(OutputGain)})
        smooth_[i] += (target_[i] - smooth_[i]) * .002f;
    float inGain = std::pow(10.f, smooth_[InputGain] / 20);
    for (auto &v : x)
    {
        if (!std::isfinite(v))
            v = 0;
        v *= inGain;
    }
    input_[size_t(host_) & 127] = x;
    dry_[size_t(host_) & 2047] = x;
    double time = double(host_) * 32000 / rate_;
    while (internal_ < int64_t(time))
    {
        ++internal_;
        double sourceTime = double(internal_) * rate_ / 32000;
        auto in = down_.read(input_, sourceTime);
        output_[size_t(internal_) & 127] = core_.next(in);
    }
    auto y = up_.read(output_, time);
    repeatDelay_[size_t(host_) & 2047] = repeat_.next(x, target_);
    auto rhythm = repeatDelay_[size_t(host_ - latency_) & 2047];
    rhythmBlend_ += std::clamp((target_[RhythmEnabled] > .5f ? 1.f : 0.f) - rhythmBlend_,
                               -1.f / 256, 1.f / 256);
    for (int c = 0; c < 2; ++c)
        y[c] = y[c] * (1 - rhythmBlend_) + rhythm[c] * rhythmBlend_;

    auto dry = dry_[size_t(host_ - latency_) & 2047];
    bypass_ += std::clamp(target_[Bypass] - bypass_, -1.f / 256, 1.f / 256);
    float gain = std::pow(10.f, smooth_[OutputGain] / 20);
    for (int c = 0; c < 2; ++c)
    {
        y[c] = (y[c] * (1 - bypass_) + dry[c] * bypass_) * gain;
        if (!std::isfinite(y[c]))
            y[c] = 0;
    }
    return y;
}
} // namespace dubmist
