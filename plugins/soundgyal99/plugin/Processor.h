// SPDX-License-Identifier: GPL-3.0-or-later
#pragma once
#include <juce_audio_utils/juce_audio_utils.h>
#include "synth.h"
#include "factory.h"
class SoundgyalProcessor final : public juce::AudioProcessor
{
    std::unique_ptr<soundgyal::DesktopEngine> engine;
    std::array<std::atomic<float> *, soundgyal::paramCount> raw{};
    soundgyal::Settings last;
    std::atomic<unsigned> editVersion{0};
    struct Audition
    {
        int note;
        float velocity;
    };
    std::array<Audition, 64> auditionQueue{};
    std::atomic<unsigned> read{0}, write{0};
    std::atomic<bool> auditionOverflow{false};
    static juce::AudioProcessorValueTreeState::ParameterLayout layout();

  public:
    juce::AudioProcessorValueTreeState parameters;
    const std::array<soundgyal::Factory, soundgyal::factoryCount> factory =
        soundgyal::makeFactory();
    std::atomic<float> peak{0};
    std::atomic<int> voiceCount{0}, midiLight{0};
    SoundgyalProcessor();
    static juce::String parameterID(size_t);
    soundgyal::Settings snapshot() const noexcept;
    void apply(const soundgyal::Settings &);
    void audition(int note, float velocity) noexcept
    {
        unsigned w = write.load(std::memory_order_relaxed);
        if (w - read.load(std::memory_order_acquire) >= 64)
        {
            auditionOverflow.store(true, std::memory_order_release);
            return;
        }
        auditionQueue[w % 64] = {note, velocity};
        write.store(w + 1, std::memory_order_release);
    }
    juce::String exportPatch();
    bool importPatch(const void *, size_t);
    void prepareToPlay(double, int) override;
    void releaseResources() override {}
    void processBlock(juce::AudioBuffer<float> &, juce::MidiBuffer &) override;
    bool isBusesLayoutSupported(const BusesLayout &) const override;
    const juce::String getName() const override
    {
        return "SOUNDGYAL 99";
    }
    bool acceptsMidi() const override
    {
        return true;
    }
    bool producesMidi() const override
    {
        return false;
    }
    bool isMidiEffect() const override
    {
        return false;
    }
    double getTailLengthSeconds() const override
    {
        return 8;
    }
    bool hasEditor() const override
    {
        return true;
    }
    juce::AudioProcessorEditor *createEditor() override;
    int getNumPrograms() override
    {
        return 1;
    }
    int getCurrentProgram() override
    {
        return 0;
    }
    void setCurrentProgram(int) override {}
    const juce::String getProgramName(int) override
    {
        return "Patch Configuration";
    }
    void changeProgramName(int, const juce::String &) override {}
    void getStateInformation(juce::MemoryBlock &) override;
    void setStateInformation(const void *, int) override;
};
