// SPDX-License-Identifier: LGPL-3.0-or-later
//! The ocean's heat transport by basin on the climate's coarse grid (P1-26 T3;
//! `docs/design/p1-26-per-cell-climate.md` §2.6): wind-driven gyres and an overturning conveyor,
//! moving the slab's heat (`ebm2d.rs`) in flux form. The slab's eddy diffusion is `ebm2d.rs`'s.
//!
//! **Stommel gyres** (Stommel 1948, "The westward intensification of wind-driven ocean currents",
//! Trans. AGU 29: 202–206; implemented from the paper's equation): the transport streamfunction
//! ψ (m³/s) solves
//!
//! ```text
//! r ∇²ψ + β ∂ψ/∂x = curl τ / ρ₀,     ψ = 0 on the coasts
//! ```
//!
//! on the cell corners of the sea, with τ the annual-mean zonal stress of the surface belt winds
//! ρ_a C_D |u| u, β = 2Ω cos φ / R and the friction r = 1.5 β Δx (a western boundary layer about
//! 1.5 cells wide). ψ is also 0 poleward of 75° and on latitude lines whose two rows hold no land
//! at all (the "Drake rule": an open channel has no Sverdrup closure, so it carries no gyre).
//! Gauss–Seidel sweeps east to west, the direction the Sverdrup balance integrates. The face
//! transports are differences of ψ between the face's two corners, so each cell's in- and
//! outflows cancel exactly whatever ψ is.
//!
//! **The overturning conveyor** (design §2.6 item 3): one sinking region per planet, in the
//! narrowest polar-connected basin — of the connected seas between 20° and 70° in either
//! hemisphere that reach the subtropics (≤ 30°) and hold sea poleward of 50° free of ice in late
//! winter, the one of the smallest mean true width (basin width is one of the reasons Ferreira et
//! al. 2018 give for Atlantic, not Pacific, sinking; a stand-in for salinity, which this model
//! lacks). Its open sea poleward of 50° sinks Ψ (`overturning_sv`); the same Ψ upwells evenly over
//! the connected sea equatorward of 50°; a potential flow carries it from upwelling to sinking
//! (∇·(∇φ) = S, solved by conjugate gradients). The heat it carries is ρc_p F (T_upwind − T_deep)
//! with T_deep 2 °C: in flux form, so heat is taken up where upwelled water warms and given back
//! where the surface water cools on its way to sinking, and Σ is conserved.
//!
//! Both act on the slab by upwind advection, sub-stepped to a Courant number of 0.5.

use super::coarse::Grid;
use super::ebm::WATER_J_M3_K;
use super::ebm2d::OceanTransport;

/// Density of sea water for the stress balance, kg/m³ (from memory, about 1,025).
pub const RHO_SEA: f64 = 1025.0; // ledger: CLM-177
/// Density of the air at the surface for the wind stress, kg/m³ (from memory, about 1.2).
pub const RHO_AIR: f64 = 1.2; // ledger: CLM-177
/// Neutral 10-m drag coefficient of the wind stress (Large & Yeager 2004, eq. 6a: 1.06–1.18·10⁻³
/// at 5–10 m/s; design §2.6, WP-14a).
pub const DRAG: f64 = 1.2e-3; // ledger: CLM-128
/// Friction of the gyres in units of β·Δx: the western boundary layer's width in cells.
pub const FRICTION_CELLS: f64 = 1.5; // ledger: CLM-128
/// No gyre poleward of this latitude, degrees.
pub const GYRE_MAX_LAT: f64 = 75.0; // ledger: CLM-129
/// Temperature of the deep water the conveyor upwells and sinks, °C (the design's round value).
pub const T_DEEP_C: f64 = 2.0; // ledger: CLM-163
/// The conveyor sinks poleward of this latitude and upwells equatorward of it, degrees.
pub const SINK_LAT: f64 = 50.0; // ledger: CLM-132
/// The basins the conveyor can sink in lie between these latitudes, degrees.
pub const BASIN_LAT: (f64, f64) = (20.0, 70.0); // ledger: CLM-132
/// A basin must reach this far toward the equator (the subtropics), degrees.
pub const BASIN_REACH_LAT: f64 = 30.0; // ledger: CLM-132
/// A cell is sea for the ocean's transport when its land fraction is below this.
pub const SEA_BELOW: f64 = 0.5;

/// What the ocean's transport needs to know of the planet.
pub struct Setup<'a> {
    pub grid: &'a Grid,
    /// Land fraction of every coarse cell.
    pub land: &'a [f64],
    /// The annual-mean surface zonal wind of every row, m/s (east positive).
    pub wind_u: &'a [f64],
    /// Rotation rate, rad/s.
    pub omega: f64,
    /// Radius, m (the climate's, Earth-normalized).
    pub radius_m: f64,
    /// Slab depth, m.
    pub slab_m: f64,
    /// Heat efficiency of the gyres (0: none).
    pub gyre_efficiency: f64,
    /// Conveyor transport, Sv (0: none).
    pub overturning_sv: f64,
    /// Late-winter sea-ice cover of every cell (March in the north, September in the south), from
    /// the previous climate; `None`: open everywhere.
    pub late_winter_ice: Option<&'a [f64]>,
}

/// The conveyor's sinking region.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sinking {
    /// Area-weighted mean latitude and longitude of the sinking cells, degrees.
    pub lat: f64,
    pub lon: f64,
    /// The basin's mean true width, km (Earth-normalized), and its hemisphere (+1 north).
    pub width_km: f64,
    pub hemisphere: i8,
    pub cells: usize,
    pub sv: f64,
}

/// The gyres and the conveyor of one climate, as face transports.
#[derive(Clone, Debug, Default)]
pub struct Ocean {
    w: usize,
    h: usize,
    /// ψ on the corners, (w) × (h + 1), m³/s (corner (i, e): the north-west corner of cell (i, e)).
    pub psi: Vec<f64>,
    /// Transport across every cell's east face (eastward) and north face (northward), m³/s:
    /// gyres × efficiency plus the conveyor.
    pub f_east: Vec<f64>,
    pub f_north: Vec<f64>,
    /// The conveyor's share of them.
    pub c_east: Vec<f64>,
    pub c_north: Vec<f64>,
    /// Water volume of every cell's slab, m³ (0 on land).
    vol: Vec<f64>,
    pub sinking: Option<Sinking>,
    /// Gauss–Seidel sweeps the gyres took.
    pub sweeps: u32,
}

impl Ocean {
    /// The largest |ψ|, Sv.
    pub fn psi_max_sv(&self) -> f64 {
        self.psi.iter().fold(0.0f64, |a, &b| a.max(b.abs())) / 1.0e6
    }

    /// The face transports' divergence of every cell, m³/s (round-off for the gyres; the
    /// conveyor's sources and sinks).
    pub fn divergence(&self, gyres_only: bool) -> Vec<f64> {
        let (w, h) = (self.w, self.h);
        (0..w * h)
            .map(|c| {
                let (i, j) = (c % w, c / w);
                let fe = |c: usize| {
                    if gyres_only {
                        self.f_east[c] - self.c_east[c]
                    } else {
                        self.f_east[c]
                    }
                };
                let fnn = |c: usize| {
                    if gyres_only {
                        self.f_north[c] - self.c_north[c]
                    } else {
                        self.f_north[c]
                    }
                };
                let west = j * w + (i + w - 1) % w;
                let south_in = if j + 1 < h { fnn(c + w) } else { 0.0 };
                fe(c) - fe(west) + fnn(c) - south_in
            })
            .collect()
    }

    /// Builds the gyres and the conveyor of a climate.
    pub fn build(s: &Setup<'_>) -> Ocean {
        let g = s.grid;
        let (w, h) = (g.w, g.h);
        let n = w * h;
        let r = s.radius_m;
        let sea = |c: usize| s.land[c] < SEA_BELOW;
        let area = g.areas();
        let vol: Vec<f64> = (0..n)
            .map(|c| {
                if sea(c) {
                    s.slab_m * (1.0 - s.land[c]) * area[c] * r * r
                } else {
                    0.0
                }
            })
            .collect();
        let mut o = Ocean {
            w,
            h,
            psi: vec![0.0; w * (h + 1)],
            f_east: vec![0.0; n],
            f_north: vec![0.0; n],
            c_east: vec![0.0; n],
            c_north: vec![0.0; n],
            vol,
            sinking: None,
            sweeps: 0,
        };
        if s.gyre_efficiency > 0.0 {
            o.gyres(s);
            for c in 0..n {
                let (i, j) = (c % w, c / w);
                let at = |ii: usize, e: usize| o.psi[e * w + ii % w];
                // East face: corners (i + 1, j) north and (i + 1, j + 1) south; u = −∂ψ/∂y.
                o.f_east[c] = s.gyre_efficiency * (at(i + 1, j + 1) - at(i + 1, j));
                // North face: corners (i, j) west and (i + 1, j) east; v = ∂ψ/∂x.
                o.f_north[c] = s.gyre_efficiency * (at(i + 1, j) - at(i, j));
            }
        }
        if s.overturning_sv > 0.0 {
            o.conveyor(s, &area);
            for c in 0..n {
                o.f_east[c] += o.c_east[c];
                o.f_north[c] += o.c_north[c];
            }
        }
        o
    }

    /// Solves Stommel's equation for ψ on the corners.
    fn gyres(&mut self, s: &Setup<'_>) {
        let g = s.grid;
        let (w, h) = (g.w, g.h);
        let r = s.radius_m;
        let land = |i: usize, j: usize| s.land[j * w + i % w] >= SEA_BELOW;
        let row_has_land: Vec<bool> = (0..h).map(|j| (0..w).any(|i| land(i, j))).collect();
        let dlam = g.lon_w[0];
        // Fixed corners (ψ = 0).
        let fixed: Vec<bool> = (0..w * (h + 1))
            .map(|k| {
                let (i, e) = (k % w, k / w);
                if e == 0 || e == h || g.lat_edge[e].abs() >= GYRE_MAX_LAT {
                    return true;
                }
                // The Drake rule: neither row of the line holds land.
                if !row_has_land[e - 1] && !row_has_land[e] {
                    return true;
                }
                let iw = (i + w - 1) % w;
                land(iw, e - 1) || land(i, e - 1) || land(iw, e) || land(i, e)
            })
            .collect();
        // Stress at every row's middle, and its curl on every corner line.
        let tau: Vec<f64> = s
            .wind_u
            .iter()
            .map(|&u| RHO_AIR * DRAG * u.abs() * u)
            .collect();
        struct Line {
            dx: f64,
            dyn_: f64,
            dys: f64,
            beta: f64,
            rf: f64,
            rhs: f64,
        }
        let lines: Vec<Option<Line>> = (0..=h)
            .map(|e| {
                if e == 0 || e == h {
                    return None;
                }
                let phi = g.lat_edge[e].to_radians();
                let dx = r * phi.cos().max(1e-6) * dlam;
                let dyn_ = r * g.dphi(e - 1);
                let dys = r * g.dphi(e);
                let beta = 2.0 * s.omega * phi.cos() / r;
                let (pn, ps) = (g.lat_c(e - 1).to_radians(), g.lat_c(e).to_radians());
                let curl = -(tau[e - 1] * pn.cos() - tau[e] * ps.cos())
                    / (r * phi.cos().max(1e-6) * (pn - ps));
                Some(Line {
                    dx,
                    dyn_,
                    dys,
                    beta,
                    rf: FRICTION_CELLS * beta * dx,
                    rhs: curl / RHO_SEA,
                })
            })
            .collect();
        let psi = &mut self.psi;
        let mut sweeps = 0;
        let scale = lines
            .iter()
            .flatten()
            .map(|l| (l.rhs / l.beta.max(1e-30) * l.dx * w as f64).abs())
            .fold(0.0, f64::max)
            .max(1.0);
        for _ in 0..5000 {
            sweeps += 1;
            let mut change = 0.0f64;
            for e in 1..h {
                let Some(l) = &lines[e] else { continue };
                let ybar = 0.5 * (l.dyn_ + l.dys);
                let cx = l.rf / (l.dx * l.dx);
                let (cn, cs) = (l.rf / (l.dyn_ * ybar), l.rf / (l.dys * ybar));
                let cb = l.beta / l.dx;
                let diag = 2.0 * cx + cn + cs + cb;
                for ii in (0..w).rev() {
                    let k = e * w + ii;
                    if fixed[k] {
                        continue;
                    }
                    let east = psi[e * w + (ii + 1) % w];
                    let west = psi[e * w + (ii + w - 1) % w];
                    let north = psi[(e - 1) * w + ii];
                    let south = psi[(e + 1) * w + ii];
                    let new =
                        (cx * (east + west) + cn * north + cs * south + cb * east - l.rhs) / diag;
                    change = change.max((new - psi[k]).abs());
                    psi[k] = new;
                }
            }
            if change < 1e-7 * scale {
                break;
            }
        }
        self.sweeps = sweeps;
    }

    /// Finds the sinking basin and the conveyor's potential flow.
    fn conveyor(&mut self, s: &Setup<'_>, area: &[f64]) {
        let g = s.grid;
        let (w, h) = (g.w, g.h);
        let n = w * h;
        let sea = |c: usize| s.land[c] < SEA_BELOW;
        let lat: Vec<f64> = (0..h).map(|j| g.lat_c(j)).collect();
        let ice = |c: usize| s.late_winter_ice.map_or(0.0, |v| v[c]);
        let nbrs = |c: usize| -> [Option<usize>; 4] {
            let (i, j) = (c % w, c / w);
            [
                Some(j * w + (i + 1) % w),
                Some(j * w + (i + w - 1) % w),
                (j > 0).then(|| c - w),
                (j + 1 < h).then(|| c + w),
            ]
        };
        // Connected seas within one set of cells (4-neighbours, x wraps); labels in scan order.
        let components = |inside: &dyn Fn(usize) -> bool| -> Vec<Option<usize>> {
            let mut lab: Vec<Option<usize>> = vec![None; n];
            let mut next = 0;
            for c0 in 0..n {
                if lab[c0].is_some() || !inside(c0) {
                    continue;
                }
                lab[c0] = Some(next);
                let mut stack = vec![c0];
                while let Some(c) = stack.pop() {
                    for nb in nbrs(c).into_iter().flatten() {
                        if lab[nb].is_none() && inside(nb) {
                            lab[nb] = Some(next);
                            stack.push(nb);
                        }
                    }
                }
                next += 1;
            }
            lab
        };
        let true_w = |j: usize| s.radius_m * lat[j].to_radians().cos() * g.lon_w[0] / 1000.0;
        let mut best: Option<(f64, i8, Vec<usize>, usize)> = None;
        for hemi in [1i8, -1] {
            // A row belongs to the band when its centre is past the band's equatorward end and
            // its poleward edge is inside the poleward end: on an equal-area grid the polar rows
            // are tall, and one reaching from 66° to 73° would join the Atlantic and the Pacific
            // through the Arctic into one basin.
            let in_band = |c: usize| {
                let j = c / w;
                let l = lat[j] * f64::from(hemi);
                let pole_edge =
                    (g.lat_edge[j] * f64::from(hemi)).max(g.lat_edge[j + 1] * f64::from(hemi));
                sea(c) && l >= BASIN_LAT.0 && pole_edge <= BASIN_LAT.1
            };
            let lab = components(&in_band);
            let count = lab.iter().flatten().max().map_or(0, |m| m + 1);
            for b in 0..count {
                let cells: Vec<usize> = (0..n).filter(|&c| lab[c] == Some(b)).collect();
                let reach = cells.iter().any(|&c| lat[c / w].abs() <= BASIN_REACH_LAT);
                let sink: Vec<usize> = cells
                    .iter()
                    .copied()
                    .filter(|&c| lat[c / w].abs() >= SINK_LAT && ice(c) < 0.5)
                    .collect();
                if !reach || sink.is_empty() {
                    continue;
                }
                // Mean true width over the rows the basin occupies.
                let mut per_row = vec![0usize; h];
                for &c in &cells {
                    per_row[c / w] += 1;
                }
                let rows: Vec<usize> = (0..h).filter(|&j| per_row[j] > 0).collect();
                let width = rows
                    .iter()
                    .map(|&j| per_row[j] as f64 * true_w(j))
                    .sum::<f64>()
                    / rows.len() as f64;
                let better = match &best {
                    None => true,
                    Some((bw, ..)) => width < *bw,
                };
                if better {
                    best = Some((width, hemi, sink, cells[0]));
                }
            }
        }
        let Some((width, hemi, sink, seed)) = best else {
            return;
        };
        // The connected sea the basin belongs to: the upwelling.
        let all = components(&sea);
        let comp = all[seed];
        let up: Vec<usize> = (0..n)
            .filter(|&c| all[c] == comp && lat[c / w].abs() < SINK_LAT)
            .collect();
        if up.is_empty() {
            return;
        }
        let q = s.overturning_sv * 1.0e6;
        let a_up: f64 = up.iter().map(|&c| area[c]).sum();
        let a_sink: f64 = sink.iter().map(|&c| area[c]).sum();
        let mut src = vec![0.0; n];
        for &c in &up {
            src[c] += q * area[c] / a_up;
        }
        for &c in &sink {
            src[c] -= q * area[c] / a_sink;
        }
        // Potential flow on the component: Σ_faces G (φ_c − φ_nb) = src (outflow positive).
        let inside: Vec<bool> = (0..n).map(|c| all[c] == comp).collect();
        let ge: Vec<f64> = (0..n)
            .map(|c| {
                let (i, j) = (c % w, c / w);
                let e = j * w + (i + 1) % w;
                if inside[c] && inside[e] {
                    g.g_ew(i, j)
                } else {
                    0.0
                }
            })
            .collect();
        let gs: Vec<f64> = (0..n)
            .map(|c| {
                let (i, j) = (c % w, c / w);
                if j + 1 < h && inside[c] && inside[c + w] {
                    g.g_ns(i, j)
                } else {
                    0.0
                }
            })
            .collect();
        let apply = |x: &[f64]| -> Vec<f64> {
            (0..n)
                .map(|c| {
                    if !inside[c] {
                        return 0.0;
                    }
                    let (i, j) = (c % w, c / w);
                    let west = j * w + (i + w - 1) % w;
                    let mut y =
                        ge[c] * (x[c] - x[j * w + (i + 1) % w]) + ge[west] * (x[c] - x[west]);
                    if j + 1 < h {
                        y += gs[c] * (x[c] - x[c + w]);
                    }
                    if j > 0 {
                        y += gs[c - w] * (x[c] - x[c - w]);
                    }
                    y
                })
                .collect()
        };
        let phi = conjugate_gradient(&apply, &src, &inside);
        for c in 0..n {
            let (i, j) = (c % w, c / w);
            self.c_east[c] = ge[c] * (phi[c] - phi[j * w + (i + 1) % w]);
            // Northward across the north face: from the row south (this cell) to the row north.
            if j > 0 {
                self.c_north[c] = gs[c - w] * (phi[c] - phi[c - w]);
            }
        }
        let (mut sx, mut sy, mut sz, mut sa) = (0.0, 0.0, 0.0, 0.0);
        for &c in &sink {
            let (i, j) = (c % w, c / w);
            let (la, lo) = (
                lat[j].to_radians(),
                (i as f64 + 0.5) / w as f64 * std::f64::consts::TAU,
            );
            sx += area[c] * la.cos() * lo.cos();
            sy += area[c] * la.cos() * lo.sin();
            sz += area[c] * la.sin();
            sa += area[c];
        }
        let _ = sa;
        self.sinking = Some(Sinking {
            lat: sz.atan2(sx.hypot(sy)).to_degrees(),
            lon: sy.atan2(sx).to_degrees().rem_euclid(360.0),
            width_km: width,
            hemisphere: hemi,
            cells: sink.len(),
            sv: s.overturning_sv,
        });
    }
}

/// Conjugate gradients for the singular, symmetric positive semi-definite Neumann Laplacian:
/// the right-hand side and the iterates are kept mean-free over `inside`. Serial sums, so the
/// result is the same on any thread count.
fn conjugate_gradient(apply: &dyn Fn(&[f64]) -> Vec<f64>, b: &[f64], inside: &[bool]) -> Vec<f64> {
    let n = b.len();
    let cnt = inside.iter().filter(|&&x| x).count().max(1) as f64;
    let demean = |v: &mut [f64]| {
        let m: f64 = (0..n).filter(|&c| inside[c]).map(|c| v[c]).sum::<f64>() / cnt;
        for c in 0..n {
            v[c] = if inside[c] { v[c] - m } else { 0.0 };
        }
    };
    let dot = |a: &[f64], b: &[f64]| -> f64 { a.iter().zip(b).map(|(x, y)| x * y).sum() };
    let mut rr = b.to_vec();
    demean(&mut rr);
    let mut x = vec![0.0; n];
    let mut p = rr.clone();
    let mut rs = dot(&rr, &rr);
    let tol = 1e-20 * rs.max(1e-300);
    for _ in 0..4 * n {
        if rs <= tol {
            break;
        }
        let ap = apply(&p);
        let pap = dot(&p, &ap);
        if pap <= 0.0 {
            break;
        }
        let a = rs / pap;
        for c in 0..n {
            x[c] += a * p[c];
            rr[c] -= a * ap[c];
        }
        let rs2 = dot(&rr, &rr);
        let beta = rs2 / rs;
        for c in 0..n {
            p[c] = rr[c] + beta * p[c];
        }
        rs = rs2;
    }
    x
}

impl OceanTransport for Ocean {
    fn step(&self, tm: &mut [f64], dt: f64, north_j: &mut [f64]) {
        let (w, h) = (self.w, self.h);
        let n = w * h;
        if self.vol.is_empty() {
            return;
        }
        // Sub-steps: Courant ≤ 0.5 on every cell's outflow.
        let mut worst = 0.0f64;
        for c in 0..n {
            if self.vol[c] <= 0.0 {
                continue;
            }
            let (i, j) = (c % w, c / w);
            let west = j * w + (i + w - 1) % w;
            let mut out = self.f_east[c].max(0.0) + (-self.f_east[west]).max(0.0);
            out += self.f_north[c].max(0.0);
            if j + 1 < h {
                out += (-self.f_north[c + w]).max(0.0);
            }
            worst = worst.max(out * dt / self.vol[c]);
        }
        if worst == 0.0 {
            return;
        }
        let subs = (worst / 0.5).ceil().max(1.0) as usize;
        let ds = dt / subs as f64;
        let rc = WATER_J_M3_K;
        let mut th: Vec<f64> = tm.iter().map(|t| t - T_DEEP_C).collect();
        let mut dq = vec![0.0; n]; // heat change × (1/ρc_p), m³·K
        for _ in 0..subs {
            dq.iter_mut().for_each(|x| *x = 0.0);
            for c in 0..n {
                let (i, j) = (c % w, c / w);
                // East face.
                let f = self.f_east[c];
                if f != 0.0 {
                    let e = j * w + (i + 1) % w;
                    let carried = f * if f > 0.0 { th[c] } else { th[e] } * ds;
                    dq[c] -= carried;
                    dq[e] += carried;
                }
                // North face.
                let f = self.f_north[c];
                if f != 0.0 && j > 0 {
                    let nn = c - w;
                    let carried = f * if f > 0.0 { th[c] } else { th[nn] } * ds;
                    dq[c] -= carried;
                    dq[nn] += carried;
                    north_j[j - 1] += carried * rc;
                }
            }
            for c in 0..n {
                if self.vol[c] > 0.0 {
                    th[c] += dq[c] / self.vol[c];
                }
            }
        }
        for c in 0..n {
            if self.vol[c] > 0.0 {
                tm[c] = th[c] + T_DEEP_C;
            }
        }
    }
}

/// The annual-mean surface zonal wind of every row of grid `g`, m/s (east positive): the belt
/// winds (`belts.rs`) weighted by each month's regimes at the row's belt latitude (the seasonal
/// hemispheric shift), averaged over the year.
pub fn belt_wind_u(p: &super::ClimateParams, g: &Grid, tilt_deg: f64) -> Vec<f64> {
    let winds = super::belts::winds(p);
    (0..g.h)
        .map(|j| {
            let phi = g.lat_c(j);
            (0..12)
                .map(|m| {
                    let hemi = super::belts::hemispheric_shift(p, tilt_deg, m);
                    let pb = phi - super::belts::shift(phi, 0.0, hemi);
                    let wt = super::belts::regime_weights(phi, pb);
                    (0..super::belts::REGIMES)
                        .map(|r| wt[r] * winds[r].0)
                        .sum::<f64>()
                })
                .sum::<f64>()
                / 12.0
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::climate::ClimateParams;

    const OMEGA: f64 = 7.292e-5;

    fn grid() -> Grid {
        Grid::new(512, 256, &[1.0e4; 256], 1.0e4, 128)
    }

    fn setup<'a>(g: &'a Grid, land: &'a [f64], u: &'a [f64], sv: f64) -> Setup<'a> {
        Setup {
            grid: g,
            land,
            wind_u: u,
            omega: OMEGA,
            radius_m: crate::climate::ebm2d::EARTH_RADIUS_M,
            slab_m: 50.0,
            gyre_efficiency: 1.0,
            overturning_sv: sv,
            late_winter_ice: None,
        }
    }

    /// Land everywhere except the sea cells `sea` picks.
    fn land_of(g: &Grid, sea: &dyn Fn(f64, usize) -> bool) -> Vec<f64> {
        (0..g.w * g.h)
            .map(|c| {
                if sea(g.lat_c(c / g.w), c % g.w) {
                    0.0
                } else {
                    1.0
                }
            })
            .collect()
    }

    #[test]
    fn a_northern_basin_turns_clockwise_with_a_western_boundary_current() {
        let g = grid();
        // A basin 60° wide from 10°N to 60°N (columns 20–41 of 128).
        let land = land_of(&g, &|lat, i| {
            (10.0..60.0).contains(&lat) && (20..42).contains(&i)
        });
        let u = belt_wind_u(&ClimateParams::default(), &g, 23.4);
        let o = Ocean::build(&setup(&g, &land, &u, 0.0));
        let psi = o.psi_max_sv();
        assert!((5.0..80.0).contains(&psi), "|ψ| max {psi} Sv");
        // Clockwise: positive ψ in the subtropical gyre (northward flow in the west, southward in
        // the east).
        let j = (0..g.h)
            .min_by(|a, b| {
                (g.lat_c(*a) - 30.0)
                    .abs()
                    .total_cmp(&(g.lat_c(*b) - 30.0).abs())
            })
            .unwrap();
        let v: Vec<f64> = (20..42).map(|i| o.f_north[j * g.w + i]).collect();
        assert!(v[0] > 0.0, "western boundary flows north: {v:?}");
        let interior = v[8..18].iter().sum::<f64>() / 10.0;
        assert!(interior < 0.0, "the interior flows south: {interior}");
        // Western intensification: the boundary current sits in ≤ 2 cells and is ≥ 3× the
        // interior's speed per cell.
        let peak = v.iter().cloned().fold(f64::MIN, f64::max);
        let k = v.iter().position(|&x| x == peak).unwrap();
        assert!(k <= 1, "peak in column {k} of the basin: {v:?}");
        assert!(
            peak >= 3.0 * interior.abs(),
            "peak {peak} vs interior {interior}"
        );
        // Non-divergent to round-off.
        let div = o.divergence(true);
        let worst = div.iter().fold(0.0f64, |a, b| a.max(b.abs()));
        assert!(worst < 1e-6 * psi * 1e6, "divergence {worst}");
    }

    #[test]
    fn the_conveyor_sinks_in_the_narrow_basin_and_conserves_heat() {
        let g = grid();
        // Two northern basins open to a southern ocean: narrow (30° wide) and wide (90°), a
        // Drake-like channel at 55–60°S.
        let land = land_of(&g, &|lat, i| {
            let lon = (i as f64 + 0.5) * 360.0 / 128.0;
            if lat < -55.0 && lat > -62.0 {
                return true;
            }
            if lat <= -62.0 || lat > 72.0 {
                return false;
            }
            if lat < -35.0 {
                return true;
            }
            (20.0..50.0).contains(&lon) || (120.0..210.0).contains(&lon)
        });
        let u = belt_wind_u(&ClimateParams::default(), &g, 23.4);
        let o = Ocean::build(&setup(&g, &land, &u, 15.0));
        let s = o.sinking.clone().expect("a sinking region");
        assert_eq!(s.hemisphere, 1);
        assert!((20.0..50.0).contains(&s.lon), "sinks at {s:?}");
        // The conveyor's sources and sinks: +15 Sv upwelling, −15 Sv sinking, Σ 0.
        let div = o.divergence(false);
        let total: f64 = div.iter().sum();
        assert!(total.abs() < 1e-3, "Σ divergence {total}");
        let sinking: f64 = div.iter().filter(|&&x| x < 0.0).sum();
        assert!((sinking / 1.0e6 + 15.0).abs() < 0.01, "sinking {sinking}");
        // Heat: Σ vol·T conserved by the transport.
        let n = g.w * g.h;
        let mut tm: Vec<f64> = (0..n)
            .map(|c| 25.0 - 0.3 * g.lat_c(c / g.w).abs())
            .collect();
        let heat = |t: &[f64]| -> f64 { (0..n).map(|c| o.vol[c] * t[c]).sum() };
        let h0 = heat(&tm);
        let mut north = vec![0.0; g.h - 1];
        o.step(&mut tm, 4.38e5, &mut north);
        let h1 = heat(&tm);
        assert!(((h1 - h0) / h0).abs() < 1e-9, "{h0} → {h1}");
        // It carries heat north at the northern basin's latitudes.
        let e = (0..g.h - 1)
            .min_by(|a, b| {
                (g.lat_edge[a + 1] - 25.0)
                    .abs()
                    .total_cmp(&(g.lat_edge[b + 1] - 25.0).abs())
            })
            .unwrap();
        assert!(north[e] > 0.0, "northward heat at 25°N: {}", north[e]);
    }
}
