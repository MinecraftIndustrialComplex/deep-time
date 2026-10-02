// SPDX-License-Identifier: LGPL-3.0-or-later
//! Lithospheric flexure (Ben, decisions log 2026-10-02: "General flexure everywhere"; the design
//! is `docs/design/flexure.md`).
//!
//! # The model
//! The lithosphere is a thin elastic plate on a fluid mantle. Its deflection w (m, down positive)
//! under a load q (kg/m²) obeys
//!
//!   ∇²(D ∇²w) + ρ_m g w = g q,   D = E·Te³ / (12 (1 − ν²)),
//!
//! with Te the effective elastic thickness. With D = 0 it is Airy isostasy, w = q / ρ_m, which is
//! what the model did before; with D > 0 a load narrower than the flexural parameter
//! α = (4D / ρ_m g)^¼ is held up by the plate and pulls its surroundings down: a foreland basin in
//! front of a belt, a moat round a volcanic island, a low bulge beyond. Gravity enters through α
//! only (load and buoyancy both carry g).
//!
//! # Loads, and what is already compensated
//! The surface stage keeps, with the crust, the load `Q` since the start of the run (m of mantle
//! displaced under Airy: kg/m² ÷ ρ_m) and the deflection `w` that goes with it. Each step it adds
//! - the crust the plate model thickened (its uplift is an Airy elevation: the thickening is
//!   u / (1 − k), k = ρ_c/ρ_m on land and (ρ_c − ρ_w)/(ρ_m − ρ_w) under water), and the crust the
//!   plateau spreading moved (`spread.rs`): for these the Airy root `a` is already in the
//!   elevation;
//! - the step's erosion and deposition, the rock other stages laid on the surface since the last
//!   step (the volcano stage's cones), the sea over the cell against the sea at the reference,
//!   and land ice: for these nothing has been applied;
//!
//! solves for the deflection of the whole load, and changes the elevation by −(δw − a). Every
//! kilogram is in `Q` once and every metre of sinking is applied once, by the plate model (`a`)
//! or by the solve. With D = 0 the solve returns the Airy sinking of every load: erosion rebounds
//! as before (without the old blur) and plate uplift is unchanged.
//!
//! # A plate that stiffens keeps its shape
//! Rigidity added by cooling joins stress-free at the shape the plate has: the bending moment is
//! D ∇²w − M₀, with a frozen moment M₀ kept with the crust. When D grows the step is solved with
//! the old D and then M₀ += ΔD·∇²w; when it falls M₀ shrinks in proportion. So an old belt's
//! local root does not turn into relief as its crust cools, and a load records the strength of the
//! plate at the time of loading (Watts 1978).
//!
//! # Lengths
//! Earth-normalized, as the plate model's (decisions file, "Tectonics scaling"): the solve's cell
//! size is the L1 cell's share of the model sphere's circumference, so a basin is as wide against
//! its belt as on Earth. Heights are not scaled.
//!
//! # The solve
//! (L D L + C) δw = r on the raster in block metric (x wraps, rows closed), L the 5-point
//! Laplacian, C the restoring density (ρ_m, less ρ_w under the sea: a basin that sinks takes
//! water). Conjugate gradients, preconditioned by exact spectral solves at three uniform
//! rigidities (a Fourier transform in x, a cosine transform in y) blended by smooth weights in
//! ln α: symmetric, so it converges for any Te field, and the exact inverse where Te is uniform.
//! Presets with L1 cells finer than about 40 Earth-normalized km solve on L1 coarsened by a power
//! of two and interpolate the deflection.
//!
//! The solver is written from the equation; no flexure code was read or ported.

use std::sync::Arc;

use rayon::prelude::*;
use rustfft::num_complex::Complex64;
use rustfft::{Fft, FftPlanner};
use serde::{Deserialize, Serialize};

use super::advect::{self, Carry};
use super::grid::{Dims, box_blur};
use crate::error::CoreError;
use crate::plates::pools::ISOSTATIC_FACTOR;
use crate::plates::state::{OCEANIC, regime};

/// Mantle, sea water and ice, kg/m³.
pub const RHO_MANTLE: f64 = 3300.0; // ledger: ISO-043
pub const RHO_WATER: f64 = 1030.0; // ledger: ISO-044
pub const RHO_ICE: f64 = 917.0; // ledger: ISO-045
/// Young's modulus, Pa, and Poisson's ratio (Turcotte & Schubert, *Geodynamics*).
pub const YOUNG_PA: f64 = 70.0e9; // ledger: ISO-041
pub const POISSON: f64 = 0.25; // ledger: ISO-042
/// The coarsest solve cell, Earth-normalized metres: finer L1 grids are coarsened towards it.
pub const SOLVE_CELL_M: f64 = 45_000.0; // ledger: ISO-058 (game-choice: numerical)

/// Fields the surface stage writes each step when flexure is on.
pub mod field {
    /// How far the ground stands above (+) or below (−) where local isostasy would put it, m:
    /// negative in foreland basins and moats, positive on bulges and on loads the plate holds up.
    pub const FLEXURE: &str = "surface/flexure_m";
    /// The whole sinking under the load since the start of the run, m (down positive).
    pub const DEFLECTION: &str = "surface/deflection_m";
    /// The effective elastic thickness the step was solved with, km.
    pub const ELASTIC_THICKNESS: &str = "surface/elastic_thickness_km";
}

/// `WorldState::globals`: solves, their conjugate-gradient iterations, and the solves that
/// stopped at `max_iterations`.
pub const SOLVES_GLOBAL: &str = "surface/flexure_solves";
pub const ITERATIONS_GLOBAL: &str = "surface/flexure_iterations";
pub const UNCONVERGED_GLOBAL: &str = "surface/flexure_unconverged";

/// The `surface.flexure` section: every key optional.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct FlexureParams {
    /// 0: local isostasy as before (`isostasy`, `isostasy_radius_km`); 1: the flexure solve;
    /// between, a blend of the two corrections. The struct's default is 0; the pipeline presets
    /// set 1.
    pub strength: f64,
    /// Multiplier on the elastic thickness everywhere (the editor's "Plate stiffness").
    pub elastic_thickness_scale: f64,
    /// Oceanic Te = this × √(age in Myr), km: 2.7 ± 0.15 measured under 38 seamounts (Calmant,
    /// Francheteau & Cazenave 1990, full text), near the 450 ± 150 °C isotherm of the cooling plate
    /// (Watts 1978, abstract).
    pub te_ocean_km_sqrt_myr: f64,
    /// … and at most this, km (Hawaii, on 80 Myr floor: 40 km, Watts & ten Brink 1989, abstract).
    pub te_ocean_max_km: f64,
    /// Continental Te long after the last orogeny, km (cratons: 70–90 km, Burov & Diament 1995).
    pub te_craton_km: f64,
    /// Continental Te recovers as 1 − e^(−t / this), t the time since the last orogeny, Myr.
    pub te_recovery_myr: f64,
    /// Te of arcs, rifts and everything else at least, km (the low end of continental Te: 5 km,
    /// Lu et al. 2024, abstract).
    pub te_min_km: f64,
    /// Solve every this many plate steps (loads in between wait for the next solve).
    pub every_steps: u32,
    /// The solve stops when its error estimate is under this, m.
    pub tolerance_m: f64,
    /// … or after this many iterations.
    pub max_iterations: u32,
}

impl Default for FlexureParams {
    fn default() -> Self {
        FlexureParams {
            strength: 0.0, // ledger: ISO-065 (game-choice: off unless a preset turns it on)
            elastic_thickness_scale: 1.0, // ledger: ISO-066
            te_ocean_km_sqrt_myr: 2.7, // ledger: ISO-048
            te_ocean_max_km: 40.0, // ledger: ISO-049
            te_craton_km: 80.0, // ledger: ISO-050
            te_recovery_myr: 300.0, // ledger: ISO-051 (uncited: this agent's choice)
            te_min_km: 5.0, // ledger: ISO-052
            every_steps: 1, // ledger: ISO-059 (game-choice: numerical)
            tolerance_m: 2.0, // ledger: ISO-059
            max_iterations: 60, // ledger: ISO-059
        }
    }
}

impl FlexureParams {
    /// Whether the solve runs at all.
    pub fn active(&self) -> bool {
        self.strength > 0.0
    }

    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        let check = |ok: bool, key: &str, reason: &str| -> Result<(), CoreError> {
            if ok {
                Ok(())
            } else {
                Err(CoreError::preset(
                    format!("$.surface.flexure.{key}"),
                    reason,
                ))
            }
        };
        check(
            (0.0..=1.0).contains(&self.strength),
            "strength",
            "must be in [0, 1]",
        )?;
        check(
            self.elastic_thickness_scale.is_finite()
                && (0.01..=10.0).contains(&self.elastic_thickness_scale),
            "elastic_thickness_scale",
            "must be in [0.01, 10]",
        )?;
        for (v, key) in [
            (self.te_ocean_km_sqrt_myr, "te_ocean_km_sqrt_myr"),
            (self.te_ocean_max_km, "te_ocean_max_km"),
            (self.te_craton_km, "te_craton_km"),
            (self.te_recovery_myr, "te_recovery_myr"),
            (self.te_min_km, "te_min_km"),
            (self.tolerance_m, "tolerance_m"),
        ] {
            check(v.is_finite() && v > 0.0, key, "must be > 0")?;
        }
        check(
            self.te_ocean_max_km <= 500.0 && self.te_craton_km <= 500.0,
            "te_craton_km",
            "elastic thicknesses must be at most 500 km",
        )?;
        check(
            (1..=1000).contains(&self.every_steps),
            "every_steps",
            "must be in [1, 1000]",
        )?;
        check(
            (1..=10_000).contains(&self.max_iterations),
            "max_iterations",
            "must be in [1, 10000]",
        )?;
        Ok(())
    }

    /// The effective elastic thickness of a cell, km, before smoothing: oceanic crust by its age
    /// (Myr), continental crust by the time since its last orogeny (Myr; NaN or infinite: never),
    /// arcs and rifts at the floor.
    pub fn elastic_thickness_km(
        &self,
        oceanic: bool,
        age_myr: f64,
        since_orogeny_myr: f64,
        weak: bool,
    ) -> f64 {
        let te = if weak {
            0.0
        } else if oceanic {
            (self.te_ocean_km_sqrt_myr * age_myr.max(0.0).sqrt()).min(self.te_ocean_max_km)
        } else if since_orogeny_myr.is_finite() {
            self.te_craton_km * (1.0 - (-since_orogeny_myr.max(0.0) / self.te_recovery_myr).exp())
        } else {
            self.te_craton_km
        };
        te.max(self.te_min_km) * self.elastic_thickness_scale
    }
}

/// Flexural rigidity over gravity, D / g, kg·m, of a plate `te_km` thick.
pub fn rigidity_per_g(te_km: f64, gravity_m_s2: f64) -> f64 {
    let te = te_km * 1000.0;
    YOUNG_PA * te * te * te / (12.0 * (1.0 - POISSON * POISSON)) / gravity_m_s2
}

/// The flexural parameter α = (4D / ρ_m g)^¼, m, of a rigidity over gravity.
pub fn flexural_parameter_m(d_per_g: f64) -> f64 {
    (4.0 * d_per_g / RHO_MANTLE).powf(0.25)
}

/// Airy share of crust under air and under water: a thickening T sinks k·T.
// ledger: ISO-046
fn airy_shares() -> (f64, f64) {
    let rho_c = RHO_MANTLE * (1.0 - 1.0 / ISOSTATIC_FACTOR);
    (
        rho_c / RHO_MANTLE,
        (rho_c - RHO_WATER) / (RHO_MANTLE - RHO_WATER),
    )
}

/// The crust thickening T behind an Airy elevation change `u` (m, either sign) of ground that
/// stood at `z0` before it, the part of the change below the sea compensated under water; and
/// the root already in the elevation, a = T − u. Returns (load in m of mantle, a).
fn thickening(u: f64, z0: f64, sea: f64) -> (f64, f64) {
    if u == 0.0 {
        return (0.0, 0.0);
    }
    let (ka, kw) = airy_shares();
    let (lo, hi) = if u > 0.0 { (z0, z0 + u) } else { (z0 + u, z0) };
    let below = (sea.min(hi) - lo).max(0.0);
    let above = (hi - lo) - below;
    let t = (below / (1.0 - kw) + above / (1.0 - ka)) * u.signum();
    (t * ka, t - u)
}

/// The surface's flexure state, kept with the crust (`SurfaceState::flexure`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FlexState {
    /// The load since the start of the run, rock and sea water, m of mantle (kg/m² ÷ ρ_m).
    pub load_m: Vec<f32>,
    /// The sinking that goes with it, m (down positive).
    pub deflection_m: Vec<f32>,
    /// The depth of sea counted in `load_m`, m; NaN where the crust is new at the place (its
    /// reference is taken at the next step).
    pub water_m: Vec<f32>,
    /// The frozen bending moment over gravity, M₀ / g, kg.
    pub moment: Vec<f32>,
    /// The rigidity over gravity the cell was last solved with, D / g, kg·m.
    pub rigidity: Vec<f32>,
    /// The surface's elevation when this state was last brought up to date, m: what later stages
    /// add to it is a load.
    pub seen_elevation_m: Vec<f32>,
    /// Plate steps since the last solve.
    pub since_solve: u32,
}

impl FlexState {
    fn fresh(n: usize) -> FlexState {
        FlexState {
            load_m: vec![0.0; n],
            deflection_m: vec![0.0; n],
            water_m: vec![f32::NAN; n],
            moment: vec![0.0; n],
            rigidity: vec![f32::NAN; n],
            seen_elevation_m: Vec::new(),
            since_solve: 0,
        }
    }

    /// Carries the state with the crust over a plate step (`advect::carried_elevation`'s rule:
    /// only source cells that held the same crust count). `elevation_m` is the surface's elevation
    /// before the step, on the old positions: what other stages added to it since this state last
    /// saw it (the volcano stage's cones) joins the load as rock with no root.
    pub fn carry(&mut self, x: &Carry<'_>, elevation_m: &[f32]) {
        let n = elevation_m.len();
        if self.load_m.len() != n {
            *self = FlexState::fresh(n);
            return;
        }
        if self.seen_elevation_m.len() == n {
            let (ka, _) = airy_shares();
            for ((q, &z), &seen) in self
                .load_m
                .iter_mut()
                .zip(elevation_m)
                .zip(&self.seen_elevation_m)
            {
                let added = z - seen;
                if added.is_finite() && added.abs() > 1.0e-3 {
                    *q += added * ka as f32;
                }
            }
        }
        // Crust that is new at a place starts unloaded and flat (its water is NaN until the
        // next step takes its reference).
        let moved = advect::carried_fields(
            x,
            [
                &self.water_m,
                &self.load_m,
                &self.deflection_m,
                &self.moment,
                &self.rigidity,
            ],
        );
        for (c, m) in moved.into_iter().enumerate() {
            let [water, load, sunk, moment, rigidity] = m
                .filter(|m| m[..4].iter().all(|y| y.is_finite()))
                .unwrap_or([f32::NAN, 0.0, 0.0, 0.0, f32::NAN]);
            self.water_m[c] = water;
            self.load_m[c] = load;
            self.deflection_m[c] = sunk;
            self.moment[c] = moment;
            self.rigidity[c] = rigidity;
        }
    }
}

/// What a solve reports.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SolveReport {
    pub iterations: u32,
    /// The last error estimate (the preconditioned residual's largest entry), m.
    pub error_m: f64,
    pub converged: bool,
}

/// The transforms that diagonalise the 5-point Laplacian on a raster that wraps in x and is
/// closed at its first and last row: a Fourier transform along x (two real rows at a time, as
/// the real and imaginary part of one complex row), a cosine transform (DCT-II) along y by one
/// complex transform of the column's length (Makhoul's reordering: even entries first, odd ones
/// reversed after them).
struct Spectral {
    w: usize,
    h: usize,
    nk: usize,
    fx: Arc<dyn Fft<f64>>,
    ix: Arc<dyn Fft<f64>>,
    fy: Arc<dyn Fft<f64>>,
    iy: Arc<dyn Fft<f64>>,
    /// e^(−iπm / 2h).
    twiddle: Vec<Complex64>,
    /// The Laplacian's eigenvalue squared, column-major (k·h + m), 1/m⁴.
    lam2: Vec<f64>,
}

impl Spectral {
    fn new(w: usize, h: usize, dx: f64) -> Spectral {
        let mut planner = FftPlanner::<f64>::new();
        let nk = w / 2 + 1;
        let twiddle: Vec<Complex64> = (0..h)
            .map(|m| Complex64::from_polar(1.0, -std::f64::consts::PI * m as f64 / (2 * h) as f64))
            .collect();
        let lx: Vec<f64> = (0..nk)
            .map(|k| 2.0 * (std::f64::consts::TAU * k as f64 / w as f64).cos() - 2.0)
            .collect();
        let ly: Vec<f64> = (0..h)
            .map(|m| 2.0 * (std::f64::consts::PI * m as f64 / h as f64).cos() - 2.0)
            .collect();
        let inv = 1.0 / (dx * dx);
        let mut lam2 = vec![0.0; nk * h];
        for k in 0..nk {
            for m in 0..h {
                let l = (lx[k] + ly[m]) * inv;
                lam2[k * h + m] = l * l;
            }
        }
        Spectral {
            w,
            h,
            nk,
            fx: planner.plan_fft_forward(w),
            ix: planner.plan_fft_inverse(w),
            fy: planner.plan_fft_forward(h),
            iy: planner.plan_fft_inverse(h),
            twiddle,
            lam2,
        }
    }

    /// `spec` (nk × h, column-major) ← the transform of `v`: Σ_i Σ_j v e^(−2πi·ik/w)
    /// cos(π(j + ½)m/h). `half` is scratch (h × nk).
    fn forward(&self, v: &[f64], half: &mut [Complex64], spec: &mut [Complex64]) {
        let (w, h, nk) = (self.w, self.h, self.nk);
        let zero = Complex64::new(0.0, 0.0);
        // Rows a and b as a + i·b: A[k] = (Z[k] + conj Z[−k]) / 2, B[k] = (Z[k] − conj Z[−k]) / 2i.
        half.par_chunks_mut(2 * nk)
            .zip(v.par_chunks(2 * w))
            .for_each_init(
                || (vec![zero; w], vec![zero; self.fx.get_inplace_scratch_len()]),
                |(buf, scratch), (out, rows)| {
                    let (a, b) = rows.split_at(w);
                    for (i, z) in buf.iter_mut().enumerate() {
                        *z = Complex64::new(a[i], b.get(i).copied().unwrap_or(0.0));
                    }
                    self.fx.process_with_scratch(buf, scratch);
                    let (oa, ob) = out.split_at_mut(nk);
                    for k in 0..nk {
                        let (z, zc) = (buf[k], buf[(w - k) % w].conj());
                        oa[k] = (z + zc) * 0.5;
                        if let Some(o) = ob.get_mut(k) {
                            let d = z - zc;
                            *o = Complex64::new(d.im, -d.re) * 0.5;
                        }
                    }
                },
            );
        let half = &*half;
        spec.par_chunks_mut(h).enumerate().for_each_init(
            || (vec![zero; h], vec![zero; self.fy.get_inplace_scratch_len()]),
            |(buf, scratch), (k, col)| {
                for j in 0..h {
                    let x = half[j * nk + k];
                    if j % 2 == 0 {
                        buf[j / 2] = x;
                    } else {
                        buf[h - 1 - j / 2] = x;
                    }
                }
                self.fy.process_with_scratch(buf, scratch);
                // ω_m V[m] = C[m] − i C[h − m].
                col[0] = buf[0];
                for m in 1..h {
                    let p = self.twiddle[m] * buf[m];
                    let q = self.twiddle[h - m] * buf[h - m];
                    col[m] = (p + Complex64::new(-q.im, q.re)) * 0.5;
                }
            },
        );
    }

    /// `out` ← the inverse transform of `spec` (overwritten).
    fn inverse(&self, spec: &mut [Complex64], out: &mut [f64]) {
        let (w, h, nk) = (self.w, self.h, self.nk);
        let zero = Complex64::new(0.0, 0.0);
        let norm_y = 1.0 / h as f64;
        spec.par_chunks_mut(h).for_each_init(
            || (vec![zero; h], vec![zero; self.iy.get_inplace_scratch_len()]),
            |(buf, scratch), col| {
                buf[0] = col[0];
                for m in 1..h {
                    let c = col[h - m];
                    buf[m] = self.twiddle[m].conj() * (col[m] + Complex64::new(c.im, -c.re));
                }
                self.iy.process_with_scratch(buf, scratch);
                for j in 0..h {
                    col[j] = if j % 2 == 0 {
                        buf[j / 2]
                    } else {
                        buf[h - 1 - j / 2]
                    } * norm_y;
                }
            },
        );
        let spec = &*spec;
        let norm_x = 1.0 / w as f64;
        out.par_chunks_mut(2 * w).enumerate().for_each_init(
            || (vec![zero; w], vec![zero; self.ix.get_inplace_scratch_len()]),
            |(buf, scratch), (pair, rows)| {
                let (ja, jb) = (2 * pair, 2 * pair + 1);
                let i1 = Complex64::new(0.0, 1.0);
                for k in 0..nk {
                    let a = spec[k * h + ja];
                    let b = if jb < h { spec[k * h + jb] } else { zero };
                    buf[k] = a + i1 * b;
                    if k > 0 && k < w - k {
                        buf[w - k] = a.conj() + i1 * b.conj();
                    }
                }
                self.ix.process_with_scratch(buf, scratch);
                let (ra, rb) = rows.split_at_mut(w);
                for (i, z) in buf.iter().enumerate() {
                    ra[i] = z.re * norm_x;
                    if let Some(x) = rb.get_mut(i) {
                        *x = z.im * norm_x;
                    }
                }
            },
        );
    }
}

/// The preconditioner: exact solves at up to three uniform rigidities, blended by weights whose
/// squares sum to one.
struct Bank {
    /// D / g of each level, kg·m.
    d: Vec<f64>,
    /// The weight of each level per cell; empty for a single level (weight 1 everywhere).
    weights: Vec<Vec<f64>>,
    c_ref: f64,
}

impl Bank {
    fn new(d: &[f64], c: &[f64], dx: f64) -> Bank {
        let n = d.len();
        let ln_alpha: Vec<f64> = d
            .par_iter()
            .map(|&d| (flexural_parameter_m(d.max(1e-30)) / dx).ln())
            .collect();
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for &a in &ln_alpha {
            lo = lo.min(a);
            hi = hi.max(a);
        }
        let c_ref = c.iter().sum::<f64>() / n.max(1) as f64;
        let node_d = |ln_a: f64| RHO_MANTLE * (ln_a.exp() * dx).powi(4) / 4.0;
        // A plate softer than this is local at the grid's scale whatever its rigidity.
        let floor = 0.4f64.ln(); // ledger: ISO-060 (game-choice: numerical)
        if hi - lo < 0.05 || hi <= floor {
            return Bank {
                d: vec![node_d(0.5 * (lo + hi))],
                weights: Vec::new(),
                c_ref,
            };
        }
        let lo = lo.max(floor);
        const K: usize = 3;
        let nodes: Vec<f64> = (0..K)
            .map(|k| lo + (hi - lo) * k as f64 / (K - 1) as f64)
            .collect();
        let mut weights = vec![vec![0.0; n]; K];
        for (c, &a) in ln_alpha.iter().enumerate() {
            let t = ((a - lo) / (hi - lo)).clamp(0.0, 1.0) * (K - 1) as f64;
            let k0 = (t.floor() as usize).min(K - 2);
            let f = t - k0 as f64;
            weights[k0][c] = (1.0 - f).sqrt();
            weights[k0 + 1][c] = f.sqrt();
        }
        Bank {
            d: nodes.into_iter().map(node_d).collect(),
            weights,
            c_ref,
        }
    }
}

/// The preconditioner of one solve: the bank, the damped Jacobi weights and their scratch.
struct Preconditioner<'a> {
    bank: Bank,
    d: &'a [f64],
    c: &'a [f64],
    /// ω / the operator's diagonal, ω = ½ (the operator's largest eigenvalue over its diagonal
    /// is under 3.2, so the sweep contracts).
    jacobi: Vec<f64>,
    left: Vec<f64>,
    part: Vec<f64>,
}

impl<'a> Preconditioner<'a> {
    fn new(d: &'a [f64], c: &'a [f64], w: usize, h: usize, dx: f64) -> Preconditioner<'a> {
        let bank = Bank::new(d, c, dx);
        let n = w * h;
        let inv = 1.0 / (dx * dx * dx * dx);
        let jacobi = if bank.weights.is_empty() {
            Vec::new()
        } else {
            (0..n)
                .into_par_iter()
                .map(|k| {
                    let (i, j) = (k % w, k / w);
                    let around = d[j * w + (i + 1) % w]
                        + d[j * w + (i + w - 1) % w]
                        + d[j.saturating_sub(1) * w + i]
                        + d[(j + 1).min(h - 1) * w + i];
                    0.5 / (c[k] + (16.0 * d[k] + around) * inv)
                })
                .collect()
        };
        Preconditioner {
            bank,
            d,
            c,
            jacobi,
            left: vec![0.0; n],
            part: vec![0.0; n],
        }
    }
}

/// The flexure solver of one raster: the transforms and the working arrays.
pub struct Solver {
    pub w: usize,
    pub h: usize,
    /// Cell side, m (Earth-normalized).
    pub dx: f64,
    spectral: Spectral,
    half: Vec<Complex64>,
    spec: Vec<Complex64>,
    tmp: Vec<f64>,
    tmp2: Vec<f64>,
}

/// A sum whose value does not depend on the thread count: per row, then over the rows in order.
fn dot(w: usize, a: &[f64], b: &[f64]) -> f64 {
    let rows: Vec<f64> = a
        .par_chunks(w)
        .zip(b.par_chunks(w))
        .map(|(x, y)| x.iter().zip(y).map(|(p, q)| p * q).sum::<f64>())
        .collect();
    rows.iter().sum()
}

fn max_abs(v: &[f64]) -> f64 {
    v.par_iter().map(|x| x.abs()).reduce(|| 0.0, f64::max)
}

impl Solver {
    pub fn new(w: usize, h: usize, dx: f64) -> Solver {
        let nk = w / 2 + 1;
        let zero = Complex64::new(0.0, 0.0);
        Solver {
            w,
            h,
            dx,
            spectral: Spectral::new(w, h, dx),
            half: vec![zero; h * nk],
            spec: vec![zero; nk * h],
            tmp: vec![0.0; w * h],
            tmp2: vec![0.0; w * h],
        }
    }

    /// out ← the 5-point Laplacian of `v` (x wraps, the first and last row are closed), 1/m².
    pub fn laplacian(&self, v: &[f64], out: &mut [f64]) {
        let (w, h) = (self.w, self.h);
        let inv = 1.0 / (self.dx * self.dx);
        out.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
            let at = |jj: usize| &v[jj * w..(jj + 1) * w];
            let (mid, north, south) = (at(j), at(j.saturating_sub(1)), at((j + 1).min(h - 1)));
            for i in 0..w {
                let west = mid[if i == 0 { w - 1 } else { i - 1 }];
                let east = mid[if i + 1 == w { 0 } else { i + 1 }];
                row[i] = (west + east + north[i] + south[i] - 4.0 * mid[i]) * inv;
            }
        });
    }

    /// out ← L(D·L v): the plate's elastic restoring force over gravity, kg/m².
    pub fn bend(&mut self, d: &[f64], v: &[f64], out: &mut [f64]) {
        let mut m = std::mem::take(&mut self.tmp);
        self.laplacian(v, &mut m);
        m.par_iter_mut()
            .zip(d.par_iter())
            .for_each(|(x, &d)| *x *= d);
        self.laplacian(&m, out);
        self.tmp = m;
    }

    /// out ← (L D L + C) v.
    fn operator(&mut self, d: &[f64], c: &[f64], v: &[f64], out: &mut [f64]) {
        self.bend(d, v, out);
        out.par_iter_mut()
            .zip(c.par_iter().zip(v.par_iter()))
            .for_each(|(o, (&c, &v))| *o += c * v);
    }

    /// z ← the preconditioner applied to `r`: the blended spectral solves between two damped
    /// Jacobi sweeps (a symmetric product, so conjugate gradients still applies). The sweeps take
    /// the cell-scale error that the blend leaves along sharp changes of stiffness (plate
    /// boundaries): measured on a run's own stiffness field, 13 iterations instead of 27.
    fn precondition(&mut self, pre: &mut Preconditioner<'_>, r: &[f64], z: &mut [f64]) {
        if pre.bank.weights.is_empty() {
            self.spectral_solve(&pre.bank, r, z);
            return;
        }
        let mut left = std::mem::take(&mut pre.left);
        let mut part = std::mem::take(&mut pre.part);
        z.par_iter_mut()
            .zip(r.par_iter().zip(pre.jacobi.par_iter()))
            .for_each(|(z, (&r, &j))| *z = r * j);
        self.operator(pre.d, pre.c, z, &mut left);
        left.par_iter_mut()
            .zip(r.par_iter())
            .for_each(|(l, &r)| *l = r - *l);
        self.spectral_solve(&pre.bank, &left, &mut part);
        z.par_iter_mut()
            .zip(part.par_iter())
            .for_each(|(z, &p)| *z += p);
        self.operator(pre.d, pre.c, z, &mut left);
        z.par_iter_mut()
            .zip(left.par_iter().zip(r.par_iter().zip(pre.jacobi.par_iter())))
            .for_each(|(z, (&l, (&r, &j)))| *z += (r - l) * j);
        pre.left = left;
        pre.part = part;
    }

    /// z ← the blend of the bank's exact solves applied to `r`.
    fn spectral_solve(&mut self, bank: &Bank, r: &[f64], z: &mut [f64]) {
        let mut half = std::mem::take(&mut self.half);
        let mut spec = std::mem::take(&mut self.spec);
        let mut part = std::mem::take(&mut self.tmp2);
        let lam2 = &self.spectral.lam2;
        let divide = |spec: &mut [Complex64], d: f64| {
            spec.par_iter_mut()
                .zip(lam2.par_iter())
                .for_each(|(s, &l)| *s /= d * l + bank.c_ref);
        };
        if bank.weights.is_empty() {
            self.spectral.forward(r, &mut half, &mut spec);
            divide(&mut spec, bank.d[0]);
            self.spectral.inverse(&mut spec, z);
        } else {
            z.par_iter_mut().for_each(|x| *x = 0.0);
            for (k, wk) in bank.weights.iter().enumerate() {
                part.par_iter_mut()
                    .zip(r.par_iter().zip(wk.par_iter()))
                    .for_each(|(p, (&r, &s))| *p = r * s);
                self.spectral.forward(&part, &mut half, &mut spec);
                divide(&mut spec, bank.d[k]);
                self.spectral.inverse(&mut spec, &mut part);
                z.par_iter_mut()
                    .zip(part.par_iter().zip(wk.par_iter()))
                    .for_each(|(z, (&p, &s))| *z += p * s);
            }
        }
        self.half = half;
        self.spec = spec;
        self.tmp2 = part;
    }

    /// Solves (L D L + C) x = b from x = 0 by preconditioned conjugate gradients: `d` is D / g
    /// per cell (kg·m), `c` the restoring density (kg/m³), `b` the load (kg/m²); x is in metres.
    /// It stops when the error estimate is under `tolerance_m` or after `max_iterations`.
    pub fn solve(
        &mut self,
        d: &[f64],
        c: &[f64],
        b: &[f64],
        tolerance_m: f64,
        max_iterations: u32,
    ) -> (Vec<f64>, SolveReport) {
        let n = self.w * self.h;
        assert!(d.len() == n && c.len() == n && b.len() == n, "raster size");
        let w = self.w;
        let mut pre = Preconditioner::new(d, c, w, self.h, self.dx);
        let mut x = vec![0.0; n];
        let mut r = b.to_vec();
        let mut z = vec![0.0; n];
        self.precondition(&mut pre, &r, &mut z);
        let mut report = SolveReport {
            iterations: 0,
            error_m: max_abs(&z),
            converged: true,
        };
        if report.error_m < tolerance_m {
            // Nothing worth a search direction: the preconditioner's answer is the correction.
            return (z, report);
        }
        let mut p = z.clone();
        let mut ap = vec![0.0; n];
        let mut rz = dot(w, &r, &z);
        report.converged = false;
        for it in 1..=max_iterations {
            self.operator(d, c, &p, &mut ap);
            let pap = dot(w, &p, &ap);
            if !(pap.is_finite() && pap > 0.0) {
                break;
            }
            let alpha = rz / pap;
            x.par_iter_mut()
                .zip(p.par_iter())
                .for_each(|(x, &p)| *x += alpha * p);
            r.par_iter_mut()
                .zip(ap.par_iter())
                .for_each(|(r, &a)| *r -= alpha * a);
            self.precondition(&mut pre, &r, &mut z);
            report.iterations = it;
            report.error_m = max_abs(&z);
            if report.error_m < tolerance_m {
                report.converged = true;
                break;
            }
            let rz_new = dot(w, &r, &z);
            let beta = rz_new / rz;
            rz = rz_new;
            p.par_iter_mut()
                .zip(z.par_iter())
                .for_each(|(p, &z)| *p = z + beta * *p);
        }
        (x, report)
    }
}

/// The mean of each `f` × `f` block of `v` (w × h).
fn restrict(v: &[f64], w: usize, f: usize) -> Vec<f64> {
    if f == 1 {
        return v.to_vec();
    }
    let ws = w / f;
    let hs = v.len() / w / f;
    let norm = 1.0 / (f * f) as f64;
    (0..ws * hs)
        .into_par_iter()
        .map(|c| {
            let (i, j) = (c % ws, c / ws);
            let mut s = 0.0;
            for dj in 0..f {
                for di in 0..f {
                    s += v[(j * f + dj) * w + i * f + di];
                }
            }
            s * norm
        })
        .collect()
}

/// Each coarse value on its `f` × `f` cells.
fn prolong_const(v: &[f64], ws: usize, f: usize) -> Vec<f64> {
    if f == 1 {
        return v.to_vec();
    }
    let hs = v.len() / ws;
    let w = ws * f;
    (0..w * hs * f)
        .into_par_iter()
        .map(|c| v[(c / w / f) * ws + (c % w) / f])
        .collect()
}

/// Bilinear interpolation from the coarse cells' centres (x wraps, rows clamp).
fn prolong_linear(v: &[f64], ws: usize, f: usize) -> Vec<f64> {
    if f == 1 {
        return v.to_vec();
    }
    let hs = v.len() / ws;
    let w = ws * f;
    (0..w * hs * f)
        .into_par_iter()
        .map(|c| {
            let (i, j) = (c % w, c / w);
            let x = (i as f64 + 0.5) / f as f64 - 0.5;
            let y = ((j as f64 + 0.5) / f as f64 - 0.5).clamp(0.0, (hs - 1) as f64);
            let (x0, y0) = (x.floor(), y.floor());
            let (tx, ty) = (x - x0, y - y0);
            let i0 = (x0 as i64).rem_euclid(ws as i64) as usize;
            let i1 = (i0 + 1) % ws;
            let j0 = y0 as usize;
            let j1 = (j0 + 1).min(hs - 1);
            (v[j0 * ws + i0] * (1.0 - tx) + v[j0 * ws + i1] * tx) * (1.0 - ty)
                + (v[j1 * ws + i0] * (1.0 - tx) + v[j1 * ws + i1] * tx) * ty
        })
        .collect()
}

/// What [`step`] reads besides the state. Slices are per L1 cell.
pub struct StepIn<'a> {
    /// The L1 raster (block metric).
    pub dims: &'a Dims,
    /// The L1 cell's side on the Earth-normalized sphere, m.
    pub norm_dx_m: f64,
    pub gravity_m_s2: f64,
    pub sea_level_m: f64,
    pub dt_myr: f64,
    /// Elevation at the start of the step: carried with the crust, plus the plate model's uplift.
    pub h_start: &'a [f64],
    /// … after erosion and deposition, before any isostasy.
    pub h_eroded: &'a [f64],
    /// … after the local isostasy's share (`1 − strength`), before the plateau spreading.
    pub h_before_spread: &'a [f64],
    /// `plates/uplift_mm_yr`, `plates/crust_type`, `crust_age` (Myr; negative on continental
    /// crust), `plates/orogeny_age_myr` (NaN: none), `regime` (the regime, not `plates/flags`:
    /// the flags are zero on the steps the plate model resamples its crust).
    pub uplift_mm_yr: &'a [f32],
    pub crust_type: &'a [f32],
    pub crust_age_myr: &'a [f32],
    pub orogeny_age_myr: &'a [f32],
    pub regime: &'a [f32],
    /// `plates/plate_id`: plates are separate plates, with no strength across their boundaries.
    pub plate_id: &'a [f32],
    /// Land ice, m (`climate/ice_thickness_m`), when a climate stage publishes it.
    pub ice_m: Option<&'a [f32]>,
    /// The `isostasy` setting: the density of eroded and deposited rock over the mantle's.
    pub rock_share: f64,
}

/// What a step did.
#[derive(Clone, Debug, Default)]
pub struct StepOut {
    /// None when the step was not a solve step (`every_steps`).
    pub solve: Option<SolveReport>,
    /// Σ of the elevation changes, m (× the cell area: the volume).
    pub sum_change_m: f64,
    /// The largest elevation change, m.
    pub max_change_m: f64,
    /// `field::FLEXURE`, `field::DEFLECTION`, `field::ELASTIC_THICKNESS`.
    pub flexure_m: Vec<f32>,
    pub deflection_m: Vec<f32>,
    pub elastic_thickness_km: Vec<f32>,
}

/// The largest power of two that keeps the solve cell at or under [`SOLVE_CELL_M`] and divides
/// the raster.
pub fn coarsening(w: usize, h: usize, norm_dx_m: f64) -> usize {
    let mut f = 1;
    while norm_dx_m * (2 * f) as f64 <= SOLVE_CELL_M
        && w.is_multiple_of(2 * f)
        && h.is_multiple_of(2 * f)
        && w / (2 * f) >= 16
        && h / (2 * f) >= 8
    {
        f *= 2;
    }
    f
}

/// One step of flexure on the elevations `h` (m; after erosion, the local isostasy's share and
/// the plateau spreading): the step's loads join the state, the plate is solved for the whole
/// load, and `h` changes by `strength` × the sinking not yet in it.
pub fn step(
    p: &FlexureParams,
    solver: &mut Option<Solver>,
    state: &mut Option<FlexState>,
    x: &StepIn<'_>,
    h: &mut [f64],
) -> Result<StepOut, CoreError> {
    let (w, hgt) = (x.dims.w, x.dims.h);
    let n = w * hgt;
    let sea = x.sea_level_m;
    let st = state.get_or_insert_with(|| FlexState::fresh(n));
    if st.load_m.len() != n {
        *st = FlexState::fresh(n);
    }
    let lift = x.dt_myr * 1.0e3; // mm/yr over the step, in m
    let water_share = RHO_WATER / RHO_MANTLE;
    let ice_share = RHO_ICE / RHO_MANTLE;
    // Sediment under water: the density today's 0.7 factor implies (docs/design/flexure.md 3.2).
    let rock_land = x.rock_share;
    let rock_sea = water_share + 0.7 * x.rock_share * (1.0 - water_share); // ledger: ISO-047

    // 1. The loads of the step, and the roots already in the elevation.
    struct Cell {
        load: f64,
        w0: f64,
        water: f64,
        q_total: f64,
    }
    let cells: Vec<Cell> = (0..n)
        .into_par_iter()
        .map(|c| {
            // Crust that is new at this place (born this step, or another plate's edge arriving)
            // starts from its state at the start of the step: no load, no sinking, the sea over
            // it as its reference; the plate model's uplift was not added to it.
            let age = x.crust_age_myr[c];
            let fresh = st.water_m[c].is_nan() || (age >= 0.0 && f64::from(age) < x.dt_myr);
            let depth = (sea - h[c]).max(0.0);
            let (mut load, sunk, water) = if fresh {
                (0.0, 0.0, (sea - x.h_start[c]).max(0.0))
            } else {
                (
                    f64::from(st.load_m[c]),
                    f64::from(st.deflection_m[c]),
                    f64::from(st.water_m[c]),
                )
            };
            let mut root = 0.0;
            // ledger: ISO-061
            // Crust the plate model thickened: orogens, arcs and hotspot volcanism. Subsidence
            // (the ocean floor cooling, a sample pulled to trench depth) is not a load.
            let u = f64::from(x.uplift_mm_yr[c]) * lift;
            let reg = x.regime[c] as u8;
            let builds =
                x.crust_type[c] as u8 != OCEANIC || reg == regime::ARC || reg == regime::HOTSPOT;
            if !fresh && u > 0.0 && builds {
                let (q, a) = thickening(u, x.h_start[c] - u, sea);
                load += q;
                root += a;
            }
            // Erosion and deposition.
            let dm = x.h_eroded[c] - x.h_start[c];
            load += dm
                * if x.h_eroded[c] <= sea {
                    rock_sea
                } else {
                    rock_land
                };
            // Crust moved by the plateau spreading.
            let ds = h[c] - x.h_before_spread[c];
            if ds != 0.0 {
                let (q, a) = thickening(ds, x.h_before_spread[c], sea);
                load += q;
                root += a;
            }
            let ice = x.ice_m.map_or(0.0, |v| f64::from(v[c]).max(0.0));
            Cell {
                load,
                w0: sunk + root,
                water,
                q_total: load + water_share * (depth - water) + ice_share * ice,
            }
        })
        .collect();

    // 2. The plate's stiffness now.
    let f = coarsening(w, hgt, x.norm_dx_m);
    let (ws, hs) = (w / f, hgt / f);
    let dxs = x.norm_dx_m * f as f64;
    let te_fine: Vec<f64> = (0..n)
        .into_par_iter()
        .map(|c| {
            let reg = x.regime[c] as u8;
            p.elastic_thickness_km(
                x.crust_type[c] as u8 == OCEANIC,
                f64::from(x.crust_age_myr[c]),
                f64::from(x.orogeny_age_myr[c]),
                reg == regime::ARC || reg == regime::RIFT,
            )
        })
        .collect();
    let mut te = restrict(&te_fine, w, f);
    let ds = Dims::new(ws, hs, dxs, x.dims.wrap_x);
    box_blur(&ds, &mut te, 2, 2); // ledger: ISO-054 (game-choice: numerical)
    // ledger: ISO-055
    // A plate boundary is a break: a trench's subducting plate does not hold up the arc beside
    // it. The cells along a boundary are at the floor, the next row half way to it.
    let neighbours = |c: usize| {
        let (i, j) = (c % w, c / w);
        [
            j * w + (i + 1) % w,
            j * w + (i + w - 1) % w,
            j.saturating_sub(1) * w + i,
            (j + 1).min(hgt - 1) * w + i,
        ]
    };
    let edge: Vec<bool> = (0..n)
        .into_par_iter()
        .map(|c| {
            neighbours(c)
                .iter()
                .any(|&m| x.plate_id[m] != x.plate_id[c])
        })
        .collect();
    let whole: Vec<f64> = (0..n)
        .into_par_iter()
        .map(|c| {
            if edge[c] {
                0.0
            } else if neighbours(c).iter().any(|&m| edge[m]) {
                0.5
            } else {
                1.0
            }
        })
        .collect();
    let floor = p.te_min_km * p.elastic_thickness_scale;
    for (t, share) in te.iter_mut().zip(restrict(&whole, w, f)) {
        *t = floor + (*t - floor).max(0.0) * share;
    }
    let te_out: Vec<f32> = prolong_const(&te, ws, f)
        .into_iter()
        .map(|t| t as f32)
        .collect();

    st.since_solve += 1;
    let solve_now = st.since_solve >= p.every_steps;
    let mut out = StepOut {
        elastic_thickness_km: te_out,
        ..StepOut::default()
    };
    let mut delta = vec![0.0; n];
    if solve_now {
        st.since_solve = 0;
        if solver
            .as_ref()
            .is_none_or(|s| s.w != ws || s.h != hs || s.dx != dxs)
        {
            *solver = Some(Solver::new(ws, hs, dxs));
        }
        let sv = solver.as_mut().expect("built above");
        let d_now: Vec<f64> = te
            .par_iter()
            .map(|&t| rigidity_per_g(t, x.gravity_m_s2))
            .collect();
        let pick = |get: &(dyn Fn(usize) -> f64 + Sync)| -> Vec<f64> {
            let fine: Vec<f64> = (0..n).into_par_iter().map(get).collect();
            restrict(&fine, w, f)
        };
        let q = pick(&|c| cells[c].q_total);
        let w0 = pick(&|c| cells[c].w0);
        let wet = pick(&|c| if h[c] <= sea { 1.0 } else { 0.0 });
        // The rigidity the cell had (new crust: what it has now) and its frozen moment; where
        // the plate weakened the moment shrinks with it.
        let d_prev = pick(&|c| {
            let d = f64::from(st.rigidity[c]);
            if d.is_finite() && d > 0.0 { d } else { -1.0 }
        });
        let mut m0 = pick(&|c| f64::from(st.moment[c]));
        let d_solve: Vec<f64> = (0..ws * hs)
            .map(|c| {
                if d_prev[c] > 0.0 && d_prev[c] < d_now[c] {
                    d_prev[c]
                } else {
                    if d_prev[c] > d_now[c] {
                        m0[c] *= d_now[c] / d_prev[c];
                    }
                    d_now[c]
                }
            })
            .collect();
        let c_rest: Vec<f64> = wet
            .iter()
            .map(|&s| RHO_MANTLE - RHO_WATER * p.strength * s)
            .collect();
        // r = ρ_m (Q − w₀) − L(D L w₀) + L(M₀).
        let mut bend = vec![0.0; ws * hs];
        sv.bend(&d_solve, &w0, &mut bend);
        let mut frozen = vec![0.0; ws * hs];
        sv.laplacian(&m0, &mut frozen);
        let r: Vec<f64> = (0..ws * hs)
            .map(|c| RHO_MANTLE * (q[c] - w0[c]) - bend[c] + frozen[c])
            .collect();
        let (dw, report) = sv.solve(&d_solve, &c_rest, &r, p.tolerance_m, p.max_iterations);
        if dw.iter().any(|v| !v.is_finite()) {
            return Err(CoreError::InvalidState(
                "the flexure solve produced a non-finite deflection".into(),
            ));
        }
        out.solve = Some(report);
        // ledger: ISO-056
        // Rigidity gained this step joins stress-free at the new shape.
        let w_new: Vec<f64> = w0.iter().zip(&dw).map(|(a, b)| a + b).collect();
        let mut curv = vec![0.0; ws * hs];
        sv.laplacian(&w_new, &mut curv);
        for c in 0..ws * hs {
            m0[c] += (d_now[c] - d_solve[c]) * curv[c];
        }
        let m0_fine = prolong_const(&m0, ws, f);
        let d_fine = prolong_const(&d_now, ws, f);
        for c in 0..n {
            st.moment[c] = m0_fine[c] as f32;
            st.rigidity[c] = d_fine[c] as f32;
        }
        delta = prolong_linear(&dw, ws, f);
    }

    // 3. The elevation takes the sinking that is not yet in it; the state takes the step.
    let mut flex = vec![0.0f32; n];
    let (mut sum, mut peak) = (0.0, 0.0f64);
    for c in 0..n {
        let cell = &cells[c];
        let change = -p.strength * delta[c];
        h[c] += change;
        sum += change;
        peak = peak.max(change.abs());
        let depth = (sea - h[c]).max(0.0);
        let w_new = cell.w0 + delta[c];
        let load = cell.load + water_share * (depth - cell.water);
        st.load_m[c] = load as f32;
        st.deflection_m[c] = w_new as f32;
        st.water_m[c] = depth as f32;
        let ice = x.ice_m.map_or(0.0, |v| f64::from(v[c]).max(0.0));
        let restoring = if h[c] <= sea {
            RHO_MANTLE - RHO_WATER
        } else {
            RHO_MANTLE
        };
        flex[c] = (RHO_MANTLE * (load + ice_share * ice - w_new) / restoring) as f32;
    }
    st.seen_elevation_m = h.iter().map(|&z| z as f32).collect();
    out.sum_change_m = sum;
    out.max_change_m = peak;
    out.deflection_m = st.deflection_m.clone();
    out.flexure_m = flex;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const G: f64 = 9.81;

    fn uniform(n: usize, te_km: f64, g: f64) -> (Vec<f64>, Vec<f64>) {
        (vec![rigidity_per_g(te_km, g); n], vec![RHO_MANTLE; n])
    }

    /// Kelvin's kei(0) = −π/4: under a point load P the plate sinks P·l² / (8D), with
    /// l = (D / ρ_m g)^¼; in the solve's units a mass m sinks m / (8 ρ_m l²). The transforms
    /// invert each other.
    #[test]
    fn a_point_load_sinks_by_the_analytic_depth() {
        let (w, h, dx) = (256usize, 128usize, 10_000.0);
        // The transforms: forward then inverse is the identity, on even and on odd rasters.
        for (tw, th) in [(w, h), (15, 9), (12, 7)] {
            let ts = Solver::new(tw, th, dx);
            let v: Vec<f64> = (0..tw * th)
                .map(|c| ((c * 7919) % 101) as f64 - 50.0)
                .collect();
            let mut back = vec![0.0; tw * th];
            let (mut half, mut spec) = (ts.half.clone(), ts.spec.clone());
            ts.spectral.forward(&v, &mut half, &mut spec);
            ts.spectral.inverse(&mut spec, &mut back);
            let worst = v
                .iter()
                .zip(&back)
                .map(|(a, b)| (a - b).abs())
                .fold(0.0, f64::max);
            assert!(worst < 1e-9, "transform round trip {tw} × {th}: {worst}");
        }

        let mut sv = Solver::new(w, h, dx);
        let te = 30.0;
        let (d, c) = uniform(w * h, te, G);
        let l = (d[0] / RHO_MANTLE).powf(0.25);
        assert!(l / dx > 4.0, "the load is a point: l = {l}");
        let mass = 1.0e15; // kg
        let mut b = vec![0.0; w * h];
        let centre = (h / 2) * w + w / 2;
        b[centre] = mass / (dx * dx);
        let (x, rep) = sv.solve(&d, &c, &b, 1e-9, 50);
        assert!(rep.converged && rep.iterations <= 1, "{rep:?}");
        let analytic = mass / (8.0 * RHO_MANTLE * l * l);
        assert!(
            (x[centre] / analytic - 1.0).abs() < 0.03,
            "depth under the load {} against {analytic}",
            x[centre]
        );
        // Compensation: the mantle pushed aside weighs what the load does.
        let displaced: f64 = x.iter().sum::<f64>() * RHO_MANTLE * dx * dx;
        assert!((displaced / mass - 1.0).abs() < 1e-9, "{displaced}");
        // A bulge rings the hollow.
        let lowest = x.iter().cloned().fold(f64::MAX, f64::min);
        assert!(lowest < 0.0 && lowest > -0.05 * analytic, "bulge {lowest}");
    }

    /// The depth under a line load and the place of its first zero crossing, by linear
    /// interpolation, m.
    fn line_load(te_km: f64, g: f64, dx: f64) -> (f64, f64, f64, f64, f64) {
        let (w, h) = (16usize, 1024usize);
        let mut sv = Solver::new(w, h, dx);
        let (d, c) = uniform(w * h, te_km, g);
        let alpha = flexural_parameter_m(d[0]);
        let per_m = 1.0e10; // kg per metre of line
        let row = h / 2;
        let mut b = vec![0.0; w * h];
        for i in 0..w {
            b[row * w + i] = per_m / dx;
        }
        let (x, rep) = sv.solve(&d, &c, &b, 1e-9, 50);
        assert!(rep.converged, "{rep:?}");
        let profile: Vec<f64> = (row..h).map(|j| x[j * w]).collect();
        let w0 = profile[0];
        let k = profile.iter().position(|&v| v < 0.0).expect("a bulge");
        let zero = (k as f64 - 1.0 + profile[k - 1] / (profile[k - 1] - profile[k])) * dx;
        let (kb, wb) = profile
            .iter()
            .enumerate()
            .fold((0, 0.0), |a, (i, &v)| if v < a.1 { (i, v) } else { a });
        (
            w0,
            zero,
            kb as f64 * dx,
            wb,
            per_m / (2.0 * RHO_MANTLE * alpha),
        )
    }

    /// Turcotte & Schubert's line load: w₀ = V α³ / 8D, zero at 3πα/4, the bulge at πα with
    /// −e^(−π) = −4.3 % of the depth; and the width goes as Te^¾ and as g^(−¼).
    #[test]
    fn a_line_load_gives_the_analytic_profile() {
        let dx = 5000.0;
        let te = 30.0;
        let alpha = flexural_parameter_m(rigidity_per_g(te, G));
        assert!((alpha / 1000.0 - 67.0).abs() < 2.0, "α = {alpha}");
        let (w0, zero, bulge_at, bulge, analytic) = line_load(te, G, dx);
        assert!(
            (w0 / analytic - 1.0).abs() < 0.01,
            "{w0} against {analytic}"
        );
        let pi = std::f64::consts::PI;
        assert!(
            (zero / (0.75 * pi * alpha) - 1.0).abs() < 0.02,
            "zero crossing at {zero}, 3πα/4 = {}",
            0.75 * pi * alpha
        );
        assert!(
            (bulge_at - pi * alpha).abs() <= 1.5 * dx,
            "bulge at {bulge_at}, πα = {}",
            pi * alpha
        );
        assert!(
            (bulge / w0 + (-pi).exp()).abs() < 0.003,
            "bulge height {} of the depth",
            bulge / w0
        );
        // Twice the elastic thickness: 2^¾ = 1.68 times the width.
        let (_, zero2, ..) = line_load(2.0 * te, G, dx);
        assert!(
            (zero2 / zero - 2.0f64.powf(0.75)).abs() < 0.02,
            "width ratio {}",
            zero2 / zero
        );
        // Mars's gravity: wider by (9.81 / 3.71)^¼ = 1.275, and so shallower under the line.
        let (w0m, zerom, ..) = line_load(te, 3.71, dx);
        assert!(
            (zerom / zero - (G / 3.71f64).powf(0.25)).abs() < 0.02,
            "{}",
            zerom / zero
        );
        assert!(w0m < w0);
    }

    fn patchy(w: usize, h: usize, lo: f64, hi: f64) -> Vec<f64> {
        // Stiff "cratons" in a soft plate, smoothed as the stage smooths Te.
        let mut te: Vec<f64> = (0..w * h)
            .map(|c| {
                let (i, j) = (c % w, c / w);
                if ((i / 24) + (j / 24)).is_multiple_of(3) {
                    hi
                } else {
                    lo
                }
            })
            .collect();
        box_blur(&Dims::new(w, h, 1.0, true), &mut te, 2, 2);
        te
    }

    /// With a stiffness that varies a thousandfold the solve converges, the answer satisfies the
    /// equation, and the buoyancy of the deflection balances the load, under water too.
    #[test]
    fn a_patchy_plate_converges_and_compensates() {
        let (w, h, dx) = (192usize, 96usize, 39_000.0);
        let mut sv = Solver::new(w, h, dx);
        let te = patchy(w, h, 6.0, 70.0);
        let d: Vec<f64> = te.iter().map(|&t| rigidity_per_g(t, G)).collect();
        let c: Vec<f64> = (0..w * h)
            .map(|k| {
                if (k % w) < w / 3 {
                    RHO_MANTLE - RHO_WATER
                } else {
                    RHO_MANTLE
                }
            })
            .collect();
        let mut b = vec![0.0; w * h];
        for (k, load) in b.iter_mut().enumerate() {
            let (i, j) = (k % w, k / w);
            if (i % 37 < 3 && j % 29 < 9) || (i + 2 * j) % 61 == 0 {
                *load = RHO_MANTLE * (500.0 + ((k * 31) % 17) as f64 * 250.0);
            }
        }
        let (x, rep) = sv.solve(&d, &c, &b, 0.01, 400);
        assert!(rep.converged && rep.iterations < 150, "{rep:?}");
        // The answer satisfies the equation: what is left of the load asks for no more sinking.
        let mut ax = vec![0.0; w * h];
        sv.operator(&d, &c, &x, &mut ax);
        let left: Vec<f64> = b.iter().zip(&ax).map(|(b, a)| b - a).collect();
        let mut more = vec![0.0; w * h];
        sv.spectral_solve(&Bank::new(&d, &c, dx), &left, &mut more);
        let worst = max_abs(&more);
        assert!(worst < 0.05, "sinking still asked for, m: {worst}");
        let load: f64 = b.iter().sum();
        let buoyancy: f64 = x.iter().zip(&c).map(|(x, c)| x * c).sum();
        assert!(
            (buoyancy / load - 1.0).abs() < 1e-3,
            "compensation {}",
            buoyancy / load
        );
        // A load on the soft plate sinks nearly where it lies; on a craton far less.
        let airy_wet = RHO_MANTLE / (RHO_MANTLE - RHO_WATER);
        assert!(x.iter().cloned().fold(f64::MIN, f64::max) < 4500.0 * airy_wet);
    }

    /// The weakest and the stiffest plates the editor allows (scale 0.25 and 3 on 5–80 km), and
    /// beyond: finite, converged, compensated.
    #[test]
    fn the_solve_is_stable_at_the_editors_extremes() {
        let (w, h, dx) = (128usize, 64usize, 39_000.0);
        let mut sv = Solver::new(w, h, dx);
        for (lo, hi) in [(1.25, 20.0), (15.0, 240.0), (0.1, 1.0), (300.0, 500.0)] {
            let te = patchy(w, h, lo, hi);
            let d: Vec<f64> = te.iter().map(|&t| rigidity_per_g(t, G)).collect();
            let c = vec![RHO_MANTLE; w * h];
            let mut b = vec![0.0; w * h];
            for load in b.iter_mut().step_by(53) {
                *load = RHO_MANTLE * 3000.0;
            }
            let (x, rep) = sv.solve(&d, &c, &b, 0.1, 400);
            assert!(rep.converged, "Te {lo}–{hi}: {rep:?}");
            assert!(x.iter().all(|v| v.is_finite()));
            let ratio = x.iter().sum::<f64>() * RHO_MANTLE / b.iter().sum::<f64>();
            assert!((ratio - 1.0).abs() < 1e-3, "Te {lo}–{hi}: {ratio}");
            assert!(x.iter().cloned().fold(f64::MIN, f64::max) <= 3000.0 * 1.001);
        }
    }

    #[test]
    fn the_elastic_thickness_follows_the_crusts_thermal_state() {
        let p = FlexureParams::default();
        assert_eq!(p.elastic_thickness_km(true, 0.0, f64::NAN, false), 5.0);
        assert!((p.elastic_thickness_km(true, 100.0, f64::NAN, false) - 27.0).abs() < 1e-9);
        assert_eq!(p.elastic_thickness_km(true, 400.0, f64::NAN, false), 40.0);
        assert_eq!(p.elastic_thickness_km(false, -1.0, f64::NAN, false), 80.0);
        assert_eq!(p.elastic_thickness_km(false, -1.0, 0.0, false), 5.0);
        let mid = p.elastic_thickness_km(false, -1.0, 300.0, false);
        assert!((mid - 80.0 * (1.0 - (-1.0f64).exp())).abs() < 1e-9);
        assert_eq!(p.elastic_thickness_km(false, -1.0, f64::NAN, true), 5.0);
        let stiff = FlexureParams {
            elastic_thickness_scale: 2.0,
            ..p.clone()
        };
        assert_eq!(
            stiff.elastic_thickness_km(false, -1.0, f64::NAN, false),
            160.0
        );
        p.validate().unwrap();
        assert!(FlexureParams { strength: 1.5, ..p }.validate().is_err());
        // The Airy shares are the plate model's: 1 km of uplift on land is 5.5 km of crust.
        let (q, a) = thickening(1000.0, 500.0, 0.0);
        assert!((a - 4500.0).abs() < 1e-6 && (q - 5500.0 * 2700.0 / 3300.0).abs() < 1e-6);
        // Under water the root is smaller, and a change that crosses the sea is split.
        let (_, under) = thickening(1000.0, -3000.0, 0.0);
        assert!(under < a && under > 2000.0);
        let (_, across) = thickening(1000.0, -500.0, 0.0);
        assert!((across - 0.5 * (a + under)).abs() < 1e-6);
        let (qn, an) = thickening(-1000.0, 1500.0, 0.0);
        assert!((qn + q).abs() < 1e-6 && (an + a).abs() < 1e-6);
    }

    struct World {
        d: Dims,
        norm_dx: f64,
        h: Vec<f64>,
        uplift: Vec<f32>,
        crust: Vec<f32>,
        age: Vec<f32>,
        orogeny: Vec<f32>,
        regime: Vec<f32>,
        plate: Vec<f32>,
        ice: Vec<f32>,
    }

    impl World {
        /// A continent of cratons at `z` m everywhere.
        fn new(w: usize, h: usize, z: f64) -> World {
            let n = w * h;
            World {
                d: Dims::new(w, h, 4800.0, true),
                norm_dx: 39_000.0,
                h: vec![z; n],
                uplift: vec![0.0; n],
                crust: vec![1.0; n],
                age: vec![-1.0; n],
                orogeny: vec![f32::NAN; n],
                regime: vec![f32::from(regime::CRATON); n],
                plate: vec![0.0; n],
                ice: vec![0.0; n],
            }
        }

        /// One step: `eroded` is the surface's net change, `spread` the spreading's.
        fn step(
            &mut self,
            p: &FlexureParams,
            sv: &mut Option<Solver>,
            st: &mut Option<FlexState>,
            eroded: &[f64],
            sea: f64,
        ) -> StepOut {
            let dt_myr = 2.0;
            let h_start: Vec<f64> = self
                .h
                .iter()
                .zip(&self.uplift)
                .map(|(&z, &u)| z + f64::from(u) * dt_myr * 1.0e3)
                .collect();
            let h_eroded: Vec<f64> = h_start.iter().zip(eroded).map(|(a, b)| a + b).collect();
            let mut h = h_eroded.clone();
            let out = step(
                p,
                sv,
                st,
                &StepIn {
                    dims: &self.d,
                    norm_dx_m: self.norm_dx,
                    gravity_m_s2: G,
                    sea_level_m: sea,
                    dt_myr,
                    h_start: &h_start,
                    h_eroded: &h_eroded,
                    h_before_spread: &h_eroded,
                    uplift_mm_yr: &self.uplift,
                    crust_type: &self.crust,
                    crust_age_myr: &self.age,
                    orogeny_age_myr: &self.orogeny,
                    regime: &self.regime,
                    plate_id: &self.plate,
                    ice_m: Some(&self.ice),
                    rock_share: 0.8,
                },
                &mut h,
            )
            .unwrap();
            self.h = h;
            out
        }
    }

    fn on() -> FlexureParams {
        FlexureParams {
            strength: 1.0,
            tolerance_m: 0.01,
            max_iterations: 200,
            ..FlexureParams::default()
        }
    }

    /// The first step only takes the reference; the stage's own first step has no uplift in it.
    fn started(wd: &mut World, p: &FlexureParams, sea: f64) -> (Option<Solver>, Option<FlexState>) {
        let (mut sv, mut st) = (None, None);
        let n = wd.h.len();
        let before = wd.h.clone();
        let out = wd.step(p, &mut sv, &mut st, &vec![0.0; n], sea);
        assert_eq!(wd.h, before, "an unloaded plate does not move");
        assert_eq!(out.max_change_m, 0.0);
        (sv, st)
    }

    /// On a plate with no strength the stage is local isostasy: erosion rebounds by `isostasy`
    /// (0.7 of it under water), the plate model's uplift is untouched, and ice sinks the ground
    /// by ρ_i/ρ_m of its thickness and lets it back up when it melts.
    #[test]
    fn a_plate_without_strength_is_local_isostasy() {
        let p = FlexureParams {
            te_craton_km: 0.001,
            te_min_km: 0.001,
            te_ocean_max_km: 0.001,
            ..on()
        };
        let (w, h) = (64usize, 32usize);
        let n = w * h;
        let mut wd = World::new(w, h, 1000.0);
        for c in 0..n {
            if c % w < 16 {
                wd.h[c] = -2000.0;
            }
        }
        let (mut sv, mut st) = started(&mut wd, &p, 0.0);
        let before = wd.h.clone();
        let land = 10 * w + 40;
        let sea_cell = 10 * w + 5;
        let lifted = 20 * w + 50;
        let mut eroded = vec![0.0; n];
        eroded[land] = -900.0;
        eroded[sea_cell] = 500.0;
        wd.uplift[lifted] = 0.25; // 500 m over the step
        wd.step(&p, &mut sv, &mut st, &eroded, 0.0);
        let change = |c: usize| wd.h[c] - before[c];
        assert!(
            (change(land) - (-900.0 + 720.0)).abs() < 0.5,
            "{}",
            change(land)
        );
        assert!(
            (change(sea_cell) - (500.0 - 0.7 * 0.8 * 500.0)).abs() < 0.5,
            "{}",
            change(sea_cell)
        );
        assert!((change(lifted) - 500.0).abs() < 0.5, "{}", change(lifted));
        assert!(change(0).abs() < 1e-6 && change(n - 1).abs() < 1e-6);
        // Ice: 3 km of it over a wide sheet.
        wd.uplift[lifted] = 0.0;
        let bare = wd.h.clone();
        for c in 0..n {
            if c % w >= 32 {
                wd.ice[c] = 3000.0;
            }
        }
        wd.step(&p, &mut sv, &mut st, &vec![0.0; n], 0.0);
        let under = 16 * w + 48;
        assert!(
            (bare[under] - wd.h[under] - 3000.0 * RHO_ICE / RHO_MANTLE).abs() < 1.0,
            "{}",
            bare[under] - wd.h[under]
        );
        wd.ice = vec![0.0; n];
        wd.step(&p, &mut sv, &mut st, &vec![0.0; n], 0.0);
        assert!((wd.h[under] - bare[under]).abs() < 0.5, "rebound");
    }

    /// A belt raised by the plate model on a craton: the plate holds part of its root, so the
    /// belt stands higher than its Airy height, a basin lies in front of it and a bulge beyond;
    /// the buoyancy of all the sinking is the belt's weight. When the belt is eroded away the
    /// basin comes back.
    #[test]
    fn a_belt_on_a_stiff_plate_gets_a_foreland_basin() {
        let p = on();
        let (w, h) = (32usize, 256usize);
        let n = w * h;
        let mut wd = World::new(w, h, 300.0);
        // No sea in this world: every load is rock on land.
        let sea = -1.0e6;
        let (mut sv, mut st) = started(&mut wd, &p, sea);
        let belt = |c: usize| (124..132).contains(&(c / w));
        for c in 0..n {
            if belt(c) {
                wd.uplift[c] = 1.0; // 2 km of Airy elevation in the step
            }
        }
        let out = wd.step(&p, &mut sv, &mut st, &vec![0.0; n], sea);
        let rep = out.solve.unwrap();
        assert!(rep.converged, "{rep:?}");
        let col = |j: usize| wd.h[j * w + 3];
        assert!(
            col(127) > 2300.0 + 500.0,
            "the belt stands above Airy: {}",
            col(127)
        );
        let basin = (132..150).map(col).fold(f64::MAX, f64::min);
        assert!(basin < 300.0 - 300.0, "a basin in front: {basin}");
        let bulge = (132..200).map(col).fold(f64::MIN, f64::max);
        assert!(bulge > 300.0 + 5.0, "a bulge beyond: {bulge}");
        assert!((col(250) - 300.0).abs() < 1.0, "far away nothing moved");
        // The basin is `surface/flexure_m` below zero, the belt above.
        assert!(out.flexure_m[135 * w + 3] < -100.0 && out.flexure_m[127 * w + 3] > 500.0);
        // Compensation: Σ sinking = Σ load (m of mantle), all on land.
        let load: f64 = st
            .as_ref()
            .unwrap()
            .load_m
            .iter()
            .map(|&q| f64::from(q))
            .sum();
        let sunk: f64 = out.deflection_m.iter().map(|&x| f64::from(x)).sum();
        assert!((sunk / load - 1.0).abs() < 1e-3, "{sunk} against {load}");
        // Erosion takes the belt away again (the crust thickened, 5.5 × the uplift): no load is
        // left and the basin is gone.
        wd.uplift = vec![0.0; n];
        let mut eroded = vec![0.0; n];
        for (c, dm) in eroded.iter_mut().enumerate() {
            if belt(c) {
                *dm = -2000.0 * 5.5 * (2700.0 / 3300.0) / 0.8;
            }
        }
        wd.step(&p, &mut sv, &mut st, &eroded, sea);
        let left = st
            .as_ref()
            .unwrap()
            .load_m
            .iter()
            .fold(0.0f32, |a, &q| a.max(q.abs()));
        assert!(left < 1.0, "load left: {left}");
        let back = wd.h[137 * w + 3];
        assert!((back - 300.0).abs() < 2.0, "the basin rebounded: {back}");
    }

    /// A plate boundary is a break: the same belt beside a boundary pulls its own plate down and
    /// leaves the other plate where it was.
    #[test]
    fn a_plate_boundary_is_a_break() {
        let p = on();
        let (w, h) = (32usize, 128usize);
        let n = w * h;
        let sea = -1.0e6;
        let basin = |two_plates: bool| {
            let mut wd = World::new(w, h, 300.0);
            if two_plates {
                for c in 0..n {
                    if c / w >= 66 {
                        wd.plate[c] = 1.0;
                    }
                }
            }
            let (mut sv, mut st) = started(&mut wd, &p, sea);
            for c in 0..n {
                if (56..64).contains(&(c / w)) {
                    wd.uplift[c] = 1.0;
                }
            }
            wd.step(&p, &mut sv, &mut st, &vec![0.0; n], sea);
            // How far the ground sank five cells in front of the belt, on either side.
            (300.0 - wd.h[51 * w], 300.0 - wd.h[68 * w])
        };
        let (own, other) = basin(false);
        assert!(
            own > 100.0 && (own - other).abs() < 1.0,
            "one plate: {own}, {other}"
        );
        let (own2, other2) = basin(true);
        assert!(own2 > 100.0, "its own plate still bends: {own2}");
        assert!(
            other2.abs() < 0.2 * other,
            "the other plate is left alone: {other2} (one plate: {other})"
        );
    }

    /// A belt built while its crust was weak has a local root. When the crust stiffens nothing
    /// moves; a load added afterwards meets the stiff plate.
    #[test]
    fn a_plate_that_stiffens_keeps_its_shape() {
        let p = on();
        let (w, h) = (32usize, 128usize);
        let n = w * h;
        let mut wd = World::new(w, h, 300.0);
        wd.orogeny = vec![0.0; n]; // a young orogen everywhere: Te at its floor
        let (mut sv, mut st) = started(&mut wd, &p, 0.0);
        let belt = |c: usize| (60..64).contains(&(c / w));
        for c in 0..n {
            if belt(c) {
                wd.uplift[c] = 1.0;
            }
        }
        wd.step(&p, &mut sv, &mut st, &vec![0.0; n], 0.0);
        wd.uplift = vec![0.0; n];
        let weak = wd.h.clone();
        assert!(
            weak[62 * w] < 2300.0 + 400.0,
            "nearly Airy on the weak plate: {}",
            weak[62 * w]
        );
        // A long time later the crust is a craton.
        wd.orogeny = vec![f32::NAN; n];
        let out = wd.step(&p, &mut sv, &mut st, &vec![0.0; n], 0.0);
        assert!(
            out.max_change_m < 1.0,
            "stiffening moved the ground by {}",
            out.max_change_m
        );
        assert!(out.elastic_thickness_km[0] > 79.0);
        // The same load again, now on the stiff plate: it is held up.
        let before = wd.h.clone();
        for c in 0..n {
            if belt(c) {
                wd.uplift[c] = 1.0;
            }
        }
        wd.step(&p, &mut sv, &mut st, &vec![0.0; n], 0.0);
        let rise = wd.h[62 * w] - before[62 * w];
        assert!(rise > 2000.0 + 3000.0, "held up by the craton: {rise}");
    }

    /// The solve grid: one L1 cell on the quick presets, two on the full ones, and the same
    /// answer through a coarsened solve.
    #[test]
    fn fine_rasters_solve_on_a_coarser_grid() {
        assert_eq!(coarsening(1024, 512, 39_090.0), 1);
        assert_eq!(coarsening(2048, 1024, 19_545.0), 2);
        assert_eq!(coarsening(4096, 2048, 9_772.0), 4);
        assert_eq!(coarsening(256, 128, 156_000.0), 1);
        let v: Vec<f64> = (0..64).map(|c| c as f64).collect();
        let r = restrict(&v, 8, 2);
        assert_eq!(r.len(), 16);
        assert_eq!(r[0], (0.0 + 1.0 + 8.0 + 9.0) / 4.0);
        assert_eq!(restrict(&prolong_const(&r, 4, 2), 8, 2), r);
        let lin = prolong_linear(&r, 4, 2);
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        assert!((mean(&lin) - mean(&r)).abs() < 0.5);
        // A wide ice sheet through the coarsened solve sinks the ground as it does on L1.
        let p = FlexureParams {
            te_craton_km: 20.0,
            ..on()
        };
        let sink = |norm_dx: f64, w: usize, h: usize| {
            let mut wd = World::new(w, h, 500.0);
            wd.norm_dx = norm_dx;
            let (mut sv, mut st) = started(&mut wd, &p, 0.0);
            for c in 0..w * h {
                if (c / w) >= h / 4 && (c / w) < 3 * h / 4 {
                    wd.ice[c] = 2000.0;
                }
            }
            wd.step(&p, &mut sv, &mut st, &vec![0.0; w * h], 0.0);
            assert_eq!(sv.as_ref().unwrap().w, 32);
            500.0 - wd.h[(h / 2) * w]
        };
        let (a, b) = (sink(39_000.0, 32, 64), sink(19_500.0, 64, 128));
        assert!((a - 2000.0 * RHO_ICE / RHO_MANTLE).abs() < 5.0, "{a}");
        assert!((a - b).abs() < 5.0, "{a} and {b}");
    }
}
