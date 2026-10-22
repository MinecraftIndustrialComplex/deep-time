// SPDX-License-Identifier: LGPL-3.0-or-later
//! The block-scale relief the projection pass draws on the world's own grid
//! ([`crate::bake::project::project_folder`]), after the resampling and before anything reads the
//! world's terrain: in world blocks, from the drawn terrain, so it does not depend on the frame
//! the simulation ran in (the world's layout reaches it only through [`Frame`]).
//!
//! - **Cone gullies** (Ben, decisions log 2026-10-22, D2: "cones get a smooth fresh summit with
//!   radially gullied older flanks", O'Hara et al. 2024). A standing stratovolcano
//!   (`volcanoes/standing.json`, written by the bake) gets radial V-shaped gullies cut into its
//!   flanks, none in its summit zone (the upper 30 % of its height over its foot, O'Hara's summit
//!   region) or its crater. Their depth follows O'Hara's eroded volume against the cone's age (the
//!   log-mean of the times since its first and its last activity): each cone's cuts take that share
//!   of its volume over its foot. Wider and fewer gullies as it ages, and a dead cone's fade on the
//!   crater's clock as the mountain character takes it over (VOL-068). No cut where a river or a
//!   lake is, under ice, or below one block over the sea.
//!
//! Everything goes into the `elevation` layer (H = E + A·N unchanged; the game, the far terrain
//! and the column evaluator read it as terrain), so the world-data format gains only the file the
//! cone gullies read, `volcanoes/standing.json` (FORMAT.md, "Standing volcanoes").

use std::f64::consts::TAU;
use std::path::Path;

use rayon::prelude::*;
use serde_json::{Value, json};

use deeptime_format::raster::{RasterRead, ZEdges};
use deeptime_format::rivers::Rivers;
use deeptime_format::{GridDesc, LayerDesc, LoadedLayer};

use crate::bake::project::Mapping;
use crate::error::CoreError;
use crate::surface::volcanoes::{Kind, VolcanoState};

/// The standing volcanoes the cone gullies read: every volcano still standing at the present, alive
/// or dead (FORMAT.md, "Standing volcanoes"). Written by the bake in the simulation's blocks, moved
/// to the world's by the projection pass like `volcanoes/active.json`.
pub const STANDING_PATH: &str = "volcanoes/standing.json";
pub const STANDING_VERSION: u32 = 1;

/// A volcano under this height is not drawn (the shaper's floor, VOL-064).
const MIN_HEIGHT_M: f64 = 30.0; // ledger: VOL-064

/// `volcanoes/standing.json`: the stratovolcanoes still standing at the present (alive or dead,
/// at least [`MIN_HEIGHT_M`] high), in id order, with the times the cone gullies' age needs.
pub fn standing_json(st: &VolcanoState) -> Value {
    let r3 = |v: f64| (v * 1000.0).round() / 1000.0;
    let mut list: Vec<_> = st
        .volcanoes
        .iter()
        .filter(|v| {
            v.kind == Kind::Strato
                && v.gone.is_none()
                && v.radius_m > 0.0
                && v.height_m >= MIN_HEIGHT_M
        })
        .collect();
    list.sort_by_key(|v| v.id);
    let volcanoes: Vec<Value> = list
        .iter()
        .map(|v| {
            json!({
                "id": v.id,
                "kind": v.kind.name(),
                "x": r3(v.x),
                "z": r3(v.z),
                "radius_m": v.radius_m.round(),
                "height_m": v.height_m.round(),
                "crater_rho": v.depression.map(|d| r3(d.rho)),
                "born_myr": r3(v.born_myr),
                "extinct_myr": v.extinct_myr.map(r3),
            })
        })
        .collect();
    json!({
        "version": STANDING_VERSION,
        "t_myr": st.t_myr,
        "fade_myr": st.fade_myr,
        "volcanoes": volcanoes,
    })
}

/// Reads `volcanoes/standing.json` of world folder `dir`, if it has one.
pub fn read_standing(dir: &Path) -> Result<Option<Value>, CoreError> {
    let path = dir.join(STANDING_PATH);
    match std::fs::read(&path) {
        Ok(b) => serde_json::from_slice(&b)
            .map(Some)
            .map_err(|e| CoreError::InvalidState(format!("{STANDING_PATH}: {e}"))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(CoreError::io(path, e)),
    }
}

/// The world's layout as the relief sees it: world points to the sphere and back, and the image
/// of a point nearest another through the world's edges. The projection pass gives its
/// [`crate::bake::project::Mapping`]; a cube's faces would give theirs.
pub trait Frame: Sync {
    /// The sphere point (unit vector) of world point (x, z).
    fn unit(&self, x: f64, z: f64) -> [f64; 3];
    /// The canonical world point of a sphere point.
    fn point(&self, v: [f64; 3]) -> (f64, f64);
    /// The image of world point `q` nearest to `p` through the edges' motions (a rigid motion, so
    /// a pattern of even angular harmonics round `q` reads the same through any of them).
    fn nearest_image(&self, p: (f64, f64), q: (f64, f64)) -> (f64, f64);
}

/// The world's ground the relief draws on: its grid, the sea, and where it must not cut.
pub struct Ground<'a> {
    pub grid: &'a GridDesc,
    pub sea_y: f64,
    /// A lake's water level is set at cell `c` (row-major).
    pub lake: &'a (dyn Fn(usize) -> bool + Sync),
    /// A river's channel or bank zone holds world point (x, z).
    pub river: &'a (dyn Fn(f64, f64) -> bool + Sync),
    /// The ice cover at world point (x, z), blocks.
    pub ice: &'a (dyn Fn(f64, f64) -> f64 + Sync),
}

/// The world's elevation layer's cells, read and written in blocks.
pub struct Elev<'a> {
    pub desc: &'a LayerDesc,
    pub bytes: &'a mut [u8],
}

impl Elev<'_> {
    #[inline]
    fn cb(&self) -> usize {
        self.desc.cell_bytes()
    }

    /// The terrain height of cell `c`, blocks.
    #[inline]
    pub fn get(&self, c: usize) -> f64 {
        let cb = self.cb();
        self.desc.physical(
            self.desc
                .dtype
                .decode_scalar(&self.bytes[c * cb..(c + 1) * cb]),
        )
    }

    /// Sets cell `c` to `y` blocks (rounded to the layer's step).
    pub fn set(&mut self, c: usize, y: f64) {
        let cb = self.cb();
        let raw = (y - self.desc.offset) / self.desc.scale;
        crate::bake::project::put_raw(self.desc.dtype, raw, &mut self.bytes[c * cb..(c + 1) * cb]);
    }
}

#[inline]
fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalized(a: [f64; 3]) -> [f64; 3] {
    let n = dot(a, a).sqrt().max(1e-300);
    [a[0] / n, a[1] / n, a[2] / n]
}

/// The angle between two unit vectors, radians (exact near 0 and π).
#[inline]
fn angle(a: [f64; 3], b: [f64; 3]) -> f64 {
    let c = cross(a, b);
    dot(c, c).sqrt().atan2(dot(a, b))
}

// --- cone gullies (D2) ---------------------------------------------------------------------------

/// The last eruption of a volcano active at the present, years ago: the floor of both ages in
/// O'Hara's log-mean (an active cone has erupted lately; the log-mean of a time and 0 would be 0).
pub const RECENT_FLOOR_YR: f64 = 1_000.0; // ledger: VOL-102 (game-choice)

/// The summit zone, as a share of the cone's height over its foot: no gullies above its top,
/// a soft edge down to its bottom (O'Hara's summit region is the upper 30 % of the height).
const SUMMIT_ZONE: (f64, f64) = (0.55, 0.70); // ledger: VOL-100
/// The cuts fade out between these radii (ρ), where the cone meets its ground.
const FOOT_FADE: (f64, f64) = (0.9, 1.15); // ledger: VOL-104 (game-choice)
/// The ring the cone's foot is read on (ρ), as the census reads the ground (VOL-082).
const FOOT_RING: (f64, f64) = (1.0, 1.3); // ledger: VOL-104 (game-choice)
/// Where the summit's height is read (ρ).
const TOP_RHO: f64 = 0.2; // ledger: VOL-104 (game-choice)
/// A cone standing less than this over its foot (blocks) gets no gullies.
const MIN_SPAN_BLOCKS: f64 = 4.0; // ledger: VOL-104 (game-choice)
/// The gullies come in as their spacing round the cone grows from the first to the second (world
/// blocks): the game draws the terrain from corners 4 blocks apart, so nothing finer than 8 blocks
/// shows.
const ONSET_SPACING: (f64, f64) = (8.0, 12.0); // ledger: VOL-103 (game-choice)
/// The deepest a gully's axis cuts, as a share of the cone's height over its foot there.
const MAX_CUT_SHARE: f64 = 0.8; // ledger: VOL-104 (game-choice)

/// O'Hara et al. 2024, Fig. 4h: the normalized eroded volume of a cone against its age (the
/// log-mean of its first and last activity), 5.36·log10(age, yr) − 1.56 %, kept within 5 to 40 %
/// (their cones reach about 10 % at a few kyr and 30 to 40 % at 10 to 100 kyr and beyond).
pub fn eroded_share(age_yr: f64) -> f64 {
    let pct = 5.36 * age_yr.max(1.0).log10() - 1.56; // ledger: VOL-101
    (pct / 100.0).clamp(0.05, 0.40) // ledger: VOL-101
}

/// O'Hara's age of a cone at `t_myr`: the log-mean (geometric mean) of the times since its first
/// activity and since its last, years, each at least [`RECENT_FLOOR_YR`].
pub fn log_mean_age_yr(t_myr: f64, born_myr: f64, extinct_myr: Option<f64>) -> f64 {
    let last = extinct_myr.map_or(RECENT_FLOOR_YR, |e| {
        ((t_myr - e) * 1e6).max(RECENT_FLOOR_YR)
    });
    let first = ((t_myr - born_myr) * 1e6).max(last);
    (first * last).sqrt()
}

/// The gullies' width angle seen from the summit, degrees, for eroded share `e`: about 20° on a
/// young cone (10 %), 40 to 60° at 30 to 40 % (O'Hara §4.1), 50° at 35 % and beyond.
pub fn width_deg(e: f64) -> f64 {
    20.0 + 30.0 * smooth(0.10, 0.35, e) // ledger: VOL-103
}

/// How many gullies a cone of width angle `deg` has: the whole round over the angle, in fours (a
/// pattern of even angular harmonics reads the same through the world's half-turns and a cube's
/// quarter-turns), 8 to 16 (O'Hara §4.3: more than 10 summit basins on most cones; the cone's
/// radius over the outlets' spacing 1 to 3, so 6 to 19 round its foot).
pub fn gully_count(deg: f64) -> u32 {
    let n = 4.0 * (360.0 / (4.0 * deg.max(1.0))).round();
    (n as u32).clamp(8, 16) // ledger: VOL-103
}

/// The shape of a gully across its basin (the basins tile the cone, divide to divide): depth
/// (1 − 2·d)^p at d periods off the axis, with p from 1.5 on a young cone (narrow V-cuts between
/// rounded interfluves) to 0.7 on an old one (wide valleys between sharp divides), for eroded
/// share `e`. Its mean across the basin is 1/(1 + p).
pub fn profile_power(e: f64) -> f64 {
    1.5 - 0.8 * smooth(0.10, 0.35, e) // ledger: VOL-103 (game-choice)
}

/// A standing stratovolcano as the cone gullies draw it.
#[derive(Clone, Debug, PartialEq)]
pub struct Cone {
    pub id: u32,
    /// Its vent on the sphere.
    pub unit: [f64; 3],
    pub radius_m: f64,
    /// Its crater's or caldera's rim, ρ (0: none).
    pub crater_rho: f64,
    /// O'Hara's age ([`log_mean_age_yr`]).
    pub age_yr: f64,
    /// 1 while active; exp(−dead/fade_myr) once dead (the crater's clock).
    pub young: f64,
}

/// The cones of a `volcanoes/standing.json`; `unit_of` maps its (x, z) to the sphere.
pub fn cones_of(v: &Value, unit_of: impl Fn(f64, f64) -> [f64; 3]) -> Vec<Cone> {
    let t = v["t_myr"].as_f64().unwrap_or(0.0);
    let fade = v["fade_myr"].as_f64().unwrap_or(5.0).max(1e-3);
    let Some(list) = v["volcanoes"].as_array() else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|o| {
            let (x, z) = (o["x"].as_f64()?, o["z"].as_f64()?);
            let radius_m = o["radius_m"].as_f64().filter(|r| *r > 0.0)?;
            let born = o["born_myr"].as_f64().unwrap_or(t);
            let extinct = o["extinct_myr"].as_f64();
            let dead = extinct.map_or(0.0, |e| (t - e).max(0.0));
            Some(Cone {
                id: o["id"].as_u64().unwrap_or(0) as u32,
                unit: unit_of(x, z),
                radius_m,
                crater_rho: o["crater_rho"].as_f64().unwrap_or(0.0).max(0.0),
                age_yr: log_mean_age_yr(t, born, extinct),
                young: (-dead / fade).exp(), // ledger: VOL-105
            })
        })
        .collect()
}

/// A cone's phases from its id (the golden ratio's fractions, as the caldera rims take theirs).
fn phases(id: u32) -> [f64; 4] {
    let g = f64::from(id) * 0.618_033_988_7;
    [
        g.fract(),
        (1.7 * g + 0.31).fract() * TAU,
        (2.3 * g + 0.57).fract() * TAU,
        (3.1 * g + 0.89).fract() * TAU,
    ]
}

/// A cell of a cone: where it is round the vent and how high it stands.
struct At {
    c: u32,
    x: f64,
    z: f64,
    rho: f64,
    theta: f64,
    /// Distance from the vent, world blocks.
    dw: f64,
    e: f64,
}

/// What one cone's gullies did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConeRecord {
    pub id: u32,
    pub gullies: u32,
    /// [`profile_power`].
    pub power: f64,
    /// O'Hara's eroded share for its age, and the share of its volume over its foot the cuts took.
    pub target: f64,
    pub removed: f64,
    /// The cuts' strength reached [`MAX_CUT_SHARE`] short of the target.
    pub capped: bool,
    pub span_blocks: f64,
    pub deepest_blocks: f64,
    pub skipped: Option<&'static str>,
}

/// One cone's cuts (cell, blocks) and its calm (cell, 0..1: how much of the steep-ground texture
/// to hold back over its body), read from the terrain `e`.
fn one_cone(
    e: &Elev,
    g: &Ground,
    frame: &dyn Frame,
    planet_r_m: f64,
    k: &Cone,
) -> (Vec<(u32, f32)>, Vec<(u32, f32)>, ConeRecord) {
    let grid = g.grid;
    let mut rec = ConeRecord {
        id: k.id,
        ..ConeRecord::default()
    };
    let vw = frame.point(k.unit);
    // The cone's box in the world: its outer ring through the frame.
    let alpha = FOOT_RING.1 * k.radius_m / planet_r_m;
    let helper = if k.unit[2].abs() < 0.9 {
        [0.0, 0.0, 1.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let e1 = normalized(cross(k.unit, helper));
    let e2 = cross(k.unit, e1);
    let (ca, sa) = (alpha.cos(), alpha.sin());
    let (mut x0, mut x1, mut z0, mut z1) = (vw.0, vw.0, vw.1, vw.1);
    for s in 0..64 {
        let t = TAU * f64::from(s) / 64.0;
        let (ct, st) = (t.cos(), t.sin());
        let p = [
            ca * k.unit[0] + sa * (ct * e1[0] + st * e2[0]),
            ca * k.unit[1] + sa * (ct * e1[1] + st * e2[1]),
            ca * k.unit[2] + sa * (ct * e1[2] + st * e2[2]),
        ];
        let w = frame.nearest_image(vw, frame.point(p));
        x0 = x0.min(w.0);
        x1 = x1.max(w.0);
        z0 = z0.min(w.1);
        z1 = z1.max(w.1);
    }
    let r = f64::from(grid.r);
    let (ox, oz) = (grid.origin_block_x as f64, grid.origin_block_z as f64);
    let (i0, i1) = (
        ((x0 - ox) / r).floor() as i64 - 1,
        ((x1 - ox) / r).ceil() as i64 + 1,
    );
    let (j0, j1) = (
        ((z0 - oz) / r).floor() as i64 - 1,
        ((z1 - oz) / r).ceil() as i64 + 1,
    );
    if (i1 - i0) * (j1 - j0) > 4_000_000 {
        rec.skipped = Some("too_large");
        return (Vec::new(), Vec::new(), rec);
    }
    let w = grid.w as usize;
    let mut cells: Vec<u32> = Vec::with_capacity(((i1 - i0 + 1) * (j1 - j0 + 1)) as usize);
    for j in j0..=j1 {
        for i in i0..=i1 {
            let (ci, cj) = grid.cell(i, j);
            cells.push((cj as usize * w + ci as usize) as u32);
        }
    }
    cells.sort_unstable();
    cells.dedup();
    let at: Vec<At> = cells
        .iter()
        .filter_map(|&c| {
            let (ci, cj) = (c as usize % w, c as usize / w);
            let (x, z) = (ox + ci as f64 * r, oz + cj as f64 * r);
            let rho = angle(frame.unit(x, z), k.unit) * planet_r_m / k.radius_m;
            if rho > FOOT_RING.1 {
                return None;
            }
            let q = frame.nearest_image((x, z), vw);
            let (dx, dz) = (x - q.0, z - q.1);
            Some(At {
                c,
                x,
                z,
                rho,
                theta: dz.atan2(dx),
                dw: dx.hypot(dz),
                e: e.get(c as usize),
            })
        })
        .collect();
    // Its foot (the mean of the ring round it) and its top.
    let (mut fs, mut fn_) = (0.0, 0usize);
    let mut top = f64::NEG_INFINITY;
    for a in &at {
        if a.rho >= FOOT_RING.0 {
            fs += a.e;
            fn_ += 1;
        }
        if a.rho < TOP_RHO {
            top = top.max(a.e);
        }
    }
    // The calm over its body (VOL-068's rule, ρ 0.9 to 1.3), whatever its gullies.
    let calm: Vec<(u32, f32)> = at
        .iter()
        .filter_map(|a| {
            let inside = 1.0 - smooth(0.9, 1.3, a.rho); // ledger: VOL-068
            let v = k.young * inside;
            (v > 0.004).then_some((a.c, v as f32))
        })
        .collect();
    if fn_ == 0 || !top.is_finite() {
        rec.skipped = Some("no_ground");
        return (Vec::new(), calm, rec);
    }
    let foot = fs / fn_ as f64;
    let span = top - foot;
    rec.span_blocks = (span * 100.0).round() / 100.0;
    if span < MIN_SPAN_BLOCKS || top < g.sea_y + 1.0 {
        rec.skipped = Some("low");
        return (Vec::new(), calm, rec);
    }
    let target = eroded_share(k.age_yr);
    let deg = width_deg(target);
    let n = gully_count(deg);
    let nf = f64::from(n);
    let pw = profile_power(target);
    let ph = phases(k.id);
    rec.gullies = n;
    rec.power = (pw * 1000.0).round() / 1000.0;
    rec.target = (target * 1000.0).round() / 1000.0;
    // Each cell's cut at full strength (blocks), then the strength that takes the target share.
    let shape: Vec<f64> = at
        .iter()
        .map(|a| {
            let h = a.e - foot;
            if h <= 0.0 || a.rho >= FOOT_FADE.1 || (g.lake)(a.c as usize) {
                return 0.0;
            }
            let zone = 1.0 - smooth(SUMMIT_ZONE.0, SUMMIT_ZONE.1, h / span); // ledger: VOL-100
            let outer = 1.0 - smooth(FOOT_FADE.0, FOOT_FADE.1, a.rho);
            let crater = if k.crater_rho > 0.0 {
                smooth(k.crater_rho, 1.3 * k.crater_rho, a.rho) // ledger: VOL-104 (game-choice)
            } else {
                1.0
            };
            let onset = smooth(ONSET_SPACING.0, ONSET_SPACING.1, a.dw * TAU / nf);
            let base = zone * outer * crater * onset;
            if base <= 0.0 {
                return 0.0;
            }
            // The axis wanders a little down the flank and the gullies differ in depth round the
            // cone (even harmonics of θ only). ledger: VOL-104 (game-choice)
            let wig = 0.12 * (4.0 * a.theta + TAU * 1.7 * a.rho + ph[1]).sin()
                + 0.08 * (8.0 * a.theta - TAU * 2.3 * a.rho + ph[2]).sin();
            let u = (nf * a.theta / TAU + wig + ph[0]).rem_euclid(1.0);
            // The basins tile the flank divide to divide: deepest on the axis, nothing on the
            // divide, sharp-floored V when young, a broad U when old. ledger: VOL-103
            let off = u.min(1.0 - u);
            let profile = (1.0 - 2.0 * off).max(0.0).powf(pw);
            let depth = 1.0 + 0.2 * (4.0 * a.theta + ph[3]).sin();
            let ice = 1.0 - smooth(0.0, 4.0, (g.ice)(a.x, a.z));
            let v = h * base * profile * depth * ice;
            if v <= 0.0 || (g.river)(a.x, a.z) {
                return 0.0;
            }
            v
        })
        .collect();
    let volume: f64 = at
        .iter()
        .filter(|a| a.rho < 1.0)
        .map(|a| (a.e - foot).max(0.0))
        .sum();
    let full: f64 = shape.iter().sum();
    if full <= 0.0 || volume <= 0.0 {
        rec.skipped = Some("no_flank");
        return (Vec::new(), calm, rec);
    }
    let want = target * volume / full;
    rec.capped = want > MAX_CUT_SHARE;
    let strength = want.min(MAX_CUT_SHARE);
    let floor = g.sea_y + 1.0;
    let mut removed = 0.0;
    let mut deepest = 0.0f64;
    let cuts: Vec<(u32, f32)> = at
        .iter()
        .zip(&shape)
        .filter_map(|(a, &s)| {
            let cut = strength * k.young * s;
            // Never below a block over the sea: no new sea, no new closed basin.
            let cut = cut.min((a.e - floor).max(0.0));
            if cut < 1.0 / 32.0 {
                return None;
            }
            removed += cut;
            deepest = deepest.max(cut);
            Some((a.c, cut as f32))
        })
        .collect();
    rec.removed = (removed / volume * 1000.0).round() / 1000.0;
    rec.deepest_blocks = (deepest * 100.0).round() / 100.0;
    (cuts, calm, rec)
}

/// Cuts the cones' gullies into `e` (the deepest cut where two cones overlap). Returns the calm
/// each cell keeps from the steep-ground texture (0..255 over the grid; empty without cones) and
/// the pass's record.
pub fn cone_gullies(
    e: &mut Elev,
    g: &Ground,
    frame: &dyn Frame,
    planet_r_m: f64,
    cones: &[Cone],
) -> (Vec<u8>, Value) {
    if cones.is_empty() {
        return (Vec::new(), json!({"cones": 0}));
    }
    let read: &Elev = e;
    let mut outs: Vec<_> = cones
        .par_iter()
        .map(|k| one_cone(read, g, frame, planet_r_m, k))
        .collect();
    let mut cuts: Vec<(u32, f32)> = Vec::new();
    let mut calm = vec![0u8; g.grid.cells()];
    let mut records = Vec::with_capacity(outs.len());
    for (c, k, rec) in outs.drain(..) {
        cuts.extend(c);
        for (cell, v) in k {
            let q = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            let slot = &mut calm[cell as usize];
            *slot = (*slot).max(q);
        }
        records.push(rec);
    }
    cuts.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(b.1.total_cmp(&a.1)));
    cuts.dedup_by_key(|c| c.0);
    for &(cell, cut) in &cuts {
        let c = cell as usize;
        let y = e.get(c);
        e.set(c, y - f64::from(cut));
    }
    // A half-turn grid's edge rows hold each point twice (cell i is cell w − i) and a cone's box
    // reaches only one of them: both take the cut.
    if g.grid.z_edges == ZEdges::HalfTurn {
        let (w, h) = (g.grid.w as usize, g.grid.h as usize);
        for j in [0, h - 1] {
            for i in 1..w / 2 {
                let (a, b) = (j * w + i, j * w + (w - i));
                let (ya, yb) = (e.get(a), e.get(b));
                if ya < yb {
                    e.set(b, ya);
                } else if yb < ya {
                    e.set(a, yb);
                }
            }
        }
    }
    let drawn: Vec<&ConeRecord> = records.iter().filter(|r| r.skipped.is_none()).collect();
    let med = |f: &dyn Fn(&ConeRecord) -> f64| -> Value {
        let mut v: Vec<f64> = drawn.iter().map(|r| f(r)).collect();
        if v.is_empty() {
            return Value::Null;
        }
        v.sort_by(f64::total_cmp);
        json!(v[v.len() / 2])
    };
    let skipped = |why: &str| records.iter().filter(|r| r.skipped == Some(why)).count();
    let note = json!({
        "cones": cones.len(),
        "gullied": drawn.len(),
        "skipped_low": skipped("low"),
        "skipped_no_flank": skipped("no_flank"),
        "skipped_other": skipped("no_ground") + skipped("too_large"),
        "capped": drawn.iter().filter(|r| r.capped).count(),
        "cells_cut": cuts.len(),
        "target_share_median": med(&|r| r.target),
        "removed_share_median": med(&|r| r.removed),
        "gullies_median": med(&|r| f64::from(r.gullies)),
        "deepest_blocks_median": med(&|r| r.deepest_blocks),
        "deepest_blocks_max": drawn.iter().map(|r| r.deepest_blocks).fold(0.0, f64::max),
        "first": drawn.iter().take(6).map(|r| json!([r.id, r.gullies, r.target, r.removed, r.span_blocks, r.deepest_blocks])).collect::<Vec<_>>(),
    });
    (calm, note)
}

// --- the projection pass's call --------------------------------------------------------------------

impl Frame for Mapping {
    fn unit(&self, x: f64, z: f64) -> [f64; 3] {
        self.g.unit(x, z)
    }

    fn point(&self, v: [f64; 3]) -> (f64, f64) {
        self.g.point_of_unit(v)
    }

    fn nearest_image(&self, p: (f64, f64), q: (f64, f64)) -> (f64, f64) {
        Mapping::nearest_image(self, p, q).0
    }
}

/// What the projection pass hands the relief: the world's grid of the elevation and the layers and
/// vectors it reads, the simulation's rivers and standing volcanoes (simulation blocks, read
/// through `map`).
pub(crate) struct OnWorld<'a> {
    pub map: &'a Mapping,
    pub grid: &'a GridDesc,
    pub water: Option<(&'a LayerDesc, &'a [u8])>,
    pub ice: Option<&'a LoadedLayer>,
    pub rivers: Option<&'a Rivers>,
    pub standing: Option<&'a Value>,
    pub sea_y: f64,
    /// The planet's radius, m.
    pub planet_r_m: f64,
}

impl OnWorld<'_> {
    /// Draws the relief into the world's elevation cells `elev` (of `elev_desc`, on `self.grid`);
    /// returns `notes.bake.projection.relief`.
    pub(crate) fn draw(&self, elev_desc: &LayerDesc, elev: &mut [u8]) -> Value {
        let t = std::time::Instant::now();
        let map = self.map;
        let lake = |c: usize| match self.water {
            Some((d, b)) => {
                let cb = d.cell_bytes();
                d.physical(d.dtype.decode_scalar(&b[c * cb..(c + 1) * cb]))
                    .is_finite()
            }
            None => false,
        };
        let river = |x: f64, z: f64| match self.rivers {
            Some(r) => {
                let (xs, zs, _) = map.src_block(map.g.unit(x, z));
                let s = r.sample(xs.round() as i64, zs.round() as i64);
                s.hit.is_some() || s.bank_cap.is_some()
            }
            None => false,
        };
        let ice = |x: f64, z: f64| self.ice.map_or(0.0, |l| l.cover(x, z));
        let ground = Ground {
            grid: self.grid,
            sea_y: self.sea_y,
            lake: &lake,
            river: &river,
            ice: &ice,
        };
        let mut e = Elev {
            desc: elev_desc,
            bytes: elev,
        };
        let cones = self
            .standing
            .map(|v| cones_of(v, |x, z| map.src.unit(x, z)))
            .unwrap_or_default();
        let (_calm, cones_note) = cone_gullies(&mut e, &ground, map, self.planet_r_m, &cones);
        json!({
            "cone_gullies": cones_note,
            "millis": t.elapsed().as_secs_f64() * 1000.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::DType;
    use deeptime_format::projection::PetroffGuyou;

    const MH: f64 = 300.0;

    /// A world on a sphere of radius `r` blocks drawn plate carrée round the equator:
    /// (x, z) ↔ (longitude x/r, latitude −z/r), x wrapping with period 2πr.
    struct Flat {
        r: f64,
    }

    impl Frame for Flat {
        fn unit(&self, x: f64, z: f64) -> [f64; 3] {
            let (lon, lat) = (x / self.r, -z / self.r);
            [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()]
        }

        fn point(&self, v: [f64; 3]) -> (f64, f64) {
            let lon = v[1].atan2(v[0]);
            let lat = v[2].clamp(-1.0, 1.0).asin();
            (lon * self.r, -lat * self.r)
        }

        fn nearest_image(&self, p: (f64, f64), q: (f64, f64)) -> (f64, f64) {
            let period = TAU * self.r;
            let k = ((p.0 - q.0) / period).round();
            (q.0 + k * period, q.1)
        }
    }

    /// 2048 × 200 cells of 2 blocks round the equator, its frame and the planet's radius in m.
    fn flat() -> (GridDesc, Flat, f64) {
        let (w, h, r) = (2048u32, 200u32, 2u32);
        let grid = GridDesc {
            name: "L3".into(),
            w,
            h,
            r,
            wraps_x: true,
            origin_block_x: -i64::from(w * r / 2),
            origin_block_z: -i64::from(h * r / 2),
            z_edges: ZEdges::Clamp,
        };
        let rb = f64::from(w * r) / TAU;
        (grid, Flat { r: rb }, rb * MH)
    }

    /// The elevation layer of `grid` from `f(x, z)` (blocks).
    fn elevation(grid: &GridDesc, f: impl Fn(f64, f64) -> f64) -> (LayerDesc, Vec<u8>) {
        let desc = crate::bake::desc("elevation", DType::I16, grid, 1.0 / 16.0, "block/16", None);
        let mut bytes = vec![0u8; grid.cells() * 2];
        let w = grid.w as usize;
        for c in 0..grid.cells() {
            let x = (grid.origin_block_x + (c % w) as i64 * i64::from(grid.r)) as f64;
            let z = (grid.origin_block_z + (c / w) as i64 * i64::from(grid.r)) as f64;
            crate::bake::project::put_raw(DType::I16, f(x, z) * 16.0, &mut bytes[2 * c..2 * c + 2]);
        }
        (desc, bytes)
    }

    fn read(desc: &LayerDesc, bytes: &[u8], c: usize) -> f64 {
        desc.physical(desc.dtype.decode_scalar(&bytes[2 * c..2 * c + 2]))
    }

    const SEA: f64 = 63.0;
    const FOOT: f64 = 100.0;
    const TALL: f64 = 50.0;
    const RADIUS_BLOCKS: f64 = 60.0;

    /// A straight-sided cone at world point `v` (blocks) on flat ground at `foot`.
    fn cone_world(
        grid: &GridDesc,
        frame: &dyn Frame,
        rm: f64,
        v: (f64, f64),
        foot: f64,
    ) -> (LayerDesc, Vec<u8>) {
        let vu = frame.unit(v.0, v.1);
        elevation(grid, |x, z| {
            let rho = angle(frame.unit(x, z), vu) * rm / (RADIUS_BLOCKS * MH);
            foot + TALL * (1.0 - rho).max(0.0)
        })
    }

    fn cone(frame: &dyn Frame, v: (f64, f64), born_ago_myr: f64, dead_myr: Option<f64>) -> Cone {
        let extinct = dead_myr.map(|d| -d);
        Cone {
            id: 41,
            unit: frame.unit(v.0, v.1),
            radius_m: RADIUS_BLOCKS * MH,
            crater_rho: 0.1,
            age_yr: log_mean_age_yr(0.0, -born_ago_myr, extinct),
            young: dead_myr.map_or(1.0, |d| (-d / 5.0).exp()),
        }
    }

    fn no_lake(_: usize) -> bool {
        false
    }
    fn no_river(_: f64, _: f64) -> bool {
        false
    }
    fn no_ice(_: f64, _: f64) -> f64 {
        0.0
    }

    fn ground(grid: &GridDesc) -> Ground<'_> {
        Ground {
            grid,
            sea_y: SEA,
            lake: &no_lake,
            river: &no_river,
            ice: &no_ice,
        }
    }

    /// Runs the cone gullies on one cone; returns the elevation after and the cone's record.
    fn gullied(
        g: &Ground,
        frame: &dyn Frame,
        rm: f64,
        k: &Cone,
        desc: &LayerDesc,
        bytes: &[u8],
    ) -> (Vec<u8>, ConeRecord) {
        let mut after = bytes.to_vec();
        let mut e = Elev {
            desc,
            bytes: &mut after,
        };
        let (_, _, rec) = one_cone(&e, g, frame, rm, k);
        let (_, note) = cone_gullies(&mut e, g, frame, rm, std::slice::from_ref(k));
        assert_eq!(note["gullied"], json!(1), "{note}");
        (after, rec)
    }

    #[test]
    fn oharas_eroded_volume_against_age() {
        // Fig. 4h: 5.36·log10(age) − 1.56 %.
        assert!((eroded_share(10_000.0) - 0.1988).abs() < 1e-9);
        assert!((eroded_share(100_000.0) - 0.2524).abs() < 1e-9);
        assert_eq!(eroded_share(10.0), 0.05);
        assert_eq!(eroded_share(1e12), 0.40);
        // The log-mean age: an active cone 2 Myr old has erupted within the last 1 kyr.
        assert!((log_mean_age_yr(0.0, -2.0, None) - (2e6f64 * 1e3).sqrt()).abs() < 1e-6);
        // A dead one: 1 Myr dead, 4 Myr since it began.
        assert!((log_mean_age_yr(0.0, -4.0, Some(-1.0)) - 2e6).abs() < 1e-6);
        // Young cones: narrow gullies, many; old ones: wide, few.
        assert_eq!(gully_count(width_deg(0.10)), 16);
        assert_eq!(gully_count(width_deg(0.40)), 8);
        assert!(width_deg(0.10) < width_deg(0.25) && width_deg(0.25) < width_deg(0.40));
    }

    #[test]
    fn the_summit_and_crater_stay_smooth_and_the_flanks_are_gullied_to_oharas_share() {
        let (grid, frame, rm) = flat();
        let v = (0.0, 0.0);
        let (desc, bytes) = cone_world(&grid, &frame, rm, v, FOOT);
        let g = ground(&grid);
        let k = cone(&frame, v, 2.0, None);
        let (after, rec) = gullied(&g, &frame, rm, &k, &desc, &bytes);
        let target = eroded_share(k.age_yr);
        assert!(rec.skipped.is_none(), "{rec:?}");
        assert!(
            (rec.removed - target).abs() < 0.01,
            "removed {} of the cone, O'Hara's share {target}",
            rec.removed
        );
        let w = grid.w as usize;
        let vu = frame.unit(v.0, v.1);
        // Per ring of ρ, the least and most cut.
        let mut rings = [(f64::INFINITY, 0.0f64); 12];
        for c in 0..grid.cells() {
            let before = read(&desc, &bytes, c);
            let cut = before - read(&desc, &after, c);
            assert!(cut >= -1e-9, "the gullies only cut");
            let x = (grid.origin_block_x + (c % w) as i64 * 2) as f64;
            let z = (grid.origin_block_z + (c / w) as i64 * 2) as f64;
            let rho = angle(frame.unit(x, z), vu) * rm / k.radius_m;
            let h_rel = (before - FOOT) / TALL;
            if h_rel >= SUMMIT_ZONE.1 || rho < 1.3 * k.crater_rho || rho >= FOOT_FADE.1 {
                assert_eq!(cut, 0.0, "a cut at ρ {rho}, {h_rel} of the height");
            }
            let ring = (rho * 10.0) as usize;
            if ring < rings.len() {
                rings[ring].0 = rings[ring].0.min(cut);
                rings[ring].1 = rings[ring].1.max(cut);
            }
        }
        // Mid-flank: gullies several blocks deep beside ridges left whole.
        for (ring, &(lo, hi)) in rings.iter().enumerate().skip(4).take(4) {
            assert!(hi > 3.0, "ring {ring}: the deepest gully is {hi} blocks");
            assert!(
                lo < 0.1 * hi,
                "ring {ring}: the ridges are cut by {lo} of {hi}"
            );
        }
        assert!(!rec.capped, "{rec:?}");
    }

    #[test]
    fn older_cones_have_fewer_wider_deeper_gullies_and_dead_ones_hand_them_over() {
        let (grid, frame, rm) = flat();
        let v = (0.0, 0.0);
        let (desc, bytes) = cone_world(&grid, &frame, rm, v, FOOT);
        let g = ground(&grid);
        let run = |k: &Cone| gullied(&g, &frame, rm, k, &desc, &bytes).1;
        let young = run(&cone(&frame, v, 0.005, None));
        let old = run(&cone(&frame, v, 30.0, None));
        assert!(young.gullies > old.gullies, "{young:?} {old:?}");
        assert!(young.power > old.power, "V-cuts young, U-cuts old");
        assert!(young.removed + 0.05 < old.removed, "{young:?} {old:?}");
        assert!(young.deepest_blocks < old.deepest_blocks);
        // Dead 10 Myr (fade 5 Myr): e⁻² of the gullies are left, the character takes the rest.
        let alive = cone(&frame, v, 12.0, None);
        let dead = cone(&frame, v, 12.0, Some(10.0));
        assert!((dead.young - (-2.0f64).exp()).abs() < 1e-12);
        let same_age = Cone {
            age_yr: alive.age_yr,
            ..dead
        };
        let (a, d) = (run(&alive), run(&same_age));
        let want = a.removed * (-2.0f64).exp();
        assert!((d.removed - want).abs() < 0.003, "{d:?} vs {want}");
    }

    #[test]
    fn no_cut_below_a_block_over_the_sea_at_a_lake_or_a_river() {
        let (grid, frame, rm) = flat();
        let v = (0.0, 0.0);
        // A cone rising out of the sea.
        let (desc, bytes) = cone_world(&grid, &frame, rm, v, SEA - 20.0);
        let w = grid.w as usize;
        let ox = grid.origin_block_x as f64;
        // A lake on the cells west of the vent at z < 0, a river east of it.
        let lake = |c: usize| (ox + (c % w) as f64 * 2.0) < -4.0 && c / w < 100;
        let river = |x: f64, _: f64| x > 4.0;
        let g = Ground {
            grid: &grid,
            sea_y: SEA,
            lake: &lake,
            river: &river,
            ice: &no_ice,
        };
        let k = cone(&frame, v, 2.0, None);
        let (after, _) = gullied(&g, &frame, rm, &k, &desc, &bytes);
        let mut cut_any = false;
        for c in 0..grid.cells() {
            let (b, a) = (read(&desc, &bytes, c), read(&desc, &after, c));
            let x = ox + (c % w) as f64 * 2.0;
            if b < SEA + 1.0 || lake(c) || river(x, 0.0) {
                assert_eq!(a, b, "cell {c}");
            } else {
                assert!(a >= SEA + 1.0 - 1.0 / 32.0, "cell {c}: {b} cut to {a}");
                cut_any |= a < b;
            }
        }
        assert!(cut_any, "the dry flank in the north-west is gullied");
    }

    #[test]
    fn the_gullies_read_the_same_across_the_seams() {
        // The world's x seam: the same cone at x = 0 and astride the seam, cut alike.
        let (grid, frame, rm) = flat();
        let g = ground(&grid);
        let w = grid.w as usize;
        let shift = w / 2 - 6; // cells: the second vent sits 12 blocks east of the seam
        let v0 = (0.0, 0.0);
        let v1 = (grid.origin_block_x as f64 + 12.0, 0.0);
        let (d0, b0) = cone_world(&grid, &frame, rm, v0, FOOT);
        let (d1, b1) = cone_world(&grid, &frame, rm, v1, FOOT);
        let (a0, _) = gullied(&g, &frame, rm, &cone(&frame, v0, 2.0, None), &d0, &b0);
        let (a1, _) = gullied(&g, &frame, rm, &cone(&frame, v1, 2.0, None), &d1, &b1);
        let mut cut = 0;
        for c in 0..grid.cells() {
            let (i, j) = (c % w, c / w);
            let c1 = j * w + (i + w - shift) % w;
            let d =
                (read(&d0, &b0, c) - read(&d0, &a0, c)) - (read(&d1, &b1, c1) - read(&d1, &a1, c1));
            assert!(d.abs() <= 1.0 / 16.0 + 1e-9, "cell {c}: {d}");
            cut += usize::from(read(&d0, &a0, c) < read(&d0, &b0, c));
        }
        assert!(cut > 1000);
        // The same run twice: the same cells, bit for bit.
        let (again, _) = gullied(&g, &frame, rm, &cone(&frame, v0, 2.0, None), &d0, &b0);
        assert_eq!(a0, again);
    }

    #[test]
    fn a_cone_on_a_petroff_guyou_edge_keeps_the_edge_glued() {
        // The north edge is glued onto itself by a half-turn: cells (i, 0) and (W − i, 0) are one
        // point of the planet, so they must keep one height.
        let c_sim = 4096;
        let pg = PetroffGuyou::new(PetroffGuyou::hemisphere_for(c_sim), 30.0);
        let map = Mapping::new(pg, c_sim, 16);
        let grid = pg.grid("L3", 2).unwrap();
        let b = pg.hemisphere() as f64;
        let rm = f64::from(c_sim) / TAU * MH;
        let v = (300.0, -b / 2.0 + 14.0);
        let (desc, bytes) = cone_world(&grid, &map, rm, v, FOOT);
        let g = ground(&grid);
        let k = cone(&map, v, 2.0, None);
        let (after, rec) = gullied(&g, &map, rm, &k, &desc, &bytes);
        assert!(rec.removed > 0.1, "{rec:?}");
        let w = grid.w as usize;
        let mut cut = 0;
        for i in 1..w {
            let (a, m) = (i, w - i);
            let (ya, ym) = (read(&desc, &after, a), read(&desc, &after, m));
            let (ba, bm) = (read(&desc, &bytes, a), read(&desc, &bytes, m));
            assert!(
                (ya - ym).abs() <= (ba - bm).abs() + 1.0 / 16.0 + 1e-9,
                "edge cells {a} and {m}: {ya} and {ym} (before {ba} and {bm})"
            );
            cut += usize::from(ya < ba);
        }
        assert!(
            cut > 4,
            "the cone's gullies reach the edge row: {cut} cells"
        );
    }

    #[test]
    fn standing_volcanoes_give_their_cones() {
        let v = json!({"version": 1, "t_myr": 500.0, "fade_myr": 5.0, "volcanoes": [
            {"id": 3, "kind": "stratovolcano", "x": 10.0, "z": -20.0, "radius_m": 14000.0,
             "height_m": 2400.0, "crater_rho": 0.07, "born_myr": 498.0, "extinct_myr": null},
            {"id": 9, "kind": "stratovolcano", "x": 50.0, "z": 5.0, "radius_m": 9000.0,
             "height_m": 900.0, "crater_rho": null, "born_myr": 480.0, "extinct_myr": 495.0}]});
        let cones = cones_of(&v, |x, z| [x, z, 0.0]);
        assert_eq!(cones.len(), 2);
        assert_eq!(
            (cones[0].id, cones[0].young, cones[0].crater_rho),
            (3, 1.0, 0.07)
        );
        assert!((cones[0].age_yr - (2e6f64 * 1e3).sqrt()).abs() < 1e-3);
        assert_eq!(cones[0].unit, [10.0, -20.0, 0.0]);
        assert!((cones[1].young - (-1.0f64).exp()).abs() < 1e-12);
        assert!((cones[1].age_yr - (20e6f64 * 5e6).sqrt()).abs() < 1e-3);
        assert_eq!(cones[1].crater_rho, 0.0);
    }
}
