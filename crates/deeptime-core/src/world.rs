// SPDX-License-Identifier: LGPL-3.0-or-later
//! The shared state every stage reads and writes (plan §8 `WorldState`): the simulated clock, the
//! named fields stages exchange and keyframes are drawn from, the event log written as geology
//! (writer 1, plan §5.5) and the site registry (§5.6). It is what a checkpoint stores, so every
//! field serialises with postcard; stages add their own state here as they arrive (plates in
//! P1-03, the L1 surface in P1-04, …).

use std::collections::BTreeMap;

use deeptime_format::{Body, BodyId, Event, EventId, GEOLOGY_WRITER, GridDesc, Site, SiteId};
use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// A named f32 raster on a planet grid (row-major, `data[j · w + i]`, physical units).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub grid: GridDesc,
    pub data: Vec<f32>,
}

impl Field {
    /// A field of `grid` filled with `value`.
    pub fn filled(grid: GridDesc, value: f32) -> Field {
        let n = grid.cells();
        Field {
            grid,
            data: vec![value; n],
        }
    }

    /// Bytes a field of `grid` takes.
    pub fn bytes_for(grid: &GridDesc) -> u64 {
        grid.cells() as u64 * 4
    }

    /// The cell at (i, j), x wrapped, z clamped.
    pub fn get(&self, i: i64, j: i64) -> f32 {
        let i = self.grid.wrap_i(i) as usize;
        let j = self.grid.clamp_j(j) as usize;
        self.data[j * self.grid.w as usize + i]
    }
}

/// Everything the stages share.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldState {
    pub seed: u64,
    /// Circumference C in blocks.
    pub circumference: u32,
    /// Simulated time, years relative to world creation (negative = the geologic past).
    pub t_years: f64,
    /// Named fields (`elevation`, …) on planet grids.
    pub fields: BTreeMap<String, Field>,
    /// Events emitted so far, in emission order (the order `eventsSince` cursors count).
    pub events: Vec<Event>,
    pub sites: Vec<Site>,
    /// Deposit bodies and other geometry (plan §5.7.4; P1-14).
    pub bodies: Vec<Body>,
    /// Per-stage summaries as JSON text (the deposit census of P1-14, …), copied into
    /// `manifest.json` `notes` by `finalize_write`.
    pub notes: BTreeMap<String, String>,
    next_event_seq: u64,
    next_site_id: SiteId,
    next_body_id: BodyId,
}

impl WorldState {
    pub fn new(seed: u64, circumference: u32, t_years: f64) -> WorldState {
        WorldState {
            seed,
            circumference,
            t_years,
            fields: BTreeMap::new(),
            events: Vec::new(),
            sites: Vec::new(),
            bodies: Vec::new(),
            notes: BTreeMap::new(),
            next_event_seq: 0,
            next_site_id: 0,
            next_body_id: 0,
        }
    }

    /// A fresh event at `t` with the next geology id; finish it with the builder methods and
    /// pass it to [`WorldState::emit`].
    pub fn event(&mut self, t: f64, kind: &str, source: &str) -> Event {
        let id = EventId::new(GEOLOGY_WRITER, self.next_event_seq);
        self.next_event_seq += 1;
        Event::new(id, t, kind, source)
    }

    /// Appends `event` to the log after checking it; returns its id.
    pub fn emit(&mut self, event: Event) -> Result<EventId, CoreError> {
        event.validate()?;
        if event.id.writer() != GEOLOGY_WRITER || event.id.seq() >= self.next_event_seq {
            return Err(CoreError::InvalidArgument(format!(
                "event {} was not allocated by this world (use WorldState::event)",
                event.id
            )));
        }
        let id = event.id;
        self.events.push(event);
        Ok(id)
    }

    /// The next site id.
    pub fn next_site_id(&mut self) -> SiteId {
        let id = self.next_site_id;
        self.next_site_id += 1;
        id
    }

    /// Registers `site` (its id from [`WorldState::next_site_id`]).
    pub fn add_site(&mut self, site: Site) -> Result<SiteId, CoreError> {
        site.validate()?;
        if site.id >= self.next_site_id || self.sites.iter().any(|s| s.id == site.id) {
            return Err(CoreError::InvalidArgument(format!(
                "site {} was not allocated by this world, or is registered twice",
                site.id
            )));
        }
        let id = site.id;
        self.sites.push(site);
        Ok(id)
    }

    /// The next body id.
    pub fn next_body_id(&mut self) -> BodyId {
        let id = self.next_body_id;
        self.next_body_id += 1;
        id
    }

    /// Registers `body` (its id from [`WorldState::next_body_id`]).
    pub fn add_body(&mut self, body: Body) -> Result<BodyId, CoreError> {
        body.validate()?;
        if body.id >= self.next_body_id || self.bodies.iter().any(|b| b.id == body.id) {
            return Err(CoreError::InvalidArgument(format!(
                "body {} was not allocated by this world, or is registered twice",
                body.id
            )));
        }
        let id = body.id;
        self.bodies.push(body);
        Ok(id)
    }

    /// Block (x, z) of a latitude and longitude on this planet: plan §5.2's longitude
    /// 360°·(x + C/2)/C (the seam is 0°), so x = lon/360·C − C/2 (wrapped into [−C/2, C/2)) and
    /// z = −lat/90·C/4.
    pub fn block_of(&self, lat: f64, lon: f64) -> (i32, i32) {
        let c = f64::from(self.circumference);
        let x = (lon / 360.0 * c - c / 2.0).round() as i64;
        let half = i64::from(self.circumference / 2);
        let x = (x + half).rem_euclid(i64::from(self.circumference)) - half;
        let z = (-lat / 90.0 * c / 4.0).round() as i64;
        (x as i32, z as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_and_sites_get_fresh_ids() {
        let mut w = WorldState::new(1, 4096, -1.0e8);
        let a = w.event(-1.0e8, "deeptime:rift_onset", "deeptime:dummy");
        let b = w.event(-0.9e8, "deeptime:collision", "deeptime:dummy");
        let b_id = w.emit(b.caused_by(a.id)).unwrap();
        let a_id = w.emit(a).unwrap();
        assert_eq!((a_id.seq(), b_id.seq()), (0, 1));
        assert_eq!(w.events[0].causes, vec![a_id]);
        let forged = deeptime_format::Event::new(
            EventId::new(GEOLOGY_WRITER, 99),
            0.0,
            "deeptime:x",
            "deeptime:y",
        );
        assert!(w.emit(forged).is_err());
        let not_finite = w.event(f64::NAN, "deeptime:x", "deeptime:y");
        assert!(w.emit(not_finite).is_err());
        assert_eq!(w.block_of(0.0, 180.0), (0, 0));
        assert_eq!(w.block_of(0.0, 0.0), (-2048, 0), "the seam");
        assert_eq!(w.block_of(0.0, 360.0), (-2048, 0));
        assert_eq!(w.block_of(90.0, 90.0), (-1024, -1024));
        assert_eq!(w.next_site_id(), 0);
    }
}
