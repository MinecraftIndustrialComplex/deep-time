// SPDX-License-Identifier: LGPL-3.0-or-later
//! The parallel passes against the serial loops they replaced (P1-17). The serial versions are
//! kept here verbatim from P1-04 as the reference; every comparison is on the bits of the floats,
//! not on a tolerance, because a resumed run has to reproduce a run at another thread count
//! exactly. The grids are random, wrap in x (and, in some cases, do not), have pits (lakes), sinks
//! and a critical slope, and are solved on pools of 1, 2, 4 and 8 threads.

use super::super::grid::Dims;
use super::super::pflood::{self, BASE, LAND, SINK};
use super::super::spl::{self, FAMILIES, LAKE_TOL_M, SplInput, SplParams, SplReport, incise};
use super::{Routing, accumulate, donors_by_neighbours, donors_in_order, receivers, stack};
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

/// Each cell's place in its lake (`spl::lake_links`, HYD-110), by cell: 0 not a lake cell, 1 a
/// lake cell draining into the same lake, 2 one draining out into the sea, a sink or another lake,
/// 3 one draining over its spill (a land cell).
fn lake_links_serial(r: &Routing, input: &SplInput<'_>) -> Vec<u8> {
    let lake = |c: usize| input.kind[c] != SINK && input.filled[c] - input.h0[c] > LAKE_TOL_M;
    let mut links = vec![0u8; input.kind.len()];
    for &c in &r.stack {
        let c = c as usize;
        if !lake(c) {
            continue;
        }
        let rc = r.rcv[c] as usize;
        let inside = rc != c && input.kind[rc] != BASE;
        links[c] = if inside && lake(rc) && (input.filled[c] - input.filled[rc]).abs() <= LAKE_TOL_M
        {
            1
        } else if inside && input.kind[rc] == LAND && !lake(rc) {
            3
        } else {
            2
        };
    }
    links
}

/// The serial `spl::solve` of P1-04, with the lakes of HYD-110.
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
    let links = lake_links_serial(r, input);
    let mut share = vec![0.0f64; n];
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
        // Upstream first: the fluxes of this solution; sinks keep all they get, a lake keeps what
        // reaches it up to its room, and deposition is capped by the supply.
        qin.iter_mut().for_each(|q| *q = 0.0);
        let mut room = vec![0.0f64; n];
        let mut upstream = vec![0.0f64; n];
        let mut lake_in = vec![0.0f64; n];
        let mut lake_room = vec![0.0f64; n];
        for &c in r.stack.iter().rev() {
            let c = c as usize;
            let h0 = input.h0[c];
            if lake_room[c] > 0.0 {
                let dep = (lake_in[c] * p.dt / a).min(lake_room[c]);
                share[c] = dep / lake_room[c];
                qin[c] = (qin[c] - dep * a / p.dt).max(0.0);
            }
            let supply = qin[c] * p.dt / a;
            let out = if kind[c] == SINK {
                h[c] = h0 + supply;
                0.0
            } else if links[c] == 1 || links[c] == 3 {
                room[c] = upstream[c] + (input.filled[c] - h0);
                qin[c]
            } else if links[c] == 2 {
                let v = upstream[c] + (input.filled[c] - h0);
                let dep = supply.min(v);
                share[c] = if v > 0.0 { dep / v } else { 0.0 };
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
                match links[c] {
                    1 => upstream[rc] += room[c],
                    3 => {
                        lake_in[rc] += out;
                        lake_room[rc] += room[c];
                    }
                    _ => {}
                }
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
    // Each lake settles what it kept over all of it.
    for &c in &r.stack {
        let c = c as usize;
        let depth = input.filled[c] - input.h0[c];
        match links[c] {
            1 | 3 => {
                share[c] = share[r.rcv[c] as usize];
                h[c] = input.h0[c] + share[c] * depth;
            }
            2 => h[c] = input.h0[c] + share[c] * depth,
            _ => {}
        }
    }
    debug_assert_eq!(h.len(), n);
    report
}

/// The serial `spl::families` of P1-04, with the lakes of HYD-110.
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
    let n = d.cells();
    mix.clear();
    mix.resize(n, [0.0; FAMILIES]);
    let links = lake_links_serial(r, input);
    let mut kept = vec![0.0f64; n];
    let mut upstream = vec![0.0f64; n];
    let mut spilled = vec![0.0f64; n];
    for &c in r.stack.iter().rev() {
        let c = c as usize;
        let mut fin = mix[c];
        let mut qin: f64 = fin.iter().sum();
        if spilled[c] > 0.0 && qin > 0.0 {
            let keep = ((qin - spilled[c]) / qin).clamp(0.0, 1.0);
            fin.iter_mut().for_each(|x| *x *= keep);
            qin = fin.iter().sum();
        }
        let ev = (input.h0[c] - h[c]) * a / dt;
        let e = match links[c] {
            1 | 3 => {
                kept[c] = upstream[c] + (-ev).max(0.0);
                0.0
            }
            2 => ev.min(0.0) - upstream[c],
            _ => ev,
        };
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
            match links[c] {
                1 => upstream[rc] += kept[c],
                3 => spilled[rc] += kept[c],
                _ => {}
            }
        }
    }
    // A lake's cells take the mix entering it at its exit (or its spill).
    for &c in &r.stack {
        let c = c as usize;
        if links[c] == 1 || links[c] == 3 {
            mix[c] = mix[r.rcv[c] as usize];
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
            let pit = if (c / 7 + seed as usize).is_multiple_of(23) {
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

/// The least fixed point of the ε relaxation, w[m] = min over neighbours c of max(h[m],
/// next_up(w[c])) with the sea at its base level: what any flood that is independent of its own
/// processing order (tiled, parallel, Dijkstra with exact ties) computes. Brute force.
fn least_fixed_point(d: &Dims, h: &[f64], base_z: &[f64], kind: &[u8]) -> Vec<f64> {
    let n = d.cells();
    let mut w: Vec<f64> = (0..n)
        .map(|c| {
            if kind[c] == BASE {
                base_z[c]
            } else {
                f64::INFINITY
            }
        })
        .collect();
    loop {
        let mut changed = false;
        for c in 0..n {
            if kind[c] == BASE {
                continue;
            }
            for (m, _) in d.neighbours(c) {
                let v = h[c].max(w[m].next_up());
                if v < w[c] {
                    w[c] = v;
                    changed = true;
                }
            }
        }
        if !changed {
            return w;
        }
    }
}

#[test]
fn the_serial_flood_is_the_least_fixed_point_except_on_exact_ties() {
    // Why the flood is not tiled: on generic (untied) heights the serial FIFO flood and the
    // order-independent least fixed point are the same surface, bit for bit; where heights are
    // exactly equal (flats fed from two sides) the serial order shows in the ulps.
    let differing = |coarse: bool, seed: u64| -> usize {
        let c = case(40, 30, seed, true, true, coarse);
        let n = c.d.cells();
        // The sea as the pipeline finds it: every connected cell at or below sea level.
        let mut kind = vec![LAND; n];
        pflood::open_sea(&c.d, &c.h, 0.0, 1, &mut kind);
        let mut filled = vec![0.0; n];
        pflood::flood(&c.d, &c.h, &c.base_z, &mut kind, &mut filled);
        let lfp = least_fixed_point(&c.d, &c.h, &c.base_z, &kind);
        (0..n)
            .filter(|&i| filled[i].to_bits() != lfp[i].to_bits())
            .count()
    };
    for seed in 1..8 {
        assert_eq!(differing(false, seed), 0, "untied heights, seed {seed}");
    }
    let tied: usize = (1..8).map(|seed| differing(true, seed)).sum();
    assert!(
        tied > 0,
        "tied heights: the serial order matters in the ulps"
    );
}

/// A fractal terrain like the L1 elevations: value noise of eight octaves on a wrapping raster.
fn fractal(w: usize, hgt: usize, seed: u64) -> Vec<f64> {
    let mut rng = SplitMix64::new(seed);
    let mut z = vec![0.0; w * hgt];
    for oct in 0..8 {
        let (gw, gh) = (4usize << oct, 2usize << oct);
        let g: Vec<f64> = (0..gw * gh).map(|_| rng.next_f64() - 0.5).collect();
        let amp = 4000.0 / f64::from(1 << oct).powf(0.9);
        for j in 0..hgt {
            let fy = j as f64 / hgt as f64 * gh as f64;
            let (j0, ty) = (fy.floor() as usize, fy.fract());
            for i in 0..w {
                let fx = i as f64 / w as f64 * gw as f64;
                let (i0, tx) = (fx.floor() as usize, fx.fract());
                let at = |a: usize, b: usize| g[b.min(gh - 1) * gw + a % gw];
                let top = at(i0, j0) * (1.0 - tx) + at(i0 + 1, j0) * tx;
                let bot = at(i0, j0 + 1) * (1.0 - tx) + at(i0 + 1, j0 + 1) * tx;
                z[j * w + i] += amp * (top * (1.0 - ty) + bot * ty);
            }
        }
    }
    z
}

/// Times each phase of one LEM step on an L1-sized terrain (`cargo test --release -p
/// deeptime-core surface_timing -- --ignored --nocapture`): best of ten per phase and thread
/// count, the P1-04 serial loops beside the parallel passes. A clean measurement of what the
/// forest passes buy on a machine whose load makes the end-to-end bench noisy.
#[test]
#[ignore = "a timing, not a check"]
fn surface_timing() {
    let (w, hgt) = (1024, 512);
    let d = Dims::new(w, hgt, 4800.0, true);
    let n = d.cells();
    let h = fractal(w, hgt, 7);
    let mut sorted = h.clone();
    sorted.sort_by(f64::total_cmp);
    let sea = sorted[n * 78 / 100];
    let mut kind = vec![LAND; n];
    pflood::open_sea(&d, &h, sea, n / 500, &mut kind);
    let base_z = vec![sea; n];
    let mut filled = vec![0.0; n];
    pflood::flood(&d, &h, &base_z, &mut kind, &mut filled);
    let weight = vec![1.0f32; n];
    let kf = vec![3e-10; n];
    let fam = vec![1u8; n];
    let sp = SplParams {
        m: 1.0,
        n: 2.0,
        g: 1.0,
        dt: 2.0e6,
        sc: 0.35,
        max_iter: 6,
        tol_m: 0.01,
    };
    let best = |f: &mut dyn FnMut()| {
        (0..10)
            .map(|_| {
                let t = std::time::Instant::now();
                f();
                t.elapsed().as_secs_f64() * 1e3
            })
            .fold(f64::INFINITY, f64::min)
    };
    let mut r = Routing::default();
    let mut area = Vec::new();
    let mut hn = vec![0.0; n];
    let mut qin = vec![0.0; n];
    let mut mix = Vec::new();
    let land = kind.iter().filter(|&&k| k == LAND).count();
    println!("{land} land cells of {n}; ms per step, best of 10");
    let sea_ms = best(&mut || {
        let mut k = vec![LAND; n];
        pflood::open_sea(&d, &h, sea, n / 500, &mut k);
    });
    let flood_ms = best(&mut || {
        let mut k = kind.clone();
        pflood::flood(&d, &h, &base_z, &mut k, &mut filled);
    });
    println!("serial: open_sea {sea_ms:.2}, flood {flood_ms:.2}");
    receivers(&d, &filled, &kind, &mut r);
    stack(&d, &kind, &mut r);
    let acc0 = best(&mut || accumulate_serial(&d, &r, &weight, &mut area));
    let input = SplInput {
        kind: &kind,
        h0: &h,
        filled: &filled,
        base_z: &base_z,
        area: &area,
        kf: &kf,
    };
    let sol0 = best(&mut || {
        solve_serial(&d, &r, &input, &sp, &mut hn, &mut qin);
    });
    let fam0 = best(&mut || families_serial(&d, &r, &input, &fam, sp.dt, &hn, &mut mix));
    println!(
        "P1-04 serial loops: accumulate {acc0:.2}, solve {sol0:.2}, families {fam0:.2}, sum {:.2}",
        acc0 + sol0 + fam0
    );
    for threads in [1, 2, 4, 8, 12] {
        in_pool(threads, || {
            // What a perfectly parallel loop gets from this machine right now (other jobs share it).
            let cal = best(&mut || {
                use rayon::prelude::*;
                let s: f64 = (0..4_000_000u32)
                    .into_par_iter()
                    .map(|i| f64::from(i).sqrt().sin())
                    .sum();
                std::hint::black_box(s);
            });
            print!("{threads:>2} threads (calibration loop {cal:.1} ms): ");
            let rcv = best(&mut || receivers(&d, &filled, &kind, &mut r));
            let stk = best(&mut || stack(&d, &kind, &mut r));
            let acc = best(&mut || accumulate(&d, &r, &weight, &mut area));
            let input = SplInput {
                kind: &kind,
                h0: &h,
                filled: &filled,
                base_z: &base_z,
                area: &area,
                kf: &kf,
            };
            let sol = best(&mut || {
                spl::solve(&d, &r, &input, &sp, &mut hn, &mut qin);
            });
            let fm = best(&mut || spl::families(&d, &r, &input, &fam, sp.dt, &hn, &mut mix));
            println!(
                "receivers {rcv:.2}, stack {stk:.2}, accumulate {acc:.2}, solve {sol:.2}, families {fm:.2}"
            );
        });
    }
}

/// The stack, its basins and its forest's arrays as one thread built them before the speed work
/// of 2026-10-28: the donors by a counting sort over the cells, the basins one walk after another,
/// the forest's arrays one loop each.
#[allow(clippy::type_complexity)]
fn stack_serial(kind: &[u8], rcv: &[u32]) -> (Vec<u32>, Vec<(u32, u32, u32)>, Vec<Vec<u32>>) {
    let n = rcv.len();
    let (count, donors) = donors_in_order(n, rcv);
    let (mut stack, mut basins) = (Vec::new(), Vec::new());
    let mut todo: Vec<u32> = Vec::new();
    for root in 0..n {
        if kind[root] == LAND || count[root] == count[root + 1] {
            continue;
        }
        let start = stack.len() as u32;
        if kind[root] == SINK {
            stack.push(root as u32);
        }
        todo.clear();
        todo.extend_from_slice(&donors[count[root] as usize..count[root + 1] as usize]);
        while let Some(c) = todo.pop() {
            stack.push(c);
            let c = c as usize;
            todo.extend_from_slice(&donors[count[c] as usize..count[c + 1] as usize]);
        }
        basins.push((root as u32, start, stack.len() as u32));
    }
    const NONE: u32 = u32::MAX;
    let l = stack.len();
    let mut pos = vec![NONE; n];
    for (i, &c) in stack.iter().enumerate() {
        pos[c as usize] = i as u32;
    }
    let rpos: Vec<u32> = stack
        .iter()
        .map(|&c| {
            let rc = rcv[c as usize];
            if rc == c { NONE } else { pos[rc as usize] }
        })
        .collect();
    let (mut dptr, mut dpos) = (vec![0u32], Vec::new());
    for &c in &stack {
        let c = c as usize;
        for &d in &donors[count[c] as usize..count[c + 1] as usize] {
            dpos.push(pos[d as usize]);
        }
        dptr.push(dpos.len() as u32);
    }
    let mut size = vec![1u32; l];
    for i in (0..l).rev() {
        if rpos[i] != NONE {
            size[rpos[i] as usize] += size[i];
        }
    }
    let (mut tops, mut leaves) = (Vec::new(), Vec::new());
    for i in 0..l {
        if rpos[i] == NONE {
            tops.push(i as u32);
        }
        if dptr[i] == dptr[i + 1] {
            leaves.push(i as u32);
        }
    }
    let (mut outlets, mut optr, mut opos) = (Vec::new(), vec![0u32], Vec::new());
    for &(root, _, _) in &basins {
        let c = root as usize;
        if kind[c] != BASE {
            continue;
        }
        outlets.push(root);
        for &d in &donors[count[c] as usize..count[c + 1] as usize] {
            opos.push(pos[d as usize]);
        }
        optr.push(opos.len() as u32);
    }
    let forest = vec![
        pos, rpos, dptr, dpos, size, tops, leaves, outlets, optr, opos,
    ];
    (stack, basins, forest)
}

/// The stack built in parallel (the donors among each cell's neighbours, the basins side by side,
/// the forest's arrays in parallel) is the serial one, array for array, on pools of 1, 3 and 8
/// threads; a receiver that is not a neighbour sends the donors back to the counting sort.
#[test]
fn the_parallel_stack_is_the_serial_stack() {
    for (seed, wrap, sea, coarse) in [
        (3, true, true, false),
        (8, false, true, true),
        (11, true, false, false),
    ] {
        let c = case(150, 90, seed, wrap, sea, coarse);
        for threads in [1, 3, 8] {
            in_pool(threads, || {
                let n = c.d.cells();
                let mut kind = c.kind.clone();
                let mut filled = vec![0.0; n];
                pflood::flood(&c.d, &c.h, &c.base_z, &mut kind, &mut filled);
                let mut r = Routing::default();
                receivers(&c.d, &filled, &kind, &mut r);
                assert_eq!(
                    donors_by_neighbours(&c.d, &r.rcv),
                    Some(donors_in_order(n, &r.rcv))
                );
                stack(&c.d, &kind, &mut r);
                let (st, basins, forest) = stack_serial(&kind, &r.rcv);
                assert_eq!(r.stack, st, "stack, seed {seed}, {threads} threads");
                assert_eq!(r.basins, basins, "basins, seed {seed}");
                let f = &r.forest;
                let got = [
                    &f.pos, &f.rpos, &f.dptr, &f.dpos, &f.size, &f.tops, &f.leaves, &f.outlets,
                    &f.optr, &f.opos,
                ];
                for (k, (a, b)) in got.iter().zip(&forest).enumerate() {
                    assert_eq!(*a, b, "forest array {k}, seed {seed}, {threads} threads");
                }
                let mut far = r.rcv.clone();
                let land = (0..n)
                    .find(|&m| kind[m] == LAND && far[m] as usize != m)
                    .unwrap();
                far[land] = ((land + n / 2) % n) as u32;
                assert_eq!(donors_by_neighbours(&c.d, &far), None);
            });
        }
    }
}
