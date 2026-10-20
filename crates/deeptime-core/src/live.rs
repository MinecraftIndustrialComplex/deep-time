// SPDX-License-Identifier: LGPL-3.0-or-later
//! The live map (docs/design/create-world-ux.md, "The map while the run goes"): while the game's
//! Create World screen follows a run, the session draws the one layer the player views a few
//! times a minute between the keyframes, into a single frame that the next one replaces. It is
//! not a keyframe: it is not kept, not numbered among them, never written to disk and cannot be
//! scrubbed to, stopped at or picked as an epoch. The keyframe grid (`keyframes.every_myr`, Ben's
//! 50 Myr of 2026-10-06) stays exactly as it is.
//!
//! It is drawn by the keyframes' own code ([`crate::keyframes::render`] of one layer at the
//! keyframes' size), so a live frame is the picture a keyframe of that layer would be at that
//! instant. Drawing reads the world and changes nothing in it: a run with live frames ends with
//! the same world data, byte for byte, as one without (`tests/live_map.rs`).
//!
//! Pacing: a frame is drawn after a step, only while someone watches (the game renews its
//! request, [`LiveMap::watch`], on every poll; a request lapses after [`LIVE_LEASE`]), and only
//! when the time since the last one ended is at least [`LivePacing::min_gap`] and at least the
//! last frame's drawing time × (1 − share) / share, so drawing takes at most `share` of the run's
//! wall time (2 % by default) however slow a frame is. A new layer is drawn at once.
//!
//! Threading: the stepping thread draws (in the session's pool) and swaps the frame in under a
//! short lock; the game's threads ask and copy at any time and never wait for a step.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use deeptime_format::keyframes::KeyframeImage;
use serde_json::json;

use crate::error::CoreError;

/// How long one [`LiveMap::watch`] request holds: the game asks again on every poll, so a screen
/// that closed or scrubbed back stops the drawing within this.
pub const LIVE_LEASE: Duration = Duration::from_secs(2);

/// When live frames are drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LivePacing {
    /// The least wall time from the end of one live frame to the start of the next.
    pub min_gap: Duration,
    /// The most of the run's wall time live frames may take: after a frame that took `d`, the
    /// next waits `d · (1 − share) / share` (`1.0`: no wait beyond `min_gap`).
    pub share: f64,
    /// The picture's size; `None`: the keyframes' (`keyframes.size`).
    pub size: Option<(u32, u32)>,
}

impl Default for LivePacing {
    /// At most four frames a second and 2 % of the run's time, at the keyframes' size
    /// (game-choice: the coordinator's call of 2026-10-20, the overhead target of the brief).
    fn default() -> LivePacing {
        LivePacing {
            min_gap: Duration::from_millis(250),
            share: 0.02,
            size: None,
        }
    }
}

impl LivePacing {
    /// A frame after every step (tests, and `deeptime run --live-every-step` to time a frame).
    pub const EVERY_STEP: LivePacing = LivePacing {
        min_gap: Duration::ZERO,
        share: 1.0,
        size: None,
    };

    /// The wait after a frame that took `took`.
    fn gap(&self, took: Duration) -> Duration {
        let share = self.share.clamp(1e-3, 1.0);
        let wait = took.mul_f64((1.0 - share) / share);
        wait.max(self.min_gap)
    }
}

/// One live frame: one layer, RGBA8, row 0 north, column 0 the seam (as every keyframe layer).
#[derive(Debug, Clone, PartialEq)]
pub struct LiveFrame {
    /// Counts the live frames of the session from 1 (a new number is a new picture).
    pub seq: u64,
    /// Simulated time, years relative to world creation (negative = the past).
    pub t: f64,
    /// The stage (group) that was stepping.
    pub stage: String,
    pub layer: String,
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

impl LiveFrame {
    /// `{"seq", "t", "stage", "layer", "w", "h"}`: what `copyLiveFrame` returns beside the pixels.
    pub fn info_json(&self) -> String {
        json!({
            "seq": self.seq, "t": self.t, "stage": self.stage, "layer": self.layer,
            "w": self.w, "h": self.h,
        })
        .to_string()
    }
}

/// What drawing live frames cost.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LiveStats {
    pub frames: u64,
    /// Wall milliseconds spent drawing, in all and the longest frame.
    pub millis: f64,
    pub max_ms: f64,
}

impl LiveStats {
    fn add(&mut self, ms: f64) {
        self.frames += 1;
        self.millis += ms;
        self.max_ms = self.max_ms.max(ms);
    }
}

struct Clock {
    /// The layer of the last frame (or of the keyframe that stood in for one).
    layer: Option<String>,
    /// No frame before this.
    next_at: Option<Instant>,
    total: LiveStats,
    /// Since the last [`LiveMap::take_stage_stats`].
    stage: LiveStats,
}

/// The session's live map: the request, the newest frame and the pacing.
pub(crate) struct LiveMap {
    request: Mutex<Option<(String, Instant)>>,
    frame: Mutex<Option<Arc<LiveFrame>>>,
    seq: AtomicU64,
    pacing: Mutex<LivePacing>,
    clock: Mutex<Clock>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    // Plain data: a panic elsewhere leaves nothing half-written that matters here.
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl LiveMap {
    pub(crate) fn new() -> LiveMap {
        LiveMap {
            request: Mutex::new(None),
            frame: Mutex::new(None),
            seq: AtomicU64::new(0),
            pacing: Mutex::new(LivePacing::default()),
            clock: Mutex::new(Clock {
                layer: None,
                next_at: None,
                total: LiveStats::default(),
                stage: LiveStats::default(),
            }),
        }
    }

    /// Asks for live frames of `layer` for the next [`LIVE_LEASE`] (`None`: no more); returns
    /// the newest frame's number (0: none yet).
    pub(crate) fn watch(&self, layer: Option<&str>, now: Instant) -> u64 {
        *lock(&self.request) = layer
            .filter(|l| !l.is_empty())
            .map(|l| (l.to_owned(), now + LIVE_LEASE));
        self.seq.load(Ordering::Acquire)
    }

    pub(crate) fn set_pacing(&self, pacing: LivePacing) {
        *lock(&self.pacing) = pacing;
    }

    pub(crate) fn pacing(&self) -> LivePacing {
        *lock(&self.pacing)
    }

    /// The layer to draw now, if a frame is due.
    pub(crate) fn due(&self, now: Instant) -> Option<String> {
        let layer = match &*lock(&self.request) {
            Some((layer, until)) if now < *until => layer.clone(),
            _ => return None,
        };
        let clock = lock(&self.clock);
        let fresh = clock.layer.as_deref() != Some(layer.as_str());
        (fresh || clock.next_at.is_none_or(|at| now >= at)).then_some(layer)
    }

    /// A keyframe was drawn at this very instant: it is the newest picture, and the next live
    /// frame waits as if it had been one.
    pub(crate) fn stood_in(&self, layer: &str, now: Instant) {
        let gap = self.pacing().min_gap;
        let mut clock = lock(&self.clock);
        clock.layer = Some(layer.to_owned());
        clock.next_at = Some(now + gap);
    }

    /// A frame of `layer` was drawn (`None`: the layer had nothing to show) in `took`.
    pub(crate) fn drawn(
        &self,
        layer: &str,
        image: Option<KeyframeImage>,
        took: Duration,
        now: Instant,
    ) {
        let gap = self.pacing().gap(took);
        {
            let mut clock = lock(&self.clock);
            clock.layer = Some(layer.to_owned());
            clock.next_at = Some(now + gap);
            let ms = took.as_secs_f64() * 1000.0;
            clock.total.add(ms);
            clock.stage.add(ms);
        }
        let Some(mut image) = image else {
            return;
        };
        let Some((name, rgba)) = image.layers.pop() else {
            return;
        };
        let seq = self.seq.load(Ordering::Acquire) + 1;
        let frame = LiveFrame {
            seq,
            t: image.t,
            stage: image.stage,
            layer: name,
            w: image.w,
            h: image.h,
            rgba,
        };
        // The old frame's pixels are freed here, outside the lock (a reader may still hold them).
        let old = lock(&self.frame).replace(Arc::new(frame));
        self.seq.store(seq, Ordering::Release);
        drop(old);
    }

    /// The newest frame, if one was drawn.
    pub(crate) fn latest(&self) -> Option<Arc<LiveFrame>> {
        lock(&self.frame).clone()
    }

    /// The newest frame's pixels into `dst` (at least `w·h·4` bytes) and its
    /// [`LiveFrame::info_json`]; `None` before the first frame.
    pub(crate) fn copy(&self, dst: &mut [u8]) -> Result<Option<String>, CoreError> {
        let Some(frame) = self.latest() else {
            return Ok(None);
        };
        if dst.len() < frame.rgba.len() {
            return Err(CoreError::InvalidArgument(format!(
                "the buffer holds {} bytes, the live frame needs {} ({}×{}×4)",
                dst.len(),
                frame.rgba.len(),
                frame.w,
                frame.h
            )));
        }
        dst[..frame.rgba.len()].copy_from_slice(&frame.rgba);
        Ok(Some(frame.info_json()))
    }

    /// What the frames cost since the last call (a stage's share, for the log).
    pub(crate) fn take_stage_stats(&self) -> LiveStats {
        std::mem::take(&mut lock(&self.clock).stage)
    }

    /// What the frames cost in all.
    pub(crate) fn stats(&self) -> LiveStats {
        lock(&self.clock).total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image(t: f64, layer: &str) -> KeyframeImage {
        KeyframeImage {
            n: 0,
            t,
            stage: "plates".into(),
            w: 2,
            h: 1,
            layers: vec![(layer.to_owned(), vec![1, 2, 3, 4, 5, 6, 7, 8])],
        }
    }

    #[test]
    fn nothing_is_drawn_unless_watched_and_a_request_lapses() {
        let live = LiveMap::new();
        let t0 = Instant::now();
        assert_eq!(live.due(t0), None);
        assert_eq!(live.watch(Some("satellite"), t0), 0);
        assert_eq!(live.due(t0).as_deref(), Some("satellite"));
        assert_eq!(live.due(t0 + LIVE_LEASE), None, "the request lapsed");
        live.watch(Some("satellite"), t0);
        live.watch(None, t0);
        assert_eq!(live.due(t0), None, "the game stopped watching");
        live.watch(Some(""), t0);
        assert_eq!(live.due(t0), None);
    }

    #[test]
    fn the_wait_after_a_frame_keeps_drawing_within_its_share() {
        let live = LiveMap::new();
        let t0 = Instant::now();
        live.watch(Some("satellite"), t0);
        // 100 ms at 2 %: the next frame waits 4.9 s.
        live.drawn(
            "satellite",
            Some(image(-5e8, "satellite")),
            Duration::from_millis(100),
            t0,
        );
        live.watch(Some("satellite"), t0 + Duration::from_secs(1));
        assert_eq!(live.due(t0 + Duration::from_millis(4_800)), None);
        assert_eq!(
            live.due(t0 + Duration::from_millis(4_900)).as_deref(),
            Some("satellite")
        );
        // A fast frame waits the least gap.
        live.drawn("satellite", None, Duration::from_millis(1), t0);
        assert_eq!(live.due(t0 + Duration::from_millis(200)), None);
        assert!(live.due(t0 + Duration::from_millis(250)).is_some());
        // Another layer is drawn at once.
        live.drawn("satellite", None, Duration::from_millis(100), t0);
        live.watch(Some("elevation"), t0);
        assert_eq!(live.due(t0).as_deref(), Some("elevation"));
        let s = live.stats();
        assert_eq!(s.frames, 3);
        assert!((s.millis - 201.0).abs() < 1e-6 && (s.max_ms - 100.0).abs() < 1e-6);
        assert_eq!(live.take_stage_stats().frames, 3);
        assert_eq!(live.take_stage_stats().frames, 0);
        assert_eq!(live.stats().frames, 3, "the total stays");
    }

    #[test]
    fn a_keyframe_at_the_same_instant_stands_in_for_a_frame() {
        let live = LiveMap::new();
        let t0 = Instant::now();
        live.watch(Some("satellite"), t0);
        live.stood_in("satellite", t0);
        assert_eq!(live.due(t0 + Duration::from_millis(100)), None);
        assert!(live.due(t0 + Duration::from_millis(250)).is_some());
        assert_eq!(live.stats().frames, 0);
        assert!(live.latest().is_none());
    }

    #[test]
    fn the_newest_frame_replaces_the_last_and_copies_with_its_info() {
        let live = LiveMap::new();
        let t0 = Instant::now();
        let mut dst = [0u8; 8];
        assert_eq!(live.copy(&mut dst).unwrap(), None);
        live.drawn(
            "satellite",
            Some(image(-4e8, "satellite")),
            Duration::ZERO,
            t0,
        );
        live.drawn(
            "satellite",
            Some(image(-3e8, "satellite")),
            Duration::ZERO,
            t0,
        );
        assert_eq!(live.watch(Some("satellite"), t0), 2);
        let info: serde_json::Value =
            serde_json::from_str(&live.copy(&mut dst).unwrap().unwrap()).unwrap();
        assert_eq!(info["seq"], 2);
        assert_eq!(info["t"], -3e8);
        assert_eq!(info["layer"], "satellite");
        assert_eq!((info["w"].as_u64(), info["h"].as_u64()), (Some(2), Some(1)));
        assert_eq!(dst, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(live.copy(&mut [0u8; 7]).is_err(), "a short buffer");
    }
}
