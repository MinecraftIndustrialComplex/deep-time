// SPDX-License-Identifier: LGPL-3.0-or-later
//! Run statistics of the plate model: the plan's statistical targets (oceanic crust mostly younger
//! than ~250 Myr; a few large plates plus small ones) and the CLI's summary line.

use super::state::{CONTINENTAL, Counters, PlateState};

/// A snapshot of the crust's statistics (visible samples).
#[derive(Clone, Debug, PartialEq)]
pub struct Summary {
    /// Plates alive.
    pub plates: usize,
    /// (plate id, fraction of the planet's surface), largest first.
    pub areas: Vec<(u32, f64)>,
    /// Oceanic fraction of the surface.
    pub ocean_frac: f64,
    /// Oceanic crust age percentiles, Myr (area-weighted: samples are equal-area).
    pub ocean_age_p50_myr: f64,
    pub ocean_age_p90_myr: f64,
    pub ocean_age_max_myr: f64,
    /// Fraction of oceanic crust younger than 250 Myr.
    pub ocean_younger_250: f64,
    /// Smallest oceanic crust age seen (must be ≥ 0).
    pub ocean_age_min_myr: f64,
    /// Samples (visible or hidden) per alive plate sum to the sample count, and every sample's
    /// plate is alive: no sample in two plates and none in none.
    pub partition_ok: bool,
    /// The run's event counters.
    pub counters: Counters,
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let i = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

/// The statistics of `state` at its current time.
pub fn summary(state: &PlateState) -> Summary {
    let t = state.t_myr;
    let n = state.samples.len();
    let mut per_plate = vec![0usize; state.plates.len()];
    let mut all = vec![0usize; state.plates.len()];
    let mut ages = Vec::new();
    let mut visible = 0usize;
    for (i, s) in state.samples.iter().enumerate() {
        let p = state.plate_of(i) as usize;
        all[p] += 1;
        if s.is_hidden() {
            continue;
        }
        visible += 1;
        per_plate[p] += 1;
        if s.kind != CONTINENTAL {
            ages.push(t - f64::from(s.formed_myr));
        }
    }
    ages.sort_by(f64::total_cmp);
    let mut areas: Vec<(u32, f64)> = state
        .alive()
        .map(|p| {
            (
                p.id,
                per_plate[p.id as usize] as f64 / visible.max(1) as f64,
            )
        })
        .collect();
    areas.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    let partition_ok = state
        .samples
        .iter()
        .enumerate()
        .all(|(i, _)| state.plates[state.plate_of(i) as usize].alive)
        && state.alive().map(|p| all[p.id as usize]).sum::<usize>() == n;
    Summary {
        plates: state.alive().count(),
        areas,
        ocean_frac: ages.len() as f64 / visible.max(1) as f64,
        ocean_age_p50_myr: percentile(&ages, 0.5),
        ocean_age_p90_myr: percentile(&ages, 0.9),
        ocean_age_max_myr: ages.last().copied().unwrap_or(f64::NAN),
        ocean_younger_250: if ages.is_empty() {
            1.0
        } else {
            ages.iter().filter(|&&a| a < 250.0).count() as f64 / ages.len() as f64
        },
        ocean_age_min_myr: ages.first().copied().unwrap_or(0.0),
        partition_ok,
        counters: state.counters.clone(),
    }
}

impl Summary {
    /// Plates covering at least `frac` of the surface.
    pub fn plates_at_least(&self, frac: f64) -> usize {
        self.areas.iter().filter(|a| a.1 >= frac).count()
    }

    /// The CLI's line (plan P1-03: `plates_final=<n> ocean_age_p90_myr=<a> …`).
    pub fn line(&self) -> String {
        let big = self.areas.first().map_or(0.0, |a| a.1);
        format!(
            "plates_final={} ocean_age_p90_myr={:.1} ocean_age_p50_myr={:.1} \
             ocean_younger_250={:.3} ocean_frac={:.3} largest_plate_frac={:.3} \
             plates_over_5pct={} plates_under_1pct={} rifts={} initiations={} reversals={} \
             subductions={} collisions={} accretions={} ridges={} births={} deaths={} lips={} \
             resamplings={}",
            self.plates,
            self.ocean_age_p90_myr,
            self.ocean_age_p50_myr,
            self.ocean_younger_250,
            self.ocean_frac,
            big,
            self.plates_at_least(0.05),
            self.areas.iter().filter(|a| a.1 < 0.01).count(),
            self.counters.rifts,
            self.counters.initiations,
            self.counters.reversals,
            self.counters.subduction_onsets,
            self.counters.collisions,
            self.counters.accretions,
            self.counters.ridges,
            self.counters.plate_births,
            self.counters.plate_deaths,
            self.counters.lips,
            self.counters.resamplings
        )
    }
}
