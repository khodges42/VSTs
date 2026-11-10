// Copyright 2017 Julius Kammerl.
//
// Author: Julius Kammerl (julius@kammerl.com)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.
//
// See http://creativecommons.org/licenses/MIT/ for more information.
//
// Adapted for Dubmist host-rate playback from KammerlPlayer::Play's
// increasing/decreasing tape-speed interpolation. The original buffer/clock
// implementation is retained separately for provenance, not compiled wholesale.
#pragma once
#include <algorithm>
#include <cmath>
namespace dubmist
{
inline double kammerlTapeStep(double base, double ramp, double progress) noexcept
{
    progress = std::clamp(progress, 0., 1.);
    if (ramp < 0)
    {
        double pitch_parameter = std::exp2(ramp);
        return base * (1. - (1. - pitch_parameter) * progress);
    }
    double pitch_parameter = std::exp2(-ramp);
    double slice_remaining_percentage = 1. - progress;
    return base * std::exp2(ramp) * (1. - (1. - pitch_parameter) * slice_remaining_percentage);
}
} // namespace dubmist
