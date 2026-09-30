// SPDX-License-Identifier: LGPL-3.0-or-later
//! Mountain character (decisions log 2026-09-29, "block-scale mountain character in Phase 1"): the
//! last amplification level's terrain gets the shape a player reads at block scale — spurs and
//! gullies down every flank, strike ridges in fold belts, cliff bands on hard rock, sharp young
//! peaks and rounded old ones — out of the simulation's own fields, not free noise:
//!
//! - **Drainage-aligned gullies.** Octaves of *stripe noise* whose crests run straight down the
//!   local slope (the gradient of the smoothed terrain, so of the amplifier's multiple-flow valleys),
//!   each octave aligned with the slope the coarser ones left, so fine gullies turn into the walls of
//!   coarse ones and join them as tributaries. The depth of an octave is `gully_gain · slope ·
//!   wavelength` (steeper ground cuts deeper), at most `gully_relief_cap` of the local relief, and
//!   fades out on rivers (the multiple-flow drainage area) and at the coast.
//! - **Fold ridges.** Where the plate model left an orogen (`plates/orogeny_type`) with a fold trend
//!   (`plates/fold_trend_deg`), ridged stripes run along the strike: parallel ranges, their height a
//!   share of the local relief. The gullies then drain off them into strike valleys (trellis
//!   drainage); rivers cut through them (water gaps).
//! - **Young peaks.** Where uplift is recent (`plates/orogeny_age_myr`, `plates/uplift_mm_yr`) the
//!   terrain's departure from its smoothed self is exaggerated (unsharp masking): higher, sharper
//!   summits and ridgelines, deeper valleys. Old ranges keep rounded (cosine) gully profiles and less
//!   relief.
//! - **Hard rock.** The rock's erodibility (`young/erodibility`, from the surface rock class) sets a
//!   hardness: hard rock gets sharper (triangle) gully profiles and, on steep ground, cliff bands
//!   (a smooth terrace of the height: benches and steep risers); soft rock keeps gentle slopes.
//!
//! Everything is a pure function of the window's arrays within [`radius_cells`] of a cell and of
//! the cell's global coordinates (the noise lattice is periodic in x with period C, like
//! `deeptime_format::detail::Noise`), so a tiled run equals a whole-raster one bit for bit and the
//! wrap is seamless. `ruggedness` (`amplify.ruggedness`, the editor's slider, 0..2) scales every
//! term; 0 leaves the terrain exactly as the passes made it.

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::window::{Win, map_cells};
use crate::error::CoreError;
use crate::world::WorldState;

/// The `amplify.character` section: how the ruggedness is spent (every key optional).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct CharacterParams {
    /// Longest gully wavelength, blocks; each further octave halves it (octaves under four cells
    /// are dropped: the game draws the terrain from corners four blocks apart).
    pub gully_wavelength_blocks: f64,
    pub gully_octaves: u32,
    /// Gully depth per unit of slope (blocks per block) and of wavelength.
    pub gully_gain: f64,
    /// An octave's depth is at most this share of the local relief.
    pub gully_relief_cap: f64,
    /// … and at most this share of its wavelength (hard rock), so a gully's walls stay under
    /// about 2π times it in slope whatever the flank; soft rock gets 60 % of it.
    pub gully_max_ratio: f64,
    /// Each further octave's depth is also multiplied by this (on top of the halving wavelength).
    pub gully_persistence: f64,
    /// Fold ridges: stripe wavelength (two crests per wavelength), blocks, and height as a share
    /// of the local relief in a Himalayan-type belt.
    pub fold_wavelength_blocks: f64,
    pub fold_gain: f64,
    /// Unsharp masking of young terrain: the smoothing radius, blocks, and the gain at full youth.
    pub sharpen_radius_blocks: f64,
    pub sharpen_gain: f64,
    /// … and the smoothing of old terrain (the same mask with the opposite sign) at youth 0.
    pub round_gain: f64,
    /// Cliff bands on hard rock: bench height, blocks, and strength.
    pub terrace_step_blocks: f64,
    pub terrace_gain: f64,
    /// Radius of the local relief, blocks.
    pub relief_radius_blocks: f64,
    /// Orogeny age at which youth has fallen to 1/e, Myr.
    pub youth_myr: f64,
    /// Uplift rate that counts as fully active, mm/yr.
    pub active_uplift_mm_yr: f64,
    /// The character fades out between these effective drainage areas, km² (rivers stay smooth).
    pub channel_area_km2: [f64; 2],
    /// Fold ridges fade out on smaller streams already, so every stream the amplifier made cuts a
    /// gap through them instead of being dammed.
    pub fold_channel_area_km2: [f64; 2],
    /// … and within this height above the sea, blocks (beaches stay smooth).
    pub coast_blocks: f64,
    /// The most the character moves a cell, blocks.
    pub max_change_blocks: f64,
}

impl Default for CharacterParams {
    fn default() -> Self {
        CharacterParams {
            gully_wavelength_blocks: 128.0,
            gully_octaves: 4,
            gully_gain: 0.2,
            gully_relief_cap: 0.35,
            gully_max_ratio: 0.25,
            gully_persistence: 0.85,
            fold_wavelength_blocks: 112.0,
            fold_gain: 0.18,
            sharpen_radius_blocks: 24.0,
            sharpen_gain: 0.6,
            round_gain: 0.5,
            terrace_step_blocks: 8.0,
            terrace_gain: 0.8,
            relief_radius_blocks: 32.0,
            youth_myr: 60.0,
            active_uplift_mm_yr: 0.1,
            channel_area_km2: [5.0, 100.0],
            fold_channel_area_km2: [1.0, 10.0],
            coast_blocks: 6.0,
            max_change_blocks: 60.0,
        }
    }
}

impl CharacterParams {
    pub fn validate(&self, path: &str) -> Result<(), CoreError> {
        let bad = |k: &str, why: &str| CoreError::preset(format!("{path}.{k}"), why);
        let range = |v: f64, lo: f64, hi: f64, k: &str| {
            if v.is_finite() && (lo..=hi).contains(&v) {
                Ok(())
            } else {
                Err(bad(k, &format!("must be in [{lo}, {hi}]")))
            }
        };
        range(self.gully_wavelength_blocks, 8.0, 512.0, "gully_wavelength_blocks")?;
        if self.gully_octaves > 6 {
            return Err(bad("gully_octaves", "must be at most 6"));
        }
        range(self.gully_gain, 0.0, 2.0, "gully_gain")?;
        range(self.gully_relief_cap, 0.0, 2.0, "gully_relief_cap")?;
        range(self.gully_max_ratio, 0.0, 1.0, "gully_max_ratio")?;
        range(self.gully_persistence, 0.1, 1.0, "gully_persistence")?;
        range(self.fold_wavelength_blocks, 16.0, 1024.0, "fold_wavelength_blocks")?;
        range(self.fold_gain, 0.0, 2.0, "fold_gain")?;
        range(self.sharpen_radius_blocks, 4.0, 256.0, "sharpen_radius_blocks")?;
        range(self.sharpen_gain, 0.0, 4.0, "sharpen_gain")?;
        range(self.round_gain, 0.0, 1.0, "round_gain")?;
        range(self.terrace_step_blocks, 2.0, 64.0, "terrace_step_blocks")?;
        range(self.terrace_gain, 0.0, 2.0, "terrace_gain")?;
        range(self.relief_radius_blocks, 4.0, 256.0, "relief_radius_blocks")?;
        range(self.youth_myr, 1.0, 10_000.0, "youth_myr")?;
        range(self.active_uplift_mm_yr, 0.01, 100.0, "active_uplift_mm_yr")?;
        for (k, [a0, a1]) in [
            ("channel_area_km2", self.channel_area_km2),
            ("fold_channel_area_km2", self.fold_channel_area_km2),
        ] {
            if !(a0.is_finite() && a1.is_finite() && a0 > 0.0 && a1 > a0) {
                return Err(bad(k, "[a0, a1] with 0 < a0 < a1 (km²)"));
            }
        }
        range(self.coast_blocks, 0.0, 64.0, "coast_blocks")?;
        range(self.max_change_blocks, 1.0, 400.0, "max_change_blocks")?;
        Ok(())
    }
}

/// Ruggedness range (the editor's slider).
pub const RUGGEDNESS_MAX: f64 = 2.0;

/// The plate model's fields the character reads, on L1, smoothed (the plate fields are nearest
/// samples: stepped at the samples' Voronoi edges) and turned into youth and fold strength.
#[derive(Clone, Debug)]
pub struct CharacterFields {
    w: usize,
    h: usize,
    /// L3 cells per L1 cell (`l1_r / l3_r`).
    ratio: f64,
    /// Youth in [0, 1]: 1 where uplift is active or the last orogeny recent.
    youth: Vec<f32>,
    /// Fold strength times the shortening direction as a doubled angle (cos 2θ, sin 2θ): an axial
    /// direction averages without its sign cancelling.
    fold_c2: Vec<f32>,
    fold_s2: Vec<f32>,
    /// Background youth where the plate fields are absent.
    pub neutral: bool,
}

/// Field names the character reads (the plate model's L1 outputs).
pub mod source {
    pub const OROGENY_TYPE: &str = "plates/orogeny_type";
    pub const OROGENY_AGE: &str = "plates/orogeny_age_myr";
    pub const FOLD_TREND: &str = "plates/fold_trend_deg";
    pub const UPLIFT: &str = "plates/uplift_mm_yr";
}

/// The youth of a cell without plate fields (and of a cell no orogeny ever reached is 0).
const NEUTRAL_YOUTH: f32 = 0.35;

impl CharacterFields {
    /// From the world's L1 plate fields (`l1_r`, `l3_r`: blocks per cell); a world without them
    /// (test pipelines without plates) gets a uniform, moderate youth and no folds.
    pub fn from_world(world: &WorldState, l1_r: u32, l3_r: u32, p: &CharacterParams) -> Self {
        let ratio = f64::from(l1_r) / f64::from(l3_r);
        let get = |name: &str| world.fields.get(name);
        let (Some(kind), Some(age), Some(trend), Some(up)) = (
            get(source::OROGENY_TYPE),
            get(source::OROGENY_AGE),
            get(source::FOLD_TREND),
            get(source::UPLIFT),
        ) else {
            return CharacterFields::neutral();
        };
        let (w, h) = (kind.grid.w as usize, kind.grid.h as usize);
        let n = w * h;
        let mut youth = vec![0f32; n];
        let mut c2 = vec![0f32; n];
        let mut s2 = vec![0f32; n];
        youth
            .par_iter_mut()
            .zip(c2.par_iter_mut().zip(s2.par_iter_mut()))
            .enumerate()
            .for_each(|(c, (y, (fc, fs)))| {
                let k = kind.data[c];
                let a = f64::from(age.data[c]);
                let y_age = if k > 0.5 && a.is_finite() {
                    (-a.max(0.0) / p.youth_myr).exp()
                } else {
                    0.0
                };
                let u = f64::from(up.data[c]);
                let y_up = if u.is_finite() {
                    (u / p.active_uplift_mm_yr).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                *y = y_age.max(y_up) as f32;
                let t = f64::from(trend.data[c]);
                // Collision belts fold most; an Andean margin's shortening is weaker and wider.
                let strength = if k > 1.5 {
                    1.0
                } else if k > 0.5 {
                    0.4
                } else {
                    0.0
                };
                if strength > 0.0 && t.is_finite() {
                    // Shortening direction n = (cos θ, sin θ) in (x east, z south) for a strike of
                    // azimuth θ (from north, clockwise); as a doubled angle.
                    let th = 2.0 * t.to_radians();
                    let s = strength * (0.3 + 0.7 * y_age);
                    *fc = (s * th.cos()) as f32;
                    *fs = (s * th.sin()) as f32;
                }
            });
        for v in [&mut youth, &mut c2, &mut s2] {
            blur_l1(v, w, h, 2);
            blur_l1(v, w, h, 2);
        }
        CharacterFields {
            w,
            h,
            ratio,
            youth,
            fold_c2: c2,
            fold_s2: s2,
            neutral: false,
        }
    }

    /// The youth and the fold strength per L1 cell (row-major, the plate model's L1 grid).
    pub fn maps(&self) -> (Vec<f32>, Vec<f32>) {
        let fold = self
            .fold_c2
            .iter()
            .zip(&self.fold_s2)
            .map(|(&c, &s)| (c * c + s * s).sqrt().min(1.0))
            .collect();
        (self.youth.clone(), fold)
    }

    /// Uniform, moderate youth and no folds (a world without plate fields).
    pub fn neutral() -> Self {
        CharacterFields {
            w: 1,
            h: 1,
            ratio: 1.0,
            youth: vec![NEUTRAL_YOUTH],
            fold_c2: vec![0.0],
            fold_s2: vec![0.0],
            neutral: true,
        }
    }

    /// (youth, fold strength, shortening direction (x, z)) at global L3 cell (gi, gj): bilinear
    /// on L1, x wrapping.
    #[inline]
    fn at(&self, gi: usize, gj: usize) -> (f64, f64, (f64, f64)) {
        if self.neutral {
            return (f64::from(NEUTRAL_YOUTH), 0.0, (1.0, 0.0));
        }
        let u = gi as f64 / self.ratio;
        let v = (gj as f64 / self.ratio).min((self.h - 1) as f64);
        let (i0, j0) = (u.floor() as usize, v.floor() as usize);
        let (fu, fv) = (u - i0 as f64, v - j0 as f64);
        let i0 = i0 % self.w;
        let i1 = (i0 + 1) % self.w;
        let j1 = (j0 + 1).min(self.h - 1);
        let bl = |a: &[f32]| {
            let g = |i: usize, j: usize| f64::from(a[j * self.w + i]);
            let t = g(i0, j0) + (g(i1, j0) - g(i0, j0)) * fu;
            let b = g(i0, j1) + (g(i1, j1) - g(i0, j1)) * fu;
            t + (b - t) * fv
        };
        let y = bl(&self.youth);
        let (c2, s2) = (bl(&self.fold_c2), bl(&self.fold_s2));
        let m = (c2 * c2 + s2 * s2).sqrt();
        if m < 1e-6 {
            return (y, 0.0, (1.0, 0.0));
        }
        let (c2, s2) = (c2 / m, s2 / m);
        let dx = (0.5 * (1.0 + c2)).max(0.0).sqrt();
        let dz = (0.5 * (1.0 - c2)).max(0.0).sqrt() * if s2 < 0.0 { -1.0 } else { 1.0 };
        (y, m.min(1.0), (dx, dz))
    }
}

/// A box blur of radius `r` on an L1 array (x wraps, rows clamp).
fn blur_l1(v: &mut [f32], w: usize, h: usize, r: usize) {
    let mut tmp = vec![0f32; v.len()];
    let k = (2 * r + 1) as f32;
    tmp.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
        for (i, o) in row.iter_mut().enumerate() {
            let mut s = 0.0;
            for d in 0..=2 * r {
                let ii = (i + w + d - r) % w;
                s += v[j * w + ii];
            }
            *o = s / k;
        }
    });
    v.par_chunks_mut(w).enumerate().for_each(|(j, row)| {
        for (i, o) in row.iter_mut().enumerate() {
            let mut s = 0.0;
            for d in 0..=2 * r {
                let jj = (j + d).saturating_sub(r).min(h - 1);
                s += tmp[jj * w + i];
            }
            *o = s / k;
        }
    });
}

/// Everything [`apply`] needs besides the window's arrays.
#[derive(Clone, Copy)]
pub struct CharacterCtx<'a> {
    pub p: &'a CharacterParams,
    pub ruggedness: f64,
    pub fields: &'a CharacterFields,
    pub seed: u64,
    /// C, blocks.
    pub circumference: u32,
    /// Blocks per cell of the level, and the block of global cell (0, 0).
    pub cell_blocks: f64,
    pub origin_x: f64,
    pub origin_z: f64,
    /// Sea level, m; its block y; metres per block vertically.
    pub sea: f32,
    pub sea_y: f64,
    pub mv: f64,
}

/// Radii, cells, of the character's steps.
#[derive(Clone, Copy, Debug)]
struct Radii {
    /// Box blur for the slope direction (two passes).
    smooth: usize,
    /// Local relief (a min/max filter).
    relief: usize,
    /// Box blur of the unsharp mask (three passes).
    sharpen: usize,
    /// Gully octaves kept.
    octaves: u32,
}

fn radii(p: &CharacterParams, cell_blocks: f64) -> Radii {
    let cells = |b: f64| (b / cell_blocks).round().max(1.0) as usize;
    let mut octaves = 0;
    let mut lam = p.gully_wavelength_blocks;
    while octaves < p.gully_octaves && lam >= 4.0 * cell_blocks {
        octaves += 1;
        lam *= 0.5;
    }
    Radii {
        smooth: cells(3.0),
        relief: cells(p.relief_radius_blocks),
        sharpen: cells(p.sharpen_radius_blocks / 2.0),
        octaves,
    }
}

/// The dependency radius, cells, of [`apply`]: the base smoothing, the slope smoothing, the
/// relief window, one cell per gully octave (each reads the slope the previous ones left) and
/// the unsharp blur, whichever chain is longest.
pub fn radius_cells(p: &CharacterParams, cell_blocks: f64) -> usize {
    let r = radii(p, cell_blocks);
    let chain = 1 + 2 * r.smooth + r.relief + (r.relief / 2 + 1) + r.octaves as usize + 1;
    chain.max(1 + 3 * r.sharpen) + 1
}

/// What [`apply`] did over a window's core.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CharacterStats {
    /// Land cells changed and Σ|change|, blocks · cells; the largest raise and cut, blocks.
    pub land_cells: u64,
    pub abs_change_blocks_cells: f64,
    pub max_raise_blocks: f32,
    pub max_cut_blocks: f32,
}

/// The scratch arrays [`apply`] works in (window-sized each).
pub struct Scratch<'a> {
    pub base: &'a mut [f32],
    pub smooth: &'a mut [f32],
    pub relief: &'a mut [f32],
    pub blur: &'a mut [f32],
    pub acc: &'a mut [f32],
    pub tmp: &'a mut [f32],
    pub tmp2: &'a mut [f32],
}

#[inline]
fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// splitmix64's finaliser.
#[inline]
fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

#[inline]
fn hash(seed: u64, salt: u32, i: i64, j: i64) -> u64 {
    let a = mix(seed ^ u64::from(salt).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let b = mix(a ^ (i as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93));
    mix(b ^ (j as u64).wrapping_mul(0xA076_1D64_78BD_642F))
}

/// 21 bits of `h` from bit `shift` as a fraction in [0, 1).
#[inline]
fn frac21(h: u64, shift: u32) -> f64 {
    ((h >> shift) & 0x1F_FFFF) as f64 * (1.0 / 2_097_152.0)
}

/// A periodic wave of `phase` (period 1) in [−1, 1], crest at phase ½: a triangle at `sharp` 1,
/// a sinusoid-like cubic at 0.
#[inline]
fn wave(phase: f64, sharp: f64) -> f64 {
    let f = phase - phase.floor();
    let tri = 1.0 - 4.0 * (f - 0.5).abs();
    let smooth = tri * (1.5 - 0.5 * tri * tri);
    smooth + sharp * (tri - smooth)
}

/// Kernel radius of the stripe families, lattice spacings.
const STRIPE_R2: f64 = 1.2 * 1.2;

/// Stripe noise at block (bx, bz): a blend of the stripe families of the 3 × 3 jittered lattice
/// points around it (spacing ≈ `lam`, periodic in x with period `c`), each family `wave`s varying
/// along `dir` (a unit vector) so that its crests run across it, with a random phase per point.
/// In [−1, 1].
#[allow(clippy::too_many_arguments)]
#[inline]
fn stripes(
    seed: u64,
    salt: u32,
    c: f64,
    bx: f64,
    bz: f64,
    dir: (f64, f64),
    lam: f64,
    sharp: f64,
) -> f64 {
    let n = (c / lam).round().max(1.0);
    let s = c / n;
    let u = (bx + c / 2.0) / s;
    let v = bz / s;
    let (fu, fv) = (u.floor(), v.floor());
    let ni = n as i64;
    let k = s / lam;
    let (mut sum, mut wsum) = (0.0, 0.0);
    for dj in -1i64..=1 {
        let lj = fv as i64 + dj;
        for di in -1i64..=1 {
            let li = fu as i64 + di;
            let h = hash(seed, salt, li.rem_euclid(ni), lj);
            let qx = li as f64 + 0.25 + 0.5 * frac21(h, 0);
            let qz = lj as f64 + 0.25 + 0.5 * frac21(h, 21);
            let (dx, dz) = (u - qx, v - qz);
            let d2 = dx * dx + dz * dz;
            if d2 >= STRIPE_R2 {
                continue;
            }
            let t = 1.0 - d2 / STRIPE_R2;
            let w = t * t * t;
            let phase = (dx * dir.0 + dz * dir.1) * k + frac21(h, 42);
            sum += w * wave(phase, sharp);
            wsum += w;
        }
    }
    if wsum > 0.0 { sum / wsum } else { 0.0 }
}

/// Value noise in [−1, 1] of wavelength `lam` (periodic in x), for slowly varying offsets.
#[inline]
fn value(seed: u64, salt: u32, c: f64, bx: f64, bz: f64, lam: f64) -> f64 {
    let n = (c / lam).round().max(1.0);
    let s = c / n;
    let u = (bx + c / 2.0) / s;
    let v = bz / s;
    let (fu, fv) = (u.floor(), v.floor());
    let ni = n as i64;
    let (fx, fz) = (u - fu, v - fv);
    let (i0, j0) = ((fu as i64).rem_euclid(ni), fv as i64);
    let i1 = (i0 + 1) % ni;
    let l = |i: i64, j: i64| frac21(hash(seed, salt, i, j), 0) * 2.0 - 1.0;
    let tx = fx * fx * (3.0 - 2.0 * fx);
    let tz = fz * fz * (3.0 - 2.0 * fz);
    let a = l(i0, j0) + (l(i1, j0) - l(i0, j0)) * tx;
    let b = l(i0, j0 + 1) + (l(i1, j0 + 1) - l(i0, j0 + 1)) * tx;
    a + (b - a) * tz
}

/// Hardness in [0, 1] from the rock's erodibility multiplier: 1 at granite's (1) and below, 0 at
/// mudstone's (3) and above.
#[inline]
fn hardness(kfac: f32) -> f64 {
    let k = f64::from(kfac).max(1e-3);
    ((3.0f64).ln() - k.ln()) / (3.0f64).ln()
}

/// Index of window cell (i + di, j + dj) under the window rules (columns wrap in a full window,
/// else are replicated; rows are replicated at the window's edge).
#[inline]
fn at(win: &Win, i: usize, j: usize, di: i64, dj: i64) -> usize {
    let jj = (j as i64 + dj).clamp(0, win.h as i64 - 1) as usize;
    let ii = if win.wraps() {
        (i as i64 + di).rem_euclid(win.w as i64) as usize
    } else {
        (i as i64 + di).clamp(0, win.w as i64 - 1) as usize
    };
    jj * win.w + ii
}

/// One box-blur pass of radius `r` (cells) from `src` into `dst` through `tmp`, separably.
fn box_blur(win: &Win, src: &[f32], tmp: &mut [f32], dst: &mut [f32], r: usize) {
    let k = 1.0 / (2 * r + 1) as f32;
    let r = r as i64;
    map_cells(win, tmp, |i, j, o| {
        let mut s = 0.0f32;
        for d in -r..=r {
            s += src[at(win, i, j, d, 0)];
        }
        *o = s * k;
    });
    map_cells(win, dst, |i, j, o| {
        let mut s = 0.0f32;
        for d in -r..=r {
            s += tmp[at(win, i, j, 0, d)];
        }
        *o = s * k;
    });
}

/// The minimum of `src` over the (2r + 1)² cells around each cell, separably, into `dst`.
fn min_filter(win: &Win, src: &[f32], tmp: &mut [f32], dst: &mut [f32], r: usize) {
    let r = r as i64;
    map_cells(win, tmp, |i, j, o| {
        let mut m = f32::INFINITY;
        for d in -r..=r {
            m = m.min(src[at(win, i, j, d, 0)]);
        }
        *o = m;
    });
    map_cells(win, dst, |i, j, o| {
        let mut m = f32::INFINITY;
        for d in -r..=r {
            m = m.min(tmp[at(win, i, j, 0, d)]);
        }
        *o = m;
    });
}

/// max − min of `src` over the (2r + 1)² cells around each cell, separably, into `dst`.
fn relief(win: &Win, src: &[f32], lo: &mut [f32], hi: &mut [f32], dst: &mut [f32], r: usize) {
    let r = r as i64;
    map_cells(win, lo, |i, j, o| {
        let mut m = f32::INFINITY;
        for d in -r..=r {
            m = m.min(src[at(win, i, j, d, 0)]);
        }
        *o = m;
    });
    map_cells(win, hi, |i, j, o| {
        let mut m = f32::NEG_INFINITY;
        for d in -r..=r {
            m = m.max(src[at(win, i, j, d, 0)]);
        }
        *o = m;
    });
    map_cells(win, dst, |i, j, o| {
        let (mut a, mut b) = (f32::INFINITY, f32::NEG_INFINITY);
        for d in -r..=r {
            let c = at(win, i, j, 0, d);
            a = a.min(lo[c]);
            b = b.max(hi[c]);
        }
        *o = b - a;
    });
}

/// Applies the character to the window's terrain `z` (m) in place, reading the effective drainage
/// area `area` (m²), the erodibility `kfac` and the lake footprint. Sea and lake cells are left as
/// they are; no land cell goes under the sea. Returns the statistics of `core`'s land cells.
#[allow(clippy::too_many_arguments)]
pub fn apply(
    win: &Win,
    z: &mut [f32],
    area: &[f32],
    kfac: &[f32],
    lake: &[u8],
    s: Scratch<'_>,
    ctx: &CharacterCtx<'_>,
    core: (usize, usize, usize, usize),
) -> CharacterStats {
    let p = ctx.p;
    let rug = ctx.ruggedness;
    if rug <= 0.0 {
        return CharacterStats::default();
    }
    let rd = radii(p, ctx.cell_blocks);
    let cb = ctx.cell_blocks;
    let c = f64::from(ctx.circumference);
    let (sea, sea_y, mv) = (ctx.sea, ctx.sea_y, ctx.mv);
    let to_y = |zm: f32| (sea_y + (f64::from(zm) - f64::from(sea)) / mv) as f32;
    let [a0, a1] = p.channel_area_km2;
    let [f0, f1] = p.fold_channel_area_km2;

    // Block heights, then a 3 × 3 binomial smoothing: the amplifier's one-cell rills are finer
    // than the game's four-block corners can draw (they alias into stripes).
    map_cells(win, s.tmp, |i, j, o| *o = to_y(z[j * win.w + i]));
    map_cells(win, s.base, |i, j, o| {
        let g = |di, dj| s.tmp[at(win, i, j, di, dj)];
        *o = (4.0 * g(0, 0) + 2.0 * (g(1, 0) + g(-1, 0) + g(0, 1) + g(0, -1))
            + g(1, 1)
            + g(1, -1)
            + g(-1, 1)
            + g(-1, -1))
            / 16.0;
    });
    // The slope's direction comes from a smoother surface.
    box_blur(win, s.base, s.tmp, s.acc, rd.smooth);
    box_blur(win, s.acc, s.tmp, s.smooth, rd.smooth);
    // Local relief, softened (a square min/max window has square edges).
    relief(win, s.smooth, s.tmp, s.tmp2, s.blur, rd.relief);
    box_blur(win, s.blur, s.tmp, s.relief, rd.relief / 2 + 1);
    // The unsharp mask's blur.
    box_blur(win, s.base, s.tmp, s.acc, rd.sharpen);
    box_blur(win, s.acc, s.tmp, s.tmp2, rd.sharpen);
    box_blur(win, s.tmp2, s.tmp, s.blur, rd.sharpen);
    // The local base level: the lowest smoothed height within the relief radius. A gully cannot
    // cut below it (a trough that outran its base level would end in a pit at the foot of the
    // slope, which the flood would fill flat).
    min_filter(win, s.smooth, s.tmp, s.tmp2, rd.relief);

    let global_xz = |i: usize, j: usize| {
        let (gi, gj) = win.global(i, j);
        (
            ctx.origin_x + gi as f64 * cb,
            ctx.origin_z + gj as f64 * cb,
            gi,
            gj,
        )
    };
    let is_land = |cell: usize| z[cell] > sea && lake[cell] == 0;
    // How much of the character a land cell takes: none on rivers and at the coast.
    let fade = |cell: usize| -> f64 {
        let a_km2 = f64::from(area[cell]) * 1.0e-6;
        let river = smoothstep(a0, a1, a_km2);
        let coast = smoothstep(0.0, p.coast_blocks.max(1e-6), f64::from(s.smooth[cell]) - sea_y);
        (1.0 - river) * coast
    };

    // Fold ridges.
    let (base, smooth, rel, low) = (&*s.base, &*s.smooth, &*s.relief, &*s.tmp2);
    // How deep a trough may go at a cell: most of its height above the local base level.
    let headroom = |cell: usize| 0.8 * f64::from(smooth[cell] - low[cell]).max(0.0);
    map_cells(win, s.acc, |i, j, o| {
        let cell = j * win.w + i;
        *o = 0.0;
        if !is_land(cell) {
            return;
        }
        let (bx, bz, gi, gj) = global_xz(i, j);
        let (_, fold, dir) = ctx.fields.at(gi, gj);
        if fold <= 0.0 {
            return;
        }
        let wob = 0.35 * value(ctx.seed, 207, c, bx, bz, 3.0 * p.fold_wavelength_blocks);
        let b = stripes(
            ctx.seed,
            211,
            c,
            bx,
            bz,
            dir,
            p.fold_wavelength_blocks,
            0.0,
        );
        // Ridged: sharp crests at the stripes' zero lines, rounded troughs; plunging along strike.
        let ridged = 0.5 - (b + wob).abs().min(1.0);
        let plunge = 0.6 + 0.4 * value(ctx.seed, 213, c, bx, bz, 1.5 * p.fold_wavelength_blocks);
        let gap = 1.0 - smoothstep(f0, f1, f64::from(area[cell]) * 1.0e-6);
        let amp = (rug * p.fold_gain * fold * plunge * f64::from(rel[cell])).min(2.0 * headroom(cell));
        *o = (amp * ridged * fade(cell) * gap) as f32;
    });

    // Gully octaves, each aligned with the slope of the smoothed terrain plus everything so far.
    let mut lam = p.gully_wavelength_blocks;
    let mut persist = 1.0;
    for k in 0..rd.octaves {
        let acc = &*s.acc;
        map_cells(win, s.tmp, |i, j, o| {
            let cell = j * win.w + i;
            *o = 0.0;
            if !is_land(cell) {
                return;
            }
            let h = |di, dj| {
                let n = at(win, i, j, di, dj);
                f64::from(smooth[n]) + f64::from(acc[n])
            };
            let gx = (h(1, 0) - h(-1, 0)) / (2.0 * cb);
            let gz = (h(0, 1) - h(0, -1)) / (2.0 * cb);
            let g = (gx * gx + gz * gz).sqrt();
            if g < 1e-4 {
                return;
            }
            let dir = (-gz / g, gx / g);
            // The depth follows the slope of the smoothed terrain (so the octaves do not feed on
            // each other's walls); the direction, everything so far.
            let m = |di, dj| f64::from(smooth[at(win, i, j, di, dj)]);
            let mx = (m(1, 0) - m(-1, 0)) / (2.0 * cb);
            let mz = (m(0, 1) - m(0, -1)) / (2.0 * cb);
            let slope = (mx * mx + mz * mz).sqrt();
            let (bx, bz, gi, gj) = global_xz(i, j);
            let (youth, _, _) = ctx.fields.at(gi, gj);
            let hard = hardness(kfac[cell]).clamp(0.0, 1.0);
            // Finer octaves are rounder: sharp crests are the coarse ridgelines'.
            let sharp = (0.2 + 0.45 * youth + 0.45 * hard).clamp(0.0, 1.0) * (1.0 - 0.2 * f64::from(k));
            let shape = (0.45 + 0.55 * youth) * (0.55 + 0.45 * hard);
            let amp = (rug * p.gully_gain * shape * slope * lam * persist)
                .min(p.gully_relief_cap * f64::from(rel[cell]))
                .min(rug * p.gully_max_ratio * lam * (0.6 + 0.4 * hard) * persist)
                .min(headroom(cell) * persist);
            let b = stripes(ctx.seed, 300 + k, c, bx, bz, dir, lam, sharp);
            *o = (amp * b * fade(cell)) as f32;
        });
        s.acc.iter_mut().zip(s.tmp.iter()).for_each(|(a, &t)| *a += t);
        lam *= 0.5;
        persist *= p.gully_persistence;
    }

    // Young peaks (unsharp mask), cliff bands on hard rock, and the result.
    let (acc, blur) = (&*s.acc, &*s.blur);
    let tmp = &mut *s.tmp;
    map_cells(win, tmp, |i, j, o| {
        let cell = j * win.w + i;
        *o = z[cell];
        if !is_land(cell) {
            return;
        }
        let (bx, bz, gi, gj) = global_xz(i, j);
        let (youth, _, _) = ctx.fields.at(gi, gj);
        let f = fade(cell);
        // On steep ground the smoothed surface is the base: the amplifier's grid-bound rills
        // (axis and diagonal gullies a few cells wide) give way to the drainage-aligned ones.
        let (mx, mz) = {
            let h = |di, dj| f64::from(smooth[at(win, i, j, di, dj)]);
            ((h(1, 0) - h(-1, 0)) / (2.0 * cb), (h(0, 1) - h(0, -1)) / (2.0 * cb))
        };
        let steep = smoothstep(0.3, 1.2, (mx * mx + mz * mz).sqrt()) * rug.min(1.0);
        let y0 = f64::from(base[cell]) + steep * f64::from(smooth[cell] - base[cell]);
        // Young: summits and ridgelines stand out (the convex part only: sharpening the concave
        // part would dig moats at the foot of every steep front, which the flood then fills flat).
        // Old: the terrain is pulled towards its smoothed self, both ways (rounded, no moats).
        let excess = y0 - f64::from(blur[cell]);
        let unsharp = rug * p.sharpen_gain * youth * excess.max(0.0)
            - (rug * p.round_gain * (1.0 - youth)).min(1.0) * excess;
        let mut y = y0 + unsharp * f + f64::from(acc[cell]);
        // Cliff bands: a smooth terrace of the height where hard rock is steep.
        let h = |di, dj| f64::from(smooth[at(win, i, j, di, dj)]);
        let gx = (h(1, 0) - h(-1, 0)) / (2.0 * cb);
        let gz = (h(0, 1) - h(0, -1)) / (2.0 * cb);
        let slope = (gx * gx + gz * gz).sqrt();
        let hard = hardness(kfac[cell]).clamp(0.0, 1.0);
        let wt = (rug * p.terrace_gain * smoothstep(0.45, 0.9, hard) * smoothstep(0.6, 2.0, slope))
            .min(1.0)
            * f;
        if wt > 0.0 {
            let step = p.terrace_step_blocks
                * (1.0 + 0.3 * value(ctx.seed, 221, c, bx, bz, 96.0));
            let off = step * value(ctx.seed, 223, c, bx, bz, 160.0);
            let q = (y + off) / step;
            let qi = q.floor();
            let fr = q - qi;
            let f3 = fr * fr * fr;
            let g3 = (1.0 - fr) * (1.0 - fr) * (1.0 - fr);
            let terr = (qi + f3 / (f3 + g3)) * step - off;
            y += wt * (terr - y);
        }
        let y_in = f64::from(to_y(z[cell]));
        let dy = (y - y_in).clamp(-p.max_change_blocks, p.max_change_blocks);
        let zm = f64::from(sea) + (y_in + dy - sea_y) * mv;
        *o = (zm as f32).max(sea + 0.05);
    });

    let mut st = CharacterStats::default();
    let (x0, y0, cw, ch) = core;
    for j in y0..y0 + ch {
        for i in x0..x0 + cw {
            let cell = j * win.w + i;
            if !is_land(cell) {
                continue;
            }
            let d = (f64::from(tmp[cell]) - f64::from(z[cell])) / mv;
            st.land_cells += 1;
            st.abs_change_blocks_cells += d.abs();
            st.max_raise_blocks = st.max_raise_blocks.max(d as f32);
            st.max_cut_blocks = st.max_cut_blocks.max(-d as f32);
        }
    }
    z.copy_from_slice(tmp);
    st
}

/// Pit depths (blocks) from which the character is softened, and to which it is gone.
pub const PIT_SOFTEN_BLOCKS: [f64; 2] = [1.5, 4.0];

/// Radius, cells, the pit depth is spread over before it softens the character (so a pit's dam,
/// a crest a cell or two wide beside it, softens too).
pub const PIT_SOFTEN_RADIUS: usize = 6;

/// Softens the character around the pits it closed: with `filled` the flooded surface of `z` (m),
/// the depth of every pit (filled − z) is spread over [`PIT_SOFTEN_RADIUS`] cells and, where it
/// passes [`PIT_SOFTEN_BLOCKS`], `z` goes back towards `z_pre` (the terrain before the character).
/// Returns how many cells of `core` it softened.
#[allow(clippy::too_many_arguments)]
pub fn soften_pits(
    win: &Win,
    z: &mut [f32],
    z_pre: &[f32],
    filled: &[f32],
    tmp: &mut [f32],
    depth: &mut [f32],
    ctx: &CharacterCtx<'_>,
    core: (usize, usize, usize, usize),
) -> u64 {
    let mv = ctx.mv as f32;
    map_cells(win, depth, |i, j, o| {
        let c = j * win.w + i;
        *o = (filled[c] - z[c]).max(0.0) / mv;
    });
    // A max filter then a box blur: the depth reaches the dam, and fades smoothly.
    let r = PIT_SOFTEN_RADIUS as i64;
    map_cells(win, tmp, |i, j, o| {
        let mut m = 0f32;
        for d in -r..=r {
            m = m.max(depth[at(win, i, j, d, 0)]);
        }
        *o = m;
    });
    map_cells(win, depth, |i, j, o| {
        let mut m = 0f32;
        for d in -r..=r {
            m = m.max(tmp[at(win, i, j, 0, d)]);
        }
        *o = m;
    });
    let spread = depth.to_vec();
    box_blur(win, &spread, tmp, depth, PIT_SOFTEN_RADIUS);
    let [d0, d1] = PIT_SOFTEN_BLOCKS;
    let mut n = 0u64;
    let (x0, y0, cw, ch) = core;
    for (c, zc) in z.iter_mut().enumerate() {
        let w = smoothstep(d0, d1, f64::from(depth[c])) as f32;
        if w > 0.0 {
            *zc += (z_pre[c] - *zc) * w;
            let (i, j) = (c % win.w, c / win.w);
            n += u64::from((x0..x0 + cw).contains(&i) && (y0..y0 + ch).contains(&j));
        }
    }
    n
}

/// The halo the character needs on top of the passes': its own radius, the flood that measures
/// the pits it closed, their softening, and the last flood.
pub fn halo_cells(p: &CharacterParams, cell_blocks: f64, repair_margin: usize) -> usize {
    radius_cells(p, cell_blocks) + 2 * repair_margin + 2 * PIT_SOFTEN_RADIUS + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    impl CharacterFields {
        /// Uniform fields (tests): youth, fold strength and shortening direction everywhere.
        fn uniform(youth: f32, fold: f32, dir_deg: f64) -> Self {
            let th = 2.0 * dir_deg.to_radians();
            CharacterFields {
                w: 4,
                h: 4,
                ratio: 8.0,
                youth: vec![youth; 16],
                fold_c2: vec![fold * th.cos() as f32; 16],
                fold_s2: vec![fold * th.sin() as f32; 16],
                neutral: false,
            }
        }
    }

    const MV: f64 = 20.0;
    const SEA_Y: f64 = 63.0;

    /// A 400-block cone (a young volcano's size here) on a 320 × 160-cell window of 2-block
    /// cells; land everywhere, 1 km² of drainage area (no rivers), erodibility `kfac`.
    fn cone(kfac: f32) -> (Win, Vec<f32>, Vec<f32>, Vec<f32>, Vec<u8>) {
        let win = Win::full(320, 160);
        let mut z = vec![0f32; win.cells()];
        for j in 0..win.h {
            for i in 0..win.w {
                let (dx, dz) = (i as f64 - 160.0, j as f64 - 80.0);
                let r = (dx * dx + dz * dz).sqrt() * 2.0;
                let y = SEA_Y + 4.0 + (300.0 - r).max(0.0);
                z[j * win.w + i] = ((y - SEA_Y) * MV) as f32;
            }
        }
        let n = win.cells();
        (win, z, vec![1.0e6; n], vec![kfac; n], vec![0; n])
    }

    /// RMS change, blocks, of the character over the cone's flanks.
    fn change(fields: &CharacterFields, kfac: f32, rug: f64) -> (f64, Vec<f32>) {
        let (win, mut z, area, k, lake) = cone(kfac);
        let before = z.clone();
        let n = win.cells();
        let mut sc: Vec<Vec<f32>> = (0..7).map(|_| vec![0f32; n]).collect();
        let [a, b, c, d, e, f, g] = &mut sc[..] else { unreachable!() };
        let p = CharacterParams::default();
        let ctx = CharacterCtx {
            p: &p,
            ruggedness: rug,
            fields,
            seed: 11,
            circumference: 640,
            cell_blocks: 2.0,
            origin_x: -320.0,
            origin_z: -160.0,
            sea: 0.0,
            sea_y: SEA_Y,
            mv: MV,
        };
        let scratch = Scratch { base: a, smooth: b, relief: c, blur: d, acc: e, tmp: f, tmp2: g };
        apply(&win, &mut z, &area, &k, &lake, scratch, &ctx, (0, 0, win.w, win.h));
        let mut sum = 0.0;
        let mut cnt = 0.0;
        for j in 30..130 {
            for i in 60..260 {
                let c = j * win.w + i;
                let d = f64::from(z[c] - before[c]) / MV;
                sum += d * d;
                cnt += 1.0;
            }
        }
        ((sum / cnt).sqrt(), z)
    }

    #[test]
    fn young_hard_rock_gets_more_character_than_old_soft_rock() {
        let young = CharacterFields::uniform(1.0, 0.0, 0.0);
        let old = CharacterFields::uniform(0.0, 0.0, 0.0);
        let (y_hard, _) = change(&young, 0.8, 1.0);
        let (y_soft, _) = change(&young, 3.0, 1.0);
        let (o_hard, _) = change(&old, 0.8, 1.0);
        assert!(y_hard > 3.0, "a young cone in hard rock changes by {y_hard} blocks RMS");
        assert!(y_hard > 1.3 * y_soft, "hard {y_hard} vs soft {y_soft}");
        assert!(y_hard > 1.3 * o_hard, "young {y_hard} vs old {o_hard}");
        // Ruggedness 0 leaves the terrain alone, and more ruggedness is more character.
        let (none, _) = change(&young, 0.8, 0.0);
        assert_eq!(none, 0.0);
        let (more, _) = change(&young, 0.8, 2.0);
        assert!(more > y_hard, "ruggedness 2: {more} vs 1: {y_hard}");
    }

    #[test]
    fn the_sea_is_left_alone_and_no_land_goes_under_it() {
        let young = CharacterFields::uniform(1.0, 1.0, 30.0);
        let (win, before, ..) = cone(0.8);
        let (_, z) = change(&young, 0.8, 2.0);
        for c in 0..win.cells() {
            if before[c] <= 0.0 {
                assert_eq!(z[c], before[c]);
            } else {
                assert!(z[c] > 0.0, "cell {c} went under the sea");
            }
        }
    }

    #[test]
    fn gullies_run_down_the_slope() {
        // Around the cone, the character varies far more across the fall line (around the cone)
        // than along it (towards the summit): its crests and valleys radiate from the peak.
        let young = CharacterFields::uniform(1.0, 0.0, 0.0);
        let (win, before, ..) = cone(0.8);
        let (_, z) = change(&young, 0.8, 1.0);
        let d = |i: f64, j: f64| {
            let c = j.round() as usize * win.w + i.round() as usize;
            f64::from(z[c] - before[c]) / MV
        };
        let (mut radial, mut around) = (0.0, 0.0);
        for k in 0..720 {
            let a = k as f64 * std::f64::consts::TAU / 720.0;
            let (ca, sa) = (a.cos(), a.sin());
            for r in [40.0, 55.0, 70.0] {
                let (i, j) = (160.0 + r * ca, 80.0 + r * sa);
                // Two cells towards the summit, and two cells around the cone.
                radial += (d(i, j) - d(i - 2.0 * ca, j - 2.0 * sa)).abs();
                around += (d(i, j) - d(i - 2.0 * sa, j + 2.0 * ca)).abs();
            }
        }
        assert!(around > 1.5 * radial, "around {around} radial {radial}");
    }

    #[test]
    fn stripes_are_periodic_bounded_and_run_across_their_direction() {
        let c = 4096.0;
        for k in 0..200 {
            let bx = -c / 2.0 + (k as f64 * 37.3) % c;
            let bz = -900.0 + k as f64 * 9.1;
            let d = (0.6, 0.8);
            let a = stripes(9, 1, c, bx, bz, d, 48.0, 0.5);
            assert!((-1.0..=1.0).contains(&a));
            let lap = stripes(9, 1, c, bx + c, bz, d, 48.0, 0.5);
            assert!((a - lap).abs() < 1e-9, "periodic in x: {a} {lap}");
        }
        // Along the crests (perpendicular to `dir`) the value changes far less than across them.
        let (mut along, mut across) = (0.0, 0.0);
        for k in 0..400 {
            let bx = k as f64 * 13.7;
            let bz = k as f64 * 7.3;
            let d = (1.0, 0.0);
            let v = stripes(3, 2, c, bx, bz, d, 48.0, 0.0);
            along += (stripes(3, 2, c, bx, bz + 3.0, d, 48.0, 0.0) - v).abs();
            across += (stripes(3, 2, c, bx + 3.0, bz, d, 48.0, 0.0) - v).abs();
        }
        assert!(across > 2.0 * along, "across {across} along {along}");
    }

    #[test]
    fn wave_and_hardness_hold_their_ends() {
        assert_eq!(wave(0.5, 0.0), 1.0);
        assert_eq!(wave(0.0, 1.0), -1.0);
        assert!((wave(0.25, 1.0)).abs() < 1e-12);
        assert!((hardness(1.0) - 1.0).abs() < 1e-12);
        assert!(hardness(3.0).abs() < 1e-12);
        assert!(hardness(1.5) > 0.5 && hardness(1.5) < 0.75);
    }

    #[test]
    fn defaults_validate_and_the_radius_covers_the_relief() {
        let p = CharacterParams::default();
        p.validate("$.amplify.character").unwrap();
        let r = radius_cells(&p, 2.0);
        assert!(r > (p.relief_radius_blocks / 2.0) as usize, "{r}");
        // Octaves under four cells are dropped.
        assert_eq!(radii(&p, 2.0).octaves, 4);
        assert_eq!(radii(&p, 8.0).octaves, 3);
    }
}
