// SPDX-License-Identifier: LGPL-3.0-or-later
//! The plate model's events (plan §5.5; params documented in `crates/deeptime-format/EVENTS.md`).
//! Every event has a point place (the feature's centre, so the timelapse can mark it); plate ids
//! are params. Areas and lengths are on the real planet, speeds Earth-normalized (`normalise.rs`).

use deeptime_format::{Event, Place};
use glam::DVec3;

use crate::sphere::lat_lon_deg;
use crate::world::WorldState;

/// The source of every plate event.
pub const SOURCE: &str = "deeptime:plates";

pub const PLATE_BIRTH: &str = "deeptime:plate_birth";
pub const PLATE_DEATH: &str = "deeptime:plate_death";
pub const RIFT_ONSET: &str = "deeptime:rift_onset";
pub const RIDGE_BIRTH: &str = "deeptime:ridge_birth";
pub const SUBDUCTION_ONSET: &str = "deeptime:subduction_onset";
pub const COLLISION: &str = "deeptime:collision";
pub const TERRANE_ACCRETION: &str = "deeptime:terrane_accretion";
pub const OROGENY_PHASE: &str = "deeptime:orogeny_phase";
pub const HOTSPOT_TRACK: &str = "deeptime:hotspot_track";
pub const LARGE_IGNEOUS_PROVINCE: &str = "deeptime:large_igneous_province";

/// The point place of a direction.
pub fn point(p: DVec3) -> Place {
    let (lat, lon) = lat_lon_deg(p);
    Place::Point {
        lat: lat as f32,
        lon: lon as f32,
    }
}

/// A fresh plate event of `kind` at `t_myr` (Myr, relative to world creation) and point `at`.
pub fn event(world: &mut WorldState, t_myr: f64, kind: &str, at: DVec3) -> Event {
    world.event(t_myr * 1.0e6, kind, SOURCE).at(point(at))
}
