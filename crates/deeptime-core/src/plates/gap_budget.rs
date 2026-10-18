// SPDX-License-Identifier: LGPL-3.0-or-later
//! The land's gap budget: measurement only, no rule reads it (STATUS follow-up of 2026-10-17,
//! "which process lowers a continental column's surface faster than it thins it").
//!
//! # What it measures
//! Every sample's gap g = z − z_A: its elevation over the Airy height of its column,
//! [`super::collapse::iso_height`] (z_0, the middle of `continent_elevation_m`, for the normal
//! column, plus (T − T_n) / 7.0, less the eclogite root's pull). On a planet where height kept
//! step with thickness the continental gap would stay near zero; on `earthlike_quick_16k` at
//! 500 Myr the 30 to 55 km crust stands 0.5 to 3 km below its Airy height (build log
//! 2026-10-16). The budget says which process made that gap.
//!
//! # How
//! Each sample keeps its gap split into parts that add up to it. Every process of a plate step
//! is bracketed: after it, each sample's change of gap goes to that process's part (to the
//! sample's ocean share as well while it is oceanic crust). The surface stage's feedback,
//! applied at the start of the step, is split by what moved the surface: the rock moved
//! (erosion and deposition, which change the column's thickness too), its local isostatic
//! rebound, the local response to the rest of the change of sea water and ice over the column,
//! the regional part of the flexure's answer to those loads (the plate spreading them), the
//! flexure's bending of the roots the plate model and the spreading put in the elevation, its
//! answer to the moment a weakened plate releases, the rest of the flexure (its answer to the
//! loads later stages laid on the surface, the volcano stage's cones and the impacts, and the
//! solves' unfinished parts), the plateau spreading, what the volcano stage and the impacts
//! added to the surface, and the carry (whatever else the feedback holds: the semi-Lagrangian
//! advection of the surface on L1, the L0/L1 sampling and the clamps). The surface stage writes the per-cell parts as
//! fields ([`field`]) while the budget is on. A resampling blends the parts with the same
//! weights as the column it blends, books the gap its blending adds (the gap is not linear in
//! the pools) and the gap of the crust it accretes (thickness with no height). A sample born later (new sea floor) starts with its gap in the
//! `birth` part. Each part also keeps what of it was made while the sample was oceanic crust
//! (the ocean share), so a continental column's gap splits into what it brought from the ocean
//! floor and what was made on the continent.
//!
//! The budget is off unless [`ON_ENV`] is set; it changes nothing the model does.

use rayon::prelude::*;

use super::collapse::iso_height;
use super::state::{CONTINENTAL, Sample, pool};

/// Set to switch the budget on.
pub const ON_ENV: &str = "DEEPTIME_GAP_BUDGET";

/// Whether the environment asks for the budget (`PlatesStage::new` reads it; the surface stage
/// writes its fields while the plate state holds a budget).
pub fn enabled() -> bool {
    std::env::var_os(ON_ENV).is_some()
}

/// The surface stage's parts of the feedback, per L1 cell, m (written only while the plate state
/// holds a budget; the plate model removes them when it applies the feedback).
pub mod field {
    /// The local isostatic response to the step's erosion and deposition: the local isostasy's
    /// share and the flexure's local (Airy) answer to the same load.
    pub const REBOUND: &str = "surface/gap_rebound_m";
    /// The local isostatic response to the rest of the step's change of sea water and land ice
    /// over the cell (the sea rising or falling, the surface carried under it; the water the
    /// plate model's thickened crust and the spreading push aside goes with [`ROOTS`]).
    pub const WATER: &str = "surface/gap_water_m";
    /// The regional part of the flexure's answer to the step's loads (rock, sea water, ice): its
    /// answer less the local answers of [`REBOUND`] and [`WATER`], what the plate's strength
    /// spreads over its neighbours.
    pub const FLEXURE: &str = "surface/gap_flexure_m";
    /// The flexure's answer to the roots the plate model's uplift and the plateau spreading put
    /// in the elevation, with the sea water their crust pushed aside (`flexure.rs`: the plate
    /// bends under the root it resolves; locally, on land and under the sea, they are
    /// compensated already).
    pub const ROOTS: &str = "surface/gap_roots_m";
    /// The flexure's answer to the moment a weakened plate releases (`flexure.rs`: rigidity lost
    /// to reheating or a plate's broken edge releases the moment in proportion).
    pub const RELEASED: &str = "surface/gap_released_m";
    /// The rest of the flexure's change: its answer to the loads later stages laid on the surface
    /// since the last step (the volcano stage's cones, the impacts; their heights are in
    /// [`LATER`]) and to the last solve's unfinished part, both carried in the unsolved residual,
    /// less what the step's own solve left unfinished (and the split solves' residuals).
    pub const FLEX_REST: &str = "surface/gap_flex_rest_m";
    /// The plateau spreading's change (`spread.rs`).
    pub const SPREAD: &str = "surface/gap_spread_m";
    /// What the stages after the surface stage added to the surface and handed to the plate
    /// model with it: the volcano stage's cones and lava, the impacts' craters and ejecta. The
    /// surface stage lays the field at zero; those stages add their change to it.
    pub const LATER: &str = "surface/gap_later_m";
}

/// What changed a gap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// The gap when the sample was made: the start of the run, or new sea floor.
    Birth = 0,
    /// Surface: the rock erosion removed and deposition laid down (the surface and the column's
    /// thickness change together).
    Erosion,
    /// Surface: the local isostatic rebound of that rock.
    Rebound,
    /// Surface: the local isostatic response to the rest of the change of sea water and ice.
    Water,
    /// Surface: the regional part of the flexure's answer to the step's loads.
    Flexure,
    /// Surface: the flexure's bending of the plate model's and the spreading's roots.
    Roots,
    /// Surface: the flexure's answer to the moment a weakened plate releases.
    Released,
    /// Surface: the rest of the flexure (other stages' loads, unfinished solves).
    FlexRest,
    /// Surface: the plateau spreading.
    Spread,
    /// Surface: the volcano stage's and the impacts' changes of the surface.
    Later,
    /// Surface: the rest of the feedback (advection on L1, sampling, clamps).
    Carry,
    /// `subduction::resolve`.
    Subduction,
    /// `shortening::advance`.
    Shortening,
    /// `belts::stack`.
    Belts,
    /// `subduction::fronts_and_uplift`: arcs and Andean margins.
    Arcs,
    /// `collapse::founder`.
    Founder,
    /// `collapse::step`.
    Collapse,
    /// `transition::step`.
    Transition,
    /// `collision::collide`.
    Collision,
    /// `rifting::heal`, `maybe_rift`, `maybe_initiate_subduction`.
    Rifting,
    /// `hotspots::step`.
    Hotspots,
    /// `rate::limit`: thickening beyond a step's ceiling released (after the processes above and
    /// after a resampling).
    Rate,
    /// The plate model's own surface step (`pools::surface_step`).
    PlateSurface,
    /// `finish_samples`.
    Finish,
    /// `subduction::close_contacts`, `ridges::bury`.
    Bury,
    /// A resampling's blend of the old samples.
    ResampleBlend,
    /// A resampling's accretion of the crust under the top cover.
    ResampleAccrete,
    /// Anything else (the motion).
    Other,
}

/// The surface stage's parts of the feedback and their fields, in the order
/// [`GapBudget::book_surface`] takes them.
pub const SURFACE_PARTS: [(Part, &str); 8] = [
    (Part::Rebound, field::REBOUND),
    (Part::Water, field::WATER),
    (Part::Flexure, field::FLEXURE),
    (Part::Roots, field::ROOTS),
    (Part::Released, field::RELEASED),
    (Part::FlexRest, field::FLEX_REST),
    (Part::Spread, field::SPREAD),
    (Part::Later, field::LATER),
];

/// The number of parts.
pub const PARTS: usize = 28;

/// The parts' names, in order.
pub const NAMES: [&str; PARTS] = [
    "birth",
    "erosion",
    "rebound",
    "water",
    "flexure",
    "roots",
    "released",
    "flex_rest",
    "spread",
    "volcanoes_impacts",
    "carry",
    "subduction",
    "shortening",
    "belts",
    "arcs",
    "founder",
    "collapse",
    "transition",
    "collision",
    "rifting",
    "hotspots",
    "rate",
    "plate_surface",
    "finish",
    "bury",
    "resample_blend",
    "resample_accrete",
    "other",
];

/// The thickness bands, km (the crust bins of `deeptime run`, the two thickest merged).
// ledger: PLT-283 (game-choice: reporting bands, read by no rule; agent, 2026-10-17).
pub const EDGES_KM: [f64; 6] = [0.0, 30.0, 40.0, 55.0, 70.0, f64::INFINITY];

/// The number of bands.
pub const BANDS: usize = EDGES_KM.len() - 1;

/// The band of a thickness in m.
pub fn band(thickness_m: f32) -> usize {
    let t = f64::from(thickness_m) / 1000.0;
    (0..BANDS)
        .find(|&b| t < EDGES_KM[b + 1])
        .unwrap_or(BANDS - 1)
}

/// One sample's account.
#[derive(Clone, Debug, PartialEq)]
struct Track {
    parts: [f32; PARTS],
    /// Of each part, what was made while the sample was oceanic crust (its birth included).
    ocean: [f32; PARTS],
    z: f32,
    iso: f32,
    /// The column's thickness at the last booking, m (the sum of its pools).
    thick: f32,
    continental: bool,
    /// This step's change of elevation and of thickness by part, m ([`GapBudget::begin_step`]
    /// clears them; the spike record, [`StepSpike`]).
    step_dz: [f32; PARTS],
    step_dt: [f32; PARTS],
}

/// How a resampling made a sample: the top cover's old corners and weights, and the pools and
/// elevation it blended before any accretion (which since plate task 8 raises the surface by the
/// accreted crust's Airy height). `None`: new crust.
#[derive(Clone, Copy, Debug)]
pub struct Trace {
    pub tri: [u32; 3],
    pub w: [f64; 3],
    pub pools: [f32; pool::COUNT],
    pub z: f32,
}

/// One band's state at a snapshot: visible continental samples.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BandStock {
    pub n: u64,
    /// Means, m.
    pub z: f64,
    pub iso: f64,
    pub gap: f64,
    /// The median gap, m.
    pub gap_p50: f64,
    /// The mean ocean share (the part of the gap made while the sample was oceanic crust), m.
    pub ocean: f64,
    /// The mean ocean share of each part, m.
    pub ocean_parts: [f64; PARTS],
    /// The mean of each part, m.
    pub parts: [f64; PARTS],
    /// Since the last snapshot: each process's change of the gap per column of the band, m/Myr
    /// (summed over the samples in the band after the process, over the samples in the band at
    /// the steps' ends).
    pub rate: [f64; PARTS],
    /// The same for the surface: each process's change of the elevation, m/Myr per column (its
    /// change of the Airy height is `lift - rate`).
    pub lift: [f64; PARTS],
}

/// The budget at one time.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    pub t_myr: f64,
    /// The share of the visible continental samples in each band.
    pub bands: Vec<BandStock>,
}

/// One step's largest changes of a visible sample, and what made them (the spike record: build
/// log 2026-10-18, "which plate process makes the one-step spikes"). Measured from the step's
/// start, after the surface's feedback, to its end, a resampling included: the change the
/// surface stage reads as the plate model's uplift.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StepSpike {
    pub t_myr: f64,
    /// The largest rise and fall of a visible sample's elevation, m.
    pub rise_m: f64,
    pub fall_m: f64,
    /// The largest thickening of a visible sample's column, m.
    pub thicken_m: f64,
    /// The largest `uplift_mm_yr` of a visible sample times the step, m (what the surface reads).
    pub uplift_m: f64,
    /// The sample of `rise_m`: kind, regime, thickness and elevation at the step's end, m, and
    /// its change of elevation and of thickness by part, m.
    pub kind: u8,
    pub regime: u8,
    pub thickness_m: f64,
    pub z_m: f64,
    pub dz: [f32; PARTS],
    pub dt: [f32; PARTS],
    /// The sample of `thicken_m`: its change of elevation and of thickness by part, m.
    pub thick_dz: [f32; PARTS],
    pub thick_dt: [f32; PARTS],
    /// Visible samples that rose by more than [`SPIKE_EDGES_M`].
    pub over: [u32; 4],
    /// Per part, the largest rise and thickening one visible sample took from it, m.
    pub part_rise: [f32; PARTS],
    pub part_thicken: [f32; PARTS],
}

/// The rises the spike record counts, m.
// ledger: PLT-283 (game-choice: reporting bands, read by no rule; agent, 2026-10-18).
pub const SPIKE_EDGES_M: [f32; 4] = [5000.0, 10_000.0, 20_000.0, 30_000.0];

/// The budget (`PlateState::gap_budget`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GapBudget {
    z_0: f64,
    gravity: f64,
    track: Vec<Track>,
    /// Since the last snapshot: Σ change of gap per band and part, m (one sample's change once
    /// per process), then Σ change of elevation; and the samples per band at the steps' ends.
    flux: Vec<[f64; 2 * PARTS]>,
    sample_steps: Vec<f64>,
    /// Times the sample count changed outside a resampling (the accounts were started again).
    pub restarts: u32,
    pub snapshots: Vec<Snapshot>,
    /// Every step's spike record.
    pub spikes: Vec<StepSpike>,
    /// Per part: visible sample-steps on which the part alone raised a sample by more than
    /// [`SPIKE_EDGES_M`], then thickened its column by more than 7 times as much.
    pub part_over: Vec<[u64; 8]>,
}

fn gap_of(s: &Sample, z_0: f64, gravity: f64) -> (f32, f32) {
    (s.elevation_m, iso_height(&s.pools, z_0, gravity) as f32)
}

/// A column's thickness as the Airy height reads it, m.
fn thick_of(pools: &[f32; pool::COUNT]) -> f32 {
    pools.iter().map(|&x| x.max(0.0)).sum()
}

impl GapBudget {
    /// Starts the accounts: every sample's gap is its birth.
    pub fn start(samples: &[Sample], z_0: f64, gravity: f64) -> GapBudget {
        let mut b = GapBudget {
            z_0,
            gravity,
            track: Vec::new(),
            flux: vec![[0.0; 2 * PARTS]; BANDS],
            sample_steps: vec![0.0; BANDS],
            restarts: 0,
            snapshots: Vec::new(),
            spikes: Vec::new(),
            part_over: vec![[0; 8]; PARTS],
        };
        b.begin(samples);
        b
    }

    /// The start of a step (after the surface's feedback): clears every sample's change of the
    /// step.
    pub fn begin_step(&mut self, samples: &[Sample]) {
        if samples.len() != self.track.len() {
            self.restarts += 1;
            self.begin(samples);
            return;
        }
        self.track.par_iter_mut().for_each(|t| {
            t.step_dz = [0.0; PARTS];
            t.step_dt = [0.0; PARTS];
        });
    }

    /// Records the step's spike ([`StepSpike`]); `samples` as at the step's end, every change
    /// booked.
    fn record_spike(&mut self, samples: &[Sample], t_myr: f64, dt_myr: f64) {
        if samples.len() != self.track.len() {
            return;
        }
        let mut sp = StepSpike {
            t_myr,
            ..StepSpike::default()
        };
        let (mut rise_at, mut thick_at) = (None, None);
        for (v, (s, t)) in samples.iter().zip(&self.track).enumerate() {
            if s.is_hidden() {
                continue;
            }
            let dz = f64::from(t.step_dz.iter().sum::<f32>());
            let dt = f64::from(t.step_dt.iter().sum::<f32>());
            if dz > sp.rise_m {
                sp.rise_m = dz;
                rise_at = Some(v);
            }
            sp.fall_m = sp.fall_m.min(dz);
            if dt > sp.thicken_m {
                sp.thicken_m = dt;
                thick_at = Some(v);
            }
            sp.uplift_m = sp
                .uplift_m
                .max(f64::from(s.uplift_mm_yr) * dt_myr * 1.0e3);
            for (n, e) in sp.over.iter_mut().zip(SPIKE_EDGES_M) {
                *n += u32::from(dz > f64::from(e));
            }
            for p in 0..PARTS {
                let (r, h) = (t.step_dz[p], t.step_dt[p]);
                sp.part_rise[p] = sp.part_rise[p].max(r);
                sp.part_thicken[p] = sp.part_thicken[p].max(h);
                for (k, e) in SPIKE_EDGES_M.iter().enumerate() {
                    self.part_over[p][k] += u64::from(r > *e);
                    self.part_over[p][4 + k] += u64::from(h > 7.0 * e);
                }
            }
        }
        if let Some(v) = rise_at {
            let (s, t) = (&samples[v], &self.track[v]);
            sp.kind = s.kind;
            sp.regime = s.regime;
            sp.thickness_m = f64::from(t.thick);
            sp.z_m = f64::from(s.elevation_m);
            sp.dz = t.step_dz;
            sp.dt = t.step_dt;
        }
        if let Some(v) = thick_at {
            sp.thick_dz = self.track[v].step_dz;
            sp.thick_dt = self.track[v].step_dt;
        }
        self.spikes.push(sp);
    }

    fn begin(&mut self, samples: &[Sample]) {
        let (z_0, g) = (self.z_0, self.gravity);
        self.track = samples
            .par_iter()
            .map(|s| {
                let (z, iso) = gap_of(s, z_0, g);
                let mut parts = [0.0; PARTS];
                parts[Part::Birth as usize] = z - iso;
                Track {
                    parts,
                    ocean: if s.kind == CONTINENTAL {
                        [0.0; PARTS]
                    } else {
                        parts
                    },
                    z,
                    iso,
                    thick: thick_of(&s.pools),
                    continental: s.kind == CONTINENTAL,
                    step_dz: [0.0; PARTS],
                    step_dt: [0.0; PARTS],
                }
            })
            .collect();
    }

    /// Brings every sample's account up to date, `split` dividing its change (dz, the elevation's;
    /// diso, the Airy height's) among the parts: the gap's (`d`) and the elevation's (`l`); the
    /// band sums take the same. `thick_part` takes the change of the column's thickness.
    fn update<F>(&mut self, samples: &[Sample], thick_part: Part, split: F)
    where
        F: Fn(usize, f64, f64, &mut [f64; PARTS], &mut [f64; PARTS]) + Sync,
    {
        if samples.len() != self.track.len() {
            self.restarts += 1;
            self.begin(samples);
            return;
        }
        let (z_0, g) = (self.z_0, self.gravity);
        let sums = self
            .track
            .par_iter_mut()
            .zip(samples.par_iter())
            .enumerate()
            .fold(
                || vec![[0.0f64; 2 * PARTS]; BANDS],
                |mut acc, (v, (t, s))| {
                    let (z, iso) = gap_of(s, z_0, g);
                    let thick = thick_of(&s.pools);
                    let (dz, diso) = (f64::from(z - t.z), f64::from(iso - t.iso));
                    if dz == 0.0
                        && diso == 0.0
                        && thick == t.thick
                        && t.continental == (s.kind == CONTINENTAL)
                    {
                        return acc;
                    }
                    let (mut d, mut l) = ([0.0f64; PARTS], [0.0f64; PARTS]);
                    split(v, dz, diso, &mut d, &mut l);
                    for (p, x) in t.parts.iter_mut().zip(&d) {
                        *p += *x as f32;
                    }
                    for (p, x) in t.step_dz.iter_mut().zip(&l) {
                        *p += *x as f32;
                    }
                    t.step_dt[thick_part as usize] += thick - t.thick;
                    t.thick = thick;
                    if !t.continental {
                        for (p, x) in t.ocean.iter_mut().zip(&d) {
                            *p += *x as f32;
                        }
                    }
                    if s.kind == CONTINENTAL && !s.is_hidden() {
                        let row = &mut acc[band(s.thickness_m)];
                        for (a, x) in row.iter_mut().zip(d.iter().chain(&l)) {
                            *a += x;
                        }
                    }
                    t.z = z;
                    t.iso = iso;
                    t.continental = s.kind == CONTINENTAL;
                    acc
                },
            )
            .reduce(
                || vec![[0.0f64; 2 * PARTS]; BANDS],
                |mut a, b| {
                    for (ra, rb) in a.iter_mut().zip(&b) {
                        for (x, y) in ra.iter_mut().zip(rb) {
                            *x += y;
                        }
                    }
                    a
                },
            );
        for (ra, rb) in self.flux.iter_mut().zip(&sums) {
            for (x, y) in ra.iter_mut().zip(rb) {
                *x += y;
            }
        }
    }

    /// Books every sample's change of gap since the last booking to `part`.
    pub fn book(&mut self, samples: &[Sample], part: Part) {
        self.update(samples, part, |_, dz, diso, d, l| {
            d[part as usize] = dz - diso;
            l[part as usize] = dz;
        });
    }

    /// Books the surface feedback just applied: per sample, the material it took (m, + laid
    /// down) and the parts of its elevation change (m), `surface[k]` for [`SURFACE_PARTS`]`[k]`;
    /// the rest of the elevation change is the carry. `cone_iso` (per sample, m, or empty) is the
    /// change of Airy height from the volcano stage's cones joining the pools: it is theirs
    /// ([`Part::Later`]), not the erosion's.
    pub fn book_surface(
        &mut self,
        samples: &[Sample],
        material: &[f64],
        surface: &[Vec<f64>],
        cone_iso: &[f64],
    ) {
        assert_eq!(
            surface.len(),
            SURFACE_PARTS.len(),
            "one row per surface part"
        );
        self.update(samples, Part::Erosion, |v, dz, diso, d, l| {
            let m = material[v];
            let mut carry = dz - m;
            for ((part, _), x) in SURFACE_PARTS.iter().zip(surface) {
                d[*part as usize] = x[v];
                l[*part as usize] = x[v];
                carry -= x[v];
            }
            let cone = cone_iso.get(v).copied().unwrap_or(0.0);
            d[Part::Later as usize] -= cone;
            d[Part::Erosion as usize] = m - (diso - cone);
            l[Part::Erosion as usize] = m;
            d[Part::Carry as usize] = carry;
            l[Part::Carry as usize] = carry;
        });
    }

    /// After a resampling: the new samples' accounts from the old ones (`traces`, one per new
    /// sample).
    pub fn resampled(&mut self, samples: &[Sample], traces: &[Option<Trace>]) {
        let (z_0, g) = (self.z_0, self.gravity);
        let old = std::mem::take(&mut self.track);
        // Per new sample: its account, its change of gap by part and its blend's and accretion's
        // changes of elevation.
        let rows: Vec<(Track, [f64; PARTS], [f64; 2])> = samples
            .par_iter()
            .zip(traces.par_iter())
            .map(|(s, tr)| {
                let (z, iso) = gap_of(s, z_0, g);
                let continental = s.kind == CONTINENTAL;
                let mut d = [0.0f64; PARTS];
                let thick = thick_of(&s.pools);
                let Some(tr) = tr.filter(|tr| tr.tri.iter().all(|&i| (i as usize) < old.len()))
                else {
                    let mut parts = [0.0; PARTS];
                    parts[Part::Birth as usize] = z - iso;
                    let t = Track {
                        parts,
                        ocean: if continental { [0.0; PARTS] } else { parts },
                        z,
                        iso,
                        thick,
                        continental,
                        step_dz: [0.0; PARTS],
                        step_dt: [0.0; PARTS],
                    };
                    return (t, d, [0.0; 2]);
                };
                let mut parts = [0.0f64; PARTS];
                let mut ocean = [0.0f64; PARTS];
                let mut step_dz = [0.0f64; PARTS];
                let mut step_dt = [0.0f64; PARTS];
                let (mut was, mut was_z, mut was_t) = (0.0f64, 0.0f64, 0.0f64);
                for k in 0..3 {
                    let o = &old[tr.tri[k] as usize];
                    for (p, x) in parts.iter_mut().zip(&o.parts) {
                        *p += tr.w[k] * f64::from(*x);
                    }
                    for (p, x) in ocean.iter_mut().zip(&o.ocean) {
                        *p += tr.w[k] * f64::from(*x);
                    }
                    for (p, x) in step_dz.iter_mut().zip(&o.step_dz) {
                        *p += tr.w[k] * f64::from(*x);
                    }
                    for (p, x) in step_dt.iter_mut().zip(&o.step_dt) {
                        *p += tr.w[k] * f64::from(*x);
                    }
                    was += tr.w[k] * f64::from(o.z - o.iso);
                    was_z += tr.w[k] * f64::from(o.z);
                    was_t += tr.w[k] * f64::from(o.thick);
                }
                let blend_t = f64::from(thick_of(&tr.pools));
                step_dz[Part::ResampleBlend as usize] += f64::from(tr.z) - was_z;
                step_dz[Part::ResampleAccrete as usize] += f64::from(z - tr.z);
                step_dt[Part::ResampleBlend as usize] += blend_t - was_t;
                step_dt[Part::ResampleAccrete as usize] += f64::from(thick) - blend_t;
                let blended = f64::from(tr.z) - iso_height(&tr.pools, z_0, g);
                d[Part::ResampleBlend as usize] = blended - was;
                d[Part::ResampleAccrete as usize] = f64::from(z - iso) - blended;
                for (p, x) in parts.iter_mut().zip(&d) {
                    *p += x;
                }
                if !continental {
                    for (p, x) in ocean.iter_mut().zip(&d) {
                        *p += x;
                    }
                }
                let t = Track {
                    parts: parts.map(|x| x as f32),
                    ocean: ocean.map(|x| x as f32),
                    z,
                    iso,
                    thick,
                    continental,
                    step_dz: step_dz.map(|x| x as f32),
                    step_dt: step_dt.map(|x| x as f32),
                };
                (t, d, [f64::from(tr.z) - was_z, f64::from(z - tr.z)])
            })
            .collect();
        for ((_, d, lift), s) in rows.iter().zip(samples) {
            if s.kind == CONTINENTAL && !s.is_hidden() {
                let row = &mut self.flux[band(s.thickness_m)];
                for (a, x) in row.iter_mut().zip(d) {
                    *a += x;
                }
                row[PARTS + Part::ResampleBlend as usize] += lift[0];
                row[PARTS + Part::ResampleAccrete as usize] += lift[1];
            }
        }
        self.track = rows.into_iter().map(|(t, _, _)| t).collect();
    }

    /// The end of a step at `t_myr` (steps of `dt_myr`): records its spike ([`StepSpike`]) and
    /// counts the samples in each band (the denominator of the rates).
    pub fn end_step(&mut self, samples: &[Sample], t_myr: f64, dt_myr: f64) {
        self.record_spike(samples, t_myr, dt_myr);
        for s in samples {
            if s.kind == CONTINENTAL && !s.is_hidden() {
                self.sample_steps[band(s.thickness_m)] += 1.0;
            }
        }
    }

    /// Records the budget at `t_myr` (steps of `dt_myr`), and starts the next window of rates.
    pub fn snapshot(&mut self, samples: &[Sample], t_myr: f64, dt_myr: f64) {
        let mut bands = vec![BandStock::default(); BANDS];
        let mut gaps: Vec<Vec<f64>> = vec![Vec::new(); BANDS];
        if samples.len() == self.track.len() {
            for (s, t) in samples.iter().zip(&self.track) {
                if s.kind != CONTINENTAL || s.is_hidden() {
                    continue;
                }
                let b = band(s.thickness_m);
                let st = &mut bands[b];
                st.n += 1;
                st.z += f64::from(t.z);
                st.iso += f64::from(t.iso);
                st.gap += f64::from(t.z - t.iso);
                for (a, x) in st.parts.iter_mut().zip(&t.parts) {
                    *a += f64::from(*x);
                }
                for (a, x) in st.ocean_parts.iter_mut().zip(&t.ocean) {
                    *a += f64::from(*x);
                }
                gaps[b].push(f64::from(t.z - t.iso));
            }
        }
        for (b, st) in bands.iter_mut().enumerate() {
            let n = st.n.max(1) as f64;
            st.z /= n;
            st.iso /= n;
            st.gap /= n;
            st.parts.iter_mut().for_each(|x| *x /= n);
            st.ocean_parts.iter_mut().for_each(|x| *x /= n);
            st.ocean = st.ocean_parts.iter().sum();
            let g = &mut gaps[b];
            g.sort_by(f64::total_cmp);
            st.gap_p50 = g.get(g.len() / 2).copied().unwrap_or(f64::NAN);
            let per = self.sample_steps[b].max(1.0) * dt_myr;
            for (r, f) in st.rate.iter_mut().zip(&self.flux[b][..PARTS]) {
                *r = f / per;
            }
            for (r, f) in st.lift.iter_mut().zip(&self.flux[b][PARTS..]) {
                *r = f / per;
            }
        }
        self.snapshots.push(Snapshot { t_myr, bands });
        self.flux = vec![[0.0; 2 * PARTS]; BANDS];
        self.sample_steps = vec![0.0; BANDS];
    }

    /// The largest difference between a sample's parts and its gap, m (a check: the parts add up).
    pub fn worst_residual_m(&self) -> f64 {
        self.track
            .iter()
            .map(|t| {
                let sum: f64 = t.parts.iter().map(|&x| f64::from(x)).sum();
                (sum - f64::from(t.z - t.iso)).abs()
            })
            .fold(0.0, f64::max)
    }

    /// The summary lines for `deeptime run` and `stage`: the part names, then per snapshot and
    /// band (visible continental crust) the stock (means, m) and the window's rates (m/Myr).
    pub fn lines(&self) -> String {
        let mut out = format!(
            "\ngap_budget z0_m={:.0} restarts={} worst_residual_m={:.1} parts={}",
            self.z_0,
            self.restarts,
            self.worst_residual_m(),
            NAMES.join(",")
        );
        for snap in &self.snapshots {
            let total: u64 = snap.bands.iter().map(|b| b.n).sum();
            for (b, st) in snap.bands.iter().enumerate() {
                let fmt = |v: &[f64; PARTS], prec: usize| {
                    v.iter()
                        .map(|x| format!("{x:.prec$}"))
                        .collect::<Vec<_>>()
                        .join("/")
                };
                out += &format!(
                    "\ngap_stock t={:.0} band={}-{}km n={:.3} z={:.0} iso={:.0} gap={:.0} gap_p50={:.0} ocean={:.0} parts={} ocean_parts={}",
                    snap.t_myr,
                    EDGES_KM[b],
                    EDGES_KM[b + 1].min(999.0),
                    st.n as f64 / total.max(1) as f64,
                    st.z,
                    st.iso,
                    st.gap,
                    st.gap_p50,
                    st.ocean,
                    fmt(&st.parts, 0),
                    fmt(&st.ocean_parts, 0),
                );
                out += &format!(
                    "\ngap_rate t={:.0} band={}-{}km m_per_myr={} lift_m_per_myr={}",
                    snap.t_myr,
                    EDGES_KM[b],
                    EDGES_KM[b + 1].min(999.0),
                    fmt(&st.rate, 2),
                    fmt(&st.lift, 2),
                );
            }
        }
        out + &self.spike_lines()
    }

    /// The spike record's lines: the run's summary, then every step whose largest rise passed
    /// the first of [`SPIKE_EDGES_M`] or whose thickening passed 7 times it.
    pub fn spike_lines(&self) -> String {
        let named = |v: &[f32; PARTS], min: f32| {
            let mut x: Vec<(usize, f32)> = v
                .iter()
                .copied()
                .enumerate()
                .filter(|(_, x)| x.abs() >= min)
                .collect();
            x.sort_by(|a, b| b.1.abs().total_cmp(&a.1.abs()));
            x.iter()
                .map(|(p, x)| format!("{}:{x:.0}", NAMES[*p]))
                .collect::<Vec<_>>()
                .join(",")
        };
        let peak = |f: &dyn Fn(&StepSpike) -> f64| self.spikes.iter().map(f).fold(0.0, f64::max);
        let steps_over = |e: f32| {
            self.spikes
                .iter()
                .filter(|s| s.rise_m > f64::from(e))
                .count()
        };
        let over = |k: usize| {
            self.part_over
                .iter()
                .enumerate()
                .filter(|(_, c)| c[k] > 0)
                .map(|(p, c)| format!("{}:{}", NAMES[p], c[k]))
                .collect::<Vec<_>>()
                .join(",")
        };
        let mut part_rise = [0.0f32; PARTS];
        let mut part_thicken = [0.0f32; PARTS];
        for s in &self.spikes {
            for p in 0..PARTS {
                part_rise[p] = part_rise[p].max(s.part_rise[p]);
                part_thicken[p] = part_thicken[p].max(s.part_thicken[p]);
            }
        }
        let mut out = format!(
            "\ngap_spikes steps={} rise_max_m={:.0} uplift_max_m={:.0} thicken_max_m={:.0} \
             steps_rise_over_5/10/20/30km={}/{}/{}/{} part_rise_max_m=[{}] part_thicken_max_m=[{}] \
             part_rise_over_5km=[{}] part_rise_over_10km=[{}] part_rise_over_20km=[{}] \
             part_thicken_over_35km=[{}] part_thicken_over_70km=[{}]",
            self.spikes.len(),
            peak(&|s| s.rise_m),
            peak(&|s| s.uplift_m),
            peak(&|s| s.thicken_m),
            steps_over(SPIKE_EDGES_M[0]),
            steps_over(SPIKE_EDGES_M[1]),
            steps_over(SPIKE_EDGES_M[2]),
            steps_over(SPIKE_EDGES_M[3]),
            named(&part_rise, 500.0),
            named(&part_thicken, 3500.0),
            over(0),
            over(1),
            over(2),
            over(4),
            over(5),
        );
        for s in &self.spikes {
            if s.rise_m <= f64::from(SPIKE_EDGES_M[0])
                && s.thicken_m <= 7.0 * f64::from(SPIKE_EDGES_M[0])
            {
                continue;
            }
            out += &format!(
                "\ngap_spike t={:.0} rise_m={:.0} fall_m={:.0} thicken_m={:.0} uplift_m={:.0} \
                 over_5/10/20/30km={}/{}/{}/{} at[kind={} regime={} T={:.0} z={:.0}] dz=[{}] dT=[{}] \
                 thickest_dz=[{}] thickest_dT=[{}] part_rise=[{}] part_thicken=[{}]",
                s.t_myr,
                s.rise_m,
                s.fall_m,
                s.thicken_m,
                s.uplift_m,
                s.over[0],
                s.over[1],
                s.over[2],
                s.over[3],
                s.kind,
                s.regime,
                s.thickness_m,
                s.z_m,
                named(&s.dz, 100.0),
                named(&s.dt, 700.0),
                named(&s.thick_dz, 100.0),
                named(&s.thick_dt, 700.0),
                named(&s.part_rise, 1000.0),
                named(&s.part_thicken, 7000.0),
            );
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plates::pools::{COLUMN_ISOSTATIC_FACTOR as F, continental_column};
    use crate::plates::state::OCEANIC;

    fn column(z: f32, extra_m: f32) -> Sample {
        let mut s = Sample::blank(0);
        s.kind = CONTINENTAL;
        s.pools = continental_column();
        s.pools[pool::METAMORPHIC] += extra_m;
        s.thickness_m = s.pools.iter().sum();
        s.elevation_m = z;
        s
    }

    /// The parts add up to the gap through bookings, a surface split and a resampling, and each
    /// change lands in the part it was booked to.
    #[test]
    fn the_parts_add_up_to_the_gap() {
        let g = 9.81;
        let mut samples = vec![column(525.0, 0.0), column(1000.0, 7000.0), column(0.0, 0.0)];
        samples[2].kind = OCEANIC;
        let mut b = GapBudget::start(&samples, 525.0, g);
        assert!(b.track[0].parts[Part::Birth as usize].abs() < 1.0);
        // Thickening with no height: the gap falls by T / 7.0.
        samples[0].pools[pool::METAMORPHIC] += 7000.0;
        samples[0].thickness_m += 7000.0;
        b.book(&samples, Part::ResampleAccrete);
        let accreted = b.track[0].parts[Part::ResampleAccrete as usize];
        assert!((f64::from(accreted) + 7000.0 / F).abs() < 1.0, "{accreted}");
        // Erosion of 1 km with a rebound of 800 m: the surface falls 200 m, the Airy height
        // 1000 / 7.0 m.
        samples[1].pools[pool::METAMORPHIC] -= 1000.0;
        samples[1].thickness_m -= 1000.0;
        samples[1].elevation_m -= 200.0;
        let mut surface = vec![vec![0.0; 3]; SURFACE_PARTS.len()];
        surface[0][1] = 800.0;
        b.book_surface(&samples, &[0.0, -1000.0, 0.0], &surface, &[]);
        let t = &b.track[1];
        let erosion = t.parts[Part::Erosion as usize] + t.parts[Part::Rebound as usize];
        assert!(
            (f64::from(erosion) - (-200.0 + 1000.0 / F)).abs() < 1.0,
            "{erosion}"
        );
        assert!(t.parts[Part::Carry as usize].abs() < 0.5);
        // The oceanic sample's change goes to its ocean share as well.
        samples[2].elevation_m -= 100.0;
        b.book(&samples, Part::PlateSurface);
        let ocean: f32 = b.track[2].ocean.iter().sum();
        assert!((ocean - (b.track[2].z - b.track[2].iso)).abs() < 1.0);
        assert_eq!(
            b.track[2].ocean[Part::PlateSurface as usize],
            b.track[2].parts[Part::PlateSurface as usize]
        );
        assert_eq!(
            b.track[1].ocean, [0.0; PARTS],
            "a continental sample has no ocean share"
        );
        assert!(b.worst_residual_m() < 1.0);
        // A resampling onto two samples: one blended half and half from the first two, one new.
        let mut blended = samples[0];
        let pools: [f32; pool::COUNT] =
            std::array::from_fn(|k| 0.5 * (samples[0].pools[k] + samples[1].pools[k]));
        blended.pools = pools;
        blended.pools[pool::SEDIMENT] += 500.0;
        blended.thickness_m = blended.pools.iter().sum();
        // The accretion also raised the surface 50 m (its Airy height, plate task 8).
        let blend_z = 0.5 * (samples[0].elevation_m + samples[1].elevation_m);
        blended.elevation_m = blend_z + 50.0;
        let mut fresh = samples[2];
        fresh.elevation_m = -2500.0;
        let new = vec![blended, fresh];
        b.resampled(
            &new,
            &[
                Some(Trace {
                    tri: [0, 1, 2],
                    w: [0.5, 0.5, 0.0],
                    pools,
                    z: blend_z,
                }),
                None,
            ],
        );
        assert!(b.worst_residual_m() < 1.0, "{}", b.worst_residual_m());
        let acc = b.track[0].parts[Part::ResampleAccrete as usize];
        let expect = 0.5 * f64::from(accreted) - 500.0 / F + 50.0;
        assert!((f64::from(acc) - expect).abs() < 1.0, "{acc} vs {expect}");
        b.end_step(&new, 2.0, 2.0);
        b.snapshot(&new, 2.0, 2.0);
        assert_eq!(b.snapshots[0].bands.iter().map(|s| s.n).sum::<u64>(), 1);
        assert!(b.lines().contains("gap_stock t=2"));
    }
}
