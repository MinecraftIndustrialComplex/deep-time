// SPDX-License-Identifier: LGPL-3.0-or-later
//! Folds and faults (plan §8.5 Phase 2, work item P2-01): the kinematic history of the rock.
//!
//! - [`emplace`] translates each plate step's tectonics into structures: fold trains and thrusts
//!   where the crust shortens, normal faults where a continent rifts, strike-slip faults along
//!   oblique margins (`emplace.rs` has the table).
//! - [`intrude`] adds the small intrusions that cut the layering: stocks in orogens, dykes along
//!   rifts, and plugs under extinct volcanoes.
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
pub mod intrude;
pub mod lattice;
pub mod params;

#[cfg(test)]
mod tests;

pub use emplace::{Inputs, Strain, emplace, strain};
pub use feature::{Counts, Feature, Kind, advance, wear};
pub use params::DeformationParams;

use deeptime_format::structure::{PROFILE_SAMPLES, Structure};

/// The fold profile every fold train uses: a sine in Noddy's table format
/// (`deeptime_noddy::Profile::sine_table`), one wavelength in [`PROFILE_SAMPLES`] values of
/// 1/100. Noddy's own default "sine" profile is coarser: 21 control points, each the sine
/// rounded down to a whole hundredth, joined by straight segments (noddyverse `nodLib3.c`
/// `initGraphPoints`, `proflib.c` `getArrayForProfile`), so it peaks at 0.99 and departs from
/// the sine by up to 2.2 % of the amplitude. Here every sample is the sine itself.
/// ledger: DEF-010
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
/// `surface_y(x, z)` is the ground's height at a block position (a fault's trace lies on it) and
/// `key(feature)` its age as a stratigraphic key of the horizon stacks
/// (`columns::key::of_age`); `floor_y` is the world's floor (dykes and plugs, and stocks without
/// a `base`, are rooted on it).
pub fn finish(
    features: &[Feature],
    surface_y: impl Fn(f64, f64) -> f64,
    key: impl Fn(&Feature) -> u16,
    floor_y: f64,
) -> Structure {
    Structure {
        events: features
            .iter()
            .map(|f| f.to_baked(surface_y(f.x, f.z), key(f), floor_y))
            .collect(),
        profiles: vec![sine_profile()],
        margin: deeptime_format::structure::REACH_MARGIN,
    }
}

/// How much of a structure's relief is left on the ground at `now_myr` (0 to `relief`): it fades
/// linearly over `relief_age_myr`, and as the surface has eroded or buried the ground there
/// ([`Feature::wear_m`]) by `relief_wear` times the height of its relief on the ground (`relief`
/// times the structure's own height as drawn, `m_per_block_v` metres a block: the scarp on the
/// ground is what wears away, not the offset in the rock). A strike-slip fault has none.
pub fn relief_left(f: &Feature, now_myr: f64, p: &DeformationParams, m_per_block_v: f64) -> f32 {
    if p.relief <= 0.0 {
        return 0.0;
    }
    let height_blocks = match f.kind {
        Kind::FoldTrain => f64::from(f.amplitude),
        Kind::Thrust | Kind::Normal => {
            f64::from(f.slip.abs()) * f64::from(f.dip_deg).to_radians().sin()
        }
        Kind::StrikeSlip | Kind::Stock | Kind::Dyke | Kind::Plug => 0.0,
    };
    let height_m = p.relief * height_blocks * m_per_block_v;
    if height_m <= 0.0 {
        return 0.0;
    }
    let age = (now_myr - f64::from(f.t_myr)).max(0.0);
    let young = (1.0 - age / p.relief_age_myr).clamp(0.0, 1.0);
    let whole = (1.0 - f64::from(f.wear_m) / (p.relief_wear * height_m)).clamp(0.0, 1.0);
    (p.relief * young * whole) as f32
}

/// Makes sure the structure's margin covers the furthest any chain of faults moves rock sideways:
/// the column rule is run over `grid` (L2), and while the largest sideways shift it finds, with a
/// quarter to spare, is over the margin, the margin is raised to one and a half times the shift
/// and the lattice run again (a wider margin brings more candidate events to a column, which can
/// lengthen a chain). Returns the last lattice's statistics.
pub fn settle_margin(
    st: &mut Structure,
    circumference: u32,
    grid: &deeptime_format::GridDesc,
    min_y: f64,
    max_y: f64,
) -> lattice::ColumnStats {
    let mut stats = lattice::ColumnStats::default();
    for _ in 0..6 {
        let index = deeptime_format::structure::StructIndex::new(st, circumference);
        let seconds = stats.seconds;
        stats = lattice::columns(st, &index, grid, min_y, max_y);
        stats.seconds += seconds;
        if stats.max_heave * 1.25 <= st.margin {
            break;
        }
        st.margin = (stats.max_heave * 1.5).ceil();
    }
    stats
}

/// The kinds of event that start each kind of structure, as bits of `Cause::kinds`.
const FOLDS: u8 = 1;
const RIFTS: u8 = 2;
const SHEARS: u8 = 4;

/// The bits of `Cause::kinds` for an event kind (0: it starts no structure).
fn cause_bits(kind: &str) -> u8 {
    match kind {
        "deeptime:collision" | "deeptime:orogeny_phase" | "deeptime:terrane_accretion" => FOLDS,
        "deeptime:subduction_onset" => FOLDS | SHEARS,
        "deeptime:rift_onset" => RIFTS,
        _ => 0,
    }
}

/// One event that can start a structure: a point event of a kind [`cause_of`] looks for, at its
/// place on the block grid.
#[derive(Clone, Copy, Debug)]
struct Cause {
    kinds: u8,
    t_start: f64,
    x: f64,
    z: f64,
    id: u64,
}

/// The world's events that can start a structure, in the order they were emitted, kept up as
/// events come ([`Causes::update`]). The structure stage reads them from here, not from the
/// world's event log, so that its step can run beside the next plate step, which adds events
/// (P1-19), and so that a structure's cause is not a scan of every event and its kind.
#[derive(Clone, Debug, Default)]
pub struct Causes {
    list: Vec<Cause>,
    /// Events of the world's log already looked at.
    seen: usize,
    circumference: u32,
}

impl Causes {
    /// Takes in the events of `events` not looked at yet. Events are only ever appended to the
    /// world's log, and what is read here (kind, place, start, id) does not change once emitted;
    /// a log shorter than the one looked at (another world) starts the list again.
    pub fn update(&mut self, events: &[deeptime_format::Event], circumference: u32) {
        if self.seen > events.len() || self.circumference != circumference {
            *self = Causes {
                circumference,
                ..Causes::default()
            };
        }
        let proj = deeptime_format::Projection::of(f64::from(circumference));
        for e in &events[self.seen..] {
            let kinds = cause_bits(e.kind.as_str());
            if kinds == 0 {
                continue;
            }
            let deeptime_format::Place::Point { lat, lon } = e.place else {
                continue;
            };
            let (x, z) = proj.block_of(f64::from(lat), f64::from(lon));
            self.list.push(Cause {
                kinds,
                t_start: e.t_start,
                x,
                z,
                id: e.id.0,
            });
        }
        self.seen = events.len();
    }
}

/// The tectonic event behind a structure of `kind` started at block position (x, z) at
/// `t_years`: the nearest event of the kinds that make such structures (a collision, an orogeny
/// phase, a terrane's accretion or a subduction onset for folds and thrusts; a rift onset for
/// normal faults; a subduction onset for strike-slip faults) that began in the 150 Myr before,
/// within an eighth of the planet's circumference; of two as near, the one emitted first. 0 when
/// there is none. `causes` holds the world's events ([`Causes::update`], for `circumference`).
pub fn cause_of(
    causes: &Causes,
    kind: Kind,
    x: f64,
    z: f64,
    circumference: u32,
    t_years: f64,
) -> u64 {
    let bits = match kind {
        Kind::FoldTrain | Kind::Thrust | Kind::Stock => FOLDS,
        Kind::Normal | Kind::Dyke => RIFTS,
        Kind::StrikeSlip => SHEARS,
        // A plug's cause is its volcano's.
        Kind::Plug => return 0,
    };
    let c = f64::from(circumference);
    let mut best = (c / 8.0, 0u64);
    for e in &causes.list {
        // ledger: DEF-036 (150 Myr, an eighth of the circumference)
        if e.t_start > t_years || e.t_start < t_years - 150.0e6 || e.kinds & bits == 0 {
            continue;
        }
        let mut dx = x - e.x;
        dx -= (dx / c + 0.5).floor() * c;
        let d = dx.hypot(z - e.z);
        if d < best.0 {
            best = (d, e.id);
        }
    }
    best.1
}

/// The source id of the structure stage's events.
pub const SOURCE: &str = "deeptime:structure";

/// One grouped entry for the event log (`EVENTS.md`, "The structures' events"): the folds and
/// thrusts one orogeny left (`deeptime:fold_belt`), or the faults of one rift or one oblique
/// margin (`deeptime:fault_system`).
#[derive(Clone, Debug, PartialEq)]
pub struct Group {
    pub kind: &'static str,
    /// `orogen`, `rift` or `strike_slip`.
    pub setting: &'static str,
    /// Years: its first and its last structure.
    pub t_start: f64,
    pub t_end: f64,
    /// The centroid of its structures as they lie now.
    pub lat: f32,
    pub lon: f32,
    pub magnitude: f32,
    /// The tectonic event behind it (0: none found).
    pub cause: u64,
    /// Numeric params, in the order written.
    pub params: Vec<(&'static str, f64)>,
}

/// The structures grouped for the event log (one entry per belt or system, not one per fold,
/// as the volcanoes' events are grouped by arc and chain): by the tectonic event that made them (`Feature::cause`), and
/// where none was found by the plate they ride. Plugs belong to their volcanoes' events.
/// `c` is the circumference in blocks; lengths are reported in km on the block metric and
/// heights in m.
pub fn groups(features: &[Feature], c: f64, m_per_block_h: f64, m_per_block_v: f64) -> Vec<Group> {
    use std::collections::BTreeMap;
    let mut sets: BTreeMap<(u8, u64, u32), Vec<&Feature>> = BTreeMap::new();
    for f in features {
        let set = match f.kind {
            Kind::FoldTrain | Kind::Thrust | Kind::Stock => 0u8,
            Kind::Normal | Kind::Dyke => 1,
            Kind::StrikeSlip => 2,
            Kind::Plug => continue,
        };
        let plate = if f.cause == 0 { f.plate } else { 0 };
        sets.entry((set, f.cause, plate)).or_default().push(f);
    }
    let km = m_per_block_h / 1000.0;
    let mut out = Vec::new();
    for ((set, cause, plate), fs) in sets {
        let count = |k: Kind| fs.iter().filter(|f| f.kind == k).count() as f64;
        let med = |k: Kind, get: &dyn Fn(&Feature) -> f64| {
            median(fs.iter().filter(|f| f.kind == k).map(|f| get(f)).collect())
        };
        // The centroid: x the way round the planet, z plainly.
        let (mut sx, mut cx, mut sz) = (0.0f64, 0.0f64, 0.0f64);
        for f in &fs {
            let a = std::f64::consts::TAU * f.x / c;
            sx += a.sin();
            cx += a.cos();
            sz += f.z;
        }
        let x = sx.atan2(cx) / std::f64::consts::TAU * c;
        let z = sz / fs.len() as f64;
        let (lat, lon) = deeptime_format::Projection::of(c).lat_lon(x, z);
        let t0 = fs.iter().map(|f| f.t_myr).fold(f32::INFINITY, f32::min);
        let t1 = fs.iter().map(|f| f.t_myr).fold(f32::NEG_INFINITY, f32::max);
        let throw_m = |f: &Feature| {
            f64::from(f.slip.abs()) * f64::from(f.dip_deg).to_radians().sin() * m_per_block_v
        };
        let max_throw = |k: Kind| {
            fs.iter()
                .filter(|f| f.kind == k)
                .map(|f| throw_m(f))
                .fold(0.0f64, f64::max)
        };
        let (kind, setting, params) = match set {
            0 => (
                "deeptime:fold_belt",
                "orogen",
                vec![
                    ("fold_trains", count(Kind::FoldTrain)),
                    ("thrusts", count(Kind::Thrust)),
                    ("stocks", count(Kind::Stock)),
                    (
                        "fold_wavelength_km",
                        med(Kind::FoldTrain, &|f| f64::from(f.wavelength)) * km,
                    ),
                    (
                        "fold_amplitude_m",
                        med(Kind::FoldTrain, &|f| f64::from(f.amplitude)) * m_per_block_v,
                    ),
                    ("max_thrust_throw_m", max_throw(Kind::Thrust)),
                ],
            ),
            1 => (
                "deeptime:fault_system",
                "rift",
                vec![
                    ("faults", count(Kind::Normal)),
                    ("dykes", count(Kind::Dyke)),
                    (
                        "fault_length_km",
                        med(Kind::Normal, &|f| 2.0 * f64::from(f.half_along)) * km,
                    ),
                    ("max_throw_m", max_throw(Kind::Normal)),
                ],
            ),
            _ => (
                "deeptime:fault_system",
                "strike_slip",
                vec![
                    ("faults", count(Kind::StrikeSlip)),
                    (
                        "fault_length_km",
                        med(Kind::StrikeSlip, &|f| 2.0 * f64::from(f.half_along)) * km,
                    ),
                    (
                        "max_offset_km",
                        fs.iter()
                            .map(|f| f64::from(f.slip.abs()))
                            .fold(0.0f64, f64::max)
                            * km,
                    ),
                ],
            ),
        };
        let mut params = params;
        if cause == 0 {
            params.push(("plate", f64::from(plate)));
        }
        // Where to look: the structure that moved the rock furthest up or down (a strike-slip
        // system's longest) among the group's younger ones, and the azimuth of its long axis (a
        // section across it runs at right angles to that).
        let height = |f: &Feature| match f.kind {
            Kind::FoldTrain => f64::from(f.amplitude),
            Kind::Thrust | Kind::Normal => {
                f64::from(f.slip.abs()) * f64::from(f.dip_deg).to_radians().sin()
            }
            Kind::StrikeSlip => f64::from(f.half_along) * 1e-3,
            Kind::Stock | Kind::Dyke | Kind::Plug => 0.0,
        };
        // (A young one counts for up to four times an old one: it cuts more of the rock.)
        let score =
            |f: &Feature| height(f) * (0.25 + 0.75 * (-f64::from(t1 - f.t_myr) / 60.0).exp());
        if let Some(best) = fs
            .iter()
            .max_by(|a, b| score(a).total_cmp(&score(b)).then(b.id.cmp(&a.id)))
        {
            params.push(("strongest_x", best.x.round()));
            params.push(("strongest_z", best.z.round()));
            params.push(("strongest_azimuth_deg", f64::from(best.azimuth_deg)));
        }
        out.push(Group {
            kind,
            setting,
            t_start: f64::from(t0) * 1.0e6,
            t_end: f64::from(t1) * 1.0e6,
            lat: lat as f32,
            lon: lon as f32,
            magnitude: (fs.len() as f32 / 50.0).min(1.0),
            cause,
            params,
        });
    }
    out
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
    st: &mut Structure,
    preset: &crate::params::PlanetPreset,
    counts: &Counts,
    regime: &[f32],
    crust_type: &[f32],
) -> Result<serde_json::Value, crate::error::CoreError> {
    use deeptime_format::structure::StructIndex;
    use serde_json::json;

    let c = preset.scale.circumference_blocks;
    let l2 = deeptime_format::GridDesc::for_planet("L2", c, preset.resolution.l2_r)?;
    let (min_y, max_y) = (
        f64::from(preset.scale.min_y),
        f64::from(preset.scale.max_y) + 1.0,
    );
    let cols = settle_margin(st, c, &l2, min_y, max_y);
    let st: &Structure = st;
    let index = StructIndex::new(st, c);
    // Every 53rd L2 column (53 is prime to the grid's width, so the sample covers the planet).
    let sample: Vec<(f64, f64)> = (0..l2.cells())
        .step_by(53)
        .map(|k| {
            let (i, j) = (k % l2.w as usize, k / l2.w as usize);
            (l2.block_x(i as i64) as f64, l2.block_z(j as i64) as f64)
        })
        .collect();
    let step = 4.0;
    let check = lattice::noddy_check(
        features,
        st,
        &index,
        &sample,
        min_y,
        max_y,
        step,
        lattice::STAGE_PLANE_GUARD_BLOCKS,
    );
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
            "stock": alive(Kind::Stock),
            "dyke": alive(Kind::Dyke),
            "plug": alive(Kind::Plug),
        },
        "formed": {
            "fold_train": counts.fold_trains,
            "thrust": counts.thrusts,
            "normal_fault": counts.normal_faults,
            "strike_slip_fault": counts.strike_slip_faults,
            "stock": counts.stocks,
            "dyke": counts.dykes,
        },
        // Fold trains and thrusts started, by the rule that strained their cell.
        "formed_by": {
            "collision": {"fold_train": counts.shortened_by[0][0], "thrust": counts.shortened_by[0][1]},
            "arc": {"fold_train": counts.shortened_by[1][0], "thrust": counts.shortened_by[1][1]},
            "foreland": {"fold_train": counts.shortened_by[2][0], "thrust": counts.shortened_by[2][1]},
            "basin": {"fold_train": counts.shortened_by[3][0], "thrust": counts.shortened_by[3][1]},
        },
        // The Andean rule's inputs (DEF-026): continental arc and back-arc cell-steps, their mean
        // lift factor, the share at full lift, and their mean Andean strain.
        "arc_rule": {
            "cell_steps": counts.arc_cell_steps,
            "mean_lift": counts.arc_lift_sum / counts.arc_cell_steps.max(1) as f64,
            "lifted_share": counts.arc_lifted_steps as f64 / counts.arc_cell_steps.max(1) as f64,
            "mean_strain": counts.arc_strain_sum / counts.arc_cell_steps.max(1) as f64,
        },
        "saturated": counts.saturated,
        "overprinted": counts.overprinted,
        "subducted": counts.subducted,
        "median": {
            "fold_wavelength_blocks": of(Kind::FoldTrain, &|f| f64::from(f.wavelength)),
            "fold_amplitude_blocks": of(Kind::FoldTrain, &|f| f64::from(f.amplitude)),
            "fold_length_blocks": of(Kind::FoldTrain, &|f| 2.0 * f64::from(f.half_along)),
            "thrust_slip_blocks": of(Kind::Thrust, &|f| f64::from(f.slip.abs())),
            "normal_slip_blocks": of(Kind::Normal, &|f| f64::from(f.slip.abs())),
            "stock_radius_blocks": of(Kind::Stock, &|f| f64::from(f.half_across)),
            "dyke_length_blocks": of(Kind::Dyke, &|f| 2.0 * f64::from(f.half_along)),
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
            "margin_blocks": st.margin,
            "points_at_4_blocks": voxels,
        },
        "noddy_check": {
            "columns": check.columns,
            "columns_with_events": check.columns_with_events,
            "points": check.points,
            "worst": check.worst,
            "side_flips": check.side_flips,
            "max_history": check.max_history,
            "long_histories": check.long_histories,
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
            if matches!(
                ev.shape,
                deeptime_format::structure::Shape::Intrusion { .. }
            ) {
                continue;
            }
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
