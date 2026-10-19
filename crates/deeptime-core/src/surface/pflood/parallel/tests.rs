// SPDX-License-Identifier: LGPL-3.0-or-later
//! The parallel flood against two serial references written for the tests: the plain
//! Priority-Flood+ε (Barnes et al. 2014, Algorithm 3: one priority queue, no pit queue; ties by
//! cell index) for F, and the plain Priority-Flood (ε = 0) for the first phase's W. Bit for bit,
//! on fractal and on quantised (flat-rich) terrain, wrapping and not, with and without sea, at
//! several strip heights and thread counts.

use std::collections::BinaryHeap;

use super::super::super::grid::Dims;
use super::super::{BASE, LAND, SINK};
use super::*;
use crate::sphere::SplitMix64;

/// The plain Priority-Flood with an ε step `up` (next_up for F, the identity for W), sinks as the
/// serial flood makes them (the lowest unreached cell, lowest index among equals).
fn reference(
    d: &Dims,
    h: &[f64],
    base_z: &[f64],
    kind: &mut [u8],
    out: &mut [f64],
    up: fn(f64) -> f64,
) -> usize {
    let n = d.cells();
    let mut closed = vec![false; n];
    let mut q: BinaryHeap<Entry> = BinaryHeap::new();
    for c in 0..n {
        if kind[c] == BASE {
            closed[c] = true;
            out[c] = base_z[c];
            q.push(Entry {
                z: out[c],
                c: c as u32,
            });
        }
    }
    let mut sinks = 0;
    loop {
        while let Some(Entry { z, c }) = q.pop() {
            for (m, _) in d.neighbours(c as usize) {
                if closed[m] {
                    continue;
                }
                closed[m] = true;
                out[m] = h[m].max(up(z));
                q.push(Entry {
                    z: out[m],
                    c: m as u32,
                });
            }
        }
        let Some(first) = (0..n).find(|&c| !closed[c]) else {
            return sinks;
        };
        let mut best = first;
        for c in first..n {
            if !closed[c] && h[c] < h[best] {
                best = c;
            }
        }
        closed[best] = true;
        out[best] = h[best];
        kind[best] = SINK;
        sinks += 1;
        q.push(Entry {
            z: out[best],
            c: best as u32,
        });
    }
}

fn in_pool<R: Send>(threads: usize, f: impl FnOnce() -> R + Send) -> R {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap()
        .install(f)
}

/// Value noise of a few octaves on a raster, plus pits; `step` > 0 quantises the heights so flats
/// and exact ties are everywhere.
fn terrain(w: usize, hgt: usize, seed: u64, step: f64) -> Vec<f64> {
    let mut rng = SplitMix64::new(seed);
    let mut z = vec![0.0; w * hgt];
    for oct in 0..5 {
        let (gw, gh) = (2usize << oct, 2usize << oct);
        let g: Vec<f64> = (0..gw * gh).map(|_| rng.next_f64() - 0.5).collect();
        let amp = 800.0 / f64::from(1 << oct);
        for j in 0..hgt {
            let fy = j as f64 / hgt as f64 * (gh - 1) as f64;
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
    for v in z.iter_mut() {
        if rng.next_f64() < 0.03 {
            *v -= 150.0;
        }
        if step > 0.0 {
            *v = (*v / step).floor() * step;
        }
    }
    z
}

/// Compares the parallel flood with the reference on one terrain, at several strip heights and
/// thread counts; returns the cells raised above their height.
fn check(d: &Dims, h: &[f64], sea: Option<f64>, label: &str) -> usize {
    let n = d.cells();
    let mut kind0 = vec![LAND; n];
    if let Some(s) = sea {
        let mut serial = vec![LAND; n];
        super::super::open_sea(d, h, s, 3, &mut serial);
        for threads in [1, 3, 8] {
            let mut k = vec![SINK; n];
            in_pool(threads, || open_sea(d, h, s, 3, &mut k));
            assert_eq!(k, serial, "{label}: open sea, {threads} threads");
        }
        kind0 = serial;
    }
    let base_z = vec![sea.unwrap_or(0.0); n];
    let mut kref = kind0.clone();
    let mut fref = vec![0.0; n];
    let sref = reference(d, h, &base_z, &mut kref, &mut fref, f64::next_up);
    // W, with the reference's sinks as base level.
    let mut kw = kref.clone();
    for k in kw.iter_mut() {
        if *k == SINK {
            *k = BASE;
        }
    }
    let bw: Vec<f64> = (0..n)
        .map(|c| if kref[c] == SINK { h[c] } else { base_z[c] })
        .collect();
    let mut wref = vec![0.0; n];
    reference(d, h, &bw, &mut kw.clone(), &mut wref, |z| z);
    for rows in [1, 2, 3, 5, 16, 1000] {
        for threads in [1, 2, 7] {
            let mut k = kind0.clone();
            let mut f = vec![f64::NAN; n];
            let s = in_pool(threads, || flood_rows(d, h, &base_z, &mut k, &mut f, rows));
            assert_eq!(s, sref, "{label}: sinks, rows {rows}, {threads} threads");
            assert_eq!(k, kref, "{label}: kinds, rows {rows}, {threads} threads");
            let bad = (0..n).filter(|&c| f[c].to_bits() != fref[c].to_bits()).count();
            assert_eq!(bad, 0, "{label}: F differs in {bad} cells, rows {rows}, {threads} threads");
            let fx = Fixed {
                kind: &kw,
                base_z: &bw,
                h,
                sink: None,
            };
            let mut wf = vec![f64::NAN; n];
            let mut lab = vec![NONE; n];
            let sinks = in_pool(threads, || fill_w(d, &fx, h, rows, &mut wf, &mut lab));
            assert!(sinks.is_empty(), "{label}: W found sinks");
            let bad = (0..n).filter(|&c| wf[c].to_bits() != wref[c].to_bits()).count();
            assert_eq!(bad, 0, "{label}: W differs in {bad} cells, rows {rows}");
        }
    }
    (0..n).filter(|&c| fref[c] > h[c]).count()
}

#[test]
fn the_parallel_flood_is_the_plain_priority_flood_bit_for_bit() {
    let mut raised = 0;
    for &(w, hgt) in &[(37, 23), (64, 40), (1, 9), (2, 7), (19, 1), (5, 5)] {
        for wrap in [true, false] {
            for seed in 1..4u64 {
                for step in [0.0, 25.0] {
                    let h = terrain(w, hgt, seed * 31 + w as u64, step);
                    let d = Dims::new(w, hgt, 1000.0, wrap);
                    let label = format!("{w}x{hgt} wrap {wrap} seed {seed} step {step}");
                    let mut sorted = h.clone();
                    sorted.sort_by(f64::total_cmp);
                    let sea = sorted[sorted.len() * 2 / 5];
                    raised += check(&d, &h, Some(sea), &format!("{label} sea"));
                    raised += check(&d, &h, None, &format!("{label} no sea"));
                }
            }
        }
    }
    assert!(raised > 1000, "the cases have lakes and flats ({raised} raised cells)");
}

#[test]
fn flats_left_by_a_filled_lake_are_resolved_by_the_least_fixed_point() {
    // A lake's sediment fill leaves its cells within ulps of one another: the flood of a filled
    // surface is where the serial FIFO order showed (STATUS.md, P1-17). Here the reference's own
    // fill is the terrain, so every lake is such a flat, fed from several sides.
    for seed in 1..5u64 {
        let (w, hgt) = (48, 32);
        let d = Dims::new(w, hgt, 1000.0, true);
        let h0 = terrain(w, hgt, seed, 0.0);
        let mut sorted = h0.clone();
        sorted.sort_by(f64::total_cmp);
        let sea = sorted[h0.len() / 3];
        let mut kind = vec![LAND; w * hgt];
        super::super::open_sea(&d, &h0, sea, 3, &mut kind);
        let mut f = vec![0.0; w * hgt];
        reference(&d, &h0, &vec![sea; w * hgt], &mut kind, &mut f, |z| z);
        check(&d, &f, Some(sea), &format!("filled seed {seed}"));
    }
}

#[test]
fn every_land_cell_drains_on_the_filled_surface() {
    let (w, hgt) = (64, 40);
    let d = Dims::new(w, hgt, 1000.0, true);
    let h = terrain(w, hgt, 9, 10.0);
    let mut kind = vec![LAND; w * hgt];
    let sea = 0.0;
    open_sea(&d, &h, sea, 3, &mut kind);
    let mut f = vec![0.0; w * hgt];
    flood(&d, &h, &vec![sea; w * hgt], &mut kind, &mut f);
    for c in 0..w * hgt {
        if kind[c] == LAND {
            assert!(f[c] >= h[c], "cell {c} below its height");
            assert!(
                d.neighbours(c).any(|(m, _)| f[m] < f[c]),
                "cell {c} has no lower neighbour"
            );
        }
    }
}
