// SPDX-License-Identifier: LGPL-3.0-or-later
//! Physical climate → the five vanilla biome inputs (plan §7.2, work item P1-09).
//!
//! The bake writes `climate/{temperature, vegetation, continents, erosion, ridges}` on L2 (i16,
//! vanilla parameter × 10,000): what the noise router's climate functions read (plan §7.1), so the
//! pack's biome source — vanilla's `OverworldBiomeBuilder`, Terralith's parameter list, Biomes O'
//! Plenty through TerraBlender — places its biomes where the simulated climate says. Everything
//! here is in **band coordinates**: `b ∈ [0, n]` over a parameter's `n` bands (b = 2.5 is the middle
//! of the third band), turned into a parameter value by the band edges of the reference
//! (`data/climate_mapping/default.json`: `OverworldBiomeBuilder`'s edges, 1.21.1). The game maps
//! these values once more, per world, onto the pack's own bands and distributions
//! (`ClimateMappingBuilder`, stored as `climate_mapping.json`); it reads the reference this bake
//! used from `notes.bake.climate.reference`.
//!
//! **Temperature** (5 bands: frozen, cold, temperate, warm, hot) from the Köppen class and the
//! warmest/coldest-month temperatures, so that each main group lands in its band:
//!
//! | Köppen | band coordinate | vanilla middle biomes there |
//! |---|---|---|
//! | EF, ET | 0 … 0.75 by the warmest month (−20 → 10 °C) | snowy plains, ice spikes |
//! | D | 0.75 … 2 by the warmest (10 → 22 °C, weight 0.65) and coldest month (−30 → 0 °C) | snowy taiga (Dfc's cold edge), taiga, plains, forest |
//! | C | 2 … 2.95 by the coldest month (0 → 18 °C) | plains, forest, birch, dark forest |
//! | A, BSh | 3 … 3.9 (coldest month 18 → 28 °C; BSh by the mean 18 → 30 °C) | savanna, forest, jungle |
//! | BW | 4.05 … 4.95 by the mean (5 → 30 °C) | desert (badlands on plateaus) |
//! | BSk | as C or D by its coldest month | plains, meadow |
//!
//! Land above its equilibrium line (`climate/glacier`) is capped at 0.35 (ice). The sea's own
//! temperature (sea ice below −2 °C → frozen, 6, 18, 25 °C → cold, temperate, warm, hot oceans)
//! drives the ocean biomes.
//!
//! **Vegetation** (humidity, 5 bands) from the aridity index P/PET — 0.03, 0.2, 0.5, 1, 2, 5 →
//! 0 … 5 on a log scale (UNEP's hyper-arid | arid | semi-arid | dry sub-humid | humid) — held
//! inside each class's range: BW ≤ 0.95 (the driest band), BS 1.05 … 1.95, Af ≥ 4.1 (jungle),
//! Am ≥ 3.2, Aw/As and Cs squeezed into 1.05 … 2.5 (savanna, plains, open forest), the
//! subarctic D (Dsc, Dsd, Dwc, Dwd, Dfc, Dfd) ≥ 3.05 (taiga), other C/D ≥ 2.05, ET/EF ≤ 1.95 (no
//! trees: vanilla picks snowy taiga at the frozen band's third humidity band as a variant, and grove on
//! its slopes).
//!
//! These two are computed per L1 cell (where the climate lives), filled across the other medium
//! by nearest value (so a coastal land cell interpolates between land values only), resampled
//! to L2 and chosen by the L2 land mask.
//!
//! **Continentalness** from the distance to the sea on land — 0, `beach_blocks`, 200, 1,000,
//! 5,000 blocks → the coast band's bottom, near-inland, mid-inland, far-inland, top — and from
//! the depth at sea — 0, 200, 2,500, 6,000 m → coast | ocean, the shelf, ocean | deep ocean, the
//! abyss.
//!
//! **Erosion** from the local relief (range of the ground, the sea surface counted as ground, in
//! a `terrain_window_blocks` window, box-smoothed): its rank among the land cells, high relief
//! first, read through the reference's land erosion distribution below the swamp band (E6).
//! Wet (vegetation ≥ 2.5), flat (the flattest 35 %), low (< 150 m) land that is not frozen goes
//! into E6 in proportion to `wetlands` — swamps and mangroves only where they belong (vanilla
//! puts E6 at hot temperatures to mangrove swamp, whatever the humidity). The deep sea keeps E4 or
//! more, away from Terralith's sky-island band.
//!
//! **Weirdness** from the ridge/valley index (the ground minus its window mean): its rank among the
//! land cells read through the reference's peaks-and-valleys distribution above −0.8, turned into
//! |weirdness| on the rising branch (|w| = (PV + 1)/3), signed by a value noise of wavelength
//! `variant_blocks` (vanilla's variant biomes). PV stays above the valley slice: no river biomes
//! until rivers exist (Phase 2).

use std::sync::OnceLock;

use deeptime_format::{GridDesc, Noise};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::climate::koppen;
use crate::error::CoreError;
use crate::world::Field;

/// The reference: band edges and vanilla's land distributions (one file with the game).
pub const REFERENCE_JSON: &str = include_str!("../../data/climate_mapping/default.json");

/// `notes.bake.climate.mapping`: which mapping wrote the layers.
pub const MAPPING: &str = "p1-09";

/// Salt of the noise that signs the weirdness.
pub const VARIANT_SALT: u32 = 1801;

/// The preset's `biomes` section: the knobs of the mapping (every key optional).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BiomeParams {
    /// Added to every temperature band coordinate (+1 = one band warmer).
    pub temperature_shift: f64,
    /// Added to every land humidity band coordinate (+1 = one band wetter).
    pub humidity_shift: f64,
    /// Width of the coast band on land (beaches, stony shores), blocks.
    pub beach_blocks: f64,
    /// The window of the relief and ridge/valley measures, blocks: the size of the patches of
    /// peaks, slopes, plateaus and plains.
    pub terrain_window_blocks: f64,
    /// Wavelength of the noise that picks vanilla's variant biomes, blocks.
    pub variant_blocks: f64,
    /// How readily wet, flat lowlands become swamps (0 = never).
    pub wetlands: f64,
}

impl Default for BiomeParams {
    fn default() -> Self {
        BiomeParams {
            temperature_shift: 0.0,
            humidity_shift: 0.0,
            beach_blocks: 16.0,
            terrain_window_blocks: 64.0,
            variant_blocks: 1024.0,
            wetlands: 1.0,
        }
    }
}

impl BiomeParams {
    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        let within = |v: f64, lo: f64, hi: f64, key: &str| {
            if v.is_finite() && (lo..=hi).contains(&v) {
                Ok(())
            } else {
                Err(CoreError::preset(
                    format!("$.biomes.{key}"),
                    format!("must be in [{lo}, {hi}]"),
                ))
            }
        };
        within(self.temperature_shift, -2.0, 2.0, "temperature_shift")?;
        within(self.humidity_shift, -2.0, 2.0, "humidity_shift")?;
        within(self.beach_blocks, 0.0, 150.0, "beach_blocks")?;
        within(
            self.terrain_window_blocks,
            8.0,
            1024.0,
            "terrain_window_blocks",
        )?;
        within(self.variant_blocks, 64.0, 16_384.0, "variant_blocks")?;
        within(self.wetlands, 0.0, 1.0, "wetlands")
    }
}

/// Band edges (ascending) of the five parameters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bands {
    /// 5 bands.
    pub temperature: Vec<f64>,
    /// 5 bands.
    pub vegetation: Vec<f64>,
    /// mushroom, deep ocean, ocean, coast, near-, mid-, far-inland.
    pub continents: Vec<f64>,
    /// E0 … E6.
    pub erosion: Vec<f64>,
    /// Slice edges of |weirdness| on the rising branch: valley, low, mid, high, peak.
    pub weirdness: Vec<f64>,
}

/// Vanilla's land distributions, as quantiles at u = i / (n − 1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Distribution {
    pub source: String,
    pub land_min_continents: f64,
    pub points: u64,
    pub erosion: Vec<f64>,
    pub peaks_valleys: Vec<f64>,
}

/// `data/climate_mapping/default.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reference {
    pub schema: u32,
    pub bands: Bands,
    pub reference: Distribution,
}

/// The reference, parsed once.
pub fn reference() -> &'static Reference {
    static R: OnceLock<Reference> = OnceLock::new();
    R.get_or_init(|| serde_json::from_str(REFERENCE_JSON).expect("default.json parses"))
}

/// The value at band coordinate `b` (clamped to [0, n]).
pub fn band_value(edges: &[f64], b: f64) -> f64 {
    let n = edges.len() - 1;
    let b = if b.is_nan() {
        0.0
    } else {
        b.clamp(0.0, n as f64)
    };
    let k = (b.floor() as usize).min(n - 1);
    edges[k] + (b - k as f64) * (edges[k + 1] - edges[k])
}

/// The band coordinate of `v` (clamped to the outer edges).
pub fn band_coord(edges: &[f64], v: f64) -> f64 {
    let n = edges.len() - 1;
    if v <= edges[0] {
        return 0.0;
    }
    for k in 0..n {
        if v <= edges[k + 1] {
            return k as f64 + (v - edges[k]) / (edges[k + 1] - edges[k]);
        }
    }
    n as f64
}

/// Piecewise linear through `anchors` (x ascending), clamped at the ends.
pub fn piecewise(anchors: &[(f64, f64)], x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    let (x0, y0) = anchors[0];
    if x <= x0 {
        return y0;
    }
    for w in anchors.windows(2) {
        let ((xa, ya), (xb, yb)) = (w[0], w[1]);
        if x <= xb {
            return ya + (yb - ya) * (x - xa) / (xb - xa);
        }
    }
    anchors[anchors.len() - 1].1
}

fn unit(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
}

/// The D formula (also BSk with a freezing coldest month).
fn thermal_d(t_warm: f64, t_cold: f64) -> f64 {
    0.75 + 1.25 * (0.65 * unit((t_warm - 10.0) / 12.0) + 0.35 * unit((t_cold + 30.0) / 30.0))
}

/// The C formula (also BSk with a mild coldest month).
fn thermal_c(t_cold: f64) -> f64 {
    2.0 + 0.95 * unit(t_cold / 18.0)
}

/// Temperature band coordinate of a land cell of Köppen class `k`.
pub fn thermal_land(k: u8, t_mean: f64, t_warm: f64, t_cold: f64, glacier: bool) -> f64 {
    let b = match k {
        29 | 30 => 0.75 * unit((t_warm + 20.0) / 30.0),
        17..=28 => thermal_d(t_warm, t_cold),
        8..=16 => thermal_c(t_cold),
        0..=3 => 3.0 + 0.9 * unit((t_cold - 18.0) / 10.0),
        4 | 5 => 4.05 + 0.9 * unit((t_mean - 5.0) / 25.0),
        6 => 3.0 + 0.9 * unit((t_mean - 18.0) / 12.0),
        7 => {
            if t_cold > 0.0 {
                thermal_c(t_cold)
            } else {
                thermal_d(t_warm, t_cold)
            }
        }
        _ => thermal_sea(t_mean),
    };
    if glacier { b.min(0.35) } else { b }
}

/// Temperature band coordinate of a sea cell (from its surface temperature).
pub fn thermal_sea(t_mean: f64) -> f64 {
    piecewise(
        &[
            (-20.0, 0.0),
            (-2.0, 1.0),
            (6.0, 2.0),
            (18.0, 3.0),
            (25.0, 4.0),
            (32.0, 5.0),
        ],
        t_mean,
    )
}

/// Without the climate stage: the band coordinate of a mean temperature.
pub fn thermal_plain(t_mean: f64) -> f64 {
    piecewise(
        &[
            (-25.0, 0.0),
            (-10.0, 1.0),
            (0.0, 2.0),
            (12.0, 3.0),
            (22.0, 4.0),
            (32.0, 5.0),
        ],
        t_mean,
    )
}

/// The aridity index's band coordinate before the class's range.
pub fn humidity_of_aridity(aridity: f64) -> f64 {
    let x = aridity.max(1e-4).ln();
    piecewise(
        &[
            (0.03f64.ln(), 0.0),
            (0.2f64.ln(), 1.0),
            (0.5f64.ln(), 2.0),
            (1.0f64.ln(), 3.0),
            (2.0f64.ln(), 4.0),
            (5.0f64.ln(), 5.0),
        ],
        x,
    )
}

/// Humidity band coordinate of a land cell of Köppen class `k`.
pub fn humidity_land(k: u8, aridity: f64) -> f64 {
    let h = humidity_of_aridity(aridity);
    let squeeze = |lo: f64, hi: f64| lo + (hi - lo) * unit((h - 1.0) / 3.0);
    match k {
        0 => h.max(4.1),
        1 => h.max(3.2),
        2 | 3 => squeeze(1.05, 2.55),
        4 | 5 => h.clamp(0.0, 0.95),
        6 | 7 => h.clamp(1.05, 1.95),
        8..=10 => squeeze(1.05, 2.5),
        11..=16 => h.max(2.05),
        19 | 20 | 23 | 24 | 27 | 28 => h.max(3.05),
        17 | 18 | 21 | 22 | 25 | 26 => h.max(2.05),
        29 | 30 => h.min(1.95),
        _ => h,
    }
}

/// A quantile function from sorted samples at u = i / (n − 1).
#[derive(Clone, Debug)]
pub struct Quantile {
    q: Vec<f64>,
    lo: f64,
    hi: f64,
}

impl Quantile {
    pub fn new(q: &[f64]) -> Quantile {
        let mut q = q.to_vec();
        q.sort_by(f64::total_cmp);
        Quantile {
            q,
            lo: 0.0,
            hi: 1.0,
        }
    }

    fn raw(&self, u: f64) -> f64 {
        let n = self.q.len();
        if n == 1 {
            return self.q[0];
        }
        let x = unit(u) * (n - 1) as f64;
        let i = (x.floor() as usize).min(n - 2);
        self.q[i] + (x - i as f64) * (self.q[i + 1] - self.q[i])
    }

    /// The CDF at `v` (the inverse of `raw`).
    fn cdf(&self, v: f64) -> f64 {
        let n = self.q.len();
        if n == 1 || v <= self.q[0] {
            return 0.0;
        }
        for i in 0..n - 1 {
            if v <= self.q[i + 1] {
                let d = self.q[i + 1] - self.q[i];
                let f = if d > 0.0 { (v - self.q[i]) / d } else { 1.0 };
                return (i as f64 + f) / (n - 1) as f64;
            }
        }
        1.0
    }

    /// The distribution conditioned on values ≤ `v`.
    pub fn below(&self, v: f64) -> Quantile {
        Quantile {
            q: self.q.clone(),
            lo: self.lo,
            hi: self.lo + (self.hi - self.lo) * self.cdf(v).max(1e-3),
        }
    }

    /// The distribution conditioned on values ≥ `v`.
    pub fn above(&self, v: f64) -> Quantile {
        Quantile {
            q: self.q.clone(),
            lo: self.lo + (self.hi - self.lo) * self.cdf(v).min(1.0 - 1e-3),
            hi: self.hi,
        }
    }

    /// The value at quantile `u` ∈ [0, 1].
    pub fn at(&self, u: f64) -> f64 {
        self.raw(self.lo + unit(u) * (self.hi - self.lo))
    }
}

/// Replaces every non-source cell by the value of (roughly) its nearest source cell: a two-pass
/// chamfer carrying values, x wrapping. Returns false (and leaves `v`) when there is no source.
pub fn fill_nearest(v: &mut [f32], src: &[bool], w: usize, h: usize) -> bool {
    if !src.iter().any(|&s| s) {
        return false;
    }
    let inf = f32::INFINITY;
    let mut d: Vec<f32> = src.iter().map(|&s| if s { 0.0 } else { inf }).collect();
    let (a, b) = (1.0f32, std::f32::consts::SQRT_2);
    let relax = |d: &mut [f32], v: &mut [f32], c: usize, n: usize, step: f32| {
        if d[n] + step < d[c] {
            d[c] = d[n] + step;
            v[c] = v[n];
        }
    };
    for _ in 0..2 {
        for j in 0..h {
            for i in 0..w {
                let c = j * w + i;
                let il = (i + w - 1) % w;
                relax(&mut d, v, c, j * w + il, a);
                if j > 0 {
                    let up = (j - 1) * w;
                    relax(&mut d, v, c, up + i, a);
                    relax(&mut d, v, c, up + il, b);
                    relax(&mut d, v, c, up + (i + 1) % w, b);
                }
            }
        }
        for j in (0..h).rev() {
            for i in (0..w).rev() {
                let c = j * w + i;
                let ir = (i + 1) % w;
                relax(&mut d, v, c, j * w + ir, a);
                if j + 1 < h {
                    let dn = (j + 1) * w;
                    relax(&mut d, v, c, dn + i, a);
                    relax(&mut d, v, c, dn + ir, b);
                    relax(&mut d, v, c, dn + (i + w - 1) % w, b);
                }
            }
        }
    }
    true
}

/// Distance to the nearest cell of the other medium (land ↔ sea), in cells, by a two-pass chamfer
/// (x wraps). Cells of the target medium get 0.
pub fn distance_cells(land: &[bool], w: usize, h: usize, to_sea: bool) -> Vec<f32> {
    let src: Vec<bool> = land.iter().map(|&l| l != to_sea).collect();
    let mut idx: Vec<f32> = vec![0.0; land.len()];
    let inf = f32::INFINITY;
    if !src.iter().any(|&s| s) {
        idx.iter_mut().for_each(|d| *d = inf);
        return idx;
    }
    let mut d: Vec<f32> = src.iter().map(|&s| if s { 0.0 } else { inf }).collect();
    let (a, b) = (1.0f32, std::f32::consts::SQRT_2);
    for _ in 0..2 {
        for j in 0..h {
            for i in 0..w {
                let c = j * w + i;
                let il = (i + w - 1) % w;
                let mut m = d[c].min(d[j * w + il] + a);
                if j > 0 {
                    let up = (j - 1) * w;
                    m = m.min(d[up + i] + a);
                    m = m.min(d[up + il] + b);
                    m = m.min(d[up + (i + 1) % w] + b);
                }
                d[c] = m;
            }
        }
        for j in (0..h).rev() {
            for i in (0..w).rev() {
                let c = j * w + i;
                let ir = (i + 1) % w;
                let mut m = d[c].min(d[j * w + ir] + a);
                if j + 1 < h {
                    let dn = (j + 1) * w;
                    m = m.min(d[dn + i] + a);
                    m = m.min(d[dn + ir] + b);
                    m = m.min(d[dn + (i + w - 1) % w] + b);
                }
                d[c] = m;
            }
        }
    }
    idx.copy_from_slice(&d);
    idx
}

/// The extreme (max or min) of `line` over the window [i − k, i + k]: wrapping or clipped at the
/// ends. A monotonic deque: O(n).
fn sliding_extreme(line: &[f32], k: usize, wrap: bool, max: bool) -> Vec<f32> {
    let n = line.len();
    let better = |a: f32, b: f32| if max { a >= b } else { a <= b };
    let mut out = vec![0f32; n];
    let mut dq: std::collections::VecDeque<(i64, f32)> = std::collections::VecDeque::new();
    let get = |i: i64| -> Option<f32> {
        if wrap {
            Some(line[i.rem_euclid(n as i64) as usize])
        } else if i >= 0 && (i as usize) < n {
            Some(line[i as usize])
        } else {
            None
        }
    };
    let k = k as i64;
    let mut next = -k;
    for i in 0..n as i64 {
        while next <= i + k {
            if let Some(v) = get(next) {
                while dq.back().is_some_and(|&(_, b)| better(v, b)) {
                    dq.pop_back();
                }
                dq.push_back((next, v));
            }
            next += 1;
        }
        while dq.front().is_some_and(|&(j, _)| j < i - k) {
            dq.pop_front();
        }
        out[i as usize] = dq.front().map_or(line[i as usize], |&(_, v)| v);
    }
    out
}

/// The mean of `line` over [i − k, i + k]: wrapping or clipped (over the cells inside).
fn sliding_mean(line: &[f32], k: usize, wrap: bool) -> Vec<f32> {
    let n = line.len();
    let mut pre = vec![0f64; n + 1];
    for i in 0..n {
        pre[i + 1] = pre[i] + f64::from(line[i]);
    }
    let total = pre[n];
    let sum = |a: i64, b: i64| -> f64 {
        // Sum over [a, b) with 0 ≤ a ≤ b ≤ n.
        pre[b as usize] - pre[a as usize]
    };
    let (n_i, k) = (n as i64, k as i64);
    (0..n_i)
        .map(|i| {
            let (a, b) = (i - k, i + k + 1);
            if wrap {
                if 2 * k + 1 >= n_i {
                    return (total / n as f64) as f32;
                }
                let s = if a < 0 {
                    sum(a + n_i, n_i) + sum(0, b)
                } else if b > n_i {
                    sum(a, n_i) + sum(0, b - n_i)
                } else {
                    sum(a, b)
                };
                (s / (2 * k + 1) as f64) as f32
            } else {
                let (a, b) = (a.max(0), b.min(n_i));
                (sum(a, b) / (b - a) as f64) as f32
            }
        })
        .collect()
}

fn transpose(v: &[f32], w: usize, h: usize) -> Vec<f32> {
    let mut t = vec![0f32; v.len()];
    t.par_chunks_mut(h).enumerate().for_each(|(i, col)| {
        for (j, o) in col.iter_mut().enumerate() {
            *o = v[j * w + i];
        }
    });
    t
}

/// A separable 2D filter: rows wrap, columns clip.
fn separable(
    v: &[f32],
    w: usize,
    h: usize,
    f: impl Fn(&[f32], bool) -> Vec<f32> + Sync,
) -> Vec<f32> {
    let mut rows = vec![0f32; v.len()];
    rows.par_chunks_mut(w)
        .zip(v.par_chunks(w))
        .for_each(|(o, r)| o.copy_from_slice(&f(r, true)));
    let t = transpose(&rows, w, h);
    let mut cols = vec![0f32; v.len()];
    cols.par_chunks_mut(h)
        .zip(t.par_chunks(h))
        .for_each(|(o, c)| o.copy_from_slice(&f(c, false)));
    transpose(&cols, h, w)
}

/// Range (max − min) over a (2k + 1)² window.
pub fn local_range(v: &[f32], w: usize, h: usize, k: usize) -> Vec<f32> {
    let hi = separable(v, w, h, |l, wrap| sliding_extreme(l, k, wrap, true));
    let lo = separable(v, w, h, |l, wrap| sliding_extreme(l, k, wrap, false));
    hi.iter().zip(&lo).map(|(a, b)| a - b).collect()
}

/// Mean over a (2k + 1)² window.
pub fn box_mean(v: &[f32], w: usize, h: usize, k: usize) -> Vec<f32> {
    separable(v, w, h, |l, wrap| sliding_mean(l, k, wrap))
}

/// Ranks of the masked cells in [0, 1] (ascending value; ties by index); NaN elsewhere.
pub fn ranks(v: &[f32], mask: &[bool]) -> Vec<f32> {
    let mut idx: Vec<u32> = (0..v.len() as u32).filter(|&i| mask[i as usize]).collect();
    idx.par_sort_unstable_by(|&a, &b| v[a as usize].total_cmp(&v[b as usize]).then(a.cmp(&b)));
    let mut out = vec![f32::NAN; v.len()];
    let n = idx.len();
    let denom = (n.max(2) - 1) as f32;
    for (r, &i) in idx.iter().enumerate() {
        out[i as usize] = r as f32 / denom;
    }
    out
}

/// The physical climate on L1 the mapping reads (P1-05's fields).
pub struct Climate<'a> {
    pub t_mean: &'a Field,
    pub t_warm: Option<&'a Field>,
    pub t_cold: Option<&'a Field>,
    pub aridity: Option<&'a Field>,
    pub koppen: &'a Field,
    pub glacier: Option<&'a Field>,
}

/// What the mapping needs besides the climate.
pub struct Ground<'a> {
    /// L2 ground elevation, m (the young landscape's, or the surface's resampled).
    pub elevation_m: &'a [f32],
    pub sea_level_m: f64,
    pub l1: &'a GridDesc,
    pub l2: &'a GridDesc,
    pub seed: u64,
    pub circumference: u32,
}

/// The five layers (router order: temperature, vegetation, continents, erosion, ridges) in
/// vanilla parameter units, and the record for `notes.bake.climate`.
pub struct Mapped {
    pub layers: [Vec<f32>; 5],
    pub note: Value,
}

/// Temperature and humidity band coordinates on L1: land values filled across the sea, the sea's
/// temperature across the land.
struct Coords {
    t_land: Vec<f32>,
    h_land: Vec<f32>,
    t_sea: Vec<f32>,
    by_class: Value,
}

fn coords(c: &Climate<'_>, l1: &GridDesc) -> Coords {
    let (w, h) = (l1.w as usize, l1.h as usize);
    let n = w * h;
    // A missing field or a NaN cell falls back to the defaults below (never to "driest").
    let at =
        |f: Option<&Field>, k: usize| f.map(|f| f64::from(f.data[k])).filter(|v| v.is_finite());
    let mut t_land = vec![f32::NAN; n];
    let mut h_land = vec![f32::NAN; n];
    let mut t_sea = vec![f32::NAN; n];
    let mut land = vec![false; n];
    let mut stats = [(0u64, 0f64, 0f64); 32];
    for k in 0..n {
        let kv = c.koppen.data[k];
        let class = if kv.is_nan() {
            koppen::SEA
        } else {
            (kv.round() as i64).clamp(0, 31) as u8
        };
        let tm = f64::from(c.t_mean.data[k]);
        if class == koppen::SEA {
            t_sea[k] = thermal_sea(tm) as f32;
            continue;
        }
        land[k] = true;
        let tw = at(c.t_warm, k).unwrap_or(tm + 8.0);
        let tc = at(c.t_cold, k).unwrap_or(tm - 8.0);
        let ai = at(c.aridity, k).unwrap_or(1.0);
        let glacier = at(c.glacier, k).is_some_and(|g| g >= 0.5);
        let th = thermal_land(class, tm, tw, tc, glacier);
        let hu = humidity_land(class, ai);
        t_land[k] = th as f32;
        h_land[k] = hu as f32;
        let s = &mut stats[class as usize];
        s.0 += 1;
        s.1 += th;
        s.2 += hu;
    }
    let sea: Vec<bool> = land.iter().map(|&l| !l).collect();
    if !fill_nearest(&mut t_land, &land, w, h) {
        t_land = t_sea.clone();
    }
    if !fill_nearest(&mut h_land, &land, w, h) {
        h_land = vec![2.5; n];
    }
    if !fill_nearest(&mut t_sea, &sea, w, h) {
        t_sea = t_land.clone();
    }
    let by_class: serde_json::Map<String, Value> = stats
        .iter()
        .enumerate()
        .filter(|(_, s)| s.0 > 0)
        .map(|(i, s)| {
            (
                koppen::CODES[i].to_owned(),
                json!({
                    "cells": s.0,
                    "temperature_band": ((s.1 / s.0 as f64) * 100.0).round() / 100.0,
                    "humidity_band": ((s.2 / s.0 as f64) * 100.0).round() / 100.0,
                }),
            )
        })
        .collect();
    Coords {
        t_land,
        h_land,
        t_sea,
        by_class: Value::Object(by_class),
    }
}

fn shares(v: &[f32], mask: &[bool], edges: &[f64], abs: bool) -> Vec<f64> {
    let n = edges.len() - 1;
    let mut c = vec![0u64; n];
    let mut total = 0u64;
    for (x, &m) in v.iter().zip(mask) {
        if !m {
            continue;
        }
        let x = if abs { x.abs() } else { *x };
        let b = band_coord(edges, f64::from(x));
        c[(b.floor() as usize).min(n - 1)] += 1;
        total += 1;
    }
    c.iter()
        .map(|&k| (k as f64 / total.max(1) as f64 * 1000.0).round() / 1000.0)
        .collect()
}

/// The mapping (module documentation). `climate` is `None` without the climate stage: the
/// temperature then follows latitude (28 − 0.55·|lat| °C) and humidity is middling.
pub fn map(climate: Option<&Climate<'_>>, g: &Ground<'_>, p: &BiomeParams) -> Mapped {
    let r = reference();
    let bands = &r.bands;
    let l2 = g.l2;
    let (w, h) = (l2.w as usize, l2.h as usize);
    let n = w * h;
    let sea = g.sea_level_m;
    let elev = g.elevation_m;
    let land: Vec<bool> = elev.iter().map(|&z| f64::from(z) > sea).collect();

    // Temperature and humidity.
    let (t_land, h_land, t_sea, by_class) = match climate {
        Some(c) => {
            let co = coords(c, g.l1);
            let up = |v: Vec<f32>| {
                super::resample(
                    &Field {
                        grid: g.l1.clone(),
                        data: v,
                    },
                    l2,
                    false,
                )
            };
            (up(co.t_land), up(co.h_land), up(co.t_sea), co.by_class)
        }
        None => {
            let t: Vec<f32> = (0..n)
                .map(|k| {
                    let j = k / w;
                    let lat = 90.0 - 180.0 * (j as f64 + 0.5) / h as f64;
                    thermal_plain(28.0 - 0.55 * lat.abs()) as f32
                })
                .collect();
            (t.clone(), vec![2.5; n], t, Value::Null)
        }
    };
    let mut temperature = vec![0f32; n];
    let mut vegetation = vec![0f32; n];
    for k in 0..n {
        let tb = if land[k] { t_land[k] } else { t_sea[k] };
        temperature[k] = band_value(&bands.temperature, f64::from(tb) + p.temperature_shift) as f32;
        vegetation[k] =
            band_value(&bands.vegetation, f64::from(h_land[k]) + p.humidity_shift) as f32;
    }

    // Continentalness.
    let r2 = f64::from(l2.r);
    let to_sea = distance_cells(&land, w, h, true);
    let beach = p.beach_blocks.min(190.0);
    let land_anchors = [
        (0.0, 3.0),
        (beach.max(1e-3), 4.0),
        (200.0, 5.0),
        (1000.0, 6.0),
        (5000.0, 7.0),
    ];
    let sea_anchors = [(0.0, 3.0), (200.0, 2.585), (2500.0, 2.0), (6000.0, 1.084)];
    let mut cont_band = vec![0f32; n];
    let mut continents = vec![0f32; n];
    for k in 0..n {
        let b = if land[k] {
            // Cell centres: a land cell touching the sea is half a cell from it.
            piecewise(&land_anchors, (f64::from(to_sea[k]) - 0.5).max(0.0) * r2)
        } else {
            piecewise(&sea_anchors, sea - f64::from(elev[k]))
        };
        cont_band[k] = b as f32;
        continents[k] = band_value(&bands.continents, b) as f32;
    }

    // Relief and the ridge/valley index on the ground (the sea surface counted as ground).
    let ground: Vec<f32> = elev.iter().map(|&z| z.max(sea as f32)).collect();
    let k = ((p.terrain_window_blocks / (2.0 * r2)).round() as usize).max(1);
    let relief = box_mean(&local_range(&ground, w, h, k), w, h, k);
    let mean = box_mean(&ground, w, h, k);
    let crest: Vec<f32> = ground.iter().zip(&mean).map(|(z, m)| z - m).collect();
    let crest = box_mean(&crest, w, h, (k / 2).max(1));
    let u_relief = ranks(&relief, &land);
    let u_crest = ranks(&crest, &land);

    // Erosion.
    let e6 = bands.erosion[6];
    let q_ero = Quantile::new(&r.reference.erosion).below(e6 - 1e-3);
    let mut erosion = vec![f32::NAN; n];
    let mut wet_cells = 0u64;
    for k in 0..n {
        if !land[k] {
            continue;
        }
        let u = f64::from(u_relief[k]);
        let mut e = q_ero.at(1.0 - u);
        let wet = unit((f64::from(h_land[k]) + p.humidity_shift - 2.5) / 1.5);
        let flat = unit((0.35 - u) / 0.35);
        let low = unit(1.0 - (f64::from(elev[k]) - sea) / 150.0);
        let thawed = f64::from(t_land[k]) + p.temperature_shift >= 1.0;
        let s = p.wetlands * wet * flat * low;
        if thawed && s > 0.15 {
            e = e6 + 0.01 + (0.95 - e6 - 0.01) * unit((s - 0.15) / 0.6);
            wet_cells += 1;
        }
        erosion[k] = e as f32;
    }
    if !fill_nearest(&mut erosion, &land, w, h) {
        erosion.iter_mut().for_each(|e| *e = 0.25);
    }
    let deep_sea_floor = (0.5 * (bands.erosion[4] + bands.erosion[5])) as f32;
    for k in 0..n {
        if !land[k] && cont_band[k] < 2.0 {
            erosion[k] = erosion[k].max(deep_sea_floor);
        }
    }

    // Weirdness.
    let pv_min = 3.0 * (bands.weirdness[1] + 1.0 / 60.0) - 1.0;
    let q_pv = Quantile::new(&r.reference.peaks_valleys).above(pv_min);
    let mut mag = vec![f32::NAN; n];
    for k in 0..n {
        if land[k] {
            let pv = q_pv.at(f64::from(u_crest[k])).max(pv_min);
            mag[k] = ((pv + 1.0) / 3.0) as f32;
        }
    }
    if !fill_nearest(&mut mag, &land, w, h) {
        mag.iter_mut().for_each(|m| *m = 0.3);
    }
    let noise = Noise::new(g.seed, g.circumference);
    let mut ridges = vec![0f32; n];
    ridges.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
        let bz = l2.block_z(j as i64) as f64;
        for (i, o) in row.iter_mut().enumerate() {
            let bx = l2.block_x(i as i64) as f64;
            let m = mag[j * w + i];
            *o = if noise.value(bx, bz, p.variant_blocks, VARIANT_SALT) < 0.0 {
                -m
            } else {
                m
            };
        }
    });

    let land_cells = land.iter().filter(|&&l| l).count();
    let note = json!({
        "mapping": MAPPING,
        "params": p,
        "reference": { "bands": bands, "reference": r.reference },
        "koppen_bands": by_class,
        "window_cells": k,
        "wetland_cells": wet_cells,
        "land_cells_l2": land_cells,
        "land_shares": {
            "temperature": shares(&temperature, &land, &bands.temperature, false),
            "vegetation": shares(&vegetation, &land, &bands.vegetation, false),
            "continents": shares(&continents, &land, &bands.continents, false),
            "erosion": shares(&erosion, &land, &bands.erosion, false),
            "weirdness": shares(&ridges, &land, &bands.weirdness, true),
        },
    });
    Mapped {
        layers: [temperature, vegetation, continents, erosion, ridges],
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Vanilla's middle biomes (OverworldBiomeBuilder.MIDDLE_BIOMES, 1.21.1), by temperature and
    /// humidity band.
    const MIDDLE: [[&str; 5]; 5] = [
        [
            "snowy_plains",
            "snowy_plains",
            "snowy_plains",
            "snowy_taiga",
            "taiga",
        ],
        [
            "plains",
            "plains",
            "forest",
            "taiga",
            "old_growth_spruce_taiga",
        ],
        [
            "flower_forest",
            "plains",
            "forest",
            "birch_forest",
            "dark_forest",
        ],
        ["savanna", "savanna", "forest", "jungle", "jungle"],
        ["desert", "desert", "desert", "desert", "desert"],
    ];

    fn middle(class: &str, t_mean: f64, t_warm: f64, t_cold: f64, aridity: f64) -> &'static str {
        let k = koppen::index(class).unwrap();
        let t = thermal_land(k, t_mean, t_warm, t_cold, false);
        let hu = humidity_land(k, aridity);
        MIDDLE[(t.floor() as usize).min(4)][(hu.floor() as usize).min(4)]
    }

    #[test]
    fn reference_bands_are_the_builder_s() {
        let b = &reference().bands;
        assert_eq!(b.temperature, [-1.0, -0.45, -0.15, 0.2, 0.55, 1.0]);
        assert_eq!(b.vegetation, [-1.0, -0.35, -0.1, 0.1, 0.3, 1.0]);
        assert_eq!(b.continents.len(), 8);
        assert_eq!(b.erosion.len(), 8);
        assert_eq!(b.weirdness.len(), 6);
        for e in [
            &b.temperature,
            &b.vegetation,
            &b.continents,
            &b.erosion,
            &b.weirdness,
        ] {
            assert!(e.windows(2).all(|p| p[0] < p[1]), "{e:?} ascends");
        }
        let d = &reference().reference;
        assert!(d.erosion.windows(2).all(|p| p[0] <= p[1]));
        assert!(d.peaks_valleys.windows(2).all(|p| p[0] <= p[1]));
    }

    #[test]
    fn koppen_classes_land_on_their_biomes() {
        // Manaus-like Af, Kinshasa-like Aw, Riyadh-like BWh, Ulaanbaatar-like BWk (a cold desert
        // is still a desert), Arkhangelsk-like Dfc, London-like Cfb, Svalbard-like ET, a Vostok EF.
        assert_eq!(middle("Af", 27.0, 28.0, 26.0, 2.5), "jungle");
        assert_eq!(middle("Am", 26.0, 28.0, 24.0, 1.4), "jungle");
        assert_eq!(middle("Aw", 25.0, 27.0, 22.0, 0.9), "savanna");
        assert_eq!(middle("BWh", 26.0, 36.0, 14.0, 0.02), "desert");
        assert_eq!(middle("BWk", 8.0, 24.0, -10.0, 0.1), "desert");
        assert_eq!(middle("BSh", 24.0, 30.0, 18.0, 0.35), "savanna");
        let cfb = middle("Cfb", 11.0, 18.7, 5.0, 1.3);
        assert!(cfb.ends_with("forest"), "Cfb gives {cfb}");
        let dfc = middle("Dfc", 1.0, 16.5, -12.0, 1.1);
        assert!(dfc.contains("taiga"), "Dfc gives {dfc}");
        let dfc_cold = middle("Dfc", -6.0, 12.0, -30.0, 0.9);
        assert!(dfc_cold.contains("taiga"), "a cold Dfc gives {dfc_cold}");
        assert_eq!(middle("ET", -6.0, 6.0, -15.0, 1.5), "snowy_plains");
        // A wet ET stays below the humidity band whose variant is snowy taiga.
        assert!(humidity_land(29, 5.0) < 2.0);
        assert_eq!(middle("EF", -55.0, -30.0, -70.0, 0.5), "snowy_plains");
        // Glaciated land is ice, whatever its class.
        assert!(thermal_land(26, 5.0, 18.0, -8.0, true) < 1.0);
        // Oceans: sea ice frozen, tropical water warm.
        assert!(thermal_sea(-5.0) < 1.0);
        assert!(thermal_sea(28.0) >= 4.0);
    }

    #[test]
    fn bands_round_trip() {
        let e = &reference().bands.temperature;
        for b in [0.0, 0.3, 1.0, 2.5, 3.99, 5.0] {
            assert!((band_coord(e, band_value(e, b)) - b).abs() < 1e-9, "{b}");
        }
        assert_eq!(band_value(e, -3.0), -1.0);
        assert_eq!(band_value(e, 9.0), 1.0);
    }

    #[test]
    fn quantiles_condition() {
        let q = Quantile::new(&[0.0, 1.0, 2.0, 3.0, 4.0]);
        assert_eq!(q.at(0.5), 2.0);
        let b = q.below(2.0);
        assert!((b.at(1.0) - 2.0).abs() < 1e-9);
        assert_eq!(b.at(0.0), 0.0);
        let a = q.above(3.0);
        assert!((a.at(0.0) - 3.0).abs() < 1e-9);
        assert_eq!(a.at(1.0), 4.0);
    }

    #[test]
    fn sliding_filters_match_naive() {
        let line: Vec<f32> = (0..23).map(|i| ((i * 7919) % 31) as f32).collect();
        for &wrap in &[true, false] {
            let mx = sliding_extreme(&line, 3, wrap, true);
            let mn = sliding_extreme(&line, 3, wrap, false);
            let me = sliding_mean(&line, 3, wrap);
            for i in 0..23i64 {
                let win: Vec<f32> = (i - 3..=i + 3)
                    .filter_map(|j| {
                        if wrap {
                            Some(line[j.rem_euclid(23) as usize])
                        } else if (0..23).contains(&j) {
                            Some(line[j as usize])
                        } else {
                            None
                        }
                    })
                    .collect();
                let m = win.iter().cloned().fold(f32::MIN, f32::max);
                let l = win.iter().cloned().fold(f32::MAX, f32::min);
                let a = win.iter().sum::<f32>() / win.len() as f32;
                assert_eq!(mx[i as usize], m);
                assert_eq!(mn[i as usize], l);
                assert!((me[i as usize] - a).abs() < 1e-4);
            }
        }
    }

    #[test]
    fn fill_and_distance_wrap_in_x() {
        let (w, h) = (10, 3);
        let land: Vec<bool> = (0..w * h).map(|c| c % w != 0).collect();
        let d = distance_cells(&land, w, h, true);
        assert_eq!(d[0], 0.0);
        assert_eq!(d[1], 1.0);
        assert_eq!(d[9], 1.0, "across the seam");
        assert_eq!(d[5], 5.0);
        let mut v: Vec<f32> = (0..w * h)
            .map(|c| if c % w == 0 { 7.0 } else { f32::NAN })
            .collect();
        let src: Vec<bool> = (0..w * h).map(|c| c % w == 0).collect();
        assert!(fill_nearest(&mut v, &src, w, h));
        assert!(v.iter().all(|&x| x == 7.0));
    }

    #[test]
    fn a_planet_maps_by_its_koppen_map() {
        // 64 × 32 L1 cells, L2 twice as fine; land in the middle rows with an Af, a BWh, a Dfc and
        // an ET quarter, a ridge through the middle.
        let c = 1024u32;
        let l1 = GridDesc::for_planet("L1", c, 16).unwrap();
        let l2 = GridDesc::for_planet("L2", c, 8).unwrap();
        let (w1, h1) = (l1.w as usize, l1.h as usize);
        let quarter = |i: usize| ["Af", "BWh", "Dfc", "ET"][i * 4 / w1];
        let field = |f: &dyn Fn(usize, usize) -> f32| Field {
            grid: l1.clone(),
            data: (0..w1 * h1).map(|k| f(k % w1, k / w1)).collect(),
        };
        let is_land = |j: usize| (8..24).contains(&j);
        let kp = field(&|i, j| {
            if is_land(j) {
                f32::from(koppen::index(quarter(i)).unwrap())
            } else {
                f32::from(koppen::SEA)
            }
        });
        let tm = field(&|i, _| match quarter(i) {
            "Af" => 27.0,
            "BWh" => 26.0,
            "Dfc" => 0.0,
            _ => -8.0,
        });
        let tw = field(&|i, _| match quarter(i) {
            "Af" => 28.0,
            "BWh" => 36.0,
            "Dfc" => 16.0,
            _ => 5.0,
        });
        let tc = field(&|i, _| match quarter(i) {
            "Af" => 26.0,
            "BWh" => 14.0,
            "Dfc" => -15.0,
            _ => -20.0,
        });
        let ai = field(&|i, _| match quarter(i) {
            "Af" => 2.5,
            "BWh" => 0.03,
            "Dfc" => 1.2,
            _ => 1.0,
        });
        let (w2, h2) = (l2.w as usize, l2.h as usize);
        let elev: Vec<f32> = (0..w2 * h2)
            .map(|k| {
                let (i, j) = (k % w2, k / w2);
                if is_land(j / 2) {
                    200.0 + if i % 32 == 16 { 1500.0 } else { 0.0 }
                } else {
                    -3000.0
                }
            })
            .collect();
        let climate = Climate {
            t_mean: &tm,
            t_warm: Some(&tw),
            t_cold: Some(&tc),
            aridity: Some(&ai),
            koppen: &kp,
            glacier: None,
        };
        let g = Ground {
            elevation_m: &elev,
            sea_level_m: 0.0,
            l1: &l1,
            l2: &l2,
            seed: 3,
            circumference: c,
        };
        let m = map(Some(&climate), &g, &BiomeParams::default());
        let b = &reference().bands;
        let band = |layer: usize, edges: &[f64], i: usize, j: usize| {
            band_coord(edges, f64::from(m.layers[layer][j * w2 + i])).floor() as usize
        };
        // Sample the middle of each quarter at a lowland row.
        for (q, (tband, hlo, hhi)) in [(3, 3, 4), (4, 0, 0), (1, 3, 3), (0, 0, 2)]
            .into_iter()
            .enumerate()
        {
            let i = q * w2 / 4 + w2 / 8 + 3;
            let j = 32;
            let t = band(0, &b.temperature, i, j);
            let hv = band(1, &b.vegetation, i, j);
            assert_eq!(t, tband, "quarter {q} temperature band");
            assert!((hlo..=hhi).contains(&hv), "quarter {q} humidity band {hv}");
            let cont = band(2, &b.continents, i, j);
            assert!(cont >= 4, "inland: continents band {cont}");
        }
        // The sea: deep ocean band, frozen near the ET quarter's latitude is not asserted (the
        // sea keeps its own temperature).
        assert!(band(2, &b.continents, 5, 2) <= 1, "deep ocean");
        // Every value is in the parameter range.
        for l in &m.layers {
            assert!(l.iter().all(|v| (-1.2..=1.0).contains(v)), "range");
        }
        // No valley slice: |w| stays above the valley edge.
        assert!(m.layers[4].iter().all(|w| w.abs() > b.weirdness[1] as f32));
        // The ridge has the highest weirdness (a peak or high slice) and low erosion.
        let (ri, flat_i) = (16 + 64, 24 + 64);
        let jr = 32;
        assert!(m.layers[4][jr * w2 + ri].abs() > m.layers[4][jr * w2 + flat_i].abs());
        assert!(m.layers[3][jr * w2 + ri] < m.layers[3][jr * w2 + flat_i]);
        assert_eq!(m.note["mapping"], MAPPING);
    }
}
