// SPDX-License-Identifier: LGPL-3.0-or-later
//! The collision fronts by what stands on either side of them: measurement only, no rule reads it
//! (the docking lane, V1 item 1, 2026-10-23; ledger PLT-515). Kept with the continental crust
//! budget (`crust_budget.rs`, on with its environment variable) and printed with its lines.
//!
//! Each step, each collision front that stacked (`belts::stack`) adds its length times the step
//! (km x Myr) to a bin of the collision state's age (under 10, 10 to 30, 30 to 60, 60 Myr or
//! more) and of the overriding plate's oceanic share within the belt's reach behind the front
//! (`collision_belt_km`; under 0.25, 0.25 to 0.5, 0.5 to 0.75, 0.75 or more). Per age bin it also
//! keeps, weighted by the same km x Myr: the incoming plate's continental share within the reach
//! of the field in front of the front (the wedge's reach and `belts::FOREDEEP_REACH_KM`) and
//! within [`NEAR_KM`]; the share of the incoming continental crust at the front, and of the
//! overriding plate's edge on it, above the sea; the convergence across the front; and the
//! incoming continental samples at the front. Over the run after its first [`SKIP_MYR`].
//!
//! It also splits, over the whole run, the crust the belts found no room for by the belt's class
//! (`belts::BeltClass`: no continental crust within the belt's reach, narrower than half a full
//! belt, wider; ledger PLT-701), with each class's front-time and crossing, so the collision
//! fronts' loss can be read by where it happens.
//!
//! The question it answers: is a front in collision a continent meeting a continent (the
//! incoming side continental far in front of the front), a small body on an ocean plate (the
//! incoming side oceanic beyond the body), or an arc on an ocean plate jammed by a continent (the
//! overriding side oceanic behind the belt)?

use std::fmt::Write as _;

/// The near window in front of the front, km (Earth-normalized). ledger: PLT-515 (game-choice: a
/// measuring window, about four sample spacings).
pub const NEAR_KM: f64 = 300.0;
/// The run's first stretch left out, Myr (as the pool's measures).
pub const SKIP_MYR: f64 = 100.0;
/// The no-room split's belt classes (`belts::BeltClass`) and columns (ledger: PLT-701).
pub const NOROOM_CLASSES: [&str; 3] = ["empty", "narrow", "wide"];
const NOROOM_COLUMNS: [&str; 5] = ["front_km_myr", "crossed", "sliver", "capped", "stays"];
/// The state-age bins' lower edges, Myr.
const AGES: [f64; 4] = [0.0, 10.0, 30.0, 60.0];
/// The overriding side's oceanic-share bins' lower edges.
const OCEANIC: [f64; 4] = [0.0, 0.25, 0.5, 0.75];
/// The weighted sums kept per age bin.
const SUMS: [&str; 6] = [
    "under_cont",
    "under_near",
    "crust_dry",
    "edge_dry",
    "conv_mm_yr",
    "crust_n",
];

/// What stands on either side of one front this step (`belts::stack`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sides {
    /// The overriding plate's visible samples within the belt's reach behind the front: the
    /// share that is oceanic crust.
    pub over_oceanic: f64,
    /// The incoming plate's visible samples within the field in front of the front, and within
    /// [`NEAR_KM`] of its crust at the front: the share that is continental.
    pub under_continental: f64,
    pub under_near: f64,
    /// The incoming continental samples standing at the front, and the share of them above the
    /// sea.
    pub crust_n: u32,
    pub crust_dry: f64,
    /// The share of the overriding plate's edge samples on the front above the sea.
    pub edge_dry: f64,
}

/// The tally over a run.
#[derive(Clone, Debug, Default)]
pub struct DockTally {
    /// From when fronts are counted, Myr (the run's start plus [`SKIP_MYR`]).
    from_myr: Option<f64>,
    /// km x Myr per (state age, overriding oceanic share).
    km_myr: [[f64; 4]; 4],
    /// Per state age, the [`SUMS`] weighted by km x Myr.
    sums: [[f64; 6]; 4],
    /// Per belt class ([`NOROOM_CLASSES`]), over the whole run: the front's km x Myr, the crust
    /// that crossed, and what found no room in the belt (left over with room, at the strength
    /// height, kept by PLT-700), km³ ([`NOROOM_COLUMNS`]).
    noroom: [[f64; 5]; 3],
}

fn bin(edges: &[f64; 4], x: f64) -> usize {
    edges.iter().rposition(|&e| x >= e).unwrap_or(0)
}

impl DockTally {
    /// Starts counting [`SKIP_MYR`] after `t_myr`, the run's start.
    pub fn start(&mut self, t_myr: f64) {
        self.from_myr = Some(t_myr + SKIP_MYR);
    }

    /// Adds one front at `t_myr`: its collision state's age, its length times the step (km x
    /// Myr), its convergence (mm/yr) and what stands on either side.
    pub fn add(&mut self, t_myr: f64, age_myr: f64, km_myr: f64, conv_mm_yr: f64, s: &Sides) {
        if self.from_myr.is_none_or(|t0| t_myr < t0) || km_myr.is_nan() || km_myr <= 0.0 {
            return;
        }
        let a = bin(&AGES, age_myr);
        self.km_myr[a][bin(&OCEANIC, s.over_oceanic)] += km_myr;
        for (sum, x) in self.sums[a].iter_mut().zip([
            s.under_continental,
            s.under_near,
            s.crust_dry,
            s.edge_dry,
            conv_mm_yr,
            f64::from(s.crust_n),
        ]) {
            *sum += km_myr * x;
        }
    }

    /// Adds one front's step to the no-room split (PLT-701; every step of the run): its belt
    /// class (`belts::BeltClass::index`), km x Myr, and km³ crossed, left over with room, at the
    /// strength height and kept.
    pub fn add_noroom(&mut self, class: usize, km_myr: f64, km3: [f64; 4]) {
        let row = &mut self.noroom[class.min(2)];
        row[0] += km_myr;
        for (r, x) in row[1..].iter_mut().zip(km3) {
            *r += x;
        }
    }

    /// The summary's lines: a header, then one per state-age bin with its km x Myr by the
    /// overriding side's oceanic share and its weighted means.
    pub fn lines(&self) -> String {
        let mut out = format!(
            "\ndock_front from_myr={:.0} ages=0-10,10-30,30-60,60+ over_oceanic=0-0.25,0.25-0.5,0.5-0.75,0.75-1 means={}",
            self.from_myr.unwrap_or(f64::NAN),
            SUMS.join(",")
        );
        for (a, lo) in AGES.iter().enumerate() {
            let w: f64 = self.km_myr[a].iter().sum();
            let means: Vec<String> = self.sums[a]
                .iter()
                .map(|&x| {
                    if w > 0.0 {
                        format!("{:.3}", x / w)
                    } else {
                        "nan".to_owned()
                    }
                })
                .collect();
            let _ = write!(
                out,
                "\ndock_age={lo:.0} km_myr={} means={}",
                self.km_myr[a]
                    .iter()
                    .map(|x| format!("{x:.0}"))
                    .collect::<Vec<_>>()
                    .join(","),
                means.join(",")
            );
        }
        let _ = write!(
            out,
            "\ndock_noroom classes={} columns={}",
            NOROOM_CLASSES.join(","),
            NOROOM_COLUMNS.join(",")
        );
        for (name, row) in NOROOM_CLASSES.iter().zip(&self.noroom) {
            let _ = write!(
                out,
                "\ndock_noroom_{name} {}",
                row.iter()
                    .map(|x| format!("{x:.4e}"))
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bins_and_means() {
        let mut t = DockTally::default();
        let s = Sides {
            over_oceanic: 0.8,
            under_continental: 0.5,
            under_near: 1.0,
            crust_n: 4,
            crust_dry: 0.25,
            edge_dry: 1.0,
        };
        // Before the start and in the first stretch: nothing.
        t.add(0.0, 5.0, 100.0, 20.0, &s);
        t.start(-1000.0);
        t.add(-950.0, 5.0, 100.0, 20.0, &s);
        assert_eq!(t.km_myr, [[0.0; 4]; 4]);
        t.add(-800.0, 5.0, 100.0, 20.0, &s);
        t.add(
            -700.0,
            75.0,
            300.0,
            40.0,
            &Sides {
                over_oceanic: 0.1,
                ..s
            },
        );
        assert_eq!(t.km_myr[0][3], 100.0);
        assert_eq!(t.km_myr[3][0], 300.0);
        assert!((t.sums[3][4] / 300.0 - 40.0).abs() < 1e-12);
        assert!((t.sums[0][2] / 100.0 - 0.25).abs() < 1e-12);
        let l = t.lines();
        assert!(
            l.contains("dock_age=0 km_myr=0,0,0,100 means=0.500,1.000,0.250,1.000,20.000,4.000")
        );
        assert!(l.contains("dock_age=10 km_myr=0,0,0,0 means=nan"));
        // The no-room split counts from the run's start, by class.
        t.add_noroom(0, 10.0, [5.0, 0.0, 2.0, 0.0]);
        t.add_noroom(2, 20.0, [8.0, 1.0, 0.5, 0.0]);
        t.add_noroom(2, 20.0, [8.0, 0.0, 0.0, 3.0]);
        let l = t.lines();
        assert!(
            l.contains("dock_noroom_empty 1.0000e1,5.0000e0,0.0000e0,2.0000e0,0.0000e0"),
            "{l}"
        );
        assert!(
            l.contains("dock_noroom_narrow 0.0000e0,0.0000e0,0.0000e0,0.0000e0,0.0000e0"),
            "{l}"
        );
        assert!(
            l.contains("dock_noroom_wide 4.0000e1,1.6000e1,1.0000e0,5.0000e-1,3.0000e0"),
            "{l}"
        );
    }
}
