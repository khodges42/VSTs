// SPDX-License-Identifier: GPL-3.0-or-later
#include "engine.h"
#include <cmath>
#include <algorithm>
#include <string>
#include <fstream>
#include <iostream>
#include <stdexcept>
#include <memory>
#include <vector>
#include <limits>
void require(bool b, const char *m)
{
    if (!b)
        throw std::runtime_error(m);
}
void u16(std::ostream &f, unsigned n)
{
    f.put(char(n));
    f.put(char(n >> 8));
}
void u32(std::ostream &f, unsigned n)
{
    u16(f, n);
    u16(f, n >> 16);
}
void wav(const std::string &name, const std::vector<float> &x)
{
    std::ofstream f(name, std::ios::binary);
    f.write("RIFF", 4);
    u32(f, 36 + unsigned(x.size() * 2));
    f.write("WAVEfmt ", 8);
    u32(f, 16);
    u16(f, 1);
    u16(f, 1);
    u32(f, 48000);
    u32(f, 96000);
    u16(f, 2);
    u16(f, 16);
    f.write("data", 4);
    u32(f, unsigned(x.size() * 2));
    for (float v : x)
        u16(f, unsigned(int16_t(std::clamp(v, -1.f, 1.f) * 32767)));
    require(bool(f), "WAV write");
}
int main()
{
    try
    {
        soundgyal::Patch p;
        p.engine = 24;
        require(!p.valid(), "engine validation");
        p.engine = 8;
        p.morph = std::numeric_limits<float>::quiet_NaN();
        require(!p.valid(), "NaN validation");
        p.morph = .5f;
        for (int i = 0; i < 24; ++i)
        {
            auto e = std::make_unique<soundgyal::Engine>();
            p.engine = i;
            require(e->patch(p), "patch");
            e->noteOn(48, .8f);
            double energy = 0;
            float peak = 0;
            std::vector<float> audio(96000);
            for (int n = 0; n < 96000; ++n)
            {
                if (n == 48000)
                    e->noteOff(48);
                auto x = e->next();
                require(std::isfinite(x[0]), "nonfinite");
                require(x[0] == x[1], "mono compatibility");
                energy += x[0] * x[0];
                peak = std::max(peak, std::abs(x[0]));
                audio[n] = x[0];
            }
            require(energy > 0.01, "silent engine");
            require(peak <= 1, "unbounded output");
            for (int n = 0; n < 480000; ++n)
                e->next();
            require(e->next()[0] == 0, "release silence");
            wav("engine-" + std::to_string(i) + ".wav", audio);
            std::cout << i << " " << soundgyal::engines[i] << " energy=" << energy
                      << " peak=" << peak << "\n";
        }
        // No event positions depend on caller partitioning. Test a deterministic VA engine.
        auto render = [](int chunk)
        {
            auto e = std::make_unique<soundgyal::Engine>();
            e->noteOn(36, 1);
            std::vector<float> a(4800);
            for (int start = 0; start < 4800; start += chunk)
                for (int n = start; n < std::min(start + chunk, 4800); ++n)
                {
                    if (n == 119)
                        e->noteOn(40, .7f);
                    if (n == 993)
                        e->noteOff(40);
                    if (n == 2001)
                        e->noteOff(36);
                    a[n] = e->next()[0];
                }
            return a;
        };
        require(render(1) == render(127), "buffer partition dependence");
        auto e = std::make_unique<soundgyal::Engine>();
        e->noteOn(30, 1);
        e->noteOn(40, 1);
        e->noteOff(40);
        for (int n = 0; n < 4800; ++n)
            e->next();
        e->allNotesOff();
        for (int n = 0; n < 480000; ++n)
            e->next();
        require(e->next()[0] == 0, "all notes off");
        std::cout << "PASS: 24 engines; validation; mono; release; partitioning; all notes off\n";
        return 0;
    }
    catch (const std::exception &e)
    {
        std::cerr << "FAIL: " << e.what() << "\n";
        return 1;
    }
}
