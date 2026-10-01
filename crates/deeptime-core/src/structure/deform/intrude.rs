// SPDX-License-Identifier: LGPL-3.0-or-later
//! The small intrusions that cut the layering (plan §8.5: "plutons, dykes and plugs cut across
//! strata"). Each is an event of the world data like a fold or a fault, but it moves nothing: it
//! is an ellipsoid of igneous rock that replaces the rock older than itself, so it cuts the folds
//! and faults before it, is offset by the ones after, and is sealed by younger cover
//! (`deeptime_format::structure`, "Intrusion").
//!
//! | What | Where and when | Shape | Rock |
//! |---|---|---|---|
//! | **stock** | in a collision belt or a continental arc while it is active (a Poisson process like the folds) | a cupola a few km across, its top from just above the ground (unroofed) to tens of blocks under it | granite in collisions, granodiorite or diorite in arcs |
//! | **dyke** | in a continental rift while it rifts | a sheet along the rift, a block or two wide, reaching the ground at its middle | basalt |
//! | **plug** | under every extinct volcano on land still standing at the end of the run | a neck up the cone's axis | rhyolite (silicic centres), andesite (arc cones) or basalt |
//!
//! Stocks and dykes ride their plates like the folds (`feature::advance`); plugs are made at the
//! end from the volcano catalogue, which rides the plates itself. The batholiths of P1-07
//! (`plutons.rs`) stay bodies: they are far larger than the structures and carry aureoles.

use super::emplace::{Inputs, length_blocks, mask_axis, strain};
use super::feature::{Counts, Feature, Kind};
use super::params::DeformationParams;
use crate::deposits::random::Rng;
use crate::plates::state::{ARC, CONTINENTAL, regime};
use crate::sphere::BlockFrame;
use crate::structure::params::StructureParams;
use crate::surface::lithology::index;
use crate::surface::volcanoes::{Kind as VolcanoKind, Volcano};

/// A feature of `kind` with nothing set but where and when.
fn blank(id: u32, kind: Kind, x: f64, z: f64, plate: u32, t_myr: f64) -> Feature {
    Feature {
        id,
        kind,
        x,
        z,
        plate,
        t_myr: t_myr as f32,
        azimuth_deg: 0.0,
        half_along: 0.0,
        half_across: 0.0,
        wavelength: 0.0,
        amplitude: 0.0,
        phase: 0.0,
        dip_deg: 0.0,
        slip: 0.0,
        cause: 0,
        adrift: 0,
        wear_m: 0.0,
        half_height: 0.0,
        depth: 0.0,
        rock: 0,
    }
}

/// Draws this step's stocks and dykes into `features`. Returns how many were started.
#[allow(clippy::too_many_arguments)]
pub fn emplace(
    x: &Inputs<'_>,
    sp: &StructureParams,
    features: &mut Vec<Feature>,
    dt_myr: f64,
    t_myr: f64,
    rng: &mut Rng,
    next_id: &mut u32,
    counts: &mut Counts,
    cause: &dyn Fn(Kind, f64, f64) -> u64,
) -> usize {
    let p: &DeformationParams = &sp.deformation;
    if !p.enabled() || p.intrusions <= 0.0 {
        return 0;
    }
    let grid = x.grid;
    let n = grid.cells();
    let w = grid.w as usize;
    let frame = BlockFrame::new(grid.circumference() as u32);
    let r = f64::from(grid.r);
    let km_to_blocks = 1000.0 / x.m_per_block_h * p.size;
    let side_km = r * x.m_per_block_h / 1000.0;
    let area = side_km * side_km;
    // Stocks rise where the crust is being thickened or fed by an arc (not in the foreland);
    // dykes where a continent is being pulled apart.
    let orogen = |c: usize| {
        x.crust_type[c] as u8 == CONTINENTAL
            && (x.regime[c] as u8 == regime::COLLISION
                || x.regime[c] as u8 == regime::ARC
                || x.flags[c] as u32 as u8 & ARC != 0)
    };
    let mut cum = [Vec::with_capacity(n), Vec::with_capacity(n)];
    let mut total = [0.0f64; 2];
    for c in 0..n {
        let s = strain(x, c, sp);
        let v = [if orogen(c) { s.shortening } else { 0.0 }, s.extension];
        for k in 0..2 {
            total[k] += v[k] * area;
            cum[k].push(total[k]);
        }
    }
    let per = p.intrusions * p.density * dt_myr / (p.size * p.size);
    let lerp = |v: [f64; 2], t: f64| v[0] + (v[1] - v[0]) * t;
    let mut born = 0usize;
    for (kind, which, rate) in [
        (Kind::Stock, 0usize, p.stock_rate_km2_myr),
        (Kind::Dyke, 1, p.dyke_rate_km2_myr),
    ] {
        if total[which] <= 0.0 {
            continue;
        }
        let count = rng.poisson(total[which] * rate * per);
        for _ in 0..count {
            if features.len() >= p.max_structures as usize {
                return born;
            }
            let target = rng.uniform() * total[which];
            let c = cum[which].partition_point(|&v| v <= target).min(n - 1);
            let (i, j) = ((c % w) as i64, (c / w) as i64);
            let bx = frame.wrap_x(grid.block_x(i) as f64 + (rng.uniform() - 0.5) * r);
            let bz = grid.block_z(j) as f64 + (rng.uniform() - 0.5) * r;
            let u = [rng.uniform(), rng.uniform(), rng.uniform()];
            let scatter = p.trend_scatter_deg * rng.normal();
            let random_trend = 180.0 * rng.uniform();
            let mut f = blank(*next_id, kind, bx, bz, x.plate_id[c].max(0.0) as u32, t_myr);
            f.cause = cause(kind, bx, bz);
            if kind == Kind::Stock {
                let radius = length_blocks(p.stock_radius_km, 0.4, km_to_blocks, rng);
                f.azimuth_deg = random_trend as f32;
                f.half_across = radius as f32;
                f.half_along = (radius * (1.0 + 0.6 * u[0])) as f32;
                f.half_height = (radius * p.stock_height_ratio) as f32;
                f.depth = (lerp(p.stock_top_depth_blocks, u[1]) + f64::from(f.half_height)) as f32;
                f.rock = if x.regime[c] as u8 == regime::COLLISION {
                    index("granite")
                } else if u[2] < 0.5 {
                    index("granodiorite")
                } else {
                    index("diorite")
                };
            } else {
                let length = length_blocks(p.dyke_length_km, 0.5, km_to_blocks, rng);
                let rift = |k: usize| {
                    x.regime[k] as u8 == regime::RIFT && x.crust_type[k] as u8 == CONTINENTAL
                };
                let trend = mask_axis(grid, c, 6, rift).map_or(random_trend, |t| t.0);
                f.azimuth_deg = (trend + scatter).rem_euclid(360.0) as f32;
                f.half_along = (0.5 * length) as f32;
                f.half_across = lerp(p.dyke_half_width_blocks, u[0]) as f32;
                f.half_height = p.dyke_half_height_blocks as f32;
                // Its top stands above the ground at the middle, so it crops out along the
                // middle half of its length (where rock older than itself is at the surface).
                f.depth = 0.7 * f.half_height;
                f.rock = index("basalt");
            }
            *next_id += 1;
            counts.add(kind);
            features.push(f);
            born += 1;
        }
    }
    born
}

/// The plugs of the extinct volcanoes on land in `volcanoes`: the solid conduit up each cone's
/// axis, dated to the volcano's death, so the edifice is cut and anything laid down since is not.
/// Ids are `first_id`, `first_id + 1`, ….
pub fn plugs(
    volcanoes: &[Volcano],
    p: &DeformationParams,
    m_per_block_h: f64,
    first_id: u32,
) -> Vec<Feature> {
    if !p.enabled() || p.intrusions <= 0.0 {
        return Vec::new();
    }
    let [lo, hi] = p.plug_radius_blocks;
    let mut out = Vec::new();
    for v in volcanoes {
        let Some(died) = v.extinct_myr else {
            continue;
        };
        if v.oceanic || v.gone.is_some() {
            continue;
        }
        let radius = (v.radius_m / m_per_block_h * p.plug_radius_share * p.size).clamp(lo, hi);
        let mut f = blank(
            first_id + out.len() as u32,
            Kind::Plug,
            v.x,
            v.z,
            v.plate,
            died,
        );
        f.half_along = radius as f32;
        f.half_across = radius as f32;
        f.half_height = p.plug_half_height_blocks as f32;
        f.depth = 0.6 * f.half_height;
        f.rock = if v.silicic || v.kind == VolcanoKind::Silicic {
            index("rhyolite")
        } else if v.kind == VolcanoKind::Strato {
            index("andesite")
        } else {
            index("basalt")
        };
        f.cause = v.cause.map_or(0, |e| e.0);
        out.push(f);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::super::emplace::tests::Planet;
    use super::*;
    use deeptime_format::structure::Shape;

    #[test]
    fn stocks_rise_in_the_orogen_and_dykes_follow_the_rift() {
        let p = Planet::new();
        let mut sp = StructureParams::default();
        sp.deformation.intrusions = 10.0;
        let mut all = Vec::new();
        let mut counts = Counts::default();
        let mut next = 0u32;
        for s in 0..15u32 {
            let mut rng = Rng::stream(11 ^ (u64::from(s) << 20), "structure/intrude");
            emplace(
                &p.inputs(),
                &sp,
                &mut all,
                2.0,
                -100.0 + 2.0 * f64::from(s),
                &mut rng,
                &mut next,
                &mut counts,
                &|_, _, _| 0,
            );
        }
        assert!(counts.stocks > 10 && counts.dykes > 10, "{counts:?}");
        let w = p.grid.w as usize;
        for f in &all {
            let i = crate::structure::plutons::cell_of(&p.grid, f.x, f.z) % w;
            match f.kind {
                Kind::Stock => {
                    // In the collision belt itself, not the foreland; granite.
                    assert!((55..=64).contains(&i), "stock at column {i}");
                    assert_eq!(f.rock, index("granite"));
                    let r_km = f64::from(f.half_across) * 0.3;
                    assert!((1.99..=10.01).contains(&r_km), "{r_km}");
                    let top = f.depth - f.half_height;
                    assert!((-4.01..=60.01).contains(&top), "{top}");
                }
                Kind::Dyke => {
                    assert!((177..=182).contains(&i), "dyke at column {i}");
                    assert_eq!(f.rock, index("basalt"));
                    // Along the rift (north), thin, and reaching above the ground.
                    let d = f.azimuth_deg.rem_euclid(180.0);
                    assert!(d.min(180.0 - d) < 50.0, "{}", f.azimuth_deg);
                    assert!((1.0..=2.5).contains(&f.half_across) && f.depth < f.half_height);
                }
                k => panic!("{k:?} from intrude::emplace"),
            }
            // Baked: an ellipsoid hung from the ground.
            let e = f.to_baked(70.0, 5000);
            let Shape::Intrusion {
                y,
                half_height,
                class,
            } = e.shape
            else {
                panic!("{e:?}")
            };
            assert_eq!(u16::from(class), f.rock);
            assert!((y - (70.0 - f64::from(f.depth))).abs() < 1e-9 && half_height > 0.0);
        }
        // Off: none. The switch for folds and faults turns them off too.
        sp.deformation.intrusions = 0.0;
        let mut none = Vec::new();
        let mut rng = Rng::stream(11, "structure/intrude");
        let run = |sp: &StructureParams, none: &mut Vec<Feature>, rng: &mut Rng| {
            emplace(
                &p.inputs(),
                sp,
                none,
                2.0,
                -50.0,
                rng,
                &mut 0,
                &mut Counts::default(),
                &|_, _, _| 0,
            )
        };
        assert_eq!(run(&sp, &mut none, &mut rng), 0);
        sp.deformation.intrusions = 1.0;
        sp.deformation.strength = 0.0;
        assert_eq!(run(&sp, &mut none, &mut rng), 0);
    }
}
