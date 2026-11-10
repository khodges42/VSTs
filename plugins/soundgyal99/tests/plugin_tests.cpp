// SPDX-License-Identifier: GPL-3.0-or-later
#include "Processor.h"
#include <iostream>
#include <chrono>
#include <stdexcept>
namespace
{
void check(bool b, const char *msg)
{
    if (!b)
        throw std::runtime_error(msg);
}
std::vector<float> render(SoundgyalProcessor &p, double rate, int block, int seconds = 2)
{
    p.setRateAndBufferSizeDetails(rate, block);
    p.prepareToPlay(rate, block);
    std::vector<float> audio(size_t(rate) * size_t(seconds) * 2);
    juce::AudioBuffer<float> buffer(2, block);
    juce::MidiBuffer midi;
    midi.ensureSize(2048);
    int total = int(rate) * seconds;
    float maximum = 0;
    struct Event
    {
        int frame;
        juce::MidiMessage message;
    };
    std::vector<Event> events;
    for (int n = 0; n < int(rate * .8); n += int(rate / 8))
    {
        int step = n / int(rate / 8);
        int note = step % 4 == 0 ? 36 : step % 4 == 1 ? 39 : step % 4 == 2 ? 43 : 34;
        events.push_back({n, juce::MidiMessage::noteOn(1, note, .8f)});
        events.push_back({n + int(rate / 12), juce::MidiMessage::noteOff(1, note)});
    }
    std::sort(events.begin(), events.end(),
              [](const Event &a, const Event &b) { return a.frame < b.frame; });
    size_t cursor = 0;
    for (int n = 0; n < total; n += block)
    {
        int count = std::min(block, total - n);
        buffer.setSize(2, count, false, false, true);
        buffer.clear();
        midi.clear();
        while (cursor < events.size() && events[cursor].frame < n + count)
        {
            midi.addEvent(events[cursor].message, events[cursor].frame - n);
            ++cursor;
        }
        p.processBlock(buffer, midi);
        for (int i = 0; i < count; ++i)
            for (int c = 0; c < 2; ++c)
            {
                float x = buffer.getSample(c, i);
                check(std::isfinite(x), "Nonfinite output");
                maximum = std::max(maximum, std::abs(x));
                audio[size_t(n + i) * 2 + size_t(c)] = x;
            }
    }
    check(maximum > 1.e-4f, "Silent preset");
    check(maximum < 4, "Gain explosion");
    return audio;
}
void writeWav(juce::File file, const std::vector<float> &x, int rate)
{
    auto stream = file.createOutputStream();
    check(stream != nullptr, "WAV stream");
    stream->setPosition(0);
    stream->truncate();
    juce::WavAudioFormat format;
    std::unique_ptr<juce::AudioFormatWriter> writer(
        format.createWriterFor(stream.release(), rate, 2, 24, {}, 0));
    check(writer != nullptr, "WAV writer");
    juce::AudioBuffer<float> buffer(2, int(x.size() / 2));
    for (int i = 0; i < buffer.getNumSamples(); ++i)
        for (int c = 0; c < 2; ++c)
            buffer.setSample(c, i, x[size_t(i) * 2 + size_t(c)]);
    check(writer->writeFromAudioSampleBuffer(buffer, 0, buffer.getNumSamples()), "WAV write");
}

std::vector<float> sustained(SoundgyalProcessor &p, const soundgyal::Settings &patch, int note)
{
    p.apply(patch);
    p.setRateAndBufferSizeDetails(48000, 127);
    p.prepareToPlay(48000, 127);
    std::vector<float> audio(48000 * 3 * 2);
    juce::AudioBuffer<float> buffer(2, 127);
    juce::MidiBuffer midi;
    for (int start = 0; start < 144000; start += 127)
    {
        int count = std::min(127, 144000 - start);
        buffer.setSize(2, count, false, false, true);
        midi.clear();
        if (start == 0)
            midi.addEvent(juce::MidiMessage::noteOn(1, note, .8f), 0);
        if (start <= 96000 && start + count > 96000)
            midi.addEvent(juce::MidiMessage::noteOff(1, note), 96000 - start);
        p.processBlock(buffer, midi);
        for (int n = 0; n < count; ++n)
            for (int c = 0; c < 2; ++c)
            {
                float x = buffer.getSample(c, n);
                check(std::isfinite(x) && std::abs(x) < 1, "Sustained preset range");
                audio[size_t(start + n) * 2 + size_t(c)] = x;
            }
    }
    return audio;
}

// Correlation near the played note's period: rejects broadband output even
// when a bass reinforcement oscillator would mask the bad recipe's tuning.
double pitchedCorrelation(const std::vector<float> &audio, int note)
{
    double period = 48000. / (440 * std::exp2((note - 69) / 12.));
    double best = -1;
    for (int lag = int(period * .97); lag <= int(std::ceil(period * 1.03)); ++lag)
    {
        double xy = 0, xx = 0, yy = 0;
        for (int i = 48000; i < 84000; ++i)
        {
            double x = audio[size_t(i) * 2], y = audio[size_t(i + lag) * 2];
            xy += x * y;
            xx += x * x;
            yy += y * y;
        }
        if (xx * yy > 1.e-12)
            best = std::max(best, xy / std::sqrt(xx * yy));
    }
    return best;
}
} // namespace
int main(int argc, char **argv)
{
    juce::ScopedJuceInitialiser_GUI gui;
    try
    {
        auto output = juce::File::getCurrentWorkingDirectory().getChildFile("validation");
        if (argc > 1)
            output = juce::File(juce::String(argv[1]));
        output.createDirectory();
        auto p = std::make_unique<SoundgyalProcessor>();
        auto start = std::chrono::steady_clock::now();
        auto independent = std::make_unique<SoundgyalProcessor>();
        check(p->factory.size() == 35, "Expected 35 factory configurations");
        auto fixture = juce::File(juce::String(__FILE__))
                           .getSiblingFile("fixtures")
                           .getChildFile("Jungle Hoover-0.2.sg99.xml")
                           .loadFileAsString();
        check(p->importPatch(fixture.toRawUTF8(), fixture.getNumBytesAsUTF8()),
              "Legacy Hoover state must still restore");
        auto legacy = p->snapshot();
        check(legacy[soundgyal::Model] == 16, "Saved Swarm sessions must not migrate silently");
        auto fixed = p->factory[1].patch;
        // Preserve audible before/after examples, then remove the sub for diagnosis.
        writeWav(output.getChildFile("Hoover-before-note60.wav"), sustained(*p, legacy, 60), 48000);
        writeWav(output.getChildFile("Hoover-after-note60.wav"), sustained(*p, fixed, 60), 48000);
        legacy[soundgyal::Sub] = fixed[soundgyal::Sub] = 0;
        for (int note : {48, 60, 72})
        {
            auto beforeAudio = sustained(*p, legacy, note);
            auto afterAudio = sustained(*p, fixed, note);
            double beforeScore = pitchedCorrelation(beforeAudio, note);
            double afterScore = pitchedCorrelation(afterAudio, note);
            std::cout << "HOOVER note=" << note << " original=" << beforeScore
                      << " fixed=" << afterScore << "\n";
            check(afterScore > .6 && afterScore > beforeScore + .15,
                  "Hoover must remain pitched without sub reinforcement");
        }
        auto moving = sustained(*p, fixed, 60);
        for (int r = 0; r < 3; ++r)
            fixed[soundgyal::RouteBase + 3 * size_t(r) + 2] = 0;
        auto staticVersion = sustained(*p, fixed, 60);
        double movement = 0;
        for (size_t i = 0; i < moving.size(); ++i)
            movement += std::abs(moving[i] - staticVersion[i]);
        check(movement > 10, "Hoover movement routes must affect audio");
        for (size_t i = 0; i < p->factory.size(); ++i)
        {
            auto &f = p->factory[i];
            check(f.patch.valid(), "Factory range");
            p->apply(f.patch);
            auto text = p->exportPatch();
            check(output.getChildFile(juce::String(f.name) + ".sg99.xml").replaceWithText(text),
                  "Preset write");
            check(independent->importPatch(text.toRawUTF8(), text.getNumBytesAsUTF8()),
                  "Factory import");
            check(independent->snapshot().value == p->snapshot().value,
                  "Factory exact configuration restore");
            auto audio = render(*p, 48000, 127);
            auto repeated = render(*independent, 48000, 127);
            check(audio == repeated, "Factory deterministic independent render");
            writeWav(output.getChildFile(juce::String(f.name) + ".wav"), audio, 48000);
            if (i >= 25)
                writeWav(output.getChildFile(juce::String(f.name) + "-held.wav"),
                         sustained(*p, f.patch, i == 27 || i == 34 ? 60 : 36), 48000);
            std::cout << "RENDER " << f.name << "\n";
        }
        p->apply(p->factory[5].patch);
        juce::MemoryBlock state;
        p->getStateInformation(state);
        auto original = p->snapshot();
        p->apply(soundgyal::Settings{});
        p->setStateInformation(state.getData(), int(state.getSize()));
        check(p->snapshot().value == original.value, "Host state roundtrip");
        auto q = std::make_unique<SoundgyalProcessor>();
        q->setStateInformation(state.getData(), int(state.getSize()));
        check(q->snapshot().value == original.value, "Independent state");
        auto before = p->snapshot();
        const char *bad = "<SOUNDGYAL99_PATCH schema=\"2\"/>";
        check(!p->importPatch(bad, strlen(bad)), "Unknown schema");
        check(p->snapshot().value == before.value, "Bad patch mutated state");
        for (double rate : {32000., 44100., 48000., 88200., 96000., 192000.})
        {
            p->apply(p->factory[7].patch);
            render(*p, rate, 257, 1);
            std::cout << "RATE " << rate << "\n";
        }
        // Render the same deterministic patch with different host partitions.
        p->apply(p->factory[0].patch);
        q->apply(p->factory[0].patch);
        auto a = render(*p, 48000, 1, 1), b = render(*q, 48000, 127, 1);
        check(a == b, "Host buffer partition dependence");
        p->apply(p->factory[5].patch);
        std::unique_ptr<juce::AudioProcessorEditor> editor(p->createEditor());
        editor->setVisible(true);
        juce::ComboBox *factoryBrowser = nullptr, *categoryBrowser = nullptr;
        juce::TextButton *nextPreset = nullptr;
        for (auto *child : editor->getChildren())
        {
            if (auto *combo = dynamic_cast<juce::ComboBox *>(child))
            {
                if (combo->getNumItems() == int(soundgyal::factoryCount))
                    factoryBrowser = combo;
                if (combo->getItemText(combo->getNumItems() - 1) == "DUBSTEP")
                    categoryBrowser = combo;
            }
            if (auto *button = dynamic_cast<juce::TextButton *>(child))
                if (button->getButtonText() == ">")
                    nextPreset = button;
        }
        check(factoryBrowser && categoryBrowser && nextPreset, "Factory browser controls");
        factoryBrowser->setSelectedId(35, juce::sendNotificationSync);
        auto expected = std::make_unique<SoundgyalProcessor>();
        expected->apply(p->factory.back().patch);
        check(p->snapshot().value == expected->snapshot().value, "Last new preset accessible");
        nextPreset->onClick();
        expected->apply(p->factory.front().patch);
        check(p->snapshot().value == expected->snapshot().value, "Expanded bank wraps correctly");
        categoryBrowser->setSelectedId(categoryBrowser->getNumItems(), juce::sendNotificationSync);
        check(factoryBrowser->getNumItems() == 5 && factoryBrowser->indexOfItemId(34) >= 0,
              "All five dubstep patches appear in category");
        categoryBrowser->setSelectedId(1, juce::sendNotificationSync);
        factoryBrowser->setSelectedId(2, juce::sendNotificationSync);
        auto image = editor->createComponentSnapshot(editor->getLocalBounds(), true, 1);
        juce::PNGImageFormat png;
        auto stream = output.getChildFile("soundgyal-synthesis.png").createOutputStream();
        if (stream)
        {
            stream->setPosition(0);
            stream->truncate();
        }
        check(stream && png.writeImageToStream(image, *stream), "Screenshot write");
        for (auto *child : editor->getChildren())
            if (auto *tabs = dynamic_cast<juce::TabbedComponent *>(child))
            {
                tabs->setCurrentTabIndex(2);
                auto matrix = editor->createComponentSnapshot(editor->getLocalBounds(), true, 1);
                auto m = output.getChildFile("soundgyal-matrix.png").createOutputStream();
                if (m)
                {
                    m->setPosition(0);
                    m->truncate();
                }
                check(m && png.writeImageToStream(matrix, *m), "Matrix screenshot");
            }
        std::cout
            << "PASS state, factory, rates, buffer partitioning, actual editor snapshots; elapsed "
            << std::chrono::duration<double>(std::chrono::steady_clock::now() - start).count()
            << " s\n";
        return 0;
    }
    catch (const std::exception &e)
    {
        std::cerr << "FAIL " << e.what() << "\n";
        return 1;
    }
}
