// SPDX-License-Identifier: LGPL-3.0-or-later
//! The small intrusions that cut the layering (plan §8.5: "plutons, dykes and plugs cut across
//! strata"). Each is an event of the world data like a fold or a fault, but it moves nothing: it
//! is an ellipsoid of igneous rock that replaces the rock older than itself, so it cuts the folds
//! and faults before it, is offset by the ones after, and is sealed by younger cover
//! (`deeptime_format::structure`, "Intrusion").
//!
//! | What | Where and when | Shape | Rock |
//! |---|---|---|---|
//! | **stock** | in a collision belt or a continental arc while it is active (a Poisson process like the folds) | a small pluton a few km across, as thick as Cruden & McCaffrey's law of pluton outlines gives (`stock_thickness_m`), its top from just above the ground (unroofed) to tens of blocks under it; one over a pluton of about its age rises from that pluton's roof instead (`root_on_plutons`) | granite in collisions, granodiorite or diorite in arcs |
//! | **dyke** | in a continental rift while it rifts | a sheet along the rift rising from the world's floor, a block or two wide, reaching the ground along its middle | basalt |
//! | **plug** | under every extinct volcano on land still standing at the end of the run | a neck up the cone's axis | rhyolite (silicic centres), andesite (arc cones) or basalt |
//!
//! Stocks and dykes ride their plates like the folds (`feature::advance`); plugs are made at the
//! end from the volcano catalogue, which rides the plates itself. The batholiths of P1-07
//! (`plutons.rs`) stay bodies: they are far larger than the structures. Plutons and stocks both
//! get contact aureoles, written as bodies at the end of the run (`plutons::Aureole`).
//!
//! A stock's and a plug's outline is drawn on the planet (ledger SCL-235): its semi-axes are the
//! planet's metres and its trend an azimuth on the planet, and its window is the ellipse the
//! Lambert blocks make of that outline (`sphere::Around::ellipse_blocks`), which the world's
//! projection carries back to the planet's (`bake::project::project_structure`, SCL-236). Until
//! 2026-10-23 both were drawn in Lambert blocks, so a round plug was long north–south on the planet
//! by 1 / cos² of its latitude. A dyke is a sheet and keeps its Lambert trend and length.

use super::emplace::{Inputs, length_blocks, mask_axis, strain};
use super::feature::{Counts, Feature, Kind, azimuth_of};
use super::params::DeformationParams;
use crate::deposits::random::Rng;
use crate::plates::state::{ARC, CONTINENTAL, regime};
use crate::sphere::{Around, BlockFrame};
use crate::structure::params::StructureParams;
use crate::structure::plutons::{self, Pluton};
use crate::surface::lithology::index;
use crate::surface::volcanoes::{Kind as VolcanoKind, Volcano};

/// How far a dyke's or a plug's top would stand above the ground, as a share of the ground's
/// height above the world's floor: the ellipsoid then cuts the ground where
/// (a / half_along)² + (b / half_across)² < 1 − 1 / (1 + RISE)² = 0.36, the middle 60 % of its
/// length and width, with walls that are nearly upright.
const RISE: f32 = 0.25; // ledger: DEF-051

/// How close in emplacement time a pluton and a stock over it must be for the stock to rise from
/// it, Myr. Game choice (ledger DEF-078): a cupola and its batholith belong to one magmatic
/// episode; 10 Myr is the length of a long one, so stocks of other episodes keep their own floor.
const ROOT_WINDOW_MYR: f64 = 10.0;

/// How far a stock rooted on a pluton reaches below the pluton's roof, as a share of its height
/// above it. Game choice (ledger DEF-078): enough that the stock, an ellipsoid narrowing to its
/// base, still has 0.8 of its width where it meets the roof (√(1 − 0.6²)), so the two join.
const ROOT_OVERLAP: f64 = 0.25;

/// Thickness of a stock whose map outline has semi-axes `half_along_m` and `half_across_m`, m:
/// a stock is a small pluton, so it takes the plutons' law (`plutons::thickness_km`, Cruden &
/// McCaffrey 2001, T = 0.6 D^0.6 km with D the circle-equivalent diameter). The law was fitted on
/// plutons 4.5 to 600 km across; the smallest stocks (2 km in radius) sit at its lower end.
pub fn stock_thickness_m(half_along_m: f64, half_across_m: f64) -> f64 {
    // ledger: RCK-061, DEF-079
    1000.0 * plutons::thickness_km(2.0e-3 * half_along_m, 2.0e-3 * half_across_m)
}

/// Roots every stock that lies over a pluton (`plutons`, the ones alive at the end, whose bodies
/// sit under ground at `pluton_surface_y`) emplaced within [`ROOT_WINDOW_MYR`] of it on the
/// highest such pluton's roof: its base goes down to the roof and [`ROOT_OVERLAP`] beyond, when
/// that is deeper than its own floor. `surface_y(x, z)` is the ground's y at a block position.
/// Returns, for each stock rooted, its index in `features`, the pluton's index and the y of the
/// pluton's roof under the stock's centre.
pub fn root_on_plutons(
    features: &mut [Feature],
    plutons: &[Pluton],
    pluton_surface_y: &[f64],
    surface_y: impl Fn(f64, f64) -> f64,
    frame: &BlockFrame,
    m_per_block_h: f64,
    m_per_block_v: f64,
) -> Vec<(usize, usize, f64)> {
    let mut out = Vec::new();
    for (k, f) in features.iter_mut().enumerate() {
        if f.kind != Kind::Stock {
            continue;
        }
        let top_y = surface_y(f.x, f.z) - f64::from(f.depth);
        let mut best: Option<(usize, f64)> = None;
        for (i, (pl, &py)) in plutons.iter().zip(pluton_surface_y).enumerate() {
            if (f64::from(pl.t_myr) - f64::from(f.t_myr)).abs() > ROOT_WINDOW_MYR {
                continue;
            }
            // The stock's offset from the pluton's centre on the planet, where its plan is drawn
            // (ledger: SCL-235).
            let (dx, dz) = Around::new(*frame, m_per_block_h, pl.x, pl.z).offset(f.x, f.z);
            let (sy, cy) = f64::from(pl.yaw).sin_cos();
            let (u, w) = (cy * dx + sy * dz, -sy * dx + cy * dz);
            let r2 = (u / f64::from(pl.a_m)).powi(2) + (w / f64::from(pl.c_m)).powi(2);
            if r2 >= 1.0 {
                continue;
            }
            let roof_y = plutons::centre_y(pl, py, m_per_block_v)
                + f64::from(pl.b_m) / m_per_block_v * (1.0 - r2).sqrt();
            if roof_y < top_y && best.is_none_or(|(_, y)| roof_y > y) {
                best = Some((i, roof_y));
            }
        }
        if let Some((i, roof_y)) = best {
            let need = (top_y - roof_y) * (1.0 + ROOT_OVERLAP);
            if need > f64::from(f.base) {
                f.base = need as f32;
                out.push((k, i, roof_y));
            }
        }
    }
    out
}

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
        rise: 0.0,
        depth: 0.0,
        base: 0.0,
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
    // Stocks rise where thickened crust melts after a collision or an arc feeds the crust (not
    // in the foreland); dykes where a continent is being pulled apart.
    let orogen = |c: usize| {
        x.crust_type[c] as u8 == CONTINENTAL
            && (x.melt[c] || x.regime[c] as u8 == regime::ARC || x.flags[c] as u32 as u8 & ARC != 0)
    };
    let mut cum = [Vec::with_capacity(n), Vec::with_capacity(n)];
    let mut total = [0.0f64; 2];
    for c in 0..n {
        let s = strain(x, c, sp);
        // A collision belt's magmatism does not follow the convergence (it outlasts it: the melt
        // window runs `collision_melt_end_myr` past the last stamp, ledger RCK-099).
        let shortening = if x.melt[c] {
            1.0
        } else {
            s.shortening
        };
        let v = [if orogen(c) { shortening } else { 0.0 }, s.extension];
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
                // ledger: DEF-042 (sigma 0.4)
                let radius = length_blocks(p.stock_radius_km, 0.4, km_to_blocks, rng);
                // Its outline on the planet (m), long axis `random_trend` clockwise from north,
                // drawn as the ellipse the Lambert blocks make of it (ledger: SCL-235).
                let across_m = radius * x.m_per_block_h;
                let along_m = across_m * (1.0 + 0.6 * u[0]); // ledger: DEF-043
                let (first, second, yaw) = outline_blocks(
                    &frame,
                    x.m_per_block_h,
                    (bx, bz),
                    (along_m, across_m),
                    random_trend,
                );
                f.azimuth_deg = azimuth_of(yaw.cos(), yaw.sin()) as f32;
                f.half_along = first as f32;
                f.half_across = second as f32;
                f.depth = lerp(p.stock_top_depth_blocks, u[1]) as f32;
                // Its own floor: as thick as a pluton of its outline.
                f.base = (stock_thickness_m(along_m, across_m) / x.m_per_block_v) as f32;
                // ledger: DEF-045
                f.rock = if x.melt[c] {
                    index("granite")
                } else if u[2] < 0.5 {
                    index("granodiorite")
                } else {
                    index("diorite")
                };
            } else {
                // ledger: DEF-047 (sigma 0.5)
                let length = length_blocks(p.dyke_length_km, 0.5, km_to_blocks, rng);
                let rift = |k: usize| {
                    x.regime[k] as u8 == regime::RIFT && x.crust_type[k] as u8 == CONTINENTAL
                };
                let trend = mask_axis(grid, c, 6, rift).map_or(random_trend, |t| t.0);
                f.azimuth_deg = (trend + scatter).rem_euclid(360.0) as f32;
                f.half_along = (0.5 * length) as f32;
                f.half_across = lerp(p.dyke_half_width_blocks, u[0]) as f32;
                // Its top stands above the ground at the middle, so it crops out along the
                // middle of its length (where rock older than itself is at the surface).
                f.rise = RISE;
                f.rock = index("basalt");
            }
            if !frame.projection().inside_poles(bz) {
                // Drawn past the pole (DEF-081, as emplace.rs): no ground there.
                counts.past_pole += 1;
                continue;
            }
            *next_id += 1;
            counts.add(kind);
            features.push(f);
            born += 1;
        }
    }
    born
}

/// The window of an outline on the planet at block `at` of `frame`: semi-axes `(along_m,
/// across_m)` metres, the first `azimuth_deg` clockwise from north, as the ellipse the Lambert
/// blocks make of it (first semi-axis, second, and the first's direction in radians from +x
/// toward +z; ledger SCL-235). Its area is the outline's: Lambert is equal-area.
fn outline_blocks(
    frame: &BlockFrame,
    m_per_block_h: f64,
    at: (f64, f64),
    (along_m, across_m): (f64, f64),
    azimuth_deg: f64,
) -> (f64, f64, f64) {
    // Clockwise from north is (sin, cos) east and north, so south is −cos: the yaw from east
    // toward south that `ellipse_blocks` takes.
    let (sa, ca) = azimuth_deg.to_radians().sin_cos();
    Around::new(*frame, m_per_block_h, at.0, at.1).ellipse_blocks(
        along_m,
        across_m,
        (-ca).atan2(sa),
    )
}

/// The plugs of the extinct volcanoes on land in `volcanoes`: the solid conduit up each cone's
/// axis, dated to the volcano's death, so the edifice is cut and anything laid down since is not.
/// Each is round on the planet (SCL-235) of a planet `circumference` blocks round. Ids are
/// `first_id`, `first_id + 1`, ….
pub fn plugs(
    volcanoes: &[Volcano],
    p: &DeformationParams,
    circumference: u32,
    m_per_block_h: f64,
    first_id: u32,
) -> Vec<Feature> {
    if !p.enabled() || p.intrusions <= 0.0 {
        return Vec::new();
    }
    let frame = BlockFrame::new(circumference);
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
        // Blocks of the planet: the window is the circle's ellipse in Lambert blocks.
        let r_m = radius * m_per_block_h;
        let (first, second, yaw) =
            outline_blocks(&frame, m_per_block_h, (v.x, v.z), (r_m, r_m), 90.0);
        f.azimuth_deg = azimuth_of(yaw.cos(), yaw.sin()) as f32;
        f.half_along = first as f32;
        f.half_across = second as f32;
        f.rise = RISE;
        // ledger: DEF-050
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
    use super::super::feature::direction;
    use super::*;
    use deeptime_format::structure::Shape;

    #[test]
    fn a_stock_and_a_plug_are_round_on_the_planet_at_high_latitude() {
        // SCL-235: at 30°, 55° and 70° N the window a stock or a plug is given, read as a feature
        // reads it (its azimuth's direction and half-lengths in Lambert blocks), is the planet's
        // outline: every point of its edge lies on the 6 × 4 km ellipse at the trend (a stock) or
        // the circle of 1.5 km (a plug) on the planet, and the window keeps the outline's area.
        // On the planet's tangent plane at the centre (the frame's linear map there: a block of x
        // is cos φ blocks east, a block of z 1/cos φ south) the edge is the outline to f32's
        // digits; by the great-circle offset it is off at first order in its size, by at most
        // 2 (a / R) tan φ in r², as the frame's scale changes across it.
        let c = 16_384;
        let frame = BlockFrame::new(c);
        let mh = 300.0;
        for lat in [30.0f64, 55.0, 70.0] {
            let at = (100.0, frame.z_of_lat(lat));
            let around = Around::new(frame, mh, at.0, at.1);
            let cos = lat.to_radians().cos();
            for (axes, trend) in [((6000.0, 4000.0), 35.0), ((1500.0, 1500.0), 90.0)] {
                let (first, second, yaw) = outline_blocks(&frame, mh, at, axes, trend);
                let mut f = blank(0, Kind::Stock, at.0, at.1, 0, 0.0);
                f.azimuth_deg = azimuth_of(yaw.cos(), yaw.sin()) as f32;
                (f.half_along, f.half_across) = (first as f32, second as f32);
                let along = direction(f64::from(f.azimuth_deg));
                let across = direction(f64::from(f.azimuth_deg) + 90.0);
                // The trend's unit vector on the planet, east and south.
                let (sa, ca) = trend.to_radians().sin_cos();
                let (te, ts) = (sa, -ca);
                for k in 0..72 {
                    let t = f64::from(k) * 5f64.to_radians();
                    let (u, v) = (
                        f64::from(f.half_along) * t.cos(),
                        f64::from(f.half_across) * t.sin(),
                    );
                    let x = at.0 + u * along[0] + v * across[0];
                    let z = at.1 + u * along[1] + v * across[1];
                    let r2 = |(e, s): (f64, f64)| {
                        let (pu, pv) = (e * te + s * ts, -e * ts + s * te);
                        (pu / axes.0).powi(2) + (pv / axes.1).powi(2)
                    };
                    let plane = r2(((x - at.0) * cos * mh, (z - at.1) * mh / cos));
                    assert!(
                        (plane - 1.0).abs() < 1e-5,
                        "{lat}°, {axes:?} at {k}: r2 {plane}"
                    );
                    let sphere = r2(around.offset(x, z));
                    let first_order = 2.0 * axes.0 / around.radius_m() * lat.to_radians().tan();
                    assert!(
                        (sphere - 1.0).abs() < first_order,
                        "{lat}°, {axes:?} at {k}: r2 {sphere}"
                    );
                }
                let area = first * second * mh * mh;
                assert!(
                    (area / (axes.0 * axes.1) - 1.0).abs() < 1e-9,
                    "{lat}°: area"
                );
            }
        }
    }

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
                    // Drawn on the planet: its Lambert axes keep the outline's area, so their
                    // geometric mean is between its radius and 1.6 times it (DEF-043).
                    let r_km = (f64::from(f.half_along) * f64::from(f.half_across)).sqrt() * 0.3;
                    assert!((1.99..=10.01 * 1.6f64.sqrt()).contains(&r_km), "{r_km}");
                    assert!((-4.01..=60.01).contains(&f.depth) && f.rise == 0.0);
                }
                Kind::Dyke => {
                    assert!((177..=182).contains(&i), "dyke at column {i}");
                    assert_eq!(f.rock, index("basalt"));
                    // Along the rift (north), thin, and reaching above the ground.
                    let d = f.azimuth_deg.rem_euclid(180.0);
                    assert!(d.min(180.0 - d) < 50.0, "{}", f.azimuth_deg);
                    let [lo, hi] = sp.deformation.dyke_half_width_blocks;
                    assert!((lo..=hi).contains(&f64::from(f.half_across)) && f.rise > 0.0);
                    assert!(
                        hi <= 1.5,
                        "dykes no wider than 3 blocks (decisions log 2026-10-02)"
                    );
                }
                k => panic!("{k:?} from intrude::emplace"),
            }
            // Baked: a dyke an ellipsoid rooted on the world's floor, a stock one of its own
            // thickness under its top.
            let e = f.to_baked(70.0, 5000, -330.0);
            let Shape::Intrusion {
                y,
                half_height,
                class,
            } = e.shape
            else {
                panic!("{e:?}")
            };
            assert_eq!(u16::from(class), f.rock);
            let top = y + half_height;
            if f.kind == Kind::Stock {
                assert!((top - (70.0 - f64::from(f.depth))).abs() < 1e-6);
                let m = |b: f32| f64::from(b) * p.inputs().m_per_block_h;
                let t = stock_thickness_m(m(f.half_along), m(f.half_across));
                assert!((2.0 * half_height - t / p.inputs().m_per_block_v).abs() < 1e-3);
                // A few km thick, not the world's height.
                assert!((800.0..6000.0).contains(&t), "{t}");
            } else {
                assert_eq!(y, -330.0);
                assert!(top > 70.0 + 50.0, "a dyke stands through the ground: {top}");
            }
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

    /// The narrowest default dyke still crops out without a break across the middle of its
    /// length, whatever its trend and wherever it lies between block columns: the columns where
    /// its basalt reaches the ground form one 8-connected run. (The outcrop is 0.6 of the dyke's
    /// width there, so below a half-width of 0.83 blocks a dyke lying along the grid between two
    /// columns would not crop out at all.)
    #[test]
    fn the_narrowest_dyke_crops_out_unbroken() {
        use std::collections::HashSet;
        let p = DeformationParams::default();
        let hw = p.dyke_half_width_blocks[0];
        let (ground, floor, half_along) = (70.0, -330.0, 40.0);
        // Three blocks either side of the middle (the outcrop is a block wide out to four).
        let span = 0.075 * half_along;
        for az in [0.0f32, 13.0, 30.0, 45.0, 62.0, 90.0, 117.0] {
            for (x0, z0) in [(0.0, 0.0), (0.5, 0.0), (0.5, 0.5), (0.25, 0.7)] {
                let mut f = blank(1, Kind::Dyke, x0, z0, 0, 0.0);
                f.azimuth_deg = az;
                f.half_along = half_along as f32;
                f.half_across = hw as f32;
                f.rise = RISE;
                let e = f.to_baked(ground, 1, floor);
                let Shape::Intrusion { y, half_height, .. } = e.shape else {
                    panic!("{e:?}")
                };
                // The column rule's test (`deeptime_format::structure`, "Intrusion"), and how
                // far along the dyke a column lies.
                let at = |x: i64, z: i64| {
                    let (a, b) = e.window.local(x as f64, z as f64, 4096.0);
                    let r2 = (a / e.window.half_along).powi(2) + (b / e.window.half_across).powi(2);
                    (r2 < 1.0 && y + half_height * (1.0 - r2).sqrt() >= ground, a)
                };
                let r = span as i64 + 3;
                let out: HashSet<(i64, i64)> = (-r..=r)
                    .flat_map(|i| (-r..=r).map(move |j| (i, j)))
                    .filter(|&(i, j)| {
                        let (hit, a) = at(i, j);
                        hit && a.abs() <= span + 1.5
                    })
                    .collect();
                let place = format!("azimuth {az} at ({x0}, {z0})");
                // Flood from the column nearest the middle.
                let start = *out
                    .iter()
                    .min_by(|p, q| at(p.0, p.1).1.abs().total_cmp(&at(q.0, q.1).1.abs()))
                    .unwrap_or_else(|| panic!("{place}: no outcrop"));
                let mut seen = HashSet::from([start]);
                let mut todo = vec![start];
                while let Some((i, j)) = todo.pop() {
                    for di in -1..=1 {
                        for dj in -1..=1 {
                            let q = (i + di, j + dj);
                            if out.contains(&q) && seen.insert(q) {
                                todo.push(q);
                            }
                        }
                    }
                }
                let reach = |sign: f64| seen.iter().any(|q| sign * at(q.0, q.1).1 >= span);
                assert!(
                    reach(1.0) && reach(-1.0),
                    "{place}: the outcrop breaks within {span} blocks of the middle"
                );
            }
        }
    }
}
