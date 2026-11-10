// SPDX-License-Identifier: GPL-3.0-or-later
#pragma once
#include "Processor.h"
class CorporateLook final : public juce::LookAndFeel_V4
{
  public:
    CorporateLook();
    void drawButtonBackground(juce::Graphics &, juce::Button &, const juce::Colour &, bool,
                              bool) override;
    void drawLinearSlider(juce::Graphics &, int, int, int, int, float, float, float,
                          const juce::Slider::SliderStyle, juce::Slider &) override;
};
class PropertyPage final : public juce::Component
{
    SoundgyalProcessor &processor;
    std::vector<std::unique_ptr<juce::Label>> labels;
    std::vector<std::unique_ptr<juce::Slider>> sliders;
    std::vector<std::unique_ptr<juce::ComboBox>> choices;
    std::vector<std::unique_ptr<juce::AudioProcessorValueTreeState::SliderAttachment>>
        sliderAttachments;
    std::vector<std::unique_ptr<juce::AudioProcessorValueTreeState::ComboBoxAttachment>>
        choiceAttachments;
    std::vector<juce::Component *> rows;
    std::vector<juce::String> headings;
    int columns;

  public:
    PropertyPage(SoundgyalProcessor &, std::vector<size_t>, std::vector<juce::String>, int);
    void paint(juce::Graphics &) override;
    void resized() override;
};
class SoundgyalEditor final : public juce::AudioProcessorEditor,
                              private juce::Timer,
                              private juce::MidiKeyboardStateListener
{
    SoundgyalProcessor &processor;
    CorporateLook look;
    juce::TabbedComponent tabs{juce::TabbedButtonBar::TabsAtTop};
    juce::ComboBox preset, category;
    juce::TextButton previous{"<"}, next{">"}, init{"Initialize"}, save{"Save..."}, load{"Load..."},
        about{"About..."};
    juce::Label description, status;
    juce::MidiKeyboardState keyboardState;
    juce::MidiKeyboardComponent keyboard{keyboardState,
                                         juce::MidiKeyboardComponent::horizontalKeyboard};
    juce::TooltipWindow tooltip{this, 600};
    std::unique_ptr<juce::FileChooser> chooser;
    float meter = 0;
    int selected = -1;
    void timerCallback() override;
    void handleNoteOn(juce::MidiKeyboardState *, int, int, float) override;
    void handleNoteOff(juce::MidiKeyboardState *, int, int, float) override;
    void browse(int);
    void fillPresets();

  public:
    explicit SoundgyalEditor(SoundgyalProcessor &);
    ~SoundgyalEditor() override;
    void paint(juce::Graphics &) override;
    void resized() override;
};
