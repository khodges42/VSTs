// SPDX-License-Identifier: GPL-3.0-or-later
//! Audio publishes fixed atomics. Spectrum transforms and drawing belong to GUI.
use eis_core::{BANDS, Meter};
use std::sync::atomic::{AtomicU32, Ordering};
pub const FFT_SIZE: usize = 1024;
pub struct Shared {
    values: [AtomicU32; 18],
    pub rate: AtomicU32,
    samples: [AtomicU32; FFT_SIZE * 4],
    sequence: [AtomicU32; 2],
    published: AtomicU32,
}
impl Default for Shared {
    fn default() -> Self {
        let shared = Self {
            values: std::array::from_fn(|_| AtomicU32::new((-180_f32).to_bits())),
            rate: AtomicU32::new(48000_f32.to_bits()),
            samples: std::array::from_fn(|_| AtomicU32::new(0)),
            sequence: [AtomicU32::new(0), AtomicU32::new(0)],
            published: AtomicU32::new(0),
        };
        shared.publish(Meter {
            input: -180.,
            output: -180.,
            crossovers: [120., 2500.],
            ..Meter::default()
        });
        shared
    }
}
impl Shared {
    pub fn publish(&self, m: Meter) {
        let mut a = [
            m.input,
            m.output,
            m.compensation_db,
            m.safety_active as u8 as f32,
            m.crossovers[0],
            m.crossovers[1],
            0.,
            0.,
            0.,
            0.,
            0.,
            0.,
            0.,
            0.,
            0.,
            0.,
            0.,
            0.,
        ];
        a[6..6 + BANDS].copy_from_slice(&m.up);
        a[9..9 + BANDS].copy_from_slice(&m.down);
        a[12..12 + BANDS].copy_from_slice(&m.band_input);
        a[15..15 + BANDS].copy_from_slice(&m.band_output);
        for (v, x) in self.values.iter().zip(a) {
            v.store(x.to_bits(), Ordering::Relaxed);
        }
    }
    pub fn meter(&self) -> Meter {
        let a = self
            .values
            .each_ref()
            .map(|v| f32::from_bits(v.load(Ordering::Relaxed)));
        Meter {
            input: a[0],
            output: a[1],
            compensation_db: a[2],
            safety_active: a[3] > 0.,
            crossovers: [a[4], a[5]],
            up: std::array::from_fn(|i| a[6 + i]),
            down: std::array::from_fn(|i| a[9 + i]),
            band_input: std::array::from_fn(|i| a[12 + i]),
            band_output: std::array::from_fn(|i| a[15 + i]),
        }
    }
    pub fn snapshot(&self) -> Option<(u64, [[f32; 2]; FFT_SIZE])> {
        let b = self.published.load(Ordering::Acquire) as usize;
        let before = self.sequence[b].load(Ordering::Acquire);
        if before & 1 != 0 {
            return None;
        }
        let a = std::array::from_fn(|i| {
            std::array::from_fn(|ch| {
                f32::from_bits(self.samples[b * FFT_SIZE * 2 + i * 2 + ch].load(Ordering::Relaxed))
            })
        });
        std::sync::atomic::fence(Ordering::Acquire);
        if self.sequence[b].load(Ordering::Relaxed) != before {
            return None;
        }
        Some(((before as u64) << 1 | b as u64, a))
    }
}
#[derive(Default)]
pub struct Writer {
    buffer: usize,
    offset: usize,
}
impl Writer {
    pub fn push(&mut self, shared: &Shared, x: [f32; 2]) {
        if self.offset == 0 {
            shared.sequence[self.buffer].fetch_add(1, Ordering::AcqRel);
        }
        for (ch, v) in x.iter().enumerate() {
            shared.samples[self.buffer * FFT_SIZE * 2 + self.offset * 2 + ch].store(
                if v.is_finite() {
                    v.clamp(-32., 32.).to_bits()
                } else {
                    0
                },
                Ordering::Relaxed,
            );
        }
        self.offset += 1;
        if self.offset == FFT_SIZE {
            shared.sequence[self.buffer].fetch_add(1, Ordering::Release);
            shared
                .published
                .store(self.buffer as u32, Ordering::Release);
            self.buffer ^= 1;
            self.offset = 0;
        }
    }
}
/// GUI-side radix-2 FFT. Stereo mean power avoids cancelling antiphase inputs.
pub fn spectrum(x: [[f32; 2]; FFT_SIZE]) -> [f32; FFT_SIZE / 2] {
    let mut power = [0_f32; FFT_SIZE / 2];
    for ch in [0, 1] {
        let mut re = [0_f32; FFT_SIZE];
        let mut im = [0_f32; FFT_SIZE];
        for (i, frame) in x.iter().enumerate() {
            let j = i.reverse_bits() >> (usize::BITS - 10);
            re[j] = frame[ch]
                * (0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / (FFT_SIZE - 1) as f32).cos());
        }
        let mut n = 2;
        while n <= FFT_SIZE {
            let angle = -std::f32::consts::TAU / n as f32;
            let (wr, wi) = (angle.cos(), angle.sin());
            for base in (0..FFT_SIZE).step_by(n) {
                let (mut r, mut m) = (1., 0.);
                for j in 0..n / 2 {
                    let a = base + j;
                    let b = a + n / 2;
                    let tr = r * re[b] - m * im[b];
                    let ti = r * im[b] + m * re[b];
                    re[b] = re[a] - tr;
                    im[b] = im[a] - ti;
                    re[a] += tr;
                    im[a] += ti;
                    let next = r * wr - m * wi;
                    m = r * wi + m * wr;
                    r = next;
                }
            }
            n *= 2;
        }
        for i in 0..FFT_SIZE / 2 {
            power[i] += (re[i] * re[i] + im[i] * im[i]) * 0.5 / (FFT_SIZE as f32 * 0.25).powi(2);
        }
    }
    power.map(|v| 10. * v.max(1e-12).log10())
}
