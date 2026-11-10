// SPDX-License-Identifier: GPL-3.0-or-later
#pragma once
#include "parameters.h"
namespace soundgyal
{
struct Factory
{
    const char *name;
    const char *category;
    Settings patch;
};
inline constexpr size_t factoryCount = 35;
inline std::array<Factory, factoryCount> makeFactory()
{
    std::array<Factory, factoryCount> bank{};
    auto set =
        [&](int i, const char *name, const char *category, int engine, float h, float t, float m)
    {
        auto &f = bank[i];
        f.name = name;
        f.category = category;
        f.patch[Model] = float(engine);
        f.patch[Harmonics] = h;
        f.patch[Timbre] = t;
        f.patch[Morph] = m;
        f.patch[Lpg] = 0;
    };
    set(0, "Clean Rolling Sub", "JUNGLE", 8, 0, .1f, 0);
    set(1, "Jungle Hoover", "JUNGLE", 8, .505f, .64f, .8f);
    set(2, "Dark Reese", "JUNGLE", 8, .45f, .2f, .55f);
    set(3, "Short Rave Stab", "JUNGLE", 14, .3f, .6f, .45f);
    set(4, "Dirty Break Accent", "JUNGLE", 18, .7f, .8f, .3f);
    set(5, "Wide Reese", "DRUM & BASS", 8, .65f, .35f, .6f);
    set(6, "Neuro Movement", "DRUM & BASS", 9, .55f, .5f, .4f);
    set(7, "FM Snarl", "DRUM & BASS", 10, .3f, .7f, .6f);
    set(8, "Metallic Roller", "DRUM & BASS", 20, .6f, .6f, .4f);
    set(9, "Deep Sub Pressure", "DRUM & BASS", 12, 0, 0, 0);
    set(10, "Detuned Mid Bass", "DRUM & BASS", 6, .6f, .7f, .4f);
    set(11, "Hollow Bass", "UK GARAGE", 1, .3f, .2f, .15f);
    set(12, "Bouncy Organ Bass", "UK GARAGE", 12, .25f, .25f, .1f);
    set(13, "Short 2-Step Stab", "UK GARAGE", 14, .5f, .3f, .2f);
    set(14, "Rubber Bass", "UK GARAGE", 0, .3f, .6f, .5f);
    set(15, "Subby Pluck", "UK GARAGE", 19, .2f, .25f, .6f);
    set(16, "Heavy Sine", "140", 10, 0, 0, 0);
    set(17, "Dark Wobble", "140", 0, .5f, .4f, .5f);
    set(18, "FM Growl", "140", 2, .25f, .8f, .7f);
    set(19, "Deep Atmospheric Texture", "140", 5, .3f, .3f, .7f);
    set(20, "Digital Kick", "PERCUSSION", 21, .4f, .6f, .25f);
    set(21, "Snare Destruction", "PERCUSSION", 22, .5f, .7f, .65f);
    set(22, "Metallic Percussion", "PERCUSSION", 23, .7f, .7f, .3f);
    set(23, "Folded Stab", "BREAKCORE", 7, .7f, .45f, .55f);
    set(24, "Unstable Digital Noise", "BREAKCORE", 17, .8f, .65f, .85f);
    set(25, "Dred Reverse Bass", "UK GARAGE", 8, .505f, .72f, .2f);
    set(26, "FM Donk", "UK GARAGE", 10, .22f, .32f, .5f);
    set(27, "Liquid Glass Keys", "DRUM & BASS", 12, .22f, .3f, .2f);
    set(28, "Formant Roller", "DRUM & BASS", 11, .35f, .48f, .38f);
    set(29, "Classic Wobble", "DUBSTEP", 8, .505f, .75f, .4f);
    set(30, "Riddim Square", "DUBSTEP", 1, .35f, .6f, .25f);
    set(31, "Vowel Growl", "DUBSTEP", 11, .62f, .5f, .55f);
    set(32, "Metallic Yoi", "DUBSTEP", 10, .58f, .55f, .68f);
    set(33, "Deep Dub Pluck", "DUBSTEP", 0, .25f, .3f, .55f);
    set(34, "Rave Fifth Stab", "JUNGLE", 14, .12f, .7f, .3f);
    auto route = [&](int i, int r, int source, int target, float depth)
    {
        auto &p = bank[i].patch;
        p[RouteBase + r * 3] = float(source);
        p[RouteBase + r * 3 + 1] = float(target);
        p[RouteBase + r * 3 + 2] = depth;
    };
    for (int i = 0; i < int(factoryCount); ++i)
    {
        auto &p = bank[i].patch;
        p[Output] = -12;
        p[Sub] = .18f;
        p[FilterMode] = 1;
        p[Cutoff] = 3500;
        route(i, 7, 5, 3, .45f);
    }
    for (int i : {0, 9, 16})
    {
        auto &p = bank[i].patch;
        p[Sub] = .85f;
        p[Cutoff] = 180;
        p[Attack] = .008f;
        p[Release] = .18f;
        p[Output] = -8;
        p[SubOctave] = 0;
    }
    for (int i : {1, 2, 5, 10})
    {
        auto &p = bank[i].patch;
        p[Layer] = .85f;
        p[Detune] = i == 2 ? 9.f : 22.f;
        p[Width] = i == 5 ? .9f : .35f;
        p[Cutoff] = i == 2 ? 900.f : 4300.f;
        route(i, 0, 2, 1, .12f);
    }
    for (int i : {3, 4, 8, 13, 15, 20, 21, 22, 23})
    {
        auto &p = bank[i].patch;
        p[Sustain] = 0;
        p[AmpDecay] = i >= 20 ? .22f : .16f;
        p[Release] = .08f;
        p[ModDecay] = .12f;
        p[FilterEnv] = 2.2f;
        p[Lpg] = 1;
        p[Decay] = .35f;
    }
    for (int i : {6, 7, 18})
    {
        auto &p = bank[i].patch;
        p[DriveMode] = 1;
        p[Drive] = .35f;
        p[Sub] = .35f;
        route(i, 0, 1, 2, .32f);
        route(i, 1, 2, 1, .2f);
        p[Lfo1Rate] = .5f;
        p[Lfo2Rate] = 2;
    }
    for (int i : {11, 12, 14})
    {
        auto &p = bank[i].patch;
        p[Sustain] = .25f;
        p[AmpDecay] = .15f;
        p[ModDecay] = .12f;
        p[FilterEnv] = 3;
        p[Cutoff] = 380;
        p[Sub] = .4f;
        p[Resonance] = .45f;
    }
    bank[17].patch[Cutoff] = 450;
    bank[17].patch[Resonance] = .6f;
    route(17, 0, 1, 3, .8f);
    bank[17].patch[Lfo1Rate] = .5f;
    auto &texture = bank[19].patch;
    texture[Mode] = 2;
    texture[Attack] = 1.2f;
    texture[Release] = 2.5f;
    texture[Layer] = .7f;
    texture[Width] = .8f;
    texture[Sub] = 0;
    route(19, 0, 2, 0, .35f);
    for (int i : {4, 21, 23, 24})
    {
        auto &p = bank[i].patch;
        p[DriveMode] = i == 23 ? 3.f : 2.f;
        p[Drive] = .5f;
        p[Cutoff] = 7000;
        p[Sub] = 0;
    }
    bank[22].patch[FilterMode] = 2;
    bank[22].patch[Cutoff] = 4000;
    bank[22].patch[Sub] = 0;
    bank[24].patch[FilterMode] = 3;
    bank[24].patch[Resonance] = .6f;
    route(24, 0, 1, 0, .45f);
    route(24, 1, 2, 3, .7f);
    // Pitched PWM-style hoover: do not use granular Swarm as a substitute for
    // stable oscillators. Preserve the original engine for all other patches.
    auto &hoover = bank[1].patch;
    hoover[Layer] = .72f;
    hoover[Detune] = 13;
    hoover[Width] = .65f;
    hoover[Cutoff] = 7200;
    hoover[Sub] = .12f;
    hoover[Attack] = .004f;
    hoover[Release] = .22f;
    hoover[ModAttack] = .001f;
    hoover[ModDecay] = .22f;
    hoover[Lfo1Sync] = 0;
    hoover[Lfo1Rate] = .7f;
    hoover[Lfo2Sync] = 0;
    hoover[Lfo2Rate] = .19f;
    route(1, 0, 1, 2, .10f);
    route(1, 1, 2, 1, .055f);
    route(1, 2, 3, 6, .65f);

    auto &dred = bank[25].patch;
    dred[Cutoff] = 140;
    dred[FilterEnv] = 4.5f;
    dred[ModAttack] = .17f;
    dred[ModDecay] = .08f;
    dred[ModSustain] = .8f;
    dred[Resonance] = .35f;
    dred[Sub] = .42f;
    dred[Sustain] = .55f;
    dred[AmpDecay] = .3f;
    dred[Layer] = .35f;
    dred[Detune] = 7;
    dred[Glide] = .025f;

    auto &donk = bank[26].patch;
    donk[Cutoff] = 2200;
    donk[Sustain] = 0;
    donk[AmpDecay] = .23f;
    donk[ModDecay] = .08f;
    donk[Sub] = .32f;
    donk[SubOctave] = 0;
    donk[Release] = .065f;
    route(26, 0, 3, 1, .28f);

    auto &glass = bank[27].patch;
    glass[Mode] = 2;
    glass[Sub] = 0;
    glass[Cutoff] = 6500;
    glass[AmpDecay] = 1.1f;
    glass[Sustain] = .12f;
    glass[Release] = .8f;
    glass[ModDecay] = .5f;
    glass[Layer] = .3f;
    glass[Detune] = 5;
    glass[Width] = .65f;
    route(27, 0, 3, 0, .22f);

    auto &roller = bank[28].patch;
    roller[Sub] = .4f;
    roller[Cutoff] = 4200;
    roller[DriveMode] = 1;
    roller[Drive] = .18f;
    roller[Lfo1Rate] = 1;
    roller[Lfo2Rate] = 4;
    route(28, 0, 1, 1, .2f);
    route(28, 1, 2, 2, .12f);

    auto &wobble = bank[29].patch;
    wobble[Cutoff] = 420;
    wobble[Resonance] = .52f;
    wobble[Sub] = .55f;
    wobble[Lfo1Rate] = .5f;
    wobble[Lfo1Shape] = 1;
    wobble[DriveMode] = 1;
    wobble[Drive] = .12f;
    route(29, 0, 1, 3, .75f);

    auto &riddim = bank[30].patch;
    riddim[Cutoff] = 2400;
    riddim[Resonance] = .32f;
    riddim[DriveMode] = 2;
    riddim[Drive] = .22f;
    riddim[Sub] = .48f;
    riddim[Lfo1Rate] = .5f;
    riddim[Lfo1Shape] = 3;
    route(30, 0, 1, 5, .85f);
    route(30, 1, 1, 2, .17f);

    auto &growl = bank[31].patch;
    growl[Cutoff] = 5500;
    growl[DriveMode] = 1;
    growl[Drive] = .24f;
    growl[Sub] = .48f;
    growl[Lfo1Rate] = 1;
    growl[Lfo2Rate] = 2;
    route(31, 0, 1, 1, .3f);
    route(31, 1, 2, 2, -.23f);
    route(31, 2, 3, 0, .2f);
    growl[ModDecay] = .38f;

    auto &yoi = bank[32].patch;
    yoi[Cutoff] = 4400;
    yoi[Sub] = .36f;
    yoi[DriveMode] = 3;
    yoi[Drive] = .16f;
    yoi[ModDecay] = .26f;
    yoi[Lfo1Rate] = .25f;
    route(32, 0, 3, 1, -.4f);
    route(32, 1, 1, 2, .18f);
    yoi[Sustain] = .35f;
    yoi[AmpDecay] = .32f;

    auto &pluck = bank[33].patch;
    pluck[Cutoff] = 180;
    pluck[FilterEnv] = 3.2f;
    pluck[Resonance] = .4f;
    pluck[Sub] = .6f;
    pluck[SubOctave] = 0;
    pluck[Sustain] = 0;
    pluck[AmpDecay] = .65f;
    pluck[ModDecay] = .18f;
    pluck[Release] = .14f;

    auto &stab = bank[34].patch;
    stab[Mode] = 2;
    stab[Sub] = 0;
    stab[Cutoff] = 2100;
    stab[FilterEnv] = 2;
    stab[ModDecay] = .12f;
    stab[Sustain] = 0;
    stab[AmpDecay] = .28f;
    stab[Release] = .09f;
    stab[Layer] = .45f;
    stab[Detune] = 8;
    stab[Width] = .6f;
    return bank;
}
} // namespace soundgyal
