// SPDX-License-Identifier: GPL-3.0-or-later
#include "synth.h"
#include "factory.h"
#include <iostream>
#include <memory>
#include <chrono>
#include <cstdlib>
#include <new>
#include <stdexcept>
thread_local bool monitor = false;
thread_local unsigned allocations = 0;
void *operator new(std::size_t size)
{
    if (monitor)
        ++allocations;
    if (auto *p = std::malloc(size ? size : 1))
        return p;
    throw std::bad_alloc();
}
void *operator new[](std::size_t size)
{
    return ::operator new(size);
}
void operator delete(void *p) noexcept
{
    std::free(p);
}
void operator delete[](void *p) noexcept
{
    std::free(p);
}
void operator delete(void *p, std::size_t) noexcept
{
    std::free(p);
}
void operator delete[](void *p, std::size_t) noexcept
{
    std::free(p);
}
void check(bool b, const char *m)
{
    if (!b)
        throw std::runtime_error(m);
}
int main()
{
    try
    {
        auto a = std::make_unique<soundgyal::DesktopEngine>();
        a->prepare(48000);
        auto settings = soundgyal::makeFactory()[5].patch;
        settings[soundgyal::Mode] = 2;
        settings[soundgyal::Resonance] = .95f;
        settings[soundgyal::DriveMode] = 3;
        settings[soundgyal::Drive] = 1;
        check(a->synth().settings(settings), "Valid settings");
        for (int i = 0; i < 8; ++i)
            a->synth().noteOn(i, 24 + i * 3, .8f);
        auto begin = std::chrono::steady_clock::now();
        float peak = 0;
        monitor = true;
        for (int n = 0; n < 48000; ++n)
        {
            if (n % 1000 == 0)
            {
                a->synth().noteOn(0, 30 + n % 50, .8f);
                a->synth().controller(0, 1, .8f);
                a->synth().pitchBend(0, n % 2000 ? 1.f : -1.f);
            }
            auto x = a->next();
            for (auto v : x)
            {
                if (!std::isfinite(v))
                {
                    monitor = false;
                    check(false, "Extreme settings instability");
                }
                peak = std::max(peak, std::abs(v));
            }
        }
        monitor = false;
        check(allocations == 0, "Audio path C++ allocation");
        check(peak < 16, "Extreme gain explosion");
        for (int ch = 0; ch < 16; ++ch)
            a->synth().controller(ch, 123, 0);
        for (int n = 0; n < 240000; ++n)
            a->next();
        check(a->synth().activeVoices() == 0, "Voice lifecycle");
        check(std::abs(a->next()[0]) < 1.e-7f, "Post-release silence");
        auto b = std::make_unique<soundgyal::DesktopEngine>();
        b->prepare(48000);
        a = std::make_unique<soundgyal::DesktopEngine>();
        a->prepare(48000);
        auto p = soundgyal::Settings{};
        p[soundgyal::Lpg] = 0;
        check(a->synth().settings(p), "Settings");
        p[soundgyal::RouteBase] = 1;
        p[soundgyal::RouteBase + 1] = 2;
        p[soundgyal::RouteBase + 2] = .7f;
        check(b->synth().settings(p), "Matrix settings");
        a->synth().noteOn(0, 36, 1);
        b->synth().noteOn(0, 36, 1);
        double delta = 0;
        for (int n = 0; n < 24000; ++n)
            delta += std::abs(a->next()[0] - b->next()[0]);
        check(delta > 1, "Matrix does not affect audio");
        // Sustain holds released notes, pedal up releases them.
        b->synth().controller(0, 64, 1);
        b->synth().noteOff(0, 36);
        for (int n = 0; n < 48000; ++n)
            b->next();
        check(b->synth().activeVoices() == 1, "Sustain hold");
        b->synth().controller(0, 64, 0);
        for (int n = 0; n < 48000; ++n)
            b->next();
        check(b->synth().activeVoices() == 0, "Sustain release");
        std::cout << "PASS zero C++ allocations; eight voices / stealing; extremes; matrix; "
                     "sustain; silence. Peak "
                  << peak << ". Test elapsed "
                  << std::chrono::duration<double>(std::chrono::steady_clock::now() - begin).count()
                  << " s\n";
        return 0;
    }
    catch (const std::exception &e)
    {
        monitor = false;
        std::cerr << "FAIL " << e.what() << "\n";
        return 1;
    }
}
