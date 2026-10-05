// SPDX-License-Identifier: LGPL-3.0-or-later
//! The rain's drying: an upwind budget of water vapour and cloud water carried by the wind
//! (`climate.rain_model = upwind`; Ben, decisions log 2026-10-04, "S&B + upwind moisture budget";
//! `docs/science/upwind-rain.md`).
//!
//! The scheme is Hergarten & Robl 2022's linear feedback precipitation model (LFPM 1.0, GMD 15,
//! 2063–2084; written from the paper, not from OpenLEM's GPL code) taken from a mountain range to
//! a planet. LFPM carries two vertically integrated water species along the wind: vapour, which
//! condenses at a rate W_v/τ_c, and cloud water, which falls out at W_c/τ_f and partly evaporates
//! again at β·W_c/τ_c (its equations 10–11), with the land's evapotranspiration returning a share
//! ε of the rain to the vapour (its equations 42–43). Here, per cell and month, in steady state
//! (the vapour lives about a week, much less than a month):
//!
//! ```text
//! ∇·(V W_v) − ∇·(K ∇W_v) = E_sea(W_v) + ε P − W_v/τ_c + β W_c/τ_c − k_s W_v
//! ∇·(V W_c) − ∇·(K ∇W_c) = W_v/τ_c − β W_c/τ_c − W_c/τ_f
//! P = W_c/τ_f
//! ```
//!
//! - **The sea is the source.** LFPM feeds moisture in at an upwind boundary; a planet has none,
//!   so the open sea evaporates by the bulk formula E = ρ_a C_E (|V| + u_t) q_s(T_sea) (1 − h_s
//!   W_v/W_ref), the near-surface air's humidity taken as h_s q_s at the reference column W_ref =
//!   r_ref W_sat(T̄, z) and in proportion to the column's water (linear in W_v, so the system stays
//!   linear and the evaporation is what keeps the sea's air moist).
//! - **Where the air rains.** The circulation (the caller's: the belt rain of the belt latitude,
//!   drier under the subtropical highs' eastern flanks and wetter under their western ones, the
//!   cold sea's stability, the plateau factor; `snapshot.rs`) sets how fast a column at the
//!   reference water W_ref rains, P_ref, averaged over each cell. The condensation time follows
//!   from the local balance P = W_v P_ref/W_ref: τ_c = W_ref/P_ref − β τ_f. So the sea's air
//!   rains about the circulation's rain, and air carried inland rains in proportion to the water
//!   it still holds: the drying is the rain itself, and the water closes exactly. On L1 the
//!   caller's own circulation rain is multiplied by the cell's P/P_ref (bilinear), so the fine
//!   pattern stays the circulation's and the amount the budget's.
//! - **The sea's marks.** Which flank of which basin the air came from decides the subtropical
//!   highs' subsidence over land, and the sea surface it crossed its stability. The vapour
//!   carries both as tracers from where it evaporated ([`tracers`]); the land's recycled
//!   vapour carries neither, so the marks fade inland as the land's own water replaces the
//!   sea's.
//! - **Land returns part of its rain** to the air: ε = ET/P from Fu's Budyko curve on the cell's
//!   annual rain and potential evaporation (the surface stage's), so wet warm land recycles and
//!   cold or dry land does not.
//! - **Mountains** (`k_s`): the Smith & Barstad rain the L1 terrain takes out of the column
//!   (`orographic.rs`, the snapshot's second pass), as a rate per unit vapour.
//!
//! **Any grid.** The budget sees cells (areas) and faces (two cells, a length, a centre distance
//! and the face normal in each cell's east–north basis), never rows or columns, so a lat-lon,
//! Lambert, cube or HEALPix grid only has to list its faces ([`Mesh`]; [`Mesh::from_grid`] for
//! the coarse climate grid of `coarse.rs`). The winds are given per cell in its local east–north
//! basis.
//!
//! **Numerics.** First-order upwind fluxes in flux form plus a two-point diffusive flux, so every
//! face's flux leaves one cell and enters the other (Σ A E + Σ A ε P = Σ A P + Σ A k_s W_v at
//! convergence, a test), and the matrix is an M-matrix (outflow and the sinks on the diagonal,
//! inflow off it). Block Gauss–Seidel, a 2 × 2 solve per cell, sweeps the cells from upwind to
//! downwind (a depth-first order of the month's flow graph, so one sweep carries a value the whole
//! way along an acyclic path), alternating with the reverse order for the diffusion, until no
//! cell's water changes by more than 10⁻⁹ of the largest.

use rayon::prelude::*;

use super::coarse::Grid;

/// A face between cells `a` and `b`.
#[derive(Clone, Debug, PartialEq)]
pub struct Face {
    pub a: usize,
    pub b: usize,
    /// The face's length and the distance between the two centres, m.
    pub len_m: f64,
    pub dist_m: f64,
    /// The unit normal from `a` to `b` in `a`'s and in `b`'s local (east, north) basis (the same
    /// vector on a lat-lon grid; turned across a cube's edge).
    pub n_a: [f64; 2],
    pub n_b: [f64; 2],
}

/// The cells and faces the budget runs on.
#[derive(Clone, Debug, PartialEq)]
pub struct Mesh {
    /// Cell areas, m².
    pub area_m2: Vec<f64>,
    pub faces: Vec<Face>,
}

impl Mesh {
    /// The faces of a coarse climate grid (`coarse.rs`; rows north first, x wrapping) on a sphere
    /// of radius `radius_m`: every cell's east face and the face to the row south of it.
    pub fn from_grid(g: &Grid, radius_m: f64) -> Mesh {
        let (w, h) = (g.w, g.h);
        let r = radius_m;
        let area_m2 = g.areas().iter().map(|a| a * r * r).collect();
        let mut faces = Vec::with_capacity(2 * w * h);
        for j in 0..h {
            for i in 0..w {
                let c = j * w + i;
                if w > 1 {
                    let i1 = (i + 1) % w;
                    let g_ew = g.g_ew(i, j);
                    let len = r * g.dphi(j);
                    faces.push(Face {
                        a: c,
                        b: j * w + i1,
                        len_m: len,
                        dist_m: len / g_ew.max(1e-300),
                        n_a: [1.0, 0.0],
                        n_b: [1.0, 0.0],
                    });
                }
                if j + 1 < h {
                    let len = r * g.lat_edge[j + 1].to_radians().cos().max(0.0) * g.lon_w[i];
                    let dist = r * (g.lat_c(j) - g.lat_c(j + 1)).to_radians();
                    if len > 0.0 && dist > 0.0 {
                        faces.push(Face {
                            a: c,
                            b: c + w,
                            len_m: len,
                            dist_m: dist,
                            n_a: [0.0, -1.0],
                            n_b: [0.0, -1.0],
                        });
                    }
                }
            }
        }
        Mesh { area_m2, faces }
    }

    pub fn cells(&self) -> usize {
        self.area_m2.len()
    }

    /// `x` smoothed over a length `l_m`: the solution of (1 − l² ∇²) y = x on the mesh (a
    /// screened Poisson kernel, decaying as e^(−r/l)), by Gauss–Seidel. Area-weighted sums are
    /// kept: Σ A y = Σ A x.
    pub fn smooth(&self, x: &[f64], l_m: f64) -> Vec<f64> {
        let n = self.cells();
        let mut nb: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
        for f in &self.faces {
            let g = l_m * l_m * f.len_m / f.dist_m.max(1e-300);
            nb[f.a].push((f.b, g));
            nb[f.b].push((f.a, g));
        }
        let diag: Vec<f64> = (0..n)
            .map(|c| self.area_m2[c] + nb[c].iter().map(|(_, g)| g).sum::<f64>())
            .collect();
        let mut y = x.to_vec();
        for _ in 0..2000 {
            let (mut worst, mut big) = (0.0f64, 1e-300f64);
            for c in 0..n {
                let r: f64 = nb[c].iter().map(|&(o, g)| g * y[o]).sum();
                let ny = (self.area_m2[c] * x[c] + r) / diag[c];
                worst = worst.max((ny - y[c]).abs());
                big = big.max(ny.abs());
                y[c] = ny;
            }
            if worst <= 1e-10 * big {
                break;
            }
        }
        y
    }
}

/// How far a sea basin's side of the subtropical highs reaches inland, m: about the Rossby radius
/// of deformation in the subtropics (from memory).
pub const HIGHS_REACH_M: f64 = 1.0e6; // ledger: CLM-192
/// The share of sea within the reach below which the basin's side fades inland.
const HIGHS_SEA_MIN: f64 = 0.25; // game-choice: where the flow fades over a continent

/// Where each cell lies under the subtropical highs, in [−1, 1] (east positive): over the sea its
/// basin position (`basin`, over the cell's sea share `sea`), over land the sea's within the
/// highs' reach, so a coast and the land behind it take the side of the basin they border and the
/// value fades a few reaches inland. A property of the place, not of where its air came from.
pub fn flank_position(mesh: &Mesh, basin: &[f64], sea: &[f64]) -> Vec<f64> {
    let n = mesh.cells();
    let num = mesh.smooth(
        &(0..n).map(|c| basin[c] * sea[c]).collect::<Vec<_>>(),
        HIGHS_REACH_M,
    );
    let den = mesh.smooth(sea, HIGHS_REACH_M);
    (0..n)
        .map(|c| (num[c] / den[c].max(HIGHS_SEA_MIN)).clamp(-1.0, 1.0))
        .collect()
}

/// What the budget needs of one month, per cell.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Month {
    /// The wind, m/s, in the cell's (east, north) basis.
    pub u: Vec<f64>,
    pub v: Vec<f64>,
    /// The sea's evaporation E = e0 − e1·W_v (kg/m²/s and 1/s), already times the cell's open
    /// sea share.
    pub e0: Vec<f64>,
    pub e1: Vec<f64>,
    /// Condensation rate 1/τ_c at the reference water, 1/s, and the reference water W_ref,
    /// kg/m² (the condensation grows as e^(b (W_v/W_ref − 1)) about it).
    pub k_c: Vec<f64>,
    pub w_ref: Vec<f64>,
    /// The share of the rain the cell's land gives back to the air (ε × land share).
    pub eps: Vec<f64>,
    /// The mountains' sink on the vapour, 1/s.
    pub k_s: Vec<f64>,
}

/// The budget's constants.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Constants {
    /// Eddy diffusivity of both species, m²/s (LFPM's transversal dispersion).
    pub k_q: f64,
    /// Cloud water's fallout time, s, and its re-evaporation against condensation (LFPM's β).
    pub tau_f_s: f64,
    pub beta: f64,
    /// How fast condensation grows with the column's relative humidity: b in
    /// e^(b (W_v/W_ref − 1)); 0 keeps LFPM's linear condensation.
    pub rh_gain: f64,
}

/// One month's steady state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Solved {
    /// Vapour and cloud water, kg/m².
    pub w_v: Vec<f64>,
    pub w_c: Vec<f64>,
    /// Precipitation W_c/τ_f, the sea's evaporation and the land's, kg/m²/s.
    pub p: Vec<f64>,
    pub e_sea: Vec<f64>,
    pub e_land: Vec<f64>,
    /// |Σ A (E_sea + E_land) − Σ A (P + k_s W_v)| / Σ A (E_sea + E_land).
    pub closure: f64,
    pub sweeps: u32,
}

/// The upwind-to-downwind order of the cells under face fluxes `flux` (m³/s per unit water, a→b
/// positive): a depth-first post-order of the downstream graph, reversed, so on an acyclic flow
/// every cell comes after all cells upwind of it; cycles are cut where the search closes them.
fn flow_order(n: usize, faces: &[Face], flux: &[f64]) -> Vec<usize> {
    let mut down: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut has_up = vec![false; n];
    for (f, &q) in faces.iter().zip(flux) {
        if q > 0.0 {
            down[f.a].push(f.b);
            has_up[f.b] = true;
        } else if q < 0.0 {
            down[f.b].push(f.a);
            has_up[f.a] = true;
        }
    }
    let mut seen = vec![false; n];
    let mut post = Vec::with_capacity(n);
    // Sources first (cells nothing flows into), then whatever is left (cycles).
    let starts = (0..n).filter(|&c| !has_up[c]).chain(0..n);
    let mut stack: Vec<(usize, usize)> = Vec::new();
    for s in starts {
        if seen[s] {
            continue;
        }
        seen[s] = true;
        stack.push((s, 0));
        while let Some(&mut (c, ref mut k)) = stack.last_mut() {
            if *k < down[c].len() {
                let d = down[c][*k];
                *k += 1;
                if !seen[d] {
                    seen[d] = true;
                    stack.push((d, 0));
                }
            } else {
                post.push(c);
                stack.pop();
            }
        }
    }
    post.reverse();
    post
}

/// The month's transport: per cell the outflow and diffusion on the diagonal, the inflow
/// neighbours (cell, coefficient), and the upwind-to-downwind order.
struct Transport {
    diag: Vec<f64>,
    inflow: Vec<Vec<(usize, f64)>>,
    order: Vec<usize>,
}

fn transport(mesh: &Mesh, k: &Constants, mo: &Month) -> Transport {
    let n = mesh.cells();
    let faces = &mesh.faces;
    // Face volume fluxes (normal wind × length) and diffusive conductances.
    let flux: Vec<f64> = faces
        .iter()
        .map(|f| {
            let ua = mo.u[f.a] * f.n_a[0] + mo.v[f.a] * f.n_a[1];
            let ub = mo.u[f.b] * f.n_b[0] + mo.v[f.b] * f.n_b[1];
            0.5 * (ua + ub) * f.len_m
        })
        .collect();
    let cond: Vec<f64> = faces
        .iter()
        .map(|f| k.k_q * f.len_m / f.dist_m.max(1e-300))
        .collect();
    let mut diag = vec![0.0f64; n];
    let mut inflow: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];
    for ((f, &q), &d) in faces.iter().zip(&flux).zip(&cond) {
        let (qp, qm) = (q.max(0.0), q.min(0.0));
        // Flux a→b = qp·W_a + qm·W_b + d·(W_a − W_b).
        diag[f.a] += qp + d;
        diag[f.b] += -qm + d;
        if qp + d > 0.0 {
            inflow[f.b].push((f.a, qp + d));
        }
        if -qm + d > 0.0 {
            inflow[f.a].push((f.b, -qm + d));
        }
    }
    let order = flow_order(n, faces, &flux);
    Transport {
        diag,
        inflow,
        order,
    }
}

/// Where the vapour of a solved month came from: the mixing ratios of `K` tracers that the sea's
/// evaporation puts into the air at its cell's values `src` (the land's recycled vapour carries
/// none, so a tracer of 1 everywhere gives the share of the vapour that evaporated from the sea).
/// The tracers move with the vapour and leave with its condensation and the mountains' sink
/// (cloud water that evaporates again gives them back), the same upwind finite volumes as the
/// vapour; with no recycling and a tracer of 1 everywhere the ratio is 1 everywhere (a test).
pub fn tracers<const K: usize>(
    mesh: &Mesh,
    k: &Constants,
    mo: &Month,
    s: &Solved,
    src: &[[f64; K]],
) -> Vec<[f64; K]> {
    let n = mesh.cells();
    let Transport {
        diag,
        inflow,
        order,
    } = transport(mesh, k, mo);
    // Each cell's loss per unit vapour and its source of tracer mass.
    let mut dd = vec![0.0f64; n];
    let mut sv = vec![[0.0f64; K]; n];
    for c in 0..n {
        let a = mesh.area_m2[c];
        let (w, kc) = (s.w_v[c], mo.k_c[c]);
        let wr = mo.w_ref[c].max(1e-9);
        let cond = kc * w * (k.rh_gain * (w / wr - 1.0)).min(30.0).exp();
        let back = k.beta * kc * s.w_c[c];
        let loss = if w > 1e-12 {
            ((cond - back) / w).max(0.0) + mo.k_s[c]
        } else {
            0.0
        };
        dd[c] = diag[c] + a * loss;
        let e = a * s.e_sea[c].max(0.0);
        sv[c] = src[c].map(|x| e * x);
    }
    let mut t = vec![[0.0f64; K]; n];
    for it in 0..4000u32 {
        let fwd = it % 2 == 0;
        let (mut worst, mut big) = (0.0f64, 1e-30f64);
        for idx in 0..n {
            let c = if fwd { order[idx] } else { order[n - 1 - idx] };
            let mut r = sv[c];
            for &(o, coef) in &inflow[c] {
                for (rk, tk) in r.iter_mut().zip(&t[o]) {
                    *rk += coef * tk;
                }
            }
            let nt = r.map(|x| x / dd[c].max(1e-300));
            for (a, b) in nt.iter().zip(&t[c]) {
                worst = worst.max((a - b).abs());
                big = big.max(a.abs());
            }
            t[c] = nt;
        }
        if worst <= 1e-9 * big {
            break;
        }
    }
    (0..n)
        .map(|c| {
            let w = s.w_v[c];
            t[c].map(|x| if w > 1e-12 { x / w } else { 0.0 })
        })
        .collect()
}

/// Solves one month's budget, starting from `start` (vapour, cloud) when given.
pub fn solve_month(
    mesh: &Mesh,
    k: &Constants,
    mo: &Month,
    start: Option<(&[f64], &[f64])>,
) -> Solved {
    let n = mesh.cells();
    let Transport {
        diag,
        inflow,
        order,
    } = transport(mesh, k, mo);
    let inv_f = 1.0 / k.tau_f_s;
    let (mut wv, mut wc) = match start {
        Some((a, b)) if a.len() == n && b.len() == n => (a.to_vec(), b.to_vec()),
        _ => (vec![0.0; n], vec![0.0; n]),
    };
    let mut sweeps = 0u32;
    for it in 0..4000u32 {
        sweeps += 1;
        let mut worst = 0.0f64;
        let mut big = 1e-30f64;
        let fwd = it % 2 == 0;
        for idx in 0..n {
            let c = if fwd { order[idx] } else { order[n - 1 - idx] };
            let a = mesh.area_m2[c];
            let (mut rv, mut rc) = (a * mo.e0[c], 0.0);
            for &(o, coef) in &inflow[c] {
                rv += coef * wv[o];
                rc += coef * wc[o];
            }
            let kc = mo.k_c[c];
            // Vapour: d1 W_v + A C(W_v) − A γ W_c = rv; cloud: d2 W_c − A C(W_v) = rc, with the
            // condensation C(W) = k_c W e^(b (W/W_ref − 1)). The cloud row gives W_c; what is left
            // is g(W_v) = d1 W_v + A C(W_v)(1 − Aγ/d2) − Aγ rc/d2 − rv = 0, increasing and convex
            // in W_v (Aγ < d2), so Newton's steps converge from any start after the first.
            let d1 = diag[c] + a * (mo.e1[c] + mo.k_s[c]);
            let gam = k.beta * kc + mo.eps[c] * inv_f;
            let d2 = diag[c] + a * (k.beta * kc + inv_f);
            let wr = mo.w_ref[c].max(1e-9);
            let cond = |x: f64| {
                let e = (k.rh_gain * (x / wr - 1.0)).min(30.0).exp();
                (kc * x * e, kc * e * (1.0 + k.rh_gain * x / wr))
            };
            let keep = 1.0 - a * gam / d2;
            let rhs = rv + a * gam * rc / d2;
            let mut x = wv[c];
            for _ in 0..30 {
                let (cx, dcx) = cond(x);
                let g = d1 * x + a * cx * keep - rhs;
                let dg = d1 + a * dcx * keep;
                let step = g / dg.max(1e-300);
                let nx = (x - step).max(0.0);
                let done = (nx - x).abs() <= 1e-12 * nx.max(1e-6);
                x = nx;
                if done {
                    break;
                }
            }
            let nv = x;
            let nc = ((rc + a * cond(nv).0) / d2).max(0.0);
            worst = worst.max((nv - wv[c]).abs()).max((nc - wc[c]).abs());
            big = big.max(nv);
            wv[c] = nv;
            wc[c] = nc;
        }
        if worst <= 1e-9 * big {
            break;
        }
    }
    let p: Vec<f64> = wc.iter().map(|x| x * inv_f).collect();
    let e_sea: Vec<f64> = (0..n).map(|c| mo.e0[c] - mo.e1[c] * wv[c]).collect();
    let e_land: Vec<f64> = (0..n).map(|c| mo.eps[c] * p[c]).collect();
    let (mut src, mut snk) = (0.0, 0.0);
    for c in 0..n {
        let a = mesh.area_m2[c];
        src += a * (e_sea[c] + e_land[c]);
        snk += a * (p[c] + mo.k_s[c] * wv[c]);
    }
    Solved {
        closure: (src - snk).abs() / src.abs().max(1e-300),
        w_v: wv,
        w_c: wc,
        p,
        e_sea,
        e_land,
        sweeps,
    }
}

/// Solves the twelve months in parallel, each from its `start` when given.
pub fn solve_year(
    mesh: &Mesh,
    k: &Constants,
    months: &[Month],
    start: Option<&[Solved]>,
) -> Vec<Solved> {
    months
        .par_iter()
        .enumerate()
        .map(|(m, mo)| {
            let s = start
                .and_then(|s| s.get(m))
                .map(|s| (s.w_v.as_slice(), s.w_c.as_slice()));
            solve_month(mesh, k, mo, s)
        })
        .collect()
}

/// The minimum condensation time, s (a column far above its reference water rains within the
/// hour; keeps τ_c positive where P_ref/W_ref is large).
const TAU_C_MIN_S: f64 = 3600.0; // game-choice: numerical floor, not a physical time
/// The land's share of its rain that evaporates again where no rain is known yet (the first
/// snapshot's first pass), about Earth's land ET/P.
const EPS_START: f64 = 0.6; // ledger: CLM-197
/// Fu's potential evaporation stand-in, mm/yr per °C of annual mean (the surface stage's).
const PET_MM_PER_C: f64 = 60.0; // ledger: HYD-029
const YEAR_S: f64 = 365.25 * 86_400.0;

/// What the upwind rain needs of the snapshot's coarse climate (rows north first on the coarse
/// grid; every per-cell slice has the grid's length).
pub struct ClimateIn<'a> {
    pub grid: &'a Grid,
    /// Land fraction and the land's lowland height, m.
    pub land: &'a [f64],
    pub z_low: &'a [f64],
    /// Monthly land and sea tiles, °C, and sea ice, 0–1.
    pub t_land: &'a [[f64; 12]],
    pub t_ocean: &'a [[f64; 12]],
    pub sea_ice: &'a [[f64; 12]],
    /// The circulation's rain at the reference water, kg/m²/s, per cell and month (the caller's
    /// belts, subsidence and stability, averaged over the cell).
    pub p_ref: &'a [[f64; 12]],
    /// The sea's position across its basin (`belts::basin_position`, mean over the cell's sea)
    /// and its sea surface's departure from its row's sea, °C: what the vapour carries inland
    /// as tracers of where it evaporated.
    pub basin: &'a [f64],
    pub dsst: &'a [f64],
    /// The lapse rate, °C/m.
    pub lapse: f64,
    /// The mountains' rain at the reference water, kg/m²/s per cell and month (the snapshot's
    /// second pass), if any.
    pub sink_ref: Option<&'a [[f64; 12]]>,
    /// The previous solution (warm start) and the cells' last annual rain, mm/yr (for the
    /// land's recycling; empty: not known).
    pub start: Option<&'a Rain>,
    pub annual_prev: &'a [f64],
}

/// The upwind rain of a snapshot.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rain {
    pub months: Vec<Solved>,
    /// The reference water of every cell and month, kg/m², and the rain rate there, kg/m²/s.
    pub w_ref: Vec<[f64; 12]>,
    pub p_ref: Vec<[f64; 12]>,
    /// The winds, m/s (east, north).
    pub u: Vec<[f64; 12]>,
    pub v: Vec<[f64; 12]>,
    /// Where each cell lies under the subtropical highs ([`flank_position`], the same every
    /// month), and the sea-surface departure the vapour carries (a mixing ratio: the sea's value
    /// times the share of the vapour that evaporated from the sea), per cell and month.
    pub basin_air: Vec<[f64; 12]>,
    pub dsst_air: Vec<[f64; 12]>,
    /// Annual rain per cell, mm/yr, and the land's recycled share ε used.
    pub annual_mm: Vec<f64>,
    pub eps: Vec<f64>,
    /// The worst month's closure and the most sweeps a month took.
    pub closure: f64,
    pub sweeps: u32,
}

impl Rain {
    /// Rain rate (kg/m²/s) and moisture (W_v/W_ref) of coarse cell `c` in month `m`.
    pub fn at(&self, c: usize, m: usize) -> (f64, f64) {
        let s = &self.months[m];
        (s.p[c], s.w_v[c] / self.w_ref[c][m].max(1e-9))
    }

    /// How much of the circulation's rain coarse cell `c` gets in month `m` (its rain over the
    /// rain at its reference water) and its moisture W_v/W_ref.
    pub fn dry(&self, c: usize, m: usize) -> (f64, f64) {
        let s = &self.months[m];
        (
            s.p[c] / self.p_ref[c][m].max(1e-15),
            s.w_v[c] / self.w_ref[c][m].max(1e-9),
        )
    }
}

/// The upwind rain of climate section `p` on a snapshot's coarse climate.
#[allow(clippy::needless_range_loop)]
pub fn rain(
    p: &super::ClimateParams,
    mp: &super::moisture::MoistureParams,
    inp: &ClimateIn<'_>,
) -> Rain {
    use super::moisture::{RHO_AIR, W_SAT_27, W_SAT_PER_K, W_SAT_SCALE_M, q_sat};
    let g = inp.grid;
    let n = g.w * g.h;
    let mesh = Mesh::from_grid(g, mp.radius_m);
    let (u, v, _itcz) = super::moisture::winds(mp, g, inp.land, inp.t_land, inp.t_ocean);
    let sea: Vec<f64> = inp.land.iter().map(|x| 1.0 - x.clamp(0.0, 1.0)).collect();
    let k = Constants {
        k_q: p.rain_diffusivity_m2_s,
        tau_f_s: TAU_F_S,
        beta: BETA,
        // The rain's e-folding in the column's relative humidity r = r_ref W_v/W_ref (Bretherton,
        // Peters & Back's form): e^(a (r − r_ref)) = e^(a r_ref (W_v/W_ref − 1)).
        rh_gain: p.rain_rh_slope * mp.r_ref, // ledger: CLM-200
    };
    let f: Vec<f64> = inp.land.iter().map(|x| x.clamp(0.0, 1.0)).collect();
    let mut w_ref = vec![[0.0f64; 12]; n];
    let p_ref: Vec<[f64; 12]> = inp.p_ref.to_vec();
    for c in 0..n {
        let hf = 1.0 - f[c] + f[c] * (-inp.z_low[c].max(0.0) / W_SAT_SCALE_M).exp();
        for m in 0..12 {
            let tbar = f[c] * inp.t_land[c][m] + (1.0 - f[c]) * inp.t_ocean[c][m];
            w_ref[c][m] = mp.r_ref * W_SAT_27 * (W_SAT_PER_K * (tbar - 27.0)).exp() * hf;
        }
    }
    // The pieces that do not depend on the land's recycling.
    let base: Vec<Month> = (0..12)
        .map(|m| {
            let mut mo = Month {
                u: (0..n).map(|c| p.rain_wind_share * u[c][m]).collect(),
                v: (0..n).map(|c| p.rain_wind_share * v[c][m]).collect(),
                ..Month::default()
            };
            for c in 0..n {
                let wr = w_ref[c][m].max(1e-6);
                let speed = u[c][m].hypot(v[c][m]);
                let open = (1.0 - f[c]) * (1.0 - inp.sea_ice[c][m].clamp(0.0, 1.0));
                let e0 = open * RHO_AIR * mp.c_e * (speed + mp.u_t_sea) * q_sat(inp.t_ocean[c][m]);
                mo.e0.push(e0);
                mo.e1.push(e0 * p.sea_surface_rh / wr);
                mo.w_ref.push(wr);
                let tau_p = wr / p_ref[c][m].max(1e-12);
                mo.k_c
                    .push(1.0 / (tau_p - k.beta * k.tau_f_s).max(TAU_C_MIN_S));
                mo.k_s
                    .push(inp.sink_ref.map_or(0.0, |s| s[c][m].max(0.0) / wr));
            }
            mo
        })
        .collect();
    // The land's potential evaporation (the surface stage's stand-in on the land tile's annual
    // mean at its lowland height) and the recycled share from Fu's curve.
    let pet: Vec<f64> = (0..n)
        .map(|c| {
            let t = inp.t_land[c].iter().sum::<f64>() / 12.0 - inp.lapse * inp.z_low[c].max(0.0);
            (PET_MM_PER_C * t).max(0.0)
        })
        .collect();
    let eps_of = |c: usize, annual: &[f64]| -> f64 {
        if f[c] <= 0.0 {
            return 0.0;
        }
        let share = match annual.get(c) {
            // Fu's Budyko curve (ledger: HYD-034).
            Some(&pr) if pr > 0.0 => crate::surface::climate::fu_et(pr, pet[c]) / pr,
            _ => EPS_START,
        };
        f[c] * p.rain_recycling * share.clamp(0.0, 0.95)
    };
    let passes = if inp.annual_prev.len() == n { 1 } else { 3 };
    let mut annual = inp.annual_prev.to_vec();
    let mut eps: Vec<f64> = (0..n).map(|c| eps_of(c, &annual)).collect();
    let mut months: Vec<Solved> = inp.start.map(|r| r.months.clone()).unwrap_or_default();
    let mut year: Vec<Month> = Vec::new();
    for _ in 0..passes {
        year = base
            .iter()
            .map(|mo| Month {
                eps: eps.clone(),
                ..mo.clone()
            })
            .collect();
        let st = (months.len() == 12).then_some(months.as_slice());
        months = solve_year(&mesh, &k, &year, st);
        annual = (0..n)
            .map(|c| months.iter().map(|s| s.p[c]).sum::<f64>() * YEAR_S / 12.0)
            .collect();
        eps = (0..n).map(|c| eps_of(c, &annual)).collect();
    }
    // The sea surface the month's vapour evaporated from (its departure from its row, carried
    // as a tracer; the land's recycled share carries none) sets the rain's stability over land.
    let src: Vec<[f64; 1]> = (0..n).map(|c| [inp.dsst[c]]).collect();
    let carried: Vec<Vec<[f64; 1]>> = year
        .par_iter()
        .zip(&months)
        .map(|(mo, s)| tracers(&mesh, &k, mo, s, &src))
        .collect();
    let dsst_air = (0..n).map(|c| std::array::from_fn(|m| carried[m][c][0])).collect();
    // Which side of the highs a place lies under is the place's (`flank_position`).
    let flank = flank_position(&mesh, inp.basin, &sea);
    let basin_air = (0..n).map(|c| [flank[c]; 12]).collect();
    Rain {
        basin_air,
        dsst_air,
        closure: months.iter().map(|s| s.closure).fold(0.0, f64::max),
        sweeps: months.iter().map(|s| s.sweeps).max().unwrap_or(0),
        months,
        w_ref,
        p_ref,
        u,
        v,
        annual_mm: annual,
        eps,
    }
}

/// Cloud water's fallout time, s: LFPM's real-world example takes the fallout length L_f =
/// 25 km (Hergarten & Robl 2022, section 7), here at 10 m/s.
pub const TAU_F_S: f64 = 2500.0; // ledger: CLM-193
/// Cloud water's re-evaporation against condensation (LFPM's β): the real-world example's lengths
/// L_c = L_f = 25 km and long-range length L_l = 500 km (Hergarten & Robl 2022, section 7). The
/// two decay lengths of the linear system are the eigenvalues of its matrix, so L_l L_s = L_c L_f
/// (its equation 27) and 1/L_l + 1/L_s = (1 + β)/L_c + 1/L_f (its trace): L_s = 1.25 km and
/// β = 18.05.
pub const BETA: f64 = 18.05; // ledger: CLM-194

#[cfg(test)]
mod tests {
    use super::*;

    /// A ring of `n` cells, each 100 km wide, as a mesh (x wraps; one row).
    fn ring(n: usize) -> Mesh {
        let d = 100_000.0;
        Mesh {
            area_m2: vec![d * d; n],
            faces: (0..n)
                .map(|c| Face {
                    a: c,
                    b: (c + 1) % n,
                    len_m: d,
                    dist_m: d,
                    n_a: [1.0, 0.0],
                    n_b: [1.0, 0.0],
                })
                .collect(),
        }
    }

    fn consts() -> Constants {
        Constants {
            k_q: 0.0,
            tau_f_s: 2500.0,
            beta: 18.0,
            rh_gain: 0.0,
        }
    }

    /// Sea on cells [0, sea), land beyond; wind `u` east; a uniform condensation rate.
    fn month(n: usize, sea: usize, u: f64, eps: f64) -> Month {
        let is_sea = |c: usize| c < sea;
        Month {
            u: vec![u; n],
            v: vec![0.0; n],
            e0: (0..n).map(|c| if is_sea(c) { 5e-5 } else { 0.0 }).collect(),
            e1: (0..n)
                .map(|c| if is_sea(c) { 5e-5 / 40.0 } else { 0.0 })
                .collect(),
            k_c: vec![1.0 / (8.0 * 86_400.0); n],
            eps: (0..n).map(|c| if is_sea(c) { 0.0 } else { eps }).collect(),
            k_s: vec![0.0; n],
            w_ref: vec![40.0; n],
        }
    }

    #[test]
    fn air_dries_downwind_over_land_and_the_water_closes() {
        let n = 200;
        let mesh = ring(n);
        let s = solve_month(&mesh, &consts(), &month(n, 60, 8.0, 0.0), None);
        assert!(s.closure < 1e-6, "closure {}", s.closure);
        // Eastward wind: the rain falls off from the west coast (cell 60) inland.
        let p = |c: usize| s.p[c];
        assert!(p(62) > p(100) && p(100) > p(150) && p(150) > p(199));
        // The decay length is about |V|·τ_P: 8 m/s × 8 days ≈ 5,500 km = 55 cells.
        let ratio = p(160) / p(105);
        let expect = (-55.0f64 / 55.3).exp();
        assert!(
            (ratio / expect - 1.0).abs() < 0.25,
            "ratio {ratio} vs {expect}"
        );
        // Westward wind: the east coast (cell 199) is the wet one now.
        let w = solve_month(&mesh, &consts(), &month(n, 60, -8.0, 0.0), None);
        assert!(w.p[197] > w.p[150] && w.p[150] > w.p[70]);
        assert!(w.closure < 1e-6);
    }

    #[test]
    fn land_that_recycles_its_rain_carries_moisture_further() {
        let n = 200;
        let mesh = ring(n);
        let dry = solve_month(&mesh, &consts(), &month(n, 60, 8.0, 0.0), None);
        let wet = solve_month(&mesh, &consts(), &month(n, 60, 8.0, 0.6), None);
        assert!(
            wet.p[180] > 1.5 * dry.p[180],
            "{} {}",
            wet.p[180],
            dry.p[180]
        );
        assert!(wet.closure < 1e-6);
        // The land's evaporation is the share it recycles.
        assert!((wet.e_land[150] / wet.p[150] - 0.6).abs() < 1e-9);
    }

    #[test]
    fn a_mountain_sink_shadows_the_lee_and_diffusion_keeps_the_water() {
        let n = 200;
        let mesh = ring(n);
        let mut mo = month(n, 60, 8.0, 0.3);
        for c in 80..84 {
            mo.k_s[c] = 1.0 / 86_400.0;
        }
        let k = Constants {
            k_q: 2.0e5,
            ..consts()
        };
        let base = solve_month(&mesh, &k, &month(n, 60, 8.0, 0.3), None);
        let s = solve_month(&mesh, &k, &mo, None);
        assert!(s.p[100] < 0.7 * base.p[100], "{} {}", s.p[100], base.p[100]);
        assert!(s.closure < 1e-6, "closure {}", s.closure);
        assert!(s.w_v.iter().chain(&s.w_c).all(|&x| x >= 0.0));
    }

    #[test]
    fn dry_columns_rain_less_of_their_water_and_still_close() {
        let n = 200;
        let mesh = ring(n);
        let k = Constants {
            rh_gain: 8.0,
            ..consts()
        };
        let lin = solve_month(&mesh, &consts(), &month(n, 60, 8.0, 0.3), None);
        let s = solve_month(&mesh, &k, &month(n, 60, 8.0, 0.3), None);
        assert!(s.closure < 1e-6, "closure {}", s.closure);
        // Rain per unit vapour falls as the air dries inland ...
        let eff = |c: usize| s.p[c] / s.w_v[c];
        assert!(eff(190) < 0.5 * eff(62), "{} {}", eff(190), eff(62));
        // ... so the vapour reaches further than under linear condensation.
        assert!(s.w_v[190] > lin.w_v[190], "{} {}", s.w_v[190], lin.w_v[190]);
    }

    #[test]
    fn the_sea_marks_follow_the_vapour_and_thin_with_the_lands_own() {
        let n = 200;
        let mesh = ring(n);
        let k = Constants {
            k_q: 2.0e5,
            rh_gain: 4.0,
            ..consts()
        };
        // Without recycling all vapour is the sea's: a mark of 1 everywhere stays 1.
        let mo = month(n, 60, 8.0, 0.0);
        let s = solve_month(&mesh, &k, &mo, None);
        let ones = vec![[1.0f64]; n];
        let t = tracers(&mesh, &k, &mo, &s, &ones);
        for c in 0..n {
            assert!((t[c][0] - 1.0).abs() < 1e-6, "cell {c}: {}", t[c][0]);
        }
        // With recycling the sea's share falls inland, downwind of the coast.
        let mo = month(n, 60, 8.0, 0.6);
        let s = solve_month(&mesh, &k, &mo, None);
        let t = tracers(&mesh, &k, &mo, &s, &ones);
        // Over the sea the share grows downwind as the sea's evaporation replaces the land air
        // the wind brought (the ring's land lies upwind of cell 0).
        assert!(
            t[5][0] < t[30][0] && t[30][0] < t[55][0] && t[55][0] > 0.5,
            "{:?}",
            [t[5][0], t[30][0], t[55][0]]
        );
        assert!(t[70][0] > t[120][0] && t[120][0] > t[190][0], "{:?}", [t[70][0], t[120][0], t[190][0]]);
        assert!(t[190][0] < 0.8, "{}", t[190][0]);
        // A mark set by the sea's position (here the sea's western half 1, eastern half 0)
        // arrives from the sea that lies upwind.
        // (As a share of the sea's vapour there: the ones tracer `t`.)
        let half: Vec<[f64; 1]> = (0..n).map(|c| [if c < 30 { 1.0 } else { 0.0 }]).collect();
        let h = tracers(&mesh, &k, &mo, &s, &half);
        let share = |c: usize| h[c][0] / t[c][0];
        // Downwind of the western half the share of its mark falls as the eastern half's
        // evaporation adds vapour without it.
        assert!(
            share(20) > share(45) && share(45) > share(62) && share(20) > share(62) + 0.1,
            "{:?}",
            [share(20), share(45), share(62)]
        );
    }

    #[test]
    fn smoothing_keeps_the_water_and_spreads_over_its_length() {
        let n = 200;
        let mesh = ring(n);
        let x: Vec<f64> = (0..n).map(|c| if c == 100 { 1.0 } else { 0.0 }).collect();
        let y = mesh.smooth(&x, 300_000.0);
        let (sx, sy): (f64, f64) = (x.iter().sum(), y.iter().sum());
        assert!((sx - sy).abs() < 1e-6, "{sx} {sy}");
        // Symmetric, decaying about e^(−r/l) (l = 3 cells).
        assert!((y[97] - y[103]).abs() < 1e-9);
        let r = y[106] / y[103];
        assert!((r / (-1.0f64).exp() - 1.0).abs() < 0.25, "{r}");
        let c = mesh.smooth(&vec![2.0; n], 300_000.0);
        assert!(c.iter().all(|v| (v - 2.0).abs() < 1e-6));
    }

    #[test]
    fn land_takes_the_flank_of_the_basin_it_borders() {
        // Land on cells 0..40 of a ring, a basin on 40..200 (west side −1, east side +1): the
        // land's west coast (cell 0) borders the basin's eastern side, its east coast (39) the
        // western side; the middle, 2,000 km from both, keeps little of either.
        let n = 200;
        let mesh = ring(n);
        let sea: Vec<f64> = (0..n).map(|c| if c < 40 { 0.0 } else { 1.0 }).collect();
        let basin: Vec<f64> = (0..n)
            .map(|c| if c < 40 { 0.0 } else { (c as f64 - 120.0) / 80.0 })
            .collect();
        let a = flank_position(&mesh, &basin, &sea);
        assert!(a[1] > 0.6 && a[38] < -0.6, "{} {}", a[1], a[38]);
        assert!(a[5] > a[15] && a[15] > a[20], "{:?}", [a[5], a[15], a[20]]);
        assert!(a[20].abs() < 0.2, "{}", a[20]);
        // Over the open sea it is the basin's own position.
        assert!((a[120] - basin[120]).abs() < 0.05 && a[190] > 0.8, "{} {}", a[120], a[190]);
    }

    #[test]
    fn the_flow_order_puts_every_cell_after_those_upwind_of_it() {
        // A chain 0 → 1 → … → 9 given in shuffled face order: the order is the chain's.
        let faces: Vec<Face> = [3usize, 7, 0, 5, 1, 8, 2, 6, 4]
            .iter()
            .map(|&a| Face {
                a,
                b: a + 1,
                len_m: 1.0,
                dist_m: 1.0,
                n_a: [1.0, 0.0],
                n_b: [1.0, 0.0],
            })
            .collect();
        let flux = vec![1.0; faces.len()];
        let o = flow_order(10, &faces, &flux);
        assert_eq!(o, (0..10).collect::<Vec<_>>());
    }
}
