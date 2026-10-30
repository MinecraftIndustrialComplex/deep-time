// SPDX-License-Identifier: LGPL-3.0-or-later
//! From plate tectonics to structures: each plate step, the regimes the plate model reports are
//! translated into the folds and faults they make (plan §8.5: "plate events become local events").
//!
//! | Where (per L1 cell, continental crust) | Strain | What forms |
//! |---|---|---|
//! | a collision's belt and wedge (the plate model's `plates/collision_mm_yr` above zero: a Himalayan orogeny in progress) | shortening at the convergence across the front over `collision_reference_mm_yr` (at most 1); fold tightness from the collision's real shortening (`plates/shortening`) | fold trains and thrusts along the plate model's fold trend, thrusts dipping under the high ground |
//! | a continental arc and its back-arc where the plate model is lifting the crust (an Andean orogeny) | shortening, half strength at 80 mm/yr of convergence and `arc_uplift_mm_yr` of uplift | the same, dipping under the high ground |
//! | a collision's foreland: the flexural basin (`surface/flexure_m` below 100 m under local isostasy) on the incoming plate, within the reach of the collision's front | 0.6 of the belt's shortening, none on the forebulge | gentler folds and thrusts in the basin's fill (the Zagros) |
//! | the foreland of an Andean margin, out to `foreland_km` from a continental arc | shortening, fading with distance | gentler folds and thrusts (the Subandean belt) |
//! | the rift regime | extension | normal faults along the rift, dipping towards its axis (one in four the other way) |
//! | a lifted margin inside a subduction zone's reach where convergence is more than 30° oblique | shear | strike-slip faults along the margin (slip partitioning) |
//!
//! Each is a Poisson process in space and time, like the plutons: the expected number in a step
//! is Σ strain × cell area × the kind's rate × Δt, the area on the block metric (structures are
//! sized in blocks, so a belt is as busy on the map at 60° as at the equator). Where
//! `max_overlap` structures of the kind already lie, a new one is not started (the existing ones
//! take up the strain), unless the oldest of them is `overprint_myr` older: then the new one
//! takes its place, so a belt that is active again deforms the rock laid down since, and the
//! number of structures over a place stays bounded however long the history. A structure is
//! dated to the step that made it and never changes afterwards; it rides its plate
//! (`feature::advance`), so the normal faults of a rift end up along the two passive margins the
//! rift becomes, and the folds of a collision stay in the suture when the orogen has worn down.
//!
//! What is not translated here: batholiths (they are bodies already, `plutons.rs`), the small
//! intrusions (`intrude.rs`: stocks, dykes and plugs, events too), and unconformities, which need
//! no event: rock laid down after a structure formed is simply not deformed by it, so every
//! structure is sealed by the first strata younger than itself.

use deeptime_format::GridDesc;

use super::feature::{Counts, Feature, Kind, azimuth_of, direction};
use super::params::DeformationParams;
use crate::deposits::random::Rng;
use crate::plates::belts::FOREDEEP_M;
use crate::plates::state::{ARC, CONTINENTAL, FORELAND, SUBDUCTION_ZONE, regime};
use crate::sphere::BlockFrame;
use crate::structure::params::StructureParams;

/// What a step's translation reads, per L1 cell.
pub struct Inputs<'a> {
    pub grid: &'a GridDesc,
    pub m_per_block_h: f64,
    /// Metres a block is high (`scale.m_per_block_v`): with `m_per_block_h`, how far structures
    /// are stretched upwards (`DeformationParams::block_stretch`).
    pub m_per_block_v: f64,
    pub regime: &'a [f32],
    pub crust_type: &'a [f32],
    /// Whether the cell's crust melts after a collision ([`crate::structure::collision_melt`]):
    /// the granite stocks' window.
    pub melt: &'a [bool],
    pub flags: &'a [f32],
    pub convergence: &'a [f32],
    pub obliquity: &'a [f32],
    pub thickness: &'a [f32],
    /// The plate model's fold trend, degrees clockwise from north in [0, 180); NaN where none.
    pub fold_trend: &'a [f32],
    pub plate_id: &'a [f32],
    /// The plate model's uplift over the last step, mm/yr.
    pub uplift: &'a [f32],
    /// Ground height, m (any datum: only its slope is read).
    pub elevation: &'a [f32],
    /// Distance to the nearest active orogen, km on the block metric
    /// (`traps::orogen_distance_km`): the direction a foreland thrust dips.
    pub orogen_km: &'a [f32],
    /// Distance to the nearest continental arc, km on the block metric (`traps::arc_distance_km`):
    /// the foreland of an Andean margin.
    pub arc_km: &'a [f32],
    /// The convergence across the front of the collision whose belt, wedge or foreland the cell
    /// lies in this step, mm/yr (0: none; `plates/collision_mm_yr`), and that collision's
    /// shortening so far, km (`plates/shortening`).
    pub collision_mm_yr: &'a [f32],
    pub shortening_km: &'a [f32],
    /// The surface's flexure, m (`surface/flexure_m`: negative in basins), when the surface stage
    /// made one.
    pub flexure_m: Option<&'a [f32]>,
    /// A collision's belt width, km on the plate model's sphere (`tectonics.collision_belt_km`):
    /// the width its shortening was taken up over.
    pub collision_belt_km: f64,
    /// Blocks per km of the plate model's sphere (C / 2π·`tectonics.model_radius_km`): tectonic
    /// widths are Earth-normalized, like the plate model's own distances.
    pub blocks_per_model_km: f64,
}

/// Where a cell's shortening comes from (the run summary counts the folds and thrusts by it).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Source {
    #[default]
    None,
    /// The collision regime: a belt or wedge the plate model is raising.
    Collision,
    /// A lifted Andean margin.
    Arc,
    /// The foreland of an Andean margin (the distance rule).
    Foreland,
    /// The foreland basin of a collision (the flexural basin on the incoming plate).
    Basin,
}

/// The strain a cell is under: shortening, extension and shear, each 0–1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Strain {
    /// How fast structures form here, 0-1 (the rate constants times this).
    pub shortening: f64,
    /// How tight a fold's strain is drawn here, 0-1: between the least and the most the
    /// collision's (or the crust's) shortening gives (`fold_shortening`), and how far a thrust's
    /// slip has grown.
    pub tightness: f64,
    pub extension: f64,
    pub shear: f64,
    /// Which rule gave the shortening.
    pub source: Source,
}

/// The Andean rule of cell `c` (continental crust in an arc or back-arc): its lift factor
/// min(uplift / `arc_uplift_mm_yr`, 1) and its strain. An Andean margin shortens where it is being
/// lifted: the plate model's back-arc regime reaches the whole subduction distance inland, most of
/// it undeformed.
pub fn andean(x: &Inputs<'_>, c: usize, sp: &StructureParams) -> Option<(f64, f64)> {
    if x.crust_type[c] as u8 != CONTINENTAL {
        return None;
    }
    let reg = x.regime[c] as u8;
    let flags = x.flags[c] as u32 as u8;
    if flags & ARC == 0 && reg != regime::ARC && reg != regime::BACK_ARC {
        return None;
    }
    let speed = (f64::from(x.convergence[c].max(0.0)) / 80.0).min(1.0); // ledger: DEF-026
    let p = &sp.deformation;
    let lift = (f64::from(x.uplift[c].max(0.0)) / p.arc_uplift_mm_yr.max(1e-9)).min(1.0);
    Some((lift, 0.5 * speed * lift)) // ledger: DEF-026
}

/// The strain of cell `c`.
pub fn strain(x: &Inputs<'_>, c: usize, sp: &StructureParams) -> Strain {
    if x.crust_type[c] as u8 != CONTINENTAL {
        return Strain::default();
    }
    let reg = x.regime[c] as u8;
    let flags = x.flags[c] as u32 as u8;
    let conv = f64::from(x.convergence[c].max(0.0));
    let speed = (conv / 80.0).min(1.0); // ledger: DEF-026
    let p = &sp.deformation;
    // A collision shortens its belt and wedge at the convergence across its front, so a slowing
    // collision makes fewer structures; the foreland basin in front of the wedge, 0.6 of that
    // where the crust lies in a flexural basin (none on the forebulge). ledger: DEF-075, DEF-076
    let in_collision = f64::from(x.collision_mm_yr[c].max(0.0));
    let rate = (in_collision / p.collision_reference_mm_yr).min(1.0);
    let in_foreland = flags & FORELAND != 0;
    let collision = if in_collision > 0.0 && !in_foreland {
        (rate, 1.0, Source::Collision)
    } else if in_collision > 0.0 && x.flexure_m.is_some_and(|f| f64::from(f[c]) < FOREDEEP_M) {
        (0.6 * rate, 0.6, Source::Basin) // ledger: DEF-076
    } else {
        (0.0, 0.0, Source::None)
    };
    let andean = andean(x, c, sp).map_or(0.0, |a| a.1);
    // The foreland of an Andean margin fades with the distance from the arc.
    let d = f64::from(x.arc_km[c]) * 1000.0 / x.m_per_block_h;
    let belt = p.foreland_km * x.blocks_per_model_km;
    let foreland = if d > 0.0 && d < belt {
        0.6 * (1.0 - d / belt) // ledger: DEF-027
    } else {
        0.0
    };
    let (shortening, tightness, source) =
        if collision.0 > 0.0 && collision.0 >= andean.max(foreland) {
            collision
        } else if andean >= foreland {
            let source = if andean > 0.0 {
                Source::Arc
            } else {
                Source::None
            };
            (andean, andean, source)
        } else {
            (foreland, foreland, Source::Foreland)
        };
    let extension = if reg == regime::RIFT { 1.0 } else { 0.0 };
    let obliquity = f64::from(x.obliquity[c]);
    // Slip partitioning: the margin-parallel part of oblique convergence is taken up by
    // strike-slip faults in the arc (the Sumatran fault), where the margin is being lifted.
    let shear = if flags & SUBDUCTION_ZONE != 0 && obliquity > 30.0 {
        let lift = f64::from(x.uplift[c].max(0.0)) / p.arc_uplift_mm_yr.max(1e-9);
        ((obliquity - 30.0) / 45.0).min(1.0) * speed * lift.min(1.0) // ledger: DEF-029
    } else {
        0.0
    };
    Strain {
        shortening,
        tightness,
        extension,
        shear,
        source,
    }
}

/// The long axis of the cells around `c` (within `radius` cells) that satisfy `on`, as an
/// azimuth in [0°, 180°), and the offset (blocks, x and z) of their centroid from the cell.
/// `None` when fewer than three cells are on or they have no long axis.
pub(super) fn mask_axis(
    grid: &GridDesc,
    c: usize,
    radius: i64,
    on: impl Fn(usize) -> bool,
) -> Option<(f64, [f64; 2])> {
    let w = grid.w as i64;
    let (i0, j0) = ((c as i64) % w, (c as i64) / w);
    let (mut n, mut sx, mut sz) = (0.0f64, 0.0f64, 0.0f64);
    let (mut sxx, mut szz, mut sxz) = (0.0f64, 0.0f64, 0.0f64);
    for dj in -radius..=radius {
        let j = j0 + dj;
        if j < 0 || j >= i64::from(grid.h) {
            continue;
        }
        for di in -radius..=radius {
            let k = (j * w + (i0 + di).rem_euclid(w)) as usize;
            if on(k) {
                let (x, z) = (di as f64, dj as f64);
                n += 1.0;
                sx += x;
                sz += z;
                sxx += x * x;
                szz += z * z;
                sxz += x * z;
            }
        }
    }
    if n < 3.0 {
        return None;
    }
    let (mx, mz) = (sx / n, sz / n);
    let (cxx, czz, cxz) = (sxx / n - mx * mx, szz / n - mz * mz, sxz / n - mx * mz);
    // The principal axis of the 2 × 2 covariance.
    let theta = 0.5 * (2.0 * cxz).atan2(cxx - czz);
    let (l1, l2) = {
        let mean = 0.5 * (cxx + czz);
        let diff = (0.25 * (cxx - czz) * (cxx - czz) + cxz * cxz).sqrt();
        (mean + diff, mean - diff)
    };
    if l1 <= 0.0 || l2 > 0.8 * l1 {
        return None;
    }
    let r = f64::from(grid.r);
    Some((
        azimuth_of(theta.cos(), theta.sin()).rem_euclid(180.0),
        [mx * r, mz * r],
    ))
}

/// The gradient of `f` at cell `c` over ±`span` cells, per block: (d/dx, d/dz). NaN cells count
/// as the centre's value.
fn gradient(grid: &GridDesc, f: &[f32], c: usize, span: i64) -> [f64; 2] {
    let w = grid.w as i64;
    let (i, j) = ((c as i64) % w, (c as i64) / w);
    let at = |di: i64, dj: i64| {
        let jj = (j + dj).clamp(0, i64::from(grid.h) - 1);
        let v = f[(jj * w + (i + di).rem_euclid(w)) as usize];
        if v.is_finite() {
            f64::from(v)
        } else {
            f64::from(f[c])
        }
    };
    let d = 2.0 * span as f64 * f64::from(grid.r);
    [
        (at(span, 0) - at(-span, 0)) / d,
        (at(0, span) - at(0, -span)) / d,
    ]
}

/// The plate model's fold trend at or next to cell `c` (it is NaN where the crust has not been
/// shortened yet), degrees in [0, 180).
fn trend_near(x: &Inputs<'_>, c: usize) -> Option<f64> {
    let w = x.grid.w as i64;
    let (i, j) = ((c as i64) % w, (c as i64) / w);
    for ring in 0..=3i64 {
        // ledger: DEF-009 (three rings of cells)
        for dj in -ring..=ring {
            for di in -ring..=ring {
                if di.abs().max(dj.abs()) != ring {
                    continue;
                }
                let jj = j + dj;
                if jj < 0 || jj >= i64::from(x.grid.h) {
                    continue;
                }
                let t = x.fold_trend[(jj * w + (i + di).rem_euclid(w)) as usize];
                if t.is_finite() {
                    return Some(f64::from(t));
                }
            }
        }
    }
    None
}

/// A lognormal length in blocks from `[median, min, max]` km.
pub(super) fn length_blocks(v: [f64; 3], sigma: f64, km_to_blocks: f64, rng: &mut Rng) -> f64 {
    let [median, lo, hi] = v;
    (median * (sigma * rng.normal()).exp()).clamp(lo, hi) * km_to_blocks
}

/// A dip-slip fault of dip `dip_deg` and slip `slip` along its dip (block space, 1:1) drawn `x`
/// times taller (`DeformationParams::block_stretch`): the heave (`slip · cos dip`) is kept and the
/// throw (`slip · sin dip`) multiplied, so the dip becomes atan(X tan d) and the slip
/// slip · √(cos² d + X² sin² d). Returns (dip in degrees, slip). ledger: DEF-011
pub fn stretch(dip_deg: f64, slip: f64, x: f64) -> (f64, f64) {
    let (sin, cos) = dip_deg.to_radians().sin_cos();
    let (heave, throw) = (cos, sin * x);
    (throw.atan2(heave).to_degrees(), slip * heave.hypot(throw))
}

/// Draws this step's new structures into `features` (the ones already there decide the overlap
/// rule, and the oldest may give way); `cause(kind, x, z)` names the tectonic event behind a
/// structure at a block position. Returns how many were started.
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
    let mut born = 0usize;
    if !p.enabled() {
        return born;
    }
    let grid = x.grid;
    let n = grid.cells();
    let w = grid.w as usize;
    let c_blocks = grid.circumference() as f64;
    let frame = BlockFrame::new(grid.circumference() as u32);
    let r = f64::from(grid.r);
    let km_to_blocks = 1000.0 / x.m_per_block_h * p.size;
    let stretch_x = p.block_stretch(x.m_per_block_h, x.m_per_block_v); // ledger: DEF-011

    // Cumulative strain × area per kind of strain.
    let mut cum = [
        Vec::with_capacity(n),
        Vec::with_capacity(n),
        Vec::with_capacity(n),
    ];
    let mut total = [0.0f64; 3];
    let side_km = r * x.m_per_block_h / 1000.0;
    let area = side_km * side_km;
    for c in 0..n {
        let s = strain(x, c, sp);
        if let Some((lift, a)) = andean(x, c, sp) {
            counts.arc_cell_steps += 1;
            counts.arc_lift_sum += lift;
            counts.arc_lifted_steps += u64::from(lift >= 1.0);
            counts.arc_strain_sum += a;
        }
        for (k, v) in [s.shortening, s.extension, s.shear].into_iter().enumerate() {
            if v > 0.0 {
                total[k] += v * area;
            }
            cum[k].push(total[k]);
        }
    }
    // Bigger structures cover more ground each: the rates fall with the square of the size.
    let per = p.density * dt_myr / (p.size * p.size);
    let jobs = [
        (Kind::FoldTrain, 0usize, p.fold_rate_km2_myr),
        (Kind::Thrust, 0, p.thrust_rate_km2_myr),
        (Kind::Normal, 1, p.normal_rate_km2_myr),
        (Kind::StrikeSlip, 2, p.strike_slip_rate_km2_myr),
    ];
    for (kind, which, rate) in jobs {
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
            let s = strain(x, c, sp);
            // Draw the whole structure before the overlap test, so the random stream does not
            // depend on what is already there.
            let scatter = p.trend_scatter_deg * rng.normal();
            let flip = rng.uniform();
            let u = [rng.uniform(), rng.uniform(), rng.uniform(), rng.uniform()];
            let wavelength = length_blocks(
                p.fold_wavelength_km,
                p.fold_wavelength_sigma,
                km_to_blocks,
                rng,
            );
            let length = length_blocks(p.fault_length_km, p.fault_length_sigma, km_to_blocks, rng);
            let random_trend = 180.0 * rng.uniform();
            if !frame.projection().inside_poles(bz) {
                // Drawn past the pole (the grid's rows run on to ±Zo, past ±R): no ground there,
                // and the cube bake's Jacobian, singular at the pole, made such a fault a window
                // 10⁹ blocks long over the whole planet. Dropped after the whole structure is
                // drawn, so the stream and every other structure stay as they were; dropping
                // thins the draws to the strain on the planet, the rate it asks for.
                // ledger: DEF-081
                counts.past_pole += 1;
                continue;
            }
            let covering: Vec<usize> = features
                .iter()
                .enumerate()
                .filter(|(_, f)| f.kind == kind && f.covers(bx, bz, c_blocks))
                .map(|(k, _)| k)
                .collect();
            if covering.len() >= p.max_overlap as usize {
                // Room is made by the oldest ones, if enough of them are old enough to give way.
                let mut by_age = covering;
                by_age.sort_by(|&a, &b| features[a].t_myr.total_cmp(&features[b].t_myr));
                let needed = by_age.len() + 1 - p.max_overlap as usize;
                let gives_way = |k: usize| f64::from(features[k].t_myr) <= t_myr - p.overprint_myr;
                if !by_age[..needed].iter().all(|&k| gives_way(k)) {
                    counts.saturated += 1;
                    continue;
                }
                let mut gone: Vec<usize> = by_age[..needed].to_vec();
                gone.sort_unstable_by(|a, b| b.cmp(a));
                for k in gone {
                    features.remove(k);
                    counts.overprinted += 1;
                }
            }
            let lerp = |v: [f64; 2], t: f64| v[0] + (v[1] - v[0]) * t;
            let mut f = Feature {
                id: *next_id,
                kind,
                x: bx,
                z: bz,
                plate: x.plate_id[c].max(0.0) as u32,
                t_myr: t_myr as f32,
                azimuth_deg: 0.0,
                half_along: 0.0,
                half_across: 0.0,
                wavelength: 0.0,
                amplitude: 0.0,
                phase: 0.0,
                dip_deg: 0.0,
                slip: 0.0,
                cause: cause(kind, bx, bz),
                adrift: 0,
                wear_m: 0.0,
                rise: 0.0,
                depth: 0.0,
                base: 0.0,
                rock: 0,
            };
            match kind {
                Kind::FoldTrain => {
                    let trend = trend_near(x, c).unwrap_or(random_trend);
                    // Shortening from the thickened crust, between the least and the most, and
                    // less where the strain is weaker (forelands).
                    let [e_lo, e_hi] = p.fold_shortening;
                    let thick = f64::from(x.thickness[c]).max(1.0);
                    // ledger: DEF-006
                    let e_crust = (1.0 - sp.reference_crust_m / thick).clamp(e_lo, e_hi);
                    // In a collision's belt, wedge and foreland the collision's own shortening
                    // S, taken up over the belt's width W: e = S / (S + W). ledger: DEF-077
                    let shortened = f64::from(x.shortening_km[c]);
                    let e_target = match s.source {
                        Source::Collision | Source::Basin if shortened > 0.0 => {
                            (shortened / (shortened + x.collision_belt_km)).clamp(e_lo, e_hi)
                        }
                        _ => e_crust,
                    };
                    let e = e_lo + (e_target - e_lo) * s.tightness.min(1.0);
                    f.azimuth_deg = (trend + scatter).rem_euclid(360.0) as f32;
                    f.wavelength = wavelength as f32;
                    // ledger: DEF-007, DEF-011
                    f.amplitude = (stretch_x * p.strength * wavelength * e.sqrt()
                        / std::f64::consts::PI) as f32;
                    f.half_across = (0.5 * lerp(p.fold_train_waves, u[0]) * wavelength) as f32;
                    f.half_along = (0.5 * lerp(p.fold_length_ratio, u[1]) * wavelength) as f32;
                    f.phase = u[2] as f32;
                }
                Kind::Thrust => {
                    let trend = trend_near(x, c).unwrap_or(random_trend);
                    let mut az = trend + scatter;
                    // The ramp dips under the high ground (the hinterland): in a foreland,
                    // towards the orogen; inside the orogen, up the regional slope.
                    let towards = {
                        let g = gradient(grid, x.orogen_km, c, 3);
                        if g[0] != 0.0 || g[1] != 0.0 {
                            [-g[0], -g[1]]
                        } else {
                            gradient(grid, x.elevation, c, 3)
                        }
                    };
                    let across = direction(az + 90.0);
                    let side = across[0] * towards[0] + across[1] * towards[1];
                    if side < 0.0 || (side == 0.0 && flip < 0.5) {
                        az += 180.0;
                    }
                    f.azimuth_deg = az.rem_euclid(360.0) as f32;
                    f.half_along = (0.5 * length) as f32;
                    f.half_across = (p.fault_reach_ratio * length) as f32;
                    let grow = 0.5 + 0.5 * s.tightness.min(1.0); // ledger: DEF-017
                    let (dip, slip) = stretch(
                        lerp(p.thrust_dip_deg, u[0]),
                        p.strength * p.thrust_slip_ratio * length * grow,
                        stretch_x,
                    );
                    f.dip_deg = dip as f32;
                    f.slip = -slip as f32;
                }
                Kind::Normal => {
                    let rift = |k: usize| {
                        x.regime[k] as u8 == regime::RIFT && x.crust_type[k] as u8 == CONTINENTAL
                    };
                    let (trend, centre) =
                        mask_axis(grid, c, 6, rift).unwrap_or((random_trend, [0.0, 0.0]));
                    let mut az = trend + scatter;
                    // Dipping towards the rift's axis; one in four antithetic.
                    let across = direction(az + 90.0);
                    let side = across[0] * centre[0] + across[1] * centre[1];
                    let synthetic = if side.abs() < 0.5 * r {
                        flip < 0.5
                    } else {
                        side > 0.0
                    };
                    if synthetic == (u[3] < 0.25) {
                        // ledger: DEF-031
                        az += 180.0;
                    }
                    f.azimuth_deg = az.rem_euclid(360.0) as f32;
                    f.half_along = (0.5 * length) as f32;
                    f.half_across = (p.fault_reach_ratio * length) as f32;
                    let (dip, slip) = stretch(
                        lerp(p.normal_dip_deg, u[0]),
                        p.strength * p.normal_slip_ratio * length,
                        stretch_x,
                    );
                    f.dip_deg = dip as f32;
                    f.slip = slip as f32;
                }
                Kind::StrikeSlip => {
                    let trend = trend_near(x, c).unwrap_or(random_trend);
                    f.azimuth_deg = (trend + scatter).rem_euclid(360.0) as f32;
                    f.dip_deg = 90.0; // ledger: DEF-021
                    f.half_along = (0.5 * length) as f32;
                    f.half_across = (p.fault_reach_ratio * length) as f32;
                    let sense = if flip < 0.5 { -1.0 } else { 1.0 };
                    f.slip = (sense * p.strength * p.strike_slip_ratio * length) as f32;
                }
                // Intrusions are `intrude.rs`'s.
                Kind::Stock | Kind::Dyke | Kind::Plug => continue,
            }
            *next_id += 1;
            counts.add(kind);
            counts.add_source(kind, s.source);
            features.push(f);
            born += 1;
        }
    }
    born
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use deeptime_format::structure::Shape;

    /// A small planet with a north–south collision belt (8 cells wide at columns 56 to 63, its
    /// wedge-less foreland basin on the incoming side at 64 to 79 and a forebulge at 80 to 89), a
    /// rift and an oblique margin.
    pub(crate) struct Planet {
        pub grid: GridDesc,
        pub regime: Vec<f32>,
        pub crust: Vec<f32>,
        /// The collision's melt window: its belt.
        pub melt: Vec<bool>,
        pub flags: Vec<f32>,
        pub convergence: Vec<f32>,
        pub obliquity: Vec<f32>,
        pub thickness: Vec<f32>,
        pub trend: Vec<f32>,
        pub plate: Vec<f32>,
        pub uplift: Vec<f32>,
        pub elevation: Vec<f32>,
        pub orogen_km: Vec<f32>,
        pub arc_km: Vec<f32>,
        /// The collision's convergence across its front, mm/yr, in its footprint (the belt and
        /// the foreland on the incoming side), and its shortening so far, km.
        pub collision: Vec<f32>,
        pub shortened: Vec<f32>,
        pub flexure: Option<Vec<f32>>,
        /// Metres a block is high (300 m across).
        pub m_per_block_v: f64,
    }

    impl Planet {
        pub fn new() -> Planet {
            let grid = GridDesc::for_planet("L1", 4096, 16).unwrap();
            let n = grid.cells();
            let w = grid.w as usize;
            let mut p = Planet {
                grid,
                regime: vec![f32::from(regime::CRATON); n],
                crust: vec![f32::from(CONTINENTAL); n],
                melt: vec![false; n],
                flags: vec![0.0; n],
                convergence: vec![0.0; n],
                obliquity: vec![0.0; n],
                thickness: vec![35_000.0; n],
                trend: vec![f32::NAN; n],
                plate: vec![0.0; n],
                uplift: vec![0.0; n],
                elevation: vec![300.0; n],
                orogen_km: vec![3000.0; n],
                arc_km: vec![3000.0; n],
                collision: vec![0.0; n],
                shortened: vec![0.0; n],
                flexure: Some(vec![0.0; n]),
                m_per_block_v: 20.0,
            };
            for c in 0..n {
                let (i, j) = (c % w, c / w);
                // Everything runs north-south, between 69° north and south.
                if !(30..226).contains(&j) {
                    continue;
                }
                // A collision belt 8 cells wide along i = 60, trending north (trend 0°), its
                // crest at i = 60; the foreland east of it.
                if (56..64).contains(&i) {
                    p.regime[c] = f32::from(regime::COLLISION);
                    p.melt[c] = true;
                    p.thickness[c] = 55_000.0;
                    p.trend[c] = 0.0;
                    p.orogen_km[c] = 0.0;
                    p.elevation[c] = 4000.0 - 400.0 * (i as f32 - 60.0).abs();
                    p.collision[c] = 30.0;
                    p.shortened[c] = 200.0;
                } else if (64..100).contains(&i) {
                    p.orogen_km[c] = (i as f32 - 63.0) * 4.8;
                    p.trend[c] = 0.0;
                }
                // The incoming plate's foreland: a basin 500 m below local isostasy beside the
                // belt (columns 64 to 79), then a forebulge 80 m above it (80 to 89).
                if (64..90).contains(&i) {
                    p.collision[c] = 30.0;
                    p.shortened[c] = 200.0;
                    p.flags[c] = f32::from(FORELAND);
                    if let Some(f) = p.flexure.as_mut() {
                        f[c] = if i < 80 { -500.0 } else { 80.0 };
                    }
                }
                // A rift along i = 180, 4 cells wide, trending north.
                if (178..182).contains(&i) {
                    p.regime[c] = f32::from(regime::RIFT);
                }
            }
            p
        }

        pub fn inputs(&self) -> Inputs<'_> {
            Inputs {
                grid: &self.grid,
                m_per_block_h: 300.0,
                m_per_block_v: self.m_per_block_v,
                regime: &self.regime,
                crust_type: &self.crust,
                melt: &self.melt,
                flags: &self.flags,
                convergence: &self.convergence,
                obliquity: &self.obliquity,
                thickness: &self.thickness,
                fold_trend: &self.trend,
                plate_id: &self.plate,
                uplift: &self.uplift,
                elevation: &self.elevation,
                orogen_km: &self.orogen_km,
                arc_km: &self.arc_km,
                collision_mm_yr: &self.collision,
                shortening_km: &self.shortened,
                flexure_m: self.flexure.as_deref(),
                collision_belt_km: 400.0,
                // The default 140 km of foreland is 336 blocks here (21 cells).
                blocks_per_model_km: 2.4,
            }
        }

        /// Runs `steps` steps of 2 Myr and returns everything emplaced.
        pub fn run(&self, sp: &StructureParams, steps: u32, seed: u64) -> (Vec<Feature>, Counts) {
            let mut all = Vec::new();
            let mut counts = Counts::default();
            let mut next = 0u32;
            for s in 0..steps {
                let mut rng = Rng::stream(seed ^ (u64::from(s) << 20), "structure/deform");
                emplace(
                    &self.inputs(),
                    sp,
                    &mut all,
                    2.0,
                    -100.0 + 2.0 * f64::from(s),
                    &mut rng,
                    &mut next,
                    &mut counts,
                    &|_, _, _| 0,
                );
            }
            (all, counts)
        }
    }

    #[test]
    fn strain_follows_the_regime() {
        let p = Planet::new();
        let sp = StructureParams::default();
        let w = p.grid.w as usize;
        let at = |i: usize, j: usize| strain(&p.inputs(), j * w + i, &sp);
        // A collision at the reference convergence shortens at full strength, with the plate
        // model's shortening behind the fold tightness.
        assert_eq!(at(60, 60).shortening, 1.0);
        assert_eq!(at(60, 60).source, Source::Collision);
        assert_eq!(at(60, 60).extension, 0.0);
        // The foreland basin on the incoming side shortens at 0.6; the forebulge and the plain
        // beyond it not at all (no orogen distance reaches them any more).
        assert_eq!(at(70, 60).source, Source::Basin);
        assert!((at(70, 60).shortening - 0.6).abs() < 1e-9);
        assert_eq!(at(85, 60).shortening, 0.0, "the forebulge is undeformed");
        assert_eq!(at(95, 60).shortening, 0.0, "beyond the collision's reach");
        assert_eq!(at(140, 60).shortening, 0.0, "the craton is undeformed");
        assert_eq!(at(180, 60).extension, 1.0);
        // Oceanic crust is never deformed here.
        let mut q = Planet::new();
        q.crust[60 * w + 60] = 0.0;
        assert_eq!(strain(&q.inputs(), 60 * w + 60, &sp), Strain::default());
        // Oblique convergence in a subduction zone's reach: shear.
        q.flags[60 * w + 30] = f32::from(SUBDUCTION_ZONE);
        q.obliquity[60 * w + 30] = 75.0;
        q.convergence[60 * w + 30] = 80.0;
        q.uplift[60 * w + 30] = 0.2;
        assert_eq!(strain(&q.inputs(), 60 * w + 30, &sp).shear, 1.0);
        // A back-arc shortens only where the plate model is lifting it.
        let c = 60 * w + 20;
        q.regime[c] = f32::from(regime::BACK_ARC);
        q.convergence[c] = 80.0;
        assert_eq!(strain(&q.inputs(), c, &sp).shortening, 0.0);
        q.uplift[c] = 0.05;
        assert!((strain(&q.inputs(), c, &sp).shortening - 0.25).abs() < 1e-6);
        q.uplift[c] = 1.0;
        assert_eq!(strain(&q.inputs(), c, &sp).shortening, 0.5);
    }

    /// A collision shortens at the convergence across its front (plate task 6): a slowing
    /// collision makes fewer structures, a stopped one none, and one faster than the reference
    /// no more than the reference (the old rule drew the collision regime at full strength
    /// whatever the convergence).
    #[test]
    fn a_slowing_collision_makes_fewer_structures() {
        let sp = StructureParams::default();
        let reference = sp.deformation.collision_reference_mm_yr as f32;
        let count = |conv: f32| {
            let mut p = Planet::new();
            let w = p.grid.w as usize;
            for c in 0..p.collision.len() {
                if p.collision[c] > 0.0 {
                    p.collision[c] = conv;
                }
            }
            let at = |p: &Planet, i: usize| strain(&p.inputs(), 60 * w + i, &sp).shortening;
            let rates = (at(&p, 60), at(&p, 70));
            let mut dense = sp.clone();
            dense.deformation.density = 20.0;
            dense.deformation.max_overlap = 16;
            let (all, counts) = p.run(&dense, 10, 11);
            let shortening: u64 = counts.fold_trains + counts.thrusts;
            (rates, all.len(), shortening)
        };
        let (r_full, _, n_full) = count(reference);
        let (r_half, _, n_half) = count(0.5 * reference);
        let (r_fast, _, n_fast) = count(5.0 * reference);
        let (r_stop, _, n_stop) = count(0.0);
        assert_eq!(r_full.0, 1.0);
        assert!(
            (r_half.0 - 0.5).abs() < 1e-6 && (r_half.1 - 0.3).abs() < 1e-6,
            "{r_half:?}"
        );
        assert_eq!(r_fast, r_full, "no more than the reference");
        assert_eq!(
            r_stop,
            (0.0, 0.0),
            "a collision that stopped shortens nothing"
        );
        assert!(n_full > 30, "{n_full}");
        assert!(
            n_half < n_full * 3 / 4 && n_half > n_full / 4,
            "half the convergence, about half the structures: {n_half} against {n_full}"
        );
        assert_eq!(n_fast, n_full, "the same draws");
        assert_eq!(n_stop, 0, "{n_stop}");
    }

    /// A collision's folds are as tight as its real shortening says (plate task 6): a collision
    /// that has shortened by 20 km over a 400 km belt folds the cover by 5 % (the least
    /// the settings give), one that has shortened by 600 km by 30 % (the most), at the same crust
    /// thickness (the old rule read only the thickness).
    #[test]
    fn fold_tightness_follows_the_real_shortening() {
        let sp = StructureParams::default();
        let [e_lo, e_hi] = sp.deformation.fold_shortening;
        let mean_ratio = |km: f32| {
            let mut p = Planet::new();
            for v in p.shortened.iter_mut().filter(|v| **v > 0.0) {
                *v = km;
            }
            let mut dense = sp.clone();
            dense.deformation.density = 30.0;
            dense.deformation.max_overlap = 16;
            let (all, _) = p.run(&dense, 10, 5);
            let x = sp.deformation.block_stretch(300.0, p.m_per_block_v) as f32;
            let folds: Vec<f64> = all
                .iter()
                .filter(|f| f.kind == Kind::FoldTrain)
                .map(|f| f64::from(f.amplitude / f.wavelength / x))
                .collect();
            assert!(folds.len() > 20, "{}", folds.len());
            folds.iter().sum::<f64>() / folds.len() as f64
        };
        let (young, middle, old) = (mean_ratio(20.0), mean_ratio(100.0), mean_ratio(600.0));
        // A/lambda = sqrt(e) / pi at the cell's strain e: the belt's cells (tightness 1) at the
        // end of the range, the basin's (0.6 of the way between the least and the most) below.
        let a = |e: f64| e.sqrt() / std::f64::consts::PI;
        assert!(young < middle && middle < old, "{young} {middle} {old}");
        assert!(
            old < a(e_hi) * 1.0001 && young > a(e_lo) * 0.9999,
            "{young} {old}"
        );
        // 100 km over a 400 km belt is 20 %: between the least and the most.
        assert!(middle > a(0.15) && middle < a(0.25), "{middle}");
    }

    /// The foreland of a collision is the flexural basin on the incoming plate (plate task 6):
    /// folds and thrusts in the basin, none on the forebulge or beyond it, and none at all in the
    /// foreland of a world with no flexure field; the old rule gave every cell within
    /// `foreland_km` of an orogen, on both sides.
    #[test]
    fn the_foreland_is_the_basin() {
        let mut sp = StructureParams::default();
        sp.deformation.density = 30.0;
        sp.deformation.max_overlap = 16;
        let p = Planet::new();
        let (all, counts) = p.run(&sp, 15, 7);
        let col =
            |f: &Feature| crate::structure::plutons::cell_of(&p.grid, f.x, f.z) % p.grid.w as usize;
        let shortened: Vec<&Feature> = all
            .iter()
            .filter(|f| matches!(f.kind, Kind::FoldTrain | Kind::Thrust))
            .collect();
        let in_belt = shortened
            .iter()
            .filter(|f| (55..=64).contains(&col(f)))
            .count();
        let in_basin = shortened
            .iter()
            .filter(|f| (64..=80).contains(&col(f)))
            .count();
        let beyond = shortened.iter().filter(|f| col(f) > 80).count();
        assert!(in_belt > 20 && in_basin > 20, "{in_belt} {in_basin}");
        assert_eq!(
            beyond, 0,
            "the forebulge and the plain beyond are undeformed"
        );
        assert!(
            counts.shortened_by[3][0] > 5 && counts.shortened_by[3][1] > 5,
            "{counts:?}"
        );
        // The basin's cells are the foreland and their strain is gentler than the belt's.
        let mut dry = Planet::new();
        dry.flexure = None;
        let (all, counts) = dry.run(&sp, 15, 7);
        assert!(
            all.iter()
                .filter(|f| matches!(f.kind, Kind::FoldTrain | Kind::Thrust))
                .all(|f| col(f) <= 64),
            "no flexure, no basin: the foreland has no structures"
        );
        assert_eq!(counts.shortened_by[3], [0, 0]);
        // The behind-the-belt (overriding) side: no foreland (the plate's own flags say which
        // side is the incoming one).
        let overriding = all
            .iter()
            .filter(|f| col(f) < 55 && f.kind != Kind::Normal)
            .count();
        assert_eq!(overriding, 0, "{overriding}");
    }

    /// The foreland of an Andean margin keeps the distance rule (`foreland_km` from a continental
    /// arc), and a collision's belt does not give one (plate task 6).
    #[test]
    fn an_andean_foreland_fades_with_the_distance_from_the_arc() {
        let mut p = Planet::new();
        let sp = StructureParams::default();
        let w = p.grid.w as usize;
        // An arc along column 120 and its foreland east of it (4.8 km a cell).
        let rows = p.grid.h as usize;
        for j in 30..rows {
            for i in 100..140 {
                p.arc_km[j * w + i] = (i as f32 - 120.0).abs() * 4.8;
            }
        }
        let at = |p: &Planet, i: usize| strain(&p.inputs(), 60 * w + i, &sp);
        let near = at(&p, 122);
        let far = at(&p, 139);
        assert_eq!(near.source, Source::Foreland);
        assert!(
            near.shortening > far.shortening && far.shortening > 0.0,
            "{near:?} {far:?}"
        );
        assert!(near.shortening < 0.6, "{near:?}");
        // The default belt is the Subandean 140 km: 150 km from the arc there is no foreland.
        assert_eq!(sp.deformation.foreland_km, 140.0);
        p.arc_km[60 * w + 150] = 150.0;
        assert_eq!(at(&p, 150).shortening, 0.0, "{:?}", at(&p, 150));
        // The collision regime's own orogen distance (the old rule's input) no longer makes one.
        for j in 30..rows {
            for i in 64..100 {
                p.collision[j * w + i] = 0.0;
                p.flags[j * w + i] = 0.0;
            }
        }
        assert_eq!(at(&p, 70).shortening, 0.0, "{:?}", at(&p, 70));
    }

    #[test]
    fn structures_form_where_the_strain_is_and_follow_its_trend() {
        let p = Planet::new();
        let mut sp = StructureParams::default();
        // Many small structures, so the statistics are tight.
        sp.deformation.density = 30.0;
        sp.deformation.max_overlap = 16;
        let (all, counts) = p.run(&sp, 15, 7);
        assert!(counts.fold_trains > 20 && counts.thrusts > 20 && counts.normal_faults > 20);
        assert_eq!(
            counts.strike_slip_faults, 0,
            "no oblique margin on this planet"
        );
        let cell = |f: &Feature| {
            let c = crate::structure::plutons::cell_of(&p.grid, f.x, f.z);
            (c % p.grid.w as usize, c / p.grid.w as usize)
        };
        let off_axis = |a: f32, axis: f32| {
            let d = (a - axis).rem_euclid(180.0);
            d.min(180.0 - d)
        };
        for f in &all {
            let (i, j) = cell(f);
            assert!((29..=226).contains(&j), "row {j}");
            match f.kind {
                Kind::FoldTrain | Kind::Thrust => {
                    assert!((55..=80).contains(&i), "{:?} at column {i}", f.kind);
                    // Along the belt (north), within the scatter.
                    assert!(off_axis(f.azimuth_deg, 0.0) < 50.0, "{}", f.azimuth_deg);
                }
                Kind::Normal => {
                    assert!((177..=182).contains(&i), "normal fault at column {i}");
                    assert!(off_axis(f.azimuth_deg, 0.0) < 50.0, "{}", f.azimuth_deg);
                    // Anderson's 55° to 70°, steepened by the vertical scale.
                    let x = sp.deformation.block_stretch(300.0, p.m_per_block_v);
                    let (lo, hi) = (stretch(55.0, 1.0, x).0, stretch(70.0, 1.0, x).0);
                    let dip = f64::from(f.dip_deg);
                    assert!(
                        f.slip > 0.0 && dip >= lo - 1e-3 && dip <= hi + 1e-3,
                        "{dip}"
                    );
                }
                _ => unreachable!(),
            }
            assert!(f.half_along > 0.0 && f.half_across > 0.0);
        }
        // Thrusts dip under the high ground: east of the crest they dip west, and in the
        // foreland towards the orogen (west). Dip direction = azimuth + 90°.
        let (mut right, mut wrong) = (0, 0);
        for f in all.iter().filter(|f| f.kind == Kind::Thrust) {
            let (i, j) = cell(f);
            // (At the ends of the belt the high ground is as much along the strike as across.)
            if !(36..220).contains(&j) {
                continue;
            }
            let dip_dir = direction(f64::from(f.azimuth_deg) + 90.0);
            let west = dip_dir[0] < 0.0;
            // (The cells next to the crest are left out: a structure is jittered within its
            // cell, and on the crest itself the ground is level.)
            if i >= 62 {
                if west { right += 1 } else { wrong += 1 }
            } else if i <= 57 {
                if west { wrong += 1 } else { right += 1 }
            }
            assert!(f.slip < 0.0, "a thrust's hanging wall goes up");
        }
        assert!(right > 10 && wrong == 0, "{right} right, {wrong} wrong");
        // Folds are tighter in the thickened core than in the foreland (amplitude over
        // wavelength before the vertical scale).
        let x = sp.deformation.block_stretch(300.0, p.m_per_block_v) as f32;
        let ratio = |f: &Feature| f.amplitude / f.wavelength / x;
        let mean = |v: Vec<f32>| v.iter().sum::<f32>() / v.len().max(1) as f32;
        let core = mean(
            all.iter()
                .filter(|f| f.kind == Kind::FoldTrain && (56..64).contains(&cell(f).0))
                .map(ratio)
                .collect(),
        );
        let foreland = mean(
            all.iter()
                .filter(|f| f.kind == Kind::FoldTrain && (68..=80).contains(&cell(f).0))
                .map(ratio)
                .collect(),
        );
        assert!(
            core > foreland && foreland > 0.05 && core < 0.2,
            "{core} {foreland}"
        );
        // Normal faults mostly dip towards the rift's axis (i = 179.5): three in four (DEF-031),
        // about 2.8 to 1 as counted here. Counted over eight runs: one run gives about 48 faults
        // away from the axis, which puts the 2:1 bound one standard deviation below the
        // expected share, and the bound failed on 4 of seeds 1 to 40 (5 of 40 once draws past
        // the pole were dropped, DEF-081). Over eight runs it is three below.
        let (mut inward, mut outward) = (0, 0);
        let mut count = |run: &[Feature]| {
            for f in run.iter().filter(|f| f.kind == Kind::Normal) {
                let axis_x = p.grid.block_x(179) as f64 + 8.0;
                let dip_dir = direction(f64::from(f.azimuth_deg) + 90.0);
                let to_axis = axis_x - f.x;
                // (Within a cell of the axis the side is a coin toss.)
                if to_axis.abs() < 20.0 {
                    continue;
                }
                if dip_dir[0] * to_axis > 0.0 {
                    inward += 1
                } else {
                    outward += 1
                }
            }
        };
        count(&all);
        for seed in 8..15 {
            count(&p.run(&sp, 15, seed).0);
        }
        assert!(inward > 2 * outward, "{inward} inward, {outward} outward");
    }

    #[test]
    fn strength_density_and_overlap_do_what_they_say() {
        let p = Planet::new();
        let sp = StructureParams::default();
        let (base, _) = p.run(&sp, 25, 3);
        // Off: nothing.
        let mut off = sp.clone();
        off.deformation.strength = 0.0;
        assert!(p.run(&off, 25, 3).0.is_empty());
        // Twice the strength: the same structures (the same draws), twice the amplitude and slip.
        let mut strong = sp.clone();
        strong.deformation.strength = 2.0;
        let (twice, _) = p.run(&strong, 25, 3);
        assert_eq!(base.len(), twice.len());
        for (a, b) in base.iter().zip(&twice) {
            assert_eq!(
                (a.x, a.z, a.kind, a.azimuth_deg),
                (b.x, b.z, b.kind, b.azimuth_deg)
            );
            assert!((b.amplitude - 2.0 * a.amplitude).abs() <= 1e-4 * a.amplitude.abs());
            assert!((b.slip - 2.0 * a.slip).abs() <= 1e-4 * a.slip.abs());
        }
        // More density: more structures; the overlap rule holds everywhere.
        let mut dense = sp.clone();
        dense.deformation.density = 20.0;
        let (many, counts) = p.run(&dense, 25, 3);
        assert!(
            many.len() > 2 * base.len().max(1),
            "{} vs {}",
            many.len(),
            base.len()
        );
        assert!(counts.saturated > 0);
        for f in &many {
            let over = many
                .iter()
                .filter(|g| g.kind == f.kind && g.id < f.id && g.covers(f.x, f.z, 4096.0))
                .count();
            assert!(
                over < dense.deformation.max_overlap as usize,
                "{over} under {}",
                f.id
            );
        }
        // The same seed gives the same structures.
        assert_eq!(p.run(&sp, 25, 3).0, base);
    }

    /// A structure drawn past a pole is not started (DEF-081). The grid's rows run on past ±R to
    /// ±Zo (R 651.9 and Zo 704 blocks on this 4,096-block planet), and this planet's belt and rift
    /// reach the rows wholly past the south pole. On master c78cfb6f's 4k test planet three faults
    /// drawn at z 657 to 695 baked as windows 10⁹ to 10¹¹ blocks long, their hanging walls read
    /// from 10⁹ blocks away (FarColumnTest).
    #[test]
    fn nothing_is_started_past_the_poles() {
        let p = Planet::new();
        let pole = deeptime_format::Projection::new(4096).pole_z();
        let (w, r) = (p.grid.w as usize, f64::from(p.grid.r));
        assert!(
            (0..p.grid.cells()).any(|c| p.regime[c] as u8 == regime::COLLISION
                && p.grid.block_z((c / w) as i64) as f64 - 0.5 * r > pole),
            "the fixture's belt reaches a row wholly past the pole"
        );
        let mut sp = StructureParams::default();
        sp.deformation.density = 30.0;
        sp.deformation.max_overlap = 16;
        sp.deformation.intrusions = 10.0;
        let (mut all, mut counts) = p.run(&sp, 15, 7);
        assert!(counts.past_pole > 0, "{counts:?}");
        let faults = counts.past_pole;
        let mut next = all.len() as u32;
        for s in 0..15u32 {
            let mut rng = Rng::stream(11 ^ (u64::from(s) << 20), "structure/intrude");
            super::super::intrude::emplace(
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
        assert!(counts.past_pole > faults, "{counts:?}");
        assert!(counts.stocks > 10 && counts.dykes > 10, "{counts:?}");
        for f in &all {
            assert!(f.z.abs() < pole, "{:?} {} at z {}", f.kind, f.id, f.z);
        }
    }

    /// The stretch (`true_height_share`, here as X times the 1:1 shape on a 300 m by 20 m block)
    /// raises structures and nothing else: the same structures (the
    /// same draws), fold amplitudes and fault throws times the scale, fault heaves unchanged,
    /// dips steepened to atan(X tan d), and the baked slip vector's vertical part times the
    /// scale with its horizontal part unchanged (what every reader of the column rule sees).
    #[test]
    fn the_stretch_raises_heights_only() {
        let p = Planet::new();
        let mut one = StructureParams::default();
        // A share of 20 / 300 of true metres is the shape at 1:1 on a 300 m by 20 m block.
        one.deformation.true_height_share = 20.0 / 300.0;
        let (base, _) = p.run(&one, 25, 3);
        assert!(base.iter().any(|f| f.kind == Kind::Normal));
        assert!(base.iter().any(|f| f.kind == Kind::Thrust));
        assert!(base.iter().any(|f| f.kind == Kind::FoldTrain));
        // At 1 the dips are Anderson's, as drawn in blocks.
        for f in &base {
            let d = f64::from(f.dip_deg);
            match f.kind {
                Kind::Normal => assert!((54.99..=70.01).contains(&d), "{d}"),
                Kind::Thrust => assert!((19.99..=40.01).contains(&d), "{d}"),
                _ => {}
            }
        }
        for x in [4.0, 15.0] {
            let mut tall = one.clone();
            tall.deformation.true_height_share = x * 20.0 / 300.0;
            let (stretched, _) = p.run(&tall, 25, 3);
            assert_eq!(base.len(), stretched.len());
            for (a, b) in base.iter().zip(&stretched) {
                assert_eq!(
                    (
                        a.x,
                        a.z,
                        a.kind,
                        a.azimuth_deg,
                        a.wavelength,
                        a.half_along,
                        a.half_across
                    ),
                    (
                        b.x,
                        b.z,
                        b.kind,
                        b.azimuth_deg,
                        b.wavelength,
                        b.half_along,
                        b.half_across
                    )
                );
                let close = |u: f64, v: f64| (u - v).abs() <= 1e-4 * v.abs().max(1.0);
                match a.kind {
                    Kind::FoldTrain => {
                        assert!(close(f64::from(b.amplitude), x * f64::from(a.amplitude)));
                    }
                    Kind::Thrust | Kind::Normal => {
                        let parts = |f: &Feature| {
                            let (s, c) = f64::from(f.dip_deg).to_radians().sin_cos();
                            (f64::from(f.slip) * c, f64::from(f.slip) * s)
                        };
                        let ((h0, t0), (h1, t1)) = (parts(a), parts(b));
                        assert!(close(h1, h0), "heave {h1} vs {h0}");
                        assert!(close(t1, x * t0), "throw {t1} vs {x} x {t0}");
                        let want = (x * f64::from(a.dip_deg).to_radians().tan()).atan();
                        assert!((f64::from(b.dip_deg).to_radians() - want).abs() < 1e-5);
                        // Baked: the hanging wall's displacement, (x, y, z) in blocks.
                        let slip = |f: &Feature| match f.to_baked(70.0, 1, -330.0).shape {
                            Shape::Fault { slip, .. } => slip,
                            s => panic!("{s:?}"),
                        };
                        let (v0, v1) = (slip(a), slip(b));
                        assert!(close(v1[0].hypot(v1[2]), v0[0].hypot(v0[2])));
                        assert!(close(v1[1], x * v0[1]), "{} vs {x} x {}", v1[1], v0[1]);
                    }
                    _ => {}
                }
            }
        }
    }

    /// The share of true metres holds at any vertical scale: on blocks 35 m high instead of 20 m
    /// the same structures are drawn 20/35 as many blocks tall, so as many metres tall, with the
    /// same heaves (the world at 35 m a block, decisions log 2026-10-02).
    #[test]
    fn the_true_height_share_holds_at_any_block_height() {
        let sp = StructureParams::default();
        assert_eq!(sp.deformation.block_stretch(300.0, 20.0), 4.0);
        let mut p35 = Planet::new();
        p35.m_per_block_v = 35.0;
        let (a, _) = Planet::new().run(&sp, 25, 3);
        let (b, _) = p35.run(&sp, 25, 3);
        assert_eq!(a.len(), b.len());
        let close = |u: f64, v: f64| (u - v).abs() <= 1e-4 * v.abs().max(1.0);
        let (mut folds, mut faults) = (0, 0);
        for (a, b) in a.iter().zip(&b) {
            assert_eq!(
                (a.x, a.z, a.kind, a.wavelength),
                (b.x, b.z, b.kind, b.wavelength)
            );
            match a.kind {
                Kind::FoldTrain => {
                    folds += 1;
                    let (m20, m35) = (20.0 * f64::from(a.amplitude), 35.0 * f64::from(b.amplitude));
                    assert!(close(m35, m20), "fold {m35} m vs {m20} m");
                }
                Kind::Thrust | Kind::Normal => {
                    faults += 1;
                    // (heave, throw) in metres.
                    let metres = |f: &Feature, v: f64| {
                        let (s, c) = f64::from(f.dip_deg).to_radians().sin_cos();
                        (300.0 * f64::from(f.slip) * c, v * f64::from(f.slip) * s)
                    };
                    let ((h20, t20), (h35, t35)) = (metres(a, 20.0), metres(b, 35.0));
                    assert!(close(h35, h20), "heave {h35} m vs {h20} m");
                    assert!(close(t35, t20), "throw {t35} m vs {t20} m");
                }
                _ => {}
            }
        }
        assert!(folds > 0 && faults > 0);
    }

    #[test]
    fn a_mask_has_an_axis() {
        let grid = GridDesc::for_planet("L1", 4096, 16).unwrap();
        let w = grid.w as usize;
        // A band running east-west.
        let (az, centre) = mask_axis(&grid, 50 * w + 50, 6, |k| (k / w).abs_diff(52) <= 1).unwrap();
        assert!((az - 90.0).abs() < 1.0, "{az}");
        assert!(
            centre[0].abs() < 1.0 && (centre[1] - 32.0).abs() < 1.0,
            "{centre:?}"
        );
        // A diagonal band (south-east: +x with +z) trends 135°.
        let (az, _) = mask_axis(&grid, 50 * w + 50, 6, |k| (k % w).abs_diff(k / w) <= 1).unwrap();
        assert!((az - 135.0).abs() < 3.0, "{az}");
        // A blob has none.
        assert!(mask_axis(&grid, 50 * w + 50, 6, |_| true).is_none());
        assert!(mask_axis(&grid, 50 * w + 50, 6, |_| false).is_none());
    }
}
