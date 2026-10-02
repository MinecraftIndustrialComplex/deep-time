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
//! **Where each Köppen class goes** on the temperature and humidity bands is data, not code:
//! `data/climate_mapping/placement.json` ([`super::placement`], a datapack may override it). The
//! defaults are the tables below.
//!
//! **Temperature** (5 bands: frozen, cold, temperate, warm, hot) from the Köppen class and the
//! warmest/coldest-month temperatures, so that each main group lands in its band:
//!
//! | Köppen | band coordinate | vanilla middle biomes there |
//! |---|---|---|
//! | ET | 0 … 0.75 by the warmest month (−20 → 10 °C) | snowy plains, ice spikes |
//! | EF | 0.6 … 0.8 by the warmest month (−20 → 0 °C) | snowy plains, ice spikes |
//! | D | 0.75 … 2 by the warmest (10 → 22 °C, weight 0.65) and coldest month (−30 → 0 °C) | snowy taiga (Dfc's cold edge), taiga, plains, forest |
//! | C | 2 … 2.95 by the coldest month (0 → 18 °C) | plains, forest, birch, dark forest |
//! | A, BSh | 3 … 3.9 (coldest month 18 → 28 °C; BSh by the mean 18 → 30 °C) | savanna, forest, jungle |
//! | BW | 4.05 … 4.95 by the mean (5 → 30 °C) | desert (badlands on plateaus) |
//! | BSk | as C or D by its coldest month | plains, meadow |
//!
//! Land above its equilibrium line (`climate/glacier`) is capped at 0.35 (ice), except a class with a
//! `glaciated` block in the placement table: EF (ice-cap climate) is held at 0.6 … 0.8 whether
//! glaciated or not, because Terralith puts its dry cold biomes (gravel desert, cold shrubland) above
//! band 0.5 and its red, terracotta snowy badlands below it; what differs is the humidity, below.
//! The sea's own
//! temperature (sea ice below −2 °C → frozen, 6, 18, 25 °C → cold, temperate, warm, hot oceans)
//! drives the ocean biomes.
//!
//! **Vegetation** (humidity, 5 bands) from the aridity index P/PET — 0.03, 0.2, 0.5, 1, 2, 5 →
//! 0 … 5 on a log scale (UNEP's hyper-arid | arid | semi-arid | dry sub-humid | humid) — held
//! inside each class's range: BW ≤ 0.95 (the driest band), BS 1.05 … 1.95, Af ≥ 4.1 (jungle),
//! Am ≥ 3.2, Aw/As and Cs squeezed into 1.05 … 2.5 (savanna, plains, open forest), the
//! subarctic D (Dsc, Dsd, Dwc, Dwd, Dfc, Dfd) ≥ 3.05 (taiga), other C/D ≥ 2.05, ET ≤ 1.95 (no
//! trees: vanilla picks snowy taiga at the frozen band's third humidity band as a variant, and grove on
//! its slopes). EF is the exception (P/PET is pinned at 10 below freezing, so the index says
//! nothing there): glaciated EF (ice cap) is a fixed 0.5, humidity band 0 (vanilla's ice spikes,
//! snowy slopes and plains; Terralith's no forest), and EF the glacier model does not cover is a
//! fixed 1.5 (barren frozen ground: Terralith's gravel desert and cold shrubland, thin snow in
//! vanilla, which has no dry frozen biome). Ben, 2026-09-29: ice only where there is ice.
//!
//! These two are computed per L1 cell (where the climate lives), filled across the other medium
//! by nearest value (so a coastal land cell interpolates between land values only), resampled
//! to L2 and chosen by the L2 land mask.
//!
//! **Continentalness** from the distance to the sea on land — 0, `beach_blocks`,
//! `inland_mid_blocks` (200), `inland_far_blocks` (1,000), `inland_interior_blocks` (5,000) blocks →
//! the coast band's bottom, near-inland, mid-inland, far-inland, top — and from
//! the depth at sea — 0, 200, 2,500, 6,000 m → coast | ocean, the shelf, ocean | deep ocean, the
//! abyss.
//!
//! **Erosion** from the local relief (range of the ground, the sea surface counted as ground, in
//! a `terrain_window_blocks` window, box-smoothed): its rank among the land cells, high relief
//! first, read through the reference's land erosion distribution below the swamp band (E6).
//! Wet (vegetation ≥ `wetland_humidity`, 2.5), flat (the flattest `wetland_flat_share`, 35 %), low
//! (< `wetland_low_m`, 150 m) land that is not frozen goes
//! into E6 in proportion to `wetlands` — swamps and mangroves only where they belong (vanilla
//! puts E6 at hot temperatures to mangrove swamp, whatever the humidity). The deep sea keeps E4 or
//! more, away from Terralith's sky-island band. A class's placement may clamp its land's erosion
//! (`erosion: {min, max}`; EF's barren ground is held below 0.3, off Terralith's packed-ice glacial
//! chasm, and its ice cap is not).
//!
//! **Weirdness** from the ridge/valley index (the ground minus its window mean): its rank among the
//! land cells read through the reference's peaks-and-valleys distribution above −0.8, turned into
//! |weirdness| on the rising branch (|w| = (PV + 1)/3), signed by a value noise of wavelength
//! `variant_blocks` (vanilla's variant biomes), except for a class whose placement names its branch:
//! `weirdness: positive`, or `{positive_below_erosion: e}` (positive where the erosion is below `e`,
//! negative above, except the `flats` it names: positive where the variant noise is above a value, at
//! a |weirdness| of at most a cap). EF has the second: its peaks are frozen peaks, never the negative
//! branch's jagged peaks or Terralith's scarlet mountains, and its flats are off the positive-branch
//! variants such as the cherry grove, except a fifth of them at low |weirdness|, where vanilla's ice
//! spikes are. PV stays above the valley slice: no river biomes
//! until rivers exist (Phase 2).

use std::sync::OnceLock;

use deeptime_format::{GridDesc, Noise};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::placement::{Placement, Weirdness};
use crate::climate::koppen;
use crate::error::CoreError;
use crate::world::Field;

/// The reference: band edges and vanilla's land distributions (one file with the game).
pub const REFERENCE_JSON: &str = include_str!("../../data/climate_mapping/default.json");

/// `notes.bake.climate.mapping`: which mapping wrote the layers.
pub const MAPPING: &str = "p1-09";

/// `w_thr` of a class that always keeps the positive branch (above any erosion).
const ALWAYS_POSITIVE: f32 = 9.0;

/// `w_cap` of a class whose flats have no limit on |weirdness|.
const NO_CAP: f32 = 9.0;

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
    /// Distance from the sea, blocks, where the mid-inland continentalness band begins.
    pub inland_mid_blocks: f64,
    /// Distance from the sea, blocks, where the far-inland band begins.
    pub inland_far_blocks: f64,
    /// Distance from the sea, blocks, where continentalness reaches its top (the deep interior).
    pub inland_interior_blocks: f64,
    /// The humidity band coordinate (0 … 5) from which land counts as wet for swamps; the wetness
    /// reaches its full weight 1.5 bands above it.
    pub wetland_humidity: f64,
    /// The share of land, flattest first, that lowland swamps may take (by relief rank).
    pub wetland_flat_share: f64,
    /// Height above the sea, m, at which lowland stops counting as low for swamps.
    pub wetland_low_m: f64,
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
            inland_mid_blocks: 200.0,
            inland_far_blocks: 1000.0,
            inland_interior_blocks: 5000.0,
            wetland_humidity: 2.5,
            wetland_flat_share: 0.35,
            wetland_low_m: 150.0,
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
        within(self.wetlands, 0.0, 1.0, "wetlands")?;
        // The ranges keep the coast band, the three inland distances in this order.
        within(self.inland_mid_blocks, 160.0, 800.0, "inland_mid_blocks")?;
        within(self.inland_far_blocks, 900.0, 3000.0, "inland_far_blocks")?;
        within(
            self.inland_interior_blocks,
            3500.0,
            20_000.0,
            "inland_interior_blocks",
        )?;
        within(self.wetland_humidity, 1.0, 4.5, "wetland_humidity")?;
        within(self.wetland_flat_share, 0.05, 0.9, "wetland_flat_share")?;
        within(self.wetland_low_m, 20.0, 600.0, "wetland_low_m")
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

pub(super) fn unit(v: f64) -> f64 {
    v.clamp(0.0, 1.0)
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

/// Distance to the nearest cell of the other medium (land ↔ sea), in cells (x wraps). Cells of
/// the target medium get 0; every cell gets infinity when there is none.
///
/// The exact Euclidean distance (P1-23). It was a two-pass 3 × 3 chamfer (steps 1 and √2), whose
/// lines of equal distance are octagons: up to 8 % too far at 22.5° off the axes, so that the
/// inland bands of the continentalness ran in straight segments along the axes and the diagonals.
/// Two separable passes: along each row the distance to the row's nearest source (wrapping), then
/// down each column the lower envelope of the parabolas (j − j′)² + row(j′)² (Felzenszwalb and
/// Huttenlocher's distance transform of a sampled function, written from the paper's description).
pub fn distance_cells(land: &[bool], w: usize, h: usize, to_sea: bool) -> Vec<f32> {
    let src: Vec<bool> = land.iter().map(|&l| l != to_sea).collect();
    if !src.iter().any(|&s| s) {
        return vec![f32::INFINITY; land.len()];
    }
    // Along the rows, wrapping: two sweeps over the row laid out twice.
    let mut row = vec![f64::INFINITY; land.len()];
    row.par_chunks_mut(w)
        .zip(src.par_chunks(w))
        .for_each(|(d, s)| {
            let mut since = f64::INFINITY;
            for k in 0..2 * w {
                let i = k % w;
                since = if s[i] { 0.0 } else { since + 1.0 };
                d[i] = d[i].min(since);
            }
            let mut since = f64::INFINITY;
            for k in (0..2 * w).rev() {
                let i = k % w;
                since = if s[i] { 0.0 } else { since + 1.0 };
                d[i] = d[i].min(since);
            }
        });
    // Down the columns: the lower envelope of the rows' parabolas.
    let cols: Vec<Vec<f32>> = (0..w)
        .into_par_iter()
        .map(|i| {
            let g: Vec<f64> = (0..h).map(|j| row[j * w + i] * row[j * w + i]).collect();
            // v: the rows whose parabolas make the envelope; z[k]: where v[k]'s part begins.
            let mut v: Vec<usize> = Vec::with_capacity(h);
            let mut z: Vec<f64> = Vec::with_capacity(h + 1);
            for q in (0..h).filter(|&q| g[q].is_finite()) {
                loop {
                    let Some(&p) = v.last() else {
                        v.push(q);
                        z.push(f64::NEG_INFINITY);
                        break;
                    };
                    let (qf, pf) = (q as f64, p as f64);
                    let s = ((g[q] + qf * qf) - (g[p] + pf * pf)) / (2.0 * (qf - pf));
                    if s <= z[v.len() - 1] {
                        v.pop();
                        z.pop();
                    } else {
                        v.push(q);
                        z.push(s);
                        break;
                    }
                }
            }
            let mut k = 0;
            (0..h)
                .map(|j| {
                    while k + 1 < v.len() && z[k + 1] < j as f64 {
                        k += 1;
                    }
                    let dj = j as f64 - v[k] as f64;
                    (dj * dj + g[v[k]]).sqrt() as f32
                })
                .collect()
        })
        .collect();
    let mut out = vec![0f32; land.len()];
    out.par_chunks_mut(w).enumerate().for_each(|(j, r)| {
        for (i, o) in r.iter_mut().enumerate() {
            *o = cols[i][j];
        }
    });
    out
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
    /// The class's erosion limits (placement `erosion`; ±2 where it has none), nearest-filled
    /// across the sea like the others.
    e_lo: Vec<f32>,
    e_hi: Vec<f32>,
    /// The class's variant-noise rule (placement `weirdness`): NaN = the noise signs it, [`ALWAYS_POSITIVE`]
    /// = the positive branch, else the erosion below which it is positive (negative above).
    w_thr: Vec<f32>,
    /// For a class with flats: the variant noise above which they are positive (1 = never), and the
    /// largest |weirdness| they take ([`NO_CAP`] = none); NaN / [`NO_CAP`] otherwise.
    w_noise: Vec<f32>,
    w_cap: Vec<f32>,
    by_class: Value,
}

fn coords(c: &Climate<'_>, l1: &GridDesc, placement: &Placement) -> Coords {
    let (w, h) = (l1.w as usize, l1.h as usize);
    let n = w * h;
    // A missing field or a NaN cell falls back to the defaults below (never to "driest").
    let at =
        |f: Option<&Field>, k: usize| f.map(|f| f64::from(f.data[k])).filter(|v| v.is_finite());
    let mut t_land = vec![f32::NAN; n];
    let mut h_land = vec![f32::NAN; n];
    let mut t_sea = vec![f32::NAN; n];
    let mut e_lo = vec![f32::NAN; n];
    let mut e_hi = vec![f32::NAN; n];
    let mut w_thr = vec![f32::NAN; n];
    let mut w_noise = vec![1.0f32; n];
    let mut w_cap = vec![NO_CAP; n];
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
        let th = placement.thermal_land(class, tm, tw, tc, glacier);
        let hu = placement.humidity_land(class, ai, glacier);
        t_land[k] = th as f32;
        h_land[k] = hu as f32;
        let lim = placement.erosion_limit(class, glacier);
        e_lo[k] = lim.map_or(-2.0, |l| l.min as f32);
        e_hi[k] = lim.map_or(2.0, |l| l.max as f32);
        w_thr[k] = match placement.weirdness(class) {
            Weirdness::Noise => f32::NAN,
            Weirdness::Positive => ALWAYS_POSITIVE,
            Weirdness::PositiveBelowErosion { erosion, flats } => {
                w_noise[k] = flats.positive_noise_above as f32;
                w_cap[k] = flats.max_magnitude.map_or(NO_CAP, |m| m as f32);
                erosion as f32
            }
        };
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
    if !fill_nearest(&mut e_lo, &land, w, h) || !fill_nearest(&mut e_hi, &land, w, h) {
        e_lo = vec![-2.0; n];
        e_hi = vec![2.0; n];
    }
    if !fill_nearest(&mut w_thr, &land, w, h)
        || !fill_nearest(&mut w_noise, &land, w, h)
        || !fill_nearest(&mut w_cap, &land, w, h)
    {
        w_thr = vec![f32::NAN; n];
        w_noise = vec![1.0; n];
        w_cap = vec![NO_CAP; n];
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
        e_lo,
        e_hi,
        w_thr,
        w_noise,
        w_cap,
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
pub fn map(
    climate: Option<&Climate<'_>>,
    g: &Ground<'_>,
    p: &BiomeParams,
    placement: &Placement,
) -> Mapped {
    let r = reference();
    let bands = &r.bands;
    let l2 = g.l2;
    let (w, h) = (l2.w as usize, l2.h as usize);
    let n = w * h;
    let sea = g.sea_level_m;
    let elev = g.elevation_m;
    let land: Vec<bool> = elev.iter().map(|&z| f64::from(z) > sea).collect();

    // Temperature and humidity.
    let (t_land, h_land, t_sea, (e_lo, e_hi, w_thr, w_noise, w_cap), by_class) = match climate {
        Some(c) => {
            let co = coords(c, g.l1, placement);
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
            // The class's rules are not blended across a border: the nearest L1 cell's.
            let near = |v: Vec<f32>| {
                super::resample(
                    &Field {
                        grid: g.l1.clone(),
                        data: v,
                    },
                    l2,
                    true,
                )
            };
            (
                up(co.t_land),
                up(co.h_land),
                up(co.t_sea),
                (
                    near(co.e_lo),
                    near(co.e_hi),
                    near(co.w_thr),
                    near(co.w_noise),
                    near(co.w_cap),
                ),
                co.by_class,
            )
        }
        None => {
            let t: Vec<f32> = (0..n)
                .map(|k| {
                    let j = k / w;
                    let lat = 90.0 - 180.0 * (j as f64 + 0.5) / h as f64;
                    thermal_plain(28.0 - 0.55 * lat.abs()) as f32
                })
                .collect();
            (
                t.clone(),
                vec![2.5; n],
                t,
                (
                    vec![-2.0; n],
                    vec![2.0; n],
                    vec![f32::NAN; n],
                    vec![1.0; n],
                    vec![NO_CAP; n],
                ),
                Value::Null,
            )
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
    let beach = p.beach_blocks.min(0.95 * p.inland_mid_blocks);
    let land_anchors = [
        (0.0, 3.0),
        (beach.max(1e-3), 4.0),
        (p.inland_mid_blocks, 5.0),
        (p.inland_far_blocks, 6.0),
        (p.inland_interior_blocks, 7.0),
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
        let wet = unit((f64::from(h_land[k]) + p.humidity_shift - p.wetland_humidity) / 1.5);
        let flat = unit((p.wetland_flat_share - u) / p.wetland_flat_share);
        let low = unit(1.0 - (f64::from(elev[k]) - sea) / p.wetland_low_m);
        let thawed = f64::from(t_land[k]) + p.temperature_shift >= 1.0;
        let s = p.wetlands * wet * flat * low;
        if thawed && s > 0.15 {
            e = e6 + 0.01 + (0.95 - e6 - 0.01) * unit((s - 0.15) / 0.6);
            wet_cells += 1;
        }
        // The class's own limits (EF: no packed-ice glacial chasm on barren polar desert).
        erosion[k] = (e as f32).clamp(e_lo[k], e_hi[k].max(e_lo[k]));
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
            let k = j * w + i;
            let m = mag[k];
            let thr = w_thr[k];
            let value = || noise.value(bx, bz, p.variant_blocks, VARIANT_SALT);
            let (negative, m) = if !land[k] || thr.is_nan() {
                (value() < 0.0, m)
            } else if thr > ALWAYS_POSITIVE - 1.0 || erosion[k] < thr {
                // A class that keeps the positive branch: all of it, or its high relief.
                (false, m)
            } else {
                // Its flats: negative unless the class lets some noise through, at most `w_cap` high.
                (value() <= f64::from(w_noise[k]), m.min(w_cap[k]))
            };
            *o = if negative { -m } else { m };
        }
    });

    let land_cells = land.iter().filter(|&&l| l).count();
    let note = json!({
        "mapping": MAPPING,
        "params": p,
        "placement": {
            "source": if placement == super::placement::builtin() { "default" } else { "content bundle" },
            "table": placement.table(),
        },
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
    use crate::bake::placement;

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
        let t = placement::builtin().thermal_land(k, t_mean, t_warm, t_cold, false);
        let hu = placement::builtin().humidity_land(k, aridity, false);
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
        assert!(placement::builtin().humidity_land(29, 5.0, false) < 2.0);
        assert_eq!(middle("EF", -55.0, -30.0, -70.0, 0.5), "snowy_plains");
        // Ice cap (glaciated EF) is humidity band 0, whatever the aridity index says.
        let (ef, p) = (koppen::index("EF").unwrap(), placement::builtin());
        let ice = p.thermal_land(ef, -55.0, -30.0, -70.0, true);
        let ice_h = p.humidity_land(ef, 10.0, true);
        assert_eq!(
            MIDDLE[ice.floor() as usize][ice_h.floor() as usize],
            "snowy_plains"
        );
        assert!(
            ice_h < 1.0,
            "band {ice_h}: ice spikes are the frozen band's first humidity band"
        );
        // Glaciated land is ice, whatever its class.
        assert!(placement::builtin().thermal_land(26, 5.0, 18.0, -8.0, true) < 1.0);
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

    /// P1-23: the distance to the sea has no grid in it. Round a single sea cell the lines of
    /// equal distance are circles (the chamfer this replaced made octagons, 8 % out at 22.5°),
    /// across the seam too, and on a random coast no cell is further from the brute-force answer
    /// than rounding.
    #[test]
    fn distance_to_the_sea_is_euclidean() {
        let (w, h) = (96usize, 81usize);
        for &(si, sj) in &[(48usize, 40usize), (0, 40), (95, 3)] {
            let land: Vec<bool> = (0..w * h).map(|c| c != sj * w + si).collect();
            let d = distance_cells(&land, w, h, true);
            let mut worst = 0.0f64;
            for j in 0..h {
                for i in 0..w {
                    let dx = (i as f64 - si as f64).abs();
                    let dx = dx.min(w as f64 - dx);
                    let want = dx.hypot(j as f64 - sj as f64);
                    worst = worst.max((f64::from(d[j * w + i]) - want).abs());
                }
            }
            assert!(worst < 1e-4, "source ({si}, {sj}): off by {worst} cells");
        }
        // The direction of the gradient is not held to the grid's eight: on a ring 30 cells from
        // the source the distance is 30 everywhere, not 30 on the axes and 32.5 between them.
        let land: Vec<bool> = (0..w * h).map(|c| c != 40 * w + 48).collect();
        let d = distance_cells(&land, w, h, true);
        let at = |a: f64| {
            let (i, j) = (48.0 + 30.0 * a.cos(), 40.0 + 30.0 * a.sin());
            let (i, j) = (i.round(), j.round());
            (
                f64::from(d[j as usize * w + i as usize]),
                (i - 48.0).hypot(j - 40.0),
            )
        };
        for k in 0..64 {
            let (got, want) = at(k as f64 * std::f64::consts::TAU / 64.0);
            assert!((got - want).abs() < 1e-4, "{got} {want}");
        }
        // A random coast against brute force.
        let mut rng = crate::synth::SplitMix(23);
        let land: Vec<bool> = (0..w * h).map(|_| rng.below(40) != 0).collect();
        let d = distance_cells(&land, w, h, true);
        let seas: Vec<(i64, i64)> = (0..w * h)
            .filter(|&c| !land[c])
            .map(|c| ((c % w) as i64, (c / w) as i64))
            .collect();
        for c in (0..w * h).step_by(7) {
            let (i, j) = ((c % w) as i64, (c / w) as i64);
            let want = seas
                .iter()
                .map(|&(si, sj)| {
                    let dx = (i - si).abs().min(w as i64 - (i - si).abs()) as f64;
                    dx.hypot((j - sj) as f64)
                })
                .fold(f64::INFINITY, f64::min);
            assert!(
                (f64::from(d[c]) - want).abs() < 1e-4,
                "cell {c}: {} {want}",
                d[c]
            );
        }
        // No sea at all: infinitely far.
        assert!(
            distance_cells(&[true; 12], 4, 3, true)
                .iter()
                .all(|x| x.is_infinite())
        );
    }

    /// A small planet: 64 x 32 L1 cells, L2 twice as fine; land in the middle rows with an Af, a
    /// BWh, a Dfc and an ET quarter, a ridge through the middle. Returns the mapped layers and
    /// L2's width.
    fn fixture(params: &BiomeParams, table: &Placement) -> (Mapped, usize) {
        fixture_of(params, table, ["Af", "BWh", "Dfc", "ET"], [false; 4])
    }

    /// As [`fixture`], with the Köppen class of each quarter (`Af`, `BWh`, `Dfc`, `ET` or `EF`; each
    /// class has its climate) and whether the glacier flag is set in it.
    fn fixture_of(
        params: &BiomeParams,
        table: &Placement,
        classes: [&str; 4],
        glaciated: [bool; 4],
    ) -> (Mapped, usize) {
        let c = 1024u32;
        let l1 = GridDesc::for_planet("L1", c, 16).unwrap();
        let l2 = GridDesc::for_planet("L2", c, 8).unwrap();
        let (w1, h1) = (l1.w as usize, l1.h as usize);
        let quarter = |i: usize| classes[i * 4 / w1];
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
            "EF" => -40.0,
            _ => -8.0,
        });
        let tw = field(&|i, _| match quarter(i) {
            "Af" => 28.0,
            "BWh" => 36.0,
            "Dfc" => 16.0,
            "EF" => -30.0,
            _ => 5.0,
        });
        let tc = field(&|i, _| match quarter(i) {
            "Af" => 26.0,
            "BWh" => 14.0,
            "Dfc" => -15.0,
            "EF" => -50.0,
            _ => -20.0,
        });
        // P/PET is pinned at 10 below freezing (PET = 0), as the climate stage writes it.
        let ai = field(&|i, _| match quarter(i) {
            "Af" => 2.5,
            "BWh" => 0.03,
            "Dfc" => 1.2,
            "EF" => 10.0,
            _ => 1.0,
        });
        let gl = field(&|i, _| f32::from(u8::from(glaciated[i * 4 / w1])));
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
            glacier: Some(&gl),
        };
        let g = Ground {
            elevation_m: &elev,
            sea_level_m: 0.0,
            l1: &l1,
            l2: &l2,
            seed: 3,
            circumference: c,
        };
        let m = map(Some(&climate), &g, params, table);
        (m, w2)
    }

    #[test]
    fn a_planet_maps_by_its_koppen_map() {
        let (m, w2) = fixture(&BiomeParams::default(), placement::builtin());
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

    /// The layer's band coordinate at the middle of an L2 quarter, at a lowland row.
    fn quarter_band(m: &Mapped, w2: usize, layer: usize, edges: &[f64], q: usize) -> f64 {
        let i = q * w2 / 4 + w2 / 8 + 3;
        band_coord(edges, f64::from(m.layers[layer][32 * w2 + i]))
    }

    #[test]
    fn a_changed_placement_table_changes_the_biome_inputs() {
        let b = &reference().bands;
        let (base, w2) = fixture(&BiomeParams::default(), placement::builtin());
        assert_eq!(base.note["placement"]["source"], "default");
        // Af (quarter 0) goes to the desert ramp and the tundra rule: from hot and wet to a
        // desert's temperature and the dry end of the humidity bands.
        let mut v: serde_json::Value = serde_json::from_str(placement::PLACEMENT_JSON).unwrap();
        v["classes"]["Af"]["temperature"] = json!("desert");
        v["classes"]["Af"]["humidity"] = json!("tundra");
        let table = Placement::from_value(&v).unwrap();
        let (changed, w2b) = fixture(&BiomeParams::default(), &table);
        assert_eq!(w2, w2b);
        assert_eq!(changed.note["placement"]["source"], "content bundle");
        assert_eq!(
            changed.note["placement"]["table"]["classes"]["Af"]["humidity"],
            "tundra"
        );
        let (t0, t1) = (
            quarter_band(&base, w2, 0, &b.temperature, 0),
            quarter_band(&changed, w2, 0, &b.temperature, 0),
        );
        assert!((3.0..4.0).contains(&t0), "Af was warm: {t0}");
        assert!(t1 >= 4.05, "Af is now a desert's temperature: {t1}");
        let (h0, h1) = (
            quarter_band(&base, w2, 1, &b.vegetation, 0),
            quarter_band(&changed, w2, 1, &b.vegetation, 0),
        );
        assert!(h0 >= 3.0, "Af was humid: {h0}");
        assert!(h1 <= 1.96, "Af is now capped dry: {h1}");
        // The other classes did not move: same bits in the other three quarters.
        for q in 1..4 {
            for layer in [0, 1] {
                let i = q * w2 / 4 + w2 / 8 + 3;
                assert_eq!(
                    base.layers[layer][32 * w2 + i].to_bits(),
                    changed.layers[layer][32 * w2 + i].to_bits(),
                    "quarter {q} layer {layer}"
                );
            }
        }
        // The layers that do not read the table are the same everywhere.
        assert_eq!(base.layers[2], changed.layers[2], "continentalness");
        assert_eq!(base.layers[4], changed.layers[4], "weirdness");
    }

    /// Ben, 2026-09-29, "Ice only where there's ice": EF above its equilibrium line is ice cap
    /// (humidity band 0: vanilla's ice spikes and snowy slopes, no forest), the rest of EF is barren
    /// frozen ground (band 1), and both stay in the upper half of the frozen band, where Terralith's
    /// dry cold biomes are and its terracotta snowy badlands are not; ET is untouched.
    #[test]
    fn ef_maps_to_ice_cap_where_glaciated_and_barren_ground_elsewhere() {
        let b = &reference().bands;
        let table = placement::builtin();
        let (base, w2) = fixture(&BiomeParams::default(), table);
        let (m, w2b) = fixture_of(
            &BiomeParams::default(),
            table,
            ["EF", "EF", "Dfc", "ET"],
            [true, false, false, false],
        );
        assert_eq!(w2, w2b);
        let t = |q: usize| quarter_band(&m, w2, 0, &b.temperature, q);
        let h = |q: usize| quarter_band(&m, w2, 1, &b.vegetation, q);
        for q in [0, 1] {
            assert!(
                (0.5..1.0).contains(&t(q)),
                "EF quarter {q} sits in the upper frozen band: {}",
                t(q)
            );
        }
        assert!(h(0) < 1.0, "ice cap is humidity band 0: {}", h(0));
        assert!(
            (1.0..2.0).contains(&h(1)),
            "barren ground is humidity band 1: {}",
            h(1)
        );
        // Dfc (quarter 2) and ET (quarter 3) are where the default table put them.
        for q in 2..4 {
            for layer in [0, 1] {
                let i = q * w2 / 4 + w2 / 8 + 3;
                assert_eq!(
                    base.layers[layer][32 * w2 + i].to_bits(),
                    m.layers[layer][32 * w2 + i].to_bits(),
                    "quarter {q} layer {layer}"
                );
            }
        }
    }

    /// EF's peaks keep the positive branch of the variant noise (vanilla's frozen peaks, no
    /// Terralith scarlet mountains) and its flats the negative one (no cherry grove, which is a
    /// positive-branch variant); barren EF is held below erosion 0.3 (off the packed-ice glacial
    /// chasm), ice cap is not. Without the keys the noise signs the weirdness and the erosion is
    /// the relief's.
    #[test]
    fn ef_peaks_are_positive_its_flats_negative_and_barren_ef_is_held_off_the_chasm() {
        // The default table lets some noise through on the flats (the next test): without that.
        let mut base: serde_json::Value = serde_json::from_str(placement::PLACEMENT_JSON).unwrap();
        base["classes"]["EF"]["weirdness"] = json!({"positive_below_erosion": -0.27});
        let table = &Placement::from_value(&base).unwrap();
        // A short wavelength, so that the noise changes sign within the small planet.
        let params = BiomeParams {
            variant_blocks: 64.0,
            ..BiomeParams::default()
        };
        let all_ef = ["EF"; 4];
        let glaciated = [true, false, true, false];
        let (m, w2) = fixture_of(&params, table, all_ef, glaciated);
        // The layers' land cells of each quarter, two columns in from the quarter's edges (an L2
        // cell takes its class from the nearest L1 cell).
        let cells = |q: usize| {
            let (lo, hi) = (q * w2 / 4 + 2, (q + 1) * w2 / 4 - 2);
            (16..48usize)
                .flat_map(move |j| (lo..hi).map(move |i| j * w2 + i))
                .collect::<Vec<_>>()
        };
        let (mut peaks, mut flats) = (0, 0);
        for q in 0..4 {
            for k in cells(q) {
                let (e, w) = (m.layers[3][k], m.layers[4][k]);
                if e < -0.27 {
                    peaks += 1;
                    assert!(w > 0.0, "a peak (erosion {e}) is positive: {w}");
                } else {
                    flats += 1;
                    assert!(w < 0.0, "a flat (erosion {e}) is negative: {w}");
                }
            }
        }
        assert!(peaks > 100 && flats > 100, "peaks {peaks}, flats {flats}");
        let top = |m: &Mapped, q: usize| {
            cells(q)
                .into_iter()
                .map(|k| m.layers[3][k])
                .fold(f32::MIN, f32::max)
        };
        // Quarters 1 and 3 are barren (not glaciated): erosion at most 0.3; 0 and 2 are ice cap.
        for q in [1, 3] {
            assert!(
                top(&m, q) <= 0.3 + 1e-6,
                "barren quarter {q}: {}",
                top(&m, q)
            );
        }
        for q in [0, 2] {
            assert!(
                top(&m, q) > 0.3,
                "ice cap quarter {q} keeps its flats: {}",
                top(&m, q)
            );
        }
        // The same planet without EF's keys: the relief's erosion everywhere, the noise's signs.
        let mut v: serde_json::Value = serde_json::from_str(placement::PLACEMENT_JSON).unwrap();
        let ef = v["classes"]["EF"].as_object_mut().unwrap();
        ef.remove("weirdness");
        ef.remove("erosion");
        ef.get_mut("glaciated")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove("erosion");
        let (free, _) = fixture_of(
            &params,
            &Placement::from_value(&v).unwrap(),
            all_ef,
            glaciated,
        );
        assert!(
            top(&free, 1) > 0.3,
            "barren EF without the limit: {}",
            top(&free, 1)
        );
        let negative_peak = (0..4)
            .flat_map(cells)
            .any(|k| free.layers[3][k] < -0.27 && free.layers[4][k] < 0.0);
        assert!(
            negative_peak,
            "the noise gives some peak the negative sign without the key"
        );
        // Magnitudes are the same either way.
        for (a, b) in m.layers[4].iter().zip(&free.layers[4]) {
            assert_eq!(a.abs().to_bits(), b.abs().to_bits());
        }
    }

    /// Ben, 2026-09-30, "Bring ice spikes back": some of EF's flats are on the positive branch, where
    /// vanilla's ice spikes are (a variant of its middle biomes), at a |weirdness| held below where
    /// Terralith's cherry groves are; its peaks stay positive and the rest of its flats negative.
    #[test]
    fn ef_flats_let_some_noise_through_below_a_magnitude() {
        let params = BiomeParams {
            variant_blocks: 64.0,
            ..BiomeParams::default()
        };
        let all_ef = ["EF"; 4];
        let with = |flats: serde_json::Value| {
            let mut v: serde_json::Value = serde_json::from_str(placement::PLACEMENT_JSON).unwrap();
            v["classes"]["EF"]["weirdness"] =
                json!({"positive_below_erosion": -0.27, "flats": flats});
            fixture_of(
                &params,
                &Placement::from_value(&v).unwrap(),
                all_ef,
                [true; 4],
            )
        };
        let (base, w2) = fixture_of(&params, placement::builtin(), all_ef, [true; 4]);
        let land: Vec<usize> = (16..48usize)
            .flat_map(|j| (8..w2 - 8).map(move |i| j * w2 + i))
            .collect();
        let stats = |m: &Mapped| {
            let (mut flats, mut positive, mut top) = (0usize, 0usize, 0f32);
            for &k in &land {
                let (e, w) = (m.layers[3][k], m.layers[4][k]);
                if e < -0.27 {
                    assert!(w > 0.0, "a peak stays positive: {w}");
                } else {
                    flats += 1;
                    positive += usize::from(w > 0.0);
                    top = top.max(w.abs());
                }
            }
            (flats, positive, top)
        };
        // The default table: the noise above 0.2 is positive, at most 0.25 high.
        let (flats, positive, top) = stats(&base);
        assert!(flats > 500, "flats: {flats}");
        assert!(
            positive > flats / 50,
            "some flats are positive: {positive} of {flats}"
        );
        assert!(
            positive < flats * 6 / 10,
            "but not most: {positive} of {flats}"
        );
        assert!(
            top <= 0.25 + 1e-6,
            "flats stay under the cherry magnitudes: {top}"
        );
        // Noise above 0 is half the noise; above 1 is none; with no cap the relief's magnitudes stand.
        let (half, _) = with(json!({"positive_noise_above": 0.0}));
        let (hf, hp, htop) = stats(&half);
        assert!(
            hp > positive && hp < hf,
            "noise above 0 gives more than above 0.2: {hp} vs {positive}"
        );
        assert!(
            htop > 0.25,
            "uncapped flats are taller than the cap: {htop}"
        );
        let (none, _) = with(json!({"positive_noise_above": 1.0, "max_magnitude": 0.1}));
        let (nf, np, ntop) = stats(&none);
        assert_eq!(np, 0, "no noise gets through above 1: {np} of {nf}");
        assert!(ntop <= 0.1 + 1e-6, "a cap of 0.1: {ntop}");
    }

    #[test]
    fn the_inland_distances_and_wetland_thresholds_are_settings() {
        let b = &reference().bands;
        let table = placement::builtin();
        let (base, w2) = fixture(&BiomeParams::default(), table);
        // The fixture's land is at most 128 blocks from the sea: a nearer mid-inland edge puts the
        // same cell further into the near-inland band.
        let near = BiomeParams {
            inland_mid_blocks: 160.0,
            ..BiomeParams::default()
        };
        let (n, _) = fixture(&near, table);
        let (c0, c1) = (
            quarter_band(&base, w2, 2, &b.continents, 0),
            quarter_band(&n, w2, 2, &b.continents, 0),
        );
        assert!(c1 > c0, "mid-inland edge 160 > 200: {c1} vs {c0}");
        // The far edges only matter past 200 blocks: nothing moves here.
        let far = BiomeParams {
            inland_far_blocks: 900.0,
            inland_interior_blocks: 3500.0,
            ..BiomeParams::default()
        };
        assert_eq!(fixture(&far, table).0.layers[2], base.layers[2]);
        // The wetlands: this fixture's land stands 200 m above the sea, so at the default 150 m
        // there is none; a 600 m ceiling admits it, and a stricter humidity threshold or a
        // smaller flat share takes some away.
        assert_eq!(base.note["wetland_cells"], 0);
        let low = BiomeParams {
            wetland_low_m: 600.0,
            ..BiomeParams::default()
        };
        let (l, _) = fixture(&low, table);
        let wet = l.note["wetland_cells"].as_u64().unwrap();
        assert!(wet > 0, "wetlands with a 600 m ceiling");
        let dry = BiomeParams {
            wetland_low_m: 600.0,
            wetland_humidity: 4.5,
            ..BiomeParams::default()
        };
        let (d, _) = fixture(&dry, table);
        assert!(
            d.note["wetland_cells"].as_u64().unwrap() < wet,
            "a stricter humidity threshold"
        );
        let thin = BiomeParams {
            wetland_low_m: 600.0,
            wetland_flat_share: 0.05,
            ..BiomeParams::default()
        };
        let (t, _) = fixture(&thin, table);
        assert!(
            t.note["wetland_cells"].as_u64().unwrap() < wet,
            "a smaller flat share"
        );
    }

    #[test]
    fn the_new_settings_are_checked() {
        let bad = |f: &dyn Fn(&mut BiomeParams)| {
            let mut p = BiomeParams::default();
            f(&mut p);
            match p.validate().expect_err("refused") {
                CoreError::PresetInvalid { path, .. } => path,
                e => panic!("{e}"),
            }
        };
        assert!(BiomeParams::default().validate().is_ok());
        assert_eq!(
            bad(&|p| p.inland_mid_blocks = 100.0),
            "$.biomes.inland_mid_blocks"
        );
        assert_eq!(
            bad(&|p| p.inland_far_blocks = 3500.0),
            "$.biomes.inland_far_blocks"
        );
        assert_eq!(
            bad(&|p| p.inland_interior_blocks = 1000.0),
            "$.biomes.inland_interior_blocks"
        );
        assert_eq!(
            bad(&|p| p.wetland_humidity = 5.0),
            "$.biomes.wetland_humidity"
        );
        assert_eq!(
            bad(&|p| p.wetland_flat_share = 0.0),
            "$.biomes.wetland_flat_share"
        );
        assert_eq!(
            bad(&|p| p.wetland_low_m = f64::NAN),
            "$.biomes.wetland_low_m"
        );
    }
}
