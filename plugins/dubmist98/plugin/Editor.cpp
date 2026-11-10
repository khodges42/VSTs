// SPDX-License-Identifier: GPL-3.0-or-later
#include "Editor.h"
using namespace dubmist;
namespace
{
auto panel = juce::Colour(0xffc8c8bd), ink = juce::Colour(0xff16191b),
     navy = juce::Colour(0xff152e50);
constexpr std::array<size_t, 11> sliderIDs{{
    Position,
    Size,
    Density,
    Texture,
    Pitch,
    Mix,
    Spread,
    Feedback,
    Reverb,
    InputGain,
    OutputGain,
}};
} // namespace
WorkgroupLook::WorkgroupLook()
{
    setColour(juce::Label::textColourId, ink);
    setColour(juce::Slider::textBoxTextColourId, ink);
    setColour(juce::Slider::textBoxBackgroundColourId, juce::Colour(0xfff0f0e8));
    setColour(juce::Slider::thumbColourId, navy);
    setColour(juce::Slider::trackColourId, juce::Colours::darkgrey);
    setColour(juce::ComboBox::backgroundColourId, juce::Colour(0xfff0f0e8));
    setColour(juce::ComboBox::textColourId, ink);
    setColour(juce::ComboBox::arrowColourId, ink);
    setColour(juce::PopupMenu::backgroundColourId, panel);
    setColour(juce::PopupMenu::textColourId, ink);
    setColour(juce::TextButton::textColourOffId, ink);
    setColour(juce::ToggleButton::textColourId, ink);
    setColour(juce::ToggleButton::tickColourId, navy);
}
void WorkgroupLook::drawButtonBackground(juce::Graphics &g, juce::Button &b, const juce::Colour &,
                                         bool over, bool down)
{
    g.fillAll(over ? panel.brighter(.1f) : panel);
    auto r = b.getLocalBounds().toFloat().reduced(.5f);
    g.setColour(down ? juce::Colours::darkgrey : juce::Colours::white);
    g.drawLine(r.getX(), r.getY(), r.getRight(), r.getY());
    g.drawLine(r.getX(), r.getY(), r.getX(), r.getBottom());
    g.setColour(down ? juce::Colours::white : juce::Colours::darkgrey);
    g.drawLine(r.getX(), r.getBottom(), r.getRight(), r.getBottom());
    g.drawLine(r.getRight(), r.getY(), r.getRight(), r.getBottom());
}
void WorkgroupLook::drawLinearSlider(juce::Graphics &g, int x, int y, int w, int h, float pos,
                                     float, float, const juce::Slider::SliderStyle, juce::Slider &)
{
    int mid = y + h / 2;
    g.setColour(juce::Colours::darkgrey);
    g.fillRect(x, mid - 2, w, 4);
    g.setColour(juce::Colours::white);
    g.drawHorizontalLine(mid + 2, float(x), float(x + w));
    juce::Rectangle<int> thumb(int(pos) - 5, mid - 8, 10, 16);
    g.setColour(panel);
    g.fillRect(thumb);
    g.setColour(juce::Colours::white);
    g.drawRect(thumb);
    g.setColour(ink);
    g.drawLine(float(thumb.getRight()), float(thumb.getY()), float(thumb.getRight()),
               float(thumb.getBottom()));
}
DubmistEditor::DubmistEditor(DubmistProcessor &p) : AudioProcessorEditor(p), processor_(p)
{
    setLookAndFeel(&look_);
    setSize(960, 610);
    setResizable(false, false);
    for (auto *c : std::array<juce::Component *, 14>{{&preset_, &mode_, &quality_, &save_, &load_,
                                                      &init_, &about_, &freeze_, &bypass_, &clear_,
                                                      &trigger_, &status_, &description_, nullptr}})
        if (c)
            addAndMakeVisible(c);
    for (size_t i = 0; i < p.presets.size(); ++i)
        preset_.addItem(p.presets[i].name, int(i + 1));
    preset_.setText("Current processing configuration", juce::dontSendNotification);
    preset_.onChange = [this]
    {
        int i = preset_.getSelectedId() - 1;
        if (i >= 0)
        {
            selected_ = i;
            processor_.apply(processor_.presets[size_t(i)].settings);
        }
    };
    for (int i = 0; i < int(modes.size()); ++i)
        mode_.addItem(modes[size_t(i)], i + 1);
    quality_.addItemList({"32 kHz / 16-bit stereo", "32 kHz / 16-bit mono",
                          "16 kHz / mu-law stereo", "16 kHz / mu-law mono"},
                         1);
    combosAttached_.push_back(
        std::make_unique<juce::AudioProcessorValueTreeState::ComboBoxAttachment>(
            p.parameters, "quality", quality_));
    mode_.onChange = [this]
    {
        int index = mode_.getSelectedId() - 1;
        if (index < 0)
            return;
        auto set = [this](const char *id, float v)
        {
            auto *p = processor_.parameters.getParameter(id);
            p->beginChangeGesture();
            p->setValueNotifyingHost(p->convertTo0to1(v));
            p->endChangeGesture();
        };
        if (index < 4)
            set("mode", float(index));
        set("rhythm_enabled", index == 4 ? 1.f : 0.f);
        showRhythm(index == 4);
    };
    addAndMakeVisible(cloudsView_);
    addAndMakeVisible(rhythmView_);
    cloudsView_.onClick = [this] { showRhythm(false); };
    rhythmView_.onClick = [this] { showRhythm(true); };
    addAndMakeVisible(division_);
    addAndMakeVisible(divisionLabel_);
    divisionLabel_.setText("Repeat division", juce::dontSendNotification);
    for (int i = 0; i < 10; ++i)
        division_.addItem(divisionNames[size_t(i)], i + 1);
    combosAttached_.push_back(
        std::make_unique<juce::AudioProcessorValueTreeState::ComboBoxAttachment>(
            p.parameters, "division", division_));
    for (auto pair : std::array<std::pair<const char *, juce::ToggleButton *>, 6>{
             {{"sync", &sync_},
              {"repeat", &repeat_},
              {"reverse", &reverse_},
              {"alternate", &alternate_},
              {"quantize", &quantize_},
              {"restart_bar", &restartBar_}}})
    {
        addAndMakeVisible(*pair.second);
        buttonsAttached_.push_back(
            std::make_unique<juce::AudioProcessorValueTreeState::ButtonAttachment>(
                p.parameters, pair.first, *pair.second));
    }
    constexpr size_t rhythmIds[] = {Probability, Slice, Step, TapePitch, TapeRamp, FreeTempo};
    for (size_t i = 0; i < 6; ++i)
    {
        auto id = rhythmIds[i];
        auto &slider = rhythmSliders_[i];
        addAndMakeVisible(slider);
        addAndMakeVisible(rhythmLabels_[i]);
        rhythmLabels_[i].setText(specs[id].name, juce::dontSendNotification);
        rhythmLabels_[i].setFont(juce::Font(12.f));
        slider.setSliderStyle(juce::Slider::LinearHorizontal);
        slider.setTextBoxStyle(juce::Slider::TextBoxRight, false, 68, 22);
        slider.setDoubleClickReturnValue(true, specs[id].initial);
        slider.setVelocityModeParameters(.2, 1, .02, true, juce::ModifierKeys::shiftModifier);
        slider.setTooltip(juce::String(specs[id].name) +
                          ". Tape speed changes pitch and duration together.");
        slidersAttached_.push_back(
            std::make_unique<juce::AudioProcessorValueTreeState::SliderAttachment>(
                p.parameters, specs[id].id, slider));
        slider.textFromValueFunction = [id](double v)
        { return juce::String(v, specs[id].discrete ? 0 : 2); };
        slider.updateText();
    }
    // The fifth row includes pitch; all four blend functions remain independent.
    for (size_t i = 0; i < 11; ++i)
    {
        auto id = sliderIDs[i];
        auto &slider = sliders_[i];
        addAndMakeVisible(slider);
        addAndMakeVisible(labels_[i]);
        labels_[i].setText(specs[id].name, juce::dontSendNotification);
        labels_[i].setFont(juce::Font(12.f));
        slider.setSliderStyle(juce::Slider::LinearHorizontal);
        slider.setTextBoxStyle(juce::Slider::TextBoxRight, false, 68, 22);
        slider.setDoubleClickReturnValue(true, specs[id].initial);
        slider.setVelocityModeParameters(.2, 1, .02, true, juce::ModifierKeys::shiftModifier);
        slider.setTooltip(juce::String(specs[id].name) +
                          ": drag, Shift-fine, double-click default, or type a value.");
        slidersAttached_.push_back(
            std::make_unique<juce::AudioProcessorValueTreeState::SliderAttachment>(
                p.parameters, specs[id].id, slider));
        slider.textFromValueFunction = [id](double v)
        { return juce::String(v, id == Pitch || id == InputGain || id == OutputGain ? 1 : 3); };
        slider.updateText();
    }
    buttonsAttached_.push_back(
        std::make_unique<juce::AudioProcessorValueTreeState::ButtonAttachment>(p.parameters,
                                                                               "freeze", freeze_));
    buttonsAttached_.push_back(
        std::make_unique<juce::AudioProcessorValueTreeState::ButtonAttachment>(p.parameters,
                                                                               "bypass", bypass_));
    auto momentary = [&](juce::TextButton &button, const char *id)
    {
        button.onStateChange = [this, &button, id]
        {
            auto *param = processor_.parameters.getParameter(id);
            param->setValueNotifyingHost(button.isDown() ? 1.f : 0.f);
        };
    };
    momentary(clear_, "clear");
    momentary(trigger_, "trigger");
    quantize_.setTooltip("Quantize capture and retrigger starts. Release fades immediately.");
    freeze_.setTooltip(
        "Stop writing live audio. Latched; automate Freeze for momentary host actions.");
    clear_.setTooltip("Clear recorded audio and FX after a short fade. Held button fires once.");
    trigger_.setTooltip("Immediate grain/loop trigger; not tempo-quantized. Density near center "
                        "permits manually triggered grains.");
    init_.onClick = [this]
    {
        selected_ = -1;
        processor_.apply(Settings{});
        preset_.setText("Initialized configuration", juce::dontSendNotification);
    };
    save_.onClick = [this]
    {
        chooser_ = std::make_unique<juce::FileChooser>(
            "Save Dubmist configuration",
            juce::File::getSpecialLocation(juce::File::userDocumentsDirectory)
                .getChildFile("Untitled.dm98.xml"),
            "*.dm98.xml");
        chooser_->launchAsync(
            juce::FileBrowserComponent::saveMode | juce::FileBrowserComponent::canSelectFiles |
                juce::FileBrowserComponent::warnAboutOverwriting,
            [safe = juce::Component::SafePointer<DubmistEditor>(this)](const juce::FileChooser &fc)
            {
                if (safe && fc.getResult() != juce::File())
                    if (!fc.getResult().replaceWithText(safe->processor_.exportPatch()))
                        juce::AlertWindow::showMessageBoxAsync(
                            juce::MessageBoxIconType::WarningIcon, "Save failed",
                            "Cannot write the selected file.");
            });
    };
    load_.onClick = [this]
    {
        chooser_ = std::make_unique<juce::FileChooser>("Load Dubmist configuration", juce::File(),
                                                       "*.dm98.xml");
        chooser_->launchAsync(
            juce::FileBrowserComponent::openMode | juce::FileBrowserComponent::canSelectFiles,
            [safe = juce::Component::SafePointer<DubmistEditor>(this)](const juce::FileChooser &fc)
            {
                if (safe)
                {
                    auto file = fc.getResult();
                    if (file == juce::File())
                        return;
                    juce::MemoryBlock data;
                    if (file.getSize() > 16384 || !file.loadFileAsData(data) ||
                        !safe->processor_.importPatch(data.getData(), data.getSize()))
                        juce::AlertWindow::showMessageBoxAsync(
                            juce::MessageBoxIconType::WarningIcon, "Load failed",
                            "Expected a complete schema 1 Dubmist patch, maximum 16 KiB.");
                    else
                    {
                        safe->selected_ = -1;
                        safe->preset_.setText(file.getFileNameWithoutExtension(),
                                              juce::dontSendNotification);
                    }
                }
            });
    };
    about_.onClick = []
    {
        juce::AlertWindow::showMessageBoxAsync(
            juce::MessageBoxIconType::InfoIcon, "About Dubmist 98",
            "Dubmist 98 (Enterprise Edition SE)\nVoidmutt Party Research Technologies\nPublished "
            "by Voidmutt Enterprises\nVersion 0.2 - Clouds and rhythmic slices "
            "preview\n\nClouds and stmlib by Emilie Gillet / Mutable Instruments (MIT).\nMonsoon "
            "control reference by Jak Plugg; no hardware artwork reused.\nRhythmic tape curves "
            "adapted from Julius Kammerl (MIT).\nNew source: Copyright "
            "2026 Dubmist contributors, GPLv3 or later.\nJUCE and bundled VST3 SDK: GPLv3 "
            "options.\nIndependent product; no endorsement or affiliation.");
    };
    showRhythm(processor_.snapshot()[RhythmEnabled] > .5f);
    timerCallback();
    startTimerHz(20);
}
DubmistEditor::~DubmistEditor()
{
    stopTimer();
    for (auto *id : {"clear", "trigger"})
        processor_.parameters.getParameter(id)->setValueNotifyingHost(0);
    setLookAndFeel(nullptr);
}
void DubmistEditor::timerCallback()
{
    in_ = std::max(processor_.inputPeak.load(), in_ * .8f);
    out_ = std::max(processor_.outputPeak.load(), out_ * .8f);
    auto s = processor_.snapshot();
    if (!s.valid())
        return;
    mode_.setSelectedId(s[RhythmEnabled] > .5f ? 5 : int(s[Mode]) + 1, juce::dontSendNotification);
    description_.setText(
        rhythmVisible_ && s[RhythmEnabled] < .5f
            ? "Select Rhythmic slices in processing mode to enable these controls."
        : s[RhythmEnabled] > .5f ? "RHYTHMIC SLICES: held history; tape speed changes pitch AND "
                                   "duration. No pitch-preserving time stretch."
        : int(s[Mode]) == 0 ? "POSITION: scan buffer   SIZE: grain duration   DENSITY: rate / "
                              "randomization   TEXTURE: window / diffusion"
        : int(s[Mode]) == 1 ? "WSOLA time-stretch / pitch mode. Controls retain their original "
                              "Clouds alternate-mode behavior."
        : int(s[Mode]) == 2
            ? "Original looping delay. Trigger is immediate, not a host-synchronized beat repeat."
            : "Original spectral processing. Mode and quality changes reset captured audio with a "
              "short fade.",
        juce::dontSendNotification);
    status_.setText(
        juce::String(s[RhythmEnabled] > .5f
                         ? (processor_.rhythmActive.load()     ? "REPEATING"
                            : processor_.clockAvailable.load() ? "RHYTHM READY"
                                                               : "HOST CLOCK UNAVAILABLE")
                         : (s[Freeze] > .5f ? "BUFFER FROZEN" : "LIVE CAPTURE")) +
            "   |   PARAMETER STATE ONLY   |   PEAK " +
            juce::String(out_ > 0 ? 20 * std::log10(out_) : -96, 1) + " dBFS",
        juce::dontSendNotification);
    repaint(690, 12, 250, 42);
}
void DubmistEditor::paint(juce::Graphics &g)
{
    g.fillAll(panel);
    g.setColour(navy);
    g.fillRect(5, 5, 950, 58);
    g.setColour(juce::Colours::white);
    g.setFont(juce::Font(25.f, juce::Font::bold));
    g.drawText("DUBMIST 98", 18, 10, 400, 29, juce::Justification::centredLeft);
    g.setFont(juce::Font(12.f));
    g.drawText("Enterprise Edition SE / Voidmutt Party Research Technologies", 20, 39, 650, 17,
               juce::Justification::centredLeft);
    for (int i = 0; i < 2; ++i)
    {
        float level = i ? out_ : in_;
        g.setColour(juce::Colour(0xff080e0a));
        g.fillRect(750, 15 + i * 22, 185, 13);
        g.setColour(level > 1 ? juce::Colours::red : juce::Colour(0xff71a76d));
        g.fillRect(
            752, 17 + i * 22,
            int(std::clamp((20 * std::log10(std::max(level, .001f)) + 60) / 60, 0.f, 1.f) * 181),
            9);
        g.setColour(juce::Colours::white);
        g.drawText(i ? "OUT" : "IN", 710, 12 + i * 22, 35, 18, juce::Justification::right);
    }
    for (int i = 0; i < 3; ++i)
    {
        auto r = juce::Rectangle<int>(8 + i * 316, 145, 310, 333);
        g.setColour(juce::Colours::darkgrey);
        g.drawRect(r);
        g.setColour(navy);
        g.setFont(juce::Font(12.f, juce::Font::bold));
        g.drawText((rhythmVisible_
                        ? std::array<const char *, 3>{{"01  CLOCK / CAPTURE", "02  SLICE PLAYBACK",
                                                       "03  TAPE / OUTPUT"}}
                        : std::array<const char *, 3>{{"01  GRANULAR PROCESSING",
                                                       "02  MONSOON BLEND CONTROLS",
                                                       "03  QUALITY / PERFORMANCE"}})[size_t(i)],
                   r.removeFromTop(27).reduced(8, 0), juce::Justification::centredLeft);
    }
    g.setColour(juce::Colours::darkgrey);
    g.drawRect(8, 484, 944, 47);
    g.drawRect(8, 548, 944, 47);
}
void DubmistEditor::resized()
{
    preset_.setBounds(12, 75, 330, 25);
    init_.setBounds(350, 75, 110, 25);
    load_.setBounds(470, 75, 90, 25);
    save_.setBounds(570, 75, 90, 25);
    about_.setBounds(850, 75, 96, 25);
    mode_.setBounds(12, 110, 470, 25);
    quality_.setBounds(650, 185, 284, 25);
    for (size_t i = 0; i < 11; ++i)
    {
        int col = i < 5 ? 0 : i < 9 ? 1 : 2;
        int row = i < 5 ? int(i) : i < 9 ? int(i) - 5 : int(i) - 9 + 1;
        int x = 20 + col * 316, y = 183 + row * 53;
        labels_[i].setBounds(x, y, 285, 17);
        sliders_[i].setBounds(x, y + 18, 285, 25);
    }
    freeze_.setBounds(650, 355, 130, 28);
    bypass_.setBounds(800, 355, 130, 28);
    trigger_.setBounds(650, 397, 135, 29);
    clear_.setBounds(795, 397, 139, 29);
    cloudsView_.setBounds(510, 110, 100, 25);
    rhythmView_.setBounds(620, 110, 100, 25);
    sync_.setBounds(20, 183, 280, 28);
    divisionLabel_.setBounds(20, 220, 280, 17);
    division_.setBounds(20, 238, 280, 25);
    quantize_.setBounds(20, 275, 280, 28);
    repeat_.setBounds(20, 375, 280, 28);
    restartBar_.setBounds(20, 418, 280, 28);
    reverse_.setBounds(336, 183, 280, 28);
    alternate_.setBounds(336, 220, 280, 28);
    auto place = [this](size_t i, int x, int y)
    {
        rhythmLabels_[i].setBounds(x, y, 280, 17);
        rhythmSliders_[i].setBounds(x, y + 18, 280, 25);
    };
    place(0, 336, 265);
    place(1, 336, 318);
    place(2, 336, 371);
    place(3, 652, 183);
    place(4, 652, 236);
    place(5, 20, 320);
    if (rhythmVisible_)
    {
        labels_[5].setBounds(652, 300, 280, 17);
        sliders_[5].setBounds(652, 318, 280, 25);
    }
    description_.setBounds(17, 490, 925, 35);
    status_.setBounds(17, 554, 925, 35);
}

void DubmistEditor::showRhythm(bool show)
{
    rhythmVisible_ = show;
    for (size_t i = 0; i < 11; ++i)
    {
        sliders_[i].setVisible(!show || i == 5);
        labels_[i].setVisible(!show || i == 5);
    }
    quality_.setVisible(!show);
    for (auto *c :
         std::array<juce::Component *, 8>{{&sync_, &repeat_, &reverse_, &alternate_, &quantize_,
                                           &restartBar_, &division_, &divisionLabel_}})
        c->setVisible(show);
    for (size_t i = 0; i < 6; ++i)
    {
        rhythmSliders_[i].setVisible(show);
        rhythmLabels_[i].setVisible(show);
    }
    trigger_.setButtonText(show ? "RETRIGGER" : "TRIGGER GRAIN");
    resized();
    timerCallback();
    repaint();
}
