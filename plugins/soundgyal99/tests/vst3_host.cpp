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
int main(int argc, char **argv)
{
    juce::ScopedJuceInitialiser_GUI gui;
    try
    {
        check(argc >= 2, "Usage: soundgyal_host absolute-plugin-bundle [capture.png]");
        juce::AudioPluginFormatManager manager;
        manager.addFormat(new juce::VST3PluginFormat());
        juce::OwnedArray<juce::PluginDescription> types;
        manager.getFormat(0)->findAllTypesForFile(types, juce::String(argv[1]));
        check(types.size() == 1, "VST3 discovery");
        juce::String error;
        auto plugin = manager.createPluginInstance(*types[0], 44100, 127, error);
        check(plugin != nullptr, error.toRawUTF8());
        plugin->setPlayConfigDetails(0, 2, 44100, 127);
        plugin->prepareToPlay(44100, 127);
        check(plugin->getParameters().size() >= 60, "Parameter discovery");
        for (auto *param : plugin->getParameters())
            if (param->getName(128) == "Harmonics")
                param->setValueNotifyingHost(.72f);
        juce::MemoryBlock state;
        plugin->getStateInformation(state);
        check(state.getSize() > 0, "State save");
        juce::AudioBuffer<float> buffer(2, 127);
        juce::MidiBuffer midi;
        midi.ensureSize(1024);
        double energy = 0;
        float peak = 0;
        for (int block = 0; block < 700; ++block)
        {
            buffer.clear();
            midi.clear();
            if (block == 0)
                midi.addEvent(juce::MidiMessage::noteOn(1, 36, .8f), 33);
            if (block == 180)
                midi.addEvent(juce::MidiMessage::noteOff(1, 36), 91);
            if (block == 220)
                midi.addEvent(juce::MidiMessage::allNotesOff(1), 0);
            plugin->processBlock(buffer, midi);
            if (block == 0)
                for (int i = 0; i < 33; ++i)
                    check(buffer.getSample(0, i) == 0, "Premature MIDI audio");
            for (int c = 0; c < 2; ++c)
                for (int i = 0; i < 127; ++i)
                {
                    float x = buffer.getSample(c, i);
                    check(std::isfinite(x), "VST3 nonfinite");
                    energy += x * x;
                    peak = std::max(peak, std::abs(x));
                    if (block > 650)
                        check(std::abs(x) < 1.e-7f, "VST3 stuck voice");
                }
        }
        check(energy > .01 && peak < 2, "VST3 audio range");
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
        restored->releaseResources();
        plugin->releaseResources();
        std::cout << "PASS actual VST3 discovery, MIDI offsets, stereo audio, note release, "
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
