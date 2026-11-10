// SPDX-License-Identifier: GPL-3.0-or-later
#include "Processor.h"
#include "Editor.h"
using namespace soundgyal;
juce::String SoundgyalProcessor::parameterID(size_t i)
{
    if (i < RouteBase)
        return baseSpecs[i].id;
    return "matrix" + juce::String(int((i - RouteBase) / 3 + 1)) +
           std::array<const char *, 3>{{"_source", "_target", "_depth"}}[(i - RouteBase) % 3];
}
juce::AudioProcessorValueTreeState::ParameterLayout SoundgyalProcessor::layout()
{
    juce::AudioProcessorValueTreeState::ParameterLayout out;
    for (size_t i = 0; i < paramCount; ++i)
    {
        float lo, hi, def;
        bool discrete;
        juce::String name;
        if (i < RouteBase)
        {
            auto s = baseSpecs[i];
            lo = s.min;
            hi = s.max;
            def = s.initial;
            discrete = s.discrete;
            name = s.name;
        }
        else
        {
            auto part = (i - RouteBase) % 3;
            lo = part == 2 ? -1.f : 0.f;
            hi = part == 0 ? 7.f : part == 1 ? 10.f : 1.f;
            def = 0;
            discrete = part != 2;
            name = "Matrix " + juce::String(int((i - RouteBase) / 3 + 1)) +
                   std::array<const char *, 3>{{" source", " target", " depth"}}[part];
        }
        juce::NormalisableRange<float> range(lo, hi, discrete ? 1.f : 0.f);
        if (i == Cutoff)
            range.setSkewForCentre(1000);
        if (i == Attack || i == AmpDecay || i == Release || i == ModAttack || i == ModDecay ||
            i == ModRelease || i == Glide)
            range.setSkewForCentre(.15f);
        out.add(std::make_unique<juce::AudioParameterFloat>(juce::ParameterID(parameterID(i), 1),
                                                            name, range, def));
    }
    return out;
}
SoundgyalProcessor::SoundgyalProcessor()
    : AudioProcessor(
          BusesProperties().withOutput("Stereo output", juce::AudioChannelSet::stereo(), true)),
      parameters(*this, nullptr, "SOUNDGYAL99", layout())
{
    for (size_t i = 0; i < paramCount; ++i)
        raw[i] = parameters.getRawParameterValue(parameterID(i));
}
Settings SoundgyalProcessor::snapshot() const noexcept
{
    Settings s;
    for (size_t i = 0; i < paramCount; ++i)
        s[i] = raw[i]->load(std::memory_order_relaxed);
    return s;
}
void SoundgyalProcessor::apply(const Settings &s)
{
    if (!s.valid())
        return;
    editVersion.fetch_add(1, std::memory_order_acq_rel);
    for (size_t i = 0; i < paramCount; ++i)
    {
        auto *p = parameters.getParameter(parameterID(i));
        p->beginChangeGesture();
        p->setValueNotifyingHost(p->convertTo0to1(s[i]));
        p->endChangeGesture();
    }
    editVersion.fetch_add(1, std::memory_order_release);
}
juce::String SoundgyalProcessor::exportPatch()
{
    auto xml = std::make_unique<juce::XmlElement>("SOUNDGYAL99_PATCH");
    xml->setAttribute("schema", 1);
    auto s = snapshot();
    for (size_t i = 0; i < paramCount; ++i)
    {
        auto *p = xml->createNewChildElement("PARAM");
        p->setAttribute("id", parameterID(i));
        p->setAttribute("value", double(s[i]));
    }
    return xml->toString();
}
bool SoundgyalProcessor::importPatch(const void *data, size_t bytes)
{
    if (bytes == 0 || bytes > 65536)
        return false;
    auto xml = juce::parseXML(juce::String::fromUTF8(static_cast<const char *>(data), int(bytes)));
    if (!xml || !xml->hasTagName("SOUNDGYAL99_PATCH") || xml->getIntAttribute("schema") != 1 ||
        xml->getNumChildElements() != int(paramCount))
        return false;
    Settings s;
    std::array<bool, paramCount> seen{};
    for (auto *child = xml->getFirstChildElement(); child; child = child->getNextElement())
    {
        if (!child->hasTagName("PARAM") || !child->hasAttribute("value"))
            return false;
        auto id = child->getStringAttribute("id");
        size_t index = paramCount;
        for (size_t i = 0; i < paramCount; ++i)
            if (id == parameterID(i))
            {
                index = i;
                break;
            }
        if (index == paramCount || seen[index])
            return false;
        seen[index] = true;
        auto text = child->getStringAttribute("value");
        char *end = nullptr;
        auto string = text.toStdString();
        double value = std::strtod(string.c_str(), &end);
        if (end != string.c_str() + string.size() || end == string.c_str() || !std::isfinite(value))
            return false;
        s[index] = float(value);
    }
    if (!s.valid())
        return false;
    apply(s);
    return true;
}
void SoundgyalProcessor::getStateInformation(juce::MemoryBlock &dest)
{
    auto text = exportPatch();
    dest.replaceAll(text.toRawUTF8(), text.getNumBytesAsUTF8());
}
void SoundgyalProcessor::setStateInformation(const void *data, int size)
{
    if (size > 0)
        importPatch(data, size_t(size));
}
bool SoundgyalProcessor::isBusesLayoutSupported(const BusesLayout &b) const
{
    return b.getMainOutputChannelSet() == juce::AudioChannelSet::stereo() &&
           b.getMainInputChannelSet().isDisabled();
}
void SoundgyalProcessor::prepareToPlay(double sr, int)
{
    if (!std::isfinite(sr) || sr < 32000 || sr > 192000)
    {
        engine.reset();
        setLatencySamples(0);
        return;
    }
    engine = std::make_unique<DesktopEngine>();
    engine->prepare(sr);
    last = snapshot();
    engine->synth().settings(last);
    setLatencySamples(engine->latency(sr));
}
void SoundgyalProcessor::processBlock(juce::AudioBuffer<float> &buffer, juce::MidiBuffer &midi)
{
    juce::ScopedNoDenormals noDenormals;
    buffer.clear();
    if (!engine)
        return;
    unsigned r = read.load(std::memory_order_relaxed), w = write.load(std::memory_order_acquire);
    while (r != w)
    {
        auto event = auditionQueue[r++ % 64];
        if (event.velocity > 0)
            engine->synth().noteOn(0, event.note, event.velocity);
        else
            engine->synth().noteOff(0, event.note);
    }
    read.store(r, std::memory_order_release);
    if (auditionOverflow.exchange(false, std::memory_order_acq_rel))
        engine->synth().controller(0, 123, 0);
    auto version = editVersion.load(std::memory_order_acquire);
    if (!(version & 1))
    {
        auto s = snapshot();
        if (version == editVersion.load(std::memory_order_acquire) && s.valid())
        {
            last = s;
            engine->synth().settings(s);
        }
    }
    if (auto *playHead = getPlayHead())
        if (auto pos = playHead->getPosition())
            if (auto bpm = pos->getBpm())
                engine->synth().tempo(float(*bpm));
    auto it = midi.cbegin();
    float maximum = 0;
    for (int frame = 0; frame < buffer.getNumSamples(); ++frame)
    {
        while (it != midi.cend() && (*it).samplePosition <= frame)
        {
            auto metadata = *it++;
            if (metadata.numBytes > 3 || metadata.data[0] >= 0xf0)
                continue;
            auto msg = metadata.getMessage();
            auto &synth = engine->synth();
            int ch = msg.getChannel() - 1;
            if (msg.isNoteOn())
            {
                synth.noteOn(ch, msg.getNoteNumber(), msg.getFloatVelocity());
                midiLight.store(8, std::memory_order_relaxed);
            }
            else if (msg.isNoteOff())
                synth.noteOff(ch, msg.getNoteNumber());
            else if (msg.isPitchWheel())
                synth.pitchBend(ch, (msg.getPitchWheelValue() - 8192) / 8192.f);
            else if (msg.isController())
                synth.controller(ch, msg.getControllerNumber(), msg.getControllerValue() / 127.f);
        }
        auto x = engine->next();
        for (int c = 0; c < std::min(2, buffer.getNumChannels()); ++c)
        {
            buffer.setSample(c, frame, x[c]);
            maximum = std::max(maximum, std::abs(x[c]));
        }
    }
    midi.clear();
    peak.store(maximum, std::memory_order_relaxed);
    voiceCount.store(engine->synth().activeVoices(), std::memory_order_relaxed);
}
juce::AudioProcessorEditor *SoundgyalProcessor::createEditor()
{
    return new SoundgyalEditor(*this);
}
juce::AudioProcessor *JUCE_CALLTYPE createPluginFilter()
{
    return new SoundgyalProcessor();
}
