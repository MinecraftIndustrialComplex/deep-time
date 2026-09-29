// SPDX-License-Identifier: LGPL-3.0-or-later
//! The parallel passes against the serial loops they replaced (P1-17). The serial versions are
//! kept here verbatim from P1-04 as the reference; every comparison is on the bits of the floats,
//! not on a tolerance, because a resumed run has to reproduce a run at another thread count
//! exactly. The grids are random, wrap in x (and, in some cases, do not), have pits (lakes), sinks
//! and a critical slope, and are solved on pools of 1, 2, 4 and 8 threads.

use super::super::grid::Dims;
use super::super::pflood::{self, BASE, LAND, SINK};
use super::super::spl::{self, FAMILIES, LAKE_TOL_M, SplInput, SplParams, SplReport, incise};
use super::{Routing, accumulate, receivers, stack};
use crate::sphere::SplitMix64;

/// The serial `accumulate` of P1-04.
fn accumulate_serial(d: &Dims, r: &Routing, weight: &[f32], area: &mut Vec<f64>) {
    let a = d.cell_area();
    area.clear();
    area.extend(weight.iter().map(|&w| f64::from(w) * a));
    for &c in r.stack.iter().rev() {
        let c = c as usize;
        let rc = r.rcv[c] as usize;
        if rc != c {
            area[rc] += area[c];
        }
    }
}

/// The serial `spl::solve` of P1-04.
fn solve_serial(
    d: &Dims,
    r: &Routing,
    input: &SplInput<'_>,
    p: &SplParams,
    h: &mut [f64],
    qin: &mut [f64],
) -> SplReport {
    let a = d.cell_area();
    let n = d.cells();
    h.copy_from_slice(input.h0);
    qin.iter_mut().for_each(|q| *q = 0.0);
    let mut report = SplReport::default();
    let kind = input.kind;
    for it in 0..p.max_iter.max(1) {
        let mut change: f64 = 0.0;
        // Downstream first: every receiver is solved before its donors.
        for &c in &r.stack {
            let c = c as usize;
            let before = h[c];
            let h0 = input.h0[c];
            // Sinks and lakes only take sediment: the flux pass fills them.
            if kind[c] != SINK && input.filled[c] - h0 <= LAKE_TOL_M {
                let rc = r.rcv[c] as usize;
                let base = if kind[rc] == BASE {
                    input.base_z[rc]
                } else if input.filled[rc] - input.h0[rc] > LAKE_TOL_M {
                    h[rc].max(input.filled[rc])
                } else {
                    h[rc]
                };
                let ar = input.area[c].max(0.01 * a);
                let len = f64::from(r.len[c]).max(1e-9);
                let b = h0 + p.g * qin[c] / ar * p.dt;
                let f = input.kf[c] * ar.powf(p.m) * p.dt / len.powf(p.n);
                let mut x = incise(b, base, f, p.n);
                if x > base + p.sc * len {
                    x = base + p.sc * len;
                }
                h[c] = x;
            }
            change = change.max((h[c] - before).abs());
        }
        // Upstream first: the fluxes of this solution; sinks keep all they get, lakes fill up to
        // their level, and deposition is capped by the supply.
        qin.iter_mut().for_each(|q| *q = 0.0);
        for &c in r.stack.iter().rev() {
            let c = c as usize;
            let h0 = input.h0[c];
            let supply = qin[c] * p.dt / a;
            let out = if kind[c] == SINK {
                h[c] = h0 + supply;
                0.0
            } else if input.filled[c] - h0 > LAKE_TOL_M {
                let dep = supply.min(input.filled[c] - h0);
                h[c] = h0 + dep;
                (qin[c] - dep * a / p.dt).max(0.0)
            } else {
                let out = qin[c] + (h0 - h[c]) * a / p.dt;
                if out < 0.0 {
                    h[c] = h0 + supply;
                    0.0
                } else {
                    out
                }
            };
            let rc = r.rcv[c] as usize;
            if rc != c {
                qin[rc] += out;
            }
        }
        report.iterations = it + 1;
        report.residual_m = if it == 0 { f64::INFINITY } else { change };
        if it > 0 && change < p.tol_m {
            break;
        }
        if p.g == 0.0 {
            // Without deposition the first pass is already the solution (no flux feedback).
            report.residual_m = 0.0;
            break;
        }
    }
    debug_assert_eq!(h.len(), n);
    report
}

/// The serial `spl::families` of P1-04.
fn families_serial(
    d: &Dims,
    r: &Routing,
    input: &SplInput<'_>,
    fam: &[u8],
    dt: f64,
    h: &[f64],
    mix: &mut Vec<[f64; FAMILIES]>,
) {
    let a = d.cell_area();
    mix.clear();
    mix.resize(d.cells(), [0.0; FAMILIES]);
    for &c in r.stack.iter().rev() {
        let c = c as usize;
        let fin = mix[c];
        let qin: f64 = fin.iter().sum();
        let e = (input.h0[c] - h[c]) * a / dt;
        let mut out = fin;
        if e > 0.0 {
            out[usize::from(fam[c]).min(FAMILIES - 1)] += e;
        } else if qin > 0.0 {
            let keep = ((qin + e) / qin).clamp(0.0, 1.0);
            out.iter_mut().for_each(|x| *x *= keep);
        }
        let rc = r.rcv[c] as usize;
        if rc != c {
            for k in 0..FAMILIES {
                mix[rc][k] += out[k];
            }
        }
    }
}

/// A noisy DEM with pits and a sea band along the first rows (none when `sea` is false: the
/// land drains to sinks), and the erodibility, runoff weights and source families to go with it.
struct Case {
    d: Dims,
    h: Vec<f64>,
    kind: Vec<u8>,
    base_z: Vec<f64>,
    kf: Vec<f64>,
    weight: Vec<f32>,
    fam: Vec<u8>,
}

fn case(w: usize, hgt: usize, seed: u64, wrap: bool, sea: bool, coarse: bool) -> Case {
    let d = Dims::new(w, hgt, 1000.0, wrap);
    let mut rng = SplitMix64::new(seed);
    let n = w * hgt;
    let mut h = vec![0.0; n];
    let mut kind = vec![LAND; n];
    for c in 0..n {
        let j = c / w;
        if sea && j < 3 {
            h[c] = -100.0 - rng.next_f64() * 50.0;
            kind[c] = BASE;
        } else {
            let noise = rng.next_f64() * 400.0;
            // `coarse` quantises the heights so that ties (flats) are everywhere.
            let noise = if coarse {
                (noise / 50.0).floor() * 50.0
            } else {
                noise
            };
            let pit = if (c / 7 + seed as usize) % 23 == 0 {
                -300.0
            } else {
                0.0
            };
            h[c] = j as f64 * 20.0 + noise + pit;
        }
    }
    let kf = (0..n).map(|_| 1e-9 * (0.5 + rng.next_f64())).collect();
    let weight = (0..n).map(|_| (0.2 + rng.next_f64()) as f32).collect();
    let fam = (0..n).map(|_| (rng.next_f64() * 4.0) as u8).collect();
    Case {
        d,
        h,
        kind,
        base_z: vec![0.0; n],
        kf,
        weight,
        fam,
    }
}

fn same(what: &str, a: &[f64], b: &[f64]) {
    assert_eq!(a.len(), b.len(), "{what}: length");
    for (i, (x, y)) in a.iter().zip(b).enumerate() {
        assert_eq!(x.to_bits(), y.to_bits(), "{what}[{i}]: {x} vs {y}");
    }
}

fn in_pool<R: Send>(threads: usize, f: impl FnOnce() -> R + Send) -> R {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap()
        .install(f)
}

struct Solved {
    area: Vec<f64>,
    h: Vec<f64>,
    qin: Vec<f64>,
    report: SplReport,
    mix: Vec<[f64; FAMILIES]>,
}

/// Flood, route and solve with the parallel passes, on a pool of `threads`.
fn parallel(c: &Case, sp: &SplParams, threads: usize) -> Solved {
    in_pool(threads, || {
        let n = c.d.cells();
        let mut kind = c.kind.clone();
        let mut filled = vec![0.0; n];
        pflood::flood(&c.d, &c.h, &c.base_z, &mut kind, &mut filled);
        let mut r = Routing::default();
        receivers(&c.d, &filled, &kind, &mut r);
        stack(&c.d, &kind, &mut r);
        let mut area = Vec::new();
        accumulate(&c.d, &r, &c.weight, &mut area);
        let input = SplInput {
            kind: &kind,
            h0: &c.h,
            filled: &filled,
            base_z: &c.base_z,
            area: &area,
            kf: &c.kf,
        };
        let mut h = vec![0.0; n];
        let mut qin = vec![0.0; n];
        let report = spl::solve(&c.d, &r, &input, sp, &mut h, &mut qin);
        let mut mix = Vec::new();
        spl::families(&c.d, &r, &input, &c.fam, sp.dt, &h, &mut mix);
        Solved {
            area,
            h,
            qin,
            report,
            mix,
        }
    })
}

/// The same with the serial loops.
fn serial(c: &Case, sp: &SplParams) -> Solved {
    let n = c.d.cells();
    let mut kind = c.kind.clone();
    let mut filled = vec![0.0; n];
    pflood::flood(&c.d, &c.h, &c.base_z, &mut kind, &mut filled);
    let mut r = Routing::default();
    receivers(&c.d, &filled, &kind, &mut r);
    stack(&c.d, &kind, &mut r);
    let mut area = Vec::new();
    accumulate_serial(&c.d, &r, &c.weight, &mut area);
    let input = SplInput {
        kind: &kind,
        h0: &c.h,
        filled: &filled,
        base_z: &c.base_z,
        area: &area,
        kf: &c.kf,
    };
    let mut h = vec![0.0; n];
    let mut qin = vec![0.0; n];
    let report = solve_serial(&c.d, &r, &input, sp, &mut h, &mut qin);
    let mut mix = Vec::new();
    families_serial(&c.d, &r, &input, &c.fam, sp.dt, &h, &mut mix);
    Solved {
        area,
        h,
        qin,
        report,
        mix,
    }
}

fn params(n: f64, g: f64, sc: f64) -> SplParams {
    SplParams {
        m: 0.5 * (1.0 + n),
        n,
        g,
        dt: 2.0e6,
        sc,
        max_iter: 12,
        tol_m: 1e-3,
    }
}

fn check(c: &Case, sp: &SplParams, what: &str) {
    let want = serial(c, sp);
    for threads in [1, 2, 4, 8] {
        let got = parallel(c, sp, threads);
        let tag = format!("{what}, {threads} threads");
        same(&format!("{tag}: area"), &got.area, &want.area);
        same(&format!("{tag}: h"), &got.h, &want.h);
        same(&format!("{tag}: qin"), &got.qin, &want.qin);
        assert_eq!(got.report.iterations, want.report.iterations, "{tag}");
        assert_eq!(
            got.report.residual_m.to_bits(),
            want.report.residual_m.to_bits(),
            "{tag}"
        );
        assert_eq!(got.mix.len(), want.mix.len());
        for (i, (x, y)) in got.mix.iter().zip(&want.mix).enumerate() {
            for k in 0..FAMILIES {
                assert_eq!(x[k].to_bits(), y[k].to_bits(), "{tag}: mix[{i}][{k}]");
            }
        }
    }
}

#[test]
fn the_parallel_solve_is_the_serial_solve_to_the_last_bit() {
    // Wrapping in x, with a sea: the exact solve (n = 1, no deposition), Yuan's iterated one
    // (n = 2, G = 1, with a critical slope) and a fractional exponent.
    for seed in 1..4 {
        let c = case(160, 96, seed, true, true, false);
        check(
            &c,
            &params(1.0, 0.0, f64::INFINITY),
            &format!("seed {seed} n=1 G=0"),
        );
        check(
            &c,
            &params(2.0, 1.0, 0.35),
            &format!("seed {seed} n=2 G=1 Sc"),
        );
        check(
            &c,
            &params(1.5, 0.7, f64::INFINITY),
            &format!("seed {seed} n=1.5 G=.7"),
        );
    }
}

#[test]
fn the_parallel_passes_agree_on_tied_heights_and_closed_grids() {
    // Quantised heights (flats everywhere), a grid that does not wrap, and land with no sea (the
    // flood makes sinks, which keep everything they receive).
    let sp = params(2.0, 1.0, 0.35);
    check(&case(128, 80, 11, true, true, true), &sp, "tied heights");
    check(&case(128, 80, 12, false, true, false), &sp, "closed in x");
    check(&case(96, 64, 13, true, false, false), &sp, "no sea");
    check(&case(96, 64, 14, true, false, true), &sp, "no sea, ties");
}

#[test]
fn rivers_crossing_the_seam_are_summed_in_serial_order() {
    // Very narrow bands: x = 0 and x = w − 1 are neighbours, and cells have donors on both sides
    // of the seam.
    let sp = params(1.0, 0.5, f64::INFINITY);
    check(&case(3, 400, 21, true, true, false), &sp, "3 wide");
    check(&case(5, 300, 22, true, true, false), &sp, "5 wide");
}

#[test]
fn one_thread_and_twelve_walk_the_same_forest() {
    let c = case(200, 120, 5, true, true, false);
    let sp = params(2.0, 1.0, 0.35);
    let one = parallel(&c, &sp, 1);
    let many = parallel(&c, &sp, 12);
    same("h", &one.h, &many.h);
    same("qin", &one.qin, &many.qin);
    same("area", &one.area, &many.area);
    assert!(
        one.report.iterations > 1,
        "the iteration ran ({:?})",
        one.report
    );
}
