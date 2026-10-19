// SPDX-License-Identifier: LGPL-3.0-or-later
//! The deposit engine's field history from the real pipeline (P1-14's interface,
//! `deposits/fields.rs`): every plate step adds its fields to a frame accumulator; every
//! `frame_every_myr` the time-averaged frame is packed into the world's [`RecordedHistory`].
//!
//! | field | from | how |
//! |---|---|---|
//! | `arc_magma_flux` | plates (P1-03) | 1 where the sample took arc magma this step (`ARC`), × min(1, convergence / 80 mm/yr); 0.7 of it on oceanic crust |
//! | `magma_oxidation` | plates | the sample's flux-weighted oxidation proxy where it is arc, else 0 |
//! | `convergence_rate_km_myr`, `obliquity_deg`, `slab_age_myr` | plates | at the nearest trench, where the sample is within the subduction distance this step (`SUBDUCTION_ZONE`), else 0 (1 mm/yr = 1 km/Myr) |
//! | `crust_thickness_km` | plates | the rock pools' sum |
//! | `fault_density` | plates | by regime: collision 1, rift 0.8, arc 0.6, trench 0.5, back-arc 0.4, hotspot 0.3, else 0.6·e^(−orogeny age / 50 Myr) floored at 0.05 |
//! | `orogenic_uplift_m_myr` | plates | the plate model's uplift where it is positive on continental crust with an orogeny |
//! | `rift_subsidence_m_myr` | plates | minus the plate model's uplift where it is negative on continental crust |
//! | `heat_flow_mw_m2` | plates | the sample's heat-flow proxy |
//! | `dist_litho_step_km` | plates | at each frame: distance (km, block metric) to the nearest edge between craton and non-craton regime cells |
//! | `inherited_bif` | plates + timeline | the frame's share of time the cell was an inherited iron-formation basin under a stable interior (`deposits::bif_basins`, carried with the crust) × the share of the BIF acme before the run's start (`deposits::clocks`; plan §8.6: inherited cratonic basement where the timeline starts after the acme) |
//! | `elevation_m` | surface (P1-04) | above sea level of the time |
//! | `erosion_rate_m_myr`, `deposition_rate_m_myr` | surface | the step's net erosion or deposition |
//! | `topographic_gradient_m_km` | surface | at each frame: gradient of the frame-mean land elevation smoothed over ~50 km |
//! | `shelf` | surface | 1 where the sea is shallower than 200 m |
//! | `carbonate_platform`, `evaporite`, `euxinic_basin` | surface | max(share of the step's deposit in that facies, share of the facies in the top 1 km of the strata log) |
//! | `coal_facies` | surface | 1 where the peat-mire rule held and the cell aggraded this step (`facies.rs`) |
//! | `source_rock_toc` | surface | the richest layer of the log: black shale 5, coal 3, shale 1, mudstone 0.8 wt% TOC, × min(1, thickness / 20 m) |
//! | `burial_temp_c` | surface | that layer's mid-depth × the geotherm (heat flow / 2.5 W/m/K) + the surface temperature; the surface temperature where there is none |
//! | `reservoir_quality`, `seal` | surface | reservoir-tagged rock above the source layer (/ 50 m, capped at 1); seal-tagged rock above the topmost reservoir (/ 20 m) |
//! | `water_table_depth_m` | surface + climate | land: 5 m + 150 m·(1 − min(1, P/PET)) + 5 % of the local relief; sea: 0 |
//! | `precip_mm`, `t_mean_c` | climate (P1-05, `crate::climate`), read through `climate.rs`; the zonal stand-in without a `climate` section | the climate the surface step ran with |
//! | clocks `oxygenated_oceans`, `land_plants`, `bif_acme` | timeline | the share of the frame inside each clock's window of Earth's history, by the planet's age (`deposits::clocks`; stage 2 WP-9/10 replaced the fixed stand-ins 1, 1, 0) |
//!
//! `trap_structure` and the `host/<class>` fractions come from the structure stage (P1-07,
//! `structure/traps.rs`, `structure/hosts.rs`), which adds them to each frame after this stage
//! closes it.

use std::collections::BTreeMap;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::grid::{Dims, box_blur};
use super::lithology::{Classes, has_tag};
use super::strata_log::StrataLog;
use crate::deposits::clocks::Epoch;
use crate::deposits::fields::{FieldFrame, NO_ORIGIN, ORIGIN_FIELDS, lat_of_row};
use crate::plates::state::{ARC, CONTINENTAL, SUBDUCTION_ZONE, regime};

/// Prefix of every field name.
const P: &str = "deeptime:";

/// Names of the per-step fields (averaged into frames).
pub const STEP_FIELDS: [&str; 26] = [
    "deeptime:arc_magma_flux",
    "deeptime:magma_oxidation",
    "deeptime:convergence_rate_km_myr",
    "deeptime:obliquity_deg",
    "deeptime:slab_age_myr",
    "deeptime:crust_thickness_km",
    "deeptime:fault_density",
    "deeptime:orogenic_uplift_m_myr",
    "deeptime:rift_subsidence_m_myr",
    "deeptime:heat_flow_mw_m2",
    "deeptime:elevation_m",
    "deeptime:erosion_rate_m_myr",
    "deeptime:deposition_rate_m_myr",
    "deeptime:shelf",
    "deeptime:carbonate_platform",
    "deeptime:evaporite",
    "deeptime:euxinic_basin",
    "deeptime:coal_facies",
    "deeptime:source_rock_toc",
    "deeptime:burial_temp_c",
    "deeptime:reservoir_quality",
    "deeptime:seal",
    "deeptime:water_table_depth_m",
    "deeptime:precip_mm",
    "deeptime:t_mean_c",
    // Used for the frame's regime-based fields; not a deposit field itself.
    "surface:craton",
];

/// The step field of the inherited iron-formation basins under a stable interior (1 in a basin
/// whose crust is a craton this step; `deposits::bif_basins`); `flush` turns its frame mean into
/// `deeptime:inherited_bif`. Accumulated only in runs that inherit basins.
pub const BIF_BASIN_STEP: &str = "surface:bif_basin";

/// Fields computed once per frame.
pub const FRAME_FIELDS: [&str; 3] = [
    "deeptime:dist_litho_step_km",
    "deeptime:inherited_bif",
    "deeptime:topographic_gradient_m_km",
];

/// Time-weighted sums of the step fields since the frame began.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FrameAcc {
    pub t_start_years: f64,
    pub span_years: f64,
    pub sums: BTreeMap<String, Vec<f32>>,
    /// Where each cell's crust was at the start of the frame (composed over the frame's steps by
    /// [`FrameAcc::compose_back`]); empty while no step has moved anything (the identity).
    pub back: Vec<u32>,
}

impl FrameAcc {
    pub fn add(&mut self, name: &str, v: &[f32], dt: f64) {
        let w = dt as f32;
        match self.sums.get_mut(name) {
            Some(s) => s.par_iter_mut().zip(v.par_iter()).for_each(|(a, &x)| {
                if x.is_finite() {
                    *a += x * w;
                }
            }),
            // A new sum in one pass, the same values as zeros and then the add (P1-19: the zeros
            // were written on one thread). `0.0 +` keeps a product of −0 at +0, as the add did.
            None => {
                let s = v
                    .par_iter()
                    .map(|&x| if x.is_finite() { 0.0 + x * w } else { 0.0 })
                    .collect();
                self.sums.insert(name.to_owned(), s);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.span_years <= 0.0
    }

    /// Adds one step's crust motion: `nearest[c]` is the cell the material now at `c` came from
    /// (the same nearest-cell map that carries the strata log), `newborn[c]` marks crust made in
    /// this step. Composes it with the motion so far, so `back[c]` names the cell that held this
    /// crust when the frame began, and carries the sums with the crust (crust made this step
    /// starts from nothing).
    ///
    /// The sums are the crust's, not the cell's: a frame is read where each cell's crust is at its
    /// end (`deposits::fields`, the deposit ledger follows a deposit's crust frame by frame), so a
    /// frame's value at a cell must be what *that crust* went through. Summed at fixed cells, a
    /// frame mixed the crust that crossed the cell during the frame: crust moves 7 to 18 L1 cells
    /// in a 10-Myr frame at 30 to 70 mm/yr on the 16k map (the plate model's 200 km arc distance
    /// is 5 cells there), so an arc's frames carried the trench's fill and the forearc's
    /// stripping, and porphyries were buried and exhumed by tens of kilometres they never saw
    /// (2026-10-05, the erosion and burial audit).
    pub fn compose_back(&mut self, nearest: &[usize], newborn: &[bool]) {
        let n = nearest.len();
        if self.back.len() != n {
            self.back = (0..n as u32).collect();
        }
        let prev = std::mem::take(&mut self.back);
        self.back = nearest
            .par_iter()
            .zip(newborn.par_iter())
            .map(|(&q, &born)| if born { NO_ORIGIN } else { prev[q] })
            .collect();
        for sum in self.sums.values_mut() {
            if sum.len() != n {
                continue;
            }
            let old = std::mem::take(sum);
            *sum = nearest
                .par_iter()
                .zip(newborn.par_iter())
                .map(|(&q, &born)| if born { 0.0 } else { old[q] })
                .collect();
        }
    }

    /// Bytes held.
    pub fn bytes(&self) -> u64 {
        self.sums.values().map(|v| v.len() as u64 * 4).sum()
    }
}

/// What the step fields are made of (per cell).
pub struct StepInputs<'a> {
    pub flags: &'a [f32],
    pub crust_type: &'a [f32],
    pub orogeny_type: &'a [f32],
    pub orogeny_age: &'a [f32],
    pub regime: &'a [f32],
    pub uplift_mm_yr: &'a [f32],
    pub oxidation: &'a [f32],
    pub convergence: &'a [f32],
    pub obliquity: &'a [f32],
    pub slab_age: &'a [f32],
    pub heat_flow: &'a [f32],
    pub thickness_m: &'a [f32],
    /// Surface elevation relative to the sea of the time, m.
    pub rel_elev: &'a [f64],
    pub erosion_m_myr: &'a [f32],
    pub deposition_m_myr: &'a [f32],
    /// Facies kind of the step's deposit per cell and its share of the deposit.
    pub kind: &'a [u8],
    pub coal: &'a [bool],
    pub slope: &'a [f32],
    pub t_c: &'a [f32],
    pub precip_mm: &'a [f32],
    pub aridity: &'a [f32],
    pub log: &'a StrataLog,
    pub regolith_m: &'a [f32],
    pub dx: f64,
}

/// Facies kinds as stored per cell for the step (`facies::Kind` as u8, 255 = no deposit).
pub const KIND_NONE: u8 = 255;
pub const KIND_CARBONATE: u8 = 1;
pub const KIND_EVAPORITE: u8 = 2;
pub const KIND_EUXINIC: u8 = 3;

/// The tags [`column`] asks about, as bits by class ([`tag_bits`]).
const TAG_KARST: u8 = 1;
const TAG_SOLUBLE: u8 = 2;
const TAG_RESERVOIR: u8 = 4;
const TAG_SEAL: u8 = 8;

/// Each class's [`column`] tags as bits (P1-19: each was looked up by name, per layer, before).
fn tag_bits() -> &'static [u8] {
    static BITS: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
    BITS.get_or_init(|| {
        let tags = [
            (TAG_KARST, "karst"),
            (TAG_SOLUBLE, "soluble"),
            (TAG_RESERVOIR, "reservoir"),
            (TAG_SEAL, "seal"),
        ];
        (0..super::lithology::table().len())
            .map(|k| {
                tags.iter()
                    .filter(|(_, t)| has_tag(k as u16, t))
                    .fold(0u8, |b, (bit, _)| b | bit)
            })
            .collect()
    })
}

/// The column's facies shares in its top 1 km, and the petroleum-system layers.
fn column(log: &StrataLog, c: usize, cls: &Classes, bits: &[u8], regolith: f64) -> [f32; 7] {
    // A class outside the table carries no tag, as `has_tag` says.
    let has = |k: u16, bit: u8| bits.get(usize::from(k)).is_some_and(|&b| b & bit != 0);
    let col = log.column(c);
    let (mut carb, mut evap, mut eux, mut seen) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    // Top-down.
    let mut depth = regolith;
    let mut best_toc = 0.0f64;
    let mut best_depth = f64::NAN;
    let mut best_i = usize::MAX;
    for (i, l) in col.iter().enumerate().rev() {
        let t = f64::from(l.thickness_m);
        if seen < 1000.0 {
            let part = t.min(1000.0 - seen);
            seen += part;
            let k = l.class;
            if k == cls.limestone || k == cls.dolostone || has(k, TAG_KARST) {
                carb += part;
            }
            if k == cls.halite || k == cls.gypsum || has(k, TAG_SOLUBLE) {
                evap += part;
            }
            if k == cls.black_shale {
                eux += part;
            }
        }
        let toc = if l.class == cls.black_shale {
            5.0
        } else if l.class == cls.coal {
            3.0
        } else if l.class == cls.shale {
            1.0
        } else if l.class == cls.mudstone {
            0.8
        } else {
            0.0
        } * (t / 20.0).min(1.0);
        if toc > best_toc {
            best_toc = toc;
            best_depth = depth + 0.5 * t;
            best_i = i;
        }
        depth += t;
    }
    // Reservoir above the source (or anywhere without one), seal above the topmost reservoir.
    let above = if best_i == usize::MAX { 0 } else { best_i + 1 };
    let mut reservoir = 0.0f64;
    let mut top_res = usize::MAX;
    for (i, l) in col.iter().enumerate().skip(above) {
        if has(l.class, TAG_RESERVOIR) {
            reservoir += f64::from(l.thickness_m);
            top_res = i;
        }
    }
    let mut seal = 0.0f64;
    if top_res != usize::MAX {
        for l in &col[top_res + 1..] {
            if has(l.class, TAG_SEAL) {
                seal += f64::from(l.thickness_m);
            }
        }
    }
    let norm = seen.max(1e-9);
    [
        (carb / norm) as f32,
        (evap / norm) as f32,
        (eux / norm) as f32,
        best_toc as f32,
        best_depth as f32,
        (reservoir / 50.0).min(1.0) as f32,
        (seal / 20.0).min(1.0) as f32,
    ]
}

/// The step's fields, in [`STEP_FIELDS`] order, into `fields`: 26 grids that the caller keeps
/// between steps and every cell of which is written (P1-19: 26 new zeroed grids a step before).
pub fn step_fields(x: &StepInputs<'_>, cls: &Classes, fields: &mut Vec<Vec<f32>>) {
    let n = x.flags.len();
    let bits = tag_bits();
    let row = |c: usize| -> [f32; 26] {
        let flags = x.flags[c] as u32 as u8;
        let cont = x.crust_type[c] as u8 == CONTINENTAL;
        let reg = x.regime[c] as u8;
        let conv = x.convergence[c];
        let arc = if flags & ARC != 0 {
            (conv / 80.0).clamp(0.0, 1.0) * if cont { 1.0 } else { 0.7 }
        } else {
            0.0
        };
        let zone = flags & SUBDUCTION_ZONE != 0;
        let fault = match reg {
            regime::COLLISION => 1.0,
            regime::RIFT => 0.8,
            regime::ARC => 0.6,
            regime::TRENCH => 0.5,
            regime::BACK_ARC => 0.4,
            regime::HOTSPOT => 0.3,
            _ => {
                let age = x.orogeny_age[c];
                if age.is_finite() {
                    (0.6 * (-age / 50.0).exp()).max(0.05)
                } else {
                    0.05
                }
            }
        };
        let up = x.uplift_mm_yr[c] * 1000.0;
        let orogenic = if cont && x.orogeny_type[c] as u8 != 0 {
            up.max(0.0)
        } else {
            0.0
        };
        let rift = if cont { (-up).max(0.0) } else { 0.0 };
        let z = x.rel_elev[c];
        let marine = z < 0.0;
        let k = x.kind[c];
        let [carb, evap, eux, toc, src_depth, res, seal] =
            column(x.log, c, cls, bits, f64::from(x.regolith_m[c]));
        let step_share = |kind: u8| if k == kind { 1.0 } else { 0.0 };
        let t_c = x.t_c[c];
        let geotherm = x.heat_flow[c] / 1000.0 / 2.5; // °C per metre
        let burial = if src_depth.is_finite() {
            t_c.max(0.0) + geotherm * src_depth
        } else {
            t_c.max(0.0)
        };
        let wtd = if marine {
            0.0
        } else {
            5.0 + 150.0 * (1.0 - x.aridity[c].min(1.0)) + 0.05 * x.slope[c] * x.dx as f32
        };
        [
            arc,
            if arc > 0.0 { x.oxidation[c] } else { 0.0 },
            if zone { conv } else { 0.0 },
            if zone { x.obliquity[c] } else { 0.0 },
            if zone { x.slab_age[c] } else { 0.0 },
            x.thickness_m[c] / 1000.0,
            fault,
            orogenic,
            rift,
            x.heat_flow[c],
            z as f32,
            x.erosion_m_myr[c],
            x.deposition_m_myr[c],
            if marine && z > -200.0 { 1.0 } else { 0.0 },
            carb.max(step_share(KIND_CARBONATE)),
            evap.max(step_share(KIND_EVAPORITE)),
            eux.max(step_share(KIND_EUXINIC)),
            if x.coal[c] { 1.0 } else { 0.0 },
            toc,
            burial,
            res,
            seal,
            wtd,
            x.precip_mm[c],
            t_c,
            if reg == regime::CRATON { 1.0 } else { 0.0 },
        ]
    };
    // Each task fills its block of cells in all 26 fields at once (P1-17: the cells' rows were
    // collected and then read back field by field on one thread, 26 strided sweeps of the
    // whole raster every step).
    const BLOCK: usize = 4096;
    fields.resize_with(26, Vec::new);
    for f in fields.iter_mut().filter(|f| f.len() != n) {
        *f = crate::alloc::filled(n, 0.0);
    }
    let mut blocks: Vec<Vec<&mut [f32]>> = (0..n.div_ceil(BLOCK))
        .map(|_| Vec::with_capacity(26))
        .collect();
    for field in fields.iter_mut() {
        for (k, part) in field.chunks_mut(BLOCK).enumerate() {
            blocks[k].push(part);
        }
    }
    blocks
        .into_par_iter()
        .enumerate()
        .for_each(|(k, mut outs)| {
            let c0 = k * BLOCK;
            for i in 0..outs[0].len() {
                let r = row(c0 + i);
                for (out, v) in outs.iter_mut().zip(r) {
                    out[i] = v;
                }
            }
        });
}

/// Distance (km, block metric) from every cell to the nearest craton edge (a cell whose
/// craton flag differs from a neighbour's): two-pass chamfer (3-4) distance transform, x wrapping.
pub fn dist_to_edge(d: &Dims, craton: &[f32]) -> Vec<f32> {
    let (w, h) = (d.w, d.h);
    let inf = f32::INFINITY;
    let mut dist = vec![inf; w * h];
    let on = |c: usize| craton[c] >= 0.5;
    // The edge cells, in parallel (P1-17).
    dist.par_iter_mut().enumerate().for_each(|(c, x)| {
        if d.neighbours(c).any(|(m, _)| on(m) != on(c)) {
            *x = 0.0;
        }
    });
    let (a, b) = (1.0f32, std::f32::consts::SQRT_2);
    let at = |i: i64, j: i64| -> Option<usize> {
        if j < 0 || j >= h as i64 {
            None
        } else {
            Some(j as usize * w + i.rem_euclid(w as i64) as usize)
        }
    };
    // One cell of a sweep: the cell's value against its four already-swept neighbours, in the
    // order given. Cells away from the x edges reach them by index steps (P1-17: the same reads
    // and the same `min`s in the same order as through `at`, so the same bits).
    type Nb = [(i64, i64, f32); 4];
    let relax = |dist: &mut [f32], i: usize, j: usize, nb: &Nb, inner_row: bool| {
        let c = j * w + i;
        let mut v = dist[c];
        if i >= 1 && i + 1 < w && inner_row {
            for &(di, dj, s) in nb {
                let m = (c as i64 + dj * w as i64 + di) as usize;
                v = v.min(dist[m] + s);
            }
        } else {
            for &(di, dj, s) in nb {
                if let Some(m) = at(i as i64 + di, j as i64 + dj) {
                    v = v.min(dist[m] + s);
                }
            }
        }
        dist[c] = v;
    };
    let fwd: Nb = [(-1, 0, a), (-1, -1, b), (0, -1, a), (1, -1, b)];
    let bwd: Nb = [(1, 0, a), (1, 1, b), (0, 1, a), (-1, 1, b)];
    // Two sweeps each way; the wrap needs a second pass for distances across the seam.
    for _ in 0..2 {
        for j in 0..h {
            let inner_row = j >= 1;
            for i in 0..w {
                relax(&mut dist, i, j, &fwd, inner_row);
            }
        }
        for j in (0..h).rev() {
            let inner_row = j + 1 < h;
            for i in (0..w).rev() {
                relax(&mut dist, i, j, &bwd, inner_row);
            }
        }
    }
    let km = (d.dx / 1000.0) as f32;
    dist.iter()
        .map(|&x| if x.is_finite() { x * km } else { 3000.0 })
        .collect()
}

/// Gradient magnitude of the frame-mean land elevation, smoothed over ~50 km, m/km (0 at sea).
pub fn regional_gradient(d: &Dims, elev: &[f32]) -> Vec<f32> {
    let mut z: Vec<f64> = elev.par_iter().map(|&e| f64::from(e.max(0.0))).collect();
    let r = ((25_000.0 / d.dx).round() as usize).max(1);
    box_blur(d, &mut z, r, 2);
    let (w, h) = (d.w, d.h);
    (0..w * h)
        .into_par_iter()
        .map(|c| {
            if elev[c] <= 0.0 {
                return 0.0;
            }
            let (i, j) = (c % w, c / w);
            let e = z[j * w + (i + 1) % w];
            let wv = z[j * w + (i + w - 1) % w];
            let s = z[(j + 1).min(h - 1) * w + i];
            let nn = z[j.saturating_sub(1) * w + i];
            let gx = (e - wv) / (2.0 * d.dx);
            let gz = (s - nn) / (2.0 * d.dx);
            (gx.hypot(gz) * 1000.0) as f32
        })
        .collect()
}

/// Closes the frame: averages, the per-frame fields and the clocks.
pub fn flush(acc: &mut FrameAcc, d: &Dims, t_end_years: f64, epoch: &Epoch) -> FieldFrame {
    let span = acc.span_years.max(1e-9);
    let mut fields: BTreeMap<String, Vec<f32>> = BTreeMap::new();
    let mut craton = Vec::new();
    let mut basin = Vec::new();
    for (name, sum) in std::mem::take(&mut acc.sums) {
        let mean: Vec<f32> = sum
            .par_iter()
            .map(|&s| (f64::from(s) / span) as f32)
            .collect();
        if name == "surface:craton" {
            craton = mean;
        } else if name == BIF_BASIN_STEP {
            basin = mean;
        } else {
            fields.insert(name, mean);
        }
    }
    let n = d.cells();
    let window = epoch.inherited_bif_window() as f32; // ledger: ORE-020
    // The distance transform's sweeps are serial; the rest of the frame is made beside them
    // (P1-19).
    let (dist, (gradient, origin)) = rayon::join(
        || (craton.len() == n).then(|| dist_to_edge(d, &craton)),
        || {
            let gradient = fields
                .get(&format!("{P}elevation_m"))
                .map(|e| regional_gradient(d, e));
            (gradient, origin_offsets(acc, d))
        },
    );
    if let Some(dist) = dist {
        fields.insert(FRAME_FIELDS[0].into(), dist);
    }
    // Inherited BIF: the basins under a stable interior (`deposits::bif_basins`), × the share of
    // the acme before the run.
    fields.insert(
        FRAME_FIELDS[1].into(),
        if basin.len() == n {
            basin.par_iter().map(|&b| b * window).collect()
        } else {
            crate::alloc::filled(n, 0.0)
        },
    );
    if let Some(g) = gradient {
        fields.insert(FRAME_FIELDS[2].into(), g);
    }
    let [di, dj] = origin;
    fields.insert(ORIGIN_FIELDS[0].into(), di);
    fields.insert(ORIGIN_FIELDS[1].into(), dj);
    acc.back.clear();
    let frame = FieldFrame {
        t_years: t_end_years,
        dt_years: acc.span_years,
        fields,
        clocks: epoch
            .clocks(t_end_years - acc.span_years, t_end_years)
            .iter()
            .map(|&(k, v)| (k.to_owned(), v))
            .collect(),
    };
    acc.t_start_years = t_end_years;
    acc.span_years = 0.0;
    frame
}

/// The crust's motion over the frame, as offsets (small integers: exact in the 16-bit packing):
/// columns and rows from each cell back to where its crust was when the frame began.
fn origin_offsets(acc: &FrameAcc, d: &Dims) -> [Vec<f32>; 2] {
    let n = d.cells();
    if acc.back.len() != n {
        return [crate::alloc::filled(n, 0.0), crate::alloc::filled(n, 0.0)];
    }
    let w = d.w as i64;
    let (di, dj): (Vec<f32>, Vec<f32>) = acc
        .back
        .par_iter()
        .enumerate()
        .map(|(c, &b)| {
            if b == NO_ORIGIN {
                return (f32::NAN, f32::NAN);
            }
            let (bi, bj) = (i64::from(b) % w, i64::from(b) / w);
            let (ci, cj) = (c as i64 % w, c as i64 / w);
            let mut di = bi - ci;
            if di >= w / 2 {
                di -= w;
            } else if di < -w / 2 {
                di += w;
            }
            (di as f32, (bj - cj) as f32)
        })
        .unzip();
    [di, dj]
}

/// Latitude of row `j` of the planet grid.
pub fn row_lat(grid: &deeptime_format::GridDesc, j: usize) -> f64 {
    lat_of_row(grid, j as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The craton-edge distance as it was before P1-17 (every neighbour through the wrapping
    /// index function, the edge cells found serially).
    fn dist_to_edge_reference(d: &Dims, craton: &[f32]) -> Vec<f32> {
        let (w, h) = (d.w, d.h);
        let mut dist = vec![f32::INFINITY; w * h];
        let on = |c: usize| craton[c] >= 0.5;
        for (c, x) in dist.iter_mut().enumerate() {
            if d.neighbours(c).any(|(m, _)| on(m) != on(c)) {
                *x = 0.0;
            }
        }
        let (a, b) = (1.0f32, std::f32::consts::SQRT_2);
        let at = |i: i64, j: i64| -> Option<usize> {
            if j < 0 || j >= h as i64 {
                None
            } else {
                Some(j as usize * w + i.rem_euclid(w as i64) as usize)
            }
        };
        for _ in 0..2 {
            for j in 0..h as i64 {
                for i in 0..w as i64 {
                    let c = j as usize * w + i as usize;
                    let mut v = dist[c];
                    for (di, dj, s) in [(-1, 0, a), (-1, -1, b), (0, -1, a), (1, -1, b)] {
                        if let Some(m) = at(i + di, j + dj) {
                            v = v.min(dist[m] + s);
                        }
                    }
                    dist[c] = v;
                }
            }
            for j in (0..h as i64).rev() {
                for i in (0..w as i64).rev() {
                    let c = j as usize * w + i as usize;
                    let mut v = dist[c];
                    for (di, dj, s) in [(1, 0, a), (1, 1, b), (0, 1, a), (-1, 1, b)] {
                        if let Some(m) = at(i + di, j + dj) {
                            v = v.min(dist[m] + s);
                        }
                    }
                    dist[c] = v;
                }
            }
        }
        let km = (d.dx / 1000.0) as f32;
        dist.iter()
            .map(|&x| if x.is_finite() { x * km } else { 3000.0 })
            .collect()
    }

    /// P1-17: the craton-edge distance gives the reference's bits on random masks (blobs and
    /// salt-and-pepper), wrapping or not, on grids one to many cells wide, on any thread count.
    #[test]
    fn the_edge_distance_equals_the_reference_bit_for_bit() {
        let mut rng = crate::sphere::SplitMix64::new(17);
        let mut cases = 0;
        for &(w, h, wrap) in &[
            (64usize, 32usize, true),
            (65, 33, false),
            (1, 9, true),
            (2, 7, true),
            (9, 1, true),
            (3, 3, false),
            (200, 70, true),
        ] {
            for k in 0..4 {
                let d = Dims::new(w, h, 4800.0, wrap);
                let craton: Vec<f32> = match k {
                    0 => (0..w * h)
                        .map(|_| (rng.next_f64() < 0.5) as u8 as f32)
                        .collect(),
                    1 => (0..w * h)
                        .map(|_| (rng.next_f64() < 0.03) as u8 as f32)
                        .collect(),
                    2 => vec![1.0; w * h],
                    _ => (0..w * h)
                        .map(|c| {
                            let (i, j) = ((c % w) as f64, (c / w) as f64);
                            ((i - 0.3 * w as f64).hypot(j - 0.5 * h as f64) < 0.25 * w as f64) as u8
                                as f32
                        })
                        .collect(),
                };
                let want = dist_to_edge_reference(&d, &craton);
                for threads in [1, 4] {
                    let pool = rayon::ThreadPoolBuilder::new()
                        .num_threads(threads)
                        .build()
                        .unwrap();
                    let got = pool.install(|| dist_to_edge(&d, &craton));
                    let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
                    assert_eq!(bits(&got), bits(&want), "{w}x{h} wrap {wrap} mask {k}");
                }
                cases += 1;
            }
        }
        assert_eq!(cases, 28);
    }

    /// A frame's value at a cell is what the crust now there went through, not what crossed the
    /// cell: crust moving one cell a step towards a trench that moves with it (two cells ahead)
    /// never sees the trench's fill, although its final cell held the trench at the frame's start
    /// (the erosion and burial audit, 2026-10-05).
    #[test]
    fn frame_sums_follow_the_crust() {
        let n = 8;
        let dt = 2.0e6;
        let mut acc = FrameAcc::default();
        let no_birth = vec![false; n];
        // Material at c came from c − 1 (the first cell is new crust each step).
        let nearest: Vec<usize> = (0..n).map(|c| c.saturating_sub(1)).collect();
        let mut born = no_birth.clone();
        born[0] = true;
        for s in 0..3usize {
            if s > 0 {
                acc.compose_back(&nearest, &born);
            }
            // 1000 m/Myr of trench fill two cells ahead of the parcel (the parcel is at s + 1).
            let mut dep = vec![0.0f32; n];
            dep[s + 3] = 1000.0;
            // The parcel's own arc erosion, 50 m/Myr.
            let mut ero = vec![0.0f32; n];
            ero[s + 1] = 50.0;
            acc.add("deeptime:deposition_rate_m_myr", &dep, dt);
            acc.add("deeptime:erosion_rate_m_myr", &ero, dt);
            acc.span_years += dt;
        }
        let mean = |name: &str, c: usize| acc.sums[name][c] / acc.span_years as f32;
        // The parcel ends at cell 3, which held the trench at the frame's start: summed at the
        // cell, its frame would show 333 m/Myr of fill.
        assert_eq!(mean("deeptime:deposition_rate_m_myr", 3), 0.0);
        assert!((mean("deeptime:erosion_rate_m_myr", 3) - 50.0).abs() < 1e-3);
        // The trench's own crust (now at cell 5) saw its fill all frame long.
        assert!((mean("deeptime:deposition_rate_m_myr", 5) - 1000.0).abs() < 1e-2);
        // Where each cell's crust was when the frame began.
        assert_eq!(acc.back[3], 1);
        assert_eq!(acc.back[1], NO_ORIGIN);
    }

    /// A frame of `span_myr` ending at `t_end_myr` with two craton cells of four, the first of
    /// them in an inherited iron-formation basin (`deposits::bif_basins`).
    fn frame_at(t_end_myr: f64, span_myr: f64, epoch: &Epoch) -> FieldFrame {
        let d = Dims::new(2, 2, 1000.0, true);
        let mut acc = FrameAcc::default();
        let dt = span_myr * 1.0e6;
        acc.add("surface:craton", &[1.0, 1.0, 0.0, 0.0], dt);
        if epoch.inherits_bif() {
            acc.add(BIF_BASIN_STEP, &[1.0, 0.0, 0.0, 0.0], dt);
        }
        acc.span_years = dt;
        flush(&mut acc, &d, t_end_myr * 1.0e6, epoch)
    }

    /// Stage 1 found the clocks fixed at 1, 1, 0 and `inherited_bif` at 0 on every recorded
    /// history (ledger ORE-017, ORE-020): no BIF could form and coal had no date limit. The frames
    /// now carry Earth's dates for the planet's age.
    #[test]
    fn frames_carry_dated_clocks_and_inherited_bif() {
        let earth = |start_myr: f64| Epoch {
            present_age_gyr: crate::deposits::clocks::EARTH_AGE_GYR,
            run_start_years: -start_myr * 1.0e6,
            inherited_bif: 1.0,
        };
        // A 1,000 Myr run: a Neoproterozoic frame has no coal-forming plants, a Carboniferous one has.
        let e = earth(1000.0);
        let old = frame_at(-900.0, 50.0, &e);
        assert_eq!(old.clocks["deeptime:clock/land_plants"], 0.0);
        assert_eq!(old.clocks["deeptime:clock/oxygenated_oceans"], 1.0);
        assert_eq!(old.clocks["deeptime:clock/bif_acme"], 0.0);
        let young = frame_at(-300.0, 50.0, &e);
        assert_eq!(young.clocks["deeptime:clock/land_plants"], 1.0);
        // The run starts after the acme: the craton cell in a basin inherits BIF; the other craton
        // cell does not (before the patch field, Ben 2026-10-02, every craton cell did).
        assert_eq!(
            old.fields["deeptime:inherited_bif"],
            vec![1.0, 0.0, 0.0, 0.0]
        );
        // A 3,000 Myr run holds the acme: its frames there carry the clock, no inherited BIF.
        let e = earth(3000.0);
        let acme = frame_at(-2300.0, 50.0, &e);
        assert_eq!(acme.clocks["deeptime:clock/bif_acme"], 1.0);
        assert_eq!(acme.clocks["deeptime:clock/oxygenated_oceans"], 0.0);
        assert_eq!(acme.fields["deeptime:inherited_bif"], vec![0.0; 4]);
    }
}
