// SPDX-License-Identifier: GPL-3.0-or-later
#include "beat_repeat.h"
#include "kammerl_curves.h"
#include <algorithm>
#include <cmath>
#include <iostream>
#include <stdexcept>
#include <vector>
using namespace dubmist;
void check(bool b, const char *message)
{
    if (!b)
        throw std::runtime_error(message);
}
Stereo marker(int n)
{
    uint32_t a = uint32_t(n) * 1664525u + 1013904223u;
    return {{float(int(a >> 16) - 32768) / 131072.f, float((n % 997) - 498) / 1996.f}};
}
struct Result
{
    std::vector<Stereo> audio;
    int64_t capture = -1, length = 0;
};
Result run(double rate, double bpm, int division, int block, bool reverse)
{
    Settings s;
    s[RhythmEnabled] = 1;
    s[Repeat] = 0;
    s[Mix] = 1;
    s[Division] = float(division);
    s[Reverse] = reverse ? 1.f : 0.f;
    BeatRepeater repeat;
    repeat.prepare(rate, s);
    int total = int(rate * 60 / bpm * 4.1), request = int(rate * 60 / bpm * 2.37);
    Result result;
    result.audio.resize(size_t(total));
    for (int start = 0; start < total; start += block)
    {
        int count = std::min(block, total - start);
        Transport t;
        t.valid = t.playing = t.sampleValid = true;
        t.bpm = bpm;
        t.ppq = start * bpm / (60 * rate);
        t.sample = start;
        repeat.beginBlock(t, count, s);
        for (int i = 0; i < count; ++i)
        {
            int n = start + i;
            s[Repeat] = n >= request ? 1.f : 0.f;
            result.audio[size_t(n)] = repeat.next(marker(n), s);
        }
    }
    result.capture = repeat.lastCaptureFrame();
    result.length = repeat.capturedLength();
    double step = divisionBeats[size_t(division)];
    double beat = request * bpm / (60 * rate);
    double boundary = std::ceil((beat - 1.e-9) / step) * step;
    int64_t expected = int64_t(std::floor(boundary * 60 * rate / bpm + .5000001));
    check(result.capture == expected, "Capture not aligned to rounded host sample");
    int fade = int(rate * .002);
    int count = 0;
    for (int64_t offset = fade * 2 + 4; offset < result.length - 4; ++offset)
    {
        // Inspect only the first selected loop; later musical boundaries can be one
        // sample longer/shorter for non-integral beat periods.
        if (offset >= result.length / 2)
            break;
        int source =
            int(result.capture - result.length + (reverse ? result.length - 1 - offset : offset));
        auto wanted = marker(source), actual = result.audio[size_t(result.capture + offset)];
        for (int c = 0; c < 2; ++c)
            check(std::abs(wanted[c] - actual[c]) < 1.e-6f, "Slice order or reverse indexing");
        ++count;
    }
    check(count > 0, "No interior slice samples verified");
    return result;
}
void lifecycle()
{
    Settings s;
    s[RhythmEnabled] = 1;
    s[Repeat] = 1;
    s[Quantize] = 0;
    s[Mix] = 1;
    BeatRepeater r;
    r.prepare(48000, s);
    Transport t;
    t.valid = t.playing = t.sampleValid = true;
    t.bpm = 140;
    t.ppq = 0;
    t.sample = 0;
    r.beginBlock(t, 24000, s);
    for (int n = 0; n < 24000; ++n)
        r.next(marker(n), s);
    check(r.active(), "Immediate repeat activation");
    t.ppq = 100;
    t.sample = 999999;
    r.beginBlock(t, 16000, s);
    check(!r.active(), "Seek must discard prior capture");
    for (int n = 0; n < 16000; ++n)
    {
        auto x = r.next({{0, 0}}, s);
        if (n > 300)
            check(std::abs(x[0]) < 1.e-7f && std::abs(x[1]) < 1.e-7f,
                  "Stale audio after seek fade");
    }
    t.ppq = 100 + 16000 * 140 / (60. * 48000);
    t.sample += 16000;
    t.playing = false;
    r.beginBlock(t, 1000, s);
    check(!r.active(), "Transport stop");
    for (int n = 0; n < 1000; ++n)
        r.next({{0, 0}}, s);
    // With probability zero, armed slices leave live input untouched.
    s[Sync] = 0;
    s[Probability] = 0;
    r.prepare(48000, s);
    for (int start = 0; start < 100000; start += 127)
    {
        r.beginBlock({}, 127, s);
        for (int n = 0; n < 127; ++n)
        {
            auto in = marker(start + n);
            check(r.next(in, s) == in, "Probability zero must pass dry input");
        }
    }
    // A frozen snapshot stays valid after the rolling history's eight-second span.
    s[Probability] = 1;
    s[Quantize] = 0;
    r.prepare(48000, s);
    for (int start = 0; start < 48000 * 10; start += 127)
    {
        r.beginBlock({}, 127, s);
        for (int n = 0; n < 127; ++n)
            r.next(marker(start + n), s);
    }
    check(r.active(), "Long repeat hold lost snapshot");
}
void tempoAndStepping()
{
    Settings s;
    s[RhythmEnabled] = 1;
    s[Repeat] = 1;
    s[Mix] = 1;
    s[Division] = 2;
    s[Step] = 1;
    s[Alternate] = 1;
    BeatRepeater r;
    r.prepare(48000, s);
    int total = 96000, split = 36000;
    int previous = -1, boundaries = 0;
    double beat = 0;
    for (int start = 0; start < total;)
    {
        int count = std::min({127, total - start, start < split ? split - start : total - start});
        double bpm = start < split ? 140 : 175;
        Transport t;
        t.valid = t.playing = t.sampleValid = true;
        t.bpm = bpm;
        t.ppq = beat;
        t.sample = start;
        r.beginBlock(t, count, s);
        for (int n = 0; n < count; ++n)
        {
            auto out = r.next(marker(start + n), s);
            for (float v : out)
                check(std::isfinite(v), "Tempo/stepping instability");
            if (r.lastBoundaryFrame() != previous)
            {
                previous = int(r.lastBoundaryFrame());
                ++boundaries;
                double currentBeat = beat + n * bpm / (60 * 48000);
                check(std::abs(currentBeat / .25 - std::round(currentBeat / .25)) < .0002,
                      "Tempo change moved off grid");
            }
        }
        beat += count * bpm / (60 * 48000);
        start += count;
    }
    check(boundaries > 10, "Missing tempo boundaries");
}
void performanceActions()
{
    constexpr int length = 6000, capture = length * 9, fade = 96;
    for (int variant = 0; variant < 4; ++variant)
    {
        Settings s;
        s[RhythmEnabled] = 1;
        s[Mix] = 1;
        s[Repeat] = 0;
        s[Division] = 2;
        if (variant == 0)
        {
            s[Step] = 1;
            s[Alternate] = 1;
        }
        if (variant == 1)
            s[TapePitch] = 12;
        if (variant == 3)
        {
            s[Step] = 1;
            s[Alternate] = 1;
            s[RestartBar] = 1;
        }
        BeatRepeater r;
        r.prepare(48000, s);
        std::vector<Stereo> audio(18 * length);
        for (int start = 0; start < int(audio.size()); start += 127)
        {
            int count = std::min(127, int(audio.size()) - start);
            Transport t;
            t.valid = t.playing = t.sampleValid = true;
            t.bpm = 120;
            t.ppq = start / 24000.;
            t.sample = start;
            r.beginBlock(t, count, s);
            for (int i = 0; i < count; ++i)
            {
                int n = start + i;
                s[Repeat] = n >= length * 8 + length / 3 ? 1.f : 0.f;
                s[Trigger] = variant == 2 && n == capture + 800 ? 1.f : 0.f;
                if (variant == 2 && n >= capture + 800)
                    s[Quantize] = 0;
                audio[size_t(n)] = r.next(marker(n), s);
            }
        }
        for (int offset = fade * 3; offset < fade * 3 + 100; ++offset)
        {
            int n, source;
            if (variant == 0)
            {
                n = capture + length + offset;
                source = capture - length - 1 - offset;
            }
            else if (variant == 1)
            {
                n = capture + offset;
                source = capture - length + offset * 2;
            }
            else if (variant == 2)
            {
                n = capture + 800 + offset;
                source = capture - length + offset;
            }
            else
            {
                n = length * 16 + offset;
                source = capture - length + offset;
            }
            auto expected = marker(source);
            for (int c = 0; c < 2; ++c)
                check(std::abs(audio[size_t(n)][c] - expected[c]) < 1.e-6f,
                      "Stepping, alternate, tape pitch, retrigger or bar restart");
        }
    }
    check(kammerlTapeStep(1, -1, 0) == 1 && kammerlTapeStep(1, -1, 1) == .5,
          "Decelerating tape curve");
    check(kammerlTapeStep(1, 1, 0) == 1 && kammerlTapeStep(1, 1, 1) == 2,
          "Accelerating tape curve");
}
int main()
{
    try
    {
        for (double rate : {44100., 48000., 88200., 96000.})
            for (double bpm : {140., 175.})
                for (int division = 0; division < 10; ++division)
                    for (bool reverse : {false, true})
                    {
                        auto a = run(rate, bpm, division, 1, reverse);
                        auto b = run(rate, bpm, division, 127, reverse);
                        auto c = run(rate, bpm, division, 512, reverse);
                        check(a.audio == b.audio && a.audio == c.audio,
                              "Host buffer partition changed timing/audio");
                    }
        lifecycle();
        tempoAndStepping();
        performanceActions();
        std::cout << "PASS 4 rates x 2 tempi x 10 divisions x forward/reverse x 3 partitions; "
                     "sample alignment; seeks/stops; probability; long holds; tempo changes; "
                     "stepping/alternate; tape pitch/curves; retrigger; bar restart\n";
        return 0;
    }
    catch (const std::exception &e)
    {
        std::cerr << "FAIL " << e.what() << "\n";
        return 1;
    }
}
