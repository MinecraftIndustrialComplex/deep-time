// SPDX-License-Identifier: LGPL-3.0-or-later
//! What the live page sees of a run: frames, events, progress and status. The driving thread
//! publishes after every step; each stream connection keeps a [`Cursor`] and waits for news.
//! Everything published stays (frames by number, events in emission order), so a page that
//! connects late, reconnects or rewinds gets all of it.
//!
//! Events stay the session's own JSON text (`events_since`, one object each): parsing and
//! printing them again would move the last digit of some floats (serde_json's default float
//! parser is not round-trip exact), and the page should show exactly what the log holds.

use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::Duration;

use serde_json::{Value, json};

/// Events per `events` message.
const EVENTS_PER_MESSAGE: usize = 500;

/// A frame's size and layer count (the image route needs them without parsing the JSON).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameSize {
    pub w: u32,
    pub h: u32,
    pub layers: usize,
}

pub struct Live {
    run: String,
    meta: Value,
    inner: Mutex<Inner>,
    changed: Condvar,
}

struct Inner {
    frames: Vec<Value>,
    sizes: Vec<FrameSize>,
    /// One JSON object per event, as the session wrote it.
    events: Vec<String>,
    progress: Value,
    progress_seq: u64,
    status: Value,
    status_seq: u64,
    closing: bool,
}

/// How far one stream connection has read.
#[derive(Debug, Default, Clone, Copy)]
pub struct Cursor {
    frames: usize,
    events: usize,
    progress: u64,
    status: u64,
}

fn has_news(g: &Inner, c: &Cursor) -> bool {
    g.frames.len() > c.frames
        || g.events.len() > c.events
        || g.progress_seq != c.progress
        || g.status_seq != c.status
}

/// `[e0,e1,…]` from event objects' JSON text.
fn array(events: &[String]) -> String {
    format!("[{}]", events.join(","))
}

impl Live {
    /// `run` names this run in image URLs (a page left open across two runs on one port notices
    /// the change); `meta` is the page header (preset, stages, timeline, …).
    pub fn new(run: String, meta: Value) -> Live {
        Live {
            run,
            meta,
            inner: Mutex::new(Inner {
                frames: Vec::new(),
                sizes: Vec::new(),
                events: Vec::new(),
                progress: Value::Null,
                progress_seq: 0,
                status: json!({ "state": "running" }),
                status_seq: 0,
                closing: false,
            }),
            changed: Condvar::new(),
        }
    }

    /// A finished run read back from its folder: everything at once, nothing more to come.
    pub fn restored(
        run: String,
        meta: Value,
        frames: Vec<(Value, FrameSize)>,
        events: Vec<String>,
        progress: Value,
        status: Value,
    ) -> Live {
        let live = Live::new(run, meta);
        {
            let mut g = live.lock();
            for (f, size) in frames {
                g.frames.push(f);
                g.sizes.push(size);
            }
            g.events = events;
            g.progress = progress;
            g.status = status;
        }
        live
    }

    pub fn meta(&self) -> &Value {
        &self.meta
    }

    /// (frames, events, progress, status): the runs list's line.
    pub fn summary(&self) -> (usize, usize, Value, Value) {
        let g = self.lock();
        (
            g.frames.len(),
            g.events.len(),
            g.progress.clone(),
            g.status.clone(),
        )
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Adds what a step produced (`events`: one JSON object's text each) and wakes the streams.
    pub fn publish(
        &self,
        frames: Vec<(Value, FrameSize)>,
        events: Vec<String>,
        progress: Option<Value>,
    ) {
        if frames.is_empty() && events.is_empty() && progress.is_none() {
            return;
        }
        let mut g = self.lock();
        for (f, size) in frames {
            g.frames.push(f);
            g.sizes.push(size);
        }
        g.events.extend(events);
        if let Some(p) = progress {
            g.progress = p;
            g.progress_seq += 1;
        }
        drop(g);
        self.changed.notify_all();
    }

    /// `{"state": "running" | "finalizing" | "done" | "failed", …}`.
    pub fn set_status(&self, status: Value) {
        let mut g = self.lock();
        g.status = status;
        g.status_seq += 1;
        drop(g);
        self.changed.notify_all();
    }

    /// Ends every stream (each sends `bye`).
    pub fn close(&self) {
        self.lock().closing = true;
        self.changed.notify_all();
    }

    pub fn frame_size(&self, n: usize) -> Option<FrameSize> {
        self.lock().sizes.get(n).copied()
    }

    /// Everything so far as JSON text (the stream's `hello`, `/state.json`), and the cursor just
    /// after it.
    pub fn snapshot(&self) -> (String, Cursor) {
        let g = self.lock();
        let text = format!(
            "{{\"run\":{},\"meta\":{},\"frames\":{},\"events\":{},\"progress\":{},\"status\":{}}}",
            json!(self.run),
            self.meta,
            Value::from(g.frames.clone()),
            array(&g.events),
            g.progress,
            g.status
        );
        let c = Cursor {
            frames: g.frames.len(),
            events: g.events.len(),
            progress: g.progress_seq,
            status: g.status_seq,
        };
        (text, c)
    }

    /// The messages after `cur` as (event name, JSON text), waiting up to `wait` for one (an
    /// empty list when none came); `None` once the server is closing and the connection has
    /// everything. Within a batch, events come before frames (a step's events are known when its
    /// frame is drawn), then progress, then status (so `done` is the last word).
    pub fn next(&self, cur: &mut Cursor, wait: Duration) -> Option<Vec<(&'static str, String)>> {
        let g = self.lock();
        let (g, _) = self
            .changed
            .wait_timeout_while(g, wait, |g| !g.closing && !has_news(g, cur))
            .unwrap_or_else(|e| e.into_inner());
        let mut out = Vec::new();
        let mut from = cur.events;
        for chunk in g.events[cur.events..].chunks(EVENTS_PER_MESSAGE) {
            out.push((
                "events",
                format!("{{\"from\":{from},\"events\":{}}}", array(chunk)),
            ));
            from += chunk.len();
        }
        cur.events = g.events.len();
        out.extend(
            g.frames[cur.frames..]
                .iter()
                .map(|f| ("frame", f.to_string())),
        );
        cur.frames = g.frames.len();
        if g.progress_seq != cur.progress {
            out.push(("progress", g.progress.to_string()));
            cur.progress = g.progress_seq;
        }
        if g.status_seq != cur.status {
            out.push(("status", g.status.to_string()));
            cur.status = g.status_seq;
        }
        if out.is_empty() && g.closing {
            return None;
        }
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursors_see_everything_once_in_order() {
        let live = Live::new("r".into(), json!({}));
        let size = FrameSize {
            w: 2,
            h: 1,
            layers: 1,
        };
        live.publish(vec![(json!({"n": 0}), size)], vec!["1".into()], None);
        let (hello, mut early) = live.snapshot();
        let hello: Value = serde_json::from_str(&hello).unwrap();
        assert_eq!(hello["frames"].as_array().unwrap().len(), 1);
        assert_eq!(hello["events"], json!([1]));
        let mut late = Cursor::default();
        live.publish(
            vec![(json!({"n": 1}), size)],
            vec!["2".into(), "{\"x\":-56.781898498535156}".into()],
            Some(json!({"p": 1})),
        );
        live.set_status(json!({"state": "done"}));
        let a = live.next(&mut early, Duration::ZERO).unwrap();
        let kinds: Vec<&str> = a.iter().map(|(k, _)| *k).collect();
        assert_eq!(kinds, ["events", "frame", "progress", "status"]);
        assert_eq!(
            a[0].1, "{\"from\":1,\"events\":[2,{\"x\":-56.781898498535156}]}",
            "event text passes through untouched"
        );
        assert_eq!(a[1].1, "{\"n\":1}");
        let b = live.next(&mut late, Duration::ZERO).unwrap();
        assert_eq!(b.iter().filter(|(k, _)| *k == "frame").count(), 2);
        assert!(b[0].1.starts_with("{\"from\":0,\"events\":[1,2,"));
        // Nothing new: an empty batch after the wait; after close: None.
        assert!(
            live.next(&mut early, Duration::from_millis(10))
                .unwrap()
                .is_empty()
        );
        live.close();
        assert!(live.next(&mut early, Duration::from_secs(5)).is_none());
        assert_eq!(live.frame_size(1), Some(size));
        assert_eq!(live.frame_size(2), None);
    }
}
