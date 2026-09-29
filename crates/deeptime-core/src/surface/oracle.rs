// SPDX-License-Identifier: LGPL-3.0-or-later
//! The surface kernels on a plain raster, for the oracle comparison with fastscapelib
//! (`tools/validation/fastscape_compare.py`, plan §8.2 "Oracle", §9.2): a W × H grid of `dx`
//! metres, periodic in x (as L1 wraps), with the first and last rows held at 0 m as the base level
//! (fastscapelib's `FIXED_VALUE` boundary), uniform uplift, the implicit stream power with the
//! same K, m, n and time step, and optional linear diffusion. The start is seeded noise of 0–1 m,
//! written out so the oracle can start from the same surface.

use std::time::Instant;

use super::grid::Dims;
use super::pflood::{BASE, LAND};
use super::{diffusion, pflood, routing, spl};
use crate::sphere::SplitMix64;

/// What to run.
#[derive(Clone, Debug)]
pub struct RasterParams {
    pub w: usize,
    pub h: usize,
    pub dx: f64,
    pub steps: usize,
    pub dt_yr: f64,
    pub k: f64,
    pub m: f64,
    pub n: f64,
    pub g: f64,
    pub uplift_m_yr: f64,
    pub kd_m2_yr: f64,
    pub seed: u64,
}

/// The run: the initial and final surfaces and the final drainage area (row-major), and time.
pub struct RasterRun {
    pub initial: Vec<f64>,
    pub elevation: Vec<f64>,
    pub area_m2: Vec<f64>,
    pub seconds: f64,
}

/// Runs the stream power (and diffusion) on the raster.
pub fn run(p: &RasterParams) -> RasterRun {
    let d = Dims::new(p.w, p.h, p.dx, true);
    let n = d.cells();
    let mut rng = SplitMix64::new(p.seed);
    let base_row = |c: usize| c / p.w == 0 || c / p.w == p.h - 1;
    let mut z: Vec<f64> = (0..n)
        .map(|c| if base_row(c) { 0.0 } else { rng.next_f64() })
        .collect();
    let initial = z.clone();
    let base_kind: Vec<u8> = (0..n)
        .map(|c| if base_row(c) { BASE } else { LAND })
        .collect();
    let base_z = vec![0.0; n];
    let kf = vec![p.k; n];
    let kd = vec![p.kd_m2_yr; n];
    let sp = spl::SplParams {
        m: p.m,
        n: p.n,
        g: p.g,
        dt: p.dt_yr,
        sc: f64::INFINITY,
        max_iter: 10,
        tol_m: 1e-3,
    };
    let mut filled = vec![0.0; n];
    let mut r = routing::Routing::default();
    let mut area = Vec::new();
    let mut out = vec![0.0; n];
    let mut qin = vec![0.0; n];
    let ones = vec![1.0f32; n];
    let started = Instant::now();
    for _ in 0..p.steps {
        let h0: Vec<f64> = (0..n)
            .map(|c| {
                if base_row(c) {
                    0.0
                } else {
                    z[c] + p.uplift_m_yr * p.dt_yr
                }
            })
            .collect();
        let mut kind = base_kind.clone();
        pflood::flood(&d, &h0, &base_z, &mut kind, &mut filled);
        routing::receivers(&d, &filled, &kind, &mut r);
        routing::stack(&d, &kind, &mut r);
        routing::accumulate(&d, &r, &ones, &mut area);
        let input = spl::SplInput {
            kind: &kind,
            h0: &h0,
            filled: &filled,
            base_z: &base_z,
            area: &area,
            kf: &kf,
        };
        spl::solve(&d, &r, &input, &sp, &mut out, &mut qin);
        z.copy_from_slice(&out);
        if p.kd_m2_yr > 0.0 {
            diffusion::step(&d, &mut z, &kd, p.dt_yr);
            for (c, v) in z.iter_mut().enumerate() {
                if base_row(c) {
                    *v = 0.0;
                }
            }
        }
    }
    RasterRun {
        initial,
        elevation: z,
        area_m2: area,
        seconds: started.elapsed().as_secs_f64(),
    }
}
