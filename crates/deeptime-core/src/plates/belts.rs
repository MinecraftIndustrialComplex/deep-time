// SPDX-License-Identifier: LGPL-3.0-or-later
//! Collision belts and collision resistance: tasks 3 and 4 of the force-balance design
//! (`docs/design/plate-force-balance.md` §3.5 and §9; the collisions design
//! `docs/design/plate-collisions-and-welding.md` §4.1, option 1C, and §4.2, option 2B). This is
//! the stage that lands on the present speed law (the design's fallback state); the force balance
//! (task 7) is to replace [`resist`].
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
//!    front with the profile `1 − (d/W)²` (highest at the front). Nothing caps it: crust above the
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
//! # Resistance inside the speed law
//! A collision resists convergence across its front with
//!
//!   r = collision_resistance_mm_yr × (h_belt / h_s) × min(1, L / slab_pull_front_km),
//!
//! `h_belt` the belt's height (its mean above sea level within half its width of the front), `h_s`
//! the strength height the crust spreading uses (`crust_strength_mpa / (ρ_c g)`, so gravity and
//! crust strength enter here too) and `L` the front's length. It acts like the speed law's own
//! relaxation: each step the convergence across the front loses `relax · r` (`relax` the speed
//! law's `1 − e^(−δt/τ)`, τ = `slab_pull_relax_myr`), never more than it has. Against the speed
//! law's pull back towards each plate's target speed this settles at a convergence of
//! `v_free − r`, or at none when `r ≥ v_free`: a plastic resistance that can stop two plates and
//! never pushes them apart. The cut is shared by area, the larger plate changing less, as a change
//! of each plate's rotation about an axis through the plate's own centre of the sphere at right
//! angles to the front's centroid. A plate with a long slab elsewhere keeps converging and widens
//! its belt into a plateau (India), a plate without stalls (the Alps): "crustal thickening in the
//! Tibetan Plateau was the dominant cause" of India's slowdown (Copley, Avouac & Royer 2010).

use std::collections::BTreeMap;

use glam::{DQuat, DVec3};

use super::Env;
use super::geometry::{Geo, dijkstra};
use super::kinematics::{arr, quat, rotation, tangent, to_group};
use super::pools::ISOSTATIC_FACTOR;
use super::shortening::{Front, normal_convergence};
use super::state::{Belt, CONTINENTAL, OROGENY_HIMALAYAN, PlateState, pool};
use super::subduction::{MIN_CONTACT, OROGENY_MIN_M_PER_MYR, ceiling};

/// The surface's flexure below which ground in front of a wedge counts as the foreland basin, m
/// (`docs/design/flexure.md` §8: `surface/flexure_m` below about −100 m beside a collision).
// ledger: PLT-236 (game-choice: the measure's threshold).
pub const FOREDEEP_M: f64 = -100.0;
/// How far beyond the wedge's toe the foreland basin is looked for, km (Earth-normalized; the
/// widest foredeeps and forebulges of DeCelles & Giles 1996 reach a few hundred km).
// ledger: PLT-236 (game-choice: the measure's reach).
pub const FOREDEEP_REACH_KM: f64 = 600.0;
/// The foredeep of the record is the one in front of a belt that stood at least this high, m
/// (the design's acceptance row: "in front of a belt above 3 km").
// ledger: PLT-236 (the measure's definition, design `plate-force-balance.md` §7).
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
    /// The wedge's width, km, and the belt's height, m, after the step.
    pub wedge_km: f64,
    pub height_m: f64,
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
        let omega_over = DVec3::from_array(state.plates[over as usize].omega);
        let Some(col) = state.contacts[c].collision.as_mut() else {
            continue;
        };
        // The belt's frame turns with the overriding plate, by the rotation of this step.
        col.belt.frame = arr((rotation(omega_over, env.dt_yr) * quat(col.belt.frame)).normalize());
        let Some(f) = fronts.get(&c) else { continue };
        if col.last_step != env.step || f.samples < MIN_CONTACT {
            continue;
        }
        let (conv, normal, frame) = (
            col.convergence_mm_yr,
            DVec3::from_array(col.normal),
            quat(col.belt.frame).inverse(),
        );
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
                frame,
            },
            f,
            flexure,
        );
        let applied = step.stacked;
        let col = state.contacts[c].collision.as_mut().expect("in collision");
        let b = &mut col.belt;
        b.height_m = applied.height_m;
        b.wedge_km = applied.wedge_km;
        b.stacked_km2_m += applied.belt_km2_m + applied.wedge_km2_m;
        b.wedge_km2_m += applied.wedge_km2_m;
        b.uplift_m += applied.max_dz_m;
        b.lifted10 = b.lifted10.max(step.lifted10);
        b.lifted100 = b.lifted100.max(step.lifted100);
        b.emerged = b.emerged.max(step.emerged);
        b.weight += step.weight;
        for a in 0..3 {
            b.sum[a] += step.sum[a];
        }
        for a in 0..6 {
            b.second[a] += step.second[a];
        }
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
    /// Mantle frame → the belt's frame.
    frame: DQuat,
}

/// One front's stacking, and what the belt's record gains.
struct FrontStep {
    stacked: Stacked,
    lifted10: u32,
    lifted100: u32,
    emerged: u32,
    weight: f64,
    sum: [f64; 3],
    second: [f64; 6],
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
    // ledger: PLT-192 (the wedge's surface slope), PLT-231 (the wedge's rule).
    let tan = k.wedge_slope_deg.to_radians().tan();
    let samples = &state.samples;
    let visible_of = |p: u32| move |v: u32| plate_of[v as usize] == p && !samples[v as usize].is_hidden();

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
    let reach_km = wedge_reach_km + if flexure.is_some() { FOREDEEP_REACH_KM } else { 0.0 };
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
    // ledger: PLT-229 (the share), PLT-121 (Airy, ISOSTATIC_FACTOR).
    let rise_km2_m =
        k.collision_stacking_share * crossed_km2 * thickness_m / ISOSTATIC_FACTOR;
    let fill = if deficit_km2_m > 0.0 {
        (rise_km2_m / deficit_km2_m).min(1.0)
    } else {
        0.0
    };
    let belt_rise_km2_m = rise_km2_m - fill * deficit_km2_m;

    // ledger: PLT-230 (the belt's profile, 1 − (d/W)², highest at the front).
    let w_rad = norm.rad(belt_km);
    let belt: Vec<(usize, f64)> = (0..n)
        .filter(|&i| d_over[i].is_finite())
        .map(|i| (i, 1.0 - (d_over[i] / w_rad).powi(2)))
        .filter(|&(_, w)| w > 0.0)
        .collect();
    let belt_weight: f64 = belt.iter().map(|&(_, w)| w).sum::<f64>() * area;

    // The rock that stays: the consumed crust's own mix, its sediments buried and metamorphosed.
    // ledger: PLT-235.
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

    let mut rises: Vec<(usize, f64, bool)> = wedge
        .iter()
        .map(|&(i, x)| (i, fill * x, true))
        .collect();
    if belt_weight > 0.0 && belt_rise_km2_m > 0.0 {
        rises.extend(
            belt.iter()
                .map(|&(i, w)| (i, belt_rise_km2_m * w / belt_weight, false)),
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
        sum: [0.0; 3],
        second: [0.0; 6],
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
        if dz >= stamp_m {
            s.orogeny = OROGENY_HIMALAYAN;
            s.orogeny_myr = env.t_myr as f32;
            let at = pos[i];
            s.fold = to_group(rot, tangent(at, side.normal).normalize_or_zero());
        }
        // Where the collision raised ground, weighted by how much, in the belt's own frame.
        let p = side.frame * pos[i];
        out.weight += dz;
        for a in 0..3 {
            out.sum[a] += dz * p[a];
        }
        let q = [p.x * p.x, p.y * p.y, p.z * p.z, p.x * p.y, p.x * p.z, p.y * p.z];
        for a in 0..6 {
            out.second[a] += dz * q[a];
        }
    }

    // The belt's height after the step: its mean above the sea within half its width.
    let half = 0.5 * w_rad;
    let near: Vec<f64> = (0..n)
        .filter(|&i| d_over[i] <= half)
        .map(|i| f64::from(state.samples[i].elevation_m).max(0.0))
        .collect();
    out.stacked.height_m = if near.is_empty() {
        z_front.max(0.0)
    } else {
        near.iter().sum::<f64>() / near.len() as f64
    };
    out.stacked.foredeep = flexure.map(|fl| foredeep(&d_under, fl, wedge_km, &norm_km(env)));
    out
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
    let Some(first) = (toe..(toe + 3).min(count)).find(|&b| mean[b].is_some_and(|x| x < FOREDEEP_M))
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

/// The collision resistance inside the speed law (see the module docs): cuts each collision's
/// convergence across its front by `relax · r`, never below none, sharing the cut between the two
/// plates by area. `omega` holds every plate's rotation after the speed law's update this step
/// and is changed in place; the caller caps the speeds after.
pub fn resist(state: &PlateState, omega: &mut [DVec3], env: &Env, relax: f64) {
    let k = env.k;
    let h_s = k.strength_height_m(env.gravity);
    let unit = env.norm.omega_of_speed(1.0);
    for c in &state.contacts {
        let Some(col) = &c.collision else { continue };
        let (u, o) = (c.under as usize, c.over as usize);
        if u == o || !state.plates[u].alive || !state.plates[o].alive || h_s <= 0.0 {
            continue;
        }
        // ledger: PLT-232 (the resistance law).
        let r = k.collision_resistance_mm_yr
            * (col.belt.height_m.max(0.0) / h_s)
            * (col.front_km / k.slab_pull_front_km).min(1.0);
        let (at, normal) = (DVec3::from_array(col.centroid), DVec3::from_array(col.normal));
        if r <= 0.0 || normal == DVec3::ZERO {
            continue;
        }
        let rel = env.norm.velocity_mm_yr(omega[u] - omega[o], at);
        let cut = (relax * r).min(normal_convergence(rel, normal).max(0.0));
        if cut <= 0.0 {
            continue;
        }
        let (a_u, a_o) = (
            f64::from(state.plates[u].samples.max(1)),
            f64::from(state.plates[o].samples.max(1)),
        );
        // The incoming plate is held back along the outward normal, the overriding one pushed
        // the other way; the smaller plate takes the larger part.
        let share_u = a_o / (a_u + a_o);
        omega[u] += at.cross(normal * (cut * share_u)) * unit;
        omega[o] -= at.cross(normal * (cut * (1.0 - share_u))) * unit;
    }
}

/// The belt's extent from its uplift-weighted moments, km on a sphere of `radius_km`: length and
/// width as `dynamics::Moments` gives them (exact for a uniform strip).
pub fn footprint(b: &Belt, radius_km: f64) -> super::dynamics::Footprint {
    let mut f = super::dynamics::Footprint {
        samples: 0,
        ..Default::default()
    };
    if b.weight <= 0.0 {
        return f;
    }
    let w = b.weight;
    let mu = [b.sum[0] / w, b.sum[1] / w, b.sum[2] / w];
    let c = [
        b.second[0] / w - mu[0] * mu[0],
        b.second[1] / w - mu[1] * mu[1],
        b.second[2] / w - mu[2] * mu[2],
        b.second[3] / w - mu[0] * mu[1],
        b.second[4] / w - mu[0] * mu[2],
        b.second[5] / w - mu[1] * mu[2],
    ];
    let [l1, l2, _] = super::dynamics::eigenvalues(c);
    if l2 > 0.0 {
        f.length_km = (12.0 * l1).sqrt() * radius_km;
        f.width_km = (12.0 * l2).sqrt() * radius_km;
    }
    f
}
