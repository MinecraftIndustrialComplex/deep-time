// SPDX-License-Identifier: LGPL-3.0-or-later
//! Geometry (plan §8.6 step 5): a deposit's bodies from its tonnage, the type's shape, aspect and
//! zoning, and the balance multipliers.
//!
//! Volume V = tonnage / ore density; the semi-axes in metres are the type's `aspect` scaled so
//! the primitive holds V (ellipsoid 4/3·π·abc, cylinder π·ac·2b). In blocks, horizontal axes are
//! divided by the horizontal scale (300 m) and multiplied by `size_mult_h`, the vertical one by
//! the vertical scale (20 m) and `size_mult_v`: a median porphyry (240 Mt / 2.6 t/m³ ≈ 9.2e7 m³)
//! is a few hundred metres across, about 1.5 blocks wide before the multiplier (plan §8.6), so
//! the horizontal multiplier does most of the work.

use std::collections::BTreeMap;

use deeptime_format::bodies::{Body, NO_CLIP, Shape, Zone};
use deeptime_format::SiteId;

use super::types::{DepositType, ExtraBody, When, ZoneDef};

/// World scales the geometry needs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scales {
    pub m_per_block_h: f64,
    pub m_per_block_v: f64,
    pub min_y: i32,
    pub max_y: i32,
}

impl Default for Scales {
    fn default() -> Self {
        Scales {
            m_per_block_h: 300.0,
            m_per_block_v: 20.0,
            min_y: -352,
            max_y: 639,
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

fn zone(def: &ZoneDef, grades: &Grades, richness: f64) -> Zone {
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
        density: if present { def.density as f32 } else { 0.0 },
    }
}

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
    let clip = if at.truncated { surface_clip } else { NO_CLIP };
    let mut ore = Body::new(
        next_id(),
        Some(site),
        shape,
        "ore",
        [at.x, centre_y, at.z],
        axes,
        at.yaw,
        at.dip,
        clip,
    );
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
    let half = (e.thickness_m / sc.m_per_block_v / 2.0).max(0.5);
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

/// The vertical half-height in metres of the ore body (before size_mult_v), for the ledger.
pub fn half_height_m(t: &DepositType, tonnage_t: f64) -> f64 {
    semi_axes_m(t.shape(), t.geometry.aspect, tonnage_t / t.sizing.ore_density_t_m3)[1]
}

/// The horizontal radius in blocks (the larger horizontal semi-axis), for separation.
pub fn radius_blocks(t: &DepositType, tonnage_t: f64, sc: &Scales) -> f64 {
    let semi = semi_axes_m(t.shape(), t.geometry.aspect, tonnage_t / t.sizing.ore_density_t_m3);
    let b = to_blocks(semi, t, sc);
    f64::from(b[0].max(b[2]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deposits::types::builtin;

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
        };
        let b = bodies_for(&t, 4, 240.0e6, &grades, &at, &Scales::default(), &mut next);
        let ore = &b[0];
        assert_eq!(ore.role, "ore");
        assert_eq!(ore.site, Some(4));
        assert_eq!(ore.clip_top_y, 120);
        assert!(ore.aabb[1][1] <= 120);
        assert!(ore.zones.iter().any(|z| z.material.as_deref() == Some("c:copper") && z.grade > 0.0));
        assert!(ore.validate().is_ok());
        assert!(b.iter().all(|x| x.id > 10));
    }
}
