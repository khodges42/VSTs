// SPDX-License-Identifier: GPL-3.0-or-later
//! Fixed-size state fields: no strings, heap replacement or locks on restoration.
use nih_plug::params::persist::PersistentField;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};
type TextWords = [[u32; 8]; 32];
#[derive(Clone)]
pub struct TextField(Arc<[AtomicU32; 256]>);
impl TextField {
    pub fn new(text: &str) -> Self {
        let s = Self(Arc::new(std::array::from_fn(|_| AtomicU32::new(0))));
        s.set_text(text);
        s
    }
    pub fn set_text(&self, text: &str) {
        let mut bytes = [0_u8; 1024];
        let mut n = text.len().min(1024);
        while !text.is_char_boundary(n) {
            n -= 1;
        }
        bytes[..n].copy_from_slice(&text.as_bytes()[..n]);
        for (a, b) in self.0.iter().zip(bytes.chunks_exact(4)) {
            a.store(
                u32::from_le_bytes([b[0], b[1], b[2], b[3]]),
                Ordering::Relaxed,
            );
        }
    }
    pub fn get(&self) -> String {
        let mut bytes = [0_u8; 1024];
        for (b, a) in bytes.chunks_exact_mut(4).zip(self.0.iter()) {
            b.copy_from_slice(&a.load(Ordering::Relaxed).to_le_bytes());
        }
        let n = bytes.iter().position(|x| *x == 0).unwrap_or(1024);
        String::from_utf8_lossy(&bytes[..n]).into_owned()
    }
}
impl<'a> PersistentField<'a, TextWords> for TextField {
    fn set(&self, v: TextWords) {
        for (i, a) in self.0.iter().enumerate() {
            a.store(v[i / 8][i % 8], Ordering::Relaxed);
        }
    }
    fn map<F, R>(&self, f: F) -> R
    where
        F: Fn(&TextWords) -> R,
    {
        let words = std::array::from_fn(|i| {
            std::array::from_fn(|j| self.0[i * 8 + j].load(Ordering::Relaxed))
        });
        f(&words)
    }
}
