// SPDX-License-Identifier: LGPL-3.0-or-later
//! The session handle table (plan §6.1 "Handles are ids"). Java holds a `long` id, never a
//! pointer: a stale or forged id finds nothing and raises `IllegalStateException` instead of
//! dereferencing freed memory. Ids start at 1 and are never reused within a process.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use deeptime_core::session::Session;

use crate::errors::BridgeError;

/// A session plus the bridge's own flag: set when a call on it panicked (plan §6.3).
pub struct SessionEntry {
    pub session: Session,
    poisoned: AtomicBool,
}

impl SessionEntry {
    pub fn new(session: Session) -> Self {
        SessionEntry {
            session,
            poisoned: AtomicBool::new(false),
        }
    }

    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(Ordering::Acquire)
    }

    pub fn poison(&self) {
        self.poisoned.store(true, Ordering::Release);
    }
}

/// `RwLock<map<u64, Arc<T>>>` with a monotonic id counter.
pub struct Handles<T> {
    map: RwLock<BTreeMap<u64, Arc<T>>>,
    next: AtomicU64,
}

impl<T> Handles<T> {
    pub const fn new() -> Self {
        Handles {
            map: RwLock::new(BTreeMap::new()),
            next: AtomicU64::new(1),
        }
    }

    pub fn insert(&self, value: T) -> u64 {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        self.map
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, Arc::new(value));
        id
    }

    pub fn get(&self, id: u64) -> Result<Arc<T>, BridgeError> {
        self.map
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get(&id)
            .cloned()
            .ok_or(BridgeError::UnknownSession(id))
    }

    pub fn remove(&self, id: u64) -> Result<Arc<T>, BridgeError> {
        self.map
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&id)
            .ok_or(BridgeError::UnknownSession(id))
    }

    pub fn len(&self) -> usize {
        self.map.read().unwrap_or_else(|e| e.into_inner()).len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T> Default for Handles<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Every live session of this process.
pub static SESSIONS: Handles<SessionEntry> = Handles::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_never_reused_and_stale_ids_fail() {
        let table: Handles<u32> = Handles::new();
        let a = table.insert(7);
        let b = table.insert(8);
        assert_ne!(a, b);
        assert_eq!(*table.get(a).unwrap(), 7);
        table.remove(a).unwrap();
        assert!(matches!(table.get(a), Err(BridgeError::UnknownSession(id)) if id == a));
        assert!(matches!(
            table.remove(a),
            Err(BridgeError::UnknownSession(_))
        ));
        assert!(table.get(0xdead_beef).is_err());
        let c = table.insert(9);
        assert!(c > b);
        assert_eq!(table.len(), 2);
    }
}
