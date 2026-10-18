// SPDX-License-Identifier: LGPL-3.0-or-later
//! Geometry (plan §8.6 step 5): a deposit's bodies from its tonnage, the type's shape, aspect and
//! zoning, and the balance multipliers.
//!
//! Volume V = tonnage / ore density; the semi-axes in metres are the type's `aspect` scaled so
//! the primitive holds V (ellipsoid 4/3·π·abc, cylinder π·ac·2b). In blocks, horizontal axes are
//! divided by the horizontal scale (300 m) and multiplied by `size_mult_h`, the vertical one by
//! the vertical scale (`scale.m_per_block_v`, 35 m by default) and `size_mult_v`: a median porphyry (240 Mt / 2.6 t/m³ ≈ 9.2e7 m³)
//! is a few hundred metres across, about 1.5 blocks wide before the multiplier (plan §8.6), so
//! the horizontal multiplier does most of the work.

use std::collections::BTreeMap;

use deeptime_format::SiteId;
use deeptime_format::bodies::{Body, NO_CLIP, Shape, Zone};

use super::random::Rng;
use super::types::{DepositType, ExtraBody, Stack, When, ZoneDef};

/// World scales the geometry needs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scales {
    pub m_per_block_h: f64,
    pub m_per_block_v: f64,
    pub min_y: i32,
    pub max_y: i32,
    /// `scale.sea_level_y`: the block y of elevation 0.
    pub sea_level_y: i32,
}

impl Default for Scales {
    fn default() -> Self {
        Scales {
            m_per_block_h: 300.0,
            m_per_block_v: deeptime_format::DEFAULT_M_PER_BLOCK_V,
            min_y: -512,
            max_y: 639,
            sea_level_y: 63,
        }
    }
}

/// Semi-axes in metres of a primitive of `shape` holding `volume_m3` with relative `aspect`.
pub fn semi_axes_m(shape: Shape, aspect: [f64; 3], volume_m3: f64) -> [f64; 3] {
    let unit = match shape.metric() {
        deeptime_format::bodies::Metric::Ellipsoid => {
            4.0 / 3.0 * std::f64::consts::PI * aspect[0] * aspect[1] * aspect[2]
        }
        deeptime_format::bodies::Metric::Cylinder => {
            std::f64::consts::PI * aspect[0] * aspect[2] * 2.0 * aspect[1]
        }
    };
    let s = (volume_m3.max(0.0) / unit).cbrt();
    aspect.map(|a| a * s)
}

/// Semi-axes in blocks after the balance multipliers (at least half a block each).
pub fn to_blocks(semi_m: [f64; 3], t: &DepositType, sc: &Scales) -> [f32; 3] {
    let h = t.balance.size_mult_h / sc.m_per_block_h;
    let v = t.balance.size_mult_v / sc.m_per_block_v;
    [
        (semi_m[0] * h).max(0.5) as f32,
        (semi_m[1] * v).max(0.5) as f32,
        (semi_m[2] * h).max(0.5) as f32,
    ]
}

/// Grade of `material` at this site as a mass fraction (0 when absent).
pub type Grades = BTreeMap<String, f64>;

/// A zone's ore density never exceeds this, however rich the deposit: a tenth of the host stays
/// rock, so a body still reads as ore in rock (vanilla's ore veins leave filler between their ore).
/// game-choice (deposit richness, Ben 2026-10-02 "Rich Deposits, Thinner Scatter"). ledger: ORE-262
pub const DENSITY_CAP: f64 = 0.9;

/// A zone's raw-ore share never exceeds this. game-choice (as [`DENSITY_CAP`]). ledger: ORE-262
pub const RAW_CAP: f64 = 0.5;

/// A body zone from the type's zone: the site's grade × the zone's `grade_mult` × `richness`
/// (at most 1); the ore density and the raw-ore share × `richness` (at most [`DENSITY_CAP`] and
/// [`RAW_CAP`]), so the richness settings reach the blocks (Ben, 2026-10-02: before, `richness`
/// changed the listed grade only). ledger: ORE-262
pub(crate) fn zone(def: &ZoneDef, grades: &Grades, richness: f64) -> Zone {
    let grade = def
        .material
        .as_ref()
        .and_then(|m| grades.get(m))
        .map_or(0.0, |g| (g * def.grade_mult * richness).min(1.0));
    let present = def.material.as_ref().is_none_or(|m| grades.contains_key(m));
    Zone {
        name: def.zone.clone(),
        material: if present { def.material.clone() } else { None },
        rock_class: def.rock_class.clone(),
        d: def.d,
        v: def.v,
        grade: grade as f32,
        density: if present {
            (def.density * richness).min(DENSITY_CAP.max(def.density)) as f32
        } else {
            0.0
        },
        raw: if present {
            (def.raw_share * richness).min(RAW_CAP.max(def.raw_share)) as f32
        } else {
            0.0
        },
    }
}

/// A leached cap reaches down to the water table and no further than this (m).
// ledger: DEP-209 (SIR 2010-5070-B p. 65: 0 to several hundred m; Cook and Porter 2005: Inspiration
// 0 to 300 m).
pub const LEACHED_CAP_MAX_M: f64 = 300.0;

/// No exposed iron cap is thinner than this many blocks at the vertical scale (a one-block sheet
/// is skipped by the soil rule and reads as a stain, not an outcrop).
// ledger: DEP-210 (game-choice, the coordinator's call 2026-10-17)
pub const GOSSAN_MIN_BLOCKS: f64 = 2.0;

/// Where the ore body sits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub x: f64,
    pub z: f64,
    /// Present surface at the site (block y).
    pub surface_y: f64,
    /// Present depth of the ore body's centre below the surface (m; negative = above it).
    pub centre_depth_m: f64,
    pub yaw: f32,
    pub dip: f32,
    /// Whether erosion has cut the body (it is then clipped at the surface).
    pub truncated: bool,
    /// Depth of the present water table below the surface at the site (m; 0 = not known: a body
    /// that follows the water table then keeps its fixed thickness).
    pub water_table_m: f64,
}

/// The ore body and its extra bodies (lithocap, gossan) for one site. Ids come from `next_id`.
pub fn bodies_for(
    t: &DepositType,
    site: SiteId,
    tonnage_t: f64,
    grades: &Grades,
    at: &Placement,
    sc: &Scales,
    next_id: &mut dyn FnMut() -> u32,
) -> Vec<Body> {
    let shape = t.shape();
    let volume = tonnage_t / t.sizing.ore_density_t_m3;
    let semi_m = semi_axes_m(shape, t.geometry.aspect, volume);
    let axes = to_blocks(semi_m, t, sc);
    let centre_y = at.surface_y - at.centre_depth_m / sc.m_per_block_v;
    let surface_clip = at.surface_y.floor() as i32;
    let id = next_id();
    let body = |clip: i32| {
        Body::new(
            id,
            Some(site),
            shape,
            "ore",
            [at.x, centre_y, at.z],
            axes,
            at.yaw,
            at.dip,
            clip,
        )
    };
    let mut ore = if at.truncated {
        body(surface_clip)
    } else {
        // Buried by the ledger, in true metres; drawn taller (`size_mult_v`), its top can still
        // reach the ground: it stays under it, a block of cover over it (2026-10-05: a shallow
        // sandstone-uranium body broke through a salt flat on the test planet).
        let b = body(NO_CLIP);
        if b.aabb[1][1] >= surface_clip {
            body(surface_clip - 1)
        } else {
            b
        }
    };
    ore.rock_class = t.geometry.rock_class.clone();
    ore.zones = t
        .geometry
        .zoning
        .iter()
        .map(|z| zone(z, grades, t.balance.richness_mult))
        .collect();
    let top_y = centre_y + f64::from(axes[1]);
    let mut out = vec![ore];
    for e in &t.geometry.extra_bodies {
        let exposed = at.truncated || top_y >= at.surface_y - 1.0;
        let want = match e.when {
            When::Preserved => !exposed,
            When::Exposed => exposed,
        };
        if want {
            out.push(extra(e, site, &axes, at, top_y, sc, next_id(), grades, t));
        }
    }
    out
}

/// Half the vertical thickness of an extra body in blocks. A body that follows the water table is
/// as thick as the table is deep (the leached cap above an enrichment blanket, which forms at the
/// top of the water table), at most [`LEACHED_CAP_MAX_M`]; an exposed one is never under
/// [`GOSSAN_MIN_BLOCKS`] thick.
fn extra_half_blocks(e: &ExtraBody, at: &Placement, sc: &Scales) -> f64 {
    let thickness_m = if e.thickness_to_water_table && at.water_table_m > 0.0 {
        at.water_table_m.min(LEACHED_CAP_MAX_M).max(e.thickness_m)
    } else {
        e.thickness_m
    };
    let min_half = match e.when {
        When::Exposed => GOSSAN_MIN_BLOCKS / 2.0,
        When::Preserved => 0.5,
    };
    (thickness_m / sc.m_per_block_v / 2.0).max(min_half)
}

#[allow(clippy::too_many_arguments)]
fn extra(
    e: &ExtraBody,
    site: SiteId,
    ore_axes: &[f32; 3],
    at: &Placement,
    ore_top_y: f64,
    sc: &Scales,
    id: u32,
    grades: &Grades,
    t: &DepositType,
) -> Body {
    let shape = Shape::from_name(&e.shape).expect("validated");
    let half = extra_half_blocks(e, at, sc);
    let (centre_y, clip) = match e.when {
        // A lithocap sits on the ore body's top, clipped at the surface.
        When::Preserved => (ore_top_y + half, at.surface_y.floor() as i32),
        // A gossan is a skin at the surface over the exposed body.
        When::Exposed => (at.surface_y - half, at.surface_y.floor() as i32),
    };
    let mut b = Body::new(
        id,
        Some(site),
        shape,
        &e.role,
        [at.x, centre_y, at.z],
        [
            (f64::from(ore_axes[0]) * e.scale_h) as f32,
            half as f32,
            (f64::from(ore_axes[2]) * e.scale_h) as f32,
        ],
        at.yaw,
        0.0,
        clip,
    );
    b.rock_class = e.rock_class.clone();
    b.zones = e
        .zones
        .iter()
        .map(|z| zone(z, grades, t.balance.richness_mult))
        .collect();
    b
}

/// The coal measures under a site, from its column's history (`geometry.stack`): the sediment
/// laid down while the cell was coal-forming, Σ_k F_k · deposition_k · Δt_k over the frames k of
/// the crust's life, in metres. F_k is the cell's favourability in frame k, recovered from the
/// hazard h_k = −(Δt/τ)·ln(1 − F_k) the placement was drawn from. `hazard` is per frame of the
/// history; `col` holds the frames the crust existed in, in order (`t_years` of each frame in
/// `frame_years`, the same values).
pub fn measures_thickness_m(
    hazard: &[f32],
    tau_myr: f64,
    frame_years: &[f64],
    col: &[super::children::ColumnSample],
) -> f64 {
    let mut total = 0.0;
    let mut s = col.iter().peekable();
    for (k, h) in hazard.iter().enumerate() {
        let Some(&t) = frame_years.get(k) else { break };
        while s.peek().is_some_and(|c| c.t_years < t) {
            s.next();
        }
        let Some(c) = s.peek().filter(|c| c.t_years == t) else {
            continue;
        };
        if !h.is_finite() || *h <= 0.0 {
            continue;
        }
        let dt_myr = c.dt_years / 1.0e6;
        let f = 1.0 - (-f64::from(*h) * tau_myr / dt_myr).exp();
        total += f * c.deposition_m_myr.max(0.0) * dt_myr;
    }
    total
}

/// The seams of one stacked deposit, top seam first.
#[derive(Clone, Debug, PartialEq)]
pub struct SeamPlan {
    /// The coal measures' thickness the count came from (m).
    pub measures_m: f64,
    /// Each seam's share of the tonnage (sums to 1).
    pub shares: Vec<f64>,
    /// The real gaps between neighbouring seams (m), one fewer than seams; their mean is the
    /// measures' thickness per seam.
    pub gaps_m: Vec<f64>,
}

impl SeamPlan {
    pub fn seams(&self) -> usize {
        self.shares.len()
    }
}

/// The seams a stack of `measures_m` metres of coal measures holds: n = round(measures /
/// `seam_spacing_m`) clamped to the type's range and to `cumulative_m` / `min_seam_m` (the
/// field's total seam thickness, m), shares and gaps drawn from `rng` (a fixed
/// number of draws per seam, so the plan is a pure function of the stream).
pub fn seam_plan(k: &Stack, measures_m: f64, cumulative_m: f64, rng: &mut Rng) -> SeamPlan {
    let mut n = ((measures_m / k.seam_spacing_m).round().max(0.0) as u64)
        .clamp(u64::from(k.min_seams), u64::from(k.max_seams));
    if k.min_seam_m > 0.0 {
        // Thinner seams than the minimum are not minable: fewer, thicker seams instead.
        let cap = (cumulative_m / k.min_seam_m).floor().max(0.0) as u64;
        n = n.min(cap.max(u64::from(k.min_seams)));
    }
    let n = n as usize;
    let mut shares: Vec<f64> = (0..n)
        .map(|_| (k.share_sigma * rng.normal()).exp())
        .collect();
    let sum: f64 = shares.iter().sum();
    shares.iter_mut().for_each(|s| *s /= sum);
    let mut gaps: Vec<f64> = (0..n.saturating_sub(1))
        .map(|_| (k.gap_sigma * rng.normal()).exp())
        .collect();
    let mean = if gaps.is_empty() {
        1.0
    } else {
        gaps.iter().sum::<f64>() / gaps.len() as f64
    };
    let per_seam = measures_m.max(0.0) / n as f64;
    gaps.iter_mut().for_each(|g| *g *= per_seam / mean);
    SeamPlan {
        measures_m,
        shares,
        gaps_m: gaps,
    }
}

/// How a stack fits the sedimentary cover of its site ([`fit_stack`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StackFit {
    /// Inside the cover as planned.
    Fits,
    /// No cover to hold a seam: no coal here.
    NoCover,
    /// Smaller (measures capped to what the cover holds) and/or raised (its centre depth).
    Capped {
        measures_m: f64,
        centre_depth_m: f64,
    },
}

/// Blocks of cover a stack keeps free under its lowest seam.
pub const COVER_MARGIN_BLOCKS: f64 = 1.0;

/// Fits a stack of `plan` centred `centre_depth_m` below the surface into `cover_blocks` of
/// sedimentary cover (the coal-measure sediment it must lie in; [`stack_bodies`] draws the
/// seams from half the measures above the centre downwards, the gaps enlarged by
/// `spacing_mult_v`). The stack keeps the depth the preservation ledger gave its top seam and is
/// capped from below: the seams that would lie in the rock under the cover (within
/// [`COVER_MARGIN_BLOCKS`] of its base) are not there, so thin cover leaves fewer, shallower
/// seams; a top seam already under the cover's base, or less than [`COVER_MARGIN_BLOCKS`] plus
/// one block of cover, leaves no coal.
pub fn fit_stack(
    k: &Stack,
    plan: &SeamPlan,
    centre_depth_m: f64,
    cover_blocks: f64,
    sc: &Scales,
) -> StackFit {
    let room = cover_blocks - COVER_MARGIN_BLOCKS;
    if !room.is_finite() || room < 1.0 {
        return StackFit::NoCover;
    }
    let mult = k.spacing_mult_v.max(1e-9);
    // The top seam's depth in blocks below the surface; the stack stands `measures · mult /
    // m_per_block_v` blocks from it down to its lowest seam.
    let top = (centre_depth_m - plan.measures_m / 2.0) / sc.m_per_block_v;
    if top >= room {
        return StackFit::NoCover;
    }
    let fits_m = (room - top) * sc.m_per_block_v / mult;
    if plan.measures_m <= fits_m {
        return StackFit::Fits;
    }
    StackFit::Capped {
        measures_m: fits_m,
        // The same top seam.
        centre_depth_m: top * sc.m_per_block_v + fits_m / 2.0,
    }
}

/// A stack's bodies and how much of its tonnage is left in them.
pub struct StackBodies {
    pub bodies: Vec<Body>,
    /// Σ share × (the part of the seam below the surface): the stack's remaining fraction of
    /// its tonnage.
    pub kept: f64,
}

/// The bodies of a stacked deposit (`geometry.stack`): one `ore` body per seam, ids consecutive
/// from `next_id`, top seam first. The ore body of `tonnage_t` is sized as usual (its footprint
/// is the field, its thickness the field's cumulative seam thickness); each seam keeps that
/// footprint and takes `share` of the thickness and of the tonnage. The stack's centre is at
/// `at.centre_depth_m`, so its top seam lies half the measures above it; the gaps between seams
/// are enlarged by `spacing_mult_v` downwards from the top seam. A seam above the ground is
/// eroded away (dropped), one crossing it is clipped at the surface, one below the world's floor
/// is dropped.
#[allow(clippy::too_many_arguments)]
pub fn stack_bodies(
    t: &DepositType,
    k: &Stack,
    site: SiteId,
    tonnage_t: f64,
    grades: &Grades,
    at: &Placement,
    plan: &SeamPlan,
    sc: &Scales,
    next_id: &mut dyn FnMut() -> u32,
) -> StackBodies {
    let shape = t.shape();
    let volume = tonnage_t / t.sizing.ore_density_t_m3;
    let semi_m = semi_axes_m(shape, t.geometry.aspect, volume);
    let h = t.balance.size_mult_h / sc.m_per_block_h;
    let v = t.balance.size_mult_v / sc.m_per_block_v;
    let (a, c) = (
        (semi_m[0] * h).max(0.5) as f32,
        (semi_m[2] * h).max(0.5) as f32,
    );
    let top_y = at.surface_y - (at.centre_depth_m - plan.measures_m / 2.0) / sc.m_per_block_v;
    let clip = at.surface_y.floor() as i32;
    let mut bodies = Vec::new();
    let mut kept = 0.0;
    let mut below = 0.0;
    for (i, share) in plan.shares.iter().enumerate() {
        if i > 0 {
            below += plan.gaps_m[i - 1] * k.spacing_mult_v / sc.m_per_block_v;
        }
        let axes = [a, (semi_m[1] * share * v).max(0.5) as f32, c];
        let centre = [at.x, top_y - below, at.z];
        let mut body = Body::new(
            0,
            Some(site),
            shape,
            "ore",
            centre,
            axes,
            at.yaw,
            at.dip,
            NO_CLIP,
        );
        let [lo, hi] = body.aabb;
        if hi[1] < sc.min_y {
            continue;
        }
        let mut frac = 1.0;
        if lo[1] > clip {
            continue;
        }
        if hi[1] > clip {
            frac = (f64::from(clip - lo[1]) / f64::from((hi[1] - lo[1]).max(1))).clamp(0.0, 1.0);
            body = Body::new(
                0,
                Some(site),
                shape,
                "ore",
                centre,
                axes,
                at.yaw,
                at.dip,
                clip,
            );
        }
        body.id = next_id();
        body.rock_class = t.geometry.rock_class.clone();
        body.zones = t
            .geometry
            .zoning
            .iter()
            .map(|z| zone(z, grades, t.balance.richness_mult))
            .collect();
        kept += share * frac;
        bodies.push(body);
    }
    StackBodies { bodies, kept }
}

/// The vertical half-height in metres of the ore body (before size_mult_v), for the ledger.
pub fn half_height_m(t: &DepositType, tonnage_t: f64) -> f64 {
    semi_axes_m(
        t.shape(),
        t.geometry.aspect,
        tonnage_t / t.sizing.ore_density_t_m3,
    )[1]
}

/// The plan footprint's semi-axes `[a, c]` in blocks, for adjoining placement.
pub fn footprint_blocks(t: &DepositType, tonnage_t: f64, sc: &Scales) -> [f64; 2] {
    let semi = semi_axes_m(
        t.shape(),
        t.geometry.aspect,
        tonnage_t / t.sizing.ore_density_t_m3,
    );
    let b = to_blocks(semi, t, sc);
    [f64::from(b[0]), f64::from(b[2])]
}

/// The horizontal radius in blocks (the larger horizontal semi-axis), for separation.
pub fn radius_blocks(t: &DepositType, tonnage_t: f64, sc: &Scales) -> f64 {
    let semi = semi_axes_m(
        t.shape(),
        t.geometry.aspect,
        tonnage_t / t.sizing.ore_density_t_m3,
    );
    let b = to_blocks(semi, t, sc);
    f64::from(b[0].max(b[2]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deposits::types::builtin;

    /// The deposit audit (2026-10-01) and WP-9/10 found the richness settings reached no block:
    /// only the listed grade. Now a body zone's ore density and raw-ore share follow the richness
    /// (Ben, 2026-10-02), up to their caps, and the bodies of a site carry them.
    #[test]
    fn richness_reaches_the_ore_density_and_the_raw_blocks() {
        let types = builtin().unwrap();
        let t = types
            .iter()
            .find(|t| t.id == "deeptime:porphyry_cu_mo_au")
            .unwrap();
        let core = t
            .geometry
            .zoning
            .iter()
            .find(|z| z.zone == "potassic_core")
            .unwrap();
        assert!(core.raw_share > 0.0, "a rich core carries raw-ore blocks");
        let grades: Grades = [("c:copper".to_owned(), 0.005)].into_iter().collect();
        let at = |r: f64| zone(core, &grades, r);
        assert!((f64::from(at(1.0).density) - core.density).abs() < 1e-6);
        assert!((f64::from(at(1.0).raw) - core.raw_share).abs() < 1e-6);
        assert!(at(1.2).density > at(1.0).density);
        assert!(at(0.5).density < at(1.0).density);
        assert!((f64::from(at(0.5).density) - 0.5 * core.density).abs() < 1e-6);
        assert!(at(2.0).raw > at(1.0).raw);
        // Saturation: never more than the caps.
        assert!((f64::from(at(10.0).density) - DENSITY_CAP).abs() < 1e-6);
        assert!((f64::from(at(100.0).raw) - RAW_CAP).abs() < 1e-6);
        // A zone without its material at the site places nothing, raw or not.
        let none = zone(core, &Grades::new(), 2.0);
        assert_eq!((none.density, none.raw), (0.0, 0.0));
        // Through the bodies of a site.
        let mut id = 0;
        let mut next = || {
            id += 1;
            id
        };
        let place = Placement {
            x: 0.0,
            z: 0.0,
            surface_y: 80.0,
            centre_depth_m: 600.0,
            yaw: 0.0,
            dip: 0.0,
            truncated: false,
            water_table_m: 0.0,
        };
        let mut rich = t.clone();
        rich.balance.richness_mult *= 1.5;
        let sc = Scales::default();
        let b1 = bodies_for(t, 1, 240.0e6, &grades, &place, &sc, &mut next);
        let b2 = bodies_for(&rich, 1, 240.0e6, &grades, &place, &sc, &mut next);
        let d = |b: &[Body]| {
            b[0].zones
                .iter()
                .find(|z| z.name == "potassic_core")
                .unwrap()
                .density
        };
        assert!(d(&b2) > d(&b1));
    }

    /// Plan §8.6: a median porphyry, 240 Mt at 2.6 t/m³, is a ≈452 m cube: ≈1.5 blocks wide at
    /// 300 m per block and ≈22.6 blocks tall at 20 m per block before multipliers.
    #[test]
    fn median_porphyry_scale() {
        let v: f64 = 240.0e6 / 2.6;
        let cube = v.cbrt();
        assert!((cube - 452.0).abs() < 1.0);
        assert!((cube / 300.0 - 1.5).abs() < 0.02);
        assert!((cube / 20.0 - 22.6).abs() < 0.1);
        let semi = semi_axes_m(Shape::StockworkShell, [1.0, 1.0, 1.0], v);
        let vol = 4.0 / 3.0 * std::f64::consts::PI * semi[0] * semi[1] * semi[2];
        assert!((vol / v - 1.0).abs() < 1e-9);
        let semi = semi_axes_m(Shape::Seam, [1.0, 0.01, 2.0], v);
        let vol = std::f64::consts::PI * semi[0] * semi[2] * 2.0 * semi[1];
        assert!((vol / v - 1.0).abs() < 1e-9);
    }

    #[test]
    fn bodies_hold_the_zones_and_clip() {
        let t = builtin().unwrap().remove(0);
        let grades: Grades = [("c:copper".to_string(), 0.0044)].into_iter().collect();
        let mut id = 10u32;
        let mut next = || {
            id += 1;
            id
        };
        let at = Placement {
            x: 100.0,
            z: -50.0,
            surface_y: 120.0,
            centre_depth_m: 100.0,
            yaw: 0.2,
            dip: 0.0,
            truncated: true,
            water_table_m: 0.0,
        };
        let b = bodies_for(&t, 4, 240.0e6, &grades, &at, &Scales::default(), &mut next);
        let ore = &b[0];
        assert_eq!(ore.role, "ore");
        assert_eq!(ore.site, Some(4));
        assert_eq!(ore.clip_top_y, 120);
        assert!(ore.aabb[1][1] <= 120);
        assert!(
            ore.zones
                .iter()
                .any(|z| z.material.as_deref() == Some("c:copper") && z.grade > 0.0)
        );
        assert!(ore.validate().is_ok());
        assert!(b.iter().all(|x| x.id > 10));
        // Buried by the ledger but drawn taller than it is: kept a block under the ground.
        let shallow = Placement {
            truncated: false,
            centre_depth_m: ore_half_m(&t) * 1.01,
            ..at
        };
        let sc = Scales::default();
        let b = bodies_for(&t, 5, 240.0e6, &grades, &shallow, &sc, &mut next);
        assert_eq!(b[0].clip_top_y, 119);
        assert!(b[0].aabb[1][1] <= 119);
        // Deep: not clipped.
        let deep = Placement {
            truncated: false,
            centre_depth_m: 5000.0,
            ..at
        };
        let b = bodies_for(&t, 6, 240.0e6, &grades, &deep, &sc, &mut next);
        assert_eq!(b[0].clip_top_y, NO_CLIP);
    }

    /// Decisions log 2026-10-17 (gossan): the iron cap of an exposed porphyry is as thick as the
    /// site's water table is deep (the leached cap sits above the enrichment blanket), at most
    /// 300 m, never under two blocks, and its top stays at the surface.
    #[test]
    fn the_gossan_reaches_the_water_table_within_bounds() {
        let t = builtin().unwrap().remove(0);
        assert_eq!(t.id, "deeptime:porphyry_cu_mo_au");
        let grades: Grades = [("c:copper".to_string(), 0.0044)].into_iter().collect();
        let sc = Scales::default();
        let exposed = Placement {
            x: 0.0,
            z: 0.0,
            surface_y: 120.0,
            centre_depth_m: 100.0,
            yaw: 0.0,
            dip: 0.0,
            truncated: true,
            water_table_m: 0.0,
        };
        let gossan_height = |wt: f64| {
            let at = Placement {
                water_table_m: wt,
                ..exposed
            };
            let mut id = 0u32;
            let bodies = bodies_for(&t, 1, 240.0e6, &grades, &at, &sc, &mut || {
                id += 1;
                id
            });
            let g = bodies.iter().find(|b| b.role == "gossan").expect("gossan");
            assert_eq!(g.clip_top_y, 120, "the top stays at the surface");
            assert!(g.validate().is_ok());
            // The body's top (centre plus half-height) is the surface.
            assert!((g.center[1] + f64::from(g.axes[1]) - 120.0).abs() < 1e-4);
            2.0 * f64::from(g.axes[1])
        };
        // Unknown water table: the fixed 40 m, lifted to the two-block floor (70 m).
        assert!((gossan_height(0.0) - 2.0).abs() < 1e-4);
        // A shallow table (humid site, 10 m) is below the floor too.
        assert!((gossan_height(10.0) - 2.0).abs() < 1e-4);
        // A dry site, table 140 m: 4 blocks of 35 m.
        assert!((gossan_height(140.0) - 4.0).abs() < 1e-4);
        // Deeper than a leached cap gets: 300 m, 8.57 blocks.
        let capped = gossan_height(900.0);
        assert!((capped - 300.0 / 35.0).abs() < 1e-3, "{capped}");
        // A buried porphyry has no gossan at all.
        let buried = Placement {
            truncated: false,
            centre_depth_m: 5000.0,
            ..exposed
        };
        let b = bodies_for(&t, 2, 240.0e6, &grades, &buried, &sc, &mut || 99);
        assert!(b.iter().all(|x| x.role != "gossan"));
    }

    /// The true half-height of a 240 Mt body of `t`, m.
    fn ore_half_m(t: &DepositType) -> f64 {
        half_height_m(t, 240.0e6)
    }

    fn coal() -> DepositType {
        builtin()
            .unwrap()
            .into_iter()
            .find(|t| t.id == "deeptime:coal_seam")
            .unwrap()
    }

    /// The measures are Σ F · deposition · Δt over the frames the crust existed in, F recovered
    /// from the hazard the placement was drawn from.
    #[test]
    fn measures_are_favourability_weighted_deposition() {
        use crate::deposits::children::ColumnSample;
        let (tau, dt_myr) = (10.0, 10.0);
        let hazard_of = |f: f64| (-(dt_myr / tau) * (1.0_f64 - f).ln()) as f32;
        let frames: Vec<f64> = (1..=4).map(|k| f64::from(k) * dt_myr * 1.0e6).collect();
        let col: Vec<ColumnSample> = frames
            .iter()
            .enumerate()
            .filter(|(k, _)| *k != 1) // the crust did not exist in frame 1
            .map(|(_, t)| ColumnSample {
                t_years: *t,
                dt_years: dt_myr * 1.0e6,
                erosion_m_myr: 0.0,
                deposition_m_myr: 40.0,
                water_table_m: 0.0,
                precip_mm: 0.0,
            })
            .collect();
        // F = 0.5 in frames 0 and 1 (frame 1 has no crust), 1 - e^-1-ish in frame 2, none in 3.
        let hazard = [hazard_of(0.5), hazard_of(0.5), hazard_of(0.25), 0.0];
        let s = measures_thickness_m(&hazard, tau, &frames, &col);
        assert!((s - (0.5 * 400.0 + 0.25 * 400.0)).abs() < 0.5, "{s}");
        assert_eq!(measures_thickness_m(&[0.0; 4], tau, &frames, &col), 0.0);
    }

    /// The seam count follows the measures, within the type's range and the minable thickness;
    /// the shares sum to one and the gaps average the measures per seam; a fixed seed gives the
    /// same plan.
    #[test]
    fn a_stack_is_held_in_its_sedimentary_cover() {
        let t = coal();
        let k = t.geometry.stack.clone().unwrap();
        // Covers and depths below are in blocks set at 20 m a block (where seed 3's gaps keep
        // every seam apart: under 6 m of real gap, two half-block seams touch at 35 m).
        let sc = Scales {
            m_per_block_v: 20.0,
            ..Scales::default()
        };
        let plan = seam_plan(&k, 300.0, 1000.0, &mut Rng::new(3));
        // 300 m of measures stand spacing_mult_v · 300 / 20 blocks tall.
        let span = 300.0 * k.spacing_mult_v / sc.m_per_block_v;
        let room = 200.0 - COVER_MARGIN_BLOCKS;
        assert!(span < room);
        let top_of =
            |centre_m: f64, measures_m: f64| (centre_m - measures_m / 2.0) / sc.m_per_block_v;
        // A shallow stack in thick cover fits.
        let shallow = 10.0 * sc.m_per_block_v + 150.0;
        assert_eq!(fit_stack(&k, &plan, shallow, 200.0, &sc), StackFit::Fits);
        // A deep one keeps its top seam and loses the seams under the cover's base.
        match fit_stack(&k, &plan, 3000.0, 200.0, &sc) {
            StackFit::Capped {
                measures_m,
                centre_depth_m,
            } => {
                assert!(measures_m < 300.0);
                let top = top_of(centre_depth_m, measures_m);
                assert!(
                    (top - top_of(3000.0, 300.0)).abs() < 1e-9,
                    "the top seam stays"
                );
                let bottom = top + measures_m * k.spacing_mult_v / sc.m_per_block_v;
                assert!((bottom - room).abs() < 1e-9, "{bottom}");
                let fewer = seam_plan(&k, measures_m, 1000.0, &mut Rng::new(3));
                assert!(fewer.seams() < plan.seams());
            }
            other => panic!("{other:?}"),
        }
        // Thin cover under a shallow stack: fewer measures, all inside it.
        match fit_stack(&k, &plan, shallow, 40.0, &sc) {
            StackFit::Capped { measures_m, .. } => assert!(measures_m < 300.0),
            other => panic!("{other:?}"),
        }
        // A top seam under the cover's base, or basement at the surface: no coal.
        assert_eq!(fit_stack(&k, &plan, 5000.0, 200.0, &sc), StackFit::NoCover);
        assert_eq!(fit_stack(&k, &plan, shallow, 1.5, &sc), StackFit::NoCover);
        assert_eq!(fit_stack(&k, &plan, shallow, 0.0, &sc), StackFit::NoCover);
    }

    #[test]
    fn the_stack_follows_the_measures() {
        let t = coal();
        let k = t.geometry.stack.clone().unwrap();
        let plan = |measures: f64, cumulative: f64, seed: u64| {
            seam_plan(&k, measures, cumulative, &mut Rng::new(seed))
        };
        let n = |m: f64| plan(m, 1000.0, 1).seams();
        assert_eq!(n(0.0), 1, "no measures: the minimum, one seam");
        assert_eq!(n(100.0), 4);
        assert_eq!(n(500.0), 20);
        assert_eq!(n(1.0e6), k.max_seams as usize, "clamped at the maximum");
        assert!(
            n(200.0) < n(400.0) && n(400.0) < n(800.0),
            "more measures, more seams"
        );
        // Seams thinner than the minimum are not minable: 5 m of coal holds at most 7.
        assert_eq!(plan(1000.0, 5.0, 1).seams(), 7);
        assert_eq!(plan(1000.0, 0.1, 1).seams(), k.min_seams as usize);
        let p = plan(600.0, 30.0, 7);
        assert_eq!(p.seams(), 24.min((30.0 / k.min_seam_m) as usize));
        assert!((p.shares.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        assert_eq!(p.gaps_m.len(), p.seams() - 1);
        let mean = p.gaps_m.iter().sum::<f64>() / p.gaps_m.len() as f64;
        assert!((mean - 600.0 / p.seams() as f64).abs() < 1e-9, "{mean}");
        assert_eq!(p, plan(600.0, 30.0, 7), "deterministic");
        assert_ne!(p, plan(600.0, 30.0, 8), "the seed matters");
    }

    /// A stack's bodies: one `ore` body per seam, consecutive ids, top first and apart; the
    /// tonnage splits across them (volumes in proportion to the shares); a seam above the ground
    /// is eroded away, one crossing it is clipped, and what is kept follows.
    #[test]
    fn stack_bodies_split_the_tonnage_and_erode_from_the_top() {
        let t = coal();
        let k = t.geometry.stack.clone().unwrap();
        let grades: Grades = [("c:coal".to_string(), 1.0)].into_iter().collect();
        // Covers and depths below are in blocks set at 20 m a block (where seed 3's gaps keep
        // every seam apart: under 6 m of real gap, two half-block seams touch at 35 m).
        let sc = Scales {
            m_per_block_v: 20.0,
            ..Scales::default()
        };
        let plan = seam_plan(&k, 500.0, 30.0, &mut Rng::new(3));
        assert!(plan.seams() >= 5);
        let build = |depth: f64| {
            let mut id = 100u32;
            let at = Placement {
                x: 10.0,
                z: -20.0,
                surface_y: 200.0,
                centre_depth_m: depth,
                yaw: 0.4,
                dip: 0.0,
                truncated: false,
                water_table_m: 0.0,
            };
            stack_bodies(&t, &k, 9, 2.0e10, &grades, &at, &plan, &sc, &mut || {
                id += 1;
                id
            })
        };
        // Deep: every seam kept, nothing clipped.
        let deep = build(900.0);
        assert_eq!(deep.bodies.len(), plan.seams());
        assert!((deep.kept - 1.0).abs() < 1e-9);
        for (i, b) in deep.bodies.iter().enumerate() {
            assert_eq!(
                (b.id, b.site, b.role.as_str()),
                (101 + i as u32, Some(9), "ore")
            );
            assert_eq!(b.clip_top_y, deeptime_format::bodies::NO_CLIP);
            assert!(b.validate().is_ok());
            if i > 0 {
                let prev = &deep.bodies[i - 1];
                assert!(
                    prev.center[1] - b.center[1] >= f64::from(prev.axes[1] + b.axes[1]),
                    "seam {i} touches the one above"
                );
            }
        }
        // Volumes follow the shares (a body's axes are at least half a block, so compare seams
        // thicker than that).
        let v: Vec<f64> = deep.bodies.iter().map(Body::volume_blocks).collect();
        let thick: Vec<usize> = (0..v.len())
            .filter(|i| deep.bodies[*i].axes[1] > 0.6)
            .collect();
        assert!(thick.len() >= 2);
        for w in thick.windows(2) {
            let (i, j) = (w[0], w[1]);
            let r = (v[i] / v[j]) / (plan.shares[i] / plan.shares[j]);
            assert!((r - 1.0).abs() < 1e-3, "seams {i}/{j}: {r}");
        }
        // The top seam at the surface is clipped there, the rest kept whole.
        let at_surface = build(plan.measures_m / 2.0);
        assert_eq!(at_surface.bodies.len(), plan.seams());
        assert_eq!(at_surface.bodies[0].clip_top_y, 200);
        assert!(at_surface.kept < 1.0 && at_surface.kept > 0.0);
        // Two blocks higher the top seam stands above the ground and is eroded away.
        let above = build(plan.measures_m / 2.0 - 40.0);
        assert!(above.bodies.len() < plan.seams(), "the top seam is eroded");
        assert!(above.kept < at_surface.kept);
        assert_eq!(above.bodies[0].id, 101, "ids start at the first kept seam");
        // Far above the ground: nothing is left.
        let gone = build(-1.0e5);
        assert!(gone.bodies.is_empty() && gone.kept == 0.0);
    }

    /// The coal type is a field of tens of km: the median body is about a thousand km² of seam,
    /// a few km per seam thick after the vertical enlargement.
    #[test]
    fn the_median_coal_field_is_tens_of_km_across() {
        let t = coal();
        let sc = Scales::default();
        let [a, c] = footprint_blocks(&t, t.sizing.tonnage_mt.p50 * 1.0e6, &sc);
        let (a_km, c_km) = (a * sc.m_per_block_h / 1000.0, c * sc.m_per_block_h / 1000.0);
        let area = std::f64::consts::PI * a_km * c_km;
        assert!((800.0..1200.0).contains(&area), "{area} km²");
        assert!((20.0..100.0).contains(&(2.0 * c_km)), "{c_km} km semi-axis");
        // P90 and P10 span 300 to 6,000 km² within a factor 1.5 (the P10 was 3,000 km² until
        // the owner's answer of 2026-10-02, "Apply all": named fields such as Witbank, 5,680 km²,
        // and Highveld, 7,000 km², are larger).
        let ar = |mt: f64| {
            let [a, c] = footprint_blocks(&t, mt * 1.0e6, &sc);
            std::f64::consts::PI * a * c * (sc.m_per_block_h / 1000.0).powi(2)
        };
        assert!((200.0..450.0).contains(&ar(t.sizing.tonnage_mt.p90)));
        assert!((4000.0..9000.0).contains(&ar(t.sizing.tonnage_mt.p10)));
    }
}
