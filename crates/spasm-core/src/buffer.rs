// SPDX-License-Identifier: GPL-3.0-or-later
//! Prepared banks. Valid counters make logical clearing constant-time.
pub struct Bank {
    data: Vec<[f32; 2]>,
    pub written: u64,
    pub valid: usize,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Slice {
    pub bank: usize,
    pub start: u64,
    pub len: usize,
    pub global_end: u64,
}
impl Bank {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![[0.; 2]; capacity.max(32)],
            written: 0,
            valid: 0,
        }
    }
    pub fn reset(&mut self) {
        self.written = 0;
        self.valid = 0;
    }
    pub fn write(&mut self, x: [f32; 2]) {
        let index = (self.written % self.data.len() as u64) as usize;
        self.data[index] = x;
        self.written = self.written.wrapping_add(1);
        self.valid = (self.valid + 1).min(self.data.len());
    }
    pub fn slice(&self, bank: usize, limit: usize, global_end: u64) -> Slice {
        let len = self.valid.min(limit);
        Slice {
            bank,
            start: self.written - len as u64,
            len,
            global_end,
        }
    }
    pub fn read(&self, s: Slice, position: f64) -> [f32; 2] {
        if s.len == 0
            || !position.is_finite()
            || s.start < self.written.saturating_sub(self.valid as u64)
            || s.start
                .checked_add(s.len as u64)
                .is_none_or(|end| end > self.written)
        {
            return [0.; 2];
        }
        let position = position.clamp(0., (s.len - 1) as f64);
        let i = position.floor() as usize;
        let j = (i + 1).min(s.len - 1);
        let t = position.fract() as f32;
        let a = self.data[((s.start + i as u64) % self.data.len() as u64) as usize];
        let b = self.data[((s.start + j as u64) % self.data.len() as u64) as usize];
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
    }
    pub fn bytes(&self) -> usize {
        self.data.capacity() * std::mem::size_of::<[f32; 2]>()
    }
}
