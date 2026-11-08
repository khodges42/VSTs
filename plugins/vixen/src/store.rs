// SPDX-License-Identifier: GPL-3.0-or-later
use crossbeam_queue::ArrayQueue;
use nih_plug::params::persist::PersistentField;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
use vixen_core::{Patch, Synth, factory};
struct Inner {
    document: Mutex<Patch>,
    pending: ArrayQueue<Synth>,
    revision: AtomicU64,
    error: Mutex<String>,
}
#[derive(Clone)]
pub struct PatchStore(Arc<Inner>);
impl Default for PatchStore {
    fn default() -> Self {
        Self(Arc::new(Inner {
            document: Mutex::new(factory::patch(0)),
            pending: ArrayQueue::new(8),
            revision: AtomicU64::new(0),
            error: Mutex::new(String::new()),
        }))
    }
}
impl PatchStore {
    pub fn read(&self) -> Patch {
        self.0
            .document
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
    pub fn revision(&self) -> u64 {
        self.0.revision.load(Ordering::Acquire)
    }
    pub fn pop(&self) -> Option<Synth> {
        self.0.pending.pop()
    }
    pub fn error(&self) -> String {
        self.0
            .error
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
    pub fn publish(&self, patch: Patch) -> Result<(), String> {
        patch.validate()?;
        // Serialize publishers only, never the consumer. UI/state calls own strings.
        let mut document = self.0.document.lock().unwrap_or_else(|e| e.into_inner());
        self.0.pending.force_push(patch.synth);
        *document = patch;
        self.0.revision.fetch_add(1, Ordering::Release);
        Ok(())
    }
}
impl<'a> PersistentField<'a, Patch> for PatchStore {
    fn set(&self, patch: Patch) {
        let error = self.publish(patch).err().unwrap_or_default();
        *self.0.error.lock().unwrap_or_else(|e| e.into_inner()) = error;
    }
    fn map<F, R>(&self, f: F) -> R
    where
        F: Fn(&Patch) -> R,
    {
        f(&self.0.document.lock().unwrap_or_else(|e| e.into_inner()))
    }
}
