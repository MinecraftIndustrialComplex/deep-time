// SPDX-License-Identifier: LGPL-3.0-or-later
//! Global resampling and oceanic crust generation (Cortial et al. 2019, §4.3 and §6), and the
//! bookkeeping of plates that die.
//!
//! Every `resample_min_myr`–`resample_max_myr` (Cortial: 10–60 steps, "depending on the
//! observed maximum plate speed", 20–120 Myr at the 2-Myr step; here when the fastest plate has
//! moved `resample_drift_km`) the crust is resampled onto a fresh lattice: the Fibonacci sphere
//! under a new random rotation, triangulated again by P1-02's hull.
//!
//! - A new sample inside a plate takes that plate's crust by barycentric interpolation ("the
//!   parameters of the other sample points are computed using barycentric interpolation of crust
//!   data from the plate they intersect"). A plate's extent is its footprint: the cells of the old
//!   lattice that its samples owned at the last resampling (each triangle split between its
//!   corners by the largest barycentric weight), moved rigidly with its group; inside the group's
//!   own triangles that is the paper's interpolation, and on its boundary strip (a triangle shared
//!   with another group) the group's own corners are interpolated. Footprints tile the sphere
//!   at a resampling, so a plate keeps its area through one until it really overlaps or leaves a
//!   gap. Where several footprints overlap, the visible crust wins (the subducting plate's hidden
//!   crust is consumed), then the overriding plate of a subduction contact, then continental, then
//!   higher crust; the conserved rock pools of hidden crust, of the same plate's and of the
//!   subducting plate's crust are added to it — tectonics.js's `Crust.overlap` (see `pools.rs`):
//!   trench sediment and forced-subducted continental crust accrete to the overriding plate
//!   instead of vanishing. Continental crust a collision took across its front is consumed
//!   instead, hidden (`stacked`, PLT-276) or still visible in the overriding plate's footprint
//!   (PLT-292): the belt stacked its share when it crossed. Of subducted ocean floor (hidden, or
//!   of a plate the top overrides) `sediment_subduction_share` of its sediment and sedimentary
//!   rock goes down with the slab instead (sediment subduction, PLT-440; 0.12 by default since
//!   2026-10-23; at 0 all of it accretes). (Until 2026-10-02 only whole
//!   triangles of one group counted, and points on a boundary strip copied the nearest visible
//!   sample within 0.75 spacings: where a plate overrode another, the subducting crust under the
//!   strip won it, and the overriding plate lost its outer half-spacing at every resampling,
//!   while the edges of diverging plates grew into the gap by up to a quarter-spacing; build log
//!   "Resampling crust loss".)
//! - A new sample in no footprint (a gap between diverging plates) becomes fresh oceanic crust of
//!   the nearest plate. With d_P the distance to the nearest plate and d_Γ to the ridge (midway
//!   to the second nearest plate), α = d_Γ/(d_Γ + d_P) and the elevation is Cortial's
//!   z = α·z̄ + (1 − α)·z_Γ, z̄ interpolated between the two plates' edges and z_Γ the ridge
//!   profile at the sample's age (the preset's age–depth law, `pools::floor_at`).
//!   The crust's age is α times the time since the last resampling, and its ridge direction is
//!   r = p × (s_i − s_j) normalised (parallel to the ridge, perpendicular to spreading).
//! - The first crust made between two plates emits a ridge_birth event (caused by the rift that
//!   separated them, when one did) once max(3, n/2000) samples of new crust lie between them; a
//!   pair's ridge is born once. Plates left without samples die; plates smaller than
//!   `min_plate_area_frac` are absorbed by the neighbour they share most boundary with.

use std::collections::BTreeMap;

use deeptime_format::EventId;
use glam::{DQuat, DVec3};
use rayon::prelude::*;

use super::Env;
use super::collapse::{iso_height, toward};
use super::events::{self, PLATE_DEATH, RIDGE_BIRTH};
use super::gap_budget;
use super::geometry::{Cover, Geo, Tree, dijkstra_two};
use super::init::random_rotation;
use super::kinematics::{arr, quat, to_world};
use super::normalise::Norm;
use super::pools::{
    accrete, floor_at, isostatic_change_m, ocean_heat_flow, oceanic_column, thickness,
};
use super::state::{
    CONTINENTAL, FORELAND, Group, OCEANIC, PlateState, Ridge, Sample, pool, regime,
};
use crate::error::CoreError;
use crate::world::WorldState;

/// What covers one new point: the group on top (if any) and the others.
type Found = (Option<(u32, Cover)>, Vec<(u32, Cover)>);

/// Whether a resampling is due after this step.
pub fn due(state: &PlateState, env: &Env) -> bool {
    let k = env.k;
    // ledger: PLT-098 (the bounds are times: the steps they span follow the step's length)
    let since_myr = (state.step - state.resample_step) as f64 * env.dt_myr;
    if since_myr < k.resample_min_myr {
        return false;
    }
    if since_myr >= k.resample_max_myr {
        return true;
    }
    let vmax = state
        .alive()
        .map(|p| DVec3::from_array(p.omega).length())
        .fold(0.0, f64::max);
    let drift_km = env
        .norm
        .km(vmax * (state.t_myr - state.resample_myr) * 1.0e6);
    drift_km >= k.resample_drift_km
}

/// Interpolates a sample from a cover of the old crust; vectors come out in the mantle frame,
/// which is the new lattice's frame (every group starts unrotated).
fn interpolate(src: &[Sample], c: &Cover, rot: DQuat) -> Sample {
    let m = c.major();
    let mut s = src[m as usize];
    let corners = c.tri.map(|v| &src[v as usize]);
    let w = c.w;
    let lerp = |f: &dyn Fn(&Sample) -> f32| -> f32 {
        (0..3).map(|k| w[k] * f64::from(f(corners[k]))).sum::<f64>() as f32
    };
    s.elevation_m = lerp(&|x| x.elevation_m);
    // The corners of the sample's own kind and their weight (the major corner is one of them).
    let same_kind: f64 = (0..3)
        .filter(|&k| corners[k].kind == s.kind)
        .map(|k| w[k])
        .sum();
    // The crust's own rock belongs to its kind: the rock pools are blended over the corners of
    // the sample's kind only, the loose sediment (which drapes across a margin) over all three.
    // Blending them all made every resampling put a share of a continent's granite, gneiss and
    // platform cover into the ocean crust beside it. On ocean crust the felsic and metamorphic
    // pools are not blended at all but stay the nearest corner's, as its kind does: they are
    // local (an island arc, the thinned continental crust of a margin's ocean side, crust
    // accreted at a resampling, the base of a thick sediment pile), and a blend spread them a
    // little further at every resampling, so that over the run they reached most of the sea
    // floor (ledger: PLT-293).
    let oceanic = s.kind == OCEANIC;
    let local = |p: usize| {
        oceanic
            && (p == pool::FELSIC_PLUTONIC || p == pool::FELSIC_VOLCANIC || p == pool::METAMORPHIC)
    };
    for p in 0..pool::COUNT {
        s.pools[p] = if local(p) {
            src[m as usize].pools[p]
        } else if p == pool::SEDIMENT || same_kind <= 0.0 {
            lerp(&|x| x.pools[p])
        } else {
            ((0..3)
                .filter(|&k| corners[k].kind == s.kind)
                .map(|k| w[k] * f64::from(corners[k].pools[p]))
                .sum::<f64>()
                / same_kind) as f32
        };
    }
    s.arc_continental_m = lerp(&|x| x.arc_continental_m);
    s.arc_oceanic_m = lerp(&|x| x.arc_oceanic_m);
    // A basement is the continental crust's own (ocean floor's is 0), blended as its pools are.
    s.basement_m = if s.kind == CONTINENTAL && same_kind > 0.0 {
        ((0..3)
            .filter(|&k| corners[k].kind == s.kind)
            .map(|k| w[k] * f64::from(corners[k].basement()))
            .sum::<f64>()
            / same_kind) as f32
    } else {
        0.0
    };
    s.oxidation = lerp(&|x| x.oxidation);
    s.uplift_mm_yr = lerp(&|x| x.uplift_mm_yr);
    s.crust_uplift_mm_yr = lerp(&|x| x.crust_uplift_mm_yr);
    // Heat flow belongs to the kind of crust (the sea floor's age law against the continental
    // sum), so only the corners of the sample's own kind are blended: a continental sample on the
    // coast must not take the young ridge crust's 300 mW/m² from an oceanic corner. (The next
    // step sets every sample's heat flow again; this only matters for what is read right after
    // a resampling, the final snapshot among them: the bake's `meta/heat_flow_mw_m2`.)
    if same_kind > 0.0 {
        let blend: f64 = (0..3)
            .filter(|&k| corners[k].kind == s.kind)
            .map(|k| w[k] * f64::from(corners[k].heat_flow_mw_m2))
            .sum();
        s.heat_flow_mw_m2 = (blend / same_kind) as f32;
    }
    if s.kind == OCEANIC {
        let (mut sw, mut sf) = (0.0, 0.0);
        for k in 0..3 {
            if corners[k].kind == OCEANIC {
                sw += w[k];
                sf += w[k] * f64::from(corners[k].formed_myr);
            }
        }
        if sw > 0.0 {
            s.formed_myr = (sf / sw) as f32;
        }
    }
    let vec = |f: &dyn Fn(&Sample) -> [f32; 3]| -> [f32; 3] {
        let v: DVec3 = (0..3).map(|k| to_world(rot, f(corners[k])) * w[k]).sum();
        [v.x as f32, v.y as f32, v.z as f32]
    };
    s.ridge = vec(&|x| x.ridge);
    s.fold = vec(&|x| x.fold);
    // The step's flags are set again next step; the collision's footprint (`belts.rs`) goes with
    // its convergence and shortening, which the nearest corner passes on, so the structure stage
    // never sees the one without the other after a resampling.
    s.flags &= FORELAND;
    s.thickness_m = thickness(&s.pools);
    s
}

/// The crust uplift (`Sample::crust_uplift_mm_yr`) of a resampling's accretion onto a column whose
/// blended pools were `blended`: the Airy change of the crust it gained, mm/yr (ledger: ISO-091).
fn accreted_uplift_mm_yr(
    blended: &[f32; pool::COUNT],
    pools: &[f32; pool::COUNT],
    gravity: f64,
    env: &Env,
) -> f32 {
    if pools == blended {
        return 0.0;
    }
    (isostatic_change_m(blended, pools, gravity) / env.dt_yr * 1.0e3) as f32
}

/// Where the mantle-frame direction `q` falls in group `group`'s footprint (rotation `rot`): the
/// old lattice triangle holding it in the group's frame, when the corner with the largest
/// barycentric weight is the group's. The cells this partition gives each old sample tile the
/// sphere at the last resampling, so the footprints of all groups are the crust's real extent
/// moved rigidly: inside a group's triangles the cover is `geometry::cover`'s, and on its boundary
/// strip (a triangle shared with another group) the weights are those of the group's own corners.
/// `hint` is an old sample near `q`.
fn footprint(
    geo: &Geo,
    src: &[Sample],
    group: u32,
    rot: DQuat,
    q: DVec3,
    hint: u32,
) -> Option<Cover> {
    let local = rot.inverse() * q;
    let h = geo.mesh.vertex_triangle(hint).unwrap_or(0);
    let t = geo.mesh.locate(local, h);
    let tri = geo.mesh.triangles()[t as usize];
    let mut w = geo.mesh.barycentric(t, local);
    let own = tri.map(|v| src[v as usize].group == group);
    let major = (0..3).fold(0, |k, i| if w[i] > w[k] { i } else { k });
    if !own[major] {
        return None;
    }
    if own.iter().any(|&o| !o) {
        let sum: f64 = (0..3).filter(|&k| own[k]).map(|k| w[k]).sum();
        for k in 0..3 {
            w[k] = if own[k] { w[k] / sum } else { 0.0 };
        }
    }
    Some(Cover { t, tri, w })
}

/// `WorldState::globals`: crust accreted at resamplings to the column on top (hidden and
/// overridden crust, its conserved pools), km³ …
pub const ACCRETED_GLOBAL: &str = "plates/resample_accreted_km3";
/// … and the continental crust a collision took across its front, consumed instead (it was
/// stacked into the belt or went down with the slab; ledger PLT-276, PLT-292), km³
/// (Earth-normalized).
pub const STACKED_CONSUMED_GLOBAL: &str = "plates/resample_stacked_km3";
/// Of the crust accreted: hidden continental crust (not a collision's) and visible crust of the
/// same or a subducting plate, km³ (the rest is the sediment of hidden ocean floor).
pub const ACCRETED_HIDDEN_CONTINENT_GLOBAL: &str = "plates/resample_accreted_continent_km3";
pub const ACCRETED_VISIBLE_GLOBAL: &str = "plates/resample_accreted_visible_km3";
/// World global: the surface accretion raised at resamplings, km² × m / 1000 (km³ of rise,
/// Earth-normalized; plate task 8).
pub const ACCRETED_RISE_GLOBAL: &str = "plates/resample_accreted_rise_km3";
/// World global (measurement, the crust budget `crust_budget.rs`): visible continental crust of
/// another plate under the top cover that neither accretes nor is consumed (no subduction
/// contact puts it under the top's plate), dropped with all its pools, km³ (Earth-normalized) …
pub const DROPPED_GLOBAL: &str = "plates/resample_dropped_km3";
/// … and the mafic pools of the continental crust that accretes, which `pools::accrete` does not
/// carry (only the conserved pools join the column on top), km³.
pub const MAFIC_DROPPED_GLOBAL: &str = "plates/resample_mafic_dropped_km3";
/// World global: subducted ocean floor's sediment and sedimentary rock that went down with the
/// slab at resamplings instead of accreting (`sediment_subduction_share`, PLT-440), km³
/// (Earth-normalized; the crust budget's sediment subduction against Stern & Scholl 2010).
pub const SEDIMENT_SUBDUCTED_GLOBAL: &str = "plates/sediment_subducted_km3";

/// A pair's frame for the ridge segments: its relative rotation's pole ê and two unit vectors
/// f̂, ĝ with f̂ × ĝ = ê, and the pair's band offset in [0, 1).
struct PoleFrame {
    e: DVec3,
    f: DVec3,
    g: DVec3,
    offset: f64,
}

/// One band of a pair's gap: the sums of its points' longitude (circular), then the ridge's
/// longitude and the mean offset from it of the points nearest each plate.
#[derive(Default)]
struct Band {
    cos: f64,
    sin: f64,
    phi: f64,
    side: [f64; 2],
    count: [u32; 2],
}

/// A gap point between two plates, in its pair's pole frame.
struct GapPoint {
    i: usize,
    pair: (u32, u32),
    band: i64,
    theta: f64,
    phi: f64,
    /// Its nearest plate is the pair's first (a).
    near_a: bool,
}

/// A pair's band offset, a fraction of a band in [0, 1) hashed from the two plates' ids, so that
/// the pairs' band edges do not line up across the planet (game-choice: agent 2026-10-23).
fn band_offset(a: u32, b: u32) -> f64 {
    let mut x = (u64::from(a) << 32 | u64::from(b)).wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    (x >> 11) as f64 / (1u64 << 53) as f64
}

/// Wraps an angle into (−π, π].
fn wrap(x: f64) -> f64 {
    use std::f64::consts::{PI, TAU};
    let y = x.rem_euclid(TAU);
    if y > PI { y - TAU } else { y }
}

/// V1 gap 2's stage A (ledger PLT-502): the ridge between two plates as segments normal to their
/// relative motion, joined by transforms. The gap points between plates a < b are put in the
/// frame of the pair's relative rotation, ê = (ω_a − ω_b) / |ω_a − ω_b|: colatitude θ about ê,
/// longitude φ around it. A transform follows a small circle about the pole (constant θ), a ridge
/// normal to the motion a meridian (constant φ). The gap is cut into bands of θ `seg_rad` wide;
/// in each band the ridge stands at the circular mean of the gap's φ, and a point on its +φ side
/// (a moves that way from b) goes to a, the rest to b. Its α (0 at the ridge, 1 at its plate's
/// edge) follows its distance from the segment as Cortial's follows the distance from the
/// midline. A band whose points nearest a do not lie on the +φ side of those nearest b (a
/// converging rotation, or a band too thin to tell) keeps the midline: `None` there, as for
/// every point not between two plates. Returns, per sample, the plate and α the segments give.
fn segment_ridges(
    gap: &[bool],
    best: &[[(f64, u32); 2]],
    plate_new: &[u32],
    pts: &[DVec3],
    omega: &[DVec3],
    norm: &Norm,
    seg_rad: f64,
) -> Vec<Option<(u32, f64)>> {
    let n = gap.len();
    let mut out = vec![None; n];
    if seg_rad.is_nan() || seg_rad <= 0.0 {
        return out;
    }
    let mut frames: BTreeMap<(u32, u32), Option<PoleFrame>> = BTreeMap::new();
    let mut bands: BTreeMap<(u32, u32, i64), Band> = BTreeMap::new();
    let mut points: Vec<GapPoint> = Vec::new();
    for i in 0..n {
        if !gap[i] {
            continue;
        }
        let [(_, l1), (_, l2)] = best[i];
        if l1 == u32::MAX || l2 == u32::MAX {
            continue;
        }
        let (p1, p2) = (plate_new[l1 as usize], plate_new[l2 as usize]);
        if p1 == p2 {
            continue;
        }
        let key = (p1.min(p2), p1.max(p2));
        let frame = frames.entry(key).or_insert_with(|| {
            let w = omega[key.0 as usize] - omega[key.1 as usize];
            let len = w.length();
            if len.is_nan() || len <= 0.0 {
                return None;
            }
            // A pair spreading at under 1 mm/yr anywhere has no pole worth the name.
            let f = w.any_orthogonal_vector().normalize_or_zero();
            if norm.velocity_mm_yr(w, f).length() < 1.0 {
                return None;
            }
            let e = w / len;
            Some(PoleFrame {
                e,
                f,
                g: e.cross(f),
                offset: band_offset(key.0, key.1),
            })
        });
        let Some(fr) = frame else {
            continue;
        };
        let p = pts[i];
        let theta = p.dot(fr.e).clamp(-1.0, 1.0).acos();
        let phi = p.dot(fr.g).atan2(p.dot(fr.f));
        let band = (theta / seg_rad + fr.offset).floor() as i64;
        let b = bands.entry((key.0, key.1, band)).or_default();
        b.cos += phi.cos();
        b.sin += phi.sin();
        points.push(GapPoint {
            i,
            pair: key,
            band,
            theta,
            phi,
            near_a: p1 == key.0,
        });
    }
    for b in bands.values_mut() {
        b.phi = b.sin.atan2(b.cos);
    }
    for g in &points {
        if let Some(b) = bands.get_mut(&(g.pair.0, g.pair.1, g.band)) {
            let k = usize::from(!g.near_a);
            b.side[k] += wrap(g.phi - b.phi);
            b.count[k] += 1;
        }
    }
    for g in &points {
        let (i, key, theta, phi) = (g.i, g.pair, g.theta, g.phi);
        let Some(b) = bands.get(&(key.0, key.1, g.band)) else {
            continue;
        };
        if b.count[0] == 0
            || b.count[1] == 0
            || b.side[0] / f64::from(b.count[0]) <= b.side[1] / f64::from(b.count[1])
        {
            continue;
        }
        let dphi = wrap(phi - b.phi);
        let q = if dphi > 0.0 { key.0 } else { key.1 };
        let [(d1, l1), (d2, _)] = best[i];
        let own = if plate_new[l1 as usize] == q { d1 } else { d2 };
        // The great-circle distance from the point to the segment's meridian.
        let to_ridge = (theta.sin() * dphi.sin().abs()).min(1.0).asin();
        let alpha = (to_ridge / (to_ridge + own).max(1e-12)).clamp(0.0, 1.0);
        out[i] = Some((q, alpha));
    }
    out
}

/// Resamples the crust onto a fresh lattice and returns its geometry. `tree` holds every sample
/// at `pos` (`Tree::over(pos, |_| true)`, the one the step's motion built).
pub fn resample(
    state: &mut PlateState,
    world: &mut WorldState,
    pos: &[DVec3],
    tree: &Tree,
    old: &Geo,
    env: &Env,
) -> Result<Geo, CoreError> {
    let k = env.k;
    let norm = env.norm;
    let t = env.t_myr;
    let n = state.samples.len();
    debug_assert_eq!(tree.index.len(), pos.len());
    let lattice = arr(random_rotation(&mut state.rng));
    // P1-19: the old lattice's triangulation, turned (`Geo::turned`), not a new one.
    let geo = old.turned(lattice);
    let new_pts = geo.points();
    let rots: Vec<DQuat> = state.groups.iter().map(|g| quat(g.rot)).collect();
    let src = &state.samples;

    // Which old groups hold each new point in their footprint (`footprint`: the old lattice's
    // cells, moved with the group), and which of them is on top.
    let group_plate: Vec<u32> = state.groups.iter().map(|g| g.plate).collect();
    let mut subducts: Vec<(u32, u32)> = state.contacts.iter().map(|c| (c.under, c.over)).collect();
    subducts.sort_unstable();
    subducts.dedup();
    let overrides = |over: u32, under: u32| subducts.binary_search(&(under, over)).is_ok();
    // Contacts in collision: the subducting plate's continental crust inside the overriding
    // plate's footprint crossed the front, and the belt counted it when it did (ledger: PLT-292;
    // until 2026-10-18 the part still visible at a resampling accreted to the column above it
    // as well, in lumps of a whole continental column, build log "Resampling lumps").
    let mut colliding: Vec<(u32, u32)> = state
        .contacts
        .iter()
        .filter(|c| c.collision.is_some())
        .map(|c| (c.under, c.over))
        .collect();
    colliding.sort_unstable();
    colliding.dedup();
    let collides = |over: u32, under: u32| colliding.binary_search(&(under, over)).is_ok();
    let (z_0, gravity) = (
        0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]),
        env.gravity,
    );
    let tracing = state.gap_budget.is_some();
    let old_limits = state
        .limits
        .as_ref()
        .filter(|l| l.ceiling_m.len() == src.len() && l.top_m.len() == src.len());
    let found: Vec<Found> = new_pts
        .par_iter()
        .map(|&q| {
            let mut covers: Vec<(u32, Cover)> = Vec::new();
            let mut tried: Vec<u32> = Vec::with_capacity(8);
            let near = tree.nearest_n(q, 8);
            for &(j, _) in &near {
                let g = src[j as usize].group;
                if tried.contains(&g) {
                    continue;
                }
                tried.push(g);
                if let Some(c) = footprint(old, src, g, rots[g as usize], q, j) {
                    covers.push((g, c));
                }
            }
            if covers.is_empty() {
                return (None, covers);
            }
            // On top: visible, then the overriding plate of a subduction contact, then
            // continental, then higher, then the lower group id.
            let key = |i: usize| {
                let (g, c) = &covers[i];
                let m = &src[c.major() as usize];
                (
                    !m.is_hidden(),
                    group_plate[*g as usize],
                    m.kind == CONTINENTAL,
                    m.elevation_m,
                    *g,
                )
            };
            let best = (0..covers.len())
                .max_by(|&a, &b| {
                    let (ka, kb) = (key(a), key(b));
                    ka.0.cmp(&kb.0)
                        .then_with(|| {
                            if overrides(ka.1, kb.1) {
                                std::cmp::Ordering::Greater
                            } else if overrides(kb.1, ka.1) {
                                std::cmp::Ordering::Less
                            } else {
                                std::cmp::Ordering::Equal
                            }
                        })
                        .then(ka.2.cmp(&kb.2))
                        .then(ka.3.total_cmp(&kb.3))
                        .then(kb.4.cmp(&ka.4))
                })
                .unwrap_or(0);
            let top = covers.swap_remove(best);
            (Some(top), covers)
        })
        .collect();

    // Covered samples. Continental crust a collision took across its front was stacked into its
    // belt or went down with the slab (`belts.rs`): it is consumed here, not accreted a second
    // time (ledger: PLT-276, PLT-292). Both are counted, m of rock per new sample.
    let mut plate_new: Vec<u32> = vec![u32::MAX; n];
    let mut tally: Vec<[f64; 8]> = vec![[0.0; 8]; n];
    let sediment_share = k.sediment_subduction_share as f32; // ledger: PLT-440
    // How each new sample was blended, for the gap budget (`gap_budget.rs`) when it is on.
    let mut traces: Vec<Option<gap_budget::Trace>> = vec![None; n];
    // The step's limits (`rate.rs`) on the new lattice: a covered sample's ceiling and top are
    // its corners' blended with its own weights, its floor the blended surface before the
    // accretion (new sea floor's are set with the gaps below).
    let mut limits: Vec<[f32; 3]> = vec![[f32::INFINITY, 0.0, f32::INFINITY]; n];
    let mut samples: Vec<Sample> = found
        .par_iter()
        .zip(plate_new.par_iter_mut())
        .zip(tally.par_iter_mut())
        .zip(traces.par_iter_mut())
        .zip(limits.par_iter_mut())
        .map(|(((((top, others), pl), tally), trace), lim)| match top {
            None => Sample::blank(0),
            Some((g, c)) => {
                let mut s = interpolate(src, c, rots[*g as usize]);
                *pl = group_plate[*g as usize];
                if let Some(old) = old_limits {
                    let blend = |v: &[f32]| -> f32 {
                        (0..3)
                            .map(|k| c.w[k] * f64::from(v[c.tri[k] as usize]))
                            .sum::<f64>() as f32
                    };
                    *lim = [blend(&old.ceiling_m), s.elevation_m, blend(&old.top_m)];
                }
                if tracing {
                    *trace = Some(gap_budget::Trace {
                        tri: c.tri,
                        w: c.w,
                        pools: s.pools,
                        z: s.elevation_m,
                    });
                }
                let blended = s.pools;
                for (g2, c2) in others {
                    let under = &src[c2.major() as usize];
                    let p2 = group_plate[*g2 as usize];
                    if under.is_hidden() || p2 == *pl || overrides(*pl, p2) {
                        let mut u = interpolate(src, c2, rots[*g2 as usize]);
                        let conserved: f64 = u.pools[..pool::CONSERVED]
                            .iter()
                            .map(|&x| f64::from(x))
                            .sum();
                        // Crust a collision took across its front is consumed, hidden or
                        // not: the belt stacked its share and the rest went down with the slab.
                        if (under.is_hidden() && under.stacked)
                            || (!under.is_hidden()
                                && under.kind == CONTINENTAL
                                && p2 != *pl
                                && collides(*pl, p2))
                        {
                            tally[1] += conserved;
                        } else {
                            // Sediment subduction: of ocean floor that went under (hidden, or
                            // of a plate the top overrides), the share of its sediment and
                            // sedimentary rock the slab takes down past the margin; the rest
                            // accretes. ledger: PLT-440
                            let mut subducted = 0.0_f64;
                            if sediment_share > 0.0
                                && under.kind != CONTINENTAL
                                && (under.is_hidden() || (p2 != *pl && overrides(*pl, p2)))
                            {
                                for kp in [pool::SEDIMENT, pool::SEDIMENTARY] {
                                    let down = u.pools[kp].max(0.0) * sediment_share;
                                    u.pools[kp] -= down;
                                    subducted += f64::from(down);
                                }
                            }
                            tally[7] += subducted;
                            let kept = conserved - subducted;
                            // What accretes: hidden continental crust, hidden ocean floor's
                            // sediment, or visible crust of the same or a subducting plate.
                            tally[0] += kept;
                            if !under.is_hidden() {
                                tally[3] += kept;
                            } else if under.kind == CONTINENTAL {
                                tally[2] += conserved;
                            }
                            if under.kind == CONTINENTAL {
                                tally[6] += u.pools[pool::CONSERVED..]
                                    .iter()
                                    .map(|&x| f64::from(x))
                                    .sum::<f64>();
                            }
                            accrete(&mut s.pools, &u.pools);
                        }
                    } else if under.kind == CONTINENTAL {
                        let u = interpolate(src, c2, rots[*g2 as usize]);
                        tally[5] += f64::from(u.thickness_m);
                    }
                }
                // Accreted crust stands up by its Airy height, capped at the column's, and the
                // surface gets it as uplift, as the collapse raises its receivers (the gap
                // budget's recommendation 1; plate task 8; before, accretion added thickness and
                // no height). ledger: PLT-284
                if s.kind == CONTINENTAL && !s.is_hidden() && s.pools != blended {
                    let z = f64::from(s.elevation_m);
                    let change = isostatic_change_m(&blended, &s.pools, gravity);
                    let rise = toward(z, change, iso_height(&s.pools, z_0, gravity))
                        .min(k.max_elevation_m)
                        - z;
                    if rise > 0.0 {
                        s.elevation_m = (z + rise) as f32;
                        s.uplift_mm_yr += (rise / env.dt_yr * 1.0e3) as f32;
                        tally[4] += rise;
                    }
                }
                // The accreted crust is crust uplift, whatever its rise (ledger: ISO-091).
                s.crust_uplift_mm_yr += accreted_uplift_mm_yr(&blended, &s.pools, gravity, env);
                s.thickness_m = thickness(&s.pools);
                s
            }
        })
        .collect();
    let area_km2 = norm.sample_area_km2();
    let sum = |k: usize| tally.iter().map(|t| t[k]).sum::<f64>() * area_km2 / 1000.0;
    for (name, k) in [
        (ACCRETED_GLOBAL, 0),
        (STACKED_CONSUMED_GLOBAL, 1),
        (ACCRETED_HIDDEN_CONTINENT_GLOBAL, 2),
        (ACCRETED_VISIBLE_GLOBAL, 3),
        (ACCRETED_RISE_GLOBAL, 4),
        (DROPPED_GLOBAL, 5),
        (MAFIC_DROPPED_GLOBAL, 6),
    ] {
        *world.globals.entry(name.to_owned()).or_insert(0.0) += sum(k);
    }
    // Only with the rule on, so a run with it off keeps the state it had before it existed.
    if sediment_share > 0.0 {
        *world
            .globals
            .entry(SEDIMENT_SUBDUCTED_GLOBAL.to_owned())
            .or_insert(0.0) += sum(7);
    }

    // Gaps: fresh oceanic crust between diverging plates.
    let gap: Vec<bool> = plate_new.iter().map(|&p| p == u32::MAX).collect();
    let mut sources = Vec::new();
    for v in 0..n as u32 {
        if !gap[v as usize] && geo.edges(v).any(|(u, _)| gap[u as usize]) {
            sources.push((v, 0.0, v));
        }
    }
    let best = dijkstra_two(
        &geo,
        &sources,
        |l| plate_new[l as usize],
        |_, b| gap[b as usize],
        std::f64::consts::PI,
    );
    let since = (t - state.resample_myr).max(env.dt_myr);
    let zr = k.ridge_elevation_m;
    let omega: Vec<DVec3> = state
        .plates
        .iter()
        .map(|p| DVec3::from_array(p.omega))
        .collect();
    let edge_z = |s: &Sample| -> f64 {
        if s.kind == CONTINENTAL {
            floor_at(k, since)
        } else {
            f64::from(s.elevation_m)
        }
    };
    // Ridge segments normal to the motion, joined by transforms (V1 gap 2's stage A): which plate
    // each gap point goes to and its α, where the segments say. Off (0) leaves every gap point to
    // the midline rule below. ledger: PLT-502
    let segment = (k.ridge_segment_km > 0.0).then(|| {
        segment_ridges(
            &gap,
            &best,
            &plate_new,
            new_pts,
            &omega,
            &norm,
            norm.rad(k.ridge_segment_km),
        )
    });
    let mut pairs: BTreeMap<(u32, u32), (usize, DVec3)> = BTreeMap::new();
    let mut orphans = Vec::new();
    for i in 0..n {
        if !gap[i] {
            continue;
        }
        let [(d1, l1), (d2, l2)] = best[i];
        if l1 == u32::MAX {
            orphans.push(i);
            continue;
        }
        let mut p1 = plate_new[l1 as usize];
        let z1 = edge_z(&samples[l1 as usize]);
        let (mut alpha, zbar, mut p2) = if l2 != u32::MAX {
            let z2 = edge_z(&samples[l2 as usize]);
            let s = (d1 + d2).max(1e-12);
            (
                ((d2 - d1) / s).clamp(0.0, 1.0),
                (d2 * z1 + d1 * z2) / s,
                Some(plate_new[l2 as usize]),
            )
        } else {
            (1.0, z1, None)
        };
        if let Some((q, a)) = segment.as_ref().and_then(|s| s[i]) {
            if q != p1 {
                p2 = Some(p1);
                p1 = q;
            }
            alpha = a;
        }
        let age = alpha * since;
        let p = new_pts[i];
        let z = alpha * zbar + (1.0 - alpha) * floor_at(k, age);
        let mut s = Sample::blank(0);
        s.kind = OCEANIC;
        s.regime = regime::RIDGE;
        s.elevation_m = z as f32;
        s.formed_myr = (t - age) as f32;
        s.pools = oceanic_column();
        s.pools[pool::SEDIMENT] = (k.pelagic_m_per_myr * age) as f32;
        s.thickness_m = thickness(&s.pools);
        s.heat_flow_mw_m2 = ocean_heat_flow(age) as f32;
        if let Some(p2) = p2 {
            let spread = norm.velocity_mm_yr(omega[p1 as usize] - omega[p2 as usize], p);
            let r = p.cross(spread).normalize_or_zero();
            s.ridge = [r.x as f32, r.y as f32, r.z as f32];
            let e = pairs
                .entry((p1.min(p2), p1.max(p2)))
                .or_insert((0, DVec3::ZERO));
            e.0 += 1;
            e.1 += p;
        }
        samples[i] = s;
        plate_new[i] = p1;
    }
    for i in orphans {
        // Unreachable gap pockets: the nearest old sample's plate.
        let (j, _) = tree.nearest(new_pts[i]).unwrap_or((0, 0.0));
        let mut s = Sample::blank(0);
        s.pools = oceanic_column();
        s.thickness_m = thickness(&s.pools);
        s.elevation_m = zr as f32;
        s.formed_myr = t as f32;
        samples[i] = s;
        plate_new[i] = group_plate[state.samples[j as usize].group as usize];
    }

    // One unrotated group per plate.
    let mut group_of: BTreeMap<u32, u32> = BTreeMap::new();
    let mut groups: Vec<Group> = Vec::new();
    for p in state.alive() {
        group_of.insert(p.id, groups.len() as u32);
        groups.push(Group {
            plate: p.id,
            rot: [0.0, 0.0, 0.0, 1.0],
        });
    }
    for (s, &p) in samples.iter_mut().zip(&plate_new) {
        s.group = match group_of.get(&p) {
            Some(&g) => g,
            None => {
                // A plate that died this step still owned crust: keep it alive.
                let g = groups.len() as u32;
                group_of.insert(p, g);
                groups.push(Group {
                    plate: p,
                    rot: [0.0, 0.0, 0.0, 1.0],
                });
                g
            }
        };
    }
    // New sea floor starts the step's limits as it stands.
    let new_limits = old_limits.map(|_| {
        let a = super::rate::allowance_m(env).unwrap_or(f64::INFINITY);
        let r = super::rate::rise_m(env).unwrap_or(f64::INFINITY);
        let mut l = super::rate::Limits::default();
        for (s, (x, &g)) in samples.iter().zip(limits.iter().zip(&gap)) {
            let x = if g {
                let z = f64::from(s.elevation_m);
                [
                    (f64::from(s.thickness_m) + a) as f32,
                    z as f32,
                    (z + r) as f32,
                ]
            } else {
                *x
            };
            l.ceiling_m.push(x[0]);
            l.floor_m.push(x[1]);
            l.top_m.push(x[2]);
        }
        l
    });
    state.samples = samples;
    state.limits = new_limits;
    if let Some(gb) = state.gap_budget.as_mut() {
        // Gap points became new sea floor: born now.
        for (t, &g) in traces.iter_mut().zip(&gap) {
            if g {
                *t = None;
            }
        }
        gb.resampled(&state.samples, &traces);
    }
    state.groups = groups;
    state.lattice = lattice;
    state.resample_step = state.step;
    state.resample_myr = t;
    state.counters.resamplings += 1;
    let resampling = state.counters.resamplings;

    // Ridge births.
    let min_ridge = 3.max(n / 2000);
    for ((a, b), (count, sum)) in pairs {
        if count < min_ridge {
            continue;
        }
        let at = sum.normalize_or(DVec3::Z);
        if let Some(r) = state.ridges.iter_mut().find(|r| r.a == a && r.b == b) {
            r.last_resampling = resampling;
            continue;
        }
        let spread = norm.speed_mm_yr(omega[a as usize] - omega[b as usize], at);
        let mut e = events::event(world, t, RIDGE_BIRTH, at)
            .magnitude((spread / k.convergence_ref_mm_yr) as f32)
            .param("plate_a", i64::from(a))
            .param("plate_b", i64::from(b))
            .param("spreading_mm_yr", spread)
            .param("new_samples", count as i64);
        if let Some(link) = state.rift_links.iter().find(|l| l.a == a && l.b == b) {
            e = e.caused_by(link.event);
        }
        let id = world.emit(e)?;
        state.counters.ridges += 1;
        state.ridges.push(Ridge {
            a,
            b,
            event: id,
            last_resampling: resampling,
        });
    }
    Ok(geo)
}

/// Marks plates without samples dead and, when `geo` is the lattice every group shares (right
/// after a resampling), absorbs plates below `min_plate_area_frac` into the neighbour they share
/// most lattice edges with. `pos` are current positions (for centroids). `causes` maps a plate to
/// the event that doomed it (its last subduction onset, a collision).
pub fn bury(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: Option<&Geo>,
    pos: &[DVec3],
    env: &Env,
    causes: &BTreeMap<u32, EventId>,
) -> Result<(), CoreError> {
    let t = env.t_myr;
    state.recount(pos);
    if let Some(geo) = geo {
        let min = (env.k.min_plate_area_frac * f64::from(state.n)).ceil() as u32;
        while let Some(small) = state
            .alive()
            .filter(|p| p.samples > 0 && p.samples < min)
            .min_by_key(|p| (p.samples, p.id))
            .map(|p| p.id)
        {
            let mut votes: BTreeMap<u32, usize> = BTreeMap::new();
            for i in 0..state.samples.len() {
                if state.plate_of(i) != small {
                    continue;
                }
                for (u, _) in geo.edges(i as u32) {
                    let pu = state.plate_of(u as usize);
                    if pu != small {
                        *votes.entry(pu).or_default() += 1;
                    }
                }
            }
            let Some((&into, _)) = votes
                .iter()
                .max_by_key(|(p, n)| (**n, std::cmp::Reverse(**p)))
            else {
                break;
            };
            let target_group = state
                .groups
                .iter()
                .position(|g| g.plate == into)
                .expect("a plate with samples has a group") as u32;
            for s in &mut state.samples {
                if state.groups[s.group as usize].plate == small {
                    s.group = target_group;
                }
            }
            let at = DVec3::from_array(state.plates[small as usize].centroid);
            let e = events::event(world, t, PLATE_DEATH, at)
                .param("plate", i64::from(small))
                .param("fate", "absorbed")
                .param("absorbed_by", i64::from(into));
            world.emit(e)?;
            let p = &mut state.plates[small as usize];
            p.alive = false;
            p.died_myr = Some(t);
            p.samples = 0;
            state.counters.plate_deaths += 1;
            state.recount(pos);
        }
        // Groups of plates absorbed above are now empty; drop them and renumber.
        let used: Vec<bool> = {
            let mut u = vec![false; state.groups.len()];
            for s in &state.samples {
                u[s.group as usize] = true;
            }
            u
        };
        if used.iter().any(|&u| !u) {
            let mut remap = vec![u32::MAX; state.groups.len()];
            let mut groups = Vec::new();
            for (g, grp) in state.groups.iter().enumerate() {
                if used[g] {
                    remap[g] = groups.len() as u32;
                    groups.push(*grp);
                }
            }
            for s in &mut state.samples {
                s.group = remap[s.group as usize];
            }
            state.groups = groups;
        }
    }
    let dead: Vec<u32> = state
        .alive()
        .filter(|p| p.samples == 0)
        .map(|p| p.id)
        .collect();
    for id in dead {
        let at = DVec3::from_array(state.plates[id as usize].centroid);
        let mut e = events::event(world, t, PLATE_DEATH, at)
            .param("plate", i64::from(id))
            .param("fate", "consumed");
        if let Some(&c) = causes.get(&id) {
            e = e.caused_by(c);
        }
        world.emit(e)?;
        let p = &mut state.plates[id as usize];
        p.alive = false;
        p.died_myr = Some(t);
        state.counters.plate_deaths += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_offsets_are_fractions_and_wrap_folds_into_a_turn() {
        for (a, b) in [(0, 1), (3, 17), (39, 40), (1, 0)] {
            let o = band_offset(a, b);
            assert!((0.0..1.0).contains(&o), "{o}");
        }
        assert_ne!(band_offset(0, 1), band_offset(0, 2));
        use std::f64::consts::PI;
        assert!((wrap(4.0) - (4.0 - 2.0 * PI)).abs() < 1e-12);
        assert!((wrap(-0.5) + 0.5).abs() < 1e-12);
        assert!((wrap(2.0 * PI + 0.25) - 0.25).abs() < 1e-12);
    }

    /// A gap 6° wide and 35° oblique to the spreading (Earth's envelope, plates.md "Earth's ridges
    /// and transforms") between plate a, turning about ẑ, and plate b, still; points `step` apart
    /// in the pair's frame, the gap points first and one sample of each plate last.
    struct Gap {
        gap: Vec<bool>,
        best: Vec<[(f64, u32); 2]>,
        plate_new: Vec<u32>,
        pts: Vec<DVec3>,
        omega: Vec<DVec3>,
    }

    impl Gap {
        fn segments(&self, norm: &Norm, seg_rad: f64) -> Vec<Option<(u32, f64)>> {
            segment_ridges(
                &self.gap,
                &self.best,
                &self.plate_new,
                &self.pts,
                &self.omega,
                norm,
                seg_rad,
            )
        }
    }

    fn oblique_gap(omega_a: DVec3) -> Gap {
        use std::f64::consts::FRAC_PI_2;
        let e = DVec3::Z;
        let f = e.any_orthogonal_vector().normalize();
        let g = e.cross(f);
        let at =
            |theta: f64, phi: f64| (f * phi.cos() + g * phi.sin()) * theta.sin() + e * theta.cos();
        let (half, step) = (3f64.to_radians(), 0.2f64.to_radians());
        let mid = |theta: f64| 35f64.to_radians().tan() * (theta - FRAC_PI_2);
        let mut grid = Vec::new();
        for r in -50..=50 {
            let theta = FRAC_PI_2 + f64::from(r) * step;
            for c in 0..30 {
                grid.push((theta, mid(theta) - half + (f64::from(c) + 0.5) * step));
            }
        }
        let m = grid.len() as u32;
        let mut pts: Vec<DVec3> = grid.iter().map(|&(t, p)| at(t, p)).collect();
        pts.extend([at(FRAC_PI_2, 1.0), at(FRAC_PI_2, -1.0)]);
        let mut best: Vec<[(f64, u32); 2]> = grid
            .iter()
            .map(|&(t, p)| {
                // a's edge on the +φ side, b's on the −φ side.
                let (da, db) = ((mid(t) + half - p) * t.sin(), (p - mid(t) + half) * t.sin());
                if da <= db {
                    [(da, m), (db, m + 1)]
                } else {
                    [(db, m + 1), (da, m)]
                }
            })
            .collect();
        best.extend([[(f64::INFINITY, u32::MAX); 2]; 2]);
        let mut gap = vec![true; m as usize];
        gap.extend([false, false]);
        let mut plate_new = vec![0; m as usize];
        plate_new.extend([0, 1]);
        Gap {
            gap,
            best,
            plate_new,
            pts,
            omega: vec![omega_a, DVec3::ZERO],
        }
    }

    /// Gerya 2012 (§1): "The perpendicularity of the ridges and the transform faults is so common
    /// that it is considered to be an 'intrinsic property' of the oceanic spreading process". On a
    /// gap 35° oblique to the spreading, each band's ridge is one meridian of the relative pole
    /// (normal to the motion, which runs along the small circles), the steps between the bands run
    /// along the motion, and their sum is tan 35° of the ridge's (a ridge normal to the motion and
    /// transforms along it on an envelope α from orthogonal: transform over ridge = tan α; PB2002's
    /// 0.71 is about 35°). α is 0 at the ridge and 1 at the plates' edges. A converging pair, or the
    /// switch off, keeps the midline. ledger: PLT-502
    #[test]
    fn ridge_segments_stand_normal_to_the_motion_with_transforms_along_it() {
        let norm = Norm::new(6370.0, 782.0, 100_000);
        let spread = DVec3::Z * norm.omega_of_speed(50.0);
        let gap = oblique_gap(spread);
        let m = gap.gap.iter().filter(|&&g| g).count();
        let seg = 4f64.to_radians();
        let out = gap.segments(&norm, seg);
        assert!(
            out[..m].iter().all(Option::is_some),
            "every gap point is on a segment"
        );
        assert!(out[m..].iter().all(Option::is_none));
        // The pair's frame as `segment_ridges` takes it.
        let e = DVec3::Z;
        let f = e.any_orthogonal_vector().normalize();
        let g = e.cross(f);
        let offset = band_offset(0, 1);
        // Per band: the φ of the cut between b's points and a's in each row of constant θ.
        let mut cuts: BTreeMap<i64, Vec<f64>> = BTreeMap::new();
        let mut rows: BTreeMap<i64, (i64, f64, f64)> = BTreeMap::new();
        for (i, p) in gap.pts[..m].iter().enumerate() {
            let theta = p.dot(e).clamp(-1.0, 1.0).acos();
            let phi = p.dot(g).atan2(p.dot(f));
            let (q, alpha) = out[i].unwrap();
            assert!((0.0..=1.0).contains(&alpha), "{alpha}");
            let row = (theta.to_degrees() * 5.0).round() as i64;
            let band = (theta / seg + offset).floor() as i64;
            let r = rows
                .entry(row)
                .or_insert((band, f64::INFINITY, f64::NEG_INFINITY));
            if q == 0 {
                r.1 = r.1.min(phi);
            } else {
                r.2 = r.2.max(phi);
            }
        }
        for &(band, min_a, max_b) in rows.values() {
            assert!(min_a > max_b, "a's points all on the +φ side of b's");
            cuts.entry(band).or_default().push(0.5 * (min_a + max_b));
        }
        // One meridian per band: every row's cut within a grid step of the band's. The first
        // and last bands are cut short by the gap's ends; the others hold 20 rows each.
        let step = 0.2f64.to_radians();
        let inner: Vec<&Vec<f64>> = cuts.values().skip(1).collect();
        let full: Vec<f64> = inner[..inner.len() - 1]
            .iter()
            .map(|c| {
                assert!((19..=21).contains(&c.len()), "{} rows", c.len());
                let lo = c.iter().copied().fold(f64::INFINITY, f64::min);
                let hi = c.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                assert!(
                    hi - lo < step + 1e-9,
                    "the band's ridge is one meridian: {lo} to {hi}"
                );
                c.iter().sum::<f64>() / c.len() as f64
            })
            .collect();
        assert!(full.len() >= 3, "{} full bands", full.len());
        // The steps between consecutive full bands: transforms, tan 35° of the ridge's length.
        let transform: f64 = full.windows(2).map(|w| (w[1] - w[0]).abs()).sum();
        let ratio = transform / ((full.len() - 1) as f64 * seg);
        let want = 35f64.to_radians().tan();
        assert!(
            (ratio - want).abs() < 0.05,
            "transform / ridge {ratio}, tan 35° {want}"
        );
        // α: near 0 beside the ridge, near 1 at the plates' edges.
        let alphas: Vec<f64> = out[..m].iter().map(|o| o.unwrap().1).collect();
        assert!(alphas.iter().copied().fold(f64::INFINITY, f64::min) < 0.05);
        assert!(alphas.iter().copied().fold(0.0, f64::max) > 0.9);
        // A converging pair (a turning the other way) keeps the midline; so does the switch off.
        let back = oblique_gap(-spread).segments(&norm, seg);
        assert!(
            back.iter().all(Option::is_none),
            "a converging pair keeps the midline"
        );
        assert!(gap.segments(&norm, 0.0).iter().all(Option::is_none));
    }
}
