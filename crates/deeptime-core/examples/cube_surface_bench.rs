// SPDX-License-Identifier: LGPL-3.0-or-later
//! The cube grid's surface solvers against the Lambert raster's at the same cell count (cube world
//! plan §3, S2's speed check; the plan's research bench measured routing at 1.0 times and
//! diffusion at 1.0 to 1.3 times). A synthetic closed planet (a sum of plane waves on the sphere,
//! the same function on both grids) runs the surface's loop with quick 16k's settings: the open
//! sea and the flood, the routing, the stream-power solve, the mouth fill, the multigrid diffusion
//! (and, timed on a copy, the Jacobi one), the slope failure, the spreading and the sea level.
//!
//! ```text
//! cargo run --release -p deeptime-core --example cube_surface_bench -- [--threads N] [--steps K] [--n N] [--repeat K]
//! ```
//! Defaults: 8 threads (the speed gate's), 12 steps of which the first 2 are not timed, n = 240
//! (6n² = 345,600 cells, quick 16k's L1 on the cube) against a 1,024-wide Lambert raster with as
//! many rows as make the same count (338: 346,112 cells), 3 repeats run cube, Lambert, cube, ...
//! side by side in time (each phase's fastest repeat is reported, so a busy moment of a shared
//! machine does not land on one grid only).

use std::collections::BTreeMap;
use std::time::Instant;

use deeptime_core::sea_level;
use deeptime_core::sphere::SplitMix64;
use deeptime_core::surface::cube::{self, CubeGrid};
use deeptime_core::surface::diffusion;
use deeptime_core::surface::grid::Dims;
use deeptime_core::surface::marine::{self, FailureParams, FillParams, FillScratch};
use deeptime_core::surface::params::{DiffusionSolver, FloodMethod};
use deeptime_core::surface::pflood::{self, BASE, LAND};
use deeptime_core::surface::routing::{self, Routing};
use deeptime_core::surface::spl::{self, SplInput, SplParams};
use deeptime_core::surface::spread;

const R: f64 = 6.371e6;
type V3 = [f64; 3];

/// The two grids and their diffusion workspaces.
// A bench builds each grid once, so the variants' sizes do not matter.
#[allow(clippy::large_enum_variant)]
enum Grid {
    Cube(CubeGrid, cube::Workspace),
    Lambert(Dims, diffusion::Workspace),
}

/// A smooth random planet at unit vector `u`: a sum of 24 plane waves, metres.
fn terrain(waves: &[(V3, f64, f64, f64)], u: V3) -> f64 {
    let s: f64 = waves
        .iter()
        .map(|&(d, k, ph, amp)| {
            let x = u[0] * d[0] + u[1] * d[1] + u[2] * d[2];
            amp * (k * std::f64::consts::PI * x + ph).sin()
        })
        .sum();
    1500.0 * s - 800.0
}

fn waves(seed: u64) -> Vec<(V3, f64, f64, f64)> {
    let mut rng = SplitMix64::new(seed);
    let mut uni = || (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
    (0..24)
        .map(|_| {
            let d = [uni() - 0.5, uni() - 0.5, uni() - 0.5];
            let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let k = 1.0 + 5.0 * uni();
            (
                [d[0] / l, d[1] / l, d[2] / l],
                k,
                std::f64::consts::TAU * uni(),
                1.0 / k,
            )
        })
        .collect()
}

/// Milliseconds per timed step of each phase.
type Times = BTreeMap<&'static str, f64>;

fn run(grid: &mut Grid, h0: Vec<f64>, steps: usize, warm: usize) -> Times {
    let n = h0.len();
    let mut h = h0;
    let sea = 0.0;
    // quick 16k's surface (presets/earthlike_quick_16k.json): a 2-Myr step, one substep.
    let dt = 2.0e6;
    let sp = SplParams {
        m: 1.0,
        n: 2.0,
        g: 1.0,
        dt,
        sc: 0.15,
        max_iter: 6,
        tol_m: 0.01,
    };
    let fp = FillParams {
        sea_level: sea,
        fill_depth_m: 0.0,
        shelf_slope: 0.0005,
        max_cells: 200_000,
    };
    let fail = FailureParams {
        critical_slope: 2.5f64.to_radians().tan(),
        rate: marine::FAILURE_RATE,
        iterations: marine::FAILURE_ITERATIONS,
    };
    let kf = vec![1.5e-9; n];
    let weight = vec![0.5f32; n];
    let base_z = vec![sea; n];
    let (mut kind, mut filled, mut area) = (vec![LAND; n], vec![0.0; n], Vec::new());
    let (mut out, mut qin) = (vec![0.0; n], vec![0.0; n]);
    let mut r = Routing::default();
    let mut scratch = FillScratch::default();
    let mut dep = vec![0.0f32; n];
    let mut avail = vec![1.0e9; n];
    let row_area: Vec<f64> = match grid {
        Grid::Lambert(d, _) => vec![1.0; d.h],
        Grid::Cube(..) => Vec::new(),
    };
    let mut t = Times::new();
    let lap = |t: &mut Times, name: &'static str, s: Instant, timed: bool| {
        if timed {
            *t.entry(name).or_insert(0.0) += s.elapsed().as_secs_f64() * 1000.0;
        }
    };
    let (mut mg_its, mut jac_its) = (0usize, 0usize);
    for step in 0..steps {
        let timed = step >= warm;
        let s = Instant::now();
        match grid {
            Grid::Cube(g, _) => {
                pflood::open_sea_with(FloodMethod::Parallel, g, &h, sea, n / 500, &mut kind);
                pflood::flood_with(
                    FloodMethod::Parallel,
                    g,
                    &h,
                    &base_z,
                    &mut kind,
                    &mut filled,
                );
            }
            Grid::Lambert(d, _) => {
                pflood::open_sea_with(FloodMethod::Parallel, d, &h, sea, n / 500, &mut kind);
                pflood::flood_with(
                    FloodMethod::Parallel,
                    d,
                    &h,
                    &base_z,
                    &mut kind,
                    &mut filled,
                );
            }
        }
        lap(&mut t, "1 flood", s, timed);
        let s = Instant::now();
        match grid {
            Grid::Cube(g, _) => {
                routing::receivers(g, &filled, &kind, &mut r);
                routing::stack(g, &kind, &mut r);
                routing::accumulate(g, &r, &weight, &mut area);
            }
            Grid::Lambert(d, _) => {
                routing::receivers(d, &filled, &kind, &mut r);
                routing::stack(d, &kind, &mut r);
                routing::accumulate(d, &r, &weight, &mut area);
            }
        }
        lap(&mut t, "2 routing", s, timed);
        let s = Instant::now();
        let input = SplInput {
            kind: &kind,
            h0: &h,
            filled: &filled,
            base_z: &base_z,
            area: &area,
            kf: &kf,
        };
        match grid {
            Grid::Cube(g, _) => spl::solve(g, &r, &input, &sp, &mut out, &mut qin),
            Grid::Lambert(d, _) => spl::solve(d, &r, &input, &sp, &mut out, &mut qin),
        };
        for c in 0..n {
            if kind[c] != BASE {
                h[c] = out[c];
            }
        }
        lap(&mut t, "3 spl", s, timed);
        let s = Instant::now();
        let sea_mask: Vec<bool> = kind.iter().map(|&k| k == BASE).collect();
        // The mouths and the cells their fills reached (printed for the last step).
        let (mut mouths, mut fill_cells) = (0usize, 0usize);
        for m in 0..n {
            if kind[m] == BASE && qin[m] > 0.0 {
                let vol = qin[m] * dt;
                mouths += 1;
                fill_cells += match grid {
                    Grid::Cube(g, _) => {
                        marine::fill(g, &sea_mask, m, vol, &fp, &mut h, &mut dep, &mut scratch)
                    }
                    Grid::Lambert(d, _) => {
                        marine::fill(d, &sea_mask, m, vol, &fp, &mut h, &mut dep, &mut scratch)
                    }
                };
            }
        }
        lap(&mut t, "4 mouth fill", s, timed);
        let kd: Vec<f64> = h
            .iter()
            .map(|&z| marine::submarine_kd(sea - z, 100.0, 60.0, 2.0, 2.0))
            .collect();
        // The Jacobi solve, timed on a copy.
        let mut hj = h.clone();
        let s = Instant::now();
        jac_its += match grid {
            Grid::Cube(g, ws) => {
                cube::solve(ws, g, &mut hj, &kd, dt, DiffusionSolver::Jacobi, 1e-5)
            }
            Grid::Lambert(d, ws) => {
                diffusion::solve(ws, d, &mut hj, &kd, dt, DiffusionSolver::Jacobi, 1e-5)
            }
        };
        lap(&mut t, "5 diffusion jacobi", s, timed);
        let s = Instant::now();
        mg_its += match grid {
            Grid::Cube(g, ws) => {
                cube::solve(ws, g, &mut h, &kd, dt, DiffusionSolver::Multigrid, 1e-5)
            }
            Grid::Lambert(d, ws) => {
                diffusion::solve(ws, d, &mut h, &kd, dt, DiffusionSolver::Multigrid, 1e-5)
            }
        };
        lap(&mut t, "6 diffusion multigrid", s, timed);
        let s = Instant::now();
        match grid {
            Grid::Cube(g, _) => {
                cube::failure::slope_failure(g, &sea_mask, sea, &fail, &mut h, &mut avail);
            }
            Grid::Lambert(d, _) => {
                marine::slope_failure(d, &sea_mask, sea, &fail, &mut h, &mut avail);
            }
        }
        lap(&mut t, "7 slope failure", s, timed);
        let s = Instant::now();
        match grid {
            Grid::Cube(g, ws) => {
                cube::spread::step_in(ws, g, &mut h, sea, 2500.0, 10.0, dt);
            }
            Grid::Lambert(d, ws) => {
                spread::step_in(ws, d, &mut h, sea, 2500.0, 10.0, dt);
            }
        }
        lap(&mut t, "8 spread", s, timed);
        let s = Instant::now();
        let level = match grid {
            Grid::Cube(g, _) => {
                sea_level::fill_level_on(g, &h, 2700.0, n / 500, FloodMethod::Parallel)
            }
            Grid::Lambert(d, _) => {
                sea_level::fill_level(d, &h, &row_area, 2700.0, n / 500, FloodMethod::Parallel)
            }
        };
        lap(&mut t, "9 sea level", s, timed);
        if step + 1 == steps {
            eprintln!(
                "  last step: sea level {level:.1} m; {mouths} mouth fills over {fill_cells} \
                 cells; multigrid iterations {mg_its}, Jacobi {jac_its} over {steps} steps"
            );
        }
    }
    let k = (steps - warm) as f64;
    t.values_mut().for_each(|v| *v /= k);
    t
}

fn main() {
    let mut threads = 8usize;
    let mut steps = 12usize;
    let mut side = 240usize;
    let mut repeat = 3usize;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        let mut v = || -> usize {
            args.next()
                .and_then(|v| v.parse().ok())
                .expect("a number after the flag")
        };
        match a.as_str() {
            "--threads" => threads = v(),
            "--steps" => steps = v(),
            "--n" => side = v(),
            "--repeat" => repeat = v(),
            other => panic!("unknown argument {other}"),
        }
    }
    let warm = 2.min(steps.saturating_sub(1));
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .expect("rayon pool");
    let wv = waves(7);
    let g = CubeGrid::new(side, R);
    let cells = g.topology().cells();
    let w = 1024usize;
    let rows = cells.div_ceil(w);
    let dx = g.mean_area().sqrt();
    let d = Dims::new(w, rows, dx, true);
    eprintln!(
        "cube n {side}: {cells} cells; Lambert {w} × {rows}: {} cells; {threads} threads, {steps} \
         steps ({warm} not timed)",
        d.cells()
    );
    let hc: Vec<f64> = (0..cells).map(|c| terrain(&wv, g.unit(c))).collect();
    let hl: Vec<f64> = (0..d.cells())
        .map(|c| {
            let (i, j) = (c % w, c / w);
            let lon =
                2.0 * std::f64::consts::PI * (i as f64 + 0.5) / w as f64 - std::f64::consts::PI;
            let sl = 1.0 - 2.0 * (j as f64 + 0.5) / rows as f64;
            let cl = (1.0 - sl * sl).sqrt();
            terrain(&wv, [cl * lon.cos(), cl * lon.sin(), sl])
        })
        .collect();
    let mut cube_grid = Grid::Cube(g, cube::Workspace::default());
    let mut lambert = Grid::Lambert(d, diffusion::Workspace::default());
    let (mut tc, mut tl) = (Times::new(), Times::new());
    let fastest = |all: &mut Times, t: Times| {
        for (k, v) in t {
            let e = all.entry(k).or_insert(f64::INFINITY);
            *e = e.min(v);
        }
    };
    for r in 0..repeat.max(1) {
        eprintln!("repeat {r}, cube:");
        fastest(&mut tc, run(&mut cube_grid, hc.clone(), steps, warm));
        eprintln!("repeat {r}, lambert:");
        fastest(&mut tl, run(&mut lambert, hl.clone(), steps, warm));
    }
    println!("phase,cube_ms_per_step,lambert_ms_per_step,cube_over_lambert");
    let (mut sc, mut sl) = (0.0, 0.0);
    for (k, &c) in &tc {
        let l = tl[k];
        sc += c;
        sl += l;
        println!("{k},{c:.2},{l:.2},{:.3}", c / l);
    }
    println!("total,{sc:.2},{sl:.2},{:.3}", sc / sl);
}
