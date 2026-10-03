// SPDX-License-Identifier: LGPL-3.0-or-later
//! Site records and "Why here?" causes (plan §5.5, §5.6; decisions log Q13).
//!
//! **Causes.** Every deposit's forming event names the events that made its mineral-system
//! components: for each component the type lists event kinds (`causes.source`, `.driver`, …), and
//! the engine takes, per component, the event of those kinds nearest the site in space and time —
//! within the type's `cause_radius_km` (block metric), starting no later than the formation and,
//! unless the type dates itself by its cause, still active or recent (100 Myr) then. Events are
//! matched by their point places (the plates stage stores region masks from P1-03 on; masks will
//! join the lookup then). A deposit whose components find nothing falls back to the nearest
//! geology event before it, so the chain is never empty while the log has any event nearby.

use deeptime_format::{Event, EventId, Place, Site};

use super::cluster::distance;

/// Point-placed events, indexed for the cause lookup.
pub struct CauseIndex {
    entries: Vec<Entry>,
    c: f64,
}

struct Entry {
    id: EventId,
    kind: String,
    t0: f64,
    t1: f64,
    x: f64,
    z: f64,
}

/// Block (x, z) of a latitude and longitude (`deeptime_format::projection`).
pub fn block_of(c: f64, lat: f64, lon: f64) -> (f64, f64) {
    deeptime_format::Projection::of(c).block_of(lat, lon)
}

/// Latitude and longitude (degrees; the seam is 0°) of block (x, z).
pub fn lat_lon_of(c: f64, x: f64, z: f64) -> [f32; 2] {
    let (lat, lon) = deeptime_format::Projection::of(c).lat_lon(x, z);
    [lat as f32, lon as f32]
}

/// How far back an event may have ended and still count as a cause (years).
pub const LOOKBACK_YEARS: f64 = 1.0e8;

impl CauseIndex {
    pub fn new(c: u32, events: &[Event], sites: &[Site]) -> CauseIndex {
        let c = f64::from(c);
        let entries = events
            .iter()
            .filter_map(|e| {
                let (lat, lon) = match e.place {
                    Place::Point { lat, lon } => (f64::from(lat), f64::from(lon)),
                    Place::Site { site } => {
                        let s = sites.iter().find(|s| s.id == site)?;
                        (f64::from(s.lat_lon[0]), f64::from(s.lat_lon[1]))
                    }
                    _ => return None,
                };
                let (x, z) = block_of(c, lat, lon);
                Some(Entry {
                    id: e.id,
                    kind: e.kind.clone(),
                    t0: e.t_start,
                    t1: e.t_end,
                    x,
                    z,
                })
            })
            .collect();
        CauseIndex { entries, c }
    }

    /// Adds an event this stage emitted (so later deposits can cite it).
    pub fn push(&mut self, e: &Event, x: f64, z: f64) {
        self.entries.push(Entry {
            id: e.id,
            kind: e.kind.clone(),
            t0: e.t_start,
            t1: e.t_end,
            x,
            z,
        });
    }

    /// The best event of `kinds` for a deposit at block (x, z) formed at `t`: within
    /// `radius_blocks`, started by `t + slack`, and (unless `any_time`) ended no earlier than
    /// `t − LOOKBACK_YEARS`. Ranked by distance / radius + time gap / 100 Myr.
    #[allow(clippy::too_many_arguments)]
    pub fn find(
        &self,
        kinds: &[String],
        x: f64,
        z: f64,
        t: f64,
        radius_blocks: f64,
        slack: f64,
        any_time: bool,
    ) -> Option<EventId> {
        let mut best: Option<(f64, EventId)> = None;
        for e in &self.entries {
            if !kinds.contains(&e.kind) || e.t0 > t + slack {
                continue;
            }
            if !any_time && e.t1 < t - LOOKBACK_YEARS {
                continue;
            }
            let d = distance(self.c, (x, z), (e.x, e.z));
            if d > radius_blocks {
                continue;
            }
            let gap = if t < e.t0 {
                e.t0 - t
            } else if t > e.t1 {
                t - e.t1
            } else {
                0.0
            };
            let score = d / radius_blocks.max(1.0) + if any_time { 0.0 } else { gap / 1.0e8 };
            if best.is_none_or(|b| score < b.0) {
                best = Some((score, e.id));
            }
        }
        best.map(|b| b.1)
    }

    /// The nearest event of any kind that started before `t`, within `radius_blocks`.
    pub fn nearest_before(&self, x: f64, z: f64, t: f64, radius_blocks: f64) -> Option<EventId> {
        self.entries
            .iter()
            .filter(|e| e.t0 <= t)
            .map(|e| (distance(self.c, (x, z), (e.x, e.z)), e.id))
            .filter(|(d, _)| *d <= radius_blocks)
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, id)| id)
    }

    /// The start time of an indexed event.
    pub fn start_of(&self, id: EventId) -> Option<f64> {
        self.entries.iter().find(|e| e.id == id).map(|e| e.t0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ev(seq: u64, kind: &str, t0: f64, t1: f64, lat: f32, lon: f32) -> Event {
        Event::new(EventId::new(1, seq), t0, kind, "deeptime:test")
            .until(t1)
            .at(Place::Point { lat, lon })
    }

    #[test]
    fn nearest_active_event_of_the_right_kind() {
        let c = 4096u32;
        let events = vec![
            ev(0, "deeptime:subduction_onset", -3.2e8, 0.0, 10.0, 60.0),
            ev(1, "deeptime:subduction_onset", -3.2e8, 0.0, 10.0, 200.0),
            ev(2, "deeptime:collision", -3.5e8, -3.4e8, 10.0, 61.0),
            ev(3, "deeptime:orogeny_phase", -1.0e8, -0.9e8, 10.0, 60.5),
        ];
        let idx = CauseIndex::new(c, &events, &[]);
        let (x, z) = block_of(4096.0, 10.0, 62.0);
        let kinds = vec!["deeptime:subduction_onset".to_string()];
        assert_eq!(
            idx.find(&kinds, x, z, -1.0e8, 200.0, 0.0, false),
            Some(EventId::new(1, 0))
        );
        // Formed before the subduction started: not a cause.
        assert_eq!(idx.find(&kinds, x, z, -4.0e8, 200.0, 0.0, false), None);
        // A collision that ended 240 Myr before is too old, unless the type dates by its cause.
        let col = vec!["deeptime:collision".to_string()];
        assert_eq!(idx.find(&col, x, z, -1.0e8, 200.0, 0.0, false), None);
        assert_eq!(
            idx.find(&col, x, z, -1.0e8, 200.0, 0.0, true),
            Some(EventId::new(1, 2))
        );
        assert_eq!(
            idx.nearest_before(x, z, -2.0e8, 200.0),
            Some(EventId::new(1, 2))
        );
        let back = lat_lon_of(4096.0, x, z);
        assert!((back[0] - 10.0).abs() < 1e-4 && (back[1] - 62.0).abs() < 1e-3);
    }
}
