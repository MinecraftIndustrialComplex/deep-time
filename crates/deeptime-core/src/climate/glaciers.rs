// SPDX-License-Identifier: LGPL-3.0-or-later
//! Glaciers from the equilibrium-line altitude (plan §8.3 item 6).
//!
//! goSPL's documentation states the rule ELA(x) = z(x) + (T(x) − T_ELA)/Γ (implemented here from
//! that statement, not from goSPL's code, which is GPL): a cell is glaciated when it lies above its
//! ELA, i.e. when its temperature is below T_ELA. Deep Time takes T as the mean of the three
//! warmest consecutive months at the surface (summer temperature decides whether snow survives)
//! and T_ELA from precipitation, by the relation Ohmura, Kasser & Funk 1992 fitted at the ELAs of
//! 70 glaciers: P = 645 + 296·T + 9·T² (P annual precipitation in mm, T the summer temperature in
//! °C), solved for T (a wet glacier survives warmer summers: −2.4 °C at P = 0, +1.2 °C at 1,000 mm,
//! +6.6 °C at 3,000 mm). ELA above sea level = (T_summer at sea level − T_ELA)/Γ.
//!
//! A **glaciation** (event `deeptime:glaciation`) starts at the first snapshot where glaciers
//! cover at least `glaciation_on_frac` of the land and ends at the first where they cover less than
//! `glaciation_off_frac` (or at the end of the run, `ongoing` = 1).

use deeptime_format::Place;
use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::world::WorldState;

/// Event source of the climate stage.
pub const SOURCE: &str = "deeptime:climate";

/// Summer temperature at the ELA for annual precipitation `p_mm` (Ohmura et al. 1992), °C.
pub fn ela_temperature(p_mm: f64) -> f64 {
    let p = p_mm.max(0.0);
    (-296.0 + (296.0f64 * 296.0 - 36.0 * (645.0 - p)).max(0.0).sqrt()) / 18.0
}

/// Mean of the three warmest consecutive months (the year wraps).
pub fn summer(t: &[f64; 12]) -> f64 {
    (0..12)
        .map(|m| (t[m] + t[(m + 1) % 12] + t[(m + 2) % 12]) / 3.0)
        .fold(f64::MIN, f64::max)
}

/// A glaciation in progress.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Open {
    pub t_start: f64,
    pub peak_frac: f64,
    pub peak_km2: f64,
    pub peak_t: f64,
    pub lat: f64,
    pub lon: f64,
}

/// What the stage remembers between snapshots.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Tracker {
    pub open: Option<Open>,
    pub count: u64,
}

/// One snapshot's ice.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct IceCover {
    pub glaciated_km2: f64,
    pub land_km2: f64,
    /// Centroid of the ice (lat, lon), degrees.
    pub lat: f64,
    pub lon: f64,
}

impl IceCover {
    pub fn frac(&self) -> f64 {
        if self.land_km2 > 0.0 {
            self.glaciated_km2 / self.land_km2
        } else {
            0.0
        }
    }
}

impl Tracker {
    /// Follows the ice at snapshot time `t` (years); `last` closes an open glaciation.
    pub fn update(
        &mut self,
        world: &mut WorldState,
        t: f64,
        ice: &IceCover,
        on: f64,
        off: f64,
        last: bool,
    ) -> Result<(), CoreError> {
        let f = ice.frac();
        match &mut self.open {
            None if f >= on => {
                self.open = Some(Open {
                    t_start: t,
                    peak_frac: f,
                    peak_km2: ice.glaciated_km2,
                    peak_t: t,
                    lat: ice.lat,
                    lon: ice.lon,
                });
            }
            Some(o) if f > o.peak_frac => {
                o.peak_frac = f;
                o.peak_km2 = ice.glaciated_km2;
                o.peak_t = t;
                o.lat = ice.lat;
                o.lon = ice.lon;
            }
            _ => {}
        }
        let ended = self.open.is_some() && f < off;
        if ended || (last && self.open.is_some()) {
            let o = self.open.take().expect("open");
            let e = world
                .event(o.t_start, "deeptime:glaciation", SOURCE)
                .until(t.max(o.t_start))
                .at(Place::Point {
                    lat: o.lat as f32,
                    lon: o.lon as f32,
                })
                .magnitude((o.peak_frac / 0.2).min(1.0) as f32)
                .param("peak_land_frac", o.peak_frac)
                .param("peak_area_km2", o.peak_km2)
                .param("peak_t_years", o.peak_t)
                .param("ongoing", i64::from(!ended));
            world.emit(e)?;
            self.count += 1;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ela_follows_ohmura() {
        assert!((ela_temperature(1000.0) - 1.16).abs() < 0.02);
        assert!((ela_temperature(0.0) + 2.35).abs() < 0.02);
        assert!((ela_temperature(3000.0) - 6.62).abs() < 0.02);
        let p = ela_temperature(1500.0);
        assert!((645.0 + 296.0 * p + 9.0 * p * p - 1500.0).abs() < 1e-6);
        let mut t = [0.0; 12];
        t[11] = 9.0;
        t[0] = 12.0;
        t[1] = 6.0;
        assert_eq!(summer(&t), 9.0, "wraps December–February");
    }

    #[test]
    fn a_glaciation_is_one_event() {
        let mut w = WorldState::new(1, 4096, -1.0e8);
        let mut tr = Tracker::default();
        let ice = |g: f64| IceCover {
            glaciated_km2: g,
            land_km2: 100.0,
            lat: -80.0,
            lon: 10.0,
        };
        for (k, g) in [0.0, 1.0, 3.0, 8.0, 5.0, 1.5, 0.5, 0.0].iter().enumerate() {
            tr.update(&mut w, k as f64 * 1e7, &ice(*g), 0.02, 0.01, false)
                .unwrap();
        }
        assert_eq!(w.events.len(), 1);
        let e = &w.events[0];
        assert_eq!(e.kind, "deeptime:glaciation");
        assert_eq!((e.t_start, e.t_end), (2e7, 6e7));
        tr.update(&mut w, 8e7, &ice(4.0), 0.02, 0.01, false)
            .unwrap();
        tr.update(&mut w, 9e7, &ice(4.0), 0.02, 0.01, true).unwrap();
        assert_eq!(w.events.len(), 2, "closed as ongoing at the end");
        assert_eq!(tr.count, 2);
    }
}
