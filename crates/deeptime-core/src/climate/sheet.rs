// SPDX-License-Identifier: LGPL-3.0-or-later
//! Ice-sheet dynamics on a coarse grid (decisions log 2026-09-30, "Yes, add ice-sheet dynamics":
//! elevation feedback and simple flow, so glacial cycles reach Earth-like size).
//!
//! The shallow-ice approximation (Hutter 1983, *Theoretical Glaciology*; Mahaffy 1976, "A
//! three-dimensional numerical model of ice sheets: tests on the Barnes Ice Cap, Northwest
//! Territories", JGR 81: 1059–1066, whose staggered explicit scheme this follows, implemented from
//! the paper): ∂H/∂t = b − ∇·q, q = −D ∇s, D = Γ H^(n+2) |∇s|^(n−1), Γ = 2A(ρg)^n/(n+2), n = 3,
//! A = `ice_flow_a` (10⁻¹⁶ Pa⁻³ yr⁻¹, EISMINT: Huybrechts et al. 1996, Ann. Glaciol. 23: 1–12), on
//! the climate's L1 grid coarsened to about `ice_grid_cells` columns (a few hundred km per cell at
//! Earth size; x wraps, no flux across the poles), time-stepped at the explicit diffusion limit.
//!
//! - **Elevation feedback.** The mass balance follows the ice *surface* s = bed − depression + H
//!   against the equilibrium line: b = β_a·(s − ELA), at most the accumulation the precipitation
//!   allows (`ice_accumulation_max_m_yr` × P/1 m, snow-limited in dry climates), above the line,
//!   and b = β_m·(s − ELA) below it (Oerlemans 1981, "Some basic experiments with a vertically
//!   integrated ice sheet model", Tellus 33: 1–11: mass-balance gradients of a few mm/yr per m),
//!   so a growing sheet climbs into its own accumulation zone and flows out into ablation below
//!   the line.
//! - **Isostasy.** The bed sinks under the ice toward ρ_i/ρ_m of its thickness with an e-folding
//!   time `ice_isostasy_kyr` (3 kyr; the elastic-lithosphere/relaxed-asthenosphere model of
//!   Le Meur & Huybrechts 1996, Ann. Glaciol. 23: 309–317, without the flexure), which lowers the
//!   surface and lets a big sheet collapse when summers warm — the 100-kyr sawtooth of Oerlemans
//!   1980 ("Model experiments on the 100,000-yr glacial cycle", Nature 287: 430–432) and Pollard
//!   1982 ("A simple ice sheet model yields realistic 100 kyr glacial cycles", Nature 296: 334–338).
//! - **Calving.** Ice not thick enough to stay grounded where the depressed bed lies below the sea
//!   (H < ρ_w/ρ_i · depth) floats away, and a grounded margin whose depressed bed lies below the
//!   sea loses `ice_calving_m_yr` (20 m/yr, Pollard 1982's calving into the sea or the proglacial
//!   lake) — without it a sheet grown under a lowered line outlives the line's return (on Earth's
//!   relief 19 % of the land stayed under ice at the present line after a 1-km lowering).

use serde::{Deserialize, Serialize};

use super::ice::{G, RHO_ICE, RHO_SEA};

/// Density of the mantle, kg/m³ (isostasy).
pub const RHO_MANTLE: f64 = 3300.0; // ledger: GLA-071
/// Glen's exponent (the flux below is written out for n = 3: D = Γ·H⁵·|∇s|²).
pub const GLEN_N: f64 = 3.0; // ledger: GLA-067
/// The shortest explicit step the flow is allowed to ask for, years (see `State::step`).
pub const MIN_STEP_YEARS: f64 = 10.0; // ledger: GLA-073

/// The coarse grid (shared with the per-cell climate, `coarse.rs`).
pub use super::coarse::Grid;

/// The sheet's parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    /// Glen's A, Pa⁻³ yr⁻¹.
    pub a: f64,
    /// Mass-balance gradients above and below the equilibrium line, (m ice/yr) per m.
    pub beta_acc: f64,
    pub beta_abl: f64,
    /// Largest accumulation at 1 m/yr of precipitation, m ice/yr.
    pub acc_max: f64,
    /// Isostatic e-folding time, years.
    pub isostasy_years: f64,
    /// Thickest ice, m.
    pub cap_m: f64,
    /// Calving at a margin standing in water, m ice/yr.
    pub calving: f64,
}

impl Params {
    pub fn from(p: &super::ClimateParams) -> Params {
        Params {
            a: p.ice_flow_a,
            beta_acc: p.ice_accumulation_gradient,
            beta_abl: p.ice_ablation_gradient,
            acc_max: p.ice_accumulation_max_m_yr,
            isostasy_years: p.ice_isostasy_kyr * 1000.0,
            cap_m: p.ice_max_thickness_m,
            calving: p.ice_calving_m_yr,
        }
    }
}

/// The sheet's state (coarse): thickness and bed depression, m.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub thick: Vec<f32>,
    pub depress: Vec<f32>,
}

/// What drives the sheet (coarse, all m relative to sea level unless stated).
pub struct Forcing<'a> {
    /// Bed elevation without the ice's depression.
    pub bed: &'a [f32],
    /// Equilibrium-line altitude.
    pub ela: &'a [f32],
    /// Precipitation, m/yr.
    pub precip: &'a [f32],
}

impl State {
    /// A sheet of `thick` in isostatic equilibrium.
    pub fn settled(thick: Vec<f32>) -> State {
        let depress = thick
            .iter()
            .map(|&h| (f64::from(h) * RHO_ICE / RHO_MANTLE) as f32)
            .collect();
        State { thick, depress }
    }

    /// Grounded ice volume, m³ (true areas).
    pub fn volume_m3(&self, g: &Grid) -> f64 {
        (0..g.h)
            .map(|j| {
                let row: f64 = self.thick[j * g.w..(j + 1) * g.w]
                    .iter()
                    .map(|&x| f64::from(x))
                    .sum();
                row * g.dx[j] * g.dy
            })
            .sum()
    }

    /// Advances the sheet by `years`. Returns the number of sub-steps taken. Serial (and so
    /// deterministic) over the rows that hold ice, border it or can accumulate it; the rest of the
    /// grid is skipped.
    pub fn step(&mut self, g: &Grid, f: &Forcing<'_>, p: &Params, years: f64) -> u32 {
        let (w, h) = (g.w, g.h);
        let n = w * h;
        if self.thick.len() != n || self.depress.len() != n {
            *self = State {
                thick: vec![0.0; n],
                depress: vec![0.0; n],
            };
        }
        let gamma = 2.0 * p.a * (RHO_ICE * G).powf(GLEN_N) / (GLEN_N + 2.0);
        let float = RHO_SEA / RHO_ICE;
        let min_d2 = g.dx.iter().map(|d| d * d).fold(g.dy * g.dy, f64::min);
        // The fastest ice (thick, steep margins) is held to the diffusivity an explicit step of
        // `MIN_STEP_YEARS` can carry: the few faces above it would otherwise set every cell's time
        // step (a year or less) for a flux the mass balance bounds anyway.
        let d_cap = 0.2 * min_d2 / MIN_STEP_YEARS;
        // Rows whose ground can grow ice at all (above the line somewhere): always stepped.
        let can_grow: Vec<bool> = (0..h)
            .map(|j| (0..w).any(|i| f.bed[j * w + i] > f.ela[j * w + i] && f.bed[j * w + i] > 0.0))
            .collect();
        let mut thick: Vec<f64> = self.thick.iter().map(|&x| f64::from(x)).collect();
        let mut depress: Vec<f64> = self.depress.iter().map(|&x| f64::from(x)).collect();
        let mut s = vec![0.0f64; n];
        let mut dfx = vec![0.0f64; n];
        let mut dfy = vec![0.0f64; n];
        let at = |i: i64, j: usize| j * w + i.rem_euclid(w as i64) as usize;
        let mut left = years;
        let mut steps = 0u32;
        while left > 1.0e-6 && steps < 200_000 {
            // Rows to step: those with ice, their neighbours, and those that can grow ice.
            let has: Vec<bool> = (0..h)
                .map(|j| thick[j * w..(j + 1) * w].iter().any(|&x| x > 0.0))
                .collect();
            let active: Vec<bool> = (0..h)
                .map(|j| {
                    can_grow[j] || has[j] || (j > 0 && has[j - 1]) || (j + 1 < h && has[j + 1])
                })
                .collect();
            if !active.iter().any(|&a| a) {
                break;
            }
            // The surface: the ice's top, else the ground, else the sea.
            for c in 0..n {
                let bed = f64::from(f.bed[c]) - depress[c];
                s[c] = if thick[c] > 0.0 {
                    bed + thick[c]
                } else {
                    bed.max(0.0)
                };
            }
            // Diffusivity on the x faces (i + ½, j) and y faces (i, j + ½).
            let mut dmax = 0.0f64;
            for j in 0..h {
                if !active[j] {
                    // No ice in or next to it: no flux across its faces.
                    dfx[j * w..(j + 1) * w].iter_mut().for_each(|x| *x = 0.0);
                    dfy[j * w..(j + 1) * w].iter_mut().for_each(|x| *x = 0.0);
                    continue;
                }
                for i in 0..w as i64 {
                    let c = at(i, j);
                    let e = at(i + 1, j);
                    let hf = 0.5 * (thick[c] + thick[e]);
                    dfx[c] = if hf > 0.0 {
                        let dsdx = (s[e] - s[c]) / g.dx[j];
                        let (jn, js) = ((j + 1).min(h - 1), j.saturating_sub(1));
                        let dsdy = if jn == js {
                            0.0
                        } else {
                            (s[at(i, jn)] + s[at(i + 1, jn)] - s[at(i, js)] - s[at(i + 1, js)])
                                / (2.0 * (jn - js) as f64 * g.dy)
                        };
                        (gamma * hf.powi(5) * (dsdx * dsdx + dsdy * dsdy)).min(d_cap)
                    } else {
                        0.0
                    };
                    dfy[c] = if j + 1 < h {
                        let nn = at(i, j + 1);
                        let hf = 0.5 * (thick[c] + thick[nn]);
                        if hf > 0.0 {
                            let dsdy = (s[nn] - s[c]) / g.dy;
                            let dxm = 0.5 * (g.dx[j] + g.dx[j + 1]);
                            let dsdx = (s[at(i + 1, j)] + s[at(i + 1, j + 1)]
                                - s[at(i - 1, j)]
                                - s[at(i - 1, j + 1)])
                                / (4.0 * dxm);
                            (gamma * hf.powi(5) * (dsdx * dsdx + dsdy * dsdy)).min(d_cap)
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };
                    dmax = dmax.max(dfx[c]).max(dfy[c]);
                }
            }
            // The elevation feedback (dH/dt ≈ β·H near the line) stays explicit-stable too.
            let dt = if dmax > 0.0 {
                0.2 * min_d2 / dmax
            } else {
                left
            }
            .min(300.0)
            .min(left);
            let relax = 1.0 - (-dt / p.isostasy_years.max(1.0)).exp();
            // The update: flux divergence (face fluxes −D ∂s/∂n per unit length) and the balance.
            let qx = |c: usize, j: usize, i: i64| -> f64 {
                -dfx[c] * (s[at(i + 1, j)] - s[c]) / g.dx[j]
            };
            let qy = |c: usize, j: usize, i: i64| -> f64 {
                if j + 1 >= h {
                    0.0
                } else {
                    -dfy[c] * (s[at(i, j + 1)] - s[c]) / g.dy
                }
            };
            let mut new = thick.clone();
            for (j, &on) in active.iter().enumerate().take(h) {
                if !on {
                    continue;
                }
                let len_n = if j + 1 < h {
                    0.5 * (g.dx[j] + g.dx[j + 1])
                } else {
                    0.0
                };
                let len_s = if j > 0 {
                    0.5 * (g.dx[j] + g.dx[j - 1])
                } else {
                    0.0
                };
                for i in 0..w as i64 {
                    let c = at(i, j);
                    let q_s = if j > 0 {
                        qy(at(i, j - 1), j - 1, i)
                    } else {
                        0.0
                    };
                    let div = (qx(c, j, i) - qx(at(i - 1, j), j, i - 1)) / g.dx[j]
                        + (qy(c, j, i) * len_n - q_s * len_s) / (g.dx[j] * g.dy);
                    let above = s[c] - f64::from(f.ela[c]);
                    let acc = p.acc_max * f64::from(f.precip[c]).max(0.0);
                    let b = if above >= 0.0 {
                        (p.beta_acc * above).min(acc)
                    } else {
                        p.beta_abl * above
                    };
                    let mut hh = (thick[c] + dt * (b - div)).clamp(0.0, p.cap_m);
                    // Calving: ice that would float where the depressed bed lies below the sea.
                    let bed = f64::from(f.bed[c]) - depress[c];
                    if bed < 0.0 {
                        if hh < float * -bed {
                            hh = 0.0;
                        } else if p.calving > 0.0 && hh > 0.0 {
                            // A grounded margin standing in water (the sea, or the lake that fills
                            // the bed the ice pressed down) calves (Pollard 1982).
                            let open = thick[at(i - 1, j)] <= 0.0
                                || thick[at(i + 1, j)] <= 0.0
                                || (j > 0 && thick[at(i, j - 1)] <= 0.0)
                                || (j + 1 < h && thick[at(i, j + 1)] <= 0.0);
                            if open {
                                hh = (hh - dt * p.calving).max(0.0);
                            }
                        }
                    }
                    new[c] = hh;
                }
            }
            thick = new;
            for c in 0..n {
                depress[c] += (thick[c] * RHO_ICE / RHO_MANTLE - depress[c]) * relax;
            }
            left -= dt;
            steps += 1;
        }
        for c in 0..n {
            self.thick[c] = thick[c] as f32;
            self.depress[c] = depress[c] as f32;
        }
        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params() -> Params {
        Params {
            calving: 0.0,
            a: 1.0e-16,
            beta_acc: 0.005,
            beta_abl: 0.01,
            acc_max: 0.5,
            isostasy_years: 3000.0,
            cap_m: 5000.0,
        }
    }

    /// A flat square continent 2,000 km across at 200 m, the sea around it, 100-km cells.
    fn continent(ela: f32) -> (Grid, Vec<f32>, Vec<f32>, Vec<f32>) {
        let (fw, fh) = (64, 32);
        let g = Grid::new(fw, fh, &[1.0e5; 32], 1.0e5, 64);
        let bed: Vec<f32> = (0..fw * fh)
            .map(|c| {
                let (i, j) = ((c % fw) as i64 - 32, (c / fw) as i64 - 16);
                if i.abs() < 10 && j.abs() < 10 {
                    200.0
                } else {
                    -3000.0
                }
            })
            .collect();
        (g, bed, vec![ela; fw * fh], vec![1.0; fw * fh])
    }

    #[test]
    fn a_sheet_grows_flows_and_holds_its_mass() {
        // ELA 100 m under a 200-m continent: the ice climbs into its own accumulation (the
        // elevation feedback), domes up and flows to the coast.
        let (g, bed, ela, precip) = continent(100.0);
        let mut st = State::default();
        let f = Forcing {
            bed: &bed,
            ela: &ela,
            precip: &precip,
        };
        st.step(&g, &f, &params(), 60_000.0);
        let centre = st.thick[16 * 64 + 32];
        let edge = st.thick[16 * 64 + 41];
        assert!(
            centre > 1500.0 && centre > 1.5 * edge,
            "centre {centre}, edge {edge}"
        );
        assert!(st.thick[0] == 0.0, "no ice on the open sea");
        assert!(st.depress[16 * 64 + 32] > 0.2 * centre * 0.9 * (RHO_ICE / RHO_MANTLE) as f32);
        // Without a mass balance the flow alone conserves the ice: a dome on land everywhere.
        let (fw, fh) = (64, 32);
        let g2 = Grid::new(fw, fh, &[1.0e5; 32], 1.0e5, 64);
        let land = vec![200.0f32; fw * fh];
        let dome: Vec<f32> = (0..fw * fh)
            .map(|c| {
                let (i, j) = ((c % fw) as f64 - 32.0, (c / fw) as f64 - 16.0);
                (3000.0 * (1.0 - (i * i + j * j) / 64.0)).max(0.0) as f32
            })
            .collect();
        let mut st2 = State::settled(dome);
        let v0 = st2.volume_m3(&g2);
        let high = vec![1.0e6f32; fw * fh];
        let f0 = Forcing {
            bed: &land,
            ela: &high,
            precip: &vec![0.0; fw * fh],
        };
        let mut p0 = params();
        p0.beta_abl = 0.0;
        st2.step(&g2, &f0, &p0, 5000.0);
        let v1 = st2.volume_m3(&g2);
        assert!((v1 - v0).abs() < 1e-4 * v0, "{v0} → {v1}");
        assert!(st2.thick[16 * fw + 40] > 0.0, "the dome spread");
        let _ = g;
    }

    #[test]
    fn a_warm_line_melts_the_sheet_and_the_same_line_keeps_a_sheet_it_did_not_grow() {
        // The line at 300 m, above the 200-m continent: no ice grows from nothing …
        let (g, bed, ela, precip) = continent(300.0);
        let f = Forcing {
            bed: &bed,
            ela: &ela,
            precip: &precip,
        };
        let mut bare = State::default();
        bare.step(&g, &f, &params(), 30_000.0);
        assert!(bare.thick.iter().all(|&x| x == 0.0));
        // … but a thick sheet, its surface high above the line, survives it (hysteresis).
        let (_, _, ela_cold, _) = continent(100.0);
        let fc = Forcing {
            bed: &bed,
            ela: &ela_cold,
            precip: &precip,
        };
        let mut big = State::default();
        big.step(&g, &fc, &params(), 60_000.0);
        let v = big.volume_m3(&g);
        big.step(&g, &f, &params(), 20_000.0);
        assert!(big.volume_m3(&g) > 0.5 * v, "the sheet held its height");
        // A line far above it melts it.
        let (_, _, ela_hot, _) = continent(4000.0);
        let fh = Forcing {
            bed: &bed,
            ela: &ela_hot,
            precip: &precip,
        };
        big.step(&g, &fh, &params(), 20_000.0);
        assert!(big.volume_m3(&g) < 0.05 * v);
    }
}
