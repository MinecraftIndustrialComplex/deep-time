// SPDX-License-Identifier: LGPL-3.0-or-later
//! Folds and faults (plan §8.5 Phase 2, work item P2-01): the kinematic history of the rock.
//!
//! - [`emplace`] translates each plate step's tectonics into structures: fold trains and thrusts
//!   where the crust shortens, normal faults where a continent rifts, strike-slip faults along
//!   oblique margins (`emplace.rs` has the table).
//! - [`feature`] is a structure as the simulation carries it, riding its plate to the present.
//! - At the end of the run [`finish`] turns the structures into the world's
//!   [`deeptime_format::structure::Structure`]: events the generator's column rule undoes at each
//!   block column, youngest first.
//! - [`lattice`] evaluates the structure of the whole planet on a lattice of columns (what the
//!   bake does to check and to measure), and compares the column rule with the Noddy port
//!   (`deeptime-noddy`), point by point, on a sample of them.
//!
//! **Why events and not deformed horizons.** The plan's sketch was to evaluate Noddy on a lattice
//! at bake time and fit the result back into the horizon stacks. The stacks live on L1 (16 blocks
//! a cell) and the column rule blends 4 × 4 cells with a B-spline, so a fault fitted into them is
//! a flexure three cells wide, not an offset, and a fold much shorter than four cells cannot be
//! stored at all. Storing the events and undoing them per column keeps a fault a plane to the
//! block and costs a few arithmetic operations a column (see `lattice.rs` for the measurement).
//! It is possible because every event used here moves a whole column at once: its strength
//! depends on the map position only, a fold shifts the column vertically, and a fault moves the
//! part above its plane by one vector (`deeptime_noddy::windowed`).

pub mod emplace;
pub mod feature;
pub mod lattice;
pub mod params;

#[cfg(test)]
mod tests;

pub use emplace::{Inputs, Strain, emplace, strain};
pub use feature::{Counts, Feature, Kind, advance};
pub use params::DeformationParams;

use deeptime_format::structure::{PROFILE_SAMPLES, Structure};

/// The fold profile every fold train uses: Noddy's sine table (`deeptime_noddy::Profile::
/// sine_table`), one wavelength in [`PROFILE_SAMPLES`] values of 1/100.
pub fn sine_profile() -> Vec<f32> {
    match deeptime_noddy::Profile::sine_table() {
        deeptime_noddy::Profile::Table(t) => {
            debug_assert_eq!(t.len(), PROFILE_SAMPLES);
            t.to_vec()
        }
        deeptime_noddy::Profile::Sine => unreachable!("sine_table is a table"),
    }
}

/// The order structures are undone in: youngest first; of two made in the same step, the one
/// made later first.
pub fn undo_order(features: &mut [Feature]) {
    features.sort_by(|a, b| {
        b.t_myr
            .partial_cmp(&a.t_myr)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.id.cmp(&a.id))
    });
}

/// The world's structure from the structures the run left: `features` in [`undo_order`];
/// `surface_y(x, z)` is the ground's height at a block position (a fault's trace lies on it).
pub fn finish(features: &[Feature], surface_y: impl Fn(f64, f64) -> f64) -> Structure {
    Structure {
        events: features
            .iter()
            .map(|f| f.to_baked(surface_y(f.x, f.z)))
            .collect(),
        profiles: vec![sine_profile()],
    }
}

/// The tectonic event behind a structure of `kind` started at block position (x, z) at
/// `t_years`: the nearest event of the kinds that make such structures (a collision, an orogeny
/// phase, a terrane's accretion or a subduction onset for folds and thrusts; a rift onset for
/// normal faults; a subduction onset for strike-slip faults) that began in the 150 Myr before,
/// within an eighth of the planet's circumference. 0 when there is none.
pub fn cause_of(
    events: &[deeptime_format::Event],
    kind: Kind,
    x: f64,
    z: f64,
    circumference: u32,
    t_years: f64,
) -> u64 {
    let kinds: &[&str] = match kind {
        Kind::FoldTrain | Kind::Thrust => &[
            "deeptime:collision",
            "deeptime:orogeny_phase",
            "deeptime:terrane_accretion",
            "deeptime:subduction_onset",
        ],
        Kind::Normal => &["deeptime:rift_onset"],
        Kind::StrikeSlip => &["deeptime:subduction_onset"],
    };
    let c = f64::from(circumference);
    let mut best = (c / 8.0, 0u64);
    for e in events {
        if e.t_start > t_years || e.t_start < t_years - 150.0e6 || !kinds.contains(&e.kind.as_str())
        {
            continue;
        }
        let deeptime_format::Place::Point { lat, lon } = e.place else {
            continue;
        };
        let ex = f64::from(lon) / 360.0 * c - c / 2.0;
        let ez = -f64::from(lat) / 90.0 * c / 4.0;
        let mut dx = x - ex;
        dx -= (dx / c + 0.5).floor() * c;
        let d = dx.hypot(z - ez);
        if d < best.0 {
            best = (d, e.id.0);
        }
    }
    best.1
}

/// The median of `v` (0 when empty).
fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    v[v.len() / 2]
}

/// The run summary's `deformation` object: how many structures of each kind the world keeps and
/// how big they are, what the column rule finds over the planet on L2 and how long it takes, and
/// the Noddy port against the column rule on a sample of those columns.
pub fn report(
    features: &[Feature],
    st: &Structure,
    preset: &crate::params::PlanetPreset,
    counts: &Counts,
    regime: &[f32],
    crust_type: &[f32],
) -> Result<serde_json::Value, crate::error::CoreError> {
    use deeptime_format::structure::StructIndex;
    use serde_json::json;

    let c = preset.scale.circumference_blocks;
    let index = StructIndex::new(st, c);
    let l2 = deeptime_format::GridDesc::for_planet("L2", c, preset.resolution.l2_r)?;
    let (min_y, max_y) = (
        f64::from(preset.scale.min_y),
        f64::from(preset.scale.max_y) + 1.0,
    );
    let cols = lattice::columns(st, &index, &l2, min_y, max_y);
    // Every 53rd L2 column (53 is prime to the grid's width, so the sample covers the planet).
    let sample: Vec<(f64, f64)> = (0..l2.cells())
        .step_by(53)
        .map(|k| {
            let (i, j) = (k % l2.w as usize, k / l2.w as usize);
            (l2.block_x(i as i64) as f64, l2.block_z(j as i64) as f64)
        })
        .collect();
    let step = 4.0;
    let check = lattice::noddy_check(features, st, &index, &sample, min_y, max_y, step);
    let voxels = cols.columns as f64 * ((max_y - min_y) / step);
    let alive = |k: Kind| features.iter().filter(|f| f.kind == k).count();
    let of = |k: Kind, get: &dyn Fn(&Feature) -> f64| {
        median(features.iter().filter(|f| f.kind == k).map(get).collect())
    };
    let frac = |n: u64| n as f64 / cols.columns.max(1) as f64;
    let l1 = deeptime_format::GridDesc::for_planet("L1", c, preset.resolution.l1_r)?;
    let coverage = coverage(st, &index, &l1, regime, crust_type);
    Ok(json!({
        "coverage": coverage,
        "structures": {
            "fold_train": alive(Kind::FoldTrain),
            "thrust": alive(Kind::Thrust),
            "normal_fault": alive(Kind::Normal),
            "strike_slip_fault": alive(Kind::StrikeSlip),
        },
        "formed": {
            "fold_train": counts.fold_trains,
            "thrust": counts.thrusts,
            "normal_fault": counts.normal_faults,
            "strike_slip_fault": counts.strike_slip_faults,
        },
        "saturated": counts.saturated,
        "subducted": counts.subducted,
        "median": {
            "fold_wavelength_blocks": of(Kind::FoldTrain, &|f| f64::from(f.wavelength)),
            "fold_amplitude_blocks": of(Kind::FoldTrain, &|f| f64::from(f.amplitude)),
            "fold_length_blocks": of(Kind::FoldTrain, &|f| 2.0 * f64::from(f.half_along)),
            "thrust_slip_blocks": of(Kind::Thrust, &|f| f64::from(f.slip.abs())),
            "normal_slip_blocks": of(Kind::Normal, &|f| f64::from(f.slip.abs())),
            "fault_length_blocks": median(
                features.iter().filter(|f| f.kind.is_fault()).map(|f| 2.0 * f64::from(f.half_along)).collect()
            ),
        },
        "lattice": {
            "grid": "L2",
            "columns": cols.columns,
            "deformed_frac": frac(cols.deformed),
            "faulted_frac": frac(cols.faulted),
            "mean_candidates": cols.candidates as f64 / cols.columns.max(1) as f64,
            "max_candidates": cols.max_candidates,
            "max_segments": cols.max_segments,
            "max_shift_blocks": cols.max_shift,
            "max_heave_blocks": cols.max_heave,
            "points_at_4_blocks": voxels,
        },
        "noddy_check": {
            "columns": check.columns,
            "columns_with_events": check.columns_with_events,
            "points": check.points,
            "worst": check.worst,
            "side_flips": check.side_flips,
            "events_per_point": check.events_per_point,
        },
        // Wall-clock numbers: the stage moves them into the note's `millis`, the one part of a
        // note that may differ between two runs of the same world.
        "timing": {
            "lattice_s": cols.seconds,
            "threads": rayon::current_num_threads(),
            "noddy_ns_point": check.ns_per_point_noddy(),
            "column_ns_point": check.ns_per_point_columns(),
            // What the general engine would take for the whole lattice on one thread, were
            // every column as busy as the sampled ones that have events.
            "noddy_planet_s": check.ns_per_point_noddy() * 1.0e-9 * voxels
                * (check.columns_with_events as f64 / check.columns.max(1) as f64),
        },
        "bytes": std::mem::size_of_val(features) + st.events.len() * std::mem::size_of::<deeptime_format::structure::StructEvent>(),
    }))
}

/// Where the structures are, by what the crust is now: per tectonic regime of the continental L1
/// cells, the share with at least one structure over the cell (`covered`), with one made in the
/// last 50 Myr of the run (`young`), and the mean number of structures over a cell. This is the
/// realism check's first half ("folded near orogens, flat on cratons"); the other half, which
/// rock is old enough to be deformed, is the column's.
pub fn coverage(
    st: &Structure,
    index: &deeptime_format::structure::StructIndex,
    l1: &deeptime_format::GridDesc,
    regime: &[f32],
    crust_type: &[f32],
) -> serde_json::Value {
    use crate::plates::state::{CONTINENTAL, regime as rg};
    let c = index.circumference();
    let newest = st
        .events
        .iter()
        .map(|e| e.t_years)
        .fold(f64::NEG_INFINITY, f64::max);
    let w = l1.w as usize;
    let mut rows = vec![[0u64; 4]; rg::NAMES.len()];
    let mut cand = Vec::new();
    for k in 0..l1.cells().min(regime.len()) {
        if crust_type[k] as u8 != CONTINENTAL {
            continue;
        }
        let r = (regime[k] as usize).min(rg::NAMES.len() - 1);
        let (x, z) = (
            l1.block_x((k % w) as i64) as f64,
            l1.block_z((k / w) as i64) as f64,
        );
        index.candidates(st, x, z, &mut cand);
        let (mut n, mut young) = (0u64, false);
        for &e in &cand {
            let ev = &st.events[e as usize];
            let (a, b) = ev.window.local(x, z, c);
            if ev.window.weight(a, b) > 0.0 {
                n += 1;
                young |= ev.t_years >= newest - 50.0e6;
            }
        }
        rows[r][0] += 1;
        rows[r][1] += u64::from(n > 0);
        rows[r][2] += u64::from(young);
        rows[r][3] += n;
    }
    let mut out = serde_json::Map::new();
    for (r, row) in rows.iter().enumerate() {
        if row[0] == 0 {
            continue;
        }
        let cells = row[0] as f64;
        out.insert(
            rg::NAMES[r].to_owned(),
            serde_json::json!({
                "cells": row[0],
                "covered": row[1] as f64 / cells,
                "young": row[2] as f64 / cells,
                "mean_structures": row[3] as f64 / cells,
            }),
        );
    }
    serde_json::Value::Object(out)
}
