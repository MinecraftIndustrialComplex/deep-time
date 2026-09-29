// SPDX-License-Identifier: LGPL-3.0-or-later
//! Times L0's geometry at the ladder's sizes (plan §5.2, P1-02) and re-checks the acceptance
//! properties at full size: Fibonacci points, the spherical Delaunay hull (V − E + F = 2), the
//! k-d tree, and the coupling of every L1 grid of the four built-in presets (weights sum to 1,
//! none negative, the triangle contains the cell).
//!
//! ```text
//! cargo run --release -p deeptime-core --example sphere_bench -- [--threads N] [--repeat K] [n...]
//! ```
//! Defaults: all of the machine's threads, 3 repeats (min and max reported), n = 100k and 250k.

use std::time::{Duration, Instant};

use deeptime_core::sphere::{Coupling, SampleTree, SphereMesh, fibonacci_sphere};
use deeptime_format::GridDesc;
use rayon::prelude::*;

/// The L1 grids of §5.2's ladder: (preset, C, r).
const L1: [(&str, u32, u32); 4] = [
    ("16k quick", 16_384, 16),
    ("16k full", 16_384, 8),
    ("32k quick", 32_768, 32),
    ("32k full", 32_768, 16),
];

fn main() {
    let mut threads = 0usize;
    let mut repeat = 3usize;
    let mut sizes = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--threads" => {
                threads = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .expect("--threads N")
            }
            "--repeat" => {
                repeat = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .expect("--repeat K")
            }
            n => sizes.push(n.parse::<usize>().expect("sample count")),
        }
    }
    if sizes.is_empty() {
        sizes = vec![100_000, 250_000];
    }
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .expect("thread pool");
    let failures = pool.install(|| run(&sizes, repeat.max(1)));
    if failures > 0 {
        eprintln!("FAILED: {failures} check(s)");
        std::process::exit(1);
    }
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

/// Runs `f` `repeat` times; returns the last result and the min and max wall time.
fn timed<T>(repeat: usize, mut f: impl FnMut() -> T) -> (T, f64, f64) {
    let (mut lo, mut hi) = (f64::INFINITY, 0.0f64);
    let mut out = None;
    for _ in 0..repeat {
        let t = Instant::now();
        let r = f();
        let e = ms(t.elapsed());
        lo = lo.min(e);
        hi = hi.max(e);
        out = Some(r);
    }
    (out.expect("repeat >= 1"), lo, hi)
}

fn run(sizes: &[usize], repeat: usize) -> usize {
    let mut failures = 0;
    println!(
        "threads={} repeat={repeat} (times in ms, min/max over the repeats)",
        rayon::current_num_threads()
    );
    for &n in sizes {
        let (pts, fib_lo, fib_hi) = timed(repeat, || fibonacci_sphere(n));
        let (mesh, hull_lo, hull_hi) =
            timed(repeat, || SphereMesh::build(&pts).expect("triangulation"));
        let (tree, tree_lo, tree_hi) = timed(repeat, || SampleTree::new(&pts));
        let mut edges: Vec<(u32, u32)> = mesh
            .triangles()
            .iter()
            .flat_map(|t| (0..3).map(move |k| (t[k].min(t[(k + 1) % 3]), t[k].max(t[(k + 1) % 3]))))
            .collect();
        edges.par_sort_unstable();
        edges.dedup();
        let (v, e, f) = (mesh.vertex_count(), edges.len(), mesh.triangles().len());
        let chi = v as i64 - e as i64 + f as i64;
        if chi != 2 || !mesh.skipped().is_empty() {
            failures += 1;
        }
        println!(
            "samples={n} fibonacci={fib_lo:.1}/{fib_hi:.1} hull={hull_lo:.1}/{hull_hi:.1} \
             kdtree={tree_lo:.1}/{tree_hi:.1} V={v} E={e} F={f} V-E+F={chi} skipped={}",
            mesh.skipped().len()
        );
        for (preset, c, r) in L1 {
            let grid = GridDesc::for_planet("L1", c, r).expect("grid");
            let (coupling, lo, hi) = timed(repeat, || Coupling::build(&grid, &mesh, &tree));
            let bad = (0..grid.cells())
                .into_par_iter()
                .filter(|&k| {
                    let l = coupling.links()[k];
                    let (i, j) = ((k % grid.w as usize) as i64, (k / grid.w as usize) as i64);
                    let sum: f64 = l.w.iter().map(|&x| f64::from(x)).sum();
                    (sum - 1.0).abs() > 1e-6
                        || l.w.iter().any(|&x| f64::from(x) < -1e-9)
                        || !mesh.contains(l.tri, coupling.cell_unit(i, j))
                })
                .count();
            if bad > 0 {
                failures += 1;
            }
            println!(
                "  L1 {preset:<9} C={c} r={r} {w}x{h} cells={cells} coupling={lo:.1}/{hi:.1} \
                 bad_cells={bad}",
                w = grid.w,
                h = grid.h,
                cells = grid.cells()
            );
        }
    }
    failures
}
