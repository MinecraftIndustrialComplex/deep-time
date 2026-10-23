// SPDX-License-Identifier: LGPL-3.0-or-later
//! The block-scale relief the projection pass draws on the world's own grid
//! ([`crate::bake::project::project_folder`]), after the resampling and before anything reads the
//! world's terrain: in world blocks, from the drawn terrain, so it does not depend on the frame
//! the simulation ran in (the world's layout reaches it only through [`Frame`]).
//!
//! - **Cone gullies** (Ben, decisions log 2026-10-22, D2: "cones get a smooth fresh summit with
//!   radially gullied older flanks", O'Hara et al. 2024). A standing stratovolcano
//!   (`volcanoes/standing.json`, written by the bake) gets radial gullies cut into its flanks,
//!   basins tiling the cone divide to divide (V-cuts when young, broad valleys when old), none in
//!   its summit zone (the upper 30 % of its height over its foot, O'Hara's summit
//!   region) or its crater. Their depth follows O'Hara's eroded volume against the cone's age (the
//!   log-mean of the times since its first and its last activity): each cone's cuts take that share
//!   of its volume over its foot. Wider and fewer gullies as it ages, and a dead cone's fade on the
//!   crater's clock as the mountain character takes it over (VOL-068). No cut where a river or a
//!   lake is, under ice, or below one block over the sea.
//! - **Steep-ground texture** (Ben, decisions log 2026-10-22, D3: "jagged detail and cliff bands in
//!   the heightfield"). Steep hard rock gets a ridged jag of a few blocks, so a face is broken into
//!   knobs and chutes instead of a smooth stair of equal steps; steep lava (basalt, andesite,
//!   rhyolite) gets cliff bands, benches and cliffs like a flood basalt's or a lava plateau's
//!   (USGS Circular 838). The rock is the world's `surface/class_top`. Not over a standing
//!   volcano's body while it is young (VOL-068's calm), nor on a lake, a river, ice or within a
//!   block of the sea.
//!
//! Everything goes into the `elevation` layer (H = E + A·N unchanged; the game, the far terrain
//! and the column evaluator read it as terrain), so the world-data format gains only the file the
//! cone gullies read, `volcanoes/standing.json` (FORMAT.md, "Standing volcanoes"). No overhangs:
//! a heightfield has none, and the far terrain keeps matching the near.

use std::f64::consts::TAU;
use std::path::Path;

use rayon::prelude::*;
use serde_json::{Value, json};

use deeptime_format::raster::{RasterRead, ZEdges, bilinear_weights};
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

/// `volcanoes/standing.json`: the volcanoes still standing at the present (alive or dead, every
/// kind, at least [`MIN_HEIGHT_M`] high), in id order, with the times the cone gullies' age needs.
/// Only the stratovolcanoes are gullied; every kind holds the steep-ground texture back over its
/// body (VOL-068).
pub fn standing_json(st: &VolcanoState) -> Value {
    let r3 = |v: f64| (v * 1000.0).round() / 1000.0;
    let mut list: Vec<_> = st
        .volcanoes
        .iter()
        .filter(|v| v.gone.is_none() && v.radius_m > 0.0 && v.height_m >= MIN_HEIGHT_M)
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
const ONSET_SPACING: (f64, f64) = (8.0, 12.0); // ledger: VOL-104 (game-choice)
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
    1.5 - 0.8 * smooth(0.10, 0.35, e) // ledger: VOL-104 (game-choice)
}

/// A standing volcano as the relief draws it.
#[derive(Clone, Debug, PartialEq)]
pub struct Cone {
    pub id: u32,
    /// A stratovolcano (gullied); a shield or a caldera complex only calms the ground over it.
    pub strato: bool,
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
                strato: o["kind"].as_str() == Some(Kind::Strato.name()),
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
    /// 1 while active, less once dead ([`Cone::young`]).
    pub young: f64,
    pub skipped: Option<&'static str>,
}

/// Values by cell index.
type CellValues = Vec<(u32, f32)>;

/// One cone's cuts (cell, blocks) and its calm (cell, 0..1: how much of the steep-ground texture
/// to hold back over its body), read from the terrain `e`.
fn one_cone(
    e: &Elev,
    g: &Ground,
    frame: &dyn Frame,
    planet_r_m: f64,
    k: &Cone,
) -> (CellValues, CellValues, ConeRecord) {
    let grid = g.grid;
    let mut rec = ConeRecord {
        id: k.id,
        young: k.young,
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
    let cube = grid.is_cube();
    // A cube grid's cells are laid out by face, not by net point: its box goes through the net's
    // lattice of cell centres instead, folded onto the faces (SCL-123).
    let (ox, oz) = if cube {
        // The net's corner, (−3s/2, −3s/2): every face's cells sit on this lattice.
        let s = f64::from(grid.face_blocks());
        (-1.5 * s, -1.5 * s)
    } else {
        (grid.origin_block_x as f64, grid.origin_block_z as f64)
    };
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
            let (ci, cj) = if cube {
                let (x, z) = (ox + (i as f64 + 0.5) * r, oz + (j as f64 + 0.5) * r);
                let Some(sp) = grid.cube_spot(x, z) else {
                    continue;
                };
                grid.face_cell(sp.face, sp.a.round() as i64, sp.b.round() as i64)
            } else {
                grid.cell(i, j)
            };
            cells.push((cj as usize * w + ci as usize) as u32);
        }
    }
    cells.sort_unstable();
    cells.dedup();
    let at: Vec<At> = cells
        .iter()
        .filter_map(|&c| {
            let (ci, cj) = (c as usize % w, c as usize / w);
            let (x, z) = if cube {
                grid.cube_point(ci as u32, cj as u32)
            } else {
                (ox + ci as f64 * r, oz + cj as f64 * r)
            };
            let t = frame.unit(x, z);
            let rho = angle(t, k.unit) * planet_r_m / k.radius_m;
            if rho.is_nan() || rho > FOOT_RING.1 {
                return None;
            }
            let q = frame.nearest_image((x, z), vw);
            let (dx, dz) = (x - q.0, z - q.1);
            // The angle round the vent: on a cube the net turns at the seams, so it is taken on
            // the planet, in the vent's own tangent frame (SCL-123).
            let theta = if cube {
                dot(t, e2).atan2(dot(t, e1))
            } else {
                dz.atan2(dx)
            };
            Some(At {
                c,
                x,
                z,
                rho,
                theta,
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
    let calm: CellValues = at
        .iter()
        .filter_map(|a| {
            let inside = 1.0 - smooth(0.9, 1.3, a.rho); // ledger: VOL-068
            let v = k.young * inside;
            (v > 0.004).then_some((a.c, v as f32))
        })
        .collect();
    if !k.strato {
        rec.skipped = Some("not_strato");
        return (Vec::new(), calm, rec);
    }
    if fn_ == 0 || !top.is_finite() {
        rec.skipped = Some("no_ground");
        return (Vec::new(), calm, rec);
    }
    if top < g.sea_y + 1.0 {
        rec.skipped = Some("sea");
        return (Vec::new(), calm, rec);
    }
    // The cone over its ground, or over the sea for an island: nothing is cut under a block over
    // the sea, so O'Hara's share is of the cone above it (their cones are subaerial).
    let foot = (fs / fn_ as f64).max(g.sea_y + 1.0); // ledger: VOL-101
    let span = top - foot;
    rec.span_blocks = (span * 100.0).round() / 100.0;
    if span < MIN_SPAN_BLOCKS {
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
            // divide, sharp-floored V when young, a broad U when old. ledger: VOL-104 (game-choice)
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
    let cuts: CellValues = at
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
    let mut cuts: CellValues = Vec::new();
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
    // The active cones and the dead ones apart: a dead cone's gullies are faded (VOL-105).
    let group = |active: bool| -> Value {
        let mine: Vec<&ConeRecord> = records
            .iter()
            .filter(|r| (r.young >= 1.0) == active && r.skipped != Some("not_strato"))
            .collect();
        let drawn: Vec<&ConeRecord> = mine
            .iter()
            .copied()
            .filter(|r| r.skipped.is_none())
            .collect();
        let med = |f: &dyn Fn(&ConeRecord) -> f64| -> Value {
            let mut v: Vec<f64> = drawn.iter().map(|r| f(r)).collect();
            if v.is_empty() {
                return Value::Null;
            }
            v.sort_by(f64::total_cmp);
            json!(v[v.len() / 2])
        };
        let skipped = |why: &str| mine.iter().filter(|r| r.skipped == Some(why)).count();
        json!({
            "cones": mine.len(),
            "gullied": drawn.len(),
            "skipped_sea": skipped("sea"),
            "skipped_low": skipped("low"),
            "skipped_no_flank": skipped("no_flank"),
            "skipped_other": skipped("no_ground") + skipped("too_large"),
            "capped": drawn.iter().filter(|r| r.capped).count(),
            "target_share_median": med(&|r| r.target),
            "removed_share_median": med(&|r| r.removed),
            "gullies_median": med(&|r| f64::from(r.gullies)),
            "span_blocks_median": med(&|r| r.span_blocks),
            "deepest_blocks_median": med(&|r| r.deepest_blocks),
            "deepest_blocks_max": drawn.iter().map(|r| r.deepest_blocks).fold(0.0, f64::max),
            "first": drawn.iter().take(6).map(|r| json!([r.id, r.gullies, r.target, r.removed, r.span_blocks, r.deepest_blocks])).collect::<Vec<_>>(),
        })
    };
    let note = json!({
        "cones": cones.len(),
        "not_strato": records.iter().filter(|r| r.skipped == Some("not_strato")).count(),
        "gullied": records.iter().filter(|r| r.skipped.is_none()).count(),
        "cells_cut": cuts.len(),
        "active": group(true),
        "dead": group(false),
    });
    (calm, note)
}

// --- steep-ground texture (D3) -------------------------------------------------------------------

/// The jag comes in as the ground's slope (blocks per block, read over [`SLOPE_REACH_CELLS`])
/// grows from the first to the second.
pub const JAG_SLOPE: (f64, f64) = (0.3, 1.2); // ledger: AMP-080 (game-choice)
/// The jag's amplitude per unit of slope, blocks, and its most.
pub const JAG_GAIN: f64 = 4.0; // ledger: AMP-080 (game-choice)
pub const JAG_MAX_BLOCKS: f64 = 8.0; // ledger: AMP-080 (game-choice)
/// The rock's hardness (AMP-048) over which the jag comes in: none on shale or mudstone, about
/// 0.6 of it on sandstone, all of it on granite, gneiss, quartzite or basalt.
pub const JAG_HARDNESS: (f64, f64) = (0.2, 0.9); // ledger: AMP-080 (game-choice)
/// The jag's noise: ridged, (1 − |n|)², of two octaves from this wavelength (blocks), so its
/// finest spans four of the game's 4-block noise corners.
const JAG_WAVELENGTH: f64 = 32.0; // ledger: AMP-081 (game-choice)
const JAG_OCTAVES: u32 = 2; // ledger: AMP-081 (game-choice)
/// The ridged noise's mean, taken off so the jag neither raises nor lowers the ground on the
/// whole (measured: test the_jag_is_ridged_and_centred).
const JAG_MEAN: f64 = 0.58; // ledger: AMP-081 (game-choice)

/// The cliff bands come in on lava as the slope grows from the first to the second.
pub const BAND_SLOPE: (f64, f64) = (0.15, 0.4); // ledger: AMP-082 (game-choice)
/// A band spans about this many blocks of run down the slope it is cut into (its height is the
/// run times the slope, read over [`REGION_REACH_CELLS`]), so its cliff (a [`CLIFF_SHARE`] of it,
/// about 7 blocks) spans two of the game's 4-block noise corners and its bench four: thinner
/// bands draw as a zigzag of single corners.
pub const BAND_RUN_BLOCKS: f64 = 24.0; // ledger: AMP-082 (game-choice)
/// The thinnest band, metres: three flows of the Columbia River basalts' mean thickness, 30 to
/// 40 m (USGS Circular 838). The bands come in this height times 1, 2, 4, … (so their levels nest
/// and stay level wherever the slope changes), the two nearest the slope's blended.
pub const BAND_MIN_M: f64 = 105.0; // ledger: AMP-084
/// The thickest band, blocks.
pub const BAND_MAX_BLOCKS: f64 = 24.0; // ledger: AMP-082 (game-choice)
/// The share of a band's run its cliff takes; the bench, flat at the band's top, takes the rest.
pub const CLIFF_SHARE: f64 = 0.3; // ledger: AMP-082 (game-choice)
/// The share of each doubling of the slope's band height over which the two nearest heights
/// blend (the rest takes the nearer alone, so its benches stay flat).
const BAND_BLEND: f64 = 0.3; // ledger: AMP-082 (game-choice)
/// The bands' levels shift by up to the thickest band over this wavelength (blocks), so the
/// benches do not sit at the same heights everywhere and are not quite level.
const BAND_WAVELENGTH: f64 = 512.0; // ledger: AMP-082 (game-choice)

/// The slope is the central difference over this many cells each way (2-block cells: a 16-block
/// baseline, over the mountain character's 8-block terraces), the bands' height over the second.
const SLOPE_REACH_CELLS: i64 = 4; // ledger: AMP-083 (game-choice)
const REGION_REACH_CELLS: i64 = 16; // ledger: AMP-083 (game-choice)
/// The texture fades in from the first height over the sea to the second (blocks), and out under
/// ice over this depth (blocks).
const COAST_FADE: (f64, f64) = (1.0, 6.0); // ledger: AMP-083 (game-choice)
const ICE_FADE_BLOCKS: f64 = 4.0; // ledger: AMP-083 (game-choice)

const JAG_SALT: u32 = 0xD3_0001;
const BAND_LEVEL_SALT: u32 = 0xD3_0003;

/// Rock hardness in [0, 1] from its erodibility multiplier: 1 at granite's (1) and below, 0 at
/// mudstone's (3) and above, logarithmic between (the mountain character's law, AMP-048).
fn hardness(kfac: f64) -> f64 {
    let k = kfac.max(1e-3);
    (((3.0f64).ln() - k.ln()) / (3.0f64).ln()).clamp(0.0, 1.0) // ledger: AMP-048
}

/// Per rock class (strata/classes.json's index, `surface/class_top`): how much of the jag it takes
/// ([`JAG_HARDNESS`] over its hardness) and whether it is lava (a volcanic class neither friable
/// nor porous: basalt, andesite, rhyolite, obsidian). Class 255 (none) takes neither.
pub fn rock_weights() -> Vec<(f32, f32)> {
    let mut out = vec![(0.0f32, 0.0f32); 256];
    for (k, c) in crate::surface::lithology::table()
        .iter()
        .enumerate()
        .take(255)
    {
        let hard = smooth(JAG_HARDNESS.0, JAG_HARDNESS.1, hardness(c.erodibility));
        let lava =
            c.family == "volcanic" && !c.tags.iter().any(|t| t == "friable" || t == "porous");
        out[k] = (hard as f32, if lava { 1.0 } else { 0.0 });
    }
    out
}

/// What the steep-ground texture reads beyond the ground: the world's noise, the rock at a world
/// point (the jag's weight and the lava share, 0..1 each) and the vertical scale.
pub struct Rock<'a> {
    pub noise: &'a deeptime_format::Noise,
    pub at: &'a (dyn Fn(f64, f64) -> (f64, f64) + Sync),
    /// Metres per block, vertically.
    pub m_per_block_v: f64,
    /// A cube world's net: its noise is blended across the turning seams
    /// ([`crate::bake::project_cube::seam_blend`]); none on any other world.
    pub seams: Option<&'a deeptime_format::cube::Cube>,
}

/// `f` at world point (x, z), blended across a cube's turning seams within `reach` blocks
/// ([`Rock::seams`]).
fn blended(rock: &Rock, x: f64, z: f64, reach: f64, f: impl Fn(f64, f64) -> f64) -> f64 {
    match rock.seams {
        Some(cube) => crate::bake::project_cube::seam_blend(cube, x, z, reach, f),
        None => f(x, z),
    }
}

/// The ridged jag noise at world point (x, z), centred: (1 − |n|)² less its mean.
fn jag_noise(
    spec: &deeptime_format::DetailSpec,
    noise: &deeptime_format::Noise,
    x: f64,
    z: f64,
) -> f64 {
    let n = spec.noise(noise, x, z);
    (1.0 - n.abs()).powi(2) - JAG_MEAN
}

/// The cliff bands' terrace of height `y` with band height `t` and level shift `off` (blocks): flat
/// benches at each band's top, cliffs over the lowest [`CLIFF_SHARE`] of each band's run. Never
/// below `y`.
fn terrace(y: f64, t: f64, off: f64) -> f64 {
    let q = (y + off) / t;
    let qi = q.floor();
    let f = q - qi;
    let g = f.max(smooth(0.0, CLIFF_SHARE, f));
    (qi + g) * t - off
}

/// Draws the steep-ground texture into `e`: a ridged jag on steep hard rock and cliff bands on
/// steep lava, everywhere on dry land a block or more over the sea, held back by `calm` (0..255
/// per cell, from [`cone_gullies`]; empty: none) over the standing volcanoes' bodies (VOL-068),
/// off lakes, rivers and ice. Returns the pass's record.
pub fn steep_texture(e: &mut Elev, g: &Ground, rock: &Rock, calm: &[u8]) -> Value {
    let grid = g.grid;
    let (w, h) = (grid.w as usize, grid.h as usize);
    let r = f64::from(grid.r);
    let (ox, oz) = (grid.origin_block_x as f64, grid.origin_block_z as f64);
    let jag_spec = deeptime_format::DetailSpec::new(0, JAG_SALT, JAG_OCTAVES, JAG_WAVELENGTH);
    let level_spec = deeptime_format::DetailSpec::new(0, BAND_LEVEL_SALT, 1, BAND_WAVELENGTH);
    // The bands come in the thinnest's height times 1, 2, 4, …, the thickest under
    // BAND_MAX_BLOCKS (ledger: AMP-082, AMP-084).
    let band_min = BAND_MIN_M / rock.m_per_block_v;
    let band_doublings = (BAND_MAX_BLOCKS / band_min).log2().floor().max(0.0);
    let band_top = band_min * band_doublings.exp2();
    let floor_y = g.sea_y + 1.0;
    let read: &Elev = e;
    let half_step = 0.5 * read.desc.scale.abs();
    let cube = grid.is_cube();
    let at = |i: i64, j: i64| -> f64 {
        let (ci, cj) = grid.cell(i, j);
        read.get(cj as usize * w + ci as usize)
    };
    // A cube grid's neighbours are taken on the cell's own face, across its seams: (i, j) is the
    // stacked cell and the steps leave its face (SCL-123).
    let slope = |i: i64, j: i64, k: i64| -> f64 {
        if cube {
            let n = i64::from(grid.w);
            let (f, b) = (j.div_euclid(n), j.rem_euclid(n));
            let face = deeptime_format::cube::Face::from_index(f as usize).expect("a face row");
            let get = |a: i64, b: i64| {
                let (ci, cj) = grid.face_cell(face, a, b);
                read.get(cj as usize * w + ci as usize)
            };
            let gx = (get(i + k, b) - get(i - k, b)) / (2.0 * k as f64 * r);
            let gz = (get(i, b + k) - get(i, b - k)) / (2.0 * k as f64 * r);
            return gx.hypot(gz);
        }
        let gx = (at(i + k, j) - at(i - k, j)) / (2.0 * k as f64 * r);
        let gz = (at(i, j + k) - at(i, j - k)) / (2.0 * k as f64 * r);
        gx.hypot(gz)
    };
    let mut delta = vec![0.0f32; w * h];
    let (jag_cells, band_cells) = delta
        .par_chunks_mut(w)
        .enumerate()
        .map(|(j, row)| {
            let mut n = (0usize, 0usize);
            for (i, out) in row.iter_mut().enumerate() {
                let c = j * w + i;
                let y0 = read.get(c);
                if y0 < floor_y || (g.lake)(c) {
                    continue;
                }
                let keep = 1.0 - calm.get(c).map_or(0.0, |&v| f64::from(v) / 255.0);
                let coast = smooth(COAST_FADE.0, COAST_FADE.1, y0 - g.sea_y);
                if keep <= 0.0 || coast <= 0.0 {
                    continue;
                }
                let (ii, jj) = (i as i64, j as i64);
                let s = slope(ii, jj, SLOPE_REACH_CELLS);
                if s < JAG_SLOPE.0.min(BAND_SLOPE.0) {
                    continue;
                }
                let (x, z) = if cube {
                    grid.cube_point(i as u32, j as u32)
                } else {
                    (ox + i as f64 * r, oz + j as f64 * r)
                };
                let (hard, lava) = (rock.at)(x, z);
                let jag_w = smooth(JAG_SLOPE.0, JAG_SLOPE.1, s) * hard;
                let band_w = smooth(BAND_SLOPE.0, BAND_SLOPE.1, s) * lava;
                if jag_w <= 0.0 && band_w <= 0.0 {
                    continue;
                }
                let ice = 1.0 - smooth(0.0, ICE_FADE_BLOCKS, (g.ice)(x, z));
                let wt = keep * coast * ice;
                if wt <= 0.0 {
                    continue;
                }
                let mut y = y0;
                if jag_w > 0.0 {
                    // ledger: AMP-080, AMP-081
                    let a = (JAG_GAIN * s).min(JAG_MAX_BLOCKS) * jag_w;
                    y += wt
                        * a
                        * blended(rock, x, z, JAG_WAVELENGTH, |x, z| {
                            jag_noise(&jag_spec, rock.noise, x, z)
                        });
                }
                if band_w > 0.0 {
                    // ledger: AMP-082
                    let sr = slope(ii, jj, REGION_REACH_CELLS);
                    let u = (BAND_RUN_BLOCKS * sr / band_min)
                        .max(1.0)
                        .log2()
                        .min(band_doublings);
                    let k = u.floor();
                    let tb = band_min * k.exp2();
                    // One shift for every height, so the levels nest.
                    let level = blended(rock, x, z, BAND_WAVELENGTH, |x, z| {
                        level_spec.noise(rock.noise, x, z)
                    });
                    let off = band_top * 0.5 * (1.0 + level);
                    let mix = smooth(0.5 - 0.5 * BAND_BLEND, 0.5 + 0.5 * BAND_BLEND, u - k);
                    let mut t = terrace(y, tb, off);
                    if mix > 0.0 {
                        t += mix * (terrace(y, 2.0 * tb, off) - t);
                    }
                    y += wt * band_w * (t - y).max(0.0);
                }
                // Never under a block over the sea (ledger: AMP-083); a move under half the
                // layer's step is no move.
                let y = y.max(floor_y.min(y0));
                if (y - y0).abs() >= half_step && !(g.river)(x, z) {
                    *out = (y - y0) as f32;
                    n.0 += usize::from(jag_w > 0.0);
                    n.1 += usize::from(band_w > 0.0);
                }
            }
            n
        })
        .reduce(|| (0, 0), |a, b| (a.0 + b.0, a.1 + b.1));
    let (mut up, mut down) = (0usize, 0usize);
    let (mut sum_abs, mut max_up, mut max_down) = (0.0f64, 0.0f64, 0.0f64);
    for (c, &d) in delta.iter().enumerate() {
        if d == 0.0 {
            continue;
        }
        let d = f64::from(d);
        if d > 0.0 {
            up += 1;
            max_up = max_up.max(d);
        } else {
            down += 1;
            max_down = max_down.max(-d);
        }
        sum_abs += d.abs();
        let y = e.get(c);
        e.set(c, y + d);
    }
    // A half-turn grid's edge rows hold each point twice (cell i is cell w − i): both take the
    // mean of what the two were given.
    if grid.z_edges == ZEdges::HalfTurn {
        for j in [0, h - 1] {
            for i in 1..w / 2 {
                let (a, b) = (j * w + i, j * w + (w - i));
                let (ya, yb) = (e.get(a), e.get(b));
                if ya != yb {
                    let m = 0.5 * (ya + yb);
                    e.set(a, m);
                    e.set(b, m);
                }
            }
        }
    }
    let moved = up + down;
    let r2 = |v: f64| (v * 100.0).round() / 100.0;
    json!({
        "cells_moved": moved,
        "cells_jagged": jag_cells,
        "cells_banded": band_cells,
        "cells_raised": up,
        "cells_lowered": down,
        "mean_abs_blocks": r2(if moved > 0 { sum_abs / moved as f64 } else { 0.0 }),
        "max_raise_blocks": r2(max_up),
        "max_cut_blocks": r2(max_down),
    })
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

/// A world's [`Frame`] with the simulation's (Lambert) grid behind it: what [`OnWorld`] reads the
/// rivers and the standing volcanoes through.
pub(crate) trait SimFrame: Frame {
    /// The simulation's block (xs, zs) of a sphere point.
    fn sim_block(&self, v: [f64; 3]) -> (f64, f64);
    /// The sphere point of the simulation's block (xs, zs).
    fn sim_unit(&self, xs: f64, zs: f64) -> [f64; 3];
}

impl SimFrame for Mapping {
    fn sim_block(&self, v: [f64; 3]) -> (f64, f64) {
        let (xs, zs, _) = self.src_block(v);
        (xs, zs)
    }

    fn sim_unit(&self, xs: f64, zs: f64) -> [f64; 3] {
        self.src.unit(xs, zs)
    }
}

/// What the projection pass hands the relief: the world's grid of the elevation and the layers and
/// vectors it reads, the simulation's rivers and standing volcanoes (simulation blocks, read
/// through `map`).
pub(crate) struct OnWorld<'a, M: SimFrame> {
    pub map: &'a M,
    pub grid: &'a GridDesc,
    pub water: Option<(&'a LayerDesc, &'a [u8])>,
    pub ice: Option<&'a LoadedLayer>,
    /// The world's `surface/class_top` (L2): the rock the steep-ground texture reads.
    pub class_top: Option<&'a LoadedLayer>,
    pub rivers: Option<&'a Rivers>,
    pub standing: Option<&'a Value>,
    /// The world's noise (the detail term's seed, the world's period and edges); none: no texture.
    pub noise: Option<deeptime_format::Noise>,
    pub sea_y: f64,
    /// The planet's radius, m.
    pub planet_r_m: f64,
    /// Metres per block, vertically.
    pub m_per_block_v: f64,
}

impl<M: SimFrame> OnWorld<'_, M> {
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
        let cube = self.grid.is_cube();
        let river = |x: f64, z: f64| match self.rivers {
            Some(r) => {
                let (xs, zs) = map.sim_block(map.unit(x, z));
                let s = r.sample(xs.round() as i64, zs.round() as i64);
                s.hit.is_some() || s.bank_cap.is_some()
            }
            None => false,
        };
        // A cube's relief passes net points; its layers read blocks (a block's centre is +½).
        let ice = |x: f64, z: f64| {
            self.ice.map_or(0.0, |l| {
                if cube {
                    l.cover(x - 0.5, z - 0.5)
                } else {
                    l.cover(x, z)
                }
            })
        };
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
            .map(|v| cones_of(v, |x, z| map.sim_unit(x, z)))
            .unwrap_or_default();
        let (calm, cones_note) = cone_gullies(&mut e, &ground, map, self.planet_r_m, &cones);
        let gullies_ms = t.elapsed().as_secs_f64() * 1000.0;
        crate::surface::amplify::amp_probe_seg("relief_gullies");
        let texture_note = match (self.noise.as_ref(), self.class_top) {
            (Some(noise), Some(class)) => {
                let weights = rock_weights();
                // The rock's weights, bilinear over the four L2 cells round the point.
                let rock_at = |x: f64, z: f64| -> (f64, f64) {
                    let gd = &class.desc.grid;
                    // On a cube: the four cells round the point on its face, across its seams.
                    let (face, u, v) = if gd.is_cube() {
                        match gd.cube_spot(x, z) {
                            Some(sp) => (Some(sp.face), sp.a, sp.b),
                            None => return (0.0, 0.0),
                        }
                    } else {
                        (
                            None,
                            (x - gd.origin_block_x as f64) / f64::from(gd.r),
                            (z - gd.origin_block_z as f64) / f64::from(gd.r),
                        )
                    };
                    let (fi, fj) = (u.floor(), v.floor());
                    let wts = bilinear_weights(u - fi, v - fj);
                    let (i0, j0) = (fi as i64, fj as i64);
                    let mut acc = (0.0, 0.0);
                    for (k, (di, dj)) in [(0, 0), (1, 0), (0, 1), (1, 1)].into_iter().enumerate() {
                        let raw = match face {
                            Some(f) => {
                                let (ci, cj) = gd.face_cell(f, i0 + di, j0 + dj);
                                class.raw_cell(ci, cj)
                            }
                            None => class.raw(i0 + di, j0 + dj),
                        };
                        let (hw, lava) = weights[if raw.is_finite() {
                            (raw.max(0.0) as usize).min(255)
                        } else {
                            255
                        }];
                        acc.0 += wts[k] * f64::from(hw);
                        acc.1 += wts[k] * f64::from(lava);
                    }
                    acc
                };
                let rock = Rock {
                    noise,
                    at: &rock_at,
                    m_per_block_v: self.m_per_block_v,
                    seams: cube.then(|| self.grid.cube()),
                };
                let mut note = steep_texture(&mut e, &ground, &rock, &calm);
                note["millis"] = json!(t.elapsed().as_secs_f64() * 1000.0 - gullies_ms);
                note
            }
            _ => json!(null),
        };
        json!({
            "cone_gullies": cones_note,
            "steep_texture": texture_note,
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
            strato: true,
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
    fn a_shield_is_not_gullied_but_calms_the_ground_over_it() {
        let (grid, frame, rm) = flat();
        let v = (0.0, 0.0);
        let (desc, bytes) = cone_world(&grid, &frame, rm, v, FOOT);
        let g = ground(&grid);
        let k = Cone {
            strato: false,
            ..cone(&frame, v, 2.0, None)
        };
        let e = Elev {
            desc: &desc,
            bytes: &mut bytes.clone(),
        };
        let (cuts, calm, rec) = one_cone(&e, &g, &frame, rm, &k);
        assert_eq!(rec.skipped, Some("not_strato"));
        assert!(cuts.is_empty());
        assert!(calm.len() > 1000 && calm.iter().any(|&(_, v)| v > 0.99));
    }

    // --- steep-ground texture ---

    const SEED: u64 = 20_261_022;

    /// The texture on `grid` with rock `rock_at` and calm `calm`; returns the elevation after and
    /// the note.
    fn textured(
        g: &Ground,
        noise: &deeptime_format::Noise,
        rock_at: &(dyn Fn(f64, f64) -> (f64, f64) + Sync),
        calm: &[u8],
        desc: &LayerDesc,
        bytes: &[u8],
    ) -> (Vec<u8>, Value) {
        let mut after = bytes.to_vec();
        let mut e = Elev {
            desc,
            bytes: &mut after,
        };
        let rock = Rock {
            noise,
            at: rock_at,
            m_per_block_v: 35.0,
            seams: None,
        };
        let note = steep_texture(&mut e, g, &rock, calm);
        (after, note)
    }

    fn cell_xz(grid: &GridDesc, c: usize) -> (f64, f64) {
        let w = grid.w as usize;
        (
            (grid.origin_block_x + (c % w) as i64 * i64::from(grid.r)) as f64,
            (grid.origin_block_z + (c / w) as i64 * i64::from(grid.r)) as f64,
        )
    }

    #[test]
    fn the_rocks_take_the_jag_by_hardness_and_lava_its_bands() {
        let w = rock_weights();
        let of = |name: &str| w[crate::surface::lithology::index(name) as usize];
        assert_eq!(of("granite"), (1.0, 0.0));
        assert_eq!(of("quartzite"), (1.0, 0.0));
        assert_eq!(of("basalt"), (1.0, 1.0));
        assert_eq!(of("andesite").1, 1.0);
        assert_eq!(of("rhyolite").1, 1.0);
        assert_eq!(of("tuff").1, 0.0, "tuff is not lava");
        assert_eq!(of("scoria").1, 0.0, "scoria is not lava");
        assert_eq!(of("shale"), (0.0, 0.0));
        assert_eq!(of("mudstone"), (0.0, 0.0));
        let sandstone = of("sandstone").0;
        assert!(sandstone > 0.4 && sandstone < 0.8, "{sandstone}");
        assert_eq!(w[255], (0.0, 0.0));
    }

    #[test]
    fn the_jag_is_ridged_and_centred() {
        let noise = deeptime_format::Noise::new(SEED, 4096);
        let spec = deeptime_format::DetailSpec::new(0, JAG_SALT, JAG_OCTAVES, JAG_WAVELENGTH);
        let (mut sum, mut lo, mut hi, mut n) = (0.0, f64::INFINITY, f64::NEG_INFINITY, 0);
        for i in 0..400 {
            for j in 0..400 {
                let v = jag_noise(&spec, &noise, f64::from(i) * 3.7, f64::from(j) * 2.9);
                sum += v;
                lo = lo.min(v);
                hi = hi.max(v);
                n += 1;
            }
        }
        let mean = sum / f64::from(n);
        assert!(mean.abs() < 0.02, "the jag's mean is {mean}");
        // Ridged: sharp crests well over the mean, broad troughs not as far under it.
        assert!(hi > 0.3 && lo > -JAG_MEAN - 1e-9, "{lo} {hi}");
    }

    #[test]
    fn steep_hard_rock_is_jagged_and_soft_or_gentle_ground_is_not() {
        let (grid, _, _) = flat();
        let noise = deeptime_format::Noise::new(SEED, grid.w * grid.r);
        let g = ground(&grid);
        // Granite west of x = 0, shale east; steep (1.5 blocks per block) in the south, gentle
        // (0.2) in the north.
        let rock = |x: f64, _: f64| if x < 0.0 { (1.0, 0.0) } else { (0.0, 0.0) };
        let (desc, bytes) = elevation(&grid, |_, z| {
            if z >= 0.0 {
                400.0 + 1.5 * z
            } else {
                400.0 + 0.2 * z
            }
        });
        let (after, note) = textured(&g, &noise, &rock, &[], &desc, &bytes);
        let (mut steep_hard, mut sum, mut sum2) = (0usize, 0.0, 0.0);
        let mut big = 0usize;
        for c in 0..grid.cells() {
            let (x, z) = cell_xz(&grid, c);
            let d = read(&desc, &after, c) - read(&desc, &bytes, c);
            if x >= 0.0 || z < -20.0 {
                assert_eq!(d, 0.0, "soft or gentle ground moved at ({x}, {z}): {d}");
                continue;
            }
            // Away from the slope's break at z = 0 and the grid's end.
            if !(20.0..180.0).contains(&z) {
                continue;
            }
            steep_hard += 1;
            sum += d;
            sum2 += d * d;
            big += usize::from(d.abs() > 1.0);
        }
        let mean = sum / steep_hard as f64;
        let sd = (sum2 / steep_hard as f64 - mean * mean).sqrt();
        assert!(
            mean.abs() < 0.15,
            "the jag shifts the ground by {mean} on average"
        );
        assert!(sd > 0.4, "the jag's spread is {sd} blocks");
        assert!(
            big * 20 > steep_hard,
            "{big} of {steep_hard} cells move over a block"
        );
        assert!(note["cells_jagged"].as_u64().unwrap() > 0, "{note}");
        assert_eq!(note["cells_banded"], json!(0));
        assert!(note["max_cut_blocks"].as_f64().unwrap() <= JAG_MAX_BLOCKS * JAG_MEAN + 1e-9);
    }

    #[test]
    fn steep_lava_gets_benches_and_cliffs_and_is_only_raised() {
        let (grid, _, _) = flat();
        let noise = deeptime_format::Noise::new(SEED, grid.w * grid.r);
        let g = ground(&grid);
        // Lava that takes no jag, on a slope of 0.5 blocks per block: bands 12 blocks high (24
        // blocks of run; four times the thinnest, 105 m at 35 m a block).
        let rock = |_: f64, _: f64| (0.0, 1.0);
        let (desc, bytes) = elevation(&grid, |_, z| 300.0 + 0.5 * z);
        let (after, note) = textured(&g, &noise, &rock, &[], &desc, &bytes);
        let w = grid.w as usize;
        let (mut bench, mut cliff, mut steps) = (0usize, 0usize, 0usize);
        for c in 0..grid.cells() {
            let d = read(&desc, &after, c) - read(&desc, &bytes, c);
            assert!(d >= 0.0, "the bands only raise: cell {c} by {d}");
            // At most 0.709 of a band (the cliff's smoothstep runs a little ahead of the slope).
            assert!(d <= 0.709 * 12.0 + 0.1, "cell {c}: {d}");
            let j = c / w;
            if (40..160).contains(&j) {
                // The rise to the next cell down the slope (2 blocks; 1 block before).
                let rise = read(&desc, &after, c + w) - read(&desc, &after, c);
                steps += 1;
                bench += usize::from(rise < 0.25);
                cliff += usize::from(rise > 2.0);
            }
        }
        let (b, k) = (bench as f64 / steps as f64, cliff as f64 / steps as f64);
        assert!(b > 0.4, "benches: {b} of the steps");
        assert!(k > 0.1, "cliffs: {k} of the steps");
        assert!(note["cells_banded"].as_u64().unwrap() > 0, "{note}");
        assert_eq!(note["cells_lowered"], json!(0));
    }

    #[test]
    fn the_texture_stays_off_the_sea_ice_lakes_rivers_and_calm_ground() {
        let (grid, _, _) = flat();
        let noise = deeptime_format::Noise::new(SEED, grid.w * grid.r);
        let w = grid.w as usize;
        let ox = grid.origin_block_x as f64;
        // Steep basalt rising out of the sea; ice in one quarter, a lake in another, a river in a
        // third, the calm of a young cone in the last, each 200 blocks wide.
        let quarter = |x: f64| ((x - ox) / 1024.0).floor() as i64;
        let lake = |c: usize| quarter(ox + (c % w) as f64 * 2.0) == 1 && (c % w) % 512 < 100;
        let river = |x: f64, _: f64| quarter(x) == 2 && (x - ox) % 1024.0 < 200.0;
        let ice = |x: f64, _: f64| {
            if quarter(x) == 0 && (x - ox) % 1024.0 < 200.0 {
                10.0
            } else {
                0.0
            }
        };
        let g = Ground {
            grid: &grid,
            sea_y: SEA,
            lake: &lake,
            river: &river,
            ice: &ice,
        };
        let calm: Vec<u8> = (0..grid.cells())
            .map(|c| {
                let x = ox + (c % w) as f64 * 2.0;
                if quarter(x) == 3 && (x - ox) % 1024.0 < 200.0 {
                    255
                } else {
                    0
                }
            })
            .collect();
        let rock = |_: f64, _: f64| (1.0, 1.0);
        let (desc, bytes) = elevation(&grid, |_, z| SEA - 30.0 + 1.5 * (z + 100.0));
        let (after, _) = textured(&g, &noise, &rock, &calm, &desc, &bytes);
        let mut moved = 0usize;
        for (c, &calm_c) in calm.iter().enumerate() {
            let (x, z) = cell_xz(&grid, c);
            let (b, a) = (read(&desc, &bytes, c), read(&desc, &after, c));
            let held = b < SEA + 1.0
                || lake(c)
                || river(x, z)
                || ice(x, z) > ICE_FADE_BLOCKS
                || calm_c == 255;
            if held {
                assert_eq!(a, b, "cell {c} at ({x}, {z}) moved");
            } else {
                assert!(a >= SEA + 1.0 - 1e-9, "cell {c}: {b} to {a}");
                moved += usize::from(a != b);
            }
        }
        assert!(moved > 10_000, "{moved} cells textured");
    }

    #[test]
    fn the_texture_keeps_a_petroff_guyou_edge_glued_and_runs_alike_twice() {
        let c_sim = 4096;
        let pg = PetroffGuyou::new(PetroffGuyou::hemisphere_for(c_sim), 30.0);
        let map = Mapping::new(pg, c_sim, 16);
        let grid = pg.grid("L3", 2).unwrap();
        let noise = deeptime_format::Noise::for_world(
            SEED,
            pg.period() as u32,
            &deeptime_format::projection::WorldProjection::PetroffGuyou(pg),
        );
        // Steep ground everywhere, a function of the sphere point so the edges start glued.
        let (desc, bytes) = elevation(&grid, |x, z| {
            let v = Frame::unit(&map, x, z);
            600.0 + 1200.0 * v[0] * v[1] + 300.0 * v[2]
        });
        let g = ground(&grid);
        let rock = |_: f64, _: f64| (1.0, 1.0);
        let (after, _) = textured(&g, &noise, &rock, &[], &desc, &bytes);
        let w = grid.w as usize;
        let h = grid.h as usize;
        let mut moved = 0;
        for j in [0, h - 1] {
            for i in 1..w {
                let (a, m) = (j * w + i, j * w + (w - i));
                assert!(
                    (read(&desc, &after, a) - read(&desc, &after, m)).abs() <= 1.0 / 16.0 + 1e-9,
                    "edge cells {a} and {m}"
                );
                moved += usize::from(read(&desc, &after, a) != read(&desc, &bytes, a));
            }
        }
        assert!(moved > 100, "{moved} edge cells textured");
        let (again, _) = textured(&g, &noise, &rock, &[], &desc, &bytes);
        assert_eq!(after, again);
    }

    #[test]
    fn standing_volcanoes_give_their_cones() {
        let v = json!({"version": 1, "t_myr": 500.0, "fade_myr": 5.0, "volcanoes": [
            {"id": 3, "kind": "stratovolcano", "x": 10.0, "z": -20.0, "radius_m": 14000.0,
             "height_m": 2400.0, "crater_rho": 0.07, "born_myr": 498.0, "extinct_myr": null},
            {"id": 9, "kind": "stratovolcano", "x": 50.0, "z": 5.0, "radius_m": 9000.0,
             "height_m": 900.0, "crater_rho": null, "born_myr": 480.0, "extinct_myr": 495.0},
            {"id": 12, "kind": "shield", "x": -40.0, "z": 5.0, "radius_m": 60000.0,
             "height_m": 4000.0, "crater_rho": 0.05, "born_myr": 499.0, "extinct_myr": null}]});
        let cones = cones_of(&v, |x, z| [x, z, 0.0]);
        assert_eq!(cones.len(), 3);
        assert!(cones[0].strato && cones[1].strato && !cones[2].strato);
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
