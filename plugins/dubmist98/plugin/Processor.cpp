// SPDX-License-Identifier: GPL-3.0-or-later
#include "Processor.h"
#include "Editor.h"
using namespace dubmist;
juce::AudioProcessorValueTreeState::ParameterLayout DubmistProcessor::layout()
{
    juce::AudioProcessorValueTreeState::ParameterLayout out;
    for (auto s : specs)
    {
        juce::NormalisableRange<float> range(s.min, s.max, s.discrete ? 1.f : 0.f);
        out.add(std::make_unique<juce::AudioParameterFloat>(juce::ParameterID(s.id, 1), s.name,
                                                            range, s.initial));
    }
    return out;
}
DubmistProcessor::DubmistProcessor()
    : AudioProcessor(BusesProperties()
                         .withInput("Stereo input", juce::AudioChannelSet::stereo(), true)
                         .withOutput("Stereo output", juce::AudioChannelSet::stereo(), true)),
      parameters(*this, nullptr, "DUBMIST98", layout())
{
    for (size_t i = 0; i < Count; ++i)
        raw_[i] = parameters.getRawParameterValue(specs[i].id);
}
Settings DubmistProcessor::snapshot() const noexcept
{
    Settings s;
    for (size_t i = 0; i < Count; ++i)
        s[i] = raw_[i]->load(std::memory_order_relaxed);
    return s;
}
void DubmistProcessor::apply(const Settings &s)
{
    if (!s.valid())
        return;
    generation_.fetch_add(1, std::memory_order_acq_rel);
    for (size_t i = 0; i < Count; ++i)
    {
        auto *p = parameters.getParameter(specs[i].id);
        p->beginChangeGesture();
        p->setValueNotifyingHost(p->convertTo0to1(s[i]));
        p->endChangeGesture();
    }
    generation_.fetch_add(1, std::memory_order_release);
}
juce::String DubmistProcessor::exportPatch()
{
    auto root = std::make_unique<juce::XmlElement>("DUBMIST98_PATCH");
    root->setAttribute("schema", 2);
    auto s = snapshot();
    s[Clear] = s[Trigger] = 0;
    for (size_t i = 0; i < Count; ++i)
    {
        auto *p = root->createNewChildElement("PARAM");
        p->setAttribute("id", specs[i].id);
        p->setAttribute("value", double(s[i]));
    }
    return root->toString();
}
bool DubmistProcessor::importPatch(const void *data, size_t size)
{
    if (!size || size > 16384)
        return false;
    auto xml = juce::parseXML(juce::String::fromUTF8(static_cast<const char *>(data), int(size)));
    if (!xml || !xml->hasTagName("DUBMIST98_PATCH") ||
        (xml->getIntAttribute("schema") != 1 && xml->getIntAttribute("schema") != 2) ||
        xml->getNumChildElements() !=
            int(xml->getIntAttribute("schema") == 1 ? legacyCount : Count))
        return false;
    Settings s;
    bool legacy = xml->getIntAttribute("schema") == 1;
    std::array<bool, Count> seen{};
    for (auto *c = xml->getFirstChildElement(); c; c = c->getNextElement())
    {
        if (!c->hasTagName("PARAM") || !c->hasAttribute("value"))
            return false;
        size_t index = Count;
        for (size_t i = 0; i < Count; ++i)
            if (c->getStringAttribute("id") == specs[i].id)
            {
                index = i;
                break;
            }
        if (index == Count || seen[index] || (legacy && index >= legacyCount))
            return false;
        seen[index] = true;
        auto text = c->getStringAttribute("value").toStdString();
        char *end = nullptr;
        double v = std::strtod(text.c_str(), &end);
        if (end != text.c_str() + text.size() || end == text.c_str() || !std::isfinite(v))
            return false;
        s[index] = float(v);
    }
    if (!s.valid() || s[Clear] != 0 || s[Trigger] != 0 || (legacy && s[Mode] > 3))
        return false;
    apply(s);
    return true;
}
void DubmistProcessor::getStateInformation(juce::MemoryBlock &b)
{
    auto s = exportPatch();
    b.replaceAll(s.toRawUTF8(), s.getNumBytesAsUTF8());
}
void DubmistProcessor::setStateInformation(const void *p, int n)
{
    if (n > 0)
        importPatch(p, size_t(n));
}
void DubmistProcessor::prepareToPlay(double rate, int)
{
    if (!std::isfinite(rate) || rate < 32000 || rate > 192000)
    {
        engine_.reset();
        setLatencySamples(0);
        return;
    }
    engine_ = std::make_unique<DesktopEngine>();
    last_ = snapshot();
    if (!last_.valid())
        last_ = Settings{};
    engine_->prepare(rate, last_);
    setLatencySamples(engine_->latency());
}
void DubmistProcessor::process(juce::AudioBuffer<float> &b, bool forceBypass)
{
    juce::ScopedNoDenormals noDenormals;
    if (!engine_)
    {
        b.clear();
        return;
    }
    unsigned v = generation_.load(std::memory_order_acquire);
    if (!(v & 1))
    {
        auto s = snapshot();
        if (v == generation_.load(std::memory_order_acquire) && s.valid())
            last_ = s;
    }
    auto s = last_;
    if (forceBypass)
        s[Bypass] = 1;
    engine_->settings(s);
    Transport transport;
    if (auto *head = getPlayHead())
        if (auto pos = head->getPosition())
        {
            auto bpm = pos->getBpm();
            auto ppq = pos->getPpqPosition();
            if (bpm && ppq)
            {
                transport.valid = true;
                transport.bpm = *bpm;
                transport.ppq = *ppq;
                transport.playing = pos->getIsPlaying();
            }
            if (auto sample = pos->getTimeInSamples())
            {
                transport.sample = *sample;
                transport.sampleValid = true;
            }
            if (auto bar = pos->getPpqPositionOfLastBarStart())
            {
                transport.barStart = *bar;
                transport.barValid = true;
            }
            if (auto signature = pos->getTimeSignature())
                transport.beatsPerBar = signature->numerator * 4. / signature->denominator;
        }
    engine_->beginBlock(transport, b.getNumSamples());

    float ip = 0, op = 0;
    for (int n = 0; n < b.getNumSamples(); ++n)
    {
        Frame x{{b.getSample(0, n), b.getSample(getTotalNumInputChannels() == 1 ? 0 : 1, n)}};
        for (float sample : x)
            ip = std::max(ip, std::abs(sample));
        auto y = engine_->next(x);
        for (int c = 0; c < std::min(2, b.getNumChannels()); ++c)
        {
            b.setSample(c, n, y[c]);
            op = std::max(op, std::abs(y[c]));
        }
    }
    rhythmActive.store(engine_->rhythm().active(), std::memory_order_relaxed);
    clockAvailable.store(engine_->rhythm().clockAvailable(), std::memory_order_relaxed);
    inputPeak.store(ip, std::memory_order_relaxed);
    outputPeak.store(op, std::memory_order_relaxed);
}
juce::AudioProcessorEditor *DubmistProcessor::createEditor()
{
    return new DubmistEditor(*this);
}
juce::AudioProcessor *JUCE_CALLTYPE createPluginFilter()
{
    return new DubmistProcessor();
}
