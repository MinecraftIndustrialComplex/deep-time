// SPDX-License-Identifier: LGPL-3.0-or-later
//! Collision belts and collision resistance: tasks 3 and 4 of the force-balance design
//! (`docs/design/plate-force-balance.md` §3.5 and §9; the collisions design
//! `docs/design/plate-collisions-and-welding.md` §4.1, option 1C, and §4.2, option 2B). This is
//! the stage that landed on the old speed law; since the switch (task 7) the collision's
//! resistance is a term of the force balance (`forces.rs`).
//!
//! # Stacking: uplift is the crust that was shortened
//! While a contact is in collision (`shortening.rs`) its front shortens each step by
//! `s = max(v_n, 0) · δt` (`v_n`: the convergence across the front), so `s · L` of continental
//! crust (`L`: the front's length) crosses the front. Of its volume `s · L · H` (`H`: the mean
//! thickness of the incoming crust at the front) the share `collision_stacking_share` stays in
//! the crust; the rest goes down with the slab. Thickening the crust by `T` raises the ground by
//! `T / 5.5` (Airy, `pools::ISOSTATIC_FACTOR`), so the step's ground to raise is
//! `share · s · L · H / 5.5` (km² × m). Nothing is created: the stacked rock is the consumed
//! crust's, by its mix of rock pools (its sediment and sedimentary rock buried and metamorphosed),
//! and the volume added is the volume that crossed times the share ([`Stacked`]).
//!
//! It goes to both sides of the suture:
//!
//! 1. **The wedge** on the incoming plate first. Its ground rises towards the front at the surface
//!    slope `wedge_slope_deg`, up to the overriding plate's height at the front:
//!    `z(d) = z_front − d · tan α`. Where the ground is below that profile it is raised to it (all
//!    of the deficit, or the same share of each sample's when the step's crust does not cover it).
//!    The wedge is as wide as the profile reaches, `(z_front − z_foreland) / tan α` (190 km for a
//!    5 km belt at 1.5°), and it stays at the front while the incoming plate's crust moves through
//!    it and under the front: a critical wedge "continu[es] to grow at constant taper as additional
//!    material is encountered at the toe" (Davis, Suppe & Dahlen 1983).
//! 2. **The belt** on the overriding plate takes the rest, over `collision_belt_km` behind the
//!    front with the profile `1 − (d/W)²` (highest at the front). Where the overriding plate
//!    offers less than half a full belt's area (a sliver, an island arc), each sample takes what
//!    half a belt would give it and the rest goes down with the slab. Nothing caps it: crust above the
//!    strength height spreads in the surface stage (`surface::spread`) and widens it into a
//!    plateau, and erosion wears it down. (The plate-only presets keep their isostatic ceiling,
//!    `uplift_ceiling_m`.)
//!
//! Both are stamped as a collision orogeny where they rose by more than 5 m/Myr (the subduction
//! fronts' rule): the `collision` regime the structure stage folds and thrusts, the orogen
//! provinces, the young-orogen weakness of the flexure. The fold direction is the shortening
//! direction, the front's normal.
//!
//! **The foreland basin** is not built here. The uplift on continental crust is a load the
//! surface stage's flexure solve (`surface/flexure.rs`) carries, and the stiff incoming plate
//! bends down in front of the wedge, which the surface stage then fills. [`stack`] reads the
//! basin back from `surface/flexure_m` for the record ([`Foredeep`]).
//!
//! # Resistance
//! A collision resists convergence across its front through the force balance (`forces.rs`,
//! design §3.4): the belt's gravitational potential energy over the incoming crust's, a force per
//! metre of front against the convergence, plastic (linearised with the step's convergence). It
//! can stop two plates and never push them apart. (Until the switch, 2026-10-04, a cut inside the
//! old speed law, `collision_resistance_mm_yr`, stood in for it.)

use std::collections::BTreeMap;

use glam::DVec3;

use super::Env;
use super::geometry::{Geo, dijkstra};
use super::kinematics::{quat, tangent, to_group};
use super::pools::ISOSTATIC_FACTOR;
use super::shortening::Front;
use super::state::{Belt, CONTINENTAL, FORELAND, OROGENY_HIMALAYAN, PlateState, pool};
use super::subduction::{MIN_CONTACT, OROGENY_MIN_M_PER_MYR, ceiling};

/// The surface's flexure below which ground in front of a wedge counts as the foreland basin, m
/// (`docs/design/flexure.md` §8: `surface/flexure_m` below about −100 m beside a collision).
// ledger: PLT-249 (game-choice: the measure's threshold).
pub const FOREDEEP_M: f64 = -100.0;
/// How far beyond the wedge's toe the foreland basin is looked for, km (Earth-normalized; the
/// widest foredeeps and forebulges of DeCelles & Giles 1996 reach a few hundred km).
// ledger: PLT-249 (game-choice: the measure's reach).
pub const FOREDEEP_REACH_KM: f64 = 600.0;
/// The foredeep of the record is the one in front of a belt that stood at least this high, m
/// (the design's acceptance row: "in front of a belt above 3 km").
// ledger: PLT-249 (the measure's definition, design `plate-force-balance.md` §7).
pub const FOREDEEP_BELT_M: f64 = 3000.0;
/// The lowest ground the wedge's profile is followed down to, m (the reach of its distance
/// field; ground lower than this is not continental lowland).
const WEDGE_FLOOR_M: f64 = -500.0;

/// The foreland basin in front of a wedge, read from the surface's flexure.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Foredeep {
    /// Width of the stretch beyond the wedge's toe that stands more than [`FOREDEEP_M`] below
    /// local isostasy, km (0 when none does), and the deepest of it, m (positive: below; when no
    /// stretch qualifies, the deepest beyond the toe, or 0).
    pub width_km: f64,
    pub depth_m: f64,
}

/// What a step's stacking did at one collision front.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stacked {
    /// Crust that crossed the front this step, km² (shortening × the front's length), and its
    /// mean thickness, m.
    pub crossed_km2: f64,
    pub thickness_m: f64,
    /// Crust added to the belt and to the wedge, km² × m: Σ added thickness × a sample's area.
    pub belt_km2_m: f64,
    pub wedge_km2_m: f64,
    /// The largest uplift, m.
    pub max_dz_m: f64,
    /// The wedge's width, km, and the belt's height (mean above sea level) and mean elevation, m,
    /// within half its width of the front, before and after the step.
    pub wedge_km: f64,
    pub height_m: f64,
    pub mean_before_m: f64,
    pub mean_m: f64,
    /// The foreland basin, when the surface's flexure was given.
    pub foredeep: Option<Foredeep>,
}

/// Stacks this step's shortened crust of every collision with a front (see the module docs) and
/// updates each collision's [`Belt`]. `fronts` are this step's (`shortening::advance`); `flexure`
/// is `surface/flexure_m` per sample (NaN where unknown), when the surface stage has one.
pub fn stack(
    state: &mut PlateState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
    fronts: &BTreeMap<usize, Front>,
    flexure: Option<&[f64]>,
) -> BTreeMap<usize, Stacked> {
    let mut out = BTreeMap::new();
    if state.contacts.iter().all(|c| c.collision.is_none()) {
        return out;
    }
    let n = state.samples.len();
    let plate_of: Vec<u32> = (0..n).map(|i| state.plate_of(i)).collect();
    for c in 0..state.contacts.len() {
        let (under, over) = (state.contacts[c].under, state.contacts[c].over);
        let Some(col) = state.contacts[c].collision.as_ref() else {
            continue;
        };
        let Some(f) = fronts.get(&c) else { continue };
        if col.last_step != env.step || f.samples < MIN_CONTACT {
            continue;
        }
        let (conv, normal) = (col.convergence_mm_yr, DVec3::from_array(col.normal));
        let step = stack_front(
            state,
            geo,
            pos,
            env,
            &plate_of,
            Side {
                under,
                over,
                conv,
                normal,
                shortening_km: col.shortening_km,
            },
            f,
            flexure,
        );
        let applied = step.stacked;
        let col = state.contacts[c].collision.as_mut().expect("in collision");
        let b = &mut col.belt;
        b.height_m = applied.height_m;
        let base = *b.base_m.get_or_insert(applied.mean_before_m);
        b.relief_m = (applied.mean_m - base).max(0.0);
        b.wedge_km = applied.wedge_km;
        b.stacked_km2_m += applied.belt_km2_m + applied.wedge_km2_m;
        b.wedge_km2_m += applied.wedge_km2_m;
        b.uplift_m += applied.max_dz_m;
        b.lifted10 = b.lifted10.max(step.lifted10);
        b.lifted100 = b.lifted100.max(step.lifted100);
        b.emerged = b.emerged.max(step.emerged);
        b.weight += step.weight;
        b.across_sum += step.across_sum;
        b.across_sq += step.across_sq;
        if applied.height_m >= b.peak_height_m {
            b.peak_height_m = applied.height_m;
            b.peak_step = env.step;
            b.peak_wedge_km = applied.wedge_km;
            b.foredeep_km = applied.foredeep.map(|d| d.width_km);
            b.foredeep_m = applied.foredeep.map(|d| d.depth_m);
        }
        out.insert(c, applied);
    }
    out
}

/// The two plates of a front and what the step moves.
struct Side {
    under: u32,
    over: u32,
    conv: f64,
    normal: DVec3,
    /// The collision's shortening so far, km.
    shortening_km: f64,
}

/// One front's stacking, and what the belt's record gains.
struct FrontStep {
    stacked: Stacked,
    lifted10: u32,
    lifted100: u32,
    emerged: u32,
    weight: f64,
    across_sum: f64,
    across_sq: f64,
}

#[allow(clippy::too_many_arguments)]
fn stack_front(
    state: &mut PlateState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
    plate_of: &[u32],
    side: Side,
    f: &Front,
    flexure: Option<&[f64]>,
) -> FrontStep {
    let k = env.k;
    let norm = env.norm;
    let n = state.samples.len();
    let area = norm.sample_area_km2();
    // ledger: PLT-192 (the wedge's surface slope), PLT-244 (the wedge's rule).
    let tan = k.wedge_slope_deg.to_radians().tan();
    let samples = &state.samples;
    let visible_of =
        |p: u32| move |v: u32| plate_of[v as usize] == p && !samples[v as usize].is_hidden();

    // The overriding plate behind the front: distance from its edge on the front.
    let over_ok = visible_of(side.over);
    let src: Vec<(u32, f64, u32)> = f.edge.iter().map(|&v| (v, 0.0, 0)).collect();
    let belt_km = k.collision_belt_km;
    let (d_over, _) = dijkstra(geo, &src, |_, b| over_ok(b), norm.rad(belt_km));
    let edge_z: Vec<f64> = f
        .edge
        .iter()
        .filter(|&&v| over_ok(v))
        .map(|&v| f64::from(samples[v as usize].elevation_m))
        .collect();
    let z_front = if edge_z.is_empty() {
        0.0
    } else {
        edge_z.iter().sum::<f64>() / edge_z.len() as f64
    };

    // The incoming plate in front of it: distance from its crust at the front.
    let under_ok = visible_of(side.under);
    let wedge_reach_km = ((z_front - WEDGE_FLOOR_M).max(0.0) / (1000.0 * tan)).min(
        // A guard for gentle slopes: the profile never runs out further than the belt is wide
        // beyond the basin's reach.
        belt_km + FOREDEEP_REACH_KM,
    );
    // Beyond the wedge's toe the foreland basin is looked for (`foredeep`) and the structure stage's
    // foreland is flagged ([`FORELAND`]).
    let reach_km = wedge_reach_km + FOREDEEP_REACH_KM;
    let src: Vec<(u32, f64, u32)> = f.crust.iter().map(|&v| (v, 0.0, 0)).collect();
    let (d_under, _) = dijkstra(geo, &src, |_, b| under_ok(b), norm.rad(reach_km));

    // The wedge's deficit under its profile.
    let mut wedge: Vec<(usize, f64)> = Vec::new();
    let mut wedge_km = 0.0f64;
    for i in 0..n {
        let d = d_under[i];
        if !d.is_finite() || !under_ok(i as u32) || samples[i].kind != CONTINENTAL {
            continue;
        }
        let d_km = norm.km(d);
        if d_km > wedge_reach_km {
            continue;
        }
        let deficit = z_front - d_km * 1000.0 * tan - f64::from(samples[i].elevation_m);
        if deficit > 0.0 {
            wedge.push((i, deficit));
            wedge_km = wedge_km.max(d_km);
        }
    }
    let deficit_km2_m: f64 = wedge.iter().map(|&(_, x)| x).sum::<f64>() * area;
    let half = 0.5 * norm.rad(belt_km);
    let near: Vec<usize> = (0..n).filter(|&i| d_over[i] <= half).collect();
    let mean_of = |z: &dyn Fn(usize) -> f64| {
        if near.is_empty() {
            z_front
        } else {
            near.iter().map(|&i| z(i)).sum::<f64>() / near.len() as f64
        }
    };
    let mean_before_m = mean_of(&|i| f64::from(samples[i].elevation_m));

    // The step's crust: what crossed the front, its thickness and the share that stays.
    let shortened_km = side.conv.max(0.0) * env.dt_myr;
    let crossed_km2 = shortened_km * f.length_km();
    let thickness_m = if f.crust.is_empty() {
        0.0
    } else {
        f.crust
            .iter()
            .map(|&j| f64::from(samples[j as usize].thickness_m))
            .sum::<f64>()
            / f.crust.len() as f64
    };
    // ledger: PLT-242 (the share), PLT-121 (Airy, ISOSTATIC_FACTOR).
    let rise_km2_m = k.collision_stacking_share * crossed_km2 * thickness_m / ISOSTATIC_FACTOR;
    let fill = if deficit_km2_m > 0.0 {
        (rise_km2_m / deficit_km2_m).min(1.0)
    } else {
        0.0
    };
    let belt_rise_km2_m = rise_km2_m - fill * deficit_km2_m;

    // ledger: PLT-243 (the belt's profile, 1 − (d/W)², highest at the front).
    let w_rad = norm.rad(belt_km);
    let belt: Vec<(usize, f64)> = (0..n)
        .filter(|&i| d_over[i].is_finite())
        .map(|i| (i, 1.0 - (d_over[i] / w_rad).powi(2)))
        .filter(|&(_, w)| w > 0.0)
        .collect();
    // Spread over the belt a front of this length has behind it (∫(1 − x²) = 2/3 of its width):
    // where the overriding plate is narrower than the belt (a sliver, an arc), the stack does not
    // pile onto the few samples there (below half the belt's area); what has no room goes down
    // with the slab.
    let belt_weight: f64 = (belt.iter().map(|&(_, w)| w).sum::<f64>() * area)
        .max(0.5 * 2.0 / 3.0 * belt_km * f.length_km());

    // The rock that stays: the consumed crust's own mix, its sediments buried and metamorphosed.
    // ledger: PLT-248.
    let mut mix = [0.0f64; pool::COUNT];
    for &j in &f.crust {
        for (m, &x) in mix.iter_mut().zip(samples[j as usize].pools.iter()) {
            *m += f64::from(x.max(0.0));
        }
    }
    mix[pool::METAMORPHIC] += mix[pool::SEDIMENT] + mix[pool::SEDIMENTARY];
    mix[pool::SEDIMENT] = 0.0;
    mix[pool::SEDIMENTARY] = 0.0;
    let total: f64 = mix.iter().sum();
    if total > 0.0 {
        for m in &mut mix {
            *m /= total;
        }
    } else {
        mix[pool::METAMORPHIC] = 1.0;
    }

    let mut rises: Vec<(usize, f64, bool)> =
        wedge.iter().map(|&(i, x)| (i, fill * x, true)).collect();
    // Signed distance from the front, km: in front (the wedge) positive, behind negative.
    let across = |i: usize, in_wedge: bool| {
        if in_wedge {
            norm.km(d_under[i])
        } else {
            -norm.km(d_over[i])
        }
    };
    if belt_weight > 0.0 && belt_rise_km2_m > 0.0 {
        let cap_m = k.strength_height_m(env.gravity);
        let z_of: Vec<f64> = belt
            .iter()
            .map(|&(i, _)| f64::from(samples[i].elevation_m))
            .collect();
        let given = fill_belt(&belt, &z_of, belt_rise_km2_m, belt_weight, area, cap_m);
        rises.extend(
            belt.iter()
                .zip(given)
                .filter(|&(_, g)| g > 0.0)
                .map(|(&(i, _), g)| (i, g, false)),
        );
    }
    let mut out = FrontStep {
        stacked: Stacked {
            crossed_km2,
            thickness_m,
            wedge_km,
            ..Default::default()
        },
        lifted10: 0,
        lifted100: 0,
        emerged: 0,
        weight: 0.0,
        across_sum: 0.0,
        across_sq: 0.0,
    };
    let stamp_m = OROGENY_MIN_M_PER_MYR * env.dt_myr;
    for (i, raw, in_wedge) in rises {
        let rot = quat(state.groups[state.samples[i].group as usize].rot);
        let s = &mut state.samples[i];
        let z = f64::from(s.elevation_m);
        let z1 = (z + raw * ceiling(z, k.uplift_ceiling_m)).min(k.max_elevation_m);
        let dz = z1 - z;
        if dz <= 0.0 {
            continue;
        }
        s.elevation_m = z1 as f32;
        let added = dz * ISOSTATIC_FACTOR;
        for (p, m) in s.pools.iter_mut().zip(mix.iter()) {
            *p += (added * m) as f32;
        }
        s.thickness_m = s.pools.iter().sum();
        if in_wedge {
            out.stacked.wedge_km2_m += added * area;
        } else {
            out.stacked.belt_km2_m += added * area;
        }
        out.stacked.max_dz_m = out.stacked.max_dz_m.max(dz);
        out.lifted10 += u32::from(dz > 10.0);
        out.lifted100 += u32::from(dz > 100.0);
        out.emerged += u32::from(z <= 0.0 && z1 > 0.0);
        // ledger: PLT-247 (the stamp), PLT-079 (the regime it makes).
        if dz >= stamp_m {
            s.orogeny = OROGENY_HIMALAYAN;
            s.orogeny_myr = env.t_myr as f32;
            let at = pos[i];
            s.fold = to_group(rot, tangent(at, side.normal).normalize_or_zero());
        }
        // Where the collision raised ground across the front, weighted by how much.
        let x = across(i, in_wedge);
        out.weight += dz;
        out.across_sum += dz * x;
        out.across_sq += dz * x * x;
    }

    // The collision's footprint, for the structure stage (`structure::deform::emplace`): every
    // sample of the belt, the wedge and the foreland within reach of the basin carries the
    // convergence and the shortening of this collision; the foreland on the incoming plate
    // beyond the wedge's toe is flagged. Where two collisions meet the larger values stay and a
    // belt or wedge sample is never a foreland one.
    // ledger: PLT-260 (the footprint); PLT-249 (the basin's reach).
    let marks: Vec<(usize, bool)> = (0..n)
        .filter_map(|i| {
            if d_over[i].is_finite() && d_over[i] < w_rad {
                return Some((i, false));
            }
            let d = d_under[i];
            (d.is_finite() && plate_of[i] == side.under && !state.samples[i].is_hidden())
                .then(|| (i, norm.km(d) > wedge_km))
        })
        .collect();
    let (conv, shortened) = (side.conv.max(0.0) as f32, side.shortening_km as f32);
    for (i, foreland) in marks {
        let s = &mut state.samples[i];
        let was_belt = s.collision_mm_yr > 0.0 && s.flags & FORELAND == 0;
        s.collision_mm_yr = s.collision_mm_yr.max(conv);
        s.shortening_km = s.shortening_km.max(shortened);
        if !foreland {
            s.flags &= !FORELAND;
        } else if !was_belt {
            s.flags |= FORELAND;
        }
    }

    // The belt's height after the step: its mean above the sea within half its width, and its
    // mean elevation there.
    let samples = &state.samples;
    out.stacked.height_m = mean_of(&|i| f64::from(samples[i].elevation_m).max(0.0)).max(0.0);
    out.stacked.mean_m = mean_of(&|i| f64::from(samples[i].elevation_m));
    out.stacked.mean_before_m = mean_before_m;
    out.stacked.foredeep = flexure.map(|fl| foredeep(&d_under, fl, wedge_km, &norm_km(env)));
    out
}

/// Shares the belt's crust (`rise_km2_m`, km² × m) among its samples by the profile `w`, over
/// `weight` (km²: the profile's area, at least half a full belt's), each sample rising at most to
/// the crust's strength height `cap_m` (its elevation `z` given). What a sample at the strength
/// height cannot take goes to the samples behind it that can, by the same profile, in a few
/// passes: the belt widens into a plateau instead of rising. What no sample can take goes down
/// with the slab. Returns each sample's rise, m. `cap_m` ≤ 0: no limit.
///
/// Why a limit at all: crust above its strength height spreads in the surface stage
/// (`surface::spread`), but that runs once a step on the surface while a fast collision stacks
/// kilometres a step at its front. A collision that never stalls (a short front on a plate pulled
/// by a long slab: the resistance's reach is the front's share of the plate's drive) then piled
/// crust 200–290 km thick on an overriding sliver, and the surface stood 14 km high there
/// (`earthlike_quick_16k`, scattered start, seed 1, 500 Myr, after the continental crust fix).
// ledger: PLT-253 (the stack rises no higher than the strength height).
fn fill_belt(
    belt: &[(usize, f64)],
    z: &[f64],
    rise_km2_m: f64,
    weight: f64,
    area: f64,
    cap_m: f64,
) -> Vec<f64> {
    let mut given = vec![0.0f64; belt.len()];
    if cap_m <= 0.0 {
        for (g, &(_, w)) in given.iter_mut().zip(belt) {
            *g = rise_km2_m * w / weight;
        }
        return given;
    }
    let room = |j: usize, given: &[f64]| (cap_m - z[j] - given[j]).max(0.0);
    let mut left = rise_km2_m;
    let full = weight / area;
    for _ in 0..6 {
        let open: f64 = belt
            .iter()
            .enumerate()
            .filter(|&(j, _)| room(j, &given) > 0.0)
            .map(|(_, &(_, w))| w)
            .sum();
        if open <= 0.0 || left <= 1e-9 * rise_km2_m {
            break;
        }
        // Per sample of profile 1: never more than the first pass gives (a sliver's samples
        // take what half a belt would give them).
        let per = left / (open.max(full) * area);
        let mut placed = 0.0;
        for (j, &(_, w)) in belt.iter().enumerate() {
            let r = room(j, &given);
            if r > 0.0 {
                let g = (per * w).min(r);
                given[j] += g;
                placed += g * area;
            }
        }
        left -= placed;
        if placed <= 1e-9 * rise_km2_m {
            break;
        }
    }
    given
}

/// Distance (rad) to km and the bin width of the foreland's profile.
struct Bins {
    model_km: f64,
    bin_km: f64,
}

fn norm_km(env: &Env) -> Bins {
    Bins {
        model_km: env.norm.model_km,
        bin_km: env.norm.km(env.norm.spacing_rad()),
    }
}

/// The foreland basin beyond the wedge's toe (`wedge_km`): the surface's flexure averaged in bins
/// one sample spacing wide by distance from the front, then the stretch of bins below
/// [`FOREDEEP_M`] that starts within two bins of the toe.
fn foredeep(d_rad: &[f64], flexure: &[f64], wedge_km: f64, bins: &Bins) -> Foredeep {
    let reach = wedge_km + FOREDEEP_REACH_KM;
    let count = (reach / bins.bin_km).ceil() as usize + 1;
    let mut sum = vec![0.0f64; count];
    let mut num = vec![0u32; count];
    for (i, &d) in d_rad.iter().enumerate() {
        let x = flexure.get(i).copied().unwrap_or(f64::NAN);
        if !d.is_finite() || !x.is_finite() {
            continue;
        }
        let b = (d * bins.model_km / bins.bin_km) as usize;
        if b < count {
            sum[b] += x;
            num[b] += 1;
        }
    }
    let toe = (wedge_km / bins.bin_km).floor() as usize;
    let mean: Vec<Option<f64>> = (0..count)
        .map(|b| (num[b] > 0).then(|| sum[b] / f64::from(num[b])))
        .collect();
    let deepest = mean[toe.min(count)..]
        .iter()
        .flatten()
        .copied()
        .fold(0.0f64, f64::min);
    let Some(first) =
        (toe..(toe + 3).min(count)).find(|&b| mean[b].is_some_and(|x| x < FOREDEEP_M))
    else {
        return Foredeep {
            width_km: 0.0,
            depth_m: -deepest,
        };
    };
    let mut depth = 0.0f64;
    let mut width = 0usize;
    for m in &mean[first..] {
        match m {
            Some(x) if *x < FOREDEEP_M => {
                width += 1;
                depth = depth.min(*x);
            }
            _ => break,
        }
    }
    Foredeep {
        width_km: width as f64 * bins.bin_km,
        depth_m: -depth,
    }
}

/// The belt's extent: its length is the front's (`length_km`, the longest it was), its width
/// √(12·variance) of where it rose across the front, weighted by the uplift (belt and wedge
/// together; exact for a uniform strip, and unaffected by how the front curves).
pub fn footprint(b: &Belt, length_km: f64) -> super::dynamics::Footprint {
    let mut f = super::dynamics::Footprint::default();
    if b.weight <= 0.0 {
        return f;
    }
    let mean = b.across_sum / b.weight;
    let var = (b.across_sq / b.weight - mean * mean).max(0.0);
    f.width_km = (12.0 * var).sqrt();
    if f.width_km > 0.0 {
        f.length_km = length_km;
    }
    f
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{PlanetPreset, plates_builtin};
    use crate::plates::geometry::{Tree, positions};
    use crate::plates::normalise::Norm;
    use crate::plates::pools::continental_column;
    use crate::plates::shortening::{self, Fired};
    use crate::plates::state::{Contact, Group, Plate, Sample, SlabTrack};
    use crate::preset::{Preset, PresetBody, with_timeline};

    const N: usize = 16_000;

    fn preset() -> PlanetPreset {
        let json = with_timeline(
            plates_builtin("plates_quick_16k").unwrap(),
            Some("supercontinent"),
            Some(100.0),
        )
        .unwrap();
        match Preset::from_json(&json).unwrap().body {
            PresetBody::Planet(p) => {
                let mut p = *p;
                // The pipeline's choices: no isostatic ceiling (the surface spreads over-thick
                // crust), a guard far above anything here.
                p.tectonics.uplift_ceiling_m = 0.0;
                p.tectonics.max_elevation_m = 30_000.0;
                p
            }
            other => panic!("not a planet preset: {other:?}"),
        }
    }

    fn plate(id: u32, omega: DVec3, centroid: DVec3) -> Plate {
        Plate {
            id,
            alive: true,
            omega: omega.to_array(),
            born_myr: 0.0,
            died_myr: None,
            birth_event: None,
            parent: None,
            lip_until_myr: 0.0,
            lip_event: None,
            samples: 0,
            centroid: centroid.to_array(),
            continental_frac: 1.0,
        }
    }

    /// The continental part of the planet: a cap within 30° of +y, so the boundary's continental
    /// stretch is where a rotation about z is close to head-on. Continental lowland 35.1 km thick
    /// at 300 m; the rest ocean floor at −4 km.
    fn arc(p: DVec3) -> bool {
        p.y > 0.866
    }

    /// Two plates split by the plane x = `split`: plate 0 (x < split) overrides, plate 1 comes
    /// in with angular velocity `omega` (rad/yr).
    fn two_plates(geo: &Geo, split: f64, omega: DVec3) -> PlateState {
        let samples: Vec<Sample> = geo
            .points()
            .iter()
            .map(|&p| {
                let mut s = Sample::blank(u32::from(p.x >= split));
                if arc(p) {
                    s.kind = CONTINENTAL;
                    s.pools = continental_column();
                    s.thickness_m = s.pools.iter().sum();
                    s.elevation_m = 300.0;
                } else {
                    s.elevation_m = -4000.0;
                }
                s
            })
            .collect();
        let ident = [0.0, 0.0, 0.0, 1.0];
        let mut st = PlateState {
            n: N as u32,
            lattice: ident,
            samples,
            groups: vec![
                Group {
                    plate: 0,
                    rot: ident,
                },
                Group {
                    plate: 1,
                    rot: ident,
                },
            ],
            plates: vec![
                plate(0, DVec3::ZERO, DVec3::NEG_X),
                plate(1, omega, DVec3::X),
            ],
            contacts: vec![Contact {
                under: 1,
                over: 0,
                onset_myr: 0.0,
                onset_step: 1,
                onset_event: None,
                onset_slab_age_myr: 0.0,
                onset_convergence_mm_yr: 0.0,
                setting: 2,
                last_step: 1,
                samples: 3,
                collision: None,
                max_uplift_mm_yr: 0.0,
                andean: false,
                centroid: [0.0, 1.0, 0.0],
                cont_in: 0,
            }],
            hotspots: Vec::new(),
            ridges: Vec::new(),
            rift_links: Vec::new(),
            rng: crate::sphere::SplitMix64::new(1),
            step: 0,
            t_myr: 0.0,
            start_myr: 0.0,
            resample_step: 0,
            resample_myr: 0.0,
            a0_km2: 1.0,
            counters: Default::default(),
            slabs: Vec::new(),
            dynamics: None,
        };
        let pos = positions(&st, geo);
        st.recount(&pos);
        st
    }

    struct Fixture {
        preset: PlanetPreset,
        geo: Geo,
        norm: Norm,
    }

    impl Fixture {
        fn new() -> Fixture {
            let preset = preset();
            let norm = Norm::new(preset.tectonics.model_radius_km, 6371.0, N);
            Fixture {
                geo: Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap(),
                preset,
                norm,
            }
        }

        fn env(&self, step: u64) -> Env<'_> {
            Env {
                k: &self.preset.tectonics,
                norm: self.norm,
                dt_myr: 2.0,
                dt_yr: 2.0e6,
                gravity: 9.81,
                step,
                t_myr: 2.0 * step as f64,
            }
        }

        /// Angular speed (rad/yr) that moves a point 90° from the pole at `mm_yr`.
        fn rate(&self, mm_yr: f64) -> f64 {
            self.norm.omega_of_speed(mm_yr)
        }

        /// One step on fixed positions: the collision state, then the stacking.
        fn step(
            &self,
            st: &mut PlateState,
            step: u64,
            flexure: Option<&[f64]>,
        ) -> (BTreeMap<usize, Stacked>, Vec<Fired>) {
            let pos = positions(st, &self.geo);
            let tree = Tree::over(&pos, |_| true);
            // (What `subduction::resolve` does at the start of a step.)
            for s in &mut st.samples {
                s.flags &= !FORELAND;
                s.collision_mm_yr = 0.0;
                s.shortening_km = 0.0;
            }
            st.contacts[0].last_step = step;
            let env = self.env(step);
            let (fronts, fired) =
                shortening::advance(st, &self.geo, &pos, &tree, &env, BTreeMap::new());
            let stacked = stack(st, &self.geo, &pos, &env, &fronts, flexure);
            (stacked, fired)
        }
    }

    /// Σ crust thickness × area over the visible samples, km² × m.
    fn crust_km2_m(st: &PlateState, area: f64) -> f64 {
        st.samples
            .iter()
            .filter(|s| !s.is_hidden())
            .map(|s| f64::from(s.thickness_m) * area)
            .sum()
    }

    /// A head-on continental collision builds its belt over many steps, not in one: the belt
    /// rises step by step on the overriding plate behind the front, the incoming plate grows a
    /// wedge whose surface slopes at `wedge_slope_deg` up to the belt, and the whole is long along
    /// the front and narrow across it. Nothing beyond the belt and the wedge moves.
    #[test]
    fn a_head_on_collision_builds_a_belt_over_many_steps() {
        let mut fx = Fixture::new();
        fx.preset.tectonics.collision_threshold_km = 1.0e9;
        let mut st = two_plates(&fx.geo, 0.0, DVec3::Z * fx.rate(30.0));
        let z0: Vec<f32> = st.samples.iter().map(|s| s.elevation_m).collect();
        let mut heights = Vec::new();
        for step in 1..=12 {
            let (stacked, fired) = fx.step(&mut st, step, None);
            assert!(fired.is_empty());
            let s = &stacked[&0];
            // The crust coming in is the continent's 35.1 km, thickened by the wedge once it
            // reaches the front.
            assert!(s.crossed_km2 > 0.0 && s.thickness_m >= 35_099.0, "{s:?}");
            heights.push(s.height_m);
        }
        // Step by step: every step adds, and the first step built a small part of it.
        assert!(heights.windows(2).all(|w| w[1] > w[0]), "{heights:?}");
        assert!(heights[0] < 0.15 * heights[11], "{heights:?}");
        let belt = &st.contacts[0].collision.as_ref().unwrap().belt;
        assert!(belt.peak_height_m > 1500.0, "{}", belt.peak_height_m);
        // The shape: along the front (6,670 km of it) and narrow across it.
        let f = footprint(
            belt,
            st.contacts[0].collision.as_ref().unwrap().peak_front_km,
        );
        assert!(f.length_km > 4000.0, "{f:?}");
        assert!(f.width_km < 800.0, "{f:?}");
        assert!(f.elongation() > 6.0, "{f:?}");
        // The wedge: the incoming plate's ground rises towards the front, at the set slope, up
        // to the belt's height at the front; it is as wide as that height over the slope.
        let tan = fx.preset.tectonics.wedge_slope_deg.to_radians().tan();
        assert!(belt.wedge_km > 0.0, "no wedge");
        let pts = fx.geo.points();
        let km = |p: DVec3| fx.norm.km(p.x.asin());
        let spacing = fx.norm.km(fx.norm.spacing_rad());
        let mut wedge_rise = 0.0f64;
        for (i, s) in st.samples.iter().enumerate() {
            let p = pts[i];
            let dz = f64::from(s.elevation_m - z0[i]);
            if !arc(p) || p.y < 0.95 {
                // Away from the cap's corners only.
                continue;
            }
            let x = km(p);
            if x >= 0.0 {
                // Incoming: the wedge, and nothing beyond its toe.
                if x > belt.wedge_km + 2.0 * spacing {
                    assert!(
                        dz.abs() < 1e-3,
                        "ground beyond the wedge rose {dz} at {x} km"
                    );
                } else {
                    wedge_rise = wedge_rise.max(dz);
                }
            } else if -x > fx.preset.tectonics.collision_belt_km + 2.0 * spacing {
                assert!(
                    dz.abs() < 1e-3,
                    "ground behind the belt rose {dz} at {x} km"
                );
            }
        }
        assert!(wedge_rise > 100.0, "the wedge rose {wedge_rise} m");
        let expect = (belt.height_m - 300.0) / (1000.0 * tan);
        assert!(
            (belt.wedge_km - expect).abs() < 0.5 * expect + 2.0 * spacing,
            "wedge {} km for a belt {} m high",
            belt.wedge_km,
            belt.height_m
        );
        // Stamped as a collision orogeny.
        let stamped = st
            .samples
            .iter()
            .filter(|s| s.orogeny == OROGENY_HIMALAYAN)
            .count();
        assert!(stamped > 50, "{stamped}");
    }

    /// The collision's footprint is marked for the structure stage (plate task 6): the belt
    /// behind the front, the wedge and the foreland on the incoming plate carry the convergence
    /// across the front and the collision's shortening so far; the incoming plate beyond the
    /// wedge's toe, within the basin's reach, is flagged as foreland; nothing farther is marked.
    #[test]
    fn a_collision_marks_its_footprint() {
        let mut fx = Fixture::new();
        fx.preset.tectonics.collision_threshold_km = 1.0e9;
        let mut st = two_plates(&fx.geo, 0.0, DVec3::Z * fx.rate(30.0));
        for step in 1..=8 {
            fx.step(&mut st, step, None);
        }
        let col = st.contacts[0].collision.clone().expect("in collision");
        assert!(col.convergence_mm_yr > 10.0 && col.shortening_km > 50.0);
        let belt_km = fx.preset.tectonics.collision_belt_km;
        let spacing = fx.norm.km(fx.norm.spacing_rad());
        let wedge_km = col.belt.wedge_km;
        let pts = fx.geo.points();
        let (mut in_belt, mut in_foreland, mut in_wedge) = (0, 0, 0);
        for (i, s) in st.samples.iter().enumerate() {
            let p = pts[i];
            // Inside the continental cap, away from its corners.
            if !arc(p) || p.y < 0.95 {
                continue;
            }
            let x = fx.norm.km(p.x.asin());
            let marked = s.collision_mm_yr > 0.0;
            let foreland = s.flags & FORELAND != 0;
            if x < 0.0 && -x < belt_km - spacing {
                // The belt on the overriding plate.
                assert!(marked && !foreland, "belt sample at {x} km: {s:?}");
                in_belt += 1;
            } else if x < 0.0 && -x > belt_km + spacing {
                assert!(
                    !marked && !foreland,
                    "overriding plate beyond the belt at {x} km"
                );
            } else if x >= 0.0 && x < wedge_km - spacing {
                assert!(marked && !foreland, "wedge sample at {x} km");
                in_wedge += 1;
            } else if x > wedge_km + spacing && x < wedge_km + FOREDEEP_REACH_KM - spacing {
                assert!(marked && foreland, "foreland sample at {x} km: {s:?}");
                in_foreland += 1;
            } else if x > wedge_km + FOREDEEP_REACH_KM + spacing {
                assert!(
                    !marked && !foreland,
                    "incoming plate beyond the reach at {x} km"
                );
            }
            if marked {
                assert!(
                    (f64::from(s.collision_mm_yr) - col.convergence_mm_yr.max(0.0)).abs() < 1e-3,
                    "{} against {}",
                    s.collision_mm_yr,
                    col.convergence_mm_yr
                );
                assert!(
                    (f64::from(s.shortening_km) - col.shortening_km).abs()
                        < 1e-3 * col.shortening_km,
                    "{} against {}",
                    s.shortening_km,
                    col.shortening_km
                );
            }
        }
        assert!(
            in_belt > 3 && in_foreland > 10,
            "{in_belt} {in_wedge} {in_foreland}"
        );
    }

    /// The crust that stacks is the crust that crossed the front: the volume added is the
    /// volume consumed times `collision_stacking_share` (all of it at 1), with the consumed
    /// crust's rock (its sediments metamorphosed). Nothing changes kind, so the continental area
    /// is untouched by the stacking.
    #[test]
    fn stacking_moves_crust_and_creates_none() {
        for share in [1.0, 0.5] {
            let mut fx = Fixture::new();
            fx.preset.tectonics.collision_threshold_km = 1.0e9;
            fx.preset.tectonics.collision_stacking_share = share;
            // A crust strong enough that the strength height (PLT-253) never binds here.
            fx.preset.tectonics.crust_strength_mpa = 2000.0;
            let mut st = two_plates(&fx.geo, 0.0, DVec3::Z * fx.rate(40.0));
            let area = fx.norm.sample_area_km2();
            let kinds: Vec<u8> = st.samples.iter().map(|s| s.kind).collect();
            let mut consumed = 0.0;
            let pool_sum = |st: &PlateState, p: usize| -> f64 {
                st.samples.iter().map(|s| f64::from(s.pools[p])).sum()
            };
            let sediments =
                |st: &PlateState| pool_sum(st, pool::SEDIMENT) + pool_sum(st, pool::SEDIMENTARY);
            let (sed0, meta0) = (sediments(&st), pool_sum(&st, pool::METAMORPHIC));
            for step in 1..=6 {
                let before = crust_km2_m(&st, area);
                let (stacked, _) = fx.step(&mut st, step, None);
                let s = &stacked[&0];
                let added = crust_km2_m(&st, area) - before;
                assert!(
                    (added - (s.belt_km2_m + s.wedge_km2_m)).abs() < 1e-4 * added.max(1.0),
                    "the record is the crust added: {added} vs {}",
                    s.belt_km2_m + s.wedge_km2_m
                );
                let crossed = s.crossed_km2 * s.thickness_m;
                assert!(
                    (added - share * crossed).abs() < 1e-4 * crossed,
                    "share {share}: added {added:.4e} for {crossed:.4e} crossed"
                );
                consumed += crossed;
            }
            assert!(consumed > 0.0);
            // The rock: the buried sediments come back metamorphosed (at least the column's
            // 8.1 km of 35.1 that were sediment, sedimentary or metamorphic rock), none as
            // sediment.
            assert!((sediments(&st) - sed0).abs() < 1e-3 * sed0);
            let meta = pool_sum(&st, pool::METAMORPHIC) - meta0;
            let least = share * consumed / area * (8100.0 / 35_100.0);
            assert!(
                meta >= 0.999 * least,
                "metamorphic rock added {meta} m, at least {least} m expected"
            );
            assert!(st.samples.iter().map(|s| s.kind).eq(kinds.iter().copied()));
            let b = &st.contacts[0].collision.as_ref().unwrap().belt;
            assert!((b.stacked_km2_m - share * consumed).abs() < 1e-4 * consumed);
        }
    }

    /// Plates sliding past each other shorten nothing, so no collision begins and nothing is
    /// built.
    #[test]
    fn a_strike_slip_contact_builds_nothing() {
        let fx = Fixture::new();
        let mut st = two_plates(&fx.geo, 0.0, DVec3::X * fx.rate(60.0));
        let z0: Vec<f32> = st.samples.iter().map(|s| s.elevation_m).collect();
        for step in 1..=10 {
            let (stacked, fired) = fx.step(&mut st, step, None);
            assert!(stacked.is_empty() && fired.is_empty(), "step {step}");
        }
        assert!(st.contacts[0].collision.is_none());
        assert!(st.samples.iter().zip(&z0).all(|(s, &z)| s.elevation_m == z));
    }

    /// The switch (force-balance design, task 7): a continent arrives at the trench of a plate a
    /// slab pulls. The plate moves only by the force balance (`forces::step`, relaxed over
    /// `motion_relax_myr`). The continent holds most of the front, so that stretch pulls nothing
    /// and the slab breaks off after `slab_breakoff_myr`; the belt grows and resists; the
    /// convergence falls, the collision stalls under the weld speed and is handed over to weld.
    #[test]
    fn a_continent_on_a_slab_pulled_plate_slows_and_stalls() {
        let mut fx = Fixture::new();
        fx.preset.tectonics.collision_threshold_km = 1.0e9;
        fx.preset.tectonics.mantle_flow_mm_yr = 0.0;
        let k = fx.preset.tectonics.clone();
        let mut st = two_plates(&fx.geo, 0.0, DVec3::Z * fx.rate(40.0));
        // The continent spans most of the converging half of the boundary (y > 0.2, all but the
        // ends near the rotation's poles), on both plates; the ocean floor is 100 Myr old, so
        // its slab is heavy. (With a continent on only part of the front the slab on the rest
        // swings the plate about the collision, which then opens: no stall, and rightly so.)
        for (s, &p) in st.samples.iter_mut().zip(fx.geo.points()) {
            if p.y > 0.2 {
                s.kind = CONTINENTAL;
                s.pools = continental_column();
                s.thickness_m = s.pools.iter().sum();
                s.elevation_m = 300.0;
            } else {
                s.formed_myr = -100.0;
            }
        }
        let r = k.model_radius_km;
        st.a0_km2 = 4.0 * std::f64::consts::PI * r * r / 14.0;
        st.slabs.push(SlabTrack {
            under: 1,
            over: 0,
            km: k.slab_depth_km,
            ..SlabTrack::default()
        });
        let (mut conv, mut slab_km) = (Vec::new(), Vec::new());
        let mut fired_at = None;
        for step in 1..=100u64 {
            let (_, fired) = fx.step(&mut st, step, None);
            if let Some(f) = fired.first() {
                assert!(f.stalled, "handed over without a stall");
                fired_at = Some(step);
                break;
            }
            if let Some(col) = &st.contacts[0].collision {
                conv.push(col.convergence_mm_yr);
            }
            let pos = positions(&st, &fx.geo);
            let env = fx.env(step);
            let ans = crate::plates::forces::step(&mut st, &fx.geo, &pos, &env, DVec3::Z);
            assert!(ans.solved);
            slab_km.push(
                st.slabs
                    .iter()
                    .find(|s| (s.under, s.over) == (1, 0))
                    .map_or(0.0, |s| s.km),
            );
            for p in 0..2 {
                if let Some(target) = ans.omega[p] {
                    let w = DVec3::from_array(st.plates[p].omega);
                    st.plates[p].omega =
                        crate::plates::forces::relax(w, target, 2.0, k.motion_relax_myr)
                            .to_array();
                }
            }
        }
        let fired_at = fired_at.expect("the collision never stalled into a weld");
        assert!(conv.len() >= 5, "{conv:?}");
        assert!(conv[0] > 10.0, "{conv:?}");
        // The slab broke off: its length fell to nothing within the break-off time and a step.
        let broke = slab_km
            .iter()
            .position(|&km| km < 0.5 * k.slab_depth_km)
            .unwrap_or_else(|| {
                panic!("the blocked slab never broke off: fired at {fired_at}, slab {slab_km:?}, convergence {conv:?}")
            });
        assert!(
            2.0 * (broke + 1) as f64 <= k.slab_breakoff_myr + 2.0,
            "broke after {broke} steps: {slab_km:?}"
        );
        // The convergence fell, under the weld speed, and stayed there for the weld wait.
        let slow = conv
            .iter()
            .position(|&v| v < k.weld_speed_mm_yr)
            .expect("it never stalled");
        assert!(conv[slow] < 0.5 * conv[0], "{conv:?}");
        let waited = 2.0 * (fired_at as usize - 1 - slow) as f64;
        assert!(waited >= k.weld_after_myr - 2.0, "fired {waited} Myr after the stall");
        assert!(st.contacts[0].collision.as_ref().unwrap().belt.height_m > 0.0);
    }

    /// The belt rises no higher than the strength height: what the front cannot take widens the
    /// belt behind it, what nothing can take goes with the slab, and without the limit (or below
    /// it) the share is the profile's.
    #[test]
    fn the_belt_widens_instead_of_rising_past_the_strength_height() {
        // Five samples of 100 km² from the front back, profile 1, 0.8, 0.6, 0.4, 0.2.
        let belt: Vec<(usize, f64)> = (0..5).map(|j| (j, 1.0 - 0.2 * j as f64)).collect();
        let area = 100.0;
        let weight: f64 = belt.iter().map(|&(_, w)| w).sum::<f64>() * area;
        let low = [0.0; 5];
        // Below the limit: the profile's shares, all of it placed.
        let g = fill_belt(&belt, &low, 30_000.0, weight, area, 5000.0);
        let unlimited = fill_belt(&belt, &low, 30_000.0, weight, area, 0.0);
        for (a, b) in g.iter().zip(&unlimited) {
            assert!((a - b).abs() < 1e-9, "{g:?} vs {unlimited:?}");
        }
        assert!((g.iter().sum::<f64>() * area - 30_000.0).abs() < 1e-6);
        // The front already at 4.9 km: it takes 100 m, the rest goes behind, up to the limit.
        let high = [4900.0, 1000.0, 1000.0, 1000.0, 1000.0];
        let g = fill_belt(&belt, &high, 150_000.0, weight, area, 5000.0);
        assert!((g[0] - 100.0).abs() < 1e-6, "{g:?}");
        for (j, &x) in g.iter().enumerate() {
            assert!(high[j] + x <= 5000.0 + 1e-6, "{g:?}");
        }
        let placed = g.iter().sum::<f64>() * area;
        assert!(placed <= 150_000.0 + 1e-6);
        let first_pass = 150_000.0 * 0.8 / weight;
        assert!(
            g[1] > first_pass,
            "the belt widens: {} against {first_pass}",
            g[1]
        );
        // Everything at the limit: nothing rises.
        let full = [5000.0; 5];
        assert!(
            fill_belt(&belt, &full, 150_000.0, weight, area, 5000.0)
                .iter()
                .all(|&x| x == 0.0)
        );
    }

    /// The foreland basin is read beyond the wedge's toe: the stretch of bins below −100 m that
    /// starts within two bins of the toe, its width and its deepest point.
    #[test]
    fn the_foredeep_is_read_beyond_the_wedge() {
        let bins = Bins {
            model_km: 1000.0,
            bin_km: 50.0,
        };
        // One sample per 50 km bin: the wedge sinks (−800 m) under 0–100 km, then the basin
        // −50, −400, −300, −150 m, a bulge, and something deep far away.
        let flex = [
            -800.0, -800.0, -50.0, -400.0, -300.0, -150.0, 40.0, 30.0, -500.0,
        ];
        let d: Vec<f64> = (0..flex.len()).map(|b| (b as f64 + 0.5) * 0.05).collect();
        let f = foredeep(&d, &flex, 100.0, &bins);
        assert_eq!(f.width_km, 150.0);
        assert_eq!(f.depth_m, 400.0);
        // No basin: the deepest beyond the toe, and no width.
        let flat = [-800.0, -800.0, 10.0, 20.0, -50.0, 0.0];
        let d: Vec<f64> = (0..flat.len()).map(|b| (b as f64 + 0.5) * 0.05).collect();
        let f = foredeep(&d, &flat, 100.0, &bins);
        assert_eq!((f.width_km, f.depth_m), (0.0, 50.0));
    }

    /// A continent welds only once its collision has stalled, and the larger plate keeps its
    /// id.
    #[test]
    fn a_continent_welds_only_after_stalling_and_the_larger_keeps_its_id() {
        let fx = Fixture::new();
        // The incoming plate (1) is the larger: the boundary is at x = −0.3.
        let mut st = two_plates(&fx.geo, -0.3, DVec3::Z * fx.rate(30.0));
        let (n0, n1) = (st.plates[0].samples, st.plates[1].samples);
        assert!(n1 > n0);
        let mut world = crate::world::WorldState::new(1, 16_384, 0.0);
        fx.step(&mut st, 1, None);
        assert!(st.contacts[0].collision.is_some());
        let pos = positions(&st, &fx.geo);
        let seeds: Vec<u32> = (0..N as u32)
            .filter(|&i| st.plate_of(i as usize) == 1 && st.samples[i as usize].is_continental())
            .take(5)
            .collect();
        let env = fx.env(1);
        let not_yet = crate::plates::collision::collide(
            &mut st, &mut world, &fx.geo, &pos, &env, 0, &seeds, false,
        )
        .unwrap();
        assert!(not_yet.deferred && !not_yet.sutured);
        assert!(st.plates.iter().all(|p| p.alive) && st.contacts[0].collision.is_some());
        let done = crate::plates::collision::collide(
            &mut st, &mut world, &fx.geo, &pos, &env, 0, &seeds, true,
        )
        .unwrap();
        assert!(done.sutured && !done.deferred);
        assert!(
            st.plates[1].alive && !st.plates[0].alive,
            "the larger keeps its id"
        );
        assert!(st.groups.iter().all(|g| g.plate == 1));
        assert_eq!(st.counters.sutures, 1);
    }
}
