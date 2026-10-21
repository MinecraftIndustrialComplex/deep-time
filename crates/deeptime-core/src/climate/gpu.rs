// SPDX-License-Identifier: LGPL-3.0-or-later
//! A climate snapshot's cells on the GPU (`docs/design/gpu-compute.md` §12 item 6): the twelve
//! months of every L1 cell (its sea-level temperature from the per-cell climate's tiles, the
//! belts' rain with the air's moisture and the orographic rate), then the annual fields, the
//! Thornthwaite PET, the equilibrium line, the Köppen class and the glacier flag, the loop of
//! `snapshot.rs` written again in WGSL, one invocation per cell.
//!
//! # What stays on the CPU
//! Everything before the loop (the EBM, the 2-D model, the air's march along each row, which is a
//! serial recurrence, the orographic transforms) and everything after it (the anomalies, the ice
//! cover and the band sheets, whose f64 sums feed the carbon cycle, in the CPU's order). The rows'
//! and columns' factors (latitude, the belts' shift, insolation, onshore share, the coarse
//! corners) are found on the CPU in f64 and uploaded. Only the per-cell climate with the belts'
//! rain (every preset's) runs here; the banded climate, the moisture budget and the upwind rain
//! stay on the CPU (`snapshot::compute_on`).
//!
//! # Precision (the error budget)
//! The months are f32, the CPU's f64. Budget, checked by `tests::the_gpu_cells_stay_within_their_
//! error_budget_of_the_cpu` on a real snapshot: temperatures within 0.01 K, precipitation (annual
//! and monthly) and aridity within 1e-4 relative, the sea ice within 1e-4, the ELA within 1e-4
//! relative or 1 cm, Köppen and the glacier flag equal except at a threshold. Where the CPU's
//! model has a step (the belt latitude crossing 0, which turns the trades; a month at 26.5 °C,
//! where Thornthwaite hands over to Willmott's form; the Köppen and ELA thresholds) a last-bit
//! difference can take the other side; the test counts those cells and holds the rest to the
//! budget.
//!
//! # Residency and sizes
//! Per call: the row table (46 words a row), the column table (16 a column) and the coarse
//! tables (37 words a coarse cell: the land tile, the sea tile and the sea ice of every month,
//! and the land/sea flags) once; then per chunk of rows (about [`CHUNK_CELLS`] cells) the cells'
//! inputs (14 planes plus one per wind regime with an orographic field, f32), one dispatch per
//! band, and the results back (10 planes, plus 24 when the months are kept). A chunk's arrays are
//! on the device at once, so the device memory is bounded by the chunk, not the planet.

use std::sync::Arc;
use std::time::Instant;

use deeptime_gpu::{Gpu, Kernel, Use, bytemuck};
use rayon::prelude::*;

use super::belts::{self, REGIMES};
use super::ebm::smoothstep;
use super::percell::DARK_W_M2;
use super::snapshot::{Cells, CellsDevice, CellsInput, CellsStats};

/// Cells in a chunk of rows (whole rows, a multiple of 16): 1 MiB a plane.
pub const CHUNK_CELLS: usize = 1 << 18;
/// The cells' input planes before the orographic ones: z, sea share, interior share, ΔSST,
/// basin, the air's eight fields, and the plateau's base height.
const CELL_PLANES: usize = 14;
/// t_mean, t_warm, t_cold, precip, aridity, ela, the three sea-ice covers, the class.
const OUT_PLANES: usize = 10;
/// The twelve months' sea-level temperatures and precipitation rates.
const MONTH_PLANES: usize = 24;
const ROW_W: usize = 46;
const COL_W: usize = 16;
/// The coarse tables: land tile, sea tile and sea ice of each month, then the flags.
const COARSE_PLANES: usize = 37;
const P_WORDS: usize = 48;

/// The kernel. `cin`: the chunk's input planes (stride `p.stride`); `rows`, `cols`, `coarse`:
/// the tables (u32; floats as their bits); `res`: the result planes; `mon`: the months'.
const CELLS: &str = r#"
struct P {
    w: u32, h: u32, cw: u32, nc: u32,
    stride: u32, row0: u32, oro_mask: u32, monthly: u32,
    gamma_ok: u32, sharp_on: u32, pad0: u32, pad1: u32,
    gamma: f32, oro_scale: f32, hydro: f32, rain_scale: f32,
    precip_min: f32, lee_floor: f32, oro_ref: f32, onshore: f32,
    subsidence: f32, west_wet: f32, plateau_m: f32, plateau_scale: f32,
    ice_temp: f32, inversion_c: f32, dark: f32, ela_offset: f32,
    sharp_a: f32, sharp_l0: f32, sharp_l1: f32, pad2: f32,
    hemi: array<vec4<f32>, 3>,
}
struct Band { row0: u32, row1: u32, pad0: u32, pad1: u32 }
@group(0) @binding(0) var<uniform> p: P;
@group(0) @binding(1) var<storage, read> cin: array<f32>;
@group(0) @binding(2) var<storage, read> rows: array<u32>;
@group(0) @binding(3) var<storage, read> cols: array<u32>;
@group(0) @binding(4) var<storage, read> coarse: array<u32>;
@group(0) @binding(5) var<storage, read_write> res: array<f32>;
@group(0) @binding(6) var<storage, read_write> mon: array<f32>;
@group(1) @binding(0) var<uniform> band: Band;

const NONE: u32 = 0xffffffffu;
const BIG: f32 = 3.40282347e38;
const ROW_W: u32 = 46u;
const COL_W: u32 = 16u;

// belts.rs `belt_precip`: the belts' rain by belt latitude, mm/yr.
var<private> BX: array<f32, 17> = array<f32, 17>(0.0, 4.0, 8.0, 12.0, 16.0, 20.0, 24.0, 28.0,
    32.0, 36.0, 42.0, 48.0, 54.0, 60.0, 70.0, 80.0, 90.0);
var<private> BY: array<f32, 17> = array<f32, 17>(2400.0, 2150.0, 1600.0, 1050.0, 650.0, 420.0,
    320.0, 330.0, 470.0, 700.0, 950.0, 1100.0, 1050.0, 850.0, 450.0, 250.0, 170.0);

fn rf(k: u32) -> f32 { return bitcast<f32>(rows[k]); }
fn colf(k: u32) -> f32 { return bitcast<f32>(cols[k]); }
fn cf(k: u32) -> f32 { return bitcast<f32>(coarse[k]); }

fn sstep(a: f32, b: f32, x: f32) -> f32 {
    let t = clamp((x - a) / (b - a), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

fn bump(x: f32, c: f32, hw: f32) -> f32 {
    let t = (x - c) / hw;
    if (abs(t) >= 1.0) { return 0.0; }
    let u = 1.0 - t * t;
    return u * u;
}

fn belt_precip(pb: f32) -> f32 {
    let a = min(abs(pb), 90.0);
    for (var k = 0u; k < 16u; k++) {
        if (a <= BX[k + 1u]) {
            return BY[k] + (BY[k + 1u] - BY[k]) * (a - BX[k]) / (BX[k + 1u] - BX[k]);
        }
    }
    return BY[16];
}

// belts.rs `subsidence_factor`.
fn subsidence(a: f32, pb: f32) -> f32 {
    let s = bump(abs(pb), 25.0, 13.0);
    return max(1.0 - p.subsidence * s * max(a, 0.0) + p.west_wet * s * max(-a, 0.0), 0.1);
}

// percell.rs `Sharpen::apply`.
fn sharpen(x: f32) -> f32 {
    if (p.sharp_on == 0u) { return clamp(x, 0.0, 1.0); }
    let l = 1.0 / (1.0 + exp(-p.sharp_a * (clamp(x, 0.0, 1.0) - 0.5)));
    return clamp((l - p.sharp_l0) / (p.sharp_l1 - p.sharp_l0), 0.0, 1.0);
}

// percell.rs `inversion` (DARK_W_M2 and winter_inversion_c: ledger CLM-134).
fn inversion(t_land: f32, q: f32, f1000: f32) -> f32 {
    let dark = clamp(1.0 - q / p.dark, 0.0, 1.0);
    let snow = 1.0 - sstep(p.ice_temp - 3.0, p.ice_temp + 3.0, t_land);
    let interior = clamp((f1000 - 0.5) / 0.5, 0.0, 1.0);
    return p.inversion_c * dark * snow * interior;
}

// coarse.rs `MaskedTaps`: the corners with the medium (flag `bit`) and weight, or the first
// corner with it when none has weight.
struct Taps { cc: array<u32, 4>, wt: array<f32, 4>, n: u32, wsum: f32, first: u32 }

fn taps(corner: array<u32, 4>, weight: array<f32, 4>, bit: u32) -> Taps {
    var c = corner;
    var w = weight;
    var t: Taps;
    t.n = 0u;
    t.wsum = 0.0;
    t.first = NONE;
    let flags = 36u * p.nc;
    for (var k = 0u; k < 4u; k++) {
        if (w[k] > 0.0 && (coarse[flags + c[k]] & bit) != 0u) {
            t.cc[t.n] = c[k];
            t.wt[t.n] = w[k];
            t.n += 1u;
            t.wsum += w[k];
        }
    }
    if (t.wsum <= 0.0) {
        for (var k = 0u; k < 4u; k++) {
            if ((coarse[flags + c[k]] & bit) != 0u) {
                t.first = c[k];
                break;
            }
        }
    }
    return t;
}

// `MaskedTaps::at` on the table at `base`: x the value, y 1 where the medium is around.
fn tap_at(t: Taps, base: u32) -> vec2<f32> {
    var tt = t;
    if (tt.wsum > 0.0) {
        var s: f32 = 0.0;
        for (var k = 0u; k < tt.n; k++) {
            s += tt.wt[k] * cf(base + tt.cc[k]);
        }
        return vec2<f32>(s / tt.wsum, 1.0);
    }
    if (tt.first != NONE) { return vec2<f32>(cf(base + tt.first), 1.0); }
    return vec2<f32>(0.0, 0.0);
}

// snapshot.rs `thornthwaite`, mm/yr.
fn thornthwaite(t: array<f32, 12>) -> f32 {
    var tt = t;
    var heat: f32 = 0.0;
    for (var m = 0u; m < 12u; m++) {
        if (tt[m] > 0.0) { heat += pow(tt[m] / 5.0, 1.514); }
    }
    if (heat <= 0.0) { return 0.0; }
    let a = 6.75e-7 * heat * heat * heat - 7.71e-5 * heat * heat + 1.792e-2 * heat + 0.49239;
    var pet: f32 = 0.0;
    for (var m = 0u; m < 12u; m++) {
        let x = tt[m];
        if (x > 0.0) {
            if (x > 26.5) {
                pet += -415.85 + 32.24 * x - 0.43 * x * x;
            } else {
                pet += 16.0 * pow(10.0 * x / heat, a);
            }
        }
    }
    return pet;
}

// glaciers.rs `ela_temperature`, °C.
fn ela_temperature(p_mm: f32) -> f32 {
    let pp = max(p_mm, 0.0);
    return (-296.0 + sqrt(max(296.0 * 296.0 - 36.0 * (645.0 - pp), 0.0))) / 18.0;
}

fn in_summer(m: u32, apr_sep: bool) -> bool { return (m >= 3u && m < 9u) == apr_sep; }

// koppen.rs `classify` (Peel et al. 2007; ledger BIO-002 to BIO-016): the index in `CODES`.
fn koppen(t: array<f32, 12>, pr: array<f32, 12>) -> u32 {
    var tt = t;
    var pp = pr;
    var tsum: f32 = 0.0;
    var map: f32 = 0.0;
    var thot: f32 = -BIG;
    var tcold: f32 = BIG;
    for (var m = 0u; m < 12u; m++) {
        tsum += tt[m];
        map += pp[m];
        thot = max(thot, tt[m]);
        tcold = min(tcold, tt[m]);
    }
    let mat = tsum / 12.0;
    var apr_sep: f32 = 0.0;
    for (var m = 3u; m < 9u; m++) { apr_sep += tt[m]; }
    let sas = apr_sep >= tsum - apr_sep;
    var ps: f32 = 0.0;
    var pw: f32 = 0.0;
    var psdry: f32 = BIG;
    var pswet: f32 = 0.0;
    var pwdry: f32 = BIG;
    var pwwet: f32 = 0.0;
    for (var m = 0u; m < 12u; m++) {
        if (in_summer(m, sas)) {
            ps += pp[m];
            psdry = min(psdry, pp[m]);
            pswet = max(pswet, pp[m]);
        } else {
            pw += pp[m];
            pwdry = min(pwdry, pp[m]);
            pwwet = max(pwwet, pp[m]);
        }
    }
    // ledger: BIO-004, BIO-005
    var pth: f32 = 2.0 * mat + 14.0;
    if (map > 0.0 && pw >= 0.7 * map) {
        pth = 2.0 * mat;
    } else if (map > 0.0 && ps >= 0.7 * map) {
        pth = 2.0 * mat + 28.0;
    }
    // ledger: BIO-002, BIO-005 (BWh 4, BWk 5, BSh 6, BSk 7)
    if (map < 10.0 * pth) {
        var k = 4u;
        if (map >= 5.0 * pth) { k += 2u; }
        if (mat < 18.0) { k += 1u; }
        return k;
    }
    // ledger: BIO-003 (ET 29, EF 30)
    if (thot < 10.0) { return select(30u, 29u, thot > 0.0); }
    // ledger: BIO-007 to BIO-010 (Af 0, Am 1, Aw 2, As 3)
    if (tcold >= 18.0) {
        var pdry: f32 = BIG;
        var driest = 0u;
        for (var m = 0u; m < 12u; m++) {
            if (pp[m] < pdry) {
                pdry = pp[m];
                driest = m;
            }
        }
        if (pdry >= 60.0) { return 0u; }
        if (pdry >= 100.0 - map / 25.0) { return 1u; }
        return select(2u, 3u, in_summer(driest, sas));
    }
    // ledger: BIO-011 to BIO-016 (s 0, w 1, f 2; a 0, b 1, c 2, d 3)
    let dry_summer = psdry < 40.0 && psdry < pwwet / 3.0;
    let dry_winter = pwdry < pswet / 10.0;
    var second = 2u;
    if (dry_summer && dry_winter) {
        second = select(0u, 1u, ps > pw);
    } else if (dry_summer) {
        second = 0u;
    } else if (dry_winter) {
        second = 1u;
    }
    var warm = 0u;
    for (var m = 0u; m < 12u; m++) {
        if (tt[m] > 10.0) { warm += 1u; }
    }
    let temperate = tcold > 0.0;
    var third = 2u;
    if (thot >= 22.0) {
        third = 0u;
    } else if (warm >= 4u) {
        third = 1u;
    } else if (!temperate && tcold < -38.0) {
        third = 3u;
    }
    if (temperate) { return 8u + 3u * second + third; }
    return 17u + 4u * second + third;
}

@compute @workgroup_size(16, 16)
fn cells(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    let jl = band.row0 + id.y;
    if (i >= p.w || jl >= band.row1) { return; }
    let j = p.row0 + jl;
    let cl = jl * p.w + i;
    let s = p.stride;
    let zc = cin[cl];
    let land = zc >= 0.0;
    let z = max(zc, 0.0);
    let sea_near = cin[s + cl];
    let interior = cin[2u * s + cl];
    let dsst = cin[3u * s + cl];
    let basin = cin[4u * s + cl];
    var air: array<f32, 8>;
    for (var k = 0u; k < 8u; k++) { air[k] = cin[(5u + k) * s + cl]; }
    // The plateau's base height (0 at sea; snapshot.rs `mee_z`).
    let zb = cin[13u * s + cl];
    var oro: array<f32, 8>;
    for (var r = 0u; r < 8u; r++) {
        if ((p.oro_mask & (1u << r)) != 0u) {
            oro[r] = cin[(14u + countOneBits(p.oro_mask & ((1u << r) - 1u))) * s + cl];
        }
    }
    let rb = j * ROW_W;
    let cb = i * COL_W;
    let phi = rf(rb);
    let sh_t = rf(rb + 1u);
    let sh_f = rf(rb + 2u);
    let north_row = rows[rb + 3u] != 0u;
    let crossed_s = rf(rb + 4u);
    // The four coarse cells around the cell and their weights (coarse.rs `corners`).
    let i0 = cols[cb + 12u];
    let i1 = cols[cb + 13u];
    let fx = colf(cb + 14u);
    let j0 = rows[rb + 29u];
    let j1 = rows[rb + 30u];
    let fy = rf(rb + 31u);
    let home = rows[rb + 32u] * p.cw + cols[cb + 15u];
    let corner = array<u32, 4>(j0 * p.cw + i0, j0 * p.cw + i1, j1 * p.cw + i0, j1 * p.cw + i1);
    let weight = array<f32, 4>((1.0 - fx) * (1.0 - fy), fx * (1.0 - fy), (1.0 - fx) * fy, fx * fy);
    let lt = taps(corner, weight, 1u);
    let st = taps(corner, weight, 2u);
    let nc = p.nc;
    let plat = exp(-max(z - p.plateau_m, 0.0) / p.plateau_scale);
    var tm: array<f32, 12>;
    var tsl_m: array<f32, 12>;
    var pm: array<f32, 12>;
    var ice: array<f32, 12>;
    for (var m = 0u; m < 12u; m++) {
        let pb = phi - (colf(cb + m) * (1.0 - sh_t) + p.hemi[m / 4u][m % 4u] * sh_t) * sh_f;
        // belts.rs `regime_weights`.
        let a = abs(pb);
        let trade = 1.0 - sstep(27.0, 33.0, a);
        let polar = sstep(57.0, 63.0, a);
        let west = max(1.0 - trade - polar, 0.0);
        let north = pb >= 0.0;
        var crossed: f32 = 0.0;
        if (north_row != north) { crossed = crossed_s; }
        var wr: array<f32, 8>;
        if (north) {
            wr[0] = trade * (1.0 - crossed);
            wr[7] = trade * crossed;
            wr[2] = west;
            wr[4] = polar;
        } else {
            wr[1] = trade * (1.0 - crossed);
            wr[6] = trade * crossed;
            wr[3] = west;
            wr[5] = polar;
        }
        // The sea-level temperature (percell.rs `tiles_at`, `sea_tile_at`, `ice_at_taps`).
        let l_at = tap_at(lt, m * nc);
        let s_at = tap_at(st, (12u + m) * nc);
        var tsl: f32;
        if (land) {
            var t_land = cf(m * nc + home);
            var t_sea = cf((12u + m) * nc + home);
            if (l_at.y > 0.5 && s_at.y > 0.5) {
                t_land = l_at.x;
                t_sea = s_at.x;
            } else if (l_at.y > 0.5) {
                t_land = l_at.x;
                t_sea = l_at.x;
            } else if (s_at.y > 0.5) {
                t_land = s_at.x;
                t_sea = s_at.x;
            }
            let inv = inversion(t_land, rf(rb + 5u + m), interior);
            tsl = t_land + sea_near * (t_sea - t_land) - inv;
        } else {
            let i_at = tap_at(st, (24u + m) * nc);
            var x = cf((24u + m) * nc + home);
            if (i_at.y > 0.5) { x = i_at.x; }
            ice[m] = sharpen(clamp(x, 0.0, 1.0));
            tsl = cf((12u + m) * nc + home);
            if (s_at.y > 0.5) {
                tsl = s_at.x;
            } else if (l_at.y > 0.5) {
                tsl = l_at.x;
            }
        }
        // The plateau's warmth (snapshot.rs; ledger: CLM-225): the row's warmth per metre of base
        // height this month times the base height. The rain's moisture stays the lowland air's.
        let tg = tsl + rf(rb + 34u + m) * zb;
        tsl_m[m] = tg;
        tm[m] = tg - p.gamma * z;
        // snapshot.rs `circ_at`.
        let fe = wr[2] + wr[3] + wr[6] + wr[7];
        let fw = 1.0 - fe;
        let fw_q = max(fw, rf(rb + 17u + m) * p.onshore * bump(abs(pb), 30.0, 15.0));
        let fe_q = 1.0 - fw_q;
        let q = fe_q * air[0] + fw_q * air[1];
        var a_eff = basin;
        var d_eff = dsst;
        if (land) {
            let mar = fe * air[2] + fw * air[3];
            let bas_q = fe_q * air[4] + fw_q * air[5];
            let mar_q = fe_q * air[2] + fw_q * air[3];
            let dcar = fe * air[6] + fw * air[7];
            a_eff = bas_q * mar_q;
            d_eff = mar * dcar;
        }
        // ledger: MOI-063 (the 0.07/K)
        let base0 = belt_precip(pb) * subsidence(a_eff, pb) * exp(0.07 * d_eff) * plat;
        var o: f32 = 0.0;
        for (var r = 0u; r < 8u; r++) {
            if (wr[r] > 0.0 && (p.oro_mask & (1u << r)) != 0u) { o += wr[r] * oro[r]; }
        }
        let ccf = clamp(exp(0.067 * (tsl - p.oro_ref)), 0.05, 3.0);
        let base = base0 * q * p.rain_scale;
        let oro_mm = o * p.oro_scale * q * ccf;
        var rate: f32;
        if (oro_mm >= 0.0) {
            rate = base + oro_mm;
        } else {
            rate = base * max(1.0 + oro_mm / max(base, 1e-9), p.lee_floor);
        }
        pm[m] = max(rate, p.precip_min) * p.hydro / 12.0;
    }
    var annual: f32 = 0.0;
    var tsum: f32 = 0.0;
    var warm: f32 = -BIG;
    var cold: f32 = BIG;
    var summer: f32 = -BIG;
    for (var m = 0u; m < 12u; m++) {
        annual += pm[m];
        tsum += tm[m];
        warm = max(warm, tm[m]);
        cold = min(cold, tm[m]);
        summer = max(summer, (tsl_m[m] + tsl_m[(m + 1u) % 12u] + tsl_m[(m + 2u) % 12u]) / 3.0);
    }
    let pet = thornthwaite(tm);
    var aridity: f32 = 10.0;
    if (pet > 0.0) { aridity = min(annual / pet, 10.0); }
    let t_ela = ela_temperature(annual) + p.ela_offset;
    var ela: f32;
    if (p.gamma_ok != 0u) {
        ela = (summer - t_ela) / p.gamma;
    } else if (summer < t_ela) {
        ela = -1.0e6;
    } else {
        ela = 1.0e6;
    }
    var cls = 31u;
    var im = vec3<f32>(0.0, 0.0, 0.0);
    if (land) {
        cls = koppen(tm, pm) + select(0u, 32u, z > ela);
    } else {
        var isum: f32 = 0.0;
        var imax: f32 = 0.0;
        var imin: f32 = 1.0;
        for (var m = 0u; m < 12u; m++) {
            isum += ice[m];
            imax = max(imax, ice[m]);
            imin = min(imin, ice[m]);
        }
        im = vec3<f32>(isum / 12.0, imax, imin);
    }
    res[cl] = tsum / 12.0;
    res[s + cl] = warm;
    res[2u * s + cl] = cold;
    res[3u * s + cl] = annual;
    res[4u * s + cl] = aridity;
    res[5u * s + cl] = ela;
    res[6u * s + cl] = im.x;
    res[7u * s + cl] = im.y;
    res[8u * s + cl] = im.z;
    res[9u * s + cl] = f32(cls);
    if (p.monthly != 0u) {
        for (var m = 0u; m < 12u; m++) {
            mon[m * s + cl] = tsl_m[m];
            mon[(12u + m) * s + cl] = pm[m] * 12.0;
        }
    }
}
"#;

/// The snapshot's [`CellsDevice`] on a wgpu device.
pub struct CellsGpu {
    gpu: Arc<Gpu>,
    kernel: Kernel,
    chunk_cells: usize,
}

impl std::fmt::Debug for CellsGpu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CellsGpu")
            .field("gpu", &self.gpu)
            .field("chunk_cells", &self.chunk_cells)
            .finish_non_exhaustive()
    }
}

impl CellsGpu {
    /// Compiles the kernel on `gpu`.
    pub fn new(gpu: Arc<Gpu>) -> Result<CellsGpu, String> {
        use Use::{Read as R, Uniform as U, Write as W};
        let m = gpu.module("climate cells", CELLS)?;
        let kernel = gpu.kernel(&m, "cells", &[U, R, R, R, R, W, W])?;
        Ok(CellsGpu {
            gpu,
            kernel,
            chunk_cells: CHUNK_CELLS,
        })
    }

    /// The same with chunks of about `cells` cells (the tests' several chunks on a small grid).
    pub fn with_chunk_cells(mut self, cells: usize) -> CellsGpu {
        self.chunk_cells = cells.max(1);
        self
    }

    /// Rows per chunk: whole rows, a multiple of 16 where the grid has them, the largest plane
    /// set within the adapter's buffer limit.
    fn chunk_rows(&self, w: usize, h: usize, planes: usize) -> usize {
        let wg = deeptime_gpu::WORKGROUP as usize;
        let mut rows = (self.chunk_cells / w.max(1) / wg * wg).max(wg).min(h);
        while rows > wg && !self.gpu.buffer_fits((planes * rows * w * 4) as u64) {
            rows = (rows / 2 / wg * wg).max(wg);
        }
        rows
    }
}

fn bits(x: f64) -> u32 {
    (x as f32).to_bits()
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1000.0
}

/// The kernel's parameters, as the WGSL struct `P` lays them out.
fn params(ci: &CellsInput<'_>, stride: usize, row0: usize, oro_mask: u32) -> [u32; P_WORDS] {
    let p = ci.p;
    let g = &ci.pc.grid;
    let [k, a, l0, l1] = ci.sharp.parts();
    let mut v = [0u32; P_WORDS];
    v[..12].copy_from_slice(&[
        ci.w as u32,
        ci.h as u32,
        g.w as u32,
        (g.w * g.h) as u32,
        stride as u32,
        row0 as u32,
        oro_mask,
        u32::from(ci.monthly),
        u32::from(ci.gamma > 1e-9),
        u32::from(k > 1.0),
        0,
        0,
    ]);
    let floats = [
        ci.gamma,
        ci.oro_scale,
        ci.hydro,
        p.belts_rain_scale,
        p.precip_min_mm,
        p.oro_lee_floor,
        p.oro_ref_temp_c,
        p.onshore,
        p.subsidence,
        p.west_flank_wet,
        p.plateau_m,
        p.plateau_scale_m,
        p.ice_temp_c,
        p.winter_inversion_c,
        DARK_W_M2,
        p.ela_offset_c,
        a,
        l0,
        l1,
        0.0,
    ];
    for (o, x) in v[12..32].iter_mut().zip(floats) {
        *o = bits(x);
    }
    for (o, &x) in v[32..44].iter_mut().zip(&ci.hemi) {
        *o = bits(x);
    }
    v
}

/// Per row: φ, the shift's t and f, φ ≥ 0, the crossed trades' share, the insolation and onshore
/// share of each month, the coarse rows j0, j1, fy and the row's coarse row, then (from word 34)
/// the plateau's warmth per metre of base height of each month.
fn row_table(ci: &CellsInput<'_>) -> Vec<u32> {
    let g = &ci.pc.grid;
    let mut v = vec![0u32; ci.h * ROW_W];
    for (j, r) in v.chunks_mut(ROW_W).enumerate() {
        let phi = ci.lat[j];
        let (t, f) = belts::shift_parts(phi);
        let (j0, j1, fy) = g.axis_y(j);
        r[0] = bits(phi);
        r[1] = bits(t);
        r[2] = bits(f);
        r[3] = u32::from(phi >= 0.0);
        r[4] = bits(smoothstep(0.0, 3.0, phi.abs()));
        for m in 0..12 {
            r[5 + m] = bits(ci.q_month[j][m]);
            r[17 + m] = bits(ci.onshore[j][m]);
        }
        r[29] = j0 as u32;
        r[30] = j1 as u32;
        r[31] = bits(fy);
        r[32] = (j / g.k) as u32;
        for m in 0..12 {
            r[34 + m] = bits(ci.mee_k[j][m]);
        }
    }
    v
}

/// Per column: the ITCZ of each month, the coarse columns i0, i1, fx and the column's coarse
/// column.
fn col_table(ci: &CellsInput<'_>) -> Vec<u32> {
    let g = &ci.pc.grid;
    let mut v = vec![0u32; ci.w * COL_W];
    for (i, r) in v.chunks_mut(COL_W).enumerate() {
        for (o, &x) in r[..12].iter_mut().zip(&ci.itcz[i]) {
            *o = bits(x);
        }
        let (i0, i1, fx) = g.axis_x(i);
        r[12] = i0 as u32;
        r[13] = i1 as u32;
        r[14] = bits(fx);
        r[15] = (i / g.k) as u32;
    }
    v
}

/// The land tiles, sea tiles and sea ice of each month, then the flags (1: land, 2: sea).
fn coarse_table(ci: &CellsInput<'_>) -> Vec<u32> {
    let pc = ci.pc;
    let nc = pc.grid.w * pc.grid.h;
    let mut v = vec![0u32; COARSE_PLANES * nc];
    for (t, table) in pc.monthly_tables().iter().enumerate() {
        for (m, month) in table.iter().enumerate() {
            let o = (t * 12 + m) * nc;
            for (x, &y) in v[o..o + nc].iter_mut().zip(month) {
                *x = bits(y);
            }
        }
    }
    for (cc, x) in v[36 * nc..].iter_mut().enumerate() {
        *x = u32::from(pc.has_land(cc)) | (u32::from(pc.has_sea(cc)) << 1);
    }
    v
}

/// The input planes of rows `r0..r1` into `buf` (one plane per `r1 − r0` rows): z (−1 at sea),
/// the sea and interior shares, ΔSST, the basin, the air's eight fields, the plateau's base
/// height, then the orographic rate of each regime in `slots`.
fn pack(ci: &CellsInput<'_>, slots: &[usize], r0: usize, r1: usize, m: usize, buf: &mut [f32]) {
    let (a, b) = (r0 * ci.w, r1 * ci.w);
    debug_assert_eq!(b - a, m);
    let narrow = |plane: &mut [f32], v: &[f64]| {
        for (o, &x) in plane.iter_mut().zip(&v[a..b]) {
            *o = x as f32;
        }
    };
    buf.par_chunks_mut(m)
        .enumerate()
        .for_each(|(k, plane)| match k {
            0 => {
                for (o, c) in plane.iter_mut().zip(a..b) {
                    *o = if ci.land[c] { ci.z[c] as f32 } else { -1.0 };
                }
            }
            1 => narrow(plane, ci.sea_near),
            2 => narrow(plane, ci.interior),
            3 => narrow(plane, ci.dsst),
            4 => narrow(plane, ci.basin),
            5..13 => narrow(plane, &ci.air[k - 5]),
            13 => narrow(plane, ci.mee_z),
            _ => plane.copy_from_slice(&ci.oro[slots[k - CELL_PLANES]][a..b]),
        });
}

/// Plane `k` of a chunk's results (planes of `m` cells).
fn plane(v: &[f32], k: usize, m: usize) -> &[f32] {
    &v[k * m..(k + 1) * m]
}

/// A chunk's `m` cells' results into `out` from cell `at`.
fn unpack(res: &[f32], mon: &[f32], m: usize, at: usize, out: &mut Cells) {
    for (k, dst) in [
        &mut out.t_mean,
        &mut out.t_warm,
        &mut out.t_cold,
        &mut out.precip,
        &mut out.aridity,
        &mut out.ela,
        &mut out.sea_ice,
        &mut out.sea_ice_max,
        &mut out.sea_ice_min,
    ]
    .into_iter()
    .enumerate()
    {
        dst[at..at + m].copy_from_slice(plane(res, k, m));
    }
    for (c, &x) in plane(res, 9, m).iter().enumerate() {
        let v = x as u32;
        out.koppen[at + c] = (v & 31) as u8;
        out.glacier[at + c] = (v >> 5) as u8;
    }
    for (k, (t, p)) in out.t_sl_month.iter_mut().zip(&mut out.p_month).enumerate() {
        t[at..at + m].copy_from_slice(plane(mon, k, m));
        p[at..at + m].copy_from_slice(plane(mon, 12 + k, m));
    }
}

impl CellsDevice for CellsGpu {
    fn name(&self) -> String {
        self.gpu.describe()
    }

    fn host_bytes(&self, w: usize, h: usize, monthly: bool) -> u64 {
        let planes = CELL_PLANES + REGIMES;
        let cap = (self.chunk_rows(w, h, planes) * w) as u64;
        let back = (OUT_PLANES + if monthly { MONTH_PLANES } else { 0 }) as u64 * cap * 4;
        let chunk = planes as u64 * cap * 4 + back;
        // The packed inputs and read-back copies, the download's staging, and on a device
        // sharing the host's memory its buffers.
        let staging = back.min(64 << 20);
        chunk + staging + if self.gpu.shared_memory { chunk } else { 0 }
    }

    fn cells(&self, ci: &CellsInput<'_>, st: &mut CellsStats) -> Result<Cells, String> {
        let gpu = &*self.gpu;
        if gpu.is_lost() {
            return Err("the device was lost".into());
        }
        let started = Instant::now();
        let (w, h) = (ci.w, ci.h);
        let n = w * h;
        let g = &ci.pc.grid;
        if g.fw != w || g.fh != h || ci.itcz.len() != w || ci.lat.len() != h {
            return Err(format!(
                "the coarse grid's fine grid {}x{} is not the snapshot's {w}x{h}",
                g.fw, g.fh
            ));
        }
        let slots: Vec<usize> = (0..REGIMES).filter(|&r| !ci.oro[r].is_empty()).collect();
        if slots.iter().any(|&r| ci.oro[r].len() != n) {
            return Err("an orographic field is not the grid's size".into());
        }
        let oro_mask = slots.iter().fold(0u32, |m, &r| m | (1 << r));
        let planes = CELL_PLANES + slots.len();
        let month_planes = if ci.monthly { MONTH_PLANES } else { 0 };
        let rows_per = self.chunk_rows(w, h, planes);
        let cap = rows_per * w;

        let rows = row_table(ci);
        let cols = col_table(ci);
        let coarse = coarse_table(ci);
        let tables = (rows.len() + cols.len() + coarse.len()) as u64 * 4;
        let device_bytes =
            ((planes + OUT_PLANES + month_planes) * cap) as u64 * 4 + tables + 4 * P_WORDS as u64;
        if device_bytes > gpu.budget {
            return Err(format!(
                "{device_bytes} bytes exceed the device budget of {} bytes",
                gpu.budget
            ));
        }
        let b_rows = gpu.storage("climate rows", rows.len() as u64 * 4)?;
        let b_cols = gpu.storage("climate cols", cols.len() as u64 * 4)?;
        let b_coarse = gpu.storage("climate coarse", coarse.len() as u64 * 4)?;
        let b_in = gpu.storage("climate cells in", (planes * cap) as u64 * 4)?;
        let b_res = gpu.storage("climate cells out", (OUT_PLANES * cap) as u64 * 4)?;
        let b_mon = gpu.storage("climate months", (month_planes * cap) as u64 * 4)?;
        gpu.upload(&b_rows, &rows);
        gpu.upload(&b_cols, &cols);
        gpu.upload(&b_coarse, &coarse);
        let b_params = gpu.uniform("climate cells params", &[0u32; P_WORDS]);
        let group = gpu.bind(
            &self.kernel,
            &[
                &b_params, &b_in, &b_rows, &b_cols, &b_coarse, &b_res, &b_mon,
            ],
        );
        let month_out = || -> Vec<Vec<f32>> {
            if ci.monthly {
                (0..12).map(|_| vec![0.0; n]).collect()
            } else {
                Vec::new()
            }
        };
        let mut out = Cells {
            t_mean: vec![0.0; n],
            t_warm: vec![0.0; n],
            t_cold: vec![0.0; n],
            precip: vec![0.0; n],
            aridity: vec![0.0; n],
            ela: vec![0.0; n],
            koppen: vec![0; n],
            glacier: vec![0; n],
            sea_ice: vec![0.0; n],
            sea_ice_max: vec![0.0; n],
            sea_ice_min: vec![0.0; n],
            t_sl_month: month_out(),
            p_month: month_out(),
        };
        let mut packed = vec![0.0f32; planes * cap];
        let mut res = vec![0.0f32; OUT_PLANES * cap];
        let mut mon = vec![0.0f32; month_planes * cap];
        st.bytes_up += tables;
        st.device_bytes = st.device_bytes.max(device_bytes);
        st.upload_ms += ms(started);

        for r0 in (0..h).step_by(rows_per) {
            let r1 = (r0 + rows_per).min(h);
            // The chunk's cells, also its planes' stride (a short last chunk moves only its own).
            let m = (r1 - r0) * w;
            let t = Instant::now();
            let up = planes * m;
            pack(ci, &slots, r0, r1, m, &mut packed[..up]);
            let pv = params(ci, m, r0, oro_mask);
            gpu.queue
                .write_buffer(&b_params, 0, bytemuck::cast_slice(&pv[..]));
            gpu.upload(&b_in, &packed[..up]);
            st.bytes_up += up as u64 * 4;
            st.upload_ms += ms(t);

            let t = Instant::now();
            let bands = gpu.bands(w, r1 - r0);
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            {
                let mut pass = enc.begin_compute_pass(&Default::default());
                bands.dispatch(&mut pass, &self.kernel, &group);
            }
            let idx = gpu.queue.submit([enc.finish()]);
            gpu.wait(Some(idx))?;
            st.kernel_ms += ms(t);
            st.dispatches += bands.count() as u32;
            st.chunks += 1;

            let t = Instant::now();
            let down = OUT_PLANES * m;
            gpu.download(&b_res, &mut res[..down])?;
            st.bytes_down += down as u64 * 4;
            let down_m = month_planes * m;
            gpu.download(&b_mon, &mut mon[..down_m])?;
            st.bytes_down += down_m as u64 * 4;
            unpack(&res, &mon, m, r0 * w, &mut out);
            st.download_ms += ms(t);
        }
        gpu.check()?;
        Ok(out)
    }
}
