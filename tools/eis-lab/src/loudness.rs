// SPDX-License-Identifier: GPL-3.0-or-later
//! Offline 48 kHz stereo BS.1770-style integrated estimate. Not a certified meter.
//! K-weighting coefficients and 400 ms / 75% overlap gates: ITU-R BS.1770-4,
//! Annex 1, Tables 1/2. No measurement code is linked into the audio processor.
#[derive(Clone, Copy, Default)]
struct Biquad {
    s1: f64,
    s2: f64,
}
impl Biquad {
    fn process(&mut self, x: f64, b: [f64; 3], a: [f64; 2]) -> f64 {
        let y = b[0] * x + self.s1;
        self.s1 = b[1] * x - a[0] * y + self.s2;
        self.s2 = b[2] * x - a[1] * y;
        y
    }
}
pub fn integrated(audio: &[Vec<f32>]) -> Option<f64> {
    if audio.len() != 2 || audio[0].len() != audio[1].len() || audio[0].len() < 19200 {
        return None;
    }
    let mut filters = [[Biquad::default(); 2]; 2];
    let mut prefix = Vec::with_capacity(audio[0].len() + 1);
    prefix.push(0_f64);
    for (left, right) in audio[0].iter().zip(&audio[1]) {
        let mut energy = 0.;
        for (sample, filter) in [left, right].iter().zip(filters.iter_mut()) {
            let x = filter[0].process(
                **sample as f64,
                [1.53512485958697, -2.69169618940638, 1.19839281085285],
                [-1.69065929318241, 0.73248077421585],
            );
            let x = filter[1].process(x, [1., -2., 1.], [-1.99004745483398, 0.99007225036621]);
            energy += x * x;
        }
        prefix.push(prefix.last().unwrap() + energy);
    }
    let absolute = 10_f64.powf((-70. + 0.691) / 10.);
    let blocks: Vec<_> = (0..=audio[0].len() - 19200)
        .step_by(4800)
        .map(|i| (prefix[i + 19200] - prefix[i]) / 19200.)
        .filter(|e| *e > absolute)
        .collect();
    if blocks.is_empty() {
        return None;
    }
    let preliminary = blocks.iter().sum::<f64>() / blocks.len() as f64;
    let threshold = absolute.max(preliminary * 0.1);
    let eligible: Vec<_> = blocks.into_iter().filter(|e| *e > threshold).collect();
    if eligible.is_empty() {
        None
    } else {
        Some(-0.691 + 10. * (eligible.iter().sum::<f64>() / eligible.len() as f64).log10())
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn silence_and_one_khz_stereo_level() {
        let silence = vec![vec![0.; 48000]; 2];
        assert!(super::integrated(&silence).is_none());
        let tone: Vec<_> = (0..48000)
            .map(|i| (std::f64::consts::TAU * 1000. * i as f64 / 48000.).sin() as f32 * 0.1)
            .collect();
        let level = super::integrated(&[tone.clone(), tone]).unwrap();
        assert!((level + 20.).abs() < 0.1, "{level}");
    }
}
