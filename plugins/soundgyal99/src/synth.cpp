// SPDX-License-Identifier: GPL-3.0-or-later
#include "synth.h"
#include "stmlib/utils/random.h"
#include <algorithm>
#include <cmath>
#include <stdexcept>
namespace soundgyal
{
namespace
{
constexpr float pi = 3.14159265358979323846f;
float clamp(float v, float a = 0, float b = 1)
{
    return std::clamp(v, a, b);
}
float shape(double phase, int kind)
{
    float p = float(phase);
    switch (kind)
    {
    case 1:
        return 1 - 4 * std::abs(p - .5f);
    case 2:
        return 2 * p - 1;
    case 3:
        return p < .5f ? 1.f : -1.f;
    default:
        return std::sin(2 * pi * p);
    }
}
float distort(float x, int mode, float amount)
{
    float g = 1 + amount * 24;
    x *= g;
    switch (mode)
    {
    case 1:
        return std::tanh(x);
    case 2:
        return clamp(x, -1, 1);
    case 3:
        return 1 - 4 * std::abs((x * .25f + .25f - std::floor(x * .25f + .25f)) - .5f);
    default:
        return x / g;
    }
}
} // namespace
float Envelope::next(float a, float d, float s) noexcept
{
    switch (stage)
    {
    case 1:
        level += 1 / (48000 * a);
        if (level >= 1)
        {
            level = 1;
            stage = 2;
        }
        break;
    case 2:
        level -= (1 - s) / (48000 * d);
        if (level <= s)
        {
            level = s;
            stage = s > 0 ? 3 : 0;
        }
        break;
    case 3:
        level = s;
        break;
    case 4:
        level -= releaseStep;
        if (level <= 0)
        {
            level = 0;
            stage = 0;
        }
        break;
    default:
        level = 0;
    }
    return level;
}
PlaitsUnit::PlaitsUnit()
{
    stmlib::BufferAllocator a(ram.data(), ram.size());
    dsp.Init(&a);
}
float PlaitsUnit::next(const Settings &s, float note, float velocity, bool gate, bool retrigger,
                       const std::array<float, 11> &mod) noexcept
{
    if (cursor == 24)
    {
        plaits::Patch p{};
        p.engine = int(s[Model]);
        p.note = note;
        p.harmonics = clamp(s[Harmonics] + mod[0]);
        p.timbre = clamp(s[Timbre] + mod[1]);
        p.morph = clamp(s[Morph] + mod[2]);
        p.decay = s[Decay];
        p.lpg_colour = s[Colour];
        plaits::Modulations m{};
        m.trigger_patched = true;
        m.trigger = gate && !retrigger ? 1.f : 0.f;
        // External level holds the LPG open for sustained bass. A separate desktop
        // flag bypasses LPG while retaining model triggers and internal envelopes.
        m.desktop_lpg_bypass = s[Lpg] < .5f;
        m.level_patched = s[Lpg] > .5f;
        m.level = gate ? velocity : 0;
        if (s[Lpg] < .5f)
        {
            m.level_patched = false;
        }
        dsp.Render(p, m, frames.data(), 24);
        cursor = 0;
    }
    auto f = frames[cursor++];
    return (f.out * (1 - s[Aux]) + f.aux * s[Aux]) / 32768.f;
}
bool Synth::settings(const Settings &s) noexcept
{
    if (!s.valid())
        return false;
    if (s[Mode] != target_[Mode] || s[Model] != target_[Model])
        reset();
    target_ = s;
    return true;
}
void Synth::tempo(float bpm) noexcept
{
    if (std::isfinite(bpm))
        tempo_ = clamp(bpm, 20, 400);
}
void Synth::start(SynthVoice &v, int ch, int n, float velocity, bool legato) noexcept
{
    bool was = v.active;
    if (!legato)
    {
        v.residual = v.last;
        v.amp = {};
        v.env = {};
    }
    v.active = true;
    v.key = true;
    v.note = n;
    v.channel = ch;
    v.velocity = velocity;
    v.age = ++clock_;
    v.amp.on(legato && was);
    v.env.on(legato && was);
    v.retrigger = was && !legato;
    if (!was || target_[Glide] < .0001f)
        v.pitch = float(n);
    if (!legato)
    {
        v.phase = 0;
        v.ic1[0] = v.ic1[1] = v.ic2[0] = v.ic2[1] = 0;
    }
}
void Synth::noteOn(int ch, int n, float velocity) noexcept
{
    if (ch < 0 || ch > 15 || n < 0 || n > 127 || !std::isfinite(velocity))
        return;
    if (velocity <= 0)
    {
        noteOff(ch, n);
        return;
    }
    velocity = clamp(velocity);
    ++held_[ch][n];
    noteAge_[ch][n] = ++clock_;
    velocity_[ch][n] = velocity;
    if (target_[Mode] < 1.5f)
    {
        start(voices_[0], ch, n, velocity, target_[Mode] > .5f);
        return;
    }
    SynthVoice *selected = &voices_[0];
    for (auto &v : voices_)
    {
        if (!v.active)
        {
            selected = &v;
            break;
        }
        if (v.amp.level < selected->amp.level ||
            (v.amp.level == selected->amp.level && v.age < selected->age))
            selected = &v;
    }
    start(*selected, ch, n, velocity, false);
}
void Synth::noteOff(int ch, int n) noexcept
{
    if (ch < 0 || ch > 15 || n < 0 || n > 127)
        return;
    if (held_[ch][n])
        --held_[ch][n];
    if (target_[Mode] < 1.5f)
    {
        auto &v = voices_[0];
        if (v.note != n || v.channel != ch || held_[ch][n])
            return;
        int last = -1, c = 0;
        uint64_t newest = 0;
        for (int k = 0; k < 16; ++k)
            for (int i = 0; i < 128; ++i)
                if (held_[k][i] && noteAge_[k][i] > newest)
                {
                    newest = noteAge_[k][i];
                    last = i;
                    c = k;
                }
        if (last >= 0)
        {
            start(v, c, last, velocity_[c][last], true);
            return;
        }
        v.key = false;
        if (!sustain_[ch])
        {
            v.amp.off(target_[Release]);
            v.env.off(target_[ModRelease]);
        }
        return;
    }
    SynthVoice *selected = nullptr;
    for (auto &v : voices_)
        if (v.active && v.key && v.note == n && v.channel == ch &&
            (!selected || v.age < selected->age))
            selected = &v;
    if (selected)
    {
        selected->key = false;
        if (!sustain_[ch])
        {
            selected->amp.off(target_[Release]);
            selected->env.off(target_[ModRelease]);
        }
    }
}
void Synth::controller(int ch, int cc, float value) noexcept
{
    if (ch < 0 || ch > 15 || !std::isfinite(value))
        return;
    value = clamp(value);
    if (cc == 1)
        wheels_[ch] = value;
    if (cc == 64)
    {
        sustain_[ch] = value >= .5f;
        if (!sustain_[ch])
            for (auto &v : voices_)
                if (v.active && v.channel == ch && !v.key)
                {
                    v.amp.off(target_[Release]);
                    v.env.off(target_[ModRelease]);
                }
    }
    if (cc == 120 || cc == 123)
    {
        held_[ch].fill(0);
        sustain_[ch] = false;
        for (auto &v : voices_)
            if (v.channel == ch)
            {
                v.key = false;
                v.amp.off(.005f);
                v.env.off(.005f);
            }
    }
    if (cc == 121)
    {
        wheels_[ch] = 0;
        bends_[ch] = 0;
        controller(ch, 64, 0);
    }
}
void Synth::pitchBend(int ch, float normalized) noexcept
{
    if (ch >= 0 && ch < 16 && std::isfinite(normalized))
        bends_[ch] = clamp(normalized, -1, 1);
}
void Synth::reset() noexcept
{
    for (auto &h : held_)
        h.fill(0);
    sustain_.fill(false);
    for (auto &v : voices_)
    {
        v.residual = v.last;
        v.active = false;
        v.key = false;
        v.amp = {};
        v.env = {};
    }
}
int Synth::activeVoices() const noexcept
{
    int n = 0;
    for (auto &v : voices_)
        n += v.active;
    return n;
}
std::array<float, 2> Synth::next() noexcept
{
    // Isolate the upstream PRNG between instances and audio threads.
    auto previous = stmlib::Random::state();
    stmlib::Random::Seed(random_);
    for (size_t i = 0; i < paramCount; ++i)
    {
        bool discrete = i < RouteBase ? baseSpecs[i].discrete : (i - RouteBase) % 3 != 2;
        smooth_[i] = discrete ? target_[i] : smooth_[i] + (target_[i] - smooth_[i]) * .0026f;
    }
    auto &s = smooth_;
    float l[2];
    for (int i = 0; i < 2; ++i)
    {
        auto b = i ? Lfo2Rate : Lfo1Rate;
        float hz = s[b + 1] > .5f ? tempo_ / (60 * s[b]) : s[b];
        lfo_[i] += hz / 48000;
        lfo_[i] -= std::floor(lfo_[i]);
        l[i] = shape(lfo_[i], int(s[b + 2]));
    }
    std::array<float, 2> result{};
    for (auto &v : voices_)
    {
        for (int c = 0; c < 2; ++c)
        {
            v.residual[c] *= .995f;
            if (std::abs(v.residual[c]) < 1.e-8f)
                v.residual[c] = 0;
            result[c] += v.residual[c];
        }
        if (!v.active)
            continue;
        float amp = v.amp.next(s[Attack], s[AmpDecay], s[Sustain]);
        float env = v.env.next(s[ModAttack], s[ModDecay], s[ModSustain]);
        if (!v.amp.stage)
        {
            v.active = false;
            v.last = {};
            continue;
        }
        std::array<float, 8> sources{
            {0, l[0], l[1], env, v.velocity, wheels_[v.channel], (v.note - 60) / 60.f, amp}};
        std::array<float, 11> mod{};
        for (size_t r = 0; r < routeCount; ++r)
        {
            size_t b = RouteBase + 3 * r;
            mod[size_t(s[b + 1])] += sources[size_t(s[b])] * s[b + 2];
        }
        v.pitch += (v.note - v.pitch) * std::min(1.f, 1.f / std::max(1.f, s[Glide] * 48000));
        float pitch = v.pitch + bends_[v.channel] * s[BendRange] + mod[6] * 12;
        bool quantum = v.main.cursor == 24;
        bool gate = v.key || sustain_[v.channel];
        float main = v.main.next(s, pitch, v.velocity, gate, v.retrigger, mod);
        float layer = v.layer.next(s, pitch + (s[Detune] + mod[8] * 60) / 100, v.velocity, gate,
                                   v.retrigger, mod);
        if (quantum)
            v.retrigger = false;
        float blend = clamp(s[Layer] + mod[7]), width = clamp(s[Width] + mod[9]);
        float mid = main * (1 - blend * .5f) + layer * blend * .5f,
              side = (main - layer) * blend * .5f * width;
        // Remove low-frequency difference signal without touching the centered sub.
        v.lowSide += (side - v.lowSide) * .015586f;
        if (std::abs(v.lowSide) < 1.e-20f)
            v.lowSide = 0;
        if (s[LowMono] > .5f)
            side -= v.lowSide;
        v.phase += 440 * std::exp2((pitch + 12 * s[SubOctave] - 69) / 12) / 48000;
        v.phase -= std::floor(v.phase);
        float sub =
            s[SubShape] > .5f ? (1 - 4 * std::abs(v.phase - .5f)) : std::sin(2 * pi * v.phase);
        float cutoff = clamp(s[Cutoff] * std::exp2(mod[3] * 4 + s[FilterEnv] * env +
                                                   s[Keytrack] * (v.note - 60) / 12.f),
                             25, 18000);
        float g = std::tan(pi * cutoff / 48000), k = 2 - 1.9f * s[Resonance],
              a1 = 1 / (1 + g * (g + k));
        for (int c = 0; c < 2; ++c)
        {
            float x = mid + (c ? -side : side);
            float hp = (x - (k + g) * v.ic1[c] - v.ic2[c]) * a1, bp = g * hp + v.ic1[c],
                  lp = g * bp + v.ic2[c];
            v.ic1[c] = g * hp + bp;
            v.ic2[c] = g * bp + lp;
            if (std::abs(v.ic1[c]) < 1.e-20f)
                v.ic1[c] = 0;
            if (std::abs(v.ic2[c]) < 1.e-20f)
                v.ic2[c] = 0;
            int mode = int(s[FilterMode]);
            if (mode)
                x = mode == 1 ? lp : mode == 2 ? hp : bp;
            float drive = clamp(s[Drive] + mod[4]);
            int dm = int(s[DriveMode]);
            // Two interpolated evaluations reduce the worst discontinuities. No claim
            // of alias-free hard clipping/folding at extreme high-frequency settings.
            if (dm)
            {
                float wet = .5f * (distort((x + v.drivePrevious[c]) * .5f, dm, drive) +
                                   distort(x, dm, drive));
                v.drivePrevious[c] = x;
                x = wet / (1 + s[Compensation] * drive * 3);
            }
            x += sub * clamp(s[Sub] + mod[10]) * .5f;
            v.last[c] = x * amp * v.velocity * clamp(1 + mod[5], 0, 2) * .6f;
            result[c] += v.last[c];
        }
    }
    float gain = std::pow(10.f, s[Output] / 20);
    for (int c = 0; c < 2; ++c)
    {
        float x = result[c] * gain;
        float y = x - dcInput_[c] + .997385f * dcOutput_[c];
        dcInput_[c] = x;
        dcOutput_[c] = std::abs(y) < 1.e-20f ? 0 : y;
        result[c] = std::isfinite(y) ? y : 0;
    }
    random_ = stmlib::Random::state();
    stmlib::Random::Seed(previous);
    return result;
}
void DesktopEngine::prepare(double rate)
{
    if (!std::isfinite(rate) || rate < 32000 || rate > 192000)
        throw std::invalid_argument("Sample rate must be 32000..192000 Hz");
    ratio_ = 48000 / rate;
    position_ = 0;
    generated_ = -1;
    history_ = {};
    synth_.reset();
    double cutoff = std::min(1., rate / 48000) * .94;
    for (int p = 0; p < 256; ++p)
    {
        double sum = 0;
        for (int j = 0; j < 64; ++j)
        {
            double t = j - 32. + p / 256.;
            double sinc = std::abs(t) < 1.e-10
                              ? cutoff
                              : std::sin(3.141592653589793 * t * cutoff) / (3.141592653589793 * t);
            double window = .5 + .5 * std::cos(3.141592653589793 * t / 33);
            kernel_[p][j] = float(sinc * window);
            sum += kernel_[p][j];
        }
        for (auto &x : kernel_[p])
            x = float(x / sum);
    }
}
std::array<float, 2> DesktopEngine::next() noexcept
{
    auto index = int64_t(position_);
    while (generated_ < index)
    {
        ++generated_;
        history_[size_t(generated_) & 127] = synth_.next();
    }
    int phase = std::min(255, int((position_ - double(index)) * 256));
    std::array<float, 2> out{};
    for (int j = 0; j < 64; ++j)
    {
        auto &h = history_[size_t(index - j) & 127];
        for (int c = 0; c < 2; ++c)
            out[c] += h[c] * kernel_[phase][j];
    }
    position_ += ratio_;
    return out;
}
} // namespace soundgyal
