// SPDX-License-Identifier: GPL-3.0-or-later
#include "Editor.h"
using namespace soundgyal;
namespace
{
auto surface = juce::Colour(0xffc8c8bd), navy = juce::Colour(0xff152e50),
     ink = juce::Colour(0xff16191b);
juce::StringArray choicesFor(size_t i)
{
    if (i == Model)
    {
        juce::StringArray a;
        for (auto *name : engines)
            a.add(name);
        return a;
    }
    if (i == Mode)
        return {"Mono", "Legato", "Polyphonic (8)"};
    if (i == FilterMode)
        return {"Bypass", "Low-pass", "High-pass", "Band-pass"};
    if (i == DriveMode)
        return {"Bypass", "Soft saturation", "Hard clip", "Wavefold"};
    if (i == SubShape)
        return {"Sine", "Triangle"};
    if (i == SubOctave)
        return {"-2 octaves", "-1 octave", "Unison"};
    if (i == Lfo1Shape || i == Lfo2Shape)
        return {"Sine", "Triangle", "Saw", "Square"};
    if (i >= RouteBase)
    {
        juce::StringArray a;
        if ((i - RouteBase) % 3 == 0)
            for (auto *n : sourceNames)
                a.add(n);
        else
            for (auto *n : targetNames)
                a.add(n);
        return a;
    }
    return {"Off", "On"};
}
juce::String hint(int engine)
{
    if (engine >= 2 && engine <= 4)
        return "HARMONICS: FM program   |   TIMBRE: brightness   |   MORPH: envelope control";
    switch (engine)
    {
    case 0:
        return "HARMONICS: filter resonance   |   TIMBRE: filter frequency   |   MORPH: oscillator "
               "shape";
    case 8:
        return "HARMONICS: oscillator detuning   |   TIMBRE / MORPH: square and saw shape / sync";
    case 10:
        return "HARMONICS: frequency ratio   |   TIMBRE: modulation index   |   MORPH: feedback";
    case 21:
        return "HARMONICS: attack FM / drive   |   TIMBRE: tone   |   MORPH: drum decay";
    case 22:
        return "Snare model: explore harmonic balance, tone and decay. AUX is the alternate drum "
               "model.";
    case 23:
        return "Hi-hat model: metallic / noise balance and articulation. AUX is the alternate hat "
               "model.";
    case 19:
    case 20:
        return "Physical model: excitation and resonator controls. Trigger and decay remain part "
               "of the model.";
    default:
        return "Original Plaits engine-specific macro behavior. MAIN and AUX may be different "
               "signals; explore both.";
    }
}
} // namespace
CorporateLook::CorporateLook()
{
    setColour(juce::ResizableWindow::backgroundColourId, surface);
    setColour(juce::Label::textColourId, ink);
    setColour(juce::Slider::textBoxTextColourId, ink);
    setColour(juce::Slider::textBoxBackgroundColourId, juce::Colour(0xfff0f0e8));
    setColour(juce::ComboBox::backgroundColourId, juce::Colour(0xfff4f4ed));
    setColour(juce::ComboBox::textColourId, ink);
    setColour(juce::ComboBox::arrowColourId, ink);
    setColour(juce::PopupMenu::backgroundColourId, surface);
    setColour(juce::PopupMenu::textColourId, ink);
    setColour(juce::TextButton::textColourOffId, ink);
    setColour(juce::TextButton::buttonColourId, surface);
    setColour(juce::TabbedButtonBar::tabTextColourId, ink);
}
void CorporateLook::drawButtonBackground(juce::Graphics &g, juce::Button &b, const juce::Colour &,
                                         bool over, bool down)
{
    auto r = b.getLocalBounds();
    g.fillAll(over ? surface.brighter(.08f) : surface);
    g.setColour(down ? juce::Colours::darkgrey : juce::Colours::white);
    g.drawLine(0, 0, float(r.getWidth()), 0);
    g.drawLine(0, 0, 0, float(r.getHeight()));
    g.setColour(down ? juce::Colours::white : juce::Colours::darkgrey);
    g.drawLine(0, float(r.getBottom() - 1), float(r.getRight()), float(r.getBottom() - 1));
    g.drawLine(float(r.getRight() - 1), 0, float(r.getRight() - 1), float(r.getBottom()));
}
void CorporateLook::drawLinearSlider(juce::Graphics &g, int x, int y, int w, int h, float pos,
                                     float, float, const juce::Slider::SliderStyle, juce::Slider &)
{
    int mid = y + h / 2;
    g.setColour(juce::Colours::darkgrey);
    g.fillRect(x, mid - 2, w, 4);
    g.setColour(juce::Colours::white);
    g.drawHorizontalLine(mid + 2, float(x), float(x + w));
    juce::Rectangle<int> thumb(int(pos) - 5, mid - 8, 10, 16);
    g.setColour(surface);
    g.fillRect(thumb);
    g.setColour(juce::Colours::white);
    g.drawRect(thumb);
    g.setColour(ink);
    g.drawLine(float(thumb.getRight()), float(thumb.getY()), float(thumb.getRight()),
               float(thumb.getBottom()));
}
PropertyPage::PropertyPage(SoundgyalProcessor &p, std::vector<size_t> ids,
                           std::vector<juce::String> headers, int cols)
    : processor(p), headings(std::move(headers)), columns(cols)
{
    for (auto i : ids)
    {
        auto label = std::make_unique<juce::Label>();
        label->setFont(juce::Font(12.f));
        label->setText(i < RouteBase
                           ? baseSpecs[i].name
                           : juce::String("Route ") + juce::String(int((i - RouteBase) / 3 + 1)) +
                                 std::array<const char *, 3>{
                                     {" source", " destination", " depth"}}[(i - RouteBase) % 3],
                       juce::dontSendNotification);
        addAndMakeVisible(*label);
        labels.push_back(std::move(label));
        bool discrete = i < RouteBase ? baseSpecs[i].discrete : (i - RouteBase) % 3 != 2;
        if (discrete)
        {
            auto combo = std::make_unique<juce::ComboBox>();
            combo->addItemList(choicesFor(i), 1);
            combo->setTooltip(labels.back()->getText());
            addAndMakeVisible(*combo);
            rows.push_back(combo.get());
            choiceAttachments.push_back(
                std::make_unique<juce::AudioProcessorValueTreeState::ComboBoxAttachment>(
                    p.parameters, SoundgyalProcessor::parameterID(i), *combo));
            choices.push_back(std::move(combo));
        }
        else
        {
            auto slider = std::make_unique<juce::Slider>(juce::Slider::LinearHorizontal,
                                                         juce::Slider::TextBoxRight);
            slider->setTextBoxStyle(juce::Slider::TextBoxRight, false, 65, 20);
            slider->setDoubleClickReturnValue(true, i < RouteBase ? baseSpecs[i].initial : 0);
            slider->setVelocityModeParameters(.2, 1, .02, true, juce::ModifierKeys::shiftModifier);
            slider->setTooltip(
                labels.back()->getText() +
                " — drag; Shift for fine adjustment; double-click to reset; type a value.");
            addAndMakeVisible(*slider);
            rows.push_back(slider.get());
            sliderAttachments.push_back(
                std::make_unique<juce::AudioProcessorValueTreeState::SliderAttachment>(
                    p.parameters, SoundgyalProcessor::parameterID(i), *slider));
            slider->textFromValueFunction = [i](double value)
            {
                return juce::String(value, i == Cutoff ? 0 : i == Output ? 1 : 3);
            };
            slider->updateText();
            sliders.push_back(std::move(slider));
        }
    }
}
void PropertyPage::resized()
{
    int per = rows.size() == 14 ? 8 : int((rows.size() + size_t(columns) - 1) / size_t(columns));
    int width = getWidth() / columns;
    for (size_t i = 0; i < rows.size(); ++i)
    {
        int col = int(i) / per, row = int(i) % per;
        int x = col * width + 14, y = 38 + row * 44;
        labels[i]->setBounds(x, y, width - 28, 17);
        rows[i]->setBounds(x, y + 17, width - 28, 23);
    }
}
void PropertyPage::paint(juce::Graphics &g)
{
    g.fillAll(surface);
    int width = getWidth() / columns;
    for (int i = 0; i < columns; ++i)
    {
        auto r = juce::Rectangle<int>(i * width + 6, 9, width - 12, getHeight() - 18);
        g.setColour(juce::Colours::darkgrey);
        g.drawRect(r);
        g.setColour(navy);
        g.setFont(juce::Font(12.f, juce::Font::bold));
        g.drawText(headings[size_t(i)], r.removeFromTop(24).reduced(8, 0),
                   juce::Justification::centredLeft);
    }
}
SoundgyalEditor::SoundgyalEditor(SoundgyalProcessor &p) : AudioProcessorEditor(p), processor(p)
{
    setLookAndFeel(&look);
    setSize(1100, 750);
    setResizable(false, false);
    for (juce::Component *c : std::array<juce::Component *, 13>{
             {static_cast<juce::Component *>(&preset), &category, &previous, &next, &init, &save,
              &load, &about, &tabs, &description, &status, &keyboard}})
        addAndMakeVisible(c);
    category.addItem("All departments", 1);
    category.addItemList(
        {"JUNGLE", "DRUM & BASS", "UK GARAGE", "140", "PERCUSSION", "BREAKCORE", "DUBSTEP"}, 2);
    category.setSelectedId(1, juce::dontSendNotification);
    category.onChange = [this] { fillPresets(); };
    fillPresets();
    preset.onChange = [this]
    {
        int i = preset.getSelectedId() - 1;
        if (i >= 0)
            browse(i);
    };
    previous.onClick = [this]
    {
        browse(selected < 0 ? int(factoryCount) - 1
                            : (selected + int(factoryCount) - 1) % int(factoryCount));
    };
    next.onClick = [this] { browse((selected + 1) % int(factoryCount)); };
    init.onClick = [this]
    {
        selected = -1;
        processor.apply(Settings{});
        preset.setText("Initialized configuration", juce::dontSendNotification);
    };
    tabs.setTabBarDepth(27);
    tabs.addTab("SYNTHESIS", surface,
                new PropertyPage(
                    p, {Model,    Harmonics, Timbre,    Morph,     Aux,          Decay,  Colour,
                        Lpg,      Mode,      Glide,     BendRange, Layer,        Detune, Width,
                        Sub,      SubShape,  SubOctave, LowMono,   FilterMode,   Cutoff, Resonance,
                        Keytrack, FilterEnv, DriveMode, Drive,     Compensation, Output},
                    {"01  PLAITS SYNTHESIS", "02  BASS / VOICE CONFIGURATION",
                     "03  FILTER / SIGNAL PROCESSING"},
                    3),
                true);
    tabs.addTab(
        "MOVEMENT", surface,
        new PropertyPage(p,
                         {Attack, AmpDecay, Sustain, Release, ModAttack, ModDecay, ModSustain,
                          ModRelease, Lfo1Rate, Lfo1Sync, Lfo1Shape, Lfo2Rate, Lfo2Sync, Lfo2Shape},
                         {"01  ENVELOPES (SECONDS)", "02  LFO CONFIGURATION"}, 2),
        true);
    std::vector<size_t> matrix;
    for (size_t c = 0; c < 3; ++c)
        for (size_t r = 0; r < 8; ++r)
            matrix.push_back(RouteBase + 3 * r + c);
    // Align eight source, destination and depth controls across matching rows.
    tabs.addTab("MODULATION MATRIX", surface,
                new PropertyPage(
                    p, matrix, {"01  MODULATION SOURCE", "02  DESTINATION", "03  SIGNED DEPTH"}, 3),
                true);
    description.setFont(juce::Font(12.f));
    status.setFont(juce::Font(12.f));
    keyboard.setAvailableRange(24, 96);
    keyboard.setLowestVisibleKey(36);
    keyboard.setKeyWidth(19);
    keyboardState.addListener(this);
    save.onClick = [this]
    {
        chooser = std::make_unique<juce::FileChooser>(
            "Save SOUNDGYAL patch",
            juce::File::getSpecialLocation(juce::File::userDocumentsDirectory)
                .getChildFile("Untitled.sg99.xml"),
            "*.sg99.xml");
        chooser->launchAsync(
            juce::FileBrowserComponent::saveMode | juce::FileBrowserComponent::canSelectFiles |
                juce::FileBrowserComponent::warnAboutOverwriting,
            [safe =
                 juce::Component::SafePointer<SoundgyalEditor>(this)](const juce::FileChooser &fc)
            {
                if (safe)
                {
                    auto file = fc.getResult();
                    if (file != juce::File() &&
                        !file.replaceWithText(safe->processor.exportPatch()))
                        juce::AlertWindow::showMessageBoxAsync(
                            juce::MessageBoxIconType::WarningIcon, "Patch save failed",
                            "The selected file could not be written.");
                }
            });
    };
    load.onClick = [this]
    {
        chooser =
            std::make_unique<juce::FileChooser>("Load SOUNDGYAL patch", juce::File(), "*.sg99.xml");
        chooser->launchAsync(
            juce::FileBrowserComponent::openMode | juce::FileBrowserComponent::canSelectFiles,
            [safe =
                 juce::Component::SafePointer<SoundgyalEditor>(this)](const juce::FileChooser &fc)
            {
                if (safe)
                {
                    auto file = fc.getResult();
                    if (file == juce::File())
                        return;
                    juce::MemoryBlock data;
                    if (file.getSize() > 65536 || !file.loadFileAsData(data) ||
                        !safe->processor.importPatch(data.getData(), data.getSize()))
                        juce::AlertWindow::showMessageBoxAsync(
                            juce::MessageBoxIconType::WarningIcon, "Patch load failed",
                            "Expected a complete, valid SOUNDGYAL schema 1 patch (64 KiB "
                            "maximum).");
                    else
                    {
                        safe->selected = -1;
                        safe->preset.setText(file.getFileNameWithoutExtension(),
                                             juce::dontSendNotification);
                    }
                }
            });
    };
    about.onClick = []
    {
        juce::AlertWindow::showMessageBoxAsync(
            juce::MessageBoxIconType::InfoIcon, "About SOUNDGYAL 99",
            "SOUNDGYAL 99 Enterprise Edition\nVersion 0.3 — Voidmutt Enterprises\n\nOriginal "
            "Plaits and stmlib DSP by Emilie Gillet / Mutable Instruments (MIT).\nNew integration: "
            "© 2026 SOUNDGYAL contributors, GPLv3 or later.\nJUCE 7 and VST3 integration used "
            "under GPLv3.\n\nIndependent product. No affiliation or endorsement.\nSYSTEM READY.");
    };
    auto current = processor.snapshot();
    for (int i = 0; i < int(factoryCount); ++i)
    {
        bool match = true;
        for (size_t k = 0; k < paramCount; ++k)
            if (std::abs(current[k] - processor.factory[size_t(i)].patch[k]) >
                1.e-5f * std::max(1.f, std::abs(current[k])))
                match = false;
        if (match)
        {
            selected = i;
            preset.setText(processor.factory[size_t(i)].name, juce::dontSendNotification);
            break;
        }
    }
    if (selected < 0)
        preset.setText("Current host configuration", juce::dontSendNotification);
    timerCallback();
    startTimerHz(20);
}
SoundgyalEditor::~SoundgyalEditor()
{
    stopTimer();
    keyboardState.allNotesOff(0);
    keyboardState.removeListener(this);
    setLookAndFeel(nullptr);
}
void SoundgyalEditor::handleNoteOn(juce::MidiKeyboardState *, int, int note, float velocity)
{
    processor.audition(note, velocity);
}
void SoundgyalEditor::handleNoteOff(juce::MidiKeyboardState *, int, int note, float)
{
    processor.audition(note, 0);
}
void SoundgyalEditor::fillPresets()
{
    auto current = preset.getText();
    preset.clear(juce::dontSendNotification);
    for (int i = 0; i < int(factoryCount); ++i)
        if (category.getSelectedId() == 1 ||
            category.getText() == processor.factory[size_t(i)].category)
            preset.addItem(processor.factory[size_t(i)].name, i + 1);
    preset.setText(current.isEmpty() ? "Current host configuration" : current,
                   juce::dontSendNotification);
}
void SoundgyalEditor::browse(int i)
{
    selected = i;
    processor.apply(processor.factory[size_t(i)].patch);
    preset.setText(processor.factory[size_t(i)].name, juce::dontSendNotification);
}
void SoundgyalEditor::timerCallback()
{
    meter = std::max(processor.peak.load(), meter * .8f);
    auto s = processor.snapshot();
    description.setText(juce::String(engines[int(s[Model])]) + "   |   " + hint(int(s[Model])),
                        juce::dontSendNotification);
    if (selected >= 0)
    {
        bool edited = false;
        for (size_t i = 0; i < paramCount; ++i)
            if (std::abs(s[i] - processor.factory[size_t(selected)].patch[i]) >
                1.e-5f * std::max(1.f, std::abs(s[i])))
                edited = true;
        preset.setText(juce::String(processor.factory[size_t(selected)].name) +
                           (edited ? " (edited)" : ""),
                       juce::dontSendNotification);
    }
    int midi = processor.midiLight.load();
    if (midi > 0)
        processor.midiLight.fetch_sub(1);
    status.setText(juce::String(midi > 0 ? "MIDI RX" : "SYSTEM READY") + "     |     " +
                       juce::String(processor.voiceCount.load()) +
                       " / 8 VOICES     |     48 kHz DSP / HOST SRC     |     PEAK " +
                       juce::String(meter > 0 ? 20 * std::log10(meter) : -96, 1) + " dBFS",
                   juce::dontSendNotification);
    repaint(820, 12, 260, 50);
}
void SoundgyalEditor::paint(juce::Graphics &g)
{
    g.fillAll(surface);
    g.setColour(navy);
    g.fillRect(5, 5, getWidth() - 10, 58);
    g.setColour(juce::Colours::white);
    g.setFont(juce::Font(25.f, juce::Font::bold));
    g.drawText("SOUNDGYAL 99", 18, 10, 400, 29, juce::Justification::centredLeft);
    g.setFont(juce::Font(12.f));
    g.drawText("Enterprise Edition  /  Voidmutt Enterprises", 20, 39, 450, 17,
               juce::Justification::centredLeft);
    g.drawText("AUDIO ENGINE ONLINE", 825, 12, 240, 18, juce::Justification::right);
    g.setColour(juce::Colour(0xff080e0a));
    g.fillRect(825, 37, 240, 13);
    g.setColour(meter > 1 ? juce::Colours::red : juce::Colour(0xff71a76d));
    g.fillRect(827, 39,
               int(std::clamp((20 * std::log10(std::max(meter, .001f)) + 60) / 60, 0.f, 1.f) * 236),
               9);
    g.setColour(juce::Colours::darkgrey);
    g.drawRect(7, 656, 1086, 25);
    g.drawRect(5, 724, 1090, 22);
}
void SoundgyalEditor::resized()
{
    category.setBounds(9, 71, 150, 26);
    previous.setBounds(166, 71, 26, 26);
    preset.setBounds(197, 71, 340, 26);
    next.setBounds(542, 71, 26, 26);
    init.setBounds(580, 71, 105, 26);
    load.setBounds(693, 71, 86, 26);
    save.setBounds(787, 71, 86, 26);
    about.setBounds(981, 71, 108, 26);
    tabs.setBounds(7, 105, 1086, 546);
    description.setBounds(12, 658, 1070, 21);
    keyboard.setBounds(9, 685, 1082, 34);
    status.setBounds(10, 726, 1080, 18);
}
