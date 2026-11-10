// SPDX-License-Identifier: GPL-3.0-or-later
#include "engine.h"
#include <algorithm>
#include <cmath>
namespace soundgyal
{
bool Patch::valid() const noexcept
{
    if (engine < 0 || engine >= 24)
        return false;
    for (float v : {harmonics, timbre, morph, aux, decay, colour, gain})
        if (!std::isfinite(v) || v < 0 || v > 1)
            return false;
    return std::isfinite(release) && release >= .005f && release <= 10;
}
Engine::Engine()
{
    stmlib::BufferAllocator a(memory_.data(), memory_.size());
    voice_.Init(&a);
}
bool Engine::patch(const Patch &p) noexcept
{
    if (!p.valid())
        return false;
    patch_ = p;
    return true;
}
void Engine::noteOn(int n, float v) noexcept
{
    if (n < 0 || n > 127 || !std::isfinite(v))
        return;
    if (v <= 0)
    {
        noteOff(n);
        return;
    }
    held_[n] = true;
    velocities_[n] = std::min(v, 1.f);
    age_[n] = ++clock_;
    note_ = n;
    velocity_ = velocities_[n];
    trigger_ = gate_ ? 1 : 0;
    gate_ = true;
}
void Engine::noteOff(int n) noexcept
{
    if (n < 0 || n > 127)
        return;
    held_[n] = false;
    if (n != note_)
        return;
    int selected = -1;
    uint64_t newest = 0;
    for (int i = 0; i < 128; ++i)
        if (held_[i] && age_[i] > newest)
        {
            newest = age_[i];
            selected = i;
        }
    gate_ = selected >= 0;
    if (gate_)
    {
        note_ = selected;
        velocity_ = velocities_[selected];
    }
}
void Engine::allNotesOff() noexcept
{
    held_.fill(false);
    gate_ = false;
    trigger_ = 0;
}
void Engine::bend(float s) noexcept
{
    if (std::isfinite(s))
        bend_ = std::clamp(s, -48.f, 48.f);
}
std::array<float, 2> Engine::next() noexcept
{
    if (cursor_ == 24)
    {
        plaits::Patch p{};
        p.note = float(note_) + bend_;
        p.engine = patch_.engine;
        p.harmonics = patch_.harmonics;
        p.timbre = patch_.timbre;
        p.morph = patch_.morph;
        p.decay = patch_.decay;
        p.lpg_colour = patch_.colour;
        plaits::Modulations m{};
        m.trigger_patched = true;
        m.trigger = (gate_ && !trigger_) ? 1.f : 0.f;
        voice_.Render(p, m, frames_.data(), frames_.size());
        trigger_ = 0;
        cursor_ = 0;
    }
    envelope_ += gate_ ? (1.f - envelope_) * (.004f) : -envelope_ / (patch_.release * 48000.f);
    if (envelope_ < 1.e-7f && !gate_)
        envelope_ = 0;
    auto f = frames_[cursor_++];
    float x = (f.out * (1 - patch_.aux) + f.aux * patch_.aux) / 32768.f;
    x *= envelope_ * velocity_ * patch_.gain;
    return {x, x};
}
} // namespace soundgyal
