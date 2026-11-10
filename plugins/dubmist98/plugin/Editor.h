// SPDX-License-Identifier: GPL-3.0-or-later
#pragma once
#include "Processor.h"
class WorkgroupLook : public juce::LookAndFeel_V4
{
  public:
    WorkgroupLook();
    void drawLinearSlider(juce::Graphics &, int, int, int, int, float, float, float,
                          const juce::Slider::SliderStyle, juce::Slider &) override;
    void drawButtonBackground(juce::Graphics &, juce::Button &, const juce::Colour &, bool,
                              bool) override;
};
class DubmistEditor final : public juce::AudioProcessorEditor, private juce::Timer
{
    DubmistProcessor &processor_;
    WorkgroupLook look_;
    juce::ComboBox preset_, mode_, quality_;
    juce::TextButton save_{"Save..."}, load_{"Load..."}, init_{"Initialize"}, about_{"About..."};
    std::array<juce::Slider, 11> sliders_;
    std::array<juce::Label, 11> labels_;
    juce::TextButton cloudsView_{"Clouds"}, rhythmView_{"Rhythm"};
    juce::ComboBox division_;
    juce::Label divisionLabel_;
    std::array<juce::Slider, 6> rhythmSliders_;
    std::array<juce::Label, 6> rhythmLabels_;
    juce::ToggleButton sync_{"HOST SYNC"}, repeat_{"REPEAT / HOLD"}, reverse_{"REVERSE"},
        alternate_{"ALTERNATE"}, quantize_{"QUANTIZE START"}, restartBar_{"RESTART ON BAR"};
    bool rhythmVisible_ = false;
    void showRhythm(bool);
    std::vector<std::unique_ptr<juce::AudioProcessorValueTreeState::SliderAttachment>>
        slidersAttached_;
    std::vector<std::unique_ptr<juce::AudioProcessorValueTreeState::ComboBoxAttachment>>
        combosAttached_;
    juce::ToggleButton freeze_{"FREEZE"}, bypass_{"BYPASS"};
    juce::TextButton clear_{"CLEAR BUFFER"}, trigger_{"TRIGGER GRAIN"};
    std::vector<std::unique_ptr<juce::AudioProcessorValueTreeState::ButtonAttachment>>
        buttonsAttached_;
    juce::Label status_, description_;
    juce::TooltipWindow tooltip_{this, 600};
    std::unique_ptr<juce::FileChooser> chooser_;
    float in_ = 0, out_ = 0;
    int selected_ = -1;
    void timerCallback() override;

  public:
    explicit DubmistEditor(DubmistProcessor &);
    ~DubmistEditor() override;
    void paint(juce::Graphics &) override;
    void resized() override;
};
