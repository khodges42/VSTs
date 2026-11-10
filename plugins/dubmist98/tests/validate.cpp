// SPDX-License-Identifier: GPL-3.0-or-later
#include "Processor.h"
#include <iostream>
#include <memory>
#include <vector>
#include <stdexcept>
#include <cstring>
#include <fstream>
#include "stmlib/utils/random.h"
namespace
{
void check(bool condition, const char *message)
{
    if (!condition)
        throw std::runtime_error(message);
}
constexpr double pi = 3.141592653589793;
dubmist::Frame source(int n, double rate, double bpm)
{
    double t = n / rate, beat = t * bpm / 60;
    double pulse = beat - std::floor(beat);
    double noise = std::sin(n * 37.17) * std::sin(n * 1.913);
    double kick =
        std::sin(2 * pi * (55 * t + 8 * (1 - std::exp(-pulse * 25)))) * std::exp(-pulse * 24);
    float l =
        float(.22 * kick + .07 * noise * std::exp(-pulse * 60) + .06 * std::sin(2 * pi * 220 * t));
    float r =
        float(.16 * kick + .06 * noise * std::exp(-pulse * 40) + .05 * std::sin(2 * pi * 330 * t));
    return {{l, r}};
}
std::vector<float> render(DubmistProcessor &p, double rate, int block, int seconds = 3,
                          double bpm = 175)
{
    p.setRateAndBufferSizeDetails(rate, block);
    p.prepareToPlay(rate, block);
    int total = int(rate) * seconds;
    std::vector<float> audio(size_t(total) * 2);
    juce::AudioBuffer<float> b(2, block);
    juce::MidiBuffer m;
    double energy = 0;
    for (int start = 0; start < total; start += block)
    {
        int count = std::min(block, total - start);
        b.setSize(2, count, false, false, true);
        for (int i = 0; i < count; ++i)
        {
            auto x = source(start + i, rate, bpm);
            b.setSample(0, i, x[0]);
            b.setSample(1, i, x[1]);
        }
        p.processBlock(b, m);
        for (int i = 0; i < count; ++i)
            for (int c = 0; c < 2; ++c)
            {
                float v = b.getSample(c, i);
                check(std::isfinite(v) && std::abs(v) < 2, "DSP stability");
                audio[size_t(start + i) * 2 + size_t(c)] = v;
                energy += v * v;
            }
    }
    check(energy > .01, "Unexpected silent processing");
    return audio;
}
void wav(juce::File f, const std::vector<float> &audio, int rate)
{
    auto stream = f.createOutputStream();
    check(stream != nullptr, "WAV open");
    stream->setPosition(0);
    stream->truncate();
    juce::WavAudioFormat format;
    std::unique_ptr<juce::AudioFormatWriter> writer(
        format.createWriterFor(stream.release(), rate, 2, 24, {}, 0));
    check(writer != nullptr, "WAV writer");
    juce::AudioBuffer<float> b(2, int(audio.size() / 2));
    for (int n = 0; n < b.getNumSamples(); ++n)
        for (int c = 0; c < 2; ++c)
            b.setSample(c, n, audio[size_t(n) * 2 + size_t(c)]);
    check(writer->writeFromAudioSampleBuffer(b, 0, b.getNumSamples()), "WAV write");
}
struct Reference
{
    alignas(16) std::array<uint8_t, 118784> large{};
    alignas(16) std::array<uint8_t, 65408> small{};
    clouds::GranularProcessor dsp;
    Reference()
    {
        std::memset(&dsp, 0, sizeof(dsp));
        dsp.Init(large.data(), large.size(), small.data(), small.size());
        dsp.set_playback_mode(clouds::PLAYBACK_MODE_GRANULAR);
        dsp.Prepare();
    }
};

struct TestPlayHead final : juce::AudioPlayHead
{
    PositionInfo position;
    juce::Optional<PositionInfo> getPosition() const override
    {
        return position;
    }
};

void recordedDrumTests(juce::File output)
{
    auto file = juce::File(juce::String(__FILE__))
                    .getSiblingFile("fixtures")
                    .getChildFile("sk5-16beat.wav");
    juce::AudioFormatManager formats;
    formats.registerBasicFormats();
    std::unique_ptr<juce::AudioFormatReader> reader(formats.createReaderFor(file));
    check(reader != nullptr, "CC0 recorded drum fixture");
    juce::AudioBuffer<float> recording(2, int(reader->lengthInSamples));
    check(reader->read(&recording, 0, recording.getNumSamples(), 0, true, true),
          "Recorded drum read");
    for (double bpm : {140., 175.})
        for (bool reversed : {false, true})
        {
            double rate = 48000;
            int pattern = int(std::round(16 * 60 * rate / bpm));
            auto signal = [&](int n)
            {
                double pos = double(n % pattern) * (recording.getNumSamples() - 1) / pattern;
                int index = int(pos);
                float f = float(pos - index);
                dubmist::Frame x{};
                for (int c = 0; c < 2; ++c)
                {
                    int channel = reader->numChannels == 1 ? 0 : c;
                    x[c] = .7f * (recording.getSample(channel, index) * (1 - f) +
                                  recording.getSample(
                                      channel, std::min(index + 1, recording.getNumSamples() - 1)) *
                                      f);
                }
                return x;
            };
            auto p = std::make_unique<DubmistProcessor>();
            dubmist::Settings s;
            s[dubmist::RhythmEnabled] = 1;
            s[dubmist::Sync] = 1;
            s[dubmist::Mix] = 1;
            s[dubmist::Reverse] = reversed ? 1.f : 0.f;
            p->apply(s);
            p->setRateAndBufferSizeDetails(rate, 127);
            p->prepareToPlay(rate, 127);
            TestPlayHead head;
            p->setPlayHead(&head);
            std::vector<float> audio(size_t(pattern) * 2), dry(size_t(pattern) * 2);
            juce::AudioBuffer<float> buffer(2, 127);
            juce::MidiBuffer midi;
            int request = int(8.37 * 60 * rate / bpm), actualRequest = -1;
            for (int start = 0; start < pattern; start += 127)
            {
                int count = std::min(127, pattern - start);
                buffer.setSize(2, count, false, false, true);
                if (start >= request && actualRequest < 0)
                {
                    s[dubmist::Repeat] = 1;
                    p->apply(s);
                    actualRequest = start;
                }
                double ppq = start * bpm / (60 * rate);
                head.position.setBpm(bpm);
                head.position.setPpqPosition(ppq);
                head.position.setTimeInSamples(start);
                head.position.setIsPlaying(true);
                head.position.setTimeSignature(juce::AudioPlayHead::TimeSignature{4, 4});
                head.position.setPpqPositionOfLastBarStart(std::floor(ppq / 4) * 4);
                for (int i = 0; i < count; ++i)
                {
                    auto x = signal(start + i);
                    for (int c = 0; c < 2; ++c)
                    {
                        buffer.setSample(c, i, x[c]);
                        dry[size_t(start + i) * 2 + size_t(c)] = x[c];
                    }
                }
                p->processBlock(buffer, midi);
                for (int i = 0; i < count; ++i)
                    for (int c = 0; c < 2; ++c)
                        audio[size_t(start + i) * 2 + size_t(c)] = buffer.getSample(c, i);
            }
            p->setPlayHead(nullptr);
            int capture =
                int(std::floor(std::ceil((actualRequest * bpm / (60 * rate) - 1.e-9) / .25) * .25 *
                                   60 * rate / bpm +
                               .5000001));
            int length = int(std::floor(.25 * 60 * rate / bpm + .5));
            int latency = p->getLatencySamples();
            for (int offset = 200; offset < length / 2; ++offset)
            {
                auto expected =
                    signal(capture - length + (reversed ? length - 1 - offset : offset));
                for (int c = 0; c < 2; ++c)
                    check(std::abs(audio[size_t(capture + latency + offset) * 2 + size_t(c)] -
                                   expected[c]) < 2.e-6f,
                          "Recorded drum sample order / plugin timing");
            }
            auto label = juce::String(int(bpm)) + juce::String(reversed ? "-reverse" : "-repeat");
            wav(output.getChildFile("SK5-" + label + ".wav"), audio, 48000);
            if (!reversed)
                wav(output.getChildFile("SK5-" + juce::String(int(bpm)) + "-input.wav"), dry,
                    48000);
        }
    std::cout << "PASS CC0 recorded SK5 loop: exact forward/reverse sample order through "
                 "host-clock VST processor at 140 and 175 BPM\n";
}
void originalReference()
{
    auto core = std::make_unique<dubmist::CloudsCore>();
    auto ref = std::make_unique<Reference>();
    std::array<clouds::ShortFrame, 32> input{}, output{}, expected{};
    uint32_t state = 33;
    for (int block = 0; block < 150; ++block)
    {
        for (int i = 0; i < 32; ++i)
        {
            auto x = source(block * 32 + i, 32000, 175);
            auto y = core->next(x);
            check(y[0] == expected[i].l / 32768.f && y[1] == expected[i].r / 32768.f,
                  "Original Clouds reference mismatch");
            input[i].l = short(x[0] * 32768);
            input[i].r = short(x[1] * 32768);
        }
        auto previous = stmlib::Random::state();
        stmlib::Random::Seed(state);
        ref->dsp.Prepare();
        auto *p = ref->dsp.mutable_parameters();
        p->position = .5f;
        p->size = .5f;
        p->density = .7f;
        p->texture = .5f;
        p->dry_wet = .5f;
        p->stereo_spread = .5f;
        ref->dsp.Process(input.data(), output.data(), 32);
        expected = output;
        state = stmlib::Random::state();
        stmlib::Random::Seed(previous);
    }
    std::cout << "PASS sample-identical original Clouds reference at native rate\n";
}
void freezeTest()
{
    auto a = std::make_unique<dubmist::CloudsCore>(), b = std::make_unique<dubmist::CloudsCore>();
    dubmist::Settings s;
    s[dubmist::Mix] = 1;
    s[dubmist::Density] = .85f;
    a->settings(s);
    b->settings(s);
    for (int n = 0; n < 64000; ++n)
    {
        auto x = source(n, 32000, 175);
        a->next(x);
        b->next(x);
    }
    s[dubmist::Freeze] = 1;
    a->settings(s);
    b->settings(s);
    double energy = 0, difference = 0;
    for (int n = 0; n < 64000; ++n)
    {
        auto x = a->next({{0, 0}}), y = b->next(source(n, 32000, 135));
        if (n > 3200)
        {
            for (int c = 0; c < 2; ++c)
            {
                energy += x[c] * x[c];
                difference += std::abs(x[c] - y[c]);
            }
        }
    }
    check(energy > .01, "Freeze lost captured audio");
    check(difference / 120000 < .0001, "Freeze kept recording new input");
    s[dubmist::Freeze] = 0;
    a->settings(s);
    double tail = 0;
    for (int n = 0; n < 128000; ++n)
    {
        auto x = a->next({{0, 0}});
        if (n > 96000)
            tail += x[0] * x[0] + x[1] * x[1];
    }
    check(tail < energy * .001, "Unfreeze did not resume capture");
    std::cout << "PASS freeze/live capture, held output energy=" << energy
              << " post-release=" << tail << "\n";
}
} // namespace
int main(int argc, char **argv)
{
    juce::ScopedJuceInitialiser_GUI gui;
    try
    {
        auto out = juce::File(argc > 1 ? juce::String(argv[1])
                                       : juce::File::getCurrentWorkingDirectory()
                                             .getChildFile("validation")
                                             .getFullPathName());
        out.createDirectory();
        originalReference();
        freezeTest();
        recordedDrumTests(out);
        auto p = std::make_unique<DubmistProcessor>(), copy = std::make_unique<DubmistProcessor>();
        auto legacyFile = juce::File(juce::String(__FILE__))
                              .getSiblingFile("fixtures")
                              .getChildFile("legacy-v1.dm98.xml")
                              .loadFileAsString();
        check(p->importPatch(legacyFile.toRawUTF8(), legacyFile.getNumBytesAsUTF8()),
              "Legacy schema 1 migration");
        check(p->snapshot()[dubmist::RhythmEnabled] == 0,
              "Legacy patches must retain Clouds processing");

        for (auto &preset : p->presets)
        {
            check(preset.settings.valid(), "Preset range");
            p->apply(preset.settings);
            auto text = p->exportPatch();
            check(copy->importPatch(text.toRawUTF8(), text.getNumBytesAsUTF8()), "Preset import");
            check(p->snapshot().values == copy->snapshot().values, "State configuration");
            auto x = render(*p, 48000, 127), y = render(*copy, 48000, 127);
            check(x == y, "Independent deterministic render");
            wav(out.getChildFile(juce::String(preset.name) + ".wav"), x, 48000);
            check(out.getChildFile(juce::String(preset.name) + ".dm98.xml").replaceWithText(text),
                  "Preset export");
            std::cout << "PRESET " << preset.name << "\n";
        }
        for (double rate : {44100., 48000., 88200., 96000.})
            for (int mode = 0; mode < 4; ++mode)
                for (int quality = 0; quality < 4; ++quality)
                {
                    dubmist::Settings s;
                    s[dubmist::Mode] = float(mode);
                    s[dubmist::Quality] = float(quality);
                    s[dubmist::Mix] = 1;
                    p->apply(s);
                    render(*p, rate, 257, 2, 135);
                }
        std::cout << "PASS four modes x four qualities x four host rates\n";
        dubmist::Settings s;
        p->apply(s);
        copy->apply(s);
        check(render(*p, 48000, 1, 2) == render(*copy, 48000, 257, 2),
              "Buffer partition dependence");
        s[dubmist::Bypass] = 1;
        for (double rate : {44100., 48000., 88200., 96000.})
        {
            p->apply(s);
            auto dry = render(*p, rate, 127, 1);
            int latency = p->getLatencySamples();
            for (int n = latency; n < int(rate); ++n)
            {
                auto expected = source(n - latency, rate, 175);
                for (int c = 0; c < 2; ++c)
                    check(std::abs(dry[size_t(n) * 2 + size_t(c)] - expected[c]) < 1.e-7,
                          "Bypass latency/channel error");
            }
        }
        // Mono input is duplicated before processing; never read an uninitialized
        // right input when the host negotiates a mono-to-stereo insert.
        p->apply(dubmist::Settings{});
        p->setPlayConfigDetails(1, 2, 48000, 127);
        p->prepareToPlay(48000, 127);
        juce::AudioBuffer<float> mono(2, 127);
        juce::MidiBuffer noMidi;
        for (int block = 0; block < 50; ++block)
        {
            mono.clear();
            for (int i = 0; i < 127; ++i)
            {
                mono.setSample(0, i, .1f * std::sin(float(block * 127 + i) * .03f));
                mono.setSample(1, i, std::numeric_limits<float>::quiet_NaN());
            }
            p->processBlock(mono, noMidi);
            for (int c = 0; c < 2; ++c)
                for (int i = 0; i < 127; ++i)
                    check(std::isfinite(mono.getSample(c, i)), "Mono input read unused channel");
        }
        p->setPlayConfigDetails(2, 2, 48000, 127);
        auto before = p->snapshot();
        const char *invalid = "<DUBMIST98_PATCH schema=\"99\"/>";
        check(!p->importPatch(invalid, strlen(invalid)), "Unknown schema");
        check(p->snapshot().values == before.values, "Invalid state mutation");
        auto capture = std::make_unique<dubmist::DesktopEngine>();
        dubmist::Settings captureSettings;
        captureSettings[dubmist::Mix] = 1;
        captureSettings[dubmist::Density] = .85f;
        capture->prepare(48000, captureSettings);
        std::vector<float> freezeAudio(48000 * 6 * 2);
        for (int n = 0; n < 48000 * 6; ++n)
        {
            if (n == 48000 * 2 || n == 48000 * 4)
            {
                captureSettings[dubmist::Freeze] = n == 48000 * 2 ? 1.f : 0.f;
                capture->settings(captureSettings);
            }
            auto y = capture->next(n < 48000 * 2 ? source(n, 48000, 175) : dubmist::Frame{{0, 0}});
            for (int c = 0; c < 2; ++c)
                freezeAudio[size_t(n) * 2 + size_t(c)] = y[c];
        }
        wav(out.getChildFile("Freeze-live-capture.wav"), freezeAudio, 48000);
        p->apply(p->presets[0].settings);
        juce::MemoryBlock state;
        p->getStateInformation(state);
        copy->setStateInformation(state.getData(), int(state.getSize()));
        check(p->snapshot().values == copy->snapshot().values, "Host state restore");
        std::unique_ptr<juce::AudioProcessorEditor> editor(p->createEditor());
        editor->setVisible(true);
        auto image = editor->createComponentSnapshot(editor->getLocalBounds(), true, 1);
        auto stream = out.getChildFile("dubmist-editor.png").createOutputStream();
        stream->setPosition(0);
        stream->truncate();
        juce::PNGImageFormat png;
        check(png.writeImageToStream(image, *stream), "Editor capture");
        for (auto *child : editor->getChildren())
            if (auto *button = dynamic_cast<juce::TextButton *>(child))
                if (button->getButtonText() == "Rhythm")
                    button->onClick();
        auto rhythmImage = editor->createComponentSnapshot(editor->getLocalBounds(), true, 1);
        auto rhythmStream = out.getChildFile("dubmist-rhythm.png").createOutputStream();
        rhythmStream->setPosition(0);
        rhythmStream->truncate();
        check(png.writeImageToStream(rhythmImage, *rhythmStream), "Rhythm editor capture");
        std::cout << "PASS state, rates, partitions, bypass and actual editor capture\n";
        return 0;
    }
    catch (const std::exception &e)
    {
        std::cerr << "FAIL " << e.what() << "\n";
        return 1;
    }
}
