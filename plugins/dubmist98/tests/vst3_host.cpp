// SPDX-License-Identifier: GPL-3.0-or-later
// Offline development host: no audio device, no DAWJ dependency.
#include <juce_audio_utils/juce_audio_utils.h>
#include <iostream>
#include <stdexcept>
#define NOMINMAX
#include <windows.h>
void check(bool b, const char *m)
{
    if (!b)
        throw std::runtime_error(m);
}
struct HostClock final : juce::AudioPlayHead
{
    PositionInfo value;
    juce::Optional<PositionInfo> getPosition() const override
    {
        return value;
    }
};
int main(int argc, char **argv)
{
    juce::ScopedJuceInitialiser_GUI gui;
    try
    {
        check(argc >= 2, "Usage: dubmist_host absolute-plugin-bundle [capture.png]");
        juce::AudioPluginFormatManager manager;
        manager.addFormat(new juce::VST3PluginFormat());
        juce::OwnedArray<juce::PluginDescription> types;
        manager.getFormat(0)->findAllTypesForFile(types, juce::String(argv[1]));
        check(types.size() == 1, "VST3 discovery");
        juce::String error;
        auto plugin = manager.createPluginInstance(*types[0], 44100, 127, error);
        check(plugin != nullptr, error.toRawUTF8());
        plugin->setPlayConfigDetails(2, 2, 44100, 127);
        plugin->prepareToPlay(44100, 127);
        check(plugin->getParameters().size() >= 16, "Parameter discovery");
        for (auto *param : plugin->getParameters())
            if (param->getName(128) == "Grain position")
                param->setValueNotifyingHost(.72f);
        for (auto *param : plugin->getParameters())
        {
            auto name = param->getName(128);
            if (name == "Rhythmic slice mode" || name == "Repeat hold" || name == "Wet / dry")
                param->setValueNotifyingHost(1);
        }
        HostClock clock;
        plugin->setPlayHead(&clock);
        juce::MemoryBlock state;
        plugin->getStateInformation(state);
        check(state.getSize() > 0, "State save");
        juce::AudioBuffer<float> buffer(2, 127);
        juce::MidiBuffer midi;
        double energy = 0, difference = 0;
        float peak = 0;
        for (int block = 0; block < 1000; ++block)
        {
            int start = block * 127;
            clock.value.setBpm(175);
            clock.value.setPpqPosition(start * 175. / (60 * 44100));
            clock.value.setTimeInSamples(start);
            clock.value.setIsPlaying(true);
            clock.value.setTimeSignature(juce::AudioPlayHead::TimeSignature{4, 4});
            for (int n = 0; n < 127; ++n)
            {
                int sample = block * 127 + n;
                buffer.setSample(0, n, .2f * std::sin(float(sample) * .031f));
                buffer.setSample(1, n, .15f * std::sin(float(sample) * .047f));
            }
            plugin->processBlock(buffer, midi);
            for (int n = 0; n < 127; ++n)
            {
                int sample = block * 127 + n, offset = sample - 3780 - plugin->getLatencySamples();
                if (offset >= 600 && offset < 1000)
                {
                    check(std::abs(buffer.getSample(0, n) - .2f * std::sin(float(offset) * .031f)) <
                              2.e-6f,
                          "Actual VST3 transport/repeat sample alignment");
                    check(std::abs(buffer.getSample(1, n) -
                                   .15f * std::sin(float(offset) * .047f)) < 2.e-6f,
                          "Actual VST3 stereo timing");
                }
            }
            for (int c = 0; c < 2; ++c)
                for (int n = 0; n < 127; ++n)
                {
                    float x = buffer.getSample(c, n);
                    check(std::isfinite(x), "VST3 nonfinite");
                    energy += x * x;
                    peak = std::max(peak, std::abs(x));
                }
            for (int n = 0; n < 127; ++n)
                difference += std::abs(buffer.getSample(0, n) - buffer.getSample(1, n));
        }
        check(energy > .01 && peak < 2 && difference > 1, "VST3 stereo processing");
        auto restored = manager.createPluginInstance(*types[0], 48000, 64, error);
        check(restored != nullptr, "Second instance");
        restored->setStateInformation(state.getData(), int(state.getSize()));
        for (int i = 0; i < plugin->getParameters().size(); ++i)
            check(std::abs(plugin->getParameters()[i]->getValue() -
                           restored->getParameters()[i]->getValue()) < 1.e-5f,
                  "VST3 state restore");
        if (argc > 2)
        {
            std::unique_ptr<juce::AudioProcessorEditor> editor(plugin->createEditorIfNeeded());
            check(editor != nullptr, "VST3 editor");
            editor->addToDesktop(juce::ComponentPeer::windowIsTemporary);
            editor->setVisible(true);
            juce::MessageManager::getInstance()->runDispatchLoopUntil(150);
            HWND window = static_cast<HWND>(editor->getPeer()->getNativeHandle());
            RECT rect{};
            GetClientRect(window, &rect);
            int width = rect.right, height = rect.bottom;
            HDC dc = GetDC(window), memory = CreateCompatibleDC(dc);
            BITMAPINFO info{};
            info.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
            info.bmiHeader.biWidth = width;
            info.bmiHeader.biHeight = -height;
            info.bmiHeader.biPlanes = 1;
            info.bmiHeader.biBitCount = 32;
            info.bmiHeader.biCompression = BI_RGB;
            void *pixels = nullptr;
            HBITMAP bitmap = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &pixels, nullptr, 0);
            auto old = SelectObject(memory, bitmap);
            RedrawWindow(window, nullptr, nullptr,
                         RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN);
            BOOL captured = PrintWindow(window, memory, PW_CLIENTONLY);
            juce::Image image(juce::Image::RGB, width, height, true);
            auto *bytes = static_cast<unsigned char *>(pixels);
            if (captured)
                for (int y = 0; y < height; ++y)
                    for (int x = 0; x < width; ++x)
                    {
                        auto i = (y * width + x) * 4;
                        image.setPixelAt(x, y, juce::Colour(bytes[i + 2], bytes[i + 1], bytes[i]));
                    }
            SelectObject(memory, old);
            DeleteObject(bitmap);
            DeleteDC(memory);
            ReleaseDC(window, dc);
            editor->removeFromDesktop();
            check(captured != 0, "Native editor capture");
            juce::PNGImageFormat png;
            auto stream = juce::File(juce::String(argv[2])).createOutputStream();
            if (stream)
            {
                stream->setPosition(0);
                stream->truncate();
            }
            check(stream && png.writeImageToStream(image, *stream), "VST3 screenshot");
        }
        plugin->setPlayHead(nullptr);
        restored->releaseResources();
        plugin->releaseResources();
        std::cout << "PASS actual VST3 host-clock repeat timing, stereo effect processing, "
                     "parameters, independent state restore, editor capture; energy="
                  << energy << " peak=" << peak << "\n";
        return 0;
    }
    catch (const std::exception &e)
    {
        std::cerr << "FAIL " << e.what() << "\n";
        return 1;
    }
}
