// SPDX-License-Identifier: LGPL-3.0-or-later
//! How fast is one stencil step on a sphere-native grid, against today's Lambert raster?
//!
//! Research tool for docs/research/notes/sphere-native-grids.md (2026-10-04). Not shipped.
//!
//! Every layout holds about the same number of cells (default: the quick 16k preset's L1 raster,
//! 1,024 × 326 = 333,824) and the same terrain (one smooth function of the unit vector). On each
//! it times, with the shipped build profile (release, thin LTO, generic x86-64 unless run.sh adds
//! `-C target-cpu=native`):
//! - `diff`: one application of the implicit diffusion operator (I + Δt·L)·h with a coefficient per
//!   cell and neighbour, which is the cost of one conjugate-gradient iteration's matrix product in
//!   `surface/diffusion.rs`; patch layouts first refresh their ghost cells (the edge remap);
//! - `rcv`: D8-style receivers (steepest descent over the 8 square or 6 hexagonal neighbours);
//! - `stack`: the donor lists and the Braun–Willett stack (serial, as shipped);
//! - `acc`: drainage area accumulated down the stack (serial);
//! - `cg`: a whole implicit diffusion solve (Jacobi-preconditioned conjugate gradients to a
//!   relative residual of 1e-8) in the layout's true metric, at D·Δt/Δx² = 30, the shelf's value
//!   (`surface/diffusion.rs`): iterations and time;
//! - `remap`: resampling the field onto the Lambert raster (4-point weighted gather per raster cell),
//!   the cost a sphere-native grid pays every time the game raster is written.
//!
//! Layouts:
//! - `lambert`: today's raster, W × H with W/H = π, wrapping in x, closed at the pole rows;
//! - `cube`: six equiangular gnomonic faces (the FV3 family) of n × n cells, one ghost ring each;
//! - `healpix`: twelve HEALPix base pixels of Nside × Nside (Górski et al. 2005), one ghost ring;
//! - `icos`: ten icosahedral diamonds of n × n triangle-lattice points with six neighbours each
//!   (the GME/NICAM storage), one ghost ring, the two pole points left out;
//! - `fib-*`: the Fibonacci sphere's spherical Delaunay graph (Deep Time's own `SphereMesh`) as a
//!   compressed neighbour list (CSR) in Fibonacci order (`plain`), in a Hilbert order (a Hilbert
//!   curve on each cube face, faces in turn, as S2 numbers cells), in random order (worst case),
//!   and in Hilbert order as a padded fixed-width list (`ell`, ELLPACK).
//!
//! Patch layouts find each ghost cell's source by reflecting the edge cell's inward neighbour and
//! taking the nearest owned cell; across a cube edge that is the adjacent cell of the next face
//! (the finite-volume neighbour, no interpolation), and the tool prints how far the sources lie
//! against the inward spacing as a check.
//!
//! Usage: sphere-grid-bench [cells=333824] [threads=8] [reps=40]

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, PI, SQRT_2};
use std::hint::black_box;
use std::time::Instant;

use deeptime_core::sphere::{SampleTree, SphereMesh, SplitMix64, fibonacci_sphere};
use glam::DVec3;
use rayon::prelude::*;

// ------------------------------------------------------------------------------------------------
// Terrain: the same smooth field on every layout.

struct Terrain {
    waves: Vec<(DVec3, f64, f64, f64)>,
}

impl Terrain {
    fn new() -> Terrain {
        let mut rng = SplitMix64::new(20261004);
        let waves = (0..32)
            .map(|m| {
                let f = 2.0 * (60f64).powf(m as f64 / 31.0);
                (rng.unit_vector(), f, 1.0 / f, rng.range(0.0, 2.0 * PI))
            })
            .collect();
        Terrain { waves }
    }

    fn at(&self, v: DVec3) -> f64 {
        self.waves
            .iter()
            .map(|&(d, f, a, p)| a * (f * d.dot(v) + p).sin())
            .sum()
    }
}

// ------------------------------------------------------------------------------------------------
// Patch layouts: faces of regular arrays with one ring of ghost cells.

#[derive(Clone, Copy, PartialEq)]
enum Stencil {
    Square,
    Hex,
}

struct Patches {
    name: &'static str,
    stencil: Stencil,
    nf: usize,
    nw: usize,
    nh: usize,
    pw: usize,
    fsz: usize,
    /// Unit vector per padded index (ghosts: their source's).
    pos: Vec<DVec3>,
    /// Padded index → the owned padded index it stands for.
    canon: Vec<u32>,
    /// (ghost, source) pairs.
    ghosts: Vec<(u32, u32)>,
    /// Owned padded indices, face by face, row by row.
    owned: Vec<u32>,
    /// Diffusion offsets (4 or 6) and routing offsets (8 or 6) with inverse step lengths.
    doff: Vec<isize>,
    roff: Vec<isize>,
    rinv: Vec<f64>,
    /// Ghost-source distance over inward spacing: median and max (the check).
    ghost_check: (f64, f64),
}

enum GhostRule<'a> {
    /// Reflect the inward neighbour and take the nearest owned cell.
    Reflect,
    /// Lambert: wrap columns, clamp rows (no flux through the pole rows).
    Lambert(&'a dyn Fn(usize, usize) -> (usize, usize)),
}

impl Patches {
    fn build(
        name: &'static str,
        stencil: Stencil,
        nf: usize,
        nw: usize,
        nh: usize,
        centre: &(dyn Fn(usize, usize, usize) -> DVec3 + Sync),
        rule: GhostRule,
    ) -> Patches {
        let pw = nw + 2;
        let ph = nh + 2;
        let fsz = pw * ph;
        let m = nf * fsz;
        let idx = |f: usize, pi: usize, pj: usize| f * fsz + pj * pw + pi;
        let mut pos = vec![DVec3::ZERO; m];
        let mut canon: Vec<u32> = (0..m as u32).collect();
        let mut owned = Vec::with_capacity(nf * nw * nh);
        for f in 0..nf {
            for j in 0..nh {
                for i in 0..nw {
                    let c = idx(f, i + 1, j + 1);
                    pos[c] = centre(f, i, j);
                    owned.push(c as u32);
                }
            }
        }
        let tree_pts: Vec<DVec3> = owned.iter().map(|&c| pos[c as usize]).collect();
        let tree = SampleTree::new(&tree_pts);
        let mut ghosts = Vec::new();
        let mut ratios = Vec::new();
        for f in 0..nf {
            for pj in 0..ph {
                for pi in 0..pw {
                    let inside = (1..=nw).contains(&pi) && (1..=nh).contains(&pj);
                    if inside {
                        continue;
                    }
                    let g = idx(f, pi, pj);
                    let src = match &rule {
                        GhostRule::Lambert(map) => {
                            let (i, j) = map(pi, pj);
                            idx(f, i, j)
                        }
                        GhostRule::Reflect => {
                            let ci = pi.clamp(1, nw);
                            let cj = pj.clamp(1, nh);
                            let di = pi as isize - ci as isize;
                            let dj = pj as isize - cj as isize;
                            let oi = (ci as isize - di) as usize;
                            let oj = (cj as isize - dj) as usize;
                            let c = idx(f, ci, cj);
                            let o = idx(f, oi, oj);
                            let guess = (2.0 * pos[c] - pos[o]).normalize();
                            let near = tree.nearest_n(guess, 3);
                            let s = near
                                .iter()
                                .map(|&(k, _)| owned[k as usize] as usize)
                                .find(|&s| s != c)
                                .unwrap_or(c);
                            let inward = (pos[c] - pos[o]).length();
                            if inward > 0.0 {
                                ratios.push((pos[c] - pos[s]).length() / inward);
                            }
                            s
                        }
                    };
                    canon[g] = src as u32;
                    pos[g] = pos[src];
                    ghosts.push((g as u32, src as u32));
                }
            }
        }
        ratios.sort_by(f64::total_cmp);
        let ghost_check = if ratios.is_empty() {
            (1.0, 1.0)
        } else {
            (ratios[ratios.len() / 2], *ratios.last().unwrap())
        };
        let p = pw as isize;
        let (doff, roff, rinv) = match stencil {
            Stencil::Square => (
                vec![1, -1, p, -p],
                vec![1, 1 - p, -p, -1 - p, -1, -1 + p, p, 1 + p],
                vec![
                    1.0,
                    1.0 / SQRT_2,
                    1.0,
                    1.0 / SQRT_2,
                    1.0,
                    1.0 / SQRT_2,
                    1.0,
                    1.0 / SQRT_2,
                ],
            ),
            Stencil::Hex => {
                let o = vec![1, -1, p, -p, 1 - p, -1 + p];
                (o.clone(), o, vec![1.0; 6])
            }
        };
        Patches {
            name,
            stencil,
            nf,
            nw,
            nh,
            pw,
            fsz,
            pos,
            canon,
            ghosts,
            owned,
            doff,
            roff,
            rinv,
            ghost_check,
        }
    }

    fn m(&self) -> usize {
        self.nf * self.fsz
    }

    fn fill(&self, h: &mut [f64]) {
        for &(g, s) in &self.ghosts {
            h[g as usize] = h[s as usize];
        }
    }

    /// Is padded row r (face-major) an interior row?
    fn interior_row(&self, r: usize) -> bool {
        let pj = r % (self.nh + 2);
        pj != 0 && pj != self.nh + 1
    }
}

/// Today's raster: W × H, lon = 2π(i + ½)/W, sin φ = 1 − 2(j + ½)/H.
fn lambert(n: usize) -> Patches {
    let w = ((n as f64) * PI).sqrt().round() as usize;
    let h = (n as f64 / w as f64).round() as usize;
    let centre = move |_f: usize, i: usize, j: usize| {
        let lon = 2.0 * PI * (i as f64 + 0.5) / w as f64;
        let z = 1.0 - 2.0 * (j as f64 + 0.5) / h as f64;
        let r = (1.0 - z * z).max(0.0).sqrt();
        DVec3::new(r * lon.cos(), r * lon.sin(), z)
    };
    let map = move |pi: usize, pj: usize| {
        let i = if pi == 0 {
            w
        } else if pi == w + 1 {
            1
        } else {
            pi
        };
        (i, pj.clamp(1, h))
    };
    Patches::build(
        "lambert",
        Stencil::Square,
        1,
        w,
        h,
        &centre,
        GhostRule::Lambert(&map),
    )
}

/// Face frames (normal, u, v) of the cube.
const CUBE: [[[f64; 3]; 3]; 6] = [
    [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
    [[0., 1., 0.], [-1., 0., 0.], [0., 0., 1.]],
    [[-1., 0., 0.], [0., -1., 0.], [0., 0., 1.]],
    [[0., -1., 0.], [1., 0., 0.], [0., 0., 1.]],
    [[0., 0., 1.], [0., 1., 0.], [-1., 0., 0.]],
    [[0., 0., -1.], [0., 1., 0.], [1., 0., 0.]],
];

fn v3(a: [f64; 3]) -> DVec3 {
    DVec3::from_array(a)
}

/// Six equiangular gnomonic faces of n × n.
fn cube(n_total: usize) -> Patches {
    let n = ((n_total as f64) / 6.0).sqrt().round() as usize;
    let centre = move |f: usize, i: usize, j: usize| {
        let a = -FRAC_PI_4 + (i as f64 + 0.5) * FRAC_PI_2 / n as f64;
        let b = -FRAC_PI_4 + (j as f64 + 0.5) * FRAC_PI_2 / n as f64;
        let [nn, u, v] = CUBE[f];
        (v3(nn) + a.tan() * v3(u) + b.tan() * v3(v)).normalize()
    };
    Patches::build("cube", Stencil::Square, 6, n, n, &centre, GhostRule::Reflect)
}

/// HEALPix centre of base pixel f, pixel (ix, iy), from the nested-scheme formulas of Górski et
/// al. 2005 (pix2ang with face coordinates; the centre is at half-integer offsets).
fn healpix_centre(nside: usize, f: usize, ix: usize, iy: usize) -> DVec3 {
    const JRLL: [i64; 12] = [2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];
    const JPLL: [i64; 12] = [1, 3, 5, 7, 0, 2, 4, 6, 1, 3, 5, 7];
    let ns = nside as i64;
    let (ix, iy) = (ix as i64, iy as i64);
    let jr = JRLL[f] * ns - ix - iy - 1;
    let fact2 = 4.0 / (12 * ns * ns) as f64;
    let (nr, z) = if jr < ns {
        (jr, 1.0 - (jr * jr) as f64 * fact2)
    } else if jr > 3 * ns {
        let nr = 4 * ns - jr;
        (nr, (nr * nr) as f64 * fact2 - 1.0)
    } else {
        (ns, (2 * ns - jr) as f64 * 2.0 / (3 * ns) as f64)
    };
    let mut tmp = JPLL[f] * nr + ix - iy;
    if tmp < 0 {
        tmp += 8 * nr;
    }
    let phi = FRAC_PI_4 * tmp as f64 / nr as f64;
    let r = (1.0 - z * z).max(0.0).sqrt();
    DVec3::new(r * phi.cos(), r * phi.sin(), z)
}

fn healpix(n_total: usize) -> Patches {
    let nside = ((n_total as f64) / 12.0).sqrt().round() as usize;
    let centre = move |f: usize, i: usize, j: usize| healpix_centre(nside, f, i, j);
    Patches::build(
        "healpix",
        Stencil::Square,
        12,
        nside,
        nside,
        &centre,
        GhostRule::Reflect,
    )
}

/// Ten icosahedral diamonds of n × n triangle-lattice points (poles left out).
fn icos(n_total: usize) -> Patches {
    let n = ((n_total as f64) / 10.0).sqrt().round() as usize;
    let lat_ring = (0.5f64).atan(); // 26.565°
    let ll = |lat: f64, lon: f64| DVec3::new(lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin());
    let pn = DVec3::Z;
    let ps = -DVec3::Z;
    let up: Vec<DVec3> = (0..5)
        .map(|k| ll(lat_ring, 2.0 * PI * k as f64 / 5.0))
        .collect();
    let lo: Vec<DVec3> = (0..5)
        .map(|k| ll(-lat_ring, 2.0 * PI * (k as f64 + 0.5) / 5.0))
        .collect();
    // Diamond corners (A top, B along s, C along t, D opposite).
    let mut corners = Vec::new();
    for k in 0..5 {
        corners.push((pn, up[k], up[(k + 1) % 5], lo[k]));
    }
    for k in 0..5 {
        corners.push((up[(k + 1) % 5], lo[k], lo[(k + 1) % 5], ps));
    }
    let centre = move |f: usize, i: usize, j: usize| {
        let (a, b, c, d) = corners[f];
        let s = (i as f64 + 1.0) / n as f64;
        let t = j as f64 / n as f64;
        let p = if s + t <= 1.0 {
            a + s * (b - a) + t * (c - a)
        } else {
            d + (1.0 - t) * (b - d) + (1.0 - s) * (c - d)
        };
        p.normalize()
    };
    Patches::build("icos", Stencil::Hex, 10, n, n, &centre, GhostRule::Reflect)
}

// ------------------------------------------------------------------------------------------------
// Mesh layouts: a neighbour list.

struct Mesh {
    name: String,
    pos: Vec<DVec3>,
    off: Vec<u32>,
    nbr: Vec<u32>,
    /// Inverse step length per edge (in mean spacings).
    inv: Vec<f64>,
    /// ELLPACK width and padded lists (empty unless built).
    ell_w: usize,
    ell_nbr: Vec<u32>,
}

impl Mesh {
    fn n(&self) -> usize {
        self.pos.len()
    }

    fn from_adjacency(name: &str, pos: Vec<DVec3>, rows: Vec<Vec<u32>>, dxm: f64) -> Mesh {
        let mut off = vec![0u32];
        let mut nbr = Vec::new();
        let mut inv = Vec::new();
        for (c, r) in rows.iter().enumerate() {
            for &k in r {
                nbr.push(k);
                inv.push(dxm / (pos[c] - pos[k as usize]).length());
            }
            off.push(nbr.len() as u32);
        }
        Mesh {
            name: name.to_string(),
            pos,
            off,
            nbr,
            inv,
            ell_w: 0,
            ell_nbr: Vec::new(),
        }
    }

    /// The same mesh with cells renumbered: `order[new] = old`.
    fn permuted(&self, name: &str, order: &[u32], dxm: f64) -> Mesh {
        let n = self.n();
        let mut new_of = vec![0u32; n];
        for (new, &old) in order.iter().enumerate() {
            new_of[old as usize] = new as u32;
        }
        let pos: Vec<DVec3> = order.iter().map(|&o| self.pos[o as usize]).collect();
        let rows: Vec<Vec<u32>> = order
            .iter()
            .map(|&o| {
                let o = o as usize;
                let mut r: Vec<u32> = self.nbr[self.off[o] as usize..self.off[o + 1] as usize]
                    .iter()
                    .map(|&k| new_of[k as usize])
                    .collect();
                r.sort_unstable();
                r
            })
            .collect();
        Mesh::from_adjacency(name, pos, rows, dxm)
    }

    fn with_ell(mut self, name: &str) -> Mesh {
        let n = self.n();
        let w = (0..n)
            .map(|c| (self.off[c + 1] - self.off[c]) as usize)
            .max()
            .unwrap_or(0);
        let mut ell = vec![0u32; n * w];
        for c in 0..n {
            let r = &self.nbr[self.off[c] as usize..self.off[c + 1] as usize];
            for k in 0..w {
                ell[c * w + k] = if k < r.len() { r[k] } else { c as u32 };
            }
        }
        self.name = name.to_string();
        self.ell_w = w;
        self.ell_nbr = ell;
        self
    }
}

/// Hilbert index of (x, y) on a 2^order square (the classic rotate-and-flip loop).
fn hilbert_d(order: u32, mut x: u32, mut y: u32) -> u64 {
    let n = 1u32 << order;
    let mut d = 0u64;
    let mut s = n >> 1;
    while s > 0 {
        let rx = u32::from(x & s > 0);
        let ry = u32::from(y & s > 0);
        d += u64::from(s) * u64::from(s) * u64::from((3 * rx) ^ ry);
        if ry == 0 {
            if rx == 1 {
                x = n - 1 - x;
                y = n - 1 - y;
            }
            std::mem::swap(&mut x, &mut y);
        }
        s >>= 1;
    }
    d
}

/// S2-style key: cube face, then the Hilbert index of the point's gnomonic coordinates on it.
fn hilbert_key(v: DVec3) -> u64 {
    let a = v.abs();
    let f = if a.x >= a.y && a.x >= a.z {
        if v.x > 0.0 { 0 } else { 2 }
    } else if a.y >= a.z {
        if v.y > 0.0 { 1 } else { 3 }
    } else if v.z > 0.0 {
        4
    } else {
        5
    };
    let [nn, u, w] = CUBE[f];
    let d = v.dot(v3(nn));
    let (pu, pw) = (v.dot(v3(u)) / d, v.dot(v3(w)) / d);
    let q = |t: f64| (((t + 1.0) * 0.5 * 65536.0).floor() as i64).clamp(0, 65535) as u32;
    ((f as u64) << 40) | hilbert_d(16, q(pu), q(pw))
}

// ------------------------------------------------------------------------------------------------
// Coefficients of the implicit diffusion operator A = I + Δt·L in the true metric.

struct PatchCoef {
    d: Vec<f64>,
    w: Vec<Vec<f64>>,
    /// Largest Σw over cells (the explicit step limit is 2/that, in units of the ideal grid's).
    max_sum: f64,
}

/// w = coef·s·(Δx̄/d)² per neighbour (coef 1 for the 5-point square stencil, 2/3 for the hexagonal
/// one: the finite-volume weight of a regular hexagon); zero where the neighbour is the cell itself
/// (Lambert's closed pole rows). `iso`: every weight s (block metric).
fn patch_coef(p: &Patches, s: f64, dxm: f64, iso: bool) -> PatchCoef {
    let m = p.m();
    let k = p.doff.len();
    let coef = if p.stencil == Stencil::Hex { 2.0 / 3.0 } else { 1.0 };
    // The neighbour each slot names, and how many of a cell's slots name each neighbour: a link
    // c–n keeps its weight only if n also names c, split over the slots that name it, so the
    // operator stays symmetric where the ghost mapping is not one-to-one (the cube's and
    // HEALPix's corners, the icosahedron's pentagons).
    let nb_of = |c: usize, kk: usize| p.canon[(c as isize + p.doff[kk]) as usize] as usize;
    let mult = |c: usize, n: usize| (0..k).filter(|&kk| nb_of(c, kk) == n).count();
    let mut w = vec![vec![0.0; m]; k];
    let mut d = vec![1.0; m];
    let mut max_sum: f64 = 0.0;
    for &c in &p.owned {
        let c = c as usize;
        let mut sum = 0.0;
        for kk in 0..k {
            let nb = nb_of(c, kk);
            let back = mult(nb, c);
            let x = if nb == c || back == 0 {
                0.0
            } else if iso {
                s / mult(c, nb) as f64
            } else {
                let dd = (p.pos[c] - p.pos[nb]).length() / dxm;
                coef * s / (dd * dd) / mult(c, nb) as f64
            };
            w[kk][c] = x;
            sum += x;
        }
        d[c] = 1.0 + sum;
        max_sum = max_sum.max(sum / s);
    }
    PatchCoef { d, w, max_sum }
}

struct MeshCoef {
    d: Vec<f64>,
    w: Vec<f64>,
    ell_w: Vec<f64>,
    max_sum: f64,
}

fn mesh_coef(m: &Mesh, s: f64, dxm: f64) -> MeshCoef {
    let n = m.n();
    let mut d = vec![1.0; n];
    let mut w = vec![0.0; m.nbr.len()];
    let mut max_sum: f64 = 0.0;
    for c in 0..n {
        let mut sum = 0.0;
        for e in m.off[c] as usize..m.off[c + 1] as usize {
            let dd = (m.pos[c] - m.pos[m.nbr[e] as usize]).length() / dxm;
            w[e] = (2.0 / 3.0) * s / (dd * dd);
            sum += w[e];
        }
        d[c] = 1.0 + sum;
        max_sum = max_sum.max(sum / s);
    }
    let mut ell_w = Vec::new();
    if m.ell_w > 0 {
        ell_w = vec![0.0; n * m.ell_w];
        for c in 0..n {
            let r0 = m.off[c] as usize;
            let len = m.off[c + 1] as usize - r0;
            for k in 0..len {
                ell_w[c * m.ell_w + k] = w[r0 + k];
            }
        }
    }
    MeshCoef {
        d,
        w,
        ell_w,
        max_sum,
    }
}

// ------------------------------------------------------------------------------------------------
// Kernels.

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn row4(o: &mut [f64], hc: &[f64], d: &[f64], h: [&[f64]; 4], w: [&[f64]; 4]) {
    let n = o.len();
    let (hc, d) = (&hc[..n], &d[..n]);
    let (h0, h1, h2, h3) = (&h[0][..n], &h[1][..n], &h[2][..n], &h[3][..n]);
    let (w0, w1, w2, w3) = (&w[0][..n], &w[1][..n], &w[2][..n], &w[3][..n]);
    for i in 0..n {
        o[i] = d[i] * hc[i] - (w0[i] * h0[i] + w1[i] * h1[i] + w2[i] * h2[i] + w3[i] * h3[i]);
    }
}

#[inline(always)]
fn row6(o: &mut [f64], hc: &[f64], d: &[f64], h: [&[f64]; 6], w: [&[f64]; 6]) {
    let n = o.len();
    let (hc, d) = (&hc[..n], &d[..n]);
    let (h0, h1, h2, h3, h4, h5) = (
        &h[0][..n], &h[1][..n], &h[2][..n], &h[3][..n], &h[4][..n], &h[5][..n],
    );
    let (w0, w1, w2, w3, w4, w5) = (
        &w[0][..n], &w[1][..n], &w[2][..n], &w[3][..n], &w[4][..n], &w[5][..n],
    );
    for i in 0..n {
        o[i] = d[i] * hc[i]
            - (w0[i] * h0[i]
                + w1[i] * h1[i]
                + w2[i] * h2[i]
                + w3[i] * h3[i]
                + w4[i] * h4[i]
                + w5[i] * h5[i]);
    }
}

/// out = A·h on the owned cells (ghosts of h must be filled).
fn patch_apply(p: &Patches, cf: &PatchCoef, h: &[f64], out: &mut [f64]) {
    let pw = p.pw;
    let nw = p.nw;
    out.par_chunks_mut(pw).enumerate().for_each(|(r, row)| {
        if !p.interior_row(r) {
            return;
        }
        let b = r * pw + 1;
        let o = &mut row[1..=nw];
        let sl = |off: isize| &h[(b as isize + off) as usize..(b as isize + off) as usize + nw];
        let ws = |k: usize| &cf.w[k][b..b + nw];
        match p.doff.len() {
            4 => row4(
                o,
                &h[b..b + nw],
                &cf.d[b..b + nw],
                [sl(p.doff[0]), sl(p.doff[1]), sl(p.doff[2]), sl(p.doff[3])],
                [ws(0), ws(1), ws(2), ws(3)],
            ),
            _ => row6(
                o,
                &h[b..b + nw],
                &cf.d[b..b + nw],
                [
                    sl(p.doff[0]),
                    sl(p.doff[1]),
                    sl(p.doff[2]),
                    sl(p.doff[3]),
                    sl(p.doff[4]),
                    sl(p.doff[5]),
                ],
                [ws(0), ws(1), ws(2), ws(3), ws(4), ws(5)],
            ),
        }
    });
}

const CHUNK: usize = 2048;

fn mesh_apply(m: &Mesh, cf: &MeshCoef, h: &[f64], out: &mut [f64]) {
    out.par_chunks_mut(CHUNK).enumerate().for_each(|(b, o)| {
        for (k, oc) in o.iter_mut().enumerate() {
            let c = b * CHUNK + k;
            let mut s = 0.0;
            for e in m.off[c] as usize..m.off[c + 1] as usize {
                s += cf.w[e] * h[m.nbr[e] as usize];
            }
            *oc = cf.d[c] * h[c] - s;
        }
    });
}

fn mesh_apply_ell(m: &Mesh, cf: &MeshCoef, h: &[f64], out: &mut [f64]) {
    let w = m.ell_w;
    out.par_chunks_mut(CHUNK).enumerate().for_each(|(b, o)| {
        for (k, oc) in o.iter_mut().enumerate() {
            let c = b * CHUNK + k;
            let nb = &m.ell_nbr[c * w..c * w + w];
            let ww = &cf.ell_w[c * w..c * w + w];
            let mut s = 0.0;
            for j in 0..w {
                s += ww[j] * h[nb[j] as usize];
            }
            *oc = cf.d[c] * h[c] - s;
        }
    });
}

fn patch_receivers(p: &Patches, h: &[f64], rcv: &mut [u32]) {
    let pw = p.pw;
    let nw = p.nw;
    rcv.par_chunks_mut(pw).enumerate().for_each(|(r, row)| {
        if !p.interior_row(r) {
            return;
        }
        let base = r * pw;
        for pi in 1..=nw {
            let c = base + pi;
            let hc = h[c];
            let mut best = c;
            let mut bs = 0.0;
            for (k, &o) in p.roff.iter().enumerate() {
                let n = (c as isize + o) as usize;
                let s = (hc - h[n]) * p.rinv[k];
                if s > bs {
                    bs = s;
                    best = n;
                }
            }
            row[pi] = p.canon[best];
        }
    });
}

fn mesh_receivers(m: &Mesh, h: &[f64], rcv: &mut [u32]) {
    rcv.par_chunks_mut(CHUNK).enumerate().for_each(|(b, o)| {
        for (k, r) in o.iter_mut().enumerate() {
            let c = b * CHUNK + k;
            let hc = h[c];
            let mut best = c as u32;
            let mut bs = 0.0;
            for e in m.off[c] as usize..m.off[c + 1] as usize {
                let n = m.nbr[e];
                let s = (hc - h[n as usize]) * m.inv[e];
                if s > bs {
                    bs = s;
                    best = n;
                }
            }
            *r = best;
        }
    });
}

struct StackScratch {
    cnt: Vec<u32>,
    doff: Vec<u32>,
    don: Vec<u32>,
    st: Vec<u32>,
    stack: Vec<u32>,
}

impl StackScratch {
    fn new(m: usize, n: usize) -> StackScratch {
        StackScratch {
            cnt: vec![0; m + 1],
            doff: vec![0; m + 1],
            don: vec![0; n],
            st: Vec::with_capacity(n),
            stack: Vec::with_capacity(n),
        }
    }
}

/// Donor lists and the Braun–Willett stack over the cells `cells` (indices into an index space of
/// size m). Returns the number of roots.
fn build_stack(cells: &[u32], rcv: &[u32], s: &mut StackScratch) -> usize {
    s.cnt.fill(0);
    for &c in cells {
        let r = rcv[c as usize];
        if r != c {
            s.cnt[r as usize] += 1;
        }
    }
    let mut acc = 0u32;
    for (i, &k) in s.cnt.iter().enumerate() {
        s.doff[i] = acc;
        acc += k;
    }
    // Reuse cnt as the fill cursor.
    s.cnt.copy_from_slice(&s.doff);
    for &c in cells {
        let r = rcv[c as usize];
        if r != c {
            let p = &mut s.cnt[r as usize];
            s.don[*p as usize] = c;
            *p += 1;
        }
    }
    s.stack.clear();
    let mut roots = 0;
    for &c in cells {
        if rcv[c as usize] != c {
            continue;
        }
        roots += 1;
        s.st.push(c);
        while let Some(x) = s.st.pop() {
            s.stack.push(x);
            let x = x as usize;
            let (a, b) = (s.doff[x] as usize, s.doff[x + 1] as usize);
            s.st.extend_from_slice(&s.don[a..b]);
        }
    }
    roots
}

fn accumulate(stack: &[u32], rcv: &[u32], a: &mut [f64]) {
    for &c in stack {
        a[c as usize] = 1.0;
    }
    for &c in stack.iter().rev() {
        let r = rcv[c as usize];
        if r != c {
            a[r as usize] += a[c as usize];
        }
    }
}

// ------------------------------------------------------------------------------------------------
// Conjugate gradients (Jacobi-preconditioned), generic over the operator.

fn pcg(
    n: usize,
    b: &[f64],
    d: &[f64],
    mask: &[bool],
    mut apply: impl FnMut(&mut [f64], &mut [f64]),
    tol: f64,
    max_it: usize,
) -> usize {
    let mut x = b.to_vec();
    let mut ax = vec![0.0; n];
    apply(&mut x, &mut ax);
    let mut r: Vec<f64> = (0..n)
        .map(|i| if mask[i] { b[i] - ax[i] } else { 0.0 })
        .collect();
    let bn: f64 = (0..n)
        .filter(|&i| mask[i])
        .map(|i| b[i] * b[i])
        .sum::<f64>()
        .sqrt();
    let mut z: Vec<f64> = (0..n).map(|i| r[i] / d[i]).collect();
    let mut p = z.clone();
    let mut rz: f64 = (0..n).map(|i| r[i] * z[i]).sum();
    let mut ap = vec![0.0; n];
    for it in 1..=max_it {
        apply(&mut p, &mut ap);
        let pap: f64 = (0..n).filter(|&i| mask[i]).map(|i| p[i] * ap[i]).sum();
        let alpha = rz / pap;
        let mut rr = 0.0;
        for i in 0..n {
            if mask[i] {
                x[i] += alpha * p[i];
                r[i] -= alpha * ap[i];
                rr += r[i] * r[i];
            }
        }
        if rr.sqrt() <= tol * bn {
            black_box(&x);
            return it;
        }
        for i in 0..n {
            z[i] = r[i] / d[i];
        }
        let rz2: f64 = (0..n).map(|i| r[i] * z[i]).sum();
        let beta = rz2 / rz;
        rz = rz2;
        for i in 0..n {
            p[i] = z[i] + beta * p[i];
        }
    }
    max_it
}

// ------------------------------------------------------------------------------------------------
// Timing.

fn median_ms(reps: usize, mut f: impl FnMut()) -> f64 {
    for _ in 0..2 {
        f();
    }
    let mut t: Vec<f64> = (0..reps)
        .map(|_| {
            let s = Instant::now();
            f();
            s.elapsed().as_secs_f64() * 1e3
        })
        .collect();
    t.sort_by(f64::total_cmp);
    t[t.len() / 2]
}

struct Row {
    layout: String,
    cells: usize,
    threads: usize,
    diff_ms: f64,
    rcv_ms: f64,
    stack_ms: f64,
    acc_ms: f64,
    roots: usize,
}

fn print_row(r: &Row) {
    let ns = |ms: f64| ms * 1e6 / r.cells as f64;
    println!(
        "| {} | {} | {} | {:.3} | {:.2} | {:.3} | {:.2} | {:.3} | {:.2} | {:.3} | {:.2} | {} |",
        r.layout,
        r.cells,
        r.threads,
        r.diff_ms,
        ns(r.diff_ms),
        r.rcv_ms,
        ns(r.rcv_ms),
        r.stack_ms,
        ns(r.stack_ms),
        r.acc_ms,
        ns(r.acc_ms),
        r.roots
    );
}

fn bench_patches(p: &Patches, t: &Terrain, dxm: f64, threads: usize, reps: usize) -> Row {
    let m = p.m();
    let mut h = vec![0.0; m];
    for &c in &p.owned {
        h[c as usize] = t.at(p.pos[c as usize]);
    }
    p.fill(&mut h);
    let cf = patch_coef(p, 1.0, dxm, false);
    let mut out = vec![0.0; m];
    let diff_ms = median_ms(reps, || {
        p.fill(&mut h);
        patch_apply(p, &cf, &h, &mut out);
        black_box(&out);
    });
    let mut rcv: Vec<u32> = (0..m as u32).collect();
    let rcv_ms = median_ms(reps, || {
        patch_receivers(p, &h, &mut rcv);
        black_box(&rcv);
    });
    let mut s = StackScratch::new(m, p.owned.len());
    let mut roots = 0;
    let stack_ms = median_ms(reps.min(15), || {
        roots = build_stack(&p.owned, &rcv, &mut s);
    });
    let mut a = vec![0.0; m];
    let acc_ms = median_ms(reps.min(15), || {
        accumulate(&s.stack, &rcv, &mut a);
        black_box(&a);
    });
    Row {
        layout: p.name.to_string(),
        cells: p.owned.len(),
        threads,
        diff_ms,
        rcv_ms,
        stack_ms,
        acc_ms,
        roots,
    }
}

fn bench_mesh(mm: &Mesh, t: &Terrain, dxm: f64, threads: usize, reps: usize) -> Row {
    let n = mm.n();
    let h: Vec<f64> = mm.pos.iter().map(|&v| t.at(v)).collect();
    let cf = mesh_coef(mm, 1.0, dxm);
    let mut out = vec![0.0; n];
    let diff_ms = if mm.ell_w > 0 {
        median_ms(reps, || {
            mesh_apply_ell(mm, &cf, &h, &mut out);
            black_box(&out);
        })
    } else {
        median_ms(reps, || {
            mesh_apply(mm, &cf, &h, &mut out);
            black_box(&out);
        })
    };
    let mut rcv = vec![0u32; n];
    let rcv_ms = median_ms(reps, || {
        mesh_receivers(mm, &h, &mut rcv);
        black_box(&rcv);
    });
    let cells: Vec<u32> = (0..n as u32).collect();
    let mut s = StackScratch::new(n, n);
    let mut roots = 0;
    let stack_ms = median_ms(reps.min(15), || {
        roots = build_stack(&cells, &rcv, &mut s);
    });
    let mut a = vec![0.0; n];
    let acc_ms = median_ms(reps.min(15), || {
        accumulate(&s.stack, &rcv, &mut a);
        black_box(&a);
    });
    Row {
        layout: mm.name.clone(),
        cells: n,
        threads,
        diff_ms,
        rcv_ms,
        stack_ms,
        acc_ms,
        roots,
    }
}

/// Geometry report: cell anisotropy (longest over shortest neighbour step per cell; max, p99) and
/// the explicit diffusion step limit against an ideal uniform square grid of the same cell count.
fn geometry_patches(p: &Patches, dxm: f64) -> (f64, f64, f64, f64) {
    let z80 = 80f64.to_radians().sin();
    let mut an80 = 0f64;
    let mut an: Vec<f64> = p
        .owned
        .iter()
        .map(|&c| {
            let c = c as usize;
            let ds: Vec<f64> = p
                .doff
                .iter()
                .map(|&o| p.canon[(c as isize + o) as usize] as usize)
                .filter(|&n| n != c)
                .map(|n| (p.pos[c] - p.pos[n]).length())
                .collect();
            let mx = ds.iter().cloned().fold(0.0, f64::max);
            let mn = ds.iter().cloned().fold(f64::INFINITY, f64::min);
            if p.pos[c].z.abs() <= z80 {
                an80 = an80.max(mx / mn);
            }
            mx / mn
        })
        .collect();
    an.sort_by(f64::total_cmp);
    let cf = patch_coef(p, 1.0, dxm, false);
    (
        *an.last().unwrap(),
        an80,
        an[an.len() * 99 / 100],
        4.0 / cf.max_sum,
    )
}

fn geometry_mesh(m: &Mesh, dxm: f64) -> (f64, f64, f64) {
    let mut an: Vec<f64> = (0..m.n())
        .map(|c| {
            let ds: Vec<f64> = (m.off[c] as usize..m.off[c + 1] as usize)
                .map(|e| (m.pos[c] - m.pos[m.nbr[e] as usize]).length())
                .collect();
            let mx = ds.iter().cloned().fold(0.0, f64::max);
            let mn = ds.iter().cloned().fold(f64::INFINITY, f64::min);
            mx / mn
        })
        .collect();
    an.sort_by(f64::total_cmp);
    let cf = mesh_coef(m, 1.0, dxm);
    (
        *an.last().unwrap(),
        an[an.len() * 99 / 100],
        4.0 / cf.max_sum,
    )
}

fn cg_patches(p: &Patches, t: &Terrain, dxm: f64, s: f64, iso: bool) -> (usize, f64) {
    let m = p.m();
    let mut b = vec![0.0; m];
    let mut mask = vec![false; m];
    for &c in &p.owned {
        b[c as usize] = t.at(p.pos[c as usize]);
        mask[c as usize] = true;
    }
    let cf = patch_coef(p, s, dxm, iso);
    let st = Instant::now();
    let it = pcg(
        m,
        &b,
        &cf.d,
        &mask,
        |x, out| {
            p.fill(x);
            patch_apply(p, &cf, x, out);
        },
        1e-8,
        5000,
    );
    (it, st.elapsed().as_secs_f64() * 1e3)
}

fn cg_mesh(mm: &Mesh, t: &Terrain, dxm: f64, s: f64) -> (usize, f64) {
    let n = mm.n();
    let b: Vec<f64> = mm.pos.iter().map(|&v| t.at(v)).collect();
    let mask = vec![true; n];
    let cf = mesh_coef(mm, s, dxm);
    let st = Instant::now();
    let it = pcg(
        n,
        &b,
        &cf.d,
        &mask,
        |x, out| mesh_apply(mm, &cf, x, out),
        1e-8,
        5000,
    );
    (it, st.elapsed().as_secs_f64() * 1e3)
}

/// Time to resample a field from a sphere grid (positions `src`) onto the Lambert raster's cells:
/// four nearest source cells, inverse-distance weights, precomputed; the timed part is the gather.
fn remap_ms(src: &[DVec3], vals: &[f64], dst: &[DVec3], reps: usize) -> f64 {
    let tree = SampleTree::new(src);
    let taps: Vec<([u32; 4], [f64; 4])> = dst
        .par_iter()
        .map(|&q| {
            let nn = tree.nearest_n(q, 4);
            let mut idx = [0u32; 4];
            let mut w = [0.0; 4];
            let mut sw = 0.0;
            for (k, &(i, d2)) in nn.iter().enumerate() {
                idx[k] = i;
                w[k] = 1.0 / (d2.sqrt() + 1e-12);
                sw += w[k];
            }
            for x in &mut w {
                *x /= sw;
            }
            (idx, w)
        })
        .collect();
    let mut out = vec![0.0; dst.len()];
    median_ms(reps, || {
        out.par_chunks_mut(CHUNK).enumerate().for_each(|(b, o)| {
            for (k, oc) in o.iter_mut().enumerate() {
                let (i, w) = &taps[b * CHUNK + k];
                *oc = w[0] * vals[i[0] as usize]
                    + w[1] * vals[i[1] as usize]
                    + w[2] * vals[i[2] as usize]
                    + w[3] * vals[i[3] as usize];
            }
        });
        black_box(&out);
    })
}

fn main() {
    let args: Vec<usize> = std::env::args()
        .skip(1)
        .map(|a| a.parse().expect("numeric argument"))
        .collect();
    let n = args.first().copied().unwrap_or(333_824);
    let threads = args.get(1).copied().unwrap_or(8);
    let reps = args.get(2).copied().unwrap_or(40);
    let dxm = (4.0 * PI / n as f64).sqrt();
    let t = Terrain::new();
    let build = Instant::now();

    let lam = lambert(n);
    let cub = cube(n);
    let hpx = healpix(n);
    let ico = icos(n);
    let patches = [&lam, &cub, &hpx, &ico];

    let fib_pts = fibonacci_sphere(lam.owned.len());
    let mesh = SphereMesh::build(&fib_pts).expect("hull");
    let adj = mesh.vertex_neighbors();
    let rows: Vec<Vec<u32>> = (0..fib_pts.len() as u32)
        .map(|v| adj.of(v).to_vec())
        .collect();
    let fib_plain = Mesh::from_adjacency("fib-plain", fib_pts.clone(), rows, dxm);
    let mut order: Vec<u32> = (0..fib_plain.n() as u32).collect();
    order.sort_by_key(|&i| hilbert_key(fib_plain.pos[i as usize]));
    let fib_hilbert = fib_plain.permuted("fib-hilbert", &order, dxm);
    let mut rnd: Vec<u32> = (0..fib_plain.n() as u32).collect();
    SplitMix64::new(7).shuffle(&mut rnd);
    let fib_random = fib_plain.permuted("fib-random", &rnd, dxm);
    let fib_ell = fib_plain
        .permuted("x", &order, dxm)
        .with_ell("fib-hilbert-ell");
    let meshes = [&fib_plain, &fib_hilbert, &fib_random, &fib_ell];
    eprintln!("built layouts in {:.1} s", build.elapsed().as_secs_f64());

    println!("# sphere-grid-bench: target {n} cells, threads 1 and {threads}, {reps} reps (median)");
    println!();
    println!("## Layouts");
    println!();
    println!(
        "| layout | cells | padded | ghost src/inward (median, max) | anisotropy max | max within 80° | p99 | explicit Δt vs ideal |"
    );
    println!("|---|---|---|---|---|---|---|---|");
    for p in patches {
        let (amax, a80, a99, dt) = geometry_patches(p, dxm);
        println!(
            "| {} | {} | {} | {:.3}, {:.3} | {:.2} | {:.2} | {:.2} | {:.3} |",
            p.name,
            p.owned.len(),
            p.m(),
            p.ghost_check.0,
            p.ghost_check.1,
            amax,
            a80,
            a99,
            dt
        );
    }
    for m in meshes {
        let (amax, a99, dt) = geometry_mesh(m, dxm);
        let deg_max = (0..m.n())
            .map(|c| m.off[c + 1] - m.off[c])
            .max()
            .unwrap();
        println!(
            "| {} | {} | {} (max degree {}) | - | {:.2} | {:.2} | {:.2} | {:.3} |",
            m.name,
            m.n(),
            m.nbr.len(),
            deg_max,
            amax,
            amax,
            a99,
            dt
        );
    }
    println!();
    println!("## Kernels (ms per pass and ns per cell)");
    println!();
    println!(
        "| layout | cells | threads | diff ms | diff ns/cell | rcv ms | rcv ns/cell | stack ms | stack ns/cell | acc ms | acc ns/cell | roots |"
    );
    println!("|---|---|---|---|---|---|---|---|---|---|---|---|");
    for th in [1, threads] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(th)
            .build()
            .unwrap();
        pool.install(|| {
            for p in patches {
                print_row(&bench_patches(p, &t, dxm, th, reps));
            }
            for m in meshes {
                print_row(&bench_mesh(m, &t, dxm, th, reps));
            }
        });
    }

    println!();
    println!(
        "## Implicit diffusion solve (Jacobi PCG to 1e-8, true metric unless noted), 1 thread"
    );
    println!();
    println!("| layout | D·Δt/Δx̄² | iterations | ms |");
    println!("|---|---|---|---|");
    let pool1 = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap();
    pool1.install(|| {
        for s in [1.0, 30.0] {
            let (it, ms) = cg_patches(&lam, &t, dxm, s, true);
            println!("| lambert (block metric, as shipped) | {s} | {it} | {ms:.1} |");
            for p in patches {
                let (it, ms) = cg_patches(p, &t, dxm, s, false);
                println!("| {} | {s} | {it} | {ms:.1} |", p.name);
            }
            for m in [&fib_plain, &fib_hilbert] {
                let (it, ms) = cg_mesh(m, &t, dxm, s);
                println!("| {} | {s} | {it} | {ms:.1} |", m.name);
            }
        }
    });

    println!();
    println!("## Resample to the Lambert raster (4-point gather per raster cell)");
    println!();
    println!("| from | threads | ms |");
    println!("|---|---|---|");
    let dst: Vec<DVec3> = lam.owned.iter().map(|&c| lam.pos[c as usize]).collect();
    for th in [1, threads] {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(th)
            .build()
            .unwrap();
        pool.install(|| {
            for p in [&cub, &ico] {
                let src: Vec<DVec3> = p.owned.iter().map(|&c| p.pos[c as usize]).collect();
                let vals: Vec<f64> = src.iter().map(|&v| t.at(v)).collect();
                println!("| {} | {th} | {:.3} |", p.name, remap_ms(&src, &vals, &dst, reps));
            }
            let vals: Vec<f64> = fib_hilbert.pos.iter().map(|&v| t.at(v)).collect();
            println!(
                "| fib-hilbert | {th} | {:.3} |",
                remap_ms(&fib_hilbert.pos, &vals, &dst, reps)
            );
        });
    }
}
