// SPDX-License-Identifier: GPL-3.0-or-later
//! Fixed 4x halfband path: interpolate twice, shape, decimate twice. 24 frames.
use crate::LATENCY;
#[derive(Clone, Copy)]
pub struct Delay {
    data: [[f32; 2]; LATENCY],
    cursor: usize,
}
impl Default for Delay {
    fn default() -> Self {
        Self {
            data: [[0.; 2]; LATENCY],
            cursor: 0,
        }
    }
}
impl Delay {
    pub fn process(&mut self, x: [f32; 2]) -> [f32; 2] {
        let y = self.data[self.cursor];
        self.data[self.cursor] = x;
        self.cursor = (self.cursor + 1) % LATENCY;
        y
    }
}
#[derive(Clone, Copy)]
struct Fir {
    data: [[f32; 33]; 2],
    cursor: usize,
}
impl Default for Fir {
    fn default() -> Self {
        Self {
            data: [[0.; 33]; 2],
            cursor: 0,
        }
    }
}
impl Fir {
    fn process(&mut self, x: [f32; 2]) -> [f32; 2] {
        // Same original project halfband coefficients as VIXEN, with local storage.
        const SIDE: [f32; 8] = [
            -0.0018878784,
            0.0038624783,
            -0.008242467,
            0.015947115,
            -0.02867656,
            0.05071856,
            -0.09801591,
            0.31594175,
        ];
        let mut y = [0.; 2];
        for ch in 0..2 {
            self.data[ch][self.cursor] = x[ch];
            y[ch] = self.data[ch][(self.cursor + 17) % 33] * 0.5007058;
            for (i, c) in SIDE.iter().enumerate() {
                let a = i * 2 + 1;
                let b = 32 - a;
                y[ch] += (self.data[ch][(self.cursor + 33 - a) % 33]
                    + self.data[ch][(self.cursor + 33 - b) % 33])
                    * c;
            }
        }
        self.cursor = (self.cursor + 1) % 33;
        y
    }
}
#[derive(Clone, Copy, Default)]
pub struct Saturator {
    up1: Fir,
    up2: Fir,
    down2: Fir,
    down1: Fir,
}
impl Saturator {
    pub fn process(&mut self, x: [f32; 2], drive: f32) -> [f32; 2] {
        let mut result = [0.; 2];
        let g = 1. + 24. * drive * drive;
        let compensation = g.sqrt() / g;
        for half in 0..2 {
            let u = self.up1.process(if half == 0 {
                x.map(|v| v * 2.)
            } else {
                [0.; 2]
            });
            let mut middle = [0.; 2];
            for sub in 0..2 {
                let u = self
                    .up2
                    .process(if sub == 0 { u.map(|v| v * 2.) } else { [0.; 2] });
                let shaped = u.map(|v| {
                    if drive < 1e-6 {
                        v
                    } else {
                        (v * g).tanh() * compensation
                    }
                });
                let down = self.down2.process(shaped);
                if sub == 0 {
                    middle = down;
                }
            }
            let down = self.down1.process(middle);
            if half == 0 {
                result = down;
            }
        }
        result
    }
}
