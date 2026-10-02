// SPDX-License-Identifier: LGPL-3.0-or-later
//! From plate tectonics to structures: each plate step, the regimes the plate model reports are
//! translated into the folds and faults they make (plan §8.5: "plate events become local events").
//!
//! | Where (per L1 cell, continental crust) | Strain | What forms |
//! |---|---|---|
//! | the collision regime (a Himalayan orogeny in progress) | shortening, full strength | fold trains and thrusts along the plate model's fold trend, thrusts dipping under the high ground |
//! | a continental arc and its back-arc where the plate model is lifting the crust (an Andean orogeny) | shortening, half strength at 80 mm/yr of convergence and `arc_uplift_mm_yr` of uplift | the same, dipping under the high ground |
//! | the foreland, out to `foreland_km` from an active orogen | shortening, fading with distance | gentler folds and thrusts (the Zagros, the Subandean belt) |
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
use crate::plates::state::{ARC, CONTINENTAL, SUBDUCTION_ZONE, regime};
use crate::sphere::BlockFrame;
use crate::structure::params::StructureParams;

/// What a step's translation reads, per L1 cell.
pub struct Inputs<'a> {
    pub grid: &'a GridDesc,
    pub m_per_block_h: f64,
    pub regime: &'a [f32],
    pub crust_type: &'a [f32],
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
    /// (`traps::orogen_distance_km`).
    pub orogen_km: &'a [f32],
    /// Blocks per km of the plate model's sphere (C / 2π·`tectonics.model_radius_km`): tectonic
    /// widths are Earth-normalized, like the plate model's own distances.
    pub blocks_per_model_km: f64,
}

/// The strain a cell is under: shortening, extension and shear, each 0–1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Strain {
    pub shortening: f64,
    pub extension: f64,
    pub shear: f64,
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
    let shortening = if reg == regime::COLLISION {
        1.0
    } else {
        // An Andean margin shortens where it is being lifted: the plate model's back-arc regime
        // reaches the whole subduction distance inland, most of it undeformed.
        let andean = if flags & ARC != 0 || reg == regime::ARC || reg == regime::BACK_ARC {
            let lift = f64::from(x.uplift[c].max(0.0)) / p.arc_uplift_mm_yr.max(1e-9);
            0.5 * speed * lift.min(1.0) // ledger: DEF-026
        } else {
            0.0
        };
        let d = f64::from(x.orogen_km[c]) * 1000.0 / x.m_per_block_h;
        let belt = p.foreland_km * x.blocks_per_model_km;
        let foreland = if d > 0.0 && d < belt {
            0.6 * (1.0 - d / belt) // ledger: DEF-027
        } else {
            0.0
        };
        andean.max(foreland)
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
        extension,
        shear,
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
                    let e = e_lo + (e_crust - e_lo) * s.shortening.min(1.0);
                    f.azimuth_deg = (trend + scatter).rem_euclid(360.0) as f32;
                    f.wavelength = wavelength as f32;
                    f.amplitude =
                        (p.strength * wavelength * e.sqrt() / std::f64::consts::PI) as f32;
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
                    f.dip_deg = lerp(p.thrust_dip_deg, u[0]) as f32;
                    f.half_along = (0.5 * length) as f32;
                    f.half_across = (p.fault_reach_ratio * length) as f32;
                    let grow = 0.5 + 0.5 * s.shortening.min(1.0); // ledger: DEF-017
                    f.slip = -(p.strength * p.thrust_slip_ratio * length * grow) as f32;
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
                    f.dip_deg = lerp(p.normal_dip_deg, u[0]) as f32;
                    f.half_along = (0.5 * length) as f32;
                    f.half_across = (p.fault_reach_ratio * length) as f32;
                    f.slip = (p.strength * p.normal_slip_ratio * length) as f32;
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
            features.push(f);
            born += 1;
        }
    }
    born
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A small planet with a north–south collision belt, a rift and an oblique margin.
    pub(crate) struct Planet {
        pub grid: GridDesc,
        pub regime: Vec<f32>,
        pub crust: Vec<f32>,
        pub flags: Vec<f32>,
        pub convergence: Vec<f32>,
        pub obliquity: Vec<f32>,
        pub thickness: Vec<f32>,
        pub trend: Vec<f32>,
        pub plate: Vec<f32>,
        pub uplift: Vec<f32>,
        pub elevation: Vec<f32>,
        pub orogen_km: Vec<f32>,
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
                flags: vec![0.0; n],
                convergence: vec![0.0; n],
                obliquity: vec![0.0; n],
                thickness: vec![35_000.0; n],
                trend: vec![f32::NAN; n],
                plate: vec![0.0; n],
                uplift: vec![0.0; n],
                elevation: vec![300.0; n],
                orogen_km: vec![3000.0; n],
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
                    p.thickness[c] = 55_000.0;
                    p.trend[c] = 0.0;
                    p.orogen_km[c] = 0.0;
                    p.elevation[c] = 4000.0 - 400.0 * (i as f32 - 60.0).abs();
                } else if (64..100).contains(&i) {
                    p.orogen_km[c] = (i as f32 - 63.0) * 4.8;
                    p.trend[c] = 0.0;
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
                regime: &self.regime,
                crust_type: &self.crust,
                flags: &self.flags,
                convergence: &self.convergence,
                obliquity: &self.obliquity,
                thickness: &self.thickness,
                fold_trend: &self.trend,
                plate_id: &self.plate,
                uplift: &self.uplift,
                elevation: &self.elevation,
                orogen_km: &self.orogen_km,
                // The default 250 km of foreland is 600 blocks here (37 cells).
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
        assert_eq!(at(60, 60).shortening, 1.0);
        assert_eq!(at(60, 60).extension, 0.0);
        // The foreland fades with distance from the orogen and ends at the belt's width.
        let near = at(66, 60).shortening;
        let far = at(100 - 1, 60).shortening;
        assert!(near > far && far > 0.0 && near < 0.6, "{near} {far}");
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
                    assert!((55..=100).contains(&i), "{:?} at column {i}", f.kind);
                    // Along the belt (north), within the scatter.
                    assert!(off_axis(f.azimuth_deg, 0.0) < 50.0, "{}", f.azimuth_deg);
                }
                Kind::Normal => {
                    assert!((177..=182).contains(&i), "normal fault at column {i}");
                    assert!(off_axis(f.azimuth_deg, 0.0) < 50.0, "{}", f.azimuth_deg);
                    assert!(f.slip > 0.0 && (55.0..=70.0).contains(&f.dip_deg));
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
        // Folds are tighter in the thickened core than in the foreland.
        let ratio = |f: &Feature| f.amplitude / f.wavelength;
        let mean = |v: Vec<f32>| v.iter().sum::<f32>() / v.len().max(1) as f32;
        let core = mean(
            all.iter()
                .filter(|f| f.kind == Kind::FoldTrain && (56..64).contains(&cell(f).0))
                .map(ratio)
                .collect(),
        );
        let foreland = mean(
            all.iter()
                .filter(|f| f.kind == Kind::FoldTrain && cell(f).0 >= 70)
                .map(ratio)
                .collect(),
        );
        assert!(
            core > foreland && foreland > 0.05 && core < 0.2,
            "{core} {foreland}"
        );
        // Normal faults mostly dip towards the rift's axis (i = 179.5).
        let (mut inward, mut outward) = (0, 0);
        for f in all.iter().filter(|f| f.kind == Kind::Normal) {
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
