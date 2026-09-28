// SPDX-License-Identifier: LGPL-3.0-or-later
//! A bounded ring of native log lines. Java drains it (`drainLog`) into its own logger, so native
//! diagnostics land in `latest.log` in order with Java's (plan §6.3). The core never prints.

use std::collections::VecDeque;
use std::sync::Mutex;

/// Severity of a native log line. Java maps it onto its own logger levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
}

impl Level {
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
        }
    }
}

/// Lines kept before the oldest are dropped.
pub const LOG_RING_CAPACITY: usize = 1024;

#[derive(Default)]
struct Inner {
    lines: VecDeque<String>,
    dropped: u64,
}

/// Bounded, thread-safe log ring. Each line is `LEVEL<TAB>message`, with any newline in the
/// message replaced by ` | ` so one entry stays one line.
pub struct LogRing {
    capacity: usize,
    inner: Mutex<Inner>,
}

impl Default for LogRing {
    fn default() -> Self {
        Self::new(LOG_RING_CAPACITY)
    }
}

impl LogRing {
    pub fn new(capacity: usize) -> Self {
        LogRing {
            capacity: capacity.max(1),
            inner: Mutex::new(Inner::default()),
        }
    }

    pub fn push(&self, level: Level, message: impl AsRef<str>) {
        let line = format!(
            "{}\t{}",
            level.as_str(),
            message.as_ref().replace(['\r', '\n'], " | ")
        );
        // A panic elsewhere must not silence logging: recover a poisoned lock.
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        if inner.lines.len() == self.capacity {
            inner.lines.pop_front();
            inner.dropped += 1;
        }
        inner.lines.push_back(line);
    }

    /// Empties the ring. A `WARN` line reporting how many lines were dropped comes first when the
    /// ring overflowed since the last drain.
    pub fn drain(&self) -> Vec<String> {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let mut out = Vec::with_capacity(inner.lines.len() + 1);
        if inner.dropped > 0 {
            out.push(format!(
                "WARN\tnative log ring overflowed: {} older lines dropped",
                inner.dropped
            ));
            inner.dropped = 0;
        }
        out.extend(inner.lines.drain(..));
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drains_in_order_and_reports_overflow() {
        let ring = LogRing::new(2);
        ring.push(Level::Info, "a");
        ring.push(Level::Warn, "b\nc");
        ring.push(Level::Error, "d");
        let lines = ring.drain();
        assert_eq!(
            lines,
            vec![
                "WARN\tnative log ring overflowed: 1 older lines dropped".to_string(),
                "WARN\tb | c".to_string(),
                "ERROR\td".to_string(),
            ]
        );
        assert!(ring.drain().is_empty());
    }
}
