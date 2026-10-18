// SPDX-License-Identifier: LGPL-3.0-or-later
//! Glacial erosion, till and fjords on the surface's grids (Ben, 2026-09-29, "Glacial erosion
//! landforms"): where ice sits, it slides over its bed and wears it down; the debris it carries
//! melts out as till where the ice melts, and what the meltwater takes goes on to lakes and the
//! sea. The deep-time surface (L1, `surface/mod.rs`) and the young landscape (L2, `young.rs`) run
//! the same model; the amplifier shapes the valleys it leaves at block scale
//! (`amplify/glacial.rs`).
//!
//! # The model (per LEM step, `substeps` times)
//! 1. **Mass balance**, linear in height (Oerlemans 2011, *Minimal Glacier Models*, whose generic
//!    balance gradient β = 0.005 m of ice per m per year is the default): b = β·(z − ELA) m of
//!    ice per year, capped at the precipitation (snow can add no more than falls) and
//!    `max_accumulation_m_yr`, and at `max_ablation_m_yr` below. The ELA is the climate's (P1-05:
//!    `climate/anomaly/ela_m`, Ohmura et al. 1992), read through [`source`].
//! 2. **Ice flux** routed down the fluvial D8 tree of the step, as Hergarten 2021 ("Modeling
//!    glacial and fluvial landform evolution at large scales using a stream-power approach",
//!    Earth Surf. Dynam. 9:937–952) routes ice on the single-flow-direction network (goSPL's
//!    documentation describes the same idea with multiple directions; goSPL is GPL and only its
//!    documentation was read, for comparison): each cell passes on what reaches it plus its
//!    accumulation, less what melts in it; a glacier ends where nothing is left (its terminus)
//!    or at the sea (a tidewater front).
//! 3. **Ice thickness** from the shallow-ice approximation (Cuffey & Paterson 2010, *The Physics
//!    of Glaciers*, 4th ed., §8): the flux per unit width of a glacier of thickness H on an
//!    ice-surface slope S is q = (2A/(n+2))·(ρgS)ⁿ·Hⁿ⁺² + u_s·H with Glen's n = 3 and A = 2.4·10⁻²⁴
//!    Pa⁻³ s⁻¹ (temperate ice), and sliding u_s = B·τ_b³ (a Weertman-type law, τ_b = ρgHS) where
//!    the bed is at the melting point. A valley glacier is `1 / form_ratio` times as wide as it
//!    is thick (a design value of 0.3; Aletsch Glacier is ~900 m thick and ~1.5 km wide, and
//!    Hergarten 2021 also takes thickness proportional to width), up to the cell; the ice
//!    surface of each cell is solved from the one downstream (receivers first), so ice can stand
//!    over a bed lower than its outlet — an overdeepening.
//! 4. **Erosion** E = K_g·u_s^l, the abrasion law of glacial landscape models (Harbor 1992, GSA
//!    Bull. 104:1364–1375, used l = 2; Humphrey & Raymond 1994, J. Glaciol. 40:539–552, measured
//!    E ≈ 10⁻⁴·u_s under the surging Variegated Glacier, l = 1), under the glacier's whole
//!    width: a glacier narrower than its cell lowers the cell by its share of it, a wider one (up
//!    to three cells either side) spreads the erosion over the cells within half its width of
//!    the centre line that lie under its surface, most at the centre (a trough, not a one-cell
//!    slot). K_g defaults to 10⁻⁵ with l = 1: temperate valley glaciers sliding
//!    100–200 m/yr then cut 1–2 mm/yr and fast outlet glaciers 10–100 mm/yr, the rates Hallet,
//!    Hunter & Bogen 1996 (Global Planet. Change 12:213–235) compiled (0.01 mm/yr polar, 0.1
//!    Norway, ~1 the Swiss Alps, 10–100 south-east Alaska); a surge's 10⁻⁴ overstates the
//!    long-term rate. **Cold beds do not slide**: sliding
//!    and erosion need a mean annual temperature above about −14…−4 °C or ice thick enough to
//!    insulate its bed (800–2,500 m), so polar ice caps protect their plateaus while their
//!    outlet glaciers cut troughs (Sugden & John 1976's selective linear erosion). **Floating ice
//!    does not erode**: a cell below sea level whose ice is thinner than ρ_w/ρ_i times the water
//!    depth is afloat (the fjord's floor stops deepening there). **Overdeepening limits itself**:
//!    where the bed rises towards the outlet more steeply than `supercooling_ratio` times the
//!    ice-surface slope, basal meltwater freezes and the bed is not eroded (glaciohydraulic
//!    supercooling: 1.2–1.7 in Alley et al. 1998's theory, J. Glaciol. 44; eroding beds
//!    settle about 1.5 times as steep as the ice surface, Alley et al. 2003, Nature
//!    424:758–760).
//! 5. **Debris** travels with the ice and melts out in proportion to the ice that melts in a cell
//!    (all of it at a terminus). `till_share` of it stays as till (moraines, ground moraine; the
//!    rest is meltwater load, which most glacial sediment is, Hallet et al. 1996); the meltwater
//!    load settles in lakes it crosses and reaches the sea at its river's mouth. Ice-borne debris
//!    reaching the sea at a tidewater front is glaciomarine diamict. **Mass is kept exactly**:
//!    eroded = till + lake + sink + sea loads (tested).
//!
//! Fjords follow: where a glacier reaches the coast its surface stands on the sea at the front,
//! the ice behind is thick and fast, and it cuts its bed below sea level until it floats or the
//! sill it has to climb stops it.
//!
//! **Determinism.** The passes are serial walks of the stack (up: donors before receivers;
//! down: receivers first), so results do not depend on the thread count.

use std::sync::atomic::{AtomicBool, Ordering};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::grid::Dims;
use super::pflood::{BASE, LAKE_TOL_M};
use super::routing::Routing;
use super::routing::forest::{NONE, SharedF64};
use crate::climate::ebm::smoothstep;
use crate::error::CoreError;

/// Where the climate's ice comes from: the one place that names the climate stage's fields, so
/// that switching to the ice-age cycles' fields (Ben, 2026-09-29) is a change of these names.
pub mod source {
    use crate::climate::field;
    /// Equilibrium-line altitude above sea level of every cell (P1-05; carried with the plates
    /// between snapshots), m.
    pub const ELA: &str = field::ELA_ALL;
    /// The climate's land ice at the end of the step, m (the deep-time climate, FORMAT.md "The
    /// climate's ice fields"): where it is thick, ice and not rivers erodes.
    pub const ICE_THICKNESS: &str = field::ICE_THICKNESS;
    /// The thickest the climate's ice stood during the step, m: the deep-time glaciers are
    /// allowed only where it had ice at some time in the step.
    pub const ICE_MAX_THICKNESS: &str = field::ICE_MAX_THICKNESS;
    /// Share of the step's time a cell was under the climate's ice: the glaciers erode for that
    /// share of the step.
    pub const ICE_TIME_SHARE: &str = field::ICE_TIME_SHARE;
    /// The last-glacial-maximum extent at the present (0/1): the young landscape's glaciers
    /// are allowed only there.
    pub const ICE_MAX_EXTENT: &str = field::ICE_MAX_EXTENT;
    /// The ELA of the glacial state, if the climate ever publishes one, m above sea level (not
    /// written today); else the young landscape lowers the present one by `young_ela_drop_m`.
    pub const ELA_GLACIAL: &str = "climate/ela_glacial_m";
}

/// Ice density, water density, kg/m³, and seconds per year.
const RHO_ICE: f64 = 917.0;
const RHO_WATER: f64 = 1028.0;
const SECONDS_PER_YEAR: f64 = 3.155_76e7;

/// The `surface.glacial` section: every key optional.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct GlacialParams {
    /// Multiplier on the glacial erosion coefficient: 0 lets glaciers sit without eroding, 1 is
    /// the calibrated value (the editor's "Glacial erosion").
    pub strength: f64,
    /// K_g of E = K_g·u_s^l, (m/yr)^(1−l).
    pub erosion_k: f64,
    /// l.
    pub erosion_exponent: f64,
    /// Mass-balance gradient β, m of ice per m of height per year.
    pub mass_balance_gradient: f64,
    /// Accumulation cap, m/yr (also never more than the precipitation).
    pub max_accumulation_m_yr: f64,
    /// Ablation cap, m/yr.
    pub max_ablation_m_yr: f64,
    /// Glen's rate factor A, Pa⁻³ s⁻¹ (Cuffey & Paterson 2010: 2.4·10⁻²⁴ at the melting point).
    pub rate_factor: f64,
    /// Sliding speed at a basal shear stress of 100 kPa, m/yr (u_s = B·τ_b³).
    pub sliding_m_yr: f64,
    /// Depth/width of a valley glacier's cross-section.
    pub form_ratio: f64,
    /// Mean annual surface temperature over which the bed goes from frozen to melting, °C.
    pub warm_bed_c: [f64; 2],
    /// Ice thickness over which a cold bed is insulated to the melting point, m.
    pub warm_bed_thickness_m: [f64; 2],
    /// A bed rising towards the outlet more steeply than this times the ice-surface slope is not
    /// eroded (Alley et al. 2003: 1.2–1.7).
    pub supercooling_ratio: f64,
    /// Share of melted-out debris left as till; the rest is meltwater load.
    pub till_share: f64,
    /// Till a cell takes per substep at most, m (the rest goes on with the meltwater).
    pub max_till_m: f64,
    /// A substep lowers the bed by at most this share of the ice thickness.
    pub max_erosion_share: f64,
    /// Ice-flux solves per LEM step.
    pub substeps: u32,
    /// The young landscape's glacial ELA is the present one lowered by this much, m, unless the
    /// climate publishes a glacial ELA: the "average Quaternary glacial conditions" of Porter
    /// 1989 (Quat. Res. 32:245–261: snowlines some 500 m lower than today; ~900 m at the last
    /// glacial maximum, Porter 2001).
    pub young_ela_drop_m: f64,
    /// How much wider than a real glacier the young landscape draws its troughs (the editor's
    /// "Glacial valley width"; Ben, 2026-09-30: default wider, 1 is physical). At the game's
    /// scale (300 m a block across) a real 2-km trough is 7 blocks wide and, 600 m deep, 17 blocks
    /// deep at 35 m up (30 at the earlier 20 m): it reads as a gorge; this widens the glaciers'
    /// footprint and the spread of their erosion on L2, and so the U the amplifier cuts under
    /// them. The default 3 draws that trough 20 blocks wide, about as wide as deep at 35 m (two
    /// thirds at 20 m); kept with the 35 m scale (2026-10-02). The deep-time surface always uses
    /// 1 (physics).
    pub valley_width: f64,
}

impl Default for GlacialParams {
    fn default() -> Self {
        GlacialParams {
            strength: 1.0,
            erosion_k: 1.0e-5,
            erosion_exponent: 1.0,
            mass_balance_gradient: 0.005,
            max_accumulation_m_yr: 2.0,
            max_ablation_m_yr: 10.0,
            rate_factor: 2.4e-24,
            sliding_m_yr: 30.0,
            form_ratio: 0.3,
            warm_bed_c: [-14.0, -4.0],
            warm_bed_thickness_m: [800.0, 2500.0],
            supercooling_ratio: 1.5,
            till_share: 0.3,
            max_till_m: 100.0,
            max_erosion_share: 0.5,
            substeps: 4,
            young_ela_drop_m: 500.0,
            valley_width: 3.0,
        }
    }
}

impl GlacialParams {
    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        let bad = |k: &str, why: &str| CoreError::preset(format!("$.surface.glacial.{k}"), why);
        let fin = |v: f64| v.is_finite();
        if !(fin(self.strength) && (0.0..=10.0).contains(&self.strength)) {
            return Err(bad("strength", "must be in [0, 10]"));
        }
        if !(fin(self.erosion_k) && self.erosion_k >= 0.0) {
            return Err(bad("erosion_k", "must be ≥ 0"));
        }
        if !(0.5..=4.0).contains(&self.erosion_exponent) {
            return Err(bad("erosion_exponent", "must be in [0.5, 4]"));
        }
        for (v, k) in [
            (self.mass_balance_gradient, "mass_balance_gradient"),
            (self.max_accumulation_m_yr, "max_accumulation_m_yr"),
            (self.max_ablation_m_yr, "max_ablation_m_yr"),
            (self.rate_factor, "rate_factor"),
            (self.form_ratio, "form_ratio"),
            (self.supercooling_ratio, "supercooling_ratio"),
            (self.max_till_m, "max_till_m"),
        ] {
            if !(fin(v) && v > 0.0) {
                return Err(bad(k, "must be > 0"));
            }
        }
        if !(fin(self.sliding_m_yr) && self.sliding_m_yr >= 0.0) {
            return Err(bad("sliding_m_yr", "must be ≥ 0"));
        }
        if self.form_ratio > 2.0 {
            return Err(bad("form_ratio", "must be at most 2"));
        }
        if !self.warm_bed_c.iter().all(|x| x.is_finite())
            || self.warm_bed_c[0] >= self.warm_bed_c[1]
        {
            return Err(bad("warm_bed_c", "must be [colder, warmer]"));
        }
        if !self.warm_bed_thickness_m.iter().all(|x| x.is_finite())
            || self.warm_bed_thickness_m[0] >= self.warm_bed_thickness_m[1]
        {
            return Err(bad("warm_bed_thickness_m", "must be [thinner, thicker]"));
        }
        if !(0.0..=1.0).contains(&self.till_share) {
            return Err(bad("till_share", "must be in [0, 1]"));
        }
        if !(fin(self.max_erosion_share)
            && self.max_erosion_share > 0.0
            && self.max_erosion_share <= 1.0)
        {
            return Err(bad("max_erosion_share", "must be in (0, 1]"));
        }
        if !(1..=50).contains(&self.substeps) {
            return Err(bad("substeps", "must be in [1, 50]"));
        }
        if !(fin(self.young_ela_drop_m) && (-2000.0..=3000.0).contains(&self.young_ela_drop_m)) {
            return Err(bad("young_ela_drop_m", "must be in [−2000, 3000]"));
        }
        if !(fin(self.valley_width) && (1.0..=8.0).contains(&self.valley_width)) {
            return Err(bad("valley_width", "must be in [1, 8]"));
        }
        Ok(())
    }

    /// True when glaciers erode at all.
    pub fn active(&self) -> bool {
        self.strength > 0.0 && self.erosion_k > 0.0
    }
}

/// The climate a glacier step sees, per cell.
pub struct IceClimate<'a> {
    /// Equilibrium-line altitude, m on the elevation datum (sea level + the ELA above it); NaN
    /// where the climate says nothing.
    pub ela_z: &'a [f64],
    /// Annual precipitation, m/yr.
    pub precip_m_yr: &'a [f32],
    /// Mean annual surface temperature, °C.
    pub t_c: &'a [f32],
    /// Where ice may be at all (> 0), when the climate has its own ice model.
    pub allowed: Option<&'a [f32]>,
    /// Share of the step the ice stands (0–1; erosion scales with it), when the climate says.
    pub time_share: Option<&'a [f32]>,
}

/// The ELA on the elevation datum from a world's ELA field (m above sea level), lowered by
/// `drop_m`; None when the field is missing or of another size.
pub fn ela_z_from(
    ela_above_sea: Option<&[f32]>,
    n: usize,
    sea: f64,
    drop_m: f64,
) -> Option<Vec<f64>> {
    let v = ela_above_sea.filter(|v| v.len() == n)?;
    Some(
        v.iter()
            .map(|&e| {
                if e.is_finite() {
                    sea + f64::from(e) - drop_m
                } else {
                    f64::NAN
                }
            })
            .collect(),
    )
}

/// The ice of one solve.
#[derive(Clone, Debug, Default)]
pub struct IceState {
    /// Ice flux through the cell (mean of what enters and leaves), m³/yr.
    pub q: Vec<f64>,
    /// Share of what reaches the cell that melts in it.
    pub melt_frac: Vec<f64>,
    /// Centre-line ice thickness, m (0 without ice).
    pub thick: Vec<f64>,
    /// Ice surface on the centre line, m (NaN without ice).
    pub surface: Vec<f64>,
    /// Ice surface over the glaciers' whole width, m (NaN without ice): a glacier wider than its
    /// cell covers the cells beside its centre line that lie under its surface.
    pub footprint: Vec<f64>,
    /// Share of the cell the glacier covers.
    pub cover: Vec<f64>,
    /// Glacier width, m (wider than a cell: it erodes the cells beside its centre line too).
    pub width: Vec<f64>,
    /// Sliding speed, m/yr.
    pub sliding: Vec<f64>,
    /// Ice-surface slope towards the receiver.
    pub slope: Vec<f64>,
    /// Cells with ice.
    pub cells: usize,
    /// The cells whose thickness is not zero or less (with ice), in stack order: the serial passes
    /// over the glaciers visit only these (P1-19).
    pub glaciated: Vec<u32>,
}

/// Per-cell results of a whole step (every substep added up), m, and the loads that reached base
/// cells, m³.
#[derive(Clone, Debug, Default)]
pub struct GlacialOut {
    pub eroded: Vec<f64>,
    pub till: Vec<f64>,
    /// Meltwater load laid down in lakes, m.
    pub lake: Vec<f64>,
    /// Ice-borne debris reaching each base cell (a tidewater front), m³.
    pub sea_ice_load: Vec<f64>,
    /// Meltwater load reaching each base cell, m³.
    pub sea_melt_load: Vec<f64>,
    /// The last substep's ice.
    pub ice: IceState,
    /// The base cells with a load, ascending: [`GlacialOut::sea_ice_load`] and
    /// [`GlacialOut::sea_melt_load`] are 0 everywhere else.
    pub sea_cells: Vec<u32>,
    /// Totals, m³.
    pub eroded_m3: f64,
    pub till_m3: f64,
    pub lake_m3: f64,
    pub sink_m3: f64,
    pub sea_m3: f64,
    /// [`step`]'s working arrays, kept between steps; all 0 between them.
    work: Work,
}

/// [`step`]'s per-cell working arrays (all 0 between its substeps: each substep clears what it
/// wrote) and the lists of what it wrote.
#[derive(Clone, Debug, Default)]
struct Work {
    /// Debris and meltwater load coming in from the donors, m³.
    lin: Vec<f64>,
    min: Vec<f64>,
    /// Erosion depth, m.
    e: Vec<f64>,
    /// Till and lake deposits, m³.
    till_add: Vec<f64>,
    lake_add: Vec<f64>,
    /// Eroded volume each glacier takes up, m³.
    src: Vec<f64>,
    /// Stack positions downstream of an eroding cell, as bits.
    marked: Vec<u64>,
    /// Cells that took up debris.
    eroding: Vec<u32>,
    /// Cells any array above was written at (repeats allowed until sorted).
    written: Vec<u32>,
}

impl GlacialOut {
    /// Gives back [`step`]'s working arrays. A caller that steps once and then goes on with other
    /// work (the young stage) would otherwise carry six grids past the step, which used to free
    /// them as it returned (P1-19).
    pub fn release_work(&mut self) {
        self.work = Work::default();
    }

    fn reset(&mut self, n: usize) {
        for v in [
            &mut self.eroded,
            &mut self.till,
            &mut self.lake,
            &mut self.sea_ice_load,
            &mut self.sea_melt_load,
        ] {
            crate::alloc::fill(v, n, 0.0);
        }
        self.sea_cells.clear();
        self.eroded_m3 = 0.0;
        self.till_m3 = 0.0;
        self.lake_m3 = 0.0;
        self.sink_m3 = 0.0;
        self.sea_m3 = 0.0;
    }
}

/// The resolved constants of the flow law.
#[derive(Clone, Copy, Debug)]
struct Flow {
    /// (2A/(n+2))·(ρg)³, per year: q_d = c_def·S³·H⁵.
    c_def: f64,
    /// B·(ρg)³: u_s = c_slide·(H·S)³.
    c_slide: f64,
    form: f64,
    dx: f64,
    warm_h: [f64; 2],
}

impl Flow {
    fn new(p: &GlacialParams, gravity: f64, dx: f64) -> Flow {
        let rho_g = RHO_ICE * gravity;
        let a_yr = p.rate_factor * SECONDS_PER_YEAR;
        let b = p.sliding_m_yr / 1.0e15; // m/yr per Pa³ (100 kPa cubed)
        Flow {
            c_def: 2.0 * a_yr / 5.0 * rho_g.powi(3),
            c_slide: b * rho_g.powi(3),
            form: p.form_ratio,
            dx,
            warm_h: p.warm_bed_thickness_m,
        }
    }

    /// Glacier width, m (at most [`MAX_SPREAD`] cells either side of its centre line).
    #[inline]
    fn width(&self, h: f64) -> f64 {
        (h / self.form).clamp(1.0e-3, self.dx * (2 * MAX_SPREAD + 1) as f64)
    }

    /// Share of the sliding a bed of mean temperature share `warm_t` (0 frozen … 1 melting) under
    /// `h` metres of ice allows.
    #[inline]
    fn warm(&self, warm_t: f64, h: f64) -> f64 {
        warm_t.max(smoothstep(self.warm_h[0], self.warm_h[1], h))
    }

    /// Ice flux of a glacier of centre-line thickness `h` on surface slope `s`, m³/yr.
    #[inline]
    fn flux(&self, h: f64, s: f64, warm_t: f64) -> f64 {
        if h <= 0.0 || s <= 0.0 {
            return 0.0;
        }
        let s3 = s * s * s;
        let h4 = h * h * h * h;
        let q = self.c_def * s3 * h4 * h + self.warm(warm_t, h) * self.c_slide * s3 * h4;
        self.width(h) * q
    }

    /// Sliding speed, m/yr.
    #[inline]
    fn sliding(&self, h: f64, s: f64, warm_t: f64) -> f64 {
        let hs = (h * s).max(0.0);
        self.warm(warm_t, h) * self.c_slide * hs * hs * hs
    }
}

/// The surface of a glacier carrying `q` from a cell of bed `bed`, `len` metres from a point whose
/// surface is `z_ref`: the lowest surface whose flux reaches `q` (bisection; the flux grows with
/// the surface). Returns (surface, thickness, slope).
fn solve_surface(
    flow: &Flow,
    q: f64,
    bed: f64,
    z_ref: f64,
    len: f64,
    warm_t: f64,
) -> (f64, f64, f64) {
    let lo0 = bed.max(z_ref);
    let (mut lo, mut hi) = (lo0, lo0 + 8000.0);
    let eval = |zs: f64| {
        let h = zs - bed;
        let s = (zs - z_ref) / len;
        flow.flux(h, s, warm_t)
    };
    if eval(hi) < q {
        let h = hi - bed;
        return (hi, h, (hi - z_ref) / len);
    }
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if eval(mid) < q {
            lo = mid;
        } else {
            hi = mid;
        }
        if hi - lo < 0.01 {
            break;
        }
    }
    (hi, hi - bed, (hi - z_ref) / len)
}

/// Mass balance of a cell, m/yr of ice.
#[inline]
fn balance(p: &GlacialParams, z: f64, ela: f64, precip_m_yr: f64) -> f64 {
    if !ela.is_finite() {
        return -p.max_ablation_m_yr;
    }
    let b = p.mass_balance_gradient * (z - ela);
    if b > 0.0 {
        b.min(p.max_accumulation_m_yr).min(precip_m_yr.max(0.0))
    } else {
        b.max(-p.max_ablation_m_yr)
    }
}

/// A glacier is at most this many cells wide either side of its centre line.
const MAX_SPREAD: i64 = 3;

/// Cell `c` moved by (`di`, `dj`) cells: x wraps when the grid does, rows stop.
#[inline]
fn offset(d: &Dims, c: usize, di: i64, dj: i64) -> Option<usize> {
    let (i, j) = ((c % d.w) as i64 + di, (c / d.w) as i64 + dj);
    if j < 0 || j >= d.h as i64 {
        return None;
    }
    let i = if d.wrap_x {
        i.rem_euclid(d.w as i64)
    } else if i < 0 || i >= d.w as i64 {
        return None;
    } else {
        i
    };
    Some(j as usize * d.w + i as usize)
}

/// Ice at least this flux (m³/yr) counts as a glacier (a cell of 1 km² accumulating 1 mm/yr).
const Q_MIN: f64 = 1.0e3;

/// Solves the ice of the surface `h` routed by `r` (the step's D8 tree on the filled surface).
///
/// The flux pass (donors before receivers) and the surface pass (receivers first) walk the
/// routing's drainage forest in parallel (`routing::forest`, P1-17): a cell's incoming flux is a
/// gather over its donors in ascending cell index, the order in which the serial reverse-stack
/// walk added them, and every other value is the cell's own, so the result is [`ice_serial`]'s bit
/// for bit (the tests compare them).
#[allow(clippy::too_many_arguments)]
pub fn ice(
    d: &Dims,
    r: &Routing,
    kind: &[u8],
    h: &[f64],
    filled: &[f64],
    sea: f64,
    clim: &IceClimate<'_>,
    p: &GlacialParams,
    gravity: f64,
    st: &mut IceState,
) {
    let f = &r.forest;
    let l = r.stack.len();
    if f.len() != l || f.pos.len() != d.cells() {
        return ice_serial(d, r, kind, h, filled, sea, clim, p, gravity, st);
    }
    let n = d.cells();
    let a = d.cell_area();
    let flow = Flow::new(p, gravity, d.dx);
    st.cells = 0;
    st.glaciated.clear();
    // Up: fluxes, by position in the stack. `out` is what a cell passes to its receiver (0 when
    // nothing reaches it: adding +0 to a sum of non-negative terms changes no bit).
    let out = SharedF64::new(l, 0.0);
    let q = SharedF64::new(l, 0.0);
    let melt_frac = SharedF64::new(l, 0.0);
    let any = AtomicBool::new(false);
    f.up(UP_CHUNK, |i| {
        let c = r.stack[i] as usize;
        let mut qin = 0.0;
        for &dp in &f.dpos[f.dptr[i] as usize..f.dptr[i + 1] as usize] {
            qin += out.get(dp as usize);
        }
        let allowed = clim.allowed.is_none_or(|m| m[c] > 0.0);
        let b = if allowed {
            balance(p, h[c], clim.ela_z[c], f64::from(clim.precip_m_yr[c]))
        } else {
            -p.max_ablation_m_yr
        };
        let acc = b * a;
        let supply = qin + acc.max(0.0);
        if supply <= 0.0 {
            return;
        }
        any.store(true, Ordering::Relaxed);
        let melt = (-acc).max(0.0).min(supply);
        let o = supply - melt;
        melt_frac.set(i, melt / supply);
        q.set(i, 0.5 * (supply + o));
        out.set(i, o);
    });
    // Every cell in one parallel pass: its stack position's value, `none` off the stack.
    let pos = &f.pos;
    let gather = |dst: &mut Vec<f64>, src: &SharedF64, none: f64| {
        (0..n)
            .into_par_iter()
            .map(|c| {
                let i = pos[c];
                if i != NONE { src.get(i as usize) } else { none }
            })
            .collect_into_vec(dst);
    };
    gather(&mut st.melt_frac, &melt_frac, 0.0);
    if !any.load(Ordering::Relaxed) {
        gather(&mut st.q, &q, 0.0);
        for v in [
            &mut st.thick,
            &mut st.cover,
            &mut st.width,
            &mut st.sliding,
            &mut st.slope,
        ] {
            crate::alloc::fill(v, n, 0.0);
        }
        crate::alloc::fill(&mut st.surface, n, f64::NAN);
        crate::alloc::fill(&mut st.footprint, n, f64::NAN);
        return;
    }
    // Down: surfaces, receivers first.
    let thick = SharedF64::new(l, 0.0);
    let surface = SharedF64::new(l, f64::NAN);
    let slope = SharedF64::new(l, 0.0);
    let width = SharedF64::new(l, 0.0);
    let cover = SharedF64::new(l, 0.0);
    let sliding = SharedF64::new(l, 0.0);
    f.down(DOWN_GRAIN, |i| {
        let c = r.stack[i] as usize;
        let qc = q.get(i);
        if qc < Q_MIN {
            q.set(i, 0.0);
            return;
        }
        let rc = r.rcv[c] as usize;
        let rp = f.rpos[i];
        let (z_ref, len) = if rc == c {
            (filled[c], d.dx)
        } else if kind[rc] == BASE {
            (sea, f64::from(r.len[c]).max(1.0))
        } else if rp != NONE && thick.get(rp as usize) > 0.0 {
            (
                surface.get(rp as usize).max(filled[rc]),
                f64::from(r.len[c]).max(1.0),
            )
        } else {
            (filled[rc], f64::from(r.len[c]).max(1.0))
        };
        let warm_t = smoothstep(p.warm_bed_c[0], p.warm_bed_c[1], f64::from(clim.t_c[c]));
        let (zs, hh, sl) = solve_surface(&flow, qc, h[c], z_ref, len, warm_t);
        if hh <= 0.0 {
            return;
        }
        surface.set(i, zs);
        thick.set(i, hh);
        slope.set(i, sl);
        let wd = flow.width(hh);
        width.set(i, wd);
        cover.set(i, (wd / d.dx).min(1.0));
        sliding.set(i, flow.sliding(hh, sl, warm_t));
    });
    gather(&mut st.q, &q, 0.0);
    gather(&mut st.thick, &thick, 0.0);
    gather(&mut st.surface, &surface, f64::NAN);
    gather(&mut st.slope, &slope, 0.0);
    gather(&mut st.width, &width, 0.0);
    gather(&mut st.cover, &cover, 0.0);
    gather(&mut st.sliding, &sliding, 0.0);
    st.cells = st.thick.par_iter().filter(|&&x| x > 0.0).count();
    // (in stack order: rayon's collect keeps it)
    st.glaciated.par_extend(
        (0..l)
            .into_par_iter()
            .filter(|&i| has_ice(thick.get(i)))
            .map(|i| r.stack[i]),
    );
    crate::alloc::fill(&mut st.footprint, n, f64::NAN);
    footprint(d, kind, h, p, st);
}

/// A thickness the serial passes count as ice: not zero or less (NaN too, as their tests
/// `h <= 0.0` let it through).
#[inline]
fn has_ice(h: f64) -> bool {
    h > 0.0 || h.is_nan()
}

/// The footprint: every glacier over the cells within half its width that lie under it.
///
/// It walks [`IceState::glaciated`], the stack's cells with ice in stack order: the cells it skips
/// have no surface, so it writes what a walk of the whole stack would, in the same order.
fn footprint(d: &Dims, kind: &[u8], h: &[f64], p: &GlacialParams, st: &mut IceState) {
    for &c in &st.glaciated {
        let c = c as usize;
        let zs = st.surface[c];
        if !zs.is_finite() {
            continue;
        }
        let half = 0.5 * st.width[c] * p.valley_width / d.dx;
        let reach = (half - 0.5).ceil().clamp(0.0, MAX_SPREAD as f64) as i64;
        for dj in -reach..=reach {
            for di in -reach..=reach {
                let dist = ((di * di + dj * dj) as f64).sqrt();
                if dist > half + 0.5 && (di, dj) != (0, 0) {
                    continue;
                }
                let Some(j) = offset(d, c, di, dj) else {
                    continue;
                };
                if kind[j] != BASE
                    && h[j] < zs
                    && (st.footprint[j].is_nan() || st.footprint[j] < zs)
                {
                    st.footprint[j] = zs;
                }
            }
        }
    }
}

/// Positions handed out per task by the parallel passes of [`ice`] (`spl.rs`'s values).
const DOWN_GRAIN: u32 = 256;
const UP_CHUNK: usize = 64;

/// [`ice`] by serial walks of the stack: routings without their forest, and the tests' reference.
#[allow(clippy::too_many_arguments)]
pub fn ice_serial(
    d: &Dims,
    r: &Routing,
    kind: &[u8],
    h: &[f64],
    filled: &[f64],
    sea: f64,
    clim: &IceClimate<'_>,
    p: &GlacialParams,
    gravity: f64,
    st: &mut IceState,
) {
    let n = d.cells();
    let a = d.cell_area();
    let flow = Flow::new(p, gravity, d.dx);
    for v in [
        &mut st.q,
        &mut st.melt_frac,
        &mut st.thick,
        &mut st.cover,
        &mut st.width,
        &mut st.sliding,
        &mut st.slope,
    ] {
        v.clear();
        v.resize(n, 0.0);
    }
    st.surface.clear();
    st.surface.resize(n, f64::NAN);
    st.footprint.clear();
    st.footprint.resize(n, f64::NAN);
    st.cells = 0;
    st.glaciated.clear();
    // Up: fluxes (donors before receivers: the reverse of the stack).
    let mut qin = vec![0.0f64; n];
    let mut any = false;
    for &c in r.stack.iter().rev() {
        let c = c as usize;
        let allowed = clim.allowed.is_none_or(|m| m[c] > 0.0);
        let b = if allowed {
            balance(p, h[c], clim.ela_z[c], f64::from(clim.precip_m_yr[c]))
        } else {
            -p.max_ablation_m_yr
        };
        let acc = b * a;
        let supply = qin[c] + acc.max(0.0);
        if supply <= 0.0 {
            continue;
        }
        any = true;
        let melt = (-acc).max(0.0).min(supply);
        let out = supply - melt;
        st.melt_frac[c] = melt / supply;
        st.q[c] = 0.5 * (supply + out);
        let rc = r.rcv[c] as usize;
        if rc != c && kind[rc] != BASE {
            qin[rc] += out;
        }
    }
    if !any {
        return;
    }
    // Down: surfaces, receivers first.
    for &c in &r.stack {
        let c = c as usize;
        let q = st.q[c];
        if q < Q_MIN {
            st.q[c] = 0.0;
            continue;
        }
        let rc = r.rcv[c] as usize;
        let (z_ref, len) = if rc == c {
            (filled[c], d.dx)
        } else if kind[rc] == BASE {
            (sea, f64::from(r.len[c]).max(1.0))
        } else if st.thick[rc] > 0.0 {
            (st.surface[rc].max(filled[rc]), f64::from(r.len[c]).max(1.0))
        } else {
            (filled[rc], f64::from(r.len[c]).max(1.0))
        };
        let warm_t = smoothstep(p.warm_bed_c[0], p.warm_bed_c[1], f64::from(clim.t_c[c]));
        let (zs, hh, s) = solve_surface(&flow, q, h[c], z_ref, len, warm_t);
        if hh <= 0.0 {
            continue;
        }
        st.surface[c] = zs;
        st.thick[c] = hh;
        st.slope[c] = s;
        st.width[c] = flow.width(hh);
        st.cover[c] = (st.width[c] / d.dx).min(1.0);
        st.sliding[c] = flow.sliding(hh, s, warm_t);
        st.cells += 1;
        st.glaciated.push(c as u32);
    }
    // The footprint: every glacier over the cells within half its width that lie under it.
    for &c in &r.stack {
        let c = c as usize;
        let zs = st.surface[c];
        if !zs.is_finite() {
            continue;
        }
        let half = 0.5 * st.width[c] * p.valley_width / d.dx;
        let reach = (half - 0.5).ceil().clamp(0.0, MAX_SPREAD as f64) as i64;
        for dj in -reach..=reach {
            for di in -reach..=reach {
                let dist = ((di * di + dj * dj) as f64).sqrt();
                if dist > half + 0.5 && (di, dj) != (0, 0) {
                    continue;
                }
                let Some(j) = offset(d, c, di, dj) else {
                    continue;
                };
                if kind[j] != BASE
                    && h[j] < zs
                    && (st.footprint[j].is_nan() || st.footprint[j] < zs)
                {
                    st.footprint[j] = zs;
                }
            }
        }
    }
}

/// Runs `p.substeps` ice solves and erosion–transport passes over `dt` years on `h` (the bed),
/// adding every change to `out` (reset first). `filled` and `r` are the step's flood and routing.
#[allow(clippy::too_many_arguments)]
pub fn step(
    d: &Dims,
    r: &Routing,
    kind: &[u8],
    filled: &[f64],
    sea: f64,
    clim: &IceClimate<'_>,
    p: &GlacialParams,
    gravity: f64,
    dt: f64,
    h: &mut [f64],
    out: &mut GlacialOut,
) {
    let n = d.cells();
    let a = d.cell_area();
    out.reset(n);
    let subs = p.substeps.max(1) as usize;
    let dts = dt / subs as f64;
    let k = p.erosion_k * p.strength;
    let mut ice_st = std::mem::take(&mut out.ice);
    let mut wk = std::mem::take(&mut out.work);
    let l = r.stack.len();
    // With the forest, the debris pass visits only the positions downstream of an eroding cell
    // (every other cell has no load: it would skip them); without, every position.
    let f = &r.forest;
    let sparse = f.len() == l && f.pos.len() == n;
    for v in [
        &mut wk.lin,
        &mut wk.min,
        &mut wk.e,
        &mut wk.till_add,
        &mut wk.lake_add,
        &mut wk.src,
    ] {
        if v.len() != n {
            crate::alloc::fill(v, n, 0.0);
        }
    }
    if wk.marked.len() != l.div_ceil(64) {
        wk.marked.clear();
        wk.marked.resize(l.div_ceil(64), 0);
    }
    let Work {
        lin,
        min,
        e,
        till_add,
        lake_add,
        src,
        marked,
        eroding,
        written,
    } = &mut wk;
    let mut spread: Vec<(usize, f64)> = Vec::new();
    let mut sea_cells: Vec<u32> = Vec::new();
    for _ in 0..subs {
        ice(d, r, kind, h, filled, sea, clim, p, gravity, &mut ice_st);
        if ice_st.cells == 0 {
            break;
        }
        // Erosion of every glaciated cell (the stack's cells with ice, in stack order).
        eroding.clear();
        written.clear();
        if k > 0.0 {
            for &c in &ice_st.glaciated {
                let c = c as usize;
                let hh = ice_st.thick[c];
                if hh <= 0.0 {
                    continue;
                }
                // Afloat: no contact with the bed.
                if h[c] < sea && hh < RHO_WATER / RHO_ICE * (sea - h[c]) {
                    continue;
                }
                let rc = r.rcv[c] as usize;
                let s = ice_st.slope[c];
                if rc != c {
                    let len = f64::from(r.len[c]).max(1.0);
                    let adverse = (h[rc] - h[c]) / len;
                    if adverse > p.supercooling_ratio * s {
                        continue;
                    }
                }
                let u = ice_st.sliding[c];
                if u <= 0.0 {
                    continue;
                }
                // The glacier's cross-section, W wide, erodes at `rate` under all of it: a
                // glacier wider than a cell spreads it over the cells within W/2 of its centre
                // line that lie under its surface, most at the centre (a parabolic trough).
                let share = clim
                    .time_share
                    .map_or(1.0, |s| f64::from(s[c]).clamp(0.0, 1.0));
                let rate = k * u.powf(p.erosion_exponent) * share;
                let w = ice_st.width[c];
                let vol = rate * dts * w * d.dx;
                let zs = ice_st.surface[c];
                // (drawn `valley_width` times as wide: the same volume, a wider trough)
                let half = 0.5 * w * p.valley_width / d.dx;
                let reach = (half - 0.5).ceil().clamp(0.0, MAX_SPREAD as f64) as i64;
                spread.clear();
                let mut sum = 0.0;
                for dj in -reach..=reach {
                    for di in -reach..=reach {
                        let dist = ((di * di + dj * dj) as f64).sqrt();
                        if dist > half + 0.5 && (di, dj) != (0, 0) {
                            continue;
                        }
                        let Some(j) = offset(d, c, di, dj) else {
                            continue;
                        };
                        if kind[j] == BASE || zs - h[j] <= 0.0 {
                            continue;
                        }
                        let wgt = (1.0 - (dist / (half + 0.5)).powi(2)).max(0.05);
                        spread.push((j, wgt));
                        sum += wgt;
                    }
                }
                for &(j, wgt) in &spread {
                    let depth = (vol * wgt / sum / a).min(p.max_erosion_share * (zs - h[j]));
                    e[j] += depth;
                    src[c] += depth * a;
                    written.push(j as u32);
                }
                if !spread.is_empty() {
                    eroding.push(c as u32);
                }
            }
        }
        // The positions the debris can reach: down from every eroding cell (a walk stops where
        // an earlier one passed).
        if sparse {
            for &c in eroding.iter() {
                let mut i = f.pos[c as usize];
                while i != NONE {
                    let (w, b) = (i as usize / 64, 1u64 << (i % 64));
                    if marked[w] & b != 0 {
                        break;
                    }
                    marked[w] |= b;
                    i = f.rpos[i as usize];
                }
            }
        } else {
            marked.iter_mut().for_each(|w| *w = u64::MAX);
            if !l.is_multiple_of(64)
                && let Some(last) = marked.last_mut()
            {
                *last = (1u64 << (l % 64)) - 1;
            }
        }
        // Debris: up the stack (donors first, so positions downwards), melting out with the ice.
        for w in (0..marked.len()).rev() {
            let mut bits = std::mem::take(&mut marked[w]);
            while bits != 0 {
                let t = 63 - bits.leading_zeros() as usize;
                bits &= !(1u64 << t);
                let c = r.stack[w * 64 + t] as usize;
                written.push(c as u32);
                let load = lin[c] + src[c];
                let mut melt_load = min[c];
                if load <= 0.0 && melt_load <= 0.0 {
                    continue;
                }
                // All of it melts out at a terminus or beyond the ice.
                let frac = if ice_st.q[c] > 0.0 {
                    ice_st.melt_frac[c]
                } else {
                    1.0
                };
                let debris = load * frac;
                let till_vol = (p.till_share * debris).min(p.max_till_m * a);
                let carried = load - debris;
                melt_load += debris - till_vol;
                till_add[c] += till_vol;
                out.till_m3 += till_vol;
                // Meltwater crossing an ice-free lake leaves its load there, up to the lake level.
                if ice_st.thick[c] <= 0.0 && melt_load > 0.0 {
                    let room = filled[c] - h[c] + e[c] - till_add[c] / a;
                    if room > LAKE_TOL_M {
                        let dep = melt_load.min(room * a);
                        lake_add[c] += dep;
                        out.lake_m3 += dep;
                        melt_load -= dep;
                    }
                }
                let rc = r.rcv[c] as usize;
                if rc == c {
                    // A closed basin keeps everything (as till).
                    till_add[c] += carried + melt_load;
                    out.sink_m3 += carried + melt_load;
                } else if kind[rc] == BASE {
                    out.sea_ice_load[rc] += carried;
                    out.sea_melt_load[rc] += melt_load;
                    out.sea_m3 += carried + melt_load;
                    sea_cells.push(rc as u32);
                } else {
                    lin[rc] += carried;
                    min[rc] += melt_load;
                    written.push(rc as u32);
                }
            }
        }
        // Every cell written above, ascending (the cells between have nothing to add), and its
        // working values cleared for the next substep.
        written.sort_unstable();
        written.dedup();
        for &c in written.iter() {
            let c = c as usize;
            if e[c] > 0.0 {
                h[c] -= e[c];
                out.eroded[c] += e[c];
                out.eroded_m3 += e[c] * a;
            }
            if till_add[c] > 0.0 {
                let t = till_add[c] / a;
                h[c] += t;
                out.till[c] += t;
            }
            if lake_add[c] > 0.0 {
                let t = lake_add[c] / a;
                h[c] += t;
                out.lake[c] += t;
            }
            for v in [
                &mut *lin,
                &mut *min,
                &mut *e,
                &mut *till_add,
                &mut *lake_add,
                &mut *src,
            ] {
                v[c] = 0.0;
            }
        }
    }
    sea_cells.sort_unstable();
    sea_cells.dedup();
    out.sea_cells = sea_cells;
    out.ice = ice_st;
    out.work = wk;
}

/// `notes` key of the deep-time totals.
pub const NOTES_KEY: &str = "glacial";

/// What the glaciers did over a run (kept in `world.notes["glacial"]`, so a checkpoint carries it).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Totals {
    pub eroded_km3: f64,
    pub till_km3: f64,
    pub lake_km3: f64,
    pub sink_km3: f64,
    pub sea_km3: f64,
    /// Steps with any glacier, and the most glaciated cells of one step and of the last.
    pub steps_with_ice: u64,
    pub ice_cells_max: u64,
    pub ice_cells_last: u64,
    /// Cells the ice cut from above to below sea level (fjords), summed over the steps.
    pub cut_below_sea_cells: u64,
    /// The fastest glacial erosion of a cell in one step, m/Myr.
    pub max_rate_m_myr: f64,
}

impl Totals {
    /// The totals in `world.notes` (default when absent or unreadable).
    pub fn read(notes: &std::collections::BTreeMap<String, String>) -> Totals {
        notes
            .get(NOTES_KEY)
            .and_then(|t| serde_json::from_str(t).ok())
            .unwrap_or_default()
    }

    pub fn write(&self, notes: &mut std::collections::BTreeMap<String, String>) {
        notes.insert(
            NOTES_KEY.into(),
            serde_json::to_string(self).unwrap_or_else(|_| "{}".into()),
        );
    }
}

/// `glacial_final …` (the CLI's line), when the run had a glacial note.
pub fn summary_line(notes: &std::collections::BTreeMap<String, String>) -> Option<String> {
    notes.get(NOTES_KEY)?;
    let t = Totals::read(notes);
    Some(format!(
        "glacial_final eroded_km3={:.3e} till_km3={:.3e} lake_km3={:.3e} sink_km3={:.3e} sea_km3={:.3e} \
         steps_with_ice={} ice_cells_max={} ice_cells_last={} cut_below_sea_cells={} max_rate_m_myr={:.0}",
        t.eroded_km3,
        t.till_km3,
        t.lake_km3,
        t.sink_km3,
        t.sea_km3,
        t.steps_with_ice,
        t.ice_cells_max,
        t.ice_cells_last,
        t.cut_below_sea_cells,
        t.max_rate_m_myr,
    ))
}

#[cfg(test)]
mod tests;
