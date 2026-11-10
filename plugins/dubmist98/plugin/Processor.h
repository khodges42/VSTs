// SPDX-License-Identifier: GPL-3.0-or-later
#pragma once
#include <juce_audio_utils/juce_audio_utils.h>
#include "engine.h"
#include <limits>
class DubmistProcessor final : public juce::AudioProcessor
{
    std::unique_ptr<dubmist::DesktopEngine> engine_;
    std::array<std::atomic<float> *, dubmist::Count> raw_{};
    std::atomic<unsigned> generation_{0};
    dubmist::Settings last_;
    static juce::AudioProcessorValueTreeState::ParameterLayout layout();
    void process(juce::AudioBuffer<float> &, bool);

  public:
    juce::AudioProcessorValueTreeState parameters;
    const std::array<dubmist::Preset, 12> presets = dubmist::factory();
    std::atomic<bool> rhythmActive{false}, clockAvailable{false};
    std::atomic<float> inputPeak{0}, outputPeak{0};
    DubmistProcessor();
    dubmist::Settings snapshot() const noexcept;
    void apply(const dubmist::Settings &);
    juce::String exportPatch();
    bool importPatch(const void *, size_t);
    void prepareToPlay(double, int) override;
    void releaseResources() override {}
    void processBlock(juce::AudioBuffer<float> &b, juce::MidiBuffer &m) override
    {
        process(b, false);
        m.clear();
    }
    void processBlockBypassed(juce::AudioBuffer<float> &b, juce::MidiBuffer &m) override
    {
        process(b, true);
        m.clear();
    }
    juce::AudioProcessorParameter *getBypassParameter() const override
    {
        return parameters.getParameter("bypass");
    }
    bool isBusesLayoutSupported(const BusesLayout &b) const override
    {
        return (b.getMainInputChannelSet() == juce::AudioChannelSet::stereo() ||
                b.getMainInputChannelSet() == juce::AudioChannelSet::mono()) &&
               b.getMainOutputChannelSet() == juce::AudioChannelSet::stereo();
    }
    const juce::String getName() const override
    {
        return "Dubmist 98";
    }
    bool acceptsMidi() const override
    {
        return false;
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
        return std::numeric_limits<double>::infinity();
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
        return "Processing configuration";
    }
    void changeProgramName(int, const juce::String &) override {}
    void getStateInformation(juce::MemoryBlock &) override;
    void setStateInformation(const void *, int) override;
};
