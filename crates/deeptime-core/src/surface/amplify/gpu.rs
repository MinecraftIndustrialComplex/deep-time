// SPDX-License-Identifier: LGPL-3.0-or-later
//! The amplifier's middle passes on the GPU (`docs/design/gpu-compute.md`): stream-power
//! erosion with its drainage area, the glacial valleys, thermal erosion and deposition, the
//! Jacobi iterations of `kernels.rs` and `glacial.rs` written again in WGSL, one invocation per
//! cell. The seeding, the floods and the mountain character stay on the CPU (the flood is a
//! priority queue; the character's port is a later step), so a window crosses to the device once
//! after the first flood and back once before the last.
//!
//! # Precision (the error budget, `gpu-compute.md` §Kernels)
//! Everything is f32, as on the CPU, and every sum runs over the eight neighbours in the CPU's
//! order, so a step differs from the CPU's only by rounding: fused multiply-adds the shader
//! compiler may form, and `pow` (the default exponents 2, 1.5 and 1 avoid it, as on the CPU).
//! The box mean of the glacial pass sums in f64 on the CPU; here it sums the differences from
//! the centre cell, Kahan-compensated, which keeps it within a few ulps of the mean. Where an
//! iteration branches on a comparison (the steepest receiver, a positive slope) a last-ulp
//! difference can pick another branch; the CPU path stays the reference, and the tolerance test
//! (`tests.rs`) bounds what that does to the terrain. One documented difference: the glacial
//! pass's lateral share is 1 where a neighbour's ice surface is not finite (a NaN there makes it
//! 1 on the CPU; an infinity could make it 0).
//!
//! # Residency
//! A window's arrays live on the device for the whole middle (14 arrays, 56 bytes a cell);
//! arrays swap by index, not by copy. The device is waited for at the end of each phase so that
//! the phases' times stay comparable with the CPU's.

use std::sync::Arc;
use std::time::Instant;

use deeptime_gpu::{Bands, Gpu, Kernel, Use, wgpu};

use super::glacial::ShapeStats;
use super::kernels::{Buf, Counts, PassDevice, Physics, Times, erosion_setup};

/// Device bytes per window cell: z, z2, a, a2, norm, own, kfac, flags, cap, tmult, eroded, sed,
/// sed2, zs.
const DEVICE_BYTES_PER_CELL: u64 = 14 * 4;

/// The parameters every kernel reads (binding 0), and the band (group 1).
const PRELUDE: &str = r#"
struct P {
    w: u32, h: u32, wraps: u32, top: u32,
    bottom: u32, radius: u32, pad0: u32, pad1: u32,
    dx: f32, sea: f32, flow_power: f32, area_exp: f32,
    slope_exp: f32, incision_m: f32, area_ref: f32, area_cap: f32,
    slope_ref: f32, slope_cap: f32, max_frac: f32, min_gradient: f32,
    thermal_slope: f32, thermal_rate: f32, cap_ref: f32, cap_area_exp: f32,
    cap_slope_exp: f32, deposit_frac: f32, g_strength: f32, g_share: f32,
    g_cap: f32, g_min_ice: f32, pad2: f32, pad3: f32,
}
struct Band { row0: u32, row1: u32, pad0: u32, pad1: u32 }
@group(0) @binding(0) var<uniform> p: P;
@group(1) @binding(0) var<uniform> band: Band;

const NONE: u32 = 0xffffffffu;
const BIG: f32 = 3.40282347e38;

// Column and row offsets of neighbour k: E, NE, N, NW, W, SW, S, SE (window.rs OFFSETS).
fn off(k: u32) -> vec2<i32> {
    var o = vec2<i32>(1, 1);
    switch k {
        case 0u: { o = vec2<i32>(1, 0); }
        case 1u: { o = vec2<i32>(1, -1); }
        case 2u: { o = vec2<i32>(0, -1); }
        case 3u: { o = vec2<i32>(-1, -1); }
        case 4u: { o = vec2<i32>(-1, 0); }
        case 5u: { o = vec2<i32>(-1, 1); }
        case 6u: { o = vec2<i32>(0, 1); }
        default: {}
    }
    return o;
}

fn step_len(k: u32) -> f32 {
    return select(1.0, 1.4142135623730951, (k & 1u) == 1u);
}

// Neighbour k of cell (i, j) under Win::nbrs' rules: none beyond a planet edge, rows clamped at
// an open edge, columns wrapped when the window holds every column, else clamped.
fn nb(i: u32, j: u32, k: u32) -> u32 {
    let o = off(k);
    var nj = i32(j) + o.y;
    if (nj < 0) {
        if (p.top != 0u) { return NONE; }
        nj = 0;
    } else if (nj >= i32(p.h)) {
        if (p.bottom != 0u) { return NONE; }
        nj = i32(p.h) - 1;
    }
    var ni = i32(i) + o.x;
    if (ni < 0) {
        ni = select(0, i32(p.w) - 1, p.wraps != 0u);
    } else if (ni >= i32(p.w)) {
        ni = select(i32(p.w) - 1, 0, p.wraps != 0u);
    }
    return u32(nj) * p.w + u32(ni);
}

// x^e as kernels.rs' pw (x >= 0 everywhere it is used).
fn pw(x: f32, e: f32) -> f32 {
    if (e == 1.0) { return x; }
    if (e == 2.0) { return x * x; }
    if (e == 1.5) { return x * sqrt(x); }
    if (x <= 0.0) { return 0.0; }
    return pow(x, e);
}

fn finite(x: f32) -> bool {
    return (bitcast<u32>(x) & 0x7f800000u) != 0x7f800000u;
}

// glacial.rs' col and row: columns wrap or clamp, rows always clamp.
fn gcol(i: u32, di: i32) -> u32 {
    let x = i32(i) + di;
    let w = i32(p.w);
    if (p.wraps != 0u) { return u32(((x % w) + w) % w); }
    return u32(clamp(x, 0, w - 1));
}

fn grow(j: u32, dj: i32) -> u32 {
    return u32(clamp(i32(j) + dj, 0, i32(p.h) - 1));
}

// Bits of `flags`: a lake, frozen (raised by the first flood), closed ground (`resample::basin`
// not OPEN: never sea).
const LAKE: u32 = 1u;
const FROZEN: u32 = 2u;
const CLOSED: u32 = 4u;

// resample::wet: under sea level and open to the sea (ledger: AMP-096).
fn wet(z: f32, f: u32) -> bool {
    return z <= p.sea && (f & CLOSED) == 0u;
}

fn sstep(a: f32, b: f32, x: f32) -> f32 {
    let t = clamp((x - a) / (b - a), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}
"#;

/// `kernels::norms`.
const NORMS: &str = r#"
@group(0) @binding(1) var<storage, read> z: array<f32>;
@group(0) @binding(2) var<storage, read> flags: array<u32>;
@group(0) @binding(3) var<storage, read_write> dst: array<f32>;
@compute @workgroup_size(16, 16)
fn norms(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let j = band.row0 + id.y;
    if (i >= p.w || j >= band.row1) { return; }
    let c = j * p.w + i;
    let zc = z[c];
    if (wet(zc, flags[c])) { dst[c] = 0.0; return; }
    var sum: f32 = 0.0;
    for (var k = 0u; k < 8u; k++) {
        let n = nb(i, j, k);
        if (n == NONE) { continue; }
        let s = (zc - z[n]) / (step_len(k) * p.dx);
        if (s > 0.0) { sum += pw(s, p.flow_power); }
    }
    dst[c] = sum;
}
"#;

/// `kernels::area_step`.
const AREA: &str = r#"
@group(0) @binding(1) var<storage, read> z: array<f32>;
@group(0) @binding(2) var<storage, read> norm: array<f32>;
@group(0) @binding(3) var<storage, read> a_in: array<f32>;
@group(0) @binding(4) var<storage, read> own: array<f32>;
@group(0) @binding(5) var<storage, read> flags: array<u32>;
@group(0) @binding(6) var<storage, read_write> dst: array<f32>;
@compute @workgroup_size(16, 16)
fn area(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let j = band.row0 + id.y;
    if (i >= p.w || j >= band.row1) { return; }
    let c = j * p.w + i;
    let zc = z[c];
    if (wet(zc, flags[c])) { dst[c] = own[c]; return; }
    var a: f32 = own[c];
    for (var k = 0u; k < 8u; k++) {
        let n = nb(i, j, k);
        if (n == NONE) { continue; }
        let zn = z[n];
        let s = (zn - zc) / (step_len(k) * p.dx);
        if (s > 0.0 && norm[n] > 0.0 && !wet(zn, flags[n])) {
            a += pw(s, p.flow_power) / norm[n] * a_in[n];
        }
    }
    dst[c] = a;
}
"#;

/// `kernels::erode_step` (`flags`: bit 0 a lake, bit 1 frozen, bit 2 closed ground).
const ERODE: &str = r#"
@group(0) @binding(1) var<storage, read> z: array<f32>;
@group(0) @binding(2) var<storage, read> a: array<f32>;
@group(0) @binding(3) var<storage, read> kfac: array<f32>;
@group(0) @binding(4) var<storage, read> flags: array<u32>;
@group(0) @binding(5) var<storage, read> cap: array<f32>;
@group(0) @binding(6) var<storage, read_write> dst: array<f32>;
@group(0) @binding(7) var<storage, read_write> eroded: array<f32>;
@compute @workgroup_size(16, 16)
fn erode(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let j = band.row0 + id.y;
    if (i >= p.w || j >= band.row1) { return; }
    let c = j * p.w + i;
    let zc = z[c];
    dst[c] = zc;
    let fc = flags[c];
    if (wet(zc, fc) || (fc & (LAKE | FROZEN)) != 0u) { return; }
    var smax: f32 = 0.0;
    var zr = zc;
    var dr = p.dx;
    var rn = c;
    for (var k = 0u; k < 8u; k++) {
        let n = nb(i, j, k);
        if (n == NONE) { continue; }
        let d = step_len(k) * p.dx;
        let zn = z[n];
        let s = (zc - zn) / d;
        if (s > smax) {
            smax = s;
            zr = zn;
            dr = d;
            rn = n;
        }
    }
    if (smax <= 0.0) { return; }
    let own_floor = (fc & CLOSED) != 0u && (flags[rn] & CLOSED) != 0u;
    let lowest = select(max(zr, p.sea), zr, own_floor) + p.min_gradient * dr;
    let room = zc - lowest;
    if (room <= 0.0) { return; }
    let spl = p.incision_m * kfac[c]
        * pw(min(a[c], p.area_cap) / p.area_ref, p.area_exp)
        * pw(min(smax, p.slope_cap) / p.slope_ref, p.slope_exp);
    let e = eroded[c];
    let delta = min(min(spl, room * p.max_frac), cap[c] - e);
    if (delta > 0.0) {
        dst[c] = zc - delta;
        eroded[c] = e + delta;
    }
}
"#;

/// `thermal::step`.
const THERMAL: &str = r#"
@group(0) @binding(1) var<storage, read> z: array<f32>;
@group(0) @binding(2) var<storage, read> tmult: array<f32>;
@group(0) @binding(3) var<storage, read_write> dst: array<f32>;
@compute @workgroup_size(16, 16)
fn thermal(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let j = band.row0 + id.y;
    if (i >= p.w || j >= band.row1) { return; }
    let c = j * p.w + i;
    let zc = z[c];
    let tc = tmult[c];
    var acc: f32 = 0.0;
    for (var k = 0u; k < 8u; k++) {
        let n = nb(i, j, k);
        if (n == NONE) { continue; }
        let d = step_len(k) * p.dx;
        let zn = z[n];
        let give = (zc - zn) - p.thermal_slope * tc * d;
        if (give > 0.0) { acc -= p.thermal_rate * give; }
        let take = (zn - zc) - p.thermal_slope * tmult[n] * d;
        if (take > 0.0) { acc += p.thermal_rate * take; }
    }
    dst[c] = zc + acc;
}
"#;

/// `kernels::deposit_step`.
const DEPOSIT: &str = r#"
@group(0) @binding(1) var<storage, read> z: array<f32>;
@group(0) @binding(2) var<storage, read> a: array<f32>;
@group(0) @binding(3) var<storage, read> norm: array<f32>;
@group(0) @binding(4) var<storage, read> sed_in: array<f32>;
@group(0) @binding(5) var<storage, read> flags: array<u32>;
@group(0) @binding(6) var<storage, read_write> dst: array<f32>;
@group(0) @binding(7) var<storage, read_write> sed_out: array<f32>;
@compute @workgroup_size(16, 16)
fn deposit(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let j = band.row0 + id.y;
    if (i >= p.w || j >= band.row1) { return; }
    let c = j * p.w + i;
    let zc = z[c];
    if (wet(zc, flags[c])) {
        dst[c] = zc;
        sed_out[c] = 0.0;
        return;
    }
    var load: f32 = 0.0;
    var smax: f32 = 0.0;
    var gap: f32 = BIG;
    for (var k = 0u; k < 8u; k++) {
        let n = nb(i, j, k);
        if (n == NONE) { continue; }
        let zn = z[n];
        let up = (zn - zc) / (step_len(k) * p.dx);
        if (up > 0.0) {
            gap = min(gap, zn - zc);
            if (norm[n] > 0.0 && !wet(zn, flags[n])) {
                load += pw(up, p.flow_power) / norm[n] * sed_in[n];
            }
        }
        let down = -up;
        if (down > smax) { smax = down; }
    }
    let capacity = p.cap_ref
        * pw(a[c] / p.area_ref, p.cap_area_exp)
        * pw(min(smax, p.slope_cap) / p.slope_ref, p.cap_slope_exp);
    var dep: f32 = 0.0;
    if (load > capacity) { dep = min(p.deposit_frac * (load - capacity), load); }
    dep = min(dep, 0.5 * gap);
    dst[c] = zc + dep;
    sed_out[c] = load - dep;
}
"#;

/// `glacial::depth`.
const DEPTH: &str = r#"
@group(0) @binding(1) var<storage, read> z: array<f32>;
@group(0) @binding(2) var<storage, read> zs: array<f32>;
@group(0) @binding(3) var<storage, read_write> dst: array<f32>;
@compute @workgroup_size(16, 16)
fn depth(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let j = band.row0 + id.y;
    if (i >= p.w || j >= band.row1) { return; }
    let c = j * p.w + i;
    let s = zs[c];
    var d: f32 = 0.0;
    if (finite(s)) { d = max(s - z[c], 0.0); }
    dst[c] = d;
}
"#;

/// `glacial::filter`'s two passes, maximum and mean, of radius `p.radius`.
const FILTER: &str = r#"
@group(0) @binding(1) var<storage, read> src: array<f32>;
@group(0) @binding(2) var<storage, read_write> dst: array<f32>;

fn at_x(i: u32, j: u32, di: i32) -> f32 { return src[j * p.w + gcol(i, di)]; }
fn at_y(i: u32, j: u32, dj: i32) -> f32 { return src[grow(j, dj) * p.w + i]; }

@compute @workgroup_size(16, 16)
fn max_x(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let j = band.row0 + id.y;
    if (i >= p.w || j >= band.row1) { return; }
    let r = i32(p.radius);
    var m: f32 = -BIG;
    for (var d = -r; d <= r; d++) { m = max(m, at_x(i, j, d)); }
    dst[j * p.w + i] = m;
}

@compute @workgroup_size(16, 16)
fn max_y(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let j = band.row0 + id.y;
    if (i >= p.w || j >= band.row1) { return; }
    let r = i32(p.radius);
    var m: f32 = -BIG;
    for (var d = -r; d <= r; d++) { m = max(m, at_y(i, j, d)); }
    dst[j * p.w + i] = m;
}

// The mean as the centre value plus the mean difference from it, the differences summed with
// Kahan's compensation: the CPU sums in f64.
@compute @workgroup_size(16, 16)
fn mean_x(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let j = band.row0 + id.y;
    if (i >= p.w || j >= band.row1) { return; }
    let r = i32(p.radius);
    let v0 = src[j * p.w + i];
    var s: f32 = 0.0;
    var comp: f32 = 0.0;
    for (var d = -r; d <= r; d++) {
        let y = (at_x(i, j, d) - v0) - comp;
        let t = s + y;
        comp = (t - s) - y;
        s = t;
    }
    dst[j * p.w + i] = v0 + s / f32(2 * r + 1);
}

@compute @workgroup_size(16, 16)
fn mean_y(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let j = band.row0 + id.y;
    if (i >= p.w || j >= band.row1) { return; }
    let r = i32(p.radius);
    let v0 = src[j * p.w + i];
    var s: f32 = 0.0;
    var comp: f32 = 0.0;
    for (var d = -r; d <= r; d++) {
        let y = (at_y(i, j, d) - v0) - comp;
        let t = s + y;
        comp = (t - s) - y;
        s = t;
    }
    dst[j * p.w + i] = v0 + s / f32(2 * r + 1);
}
"#;

/// `dmax = max(dmax, d)`.
const DMAX_FIX: &str = r#"
@group(0) @binding(1) var<storage, read_write> dmax: array<f32>;
@group(0) @binding(2) var<storage, read> d: array<f32>;
@compute @workgroup_size(16, 16)
fn dmax_fix(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let j = band.row0 + id.y;
    if (i >= p.w || j >= band.row1) { return; }
    let c = j * p.w + i;
    dmax[c] = max(dmax[c], d[c]);
}
"#;

/// `glacial::lateral` where the ice is, 0 elsewhere.
const LATERAL: &str = r#"
@group(0) @binding(1) var<storage, read> d: array<f32>;
@group(0) @binding(2) var<storage, read> zmean: array<f32>;
@group(0) @binding(3) var<storage, read> zs: array<f32>;
@group(0) @binding(4) var<storage, read_write> dst: array<f32>;
@compute @workgroup_size(16, 16)
fn lateral(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let j = band.row0 + id.y;
    if (i >= p.w || j >= band.row1) { return; }
    let c = j * p.w + i;
    if (d[c] <= 0.0) { dst[c] = 0.0; return; }
    let e = j * p.w + gcol(i, 1);
    let w = j * p.w + gcol(i, -1);
    let s = grow(j, 1) * p.w + i;
    let n = grow(j, -1) * p.w + i;
    if (!(finite(zs[e]) && finite(zs[w]) && finite(zs[s]) && finite(zs[n]))) {
        dst[c] = 1.0;
        return;
    }
    let gdx = (zs[e] - zmean[e]) - (zs[w] - zmean[w]);
    let gdy = (zs[s] - zmean[s]) - (zs[n] - zmean[n]);
    let gzx = zs[e] - zs[w];
    let gzy = zs[s] - zs[n];
    let nd = sqrt(gdx * gdx + gdy * gdy);
    let nz = sqrt(gzx * gzx + gzy * gzy);
    if (nd < 1.0e-3 || nz < 1.0e-3) { dst[c] = 1.0; return; }
    dst[c] = min(abs((gdx * gzy - gdy * gzx) / (nd * nz)), 1.0);
}
"#;

/// The cut under confined ice and the ridge weight of ice-free crests (`glacial::shape`).
/// `cut` holds the lateral share on entry, `free` the mean terrain.
const CUT_FREE: &str = r#"
@group(0) @binding(1) var<storage, read> d: array<f32>;
@group(0) @binding(2) var<storage, read> dmax: array<f32>;
@group(0) @binding(3) var<storage, read> zmax: array<f32>;
@group(0) @binding(4) var<storage, read> zs: array<f32>;
@group(0) @binding(5) var<storage, read> z: array<f32>;
@group(0) @binding(6) var<storage, read_write> cut: array<f32>;
@group(0) @binding(7) var<storage, read_write> free: array<f32>;
@compute @workgroup_size(16, 16)
fn cut_free(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let j = band.row0 + id.y;
    if (i >= p.w || j >= band.row1) { return; }
    let c = j * p.w + i;
    let dm = dmax[c];
    let mean_z = free[c];
    let lat = cut[c];
    var cv: f32 = 0.0;
    var fv: f32 = 0.0;
    if (dm > 0.0) {
        let g = sstep(p.g_min_ice, 3.0 * p.g_min_ice, dm);
        let dc = d[c];
        if (dc > 0.0) {
            let confined = clamp((zmax[c] - zs[c]) / (0.25 * dm), 0.0, 1.0);
            if (confined > 0.0) {
                let t = min(dc / dm, 1.0);
                cv = p.g_strength * g * confined * lat * dm * t * (1.0 - t);
            }
        } else {
            fv = g * sstep(0.0, 200.0, z[c] - mean_z);
        }
    }
    cut[c] = cv;
    free[c] = fv;
}
"#;

/// The carving and the ridges' share of it; `dz` keeps each cell's change for the statistics.
const APPLY: &str = r#"
@group(0) @binding(1) var<storage, read_write> z: array<f32>;
@group(0) @binding(2) var<storage, read> cut: array<f32>;
@group(0) @binding(3) var<storage, read> mean_cut: array<f32>;
@group(0) @binding(4) var<storage, read> free: array<f32>;
@group(0) @binding(5) var<storage, read> mean_free: array<f32>;
@group(0) @binding(6) var<storage, read_write> dz_out: array<f32>;
@compute @workgroup_size(16, 16)
fn apply(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let j = band.row0 + id.y;
    if (i >= p.w || j >= band.row1) { return; }
    let c = j * p.w + i;
    var dz = -cut[c];
    let f = free[c];
    if (p.g_share > 0.0 && f > 0.0) {
        dz += min(p.g_share * mean_cut[c] * f / max(mean_free[c], 0.25), p.g_cap);
    }
    if (dz != 0.0) { z[c] = z[c] + dz; }
    dz_out[c] = dz;
}
"#;

struct Kernels {
    norms: Kernel,
    area: Kernel,
    erode: Kernel,
    thermal: Kernel,
    deposit: Kernel,
    depth: Kernel,
    max_x: Kernel,
    max_y: Kernel,
    mean_x: Kernel,
    mean_y: Kernel,
    dmax_fix: Kernel,
    lateral: Kernel,
    cut_free: Kernel,
    apply: Kernel,
}

/// The amplifier's [`PassDevice`] on a wgpu device.
pub struct AmplifyGpu {
    gpu: Arc<Gpu>,
    k: Kernels,
}

impl std::fmt::Debug for AmplifyGpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AmplifyGpu")
            .field("gpu", &self.gpu)
            .finish_non_exhaustive()
    }
}

impl AmplifyGpu {
    /// Compiles the kernels on `gpu`.
    pub fn new(gpu: Arc<Gpu>) -> Result<AmplifyGpu, String> {
        use Use::{Read as R, Uniform as U, Write as W};
        let g = &gpu;
        let one = |src: &str, entry: &'static str, uses: &[Use]| -> Result<Kernel, String> {
            let m = g.module(entry, &format!("{PRELUDE}{src}"))?;
            g.kernel(&m, entry, uses)
        };
        let filter = g.module("filter", &format!("{PRELUDE}{FILTER}"))?;
        let k = Kernels {
            norms: one(NORMS, "norms", &[U, R, R, W])?,
            area: one(AREA, "area", &[U, R, R, R, R, R, W])?,
            erode: one(ERODE, "erode", &[U, R, R, R, R, R, W, W])?,
            thermal: one(THERMAL, "thermal", &[U, R, R, W])?,
            deposit: one(DEPOSIT, "deposit", &[U, R, R, R, R, R, W, W])?,
            depth: one(DEPTH, "depth", &[U, R, R, W])?,
            max_x: g.kernel(&filter, "max_x", &[U, R, W])?,
            max_y: g.kernel(&filter, "max_y", &[U, R, W])?,
            mean_x: g.kernel(&filter, "mean_x", &[U, R, W])?,
            mean_y: g.kernel(&filter, "mean_y", &[U, R, W])?,
            dmax_fix: one(DMAX_FIX, "dmax_fix", &[U, W, R])?,
            lateral: one(LATERAL, "lateral", &[U, R, R, R, W])?,
            cut_free: one(CUT_FREE, "cut_free", &[U, R, R, R, R, R, W, W])?,
            apply: one(APPLY, "apply", &[U, W, R, R, R, R, W])?,
        };
        Ok(AmplifyGpu { gpu, k })
    }
}

/// The kernels' parameters, as the WGSL struct `P` lays them out.
fn params(buf: &Buf, ph: &Physics) -> [u32; 32] {
    let win = &buf.win;
    let g = &ph.glacial;
    let f = |x: f32| x.to_bits();
    [
        win.w as u32,
        win.h as u32,
        u32::from(win.wraps()),
        u32::from(win.top_edge()),
        u32::from(win.bottom_edge()),
        g.radius_cells,
        0,
        0,
        f(ph.dx),
        f(ph.sea),
        f(ph.flow_power),
        f(ph.area_exp),
        f(ph.slope_exp),
        f(ph.incision_m),
        f(ph.area_ref_m2),
        f(ph.area_cap_m2),
        f(ph.slope_ref),
        f(ph.slope_cap),
        f(ph.max_frac),
        f(ph.min_gradient),
        f(ph.thermal_slope),
        f(ph.thermal_rate),
        f(ph.cap_ref_m),
        f(ph.cap_area_exp),
        f(ph.cap_slope_exp),
        f(ph.deposit_frac),
        f(g.strength as f32),
        f(g.ridge_share as f32),
        f(g.max_ridge_raise_m as f32),
        f(g.min_ice_m as f32),
        0,
        0,
    ]
}

/// One window's arrays on the device. `z`, `a` and `s` (the sediment) swap by index.
struct Tile<'a> {
    gpu: &'a Gpu,
    k: &'a Kernels,
    params: wgpu::Buffer,
    bands: Bands,
    z: [wgpu::Buffer; 2],
    a: [wgpu::Buffer; 2],
    s: [wgpu::Buffer; 2],
    norm: wgpu::Buffer,
    own: wgpu::Buffer,
    kfac: wgpu::Buffer,
    flags: wgpu::Buffer,
    cap: wgpu::Buffer,
    tmult: wgpu::Buffer,
    eroded: wgpu::Buffer,
    zs: wgpu::Buffer,
    zi: usize,
    ai: usize,
    si: usize,
}

impl<'a> Tile<'a> {
    fn new(
        dev: &'a AmplifyGpu,
        buf: &Buf,
        ph: &Physics,
        glacial: bool,
    ) -> Result<Tile<'a>, String> {
        let gpu = &*dev.gpu;
        let n = buf.win.cells();
        let bytes = n as u64 * 4;
        let st = |label: &str| gpu.storage(label, bytes);
        let t = Tile {
            gpu,
            k: &dev.k,
            params: gpu.uniform("amplify params", &params(buf, ph)),
            bands: gpu.bands(buf.win.w, buf.win.h),
            z: [st("z")?, st("z2")?],
            a: [st("a")?, st("a2")?],
            s: [st("sed")?, st("sed2")?],
            norm: st("norm")?,
            own: st("own")?,
            kfac: st("kfac")?,
            flags: st("flags")?,
            cap: st("cap")?,
            tmult: st("tmult")?,
            eroded: st("eroded")?,
            // Without glaciers the array is never read: a placeholder keeps the layout.
            zs: gpu.storage("zs", if glacial { bytes } else { 4 })?,
            zi: 0,
            ai: 0,
            si: 0,
        };
        gpu.upload(&t.z[0], &buf.z);
        gpu.upload(&t.a[0], &buf.a);
        gpu.upload(&t.own, &buf.own);
        gpu.upload(&t.kfac, &buf.kfac);
        gpu.upload(&t.cap, &buf.cap);
        gpu.upload(&t.tmult, &buf.tmult);
        if glacial {
            gpu.upload(&t.zs, &buf.zs);
        }
        let flags: Vec<u32> = buf
            .lake
            .iter()
            .zip(&buf.frozen)
            .zip(&buf.closed)
            .map(|((&l, &f), &b)| {
                u32::from(l != 0) | (u32::from(f != 0) << 1) | (u32::from(b != 0) << 2)
            })
            .collect();
        gpu.upload(&t.flags, &flags);
        Ok(t)
    }

    /// Records `steps` (a kernel and its buffers after the parameters) in one pass and submits.
    fn run(&self, steps: &[(&Kernel, &[&wgpu::Buffer])]) {
        let groups: Vec<_> = steps
            .iter()
            .map(|(k, bufs)| {
                let mut all = Vec::with_capacity(bufs.len() + 1);
                all.push(&self.params);
                all.extend_from_slice(bufs);
                self.gpu.bind(k, &all)
            })
            .collect();
        let mut enc = self.gpu.device.create_command_encoder(&Default::default());
        {
            let mut pass = enc.begin_compute_pass(&Default::default());
            for ((k, _), g) in steps.iter().zip(&groups) {
                self.bands.dispatch(&mut pass, k, g);
            }
        }
        self.gpu.queue.submit([enc.finish()]);
    }

    /// Norms, then one hop of the drainage area (`a` swaps).
    fn flow(&mut self) {
        let k = self.k;
        let (z, a_in, a_out) = (&self.z[self.zi], &self.a[self.ai], &self.a[1 - self.ai]);
        self.run(&[
            (&k.norms, &[z, &self.flags, &self.norm]),
            (&k.area, &[z, &self.norm, a_in, &self.own, &self.flags, a_out]),
        ]);
        self.ai = 1 - self.ai;
    }

    fn erode(&mut self) {
        self.flow();
        let k = self.k;
        let (z, z_out, a) = (&self.z[self.zi], &self.z[1 - self.zi], &self.a[self.ai]);
        self.run(&[(
            &k.erode,
            &[
                z,
                a,
                &self.kfac,
                &self.flags,
                &self.cap,
                z_out,
                &self.eroded,
            ],
        )]);
        self.zi = 1 - self.zi;
    }

    fn thermal(&mut self) {
        let k = self.k;
        let (z, z_out) = (&self.z[self.zi], &self.z[1 - self.zi]);
        self.run(&[(&k.thermal, &[z, &self.tmult, z_out])]);
        self.zi = 1 - self.zi;
    }

    fn deposit(&mut self) {
        self.flow();
        let k = self.k;
        let (z, z_out, a) = (&self.z[self.zi], &self.z[1 - self.zi], &self.a[self.ai]);
        let (s_in, s_out) = (&self.s[self.si], &self.s[1 - self.si]);
        self.run(&[(
            &k.deposit,
            &[z, a, &self.norm, s_in, &self.flags, z_out, s_out],
        )]);
        self.zi = 1 - self.zi;
        self.si = 1 - self.si;
    }

    /// `glacial::shape` with the same scratch roles: `[d, tmp, dmax, zmax, zmean, cut]` are the
    /// idle z, the idle a, the norms, both sediments and the cap. Leaves the cut in `cap` and
    /// each cell's change in `kfac`.
    fn glacial(&self, share: bool) {
        let k = self.k;
        let z = &self.z[self.zi];
        let (d, tmp, dmax) = (&self.z[1 - self.zi], &self.a[1 - self.ai], &self.norm);
        let (zmax, zmean, cut) = (&self.s[0], &self.s[1], &self.cap);
        let zs = &self.zs;
        self.run(&[
            (&k.depth, &[z, zs, d]),
            (&k.max_x, &[d, tmp]),
            (&k.max_y, &[tmp, zmax]),
            (&k.mean_x, &[zmax, tmp]),
            (&k.mean_y, &[tmp, dmax]),
            (&k.dmax_fix, &[dmax, d]),
            (&k.max_x, &[z, tmp]),
            (&k.max_y, &[tmp, cut]),
            (&k.mean_x, &[cut, tmp]),
            (&k.mean_y, &[tmp, zmax]),
            (&k.mean_x, &[z, tmp]),
            (&k.mean_y, &[tmp, zmean]),
            (&k.lateral, &[d, zmean, zs, cut]),
            (&k.cut_free, &[d, dmax, zmax, zs, z, cut, zmean]),
        ]);
        // `zmean` now holds the ridge weights, `d` and `dmax` take the box means.
        if share {
            self.run(&[
                (&k.mean_x, &[cut, tmp]),
                (&k.mean_y, &[tmp, d]),
                (&k.mean_x, &[zmean, tmp]),
                (&k.mean_y, &[tmp, dmax]),
            ]);
        }
        // The erosion is done with the erodibility: its array keeps the change.
        self.run(&[(&k.apply, &[z, cut, d, zmean, dmax, &self.kfac])]);
    }
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

impl PassDevice for AmplifyGpu {
    fn name(&self) -> String {
        self.gpu.describe()
    }

    fn host_bytes_per_cell(&self) -> u64 {
        // The upload's staging copy and the flags, and on a shared-memory adapter its buffers.
        let own = if self.gpu.shared_memory {
            DEVICE_BYTES_PER_CELL
        } else {
            0
        };
        own + 8
    }

    fn fits(&self, cells: usize) -> bool {
        let bytes = cells as u64 * 4;
        self.gpu.buffer_fits(bytes) && cells as u64 * DEVICE_BYTES_PER_CELL <= self.gpu.budget
    }

    fn middle(
        &self,
        buf: &mut Buf,
        ph: &Physics,
        counts: &Counts,
        times: &mut Times,
    ) -> Result<ShapeStats, String> {
        let gpu = &*self.gpu;
        let n = buf.win.cells();
        let glacial = counts.glacial > 0 && ph.glacial.active() && buf.zs.len() == n;

        let started = Instant::now();
        let kfac_saved = erosion_setup(buf, ph);
        let setup_ms = ms(started);

        let started = Instant::now();
        let mut t = Tile::new(self, buf, ph, glacial)?;
        if let Some(k) = kfac_saved {
            buf.kfac = k;
        }
        gpu.queue.submit([]);
        gpu.wait(None)?;
        times.transfer_ms += ms(started);

        let started = Instant::now();
        {
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            enc.clear_buffer(&t.eroded, 0, None);
            gpu.queue.submit([enc.finish()]);
        }
        for _ in 0..counts.erode {
            t.erode();
        }
        gpu.wait(None)?;
        times.erode_ms += setup_ms + ms(started);

        let started = Instant::now();
        if glacial {
            t.glacial(ph.glacial.ridge_share > 0.0);
            gpu.wait(None)?;
        }
        times.glacial_ms += ms(started);

        let started = Instant::now();
        for _ in 0..counts.thermal {
            t.thermal();
        }
        gpu.wait(None)?;
        times.thermal_ms += ms(started);

        let started = Instant::now();
        {
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            enc.copy_buffer_to_buffer(&t.eroded, 0, &t.s[0], 0, None);
            gpu.queue.submit([enc.finish()]);
            t.si = 0;
        }
        for _ in 0..counts.deposit {
            t.deposit();
        }
        gpu.wait(None)?;
        times.deposit_ms += ms(started);

        let started = Instant::now();
        let mut st = ShapeStats::default();
        if glacial {
            gpu.download(&t.cap, &mut buf.cap)?;
            // The change the glacial pass made, cell by cell.
            gpu.download(&t.kfac, &mut buf.z2)?;
            let (x0, y0, cw, ch) = buf.core;
            let w = buf.win.w;
            for j in y0..y0 + ch {
                for &dz in &buf.z2[j * w + x0..j * w + x0 + cw] {
                    if dz < 0.0 {
                        st.carved_m_cells += f64::from(-dz);
                        st.glacier_cells += 1;
                    } else if dz > 0.0 {
                        st.raised_m_cells += f64::from(dz);
                    }
                }
            }
        }
        gpu.download(&t.z[t.zi], &mut buf.z)?;
        gpu.download(&t.a[t.ai], &mut buf.a)?;
        gpu.download(&t.eroded, &mut buf.eroded)?;
        gpu.download(&t.s[t.si], &mut buf.sed)?;
        times.transfer_ms += ms(started);
        Ok(st)
    }
}
