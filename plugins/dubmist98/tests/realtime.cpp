// SPDX-License-Identifier: GPL-3.0-or-later
#include "engine.h"

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
        auto engine = std::make_unique<dubmist::DesktopEngine>();
        dubmist::Settings s;
        s[dubmist::Mix] = 1;
        s[dubmist::Feedback] = 1;
        s[dubmist::Reverb] = 1;
        engine->prepare(96000, s);
        monitor = true;
        float peak = 0;
        for (int n = 0; n < 96000 * 30; ++n)
        {
            if (n >= 96000 * 12 && n % 48000 == 0)
            {
                s[dubmist::Freeze] = s[dubmist::Freeze] > 0 ? 0.f : 1.f;
                s[dubmist::Mode] = float((n / 48000) % 4);
                s[dubmist::Quality] = float((n / 96000) % 4);
                engine->settings(s);
            }
            auto y = engine->next({{n % 12000 == 0 ? .7f : 1.e-12f, n % 9000 == 0 ? -.5f : 0}});
            for (auto v : y)
            {
                if (!std::isfinite(v) || std::abs(v) > 2)
                {
                    monitor = false;
                    check(false, "Feedback instability");
                }
                peak = std::max(peak, std::abs(v));
            }
        }
        monitor = false;
        check(allocations == 0, "Audio allocation");
        s[dubmist::Clear] = 1;
        engine->settings(s);
        monitor = true;
        for (int n = 0; n < 96000; ++n)
            engine->next({{0, 0}});
        monitor = false;
        check(allocations == 0, "Clear allocated");
        s[dubmist::RhythmEnabled] = 1;
        s[dubmist::Sync] = 0;
        s[dubmist::Repeat] = 1;
        s[dubmist::Alternate] = 1;
        s[dubmist::Step] = 1;
        engine->settings(s);
        monitor = true;
        for (int start = 0; start < 96000 * 10; start += 127)
        {
            engine->beginBlock({}, 127);
            for (int i = 0; i < 127; ++i)
            {
                auto y = engine->next({{.1f, -.1f}});
                if (!std::isfinite(y[0]))
                {
                    monitor = false;
                    check(false, "Rhythm allocation stress instability");
                }
            }
        }
        monitor = false;
        check(allocations == 0, "Rhythm processing allocated");
        std::cout << "PASS rhythm and 30 s wraparound, high feedback, freeze/mode/quality "
                     "switching, clear; "
                     "zero C++ allocations; peak "
                  << peak << "\n";
        return 0;
    }
    catch (const std::exception &e)
    {
        monitor = false;
        std::cerr << "FAIL " << e.what() << "\n";
        return 1;
    }
}
