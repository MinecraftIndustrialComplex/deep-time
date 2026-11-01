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
//!   With `ridge_segment_km` (on by default since 2026-10-23, ledger PLT-502) the gap between
//!   two diverging plates is cut into ridge segments normal to their motion
//!   (`segment_ridges`): which plate a point goes to and its α follow its band's segment, and
//!   the nearest plate and the midline above hold only where a band keeps them.
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
use super::events::{self, PLATE_BIRTH, PLATE_DEATH, RIDGE_BIRTH};
use super::gap_budget;
use super::geometry::{Cover, Geo, Tree, connect_plates, dijkstra_two, pieces};
use super::init::random_rotation;
use super::kinematics::{arr, quat, to_world};
use super::level_set;
use super::normalise::Norm;
use super::pools::{
    accrete, floor_at, isostatic_change_m, ocean_heat_flow, oceanic_column, thickness,
};
use super::settle::{WATER_LOADED, equilibrium_m};
use super::state::{
    CONTINENTAL, FORELAND, Group, OCEANIC, Plate, PlateState, Ridge, Sample, pool, regime,
};
use crate::error::CoreError;
use crate::params::TectonicParams;
use crate::world::WorldState;

/// What covers one new point: the group on top (if any) and the others.
type Found = (Option<(u32, Cover)>, Vec<(u32, Cover)>);

/// Clift & Vannucchi (2004) §34: accretion "appears to be an effective process when convergent
/// rates are <76 mm yr-1 and where the sedimentary cover is >1 km thick" …
pub const ACCRETIONARY_PILE_M: f64 = 1000.0;
/// … (the convergence, mm/yr) …
pub const ACCRETIONARY_MAX_MM_YR: f64 = 76.0;
/// … and §53: "Even at the accretionary margins, typically 83% of the incoming sedimentary pile
/// is subducted beneath the forearc wedge" (the median accreted, §39, is 17 %).
pub const ACCRETIONARY_SUBDUCTED: f64 = 0.83;

/// Sediment subduction by margin type (`sediment_subduction_sourced`, PLT-660): the share of the
/// incoming sedimentary pile `pile_m` (the subducted floor's sediment and sedimentary rock, m)
/// that goes down with the slab at a convergence of `convergence_mm_yr` (the overriding side's
/// `subduction::orthogonal_convergence_mm_yr`: the component of the plates' relative motion
/// normal to the front, PLT-312, whatever `DEEPTIME_ARC_NORMAL` sets for the arcs, since Clift &
/// Vannucchi's 76 mm/yr is an orthogonal rate). An accretionary margin (a pile over
/// 1 km and convergence under 76 mm/yr) subducts 0.83 of it (Clift & Vannucchi 2004); any other
/// margin is erosive and subducts all of it (Clift & Vannucchi 2004; von Huene & Scholl 1991's
/// type 2 margins, where "effectively all incoming sediment is subducted"). The rule does not ask
/// what the overriding plate is made of: an intra-oceanic margin with a thin pelagic cover (the
/// Marianas, Tonga) is erosive, one fed by a thick fan (the Lesser Antilles) accretes.
// ledger: PLT-660
pub fn sediment_subducted_share(pile_m: f64, convergence_mm_yr: f64) -> f64 {
    sediment_subducted_share_at(pile_m, convergence_mm_yr, ACCRETIONARY_PILE_M)
}

/// [`sediment_subducted_share`] with the accretionary pile's line `line_m`, m of rock: 1 km
/// (`ACCRETIONARY_PILE_M`), or with `trench_pile_in_place` the rock a 1 km porous column holds
/// ([`trench_line_solid_m`], PLT-762).
pub fn sediment_subducted_share_at(pile_m: f64, convergence_mm_yr: f64, line_m: f64) -> f64 {
    if pile_m > line_m && convergence_mm_yr.abs() < ACCRETIONARY_MAX_MM_YR {
        ACCRETIONARY_SUBDUCTED
    } else {
        1.0
    }
}

/// A trench sediment line of Clift & Vannucchi's (2004), `line_m` of sediment at the trench, as
/// the model's solid pools hold it: with `in_place` the rock in a porous column `line_m` thick
/// under ERO-215's porosity law (`surface::pelagic::solid_of_in_place_m`; 1 km holds 536 m), else
/// `line_m` itself. C&V [31]: Table 1's thickness is "the thickness of sediment (uncorrected for
/// porosity) on the subducting plate at the trench axis", and their accretion criterion is
/// "trench sediment thicknesses >1 km" (abstract). ledger: PLT-762
pub fn trench_line_solid_m(in_place: bool, line_m: f64) -> f64 {
    if in_place {
        crate::surface::pelagic::solid_of_in_place_m(line_m)
    } else {
        line_m
    }
}

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

/// The old sample of cover `c` (a footprint of the group with rotation `rot` in the old lattice
/// `geo`) nearest the mantle-frame direction `q`: of the cover's corners with a weight (the
/// group's own), the one at the least angle from `q` in the group's frame. A resampled sample
/// takes its categorical labels from it (`interpolate`): the material at `q` is the material of
/// the old sample whose cell holds it, not of the corner with the largest barycentric weight,
/// whose region in a triangle that is not equilateral is not the corner's cell (ledger: PLT-664).
/// It keeps a class's area on average, so its edge walks; a vote of the corners' weight at one
/// half smooths the edge but eats every body a fixed fraction of a cell per resampling, an island
/// arc's band whole within a run, and was rejected (2026-10-25, plates.md "Which old sample a new
/// one follows").
fn nearest_corner(geo: &Geo, c: &Cover, rot: DQuat, q: DVec3) -> u32 {
    let local = rot.inverse() * q;
    let pts = geo.points();
    let mut best = c.major();
    let mut best_dot = pts[best as usize].dot(local);
    for k in 0..3 {
        if c.w[k] > 0.0 {
            let d = pts[c.tri[k] as usize].dot(local);
            if d > best_dot {
                (best, best_dot) = (c.tri[k], d);
            }
        }
    }
    best
}

/// What a resampling measures a column's height from: the height its own rock and age hold it at,
/// m (ledger: PLT-850). Continental crust: its pools' Airy height (`collapse::iso_height`, about
/// the continents' reference height) and its lithosphere's thermal height (`thermal::uplift_m`,
/// ISO-110; 0 when off), the dry height the settling holds it to (`settle.rs`, ISO-078). Ocean
/// crust: the floor its age's law makes (`pools::floor_at`, GDH1 by default, PLT-351) and the
/// Airy change of its pools over the ocean's own 7 km column (`pools::isostatic_change_m`), water
/// loaded under the datum as the hotspots' magma is (`hotspots::add_magma`; ISO-044): the floor
/// law's depth is the datum's sea's, whatever the sea stands at. The step's settling then holds a
/// continental column to its sea; ocean floor has no settling (the law sinks it by its age's
/// increments, PLT-352, and the trench release lifts only floor under the curve, PLT-443), so a
/// height the resampling gave it that its rock does not hold would stay.
pub(crate) fn rock_height_m(s: &Sample, k: &TectonicParams, gravity: f64, t_myr: f64) -> f64 {
    if s.kind == CONTINENTAL {
        let z_0 = 0.5 * (k.continent_elevation_m[0] + k.continent_elevation_m[1]);
        iso_height(&s.pools, z_0, gravity) + super::thermal::uplift_m(k, s.lithosphere_km)
    } else {
        let dry = floor_at(k, s.age_myr(t_myr)) / WATER_LOADED
            + isostatic_change_m(&oceanic_column(), &s.pools, gravity);
        equilibrium_m(dry, 0.0)
    }
}

/// The tectonics, gravity and time [`rock_height_m`] needs at a resampling.
#[derive(Clone, Copy)]
struct Heights<'a> {
    k: &'a TectonicParams,
    gravity: f64,
    t_myr: f64,
}

impl Heights<'_> {
    fn of(&self, s: &Sample) -> f64 {
        rock_height_m(s, self.k, self.gravity, self.t_myr)
    }
}

/// Interpolates a sample from a cover of the old crust; vectors come out in the mantle frame,
/// which is the new lattice's frame (every group starts unrotated). `m` is the old sample nearest
/// the new point (`nearest_corner`): the sample's kind, its class and every other categorical
/// label are `m`'s, and the blends below run over the corners of `m`'s class (ledger: PLT-664).
///
/// One rule for every field it blends (ledger: PLT-851): a field of the column (its rock, its
/// records of that rock, its age, heat flow, rates and directions) is blended over the corners
/// whose rock the sample takes, the corners of its class; a field that records rock taken from
/// the nearest corner alone (the felsic and metamorphic pools on ocean crust, PLT-293) is that
/// corner's. The height follows its rock (PLT-850): the sample stands at the height its own rock
/// and age hold it at ([`rock_height_m`]) plus its class corners' blended departures from theirs
/// (what flexure, a trench's pull, a margin's held floor or a step's lag held them at). Until
/// 2026-10-31 the height was blended over all three corners: ocean floor beside a continent or an
/// arc took a share of its height and none of its rock, the continent settled back within the
/// step and the floor kept it, a lift that grew with each resampling (plates.md, "The height
/// follows the rock").
fn interpolate(src: &[Sample], c: &Cover, rot: DQuat, m: u32, h: Heights) -> Sample {
    let mut s = src[m as usize];
    let corners = c.tri.map(|v| &src[v as usize]);
    let w = c.w;
    let lerp = |f: &dyn Fn(&Sample) -> f32| -> f32 {
        (0..3).map(|k| w[k] * f64::from(f(corners[k]))).sum::<f64>() as f32
    };
    // The corners of the sample's own class, the kind and, on continental crust, whether it is
    // hyper-extended (under 10 km of its own crystalline crust, Brune et al. 2014), and their
    // weight; the nearest corner is one of them, so the weight is never 0. A blend over a class is
    // a convex mean of columns on one side of the 10 km line, and stays on that side. Blending a
    // converted margin's columns (PLT-661: an ocean basement under a thin continental crust) with
    // the core's put 10 km or more of the core's crust into them at a resampling: they stopped
    // being hyper-extended, became the taper's continental side, and the ring beyond them turned
    // at the next step, so the margin moved out at every resampling (the ratchet, arm
    // m5-margins). The rule holds with the switches off: an old arc's thin continental crust
    // stays apart from the thick crust beside it too (ledger: PLT-663).
    let hyper = super::transition::hyper_extended(&src[m as usize]);
    let same_class = |x: &Sample| {
        x.kind == s.kind && (s.kind != CONTINENTAL || super::transition::hyper_extended(x) == hyper)
    };
    let class_w: f64 = (0..3)
        .filter(|&k| same_class(corners[k]))
        .map(|k| w[k])
        .sum();
    // A field over the class's corners, normalised by their weight (over all three when none has
    // a weight, which the nearest corner's own weight rules out).
    let class_blend = |f: &dyn Fn(&Sample) -> f64| -> f64 {
        if class_w > 0.0 {
            (0..3)
                .filter(|&k| same_class(corners[k]))
                .map(|k| w[k] * f(corners[k]))
                .sum::<f64>()
                / class_w
        } else {
            (0..3).map(|k| w[k] * f(corners[k])).sum()
        }
    };
    // The crust's own rock belongs to its kind: the rock pools are blended over the corners of
    // the sample's class only, the loose sediment (which drapes across a margin) over all three.
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
        } else if p == pool::SEDIMENT {
            lerp(&|x| x.pools[p])
        } else {
            class_blend(&|x| f64::from(x.pools[p])) as f32
        };
    }
    // The records of the arcs' magma and its oxidation follow the rock they record: on ocean crust
    // the island arc's felsic rock is the nearest corner's (above), so its records are too (the
    // sample's own, from `m`); on continental crust they are blended as its pools are. Over all
    // three corners until 2026-10-31 (ledger: PLT-851).
    if !oceanic {
        s.arc_continental_m = class_blend(&|x| f64::from(x.arc_continental_m)) as f32;
        s.arc_oceanic_m = class_blend(&|x| f64::from(x.arc_oceanic_m)) as f32;
        s.oxidation = class_blend(&|x| f64::from(x.oxidation)) as f32;
    }
    // A basement is the continental crust's own (ocean floor's is 0), blended as its pools are.
    s.basement_m = if s.kind == CONTINENTAL {
        class_blend(&|x| f64::from(x.basement())) as f32
    } else {
        0.0
    };
    // The column's rates, its age and its heat flow belong to its rock as well (ledger: PLT-851).
    // Heat flow over the class's corners (the sea floor's age law against the continental sum, and
    // the thin crust's own: a continental sample on the coast must not take the young ridge crust's
    // 300 mW/m² from an oceanic corner). The next step sets every sample's heat flow again; this
    // only matters for what is read right after a resampling, the final snapshot among them: the
    // bake's `meta/heat_flow_mw_m2`.
    s.uplift_mm_yr = class_blend(&|x| f64::from(x.uplift_mm_yr)) as f32;
    s.crust_uplift_mm_yr = class_blend(&|x| f64::from(x.crust_uplift_mm_yr)) as f32;
    s.heat_flow_mw_m2 = class_blend(&|x| f64::from(x.heat_flow_mw_m2)) as f32;
    if oceanic {
        s.formed_myr = class_blend(&|x| f64::from(x.formed_myr)) as f32;
    }
    let vec = |f: &dyn Fn(&Sample) -> [f32; 3]| -> [f32; 3] {
        let v: DVec3 = (0..3)
            .filter(|&k| class_w <= 0.0 || same_class(corners[k]))
            .map(|k| to_world(rot, f(corners[k])) * w[k])
            .sum::<DVec3>()
            / if class_w > 0.0 { class_w } else { 1.0 };
        [v.x as f32, v.y as f32, v.z as f32]
    };
    s.ridge = vec(&|x| x.ridge);
    s.fold = vec(&|x| x.fold);
    // The step's flags are set again next step; the collision's footprint (`belts.rs`) goes with
    // its convergence and shortening, which the nearest corner passes on, so the structure stage
    // never sees the one without the other after a resampling.
    s.flags &= FORELAND;
    s.thickness_m = thickness(&s.pools);
    // The height last, on the sample's own rock and age (ledger: PLT-850).
    let departure = class_blend(&|x| f64::from(x.elevation_m) - h.of(x));
    let z = h.of(&s) + departure;
    s.elevation_m = if z.is_finite() {
        z as f32
    } else {
        class_blend(&|x| f64::from(x.elevation_m)) as f32
    };
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

/// Group `group`'s footprint as `footprint` finds it, with the outline below the lattice spacing
/// (`tectonics.level_set_membership`, ledger PLT-536; `level_set.rs`): in the old triangle holding
/// `q` in the group's frame, the group holds `q` when its corners' distances `phi` from their
/// groups' outlines, weighed with `q`'s barycentric weights, outweigh every other group's
/// (`level_set::margin`), not when its corner has the largest weight. Returns the cover (the
/// weights of the group's own corners, as `footprint`'s) when the triangle has a corner of the
/// group, and `q`'s sine distance from the group's outline, negative outside it.
fn footprint_level_set(
    geo: &Geo,
    src: &[Sample],
    phi: &[f64],
    group: u32,
    rot: DQuat,
    q: DVec3,
    hint: u32,
) -> (Option<Cover>, f64) {
    let local = rot.inverse() * q;
    let h = geo.mesh.vertex_triangle(hint).unwrap_or(0);
    let t = geo.mesh.locate(local, h);
    let tri = geo.mesh.triangles()[t as usize];
    let mut w = geo.mesh.barycentric(t, local);
    let groups = tri.map(|v| src[v as usize].group);
    let pts = geo.points();
    let d = level_set::margin(
        groups,
        w,
        tri.map(|v| phi[v as usize]),
        tri.map(|v| pts[v as usize]),
        group,
    );
    let own = groups.map(|g| g == group);
    if !own.contains(&true) {
        return (None, d);
    }
    if own.contains(&false) {
        let sum: f64 = (0..3).filter(|&k| own[k]).map(|k| w[k]).sum();
        if sum > 0.0 {
            for k in 0..3 {
                w[k] = if own[k] { w[k] / sum } else { 0.0 };
            }
        } else {
            // `q` on the edge facing the group's only corners: that corner's crust.
            let k = (0..3).find(|&k| own[k]).unwrap_or(0);
            w = [0.0; 3];
            w[k] = 1.0;
        }
    }
    (Some(Cover { t, tri, w }), d)
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
/// World global (measurement only, PLT-760): the new continental samples of the resamplings whose
/// island-arc class, carried from the nearest corner (`Sample::island_arc_marked`), the blend of
/// their columns' basement and island-arc crust against the 8 km line would have lost (a matured
/// arc's edge beside its ocean flank, PR #53's leak B), km² summed over the resamplings …
pub const ISLAND_CLASS_KEPT_GLOBAL: &str = "plates/resample_island_class_kept_km2";
/// … and would have given (a continent's own column beside an accreted arc, leak A), km².
pub const ISLAND_CLASS_REFUSED_GLOBAL: &str = "plates/resample_island_class_refused_km2";
/// World global: subducted ocean floor's sediment and sedimentary rock that went down with the
/// slab at resamplings instead of accreting (`sediment_subduction_share`, PLT-440), km³
/// (Earth-normalized; the crust budget's sediment subduction against Stern & Scholl 2010).
pub const SEDIMENT_SUBDUCTED_GLOBAL: &str = "plates/sediment_subducted_km3";
/// World global (measurement only, for the sediment diagnosis, `tools/review/seddiag/`): the
/// sediment and sedimentary rock of ocean floor that goes under at resamplings, before the share
/// the slab takes (PLT-440, PLT-660) is applied, slab path included, km³ (Earth-normalized),
/// whether or not sediment subduction is on. It separates what reaches the trenches from what
/// they keep. Split by the crust type of the sample on top: continental …
pub const SEDIMENT_REACHING_TRENCH_GLOBAL: &str = "plates/sediment_reaching_trench_km3";
pub const SEDIMENT_REACHING_TRENCH_CONTINENTAL_GLOBAL: &str =
    "plates/sediment_reaching_trench_continental_km3";
/// … and oceanic; the rest of the total is the one-way slab's (`SLAB_GLOBAL`'s points, which have
/// no sample on top).
pub const SEDIMENT_REACHING_TRENCH_OCEANIC_GLOBAL: &str =
    "plates/sediment_reaching_trench_oceanic_km3";
/// World global (one-way subduction, `tectonics.one_way_subduction`, only while it is on): the
/// conserved pools of crust that went down a trench and that only held crust covered at a
/// resampling, gone down with the slab (the point became new sea floor), km³ (Earth-normalized).
/// Its sediment is in [`SEDIMENT_SUBDUCTED_GLOBAL`] too. ledger: PLT-542
pub const SLAB_GLOBAL: &str = "plates/resample_slab_km3";
/// World globals (`tectonics.volcanic_margins`, PLT-799, only while it is on): the igneous crust
/// volcanic rifted margins made at resamplings beyond the ocean basement they keep under it
/// (PLT-331), the continental crust they add, km³ (Earth-normalized), and the gap points that
/// took it.
pub const VOLCANIC_MARGIN_GLOBAL: &str = "plates/volcanic_margin_km3";
pub const VOLCANIC_MARGIN_SAMPLES_GLOBAL: &str = "plates/volcanic_margin_samples";

/// A plume head's hot mantle, as the volcanic margins read it (PLT-799): where it rose (mantle
/// frame), when, how far it spread under the lithosphere (radians on the model sphere) and how
/// hot it is above the normal mantle, K.
#[derive(Clone, Copy, Debug)]
pub struct HotMantle {
    pub at: DVec3,
    pub from_myr: f64,
    pub radius_rad: f64,
    pub excess_k: f64,
}

/// The hot mantle under a point of the mantle frame `at` at `t_myr`: the hottest of `heads`
/// whose spread covers it while it lasts (`super::hotspots::sourced::HEAD_HOT_MYR` from its
/// rise), or `None`. ledger: PLT-798
pub fn hot_mantle_at(heads: &[HotMantle], at: DVec3, t_myr: f64) -> Option<HotMantle> {
    heads
        .iter()
        .filter(|h| {
            t_myr >= h.from_myr
                && t_myr <= h.from_myr + super::hotspots::sourced::HEAD_HOT_MYR
                && at.angle_between(h.at) <= h.radius_rad
        })
        .copied()
        .max_by(|a, b| a.excess_k.total_cmp(&b.excess_k))
}

/// A volcanic rifted margin's column (PLT-799): the igneous crust mantle `excess_k` hotter than
/// normal melts to when it rises to the surface under full breakup
/// (`super::hotspots::sourced::igneous_crust_m`), lavas over intrusions in the share
/// `super::hotspots::sourced::MARGIN_EXTRUSIVE_SHARE`, on new sea floor whose ocean crust is
/// the ocean column (PLT-104), over `share` of the point's cell (the rest the ocean column; see
/// [`margin_share`]). `None` where it is no thicker than the ocean column.
pub fn volcanic_margin_column(excess_k: f64, share: f64) -> Option<[f32; pool::COUNT]> {
    let ocean = f64::from(thickness(&oceanic_column()));
    let h = ocean
        + share.clamp(0.0, 1.0) * (super::hotspots::sourced::igneous_crust_m(excess_k) - ocean);
    if h.is_nan() || h <= ocean {
        return None;
    }
    let mut p = [0.0f32; pool::COUNT];
    let e = super::hotspots::sourced::MARGIN_EXTRUSIVE_SHARE;
    p[pool::MAFIC_VOLCANIC] = (h * e) as f32;
    p[pool::MAFIC_PLUTONIC] = (h * (1.0 - e)) as f32;
    Some(p)
}

/// The share of a gap point's cell inside a volcanic margin's `width_rad` (PLT-802): a gap point
/// `d1_rad` from the nearest edge sample along the lattice holds the strip from `d1 − s` to `d1`
/// seaward of the plate's outline (the outline lies half a spacing `s` beyond the edge sample,
/// and the point's cell spreads half a spacing either side of it), so the share of it within the
/// margin's width is the strip's overlap with [0, width] over `s`. Summed over the rows of a
/// straight margin the shares give the width at any lattice spacing: on the 16k quick planet
/// (71 km) the first row takes all of its cell and the second a quarter of it for 90 km.
pub fn margin_share(d1_rad: f64, width_rad: f64, spacing_rad: f64) -> f64 {
    if spacing_rad.is_nan() || spacing_rad <= 0.0 {
        return 0.0;
    }
    ((d1_rad.min(width_rad) - (d1_rad - spacing_rad).max(0.0)) / spacing_rad).clamp(0.0, 1.0)
}

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
/// every point not between two plates; so does a point whose band's meridian lies beyond the
/// other plate's edge (where an oblique gap is narrower than its offset across the band, the
/// segment ends inside the band) or more than a quarter turn round the pole from it. Returns, per
/// sample, the plate and α the segments give, and its angular distance from the segment, rad.
fn segment_ridges(
    gap: &[bool],
    best: &[[(f64, u32); 2]],
    plate_new: &[u32],
    pts: &[DVec3],
    omega: &[DVec3],
    norm: &Norm,
    seg_rad: f64,
) -> Vec<Option<(u32, f64, f64)>> {
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
        // More than a quarter turn round the pole from the band's meridian (a band that wraps
        // round the pole of the pair's frame), the distance below is to the opposite meridian.
        if dphi.abs() >= std::f64::consts::FRAC_PI_2 {
            continue;
        }
        let q = if dphi > 0.0 { key.0 } else { key.1 };
        let [(d1, l1), (d2, _)] = best[i];
        let (own, other) = if plate_new[l1 as usize] == q {
            (d1, d2)
        } else {
            (d2, d1)
        };
        // The great-circle distance from the point to the segment's meridian.
        let to_ridge = (theta.sin() * dphi.sin().abs()).min(1.0).asin();
        // The meridian beyond the other plate's edge: the segment does not cross this part of
        // the gap (at a band's end, where an oblique gap is narrower than its offset across the
        // band), so the point keeps the midline rather than going whole to one side.
        if to_ridge > other {
            continue;
        }
        let alpha = (to_ridge / (to_ridge + own).max(1e-12)).clamp(0.0, 1.0);
        out[i] = Some((q, alpha, to_ridge));
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
    // Also for the front trace's attribution (`front_trace.rs`, measurement only).
    let tracing = state.gap_budget.is_some() || super::front_trace::wants_resample();
    let old_limits = state
        .limits
        .as_ref()
        .filter(|l| l.ceiling_m.len() == src.len() && l.top_m.len() == src.len());
    // The outline below the lattice spacing (ledger: PLT-536): every old sample's distance from
    // its group's outline, and the cap on the new samples' distances.
    let spacing = norm.spacing_rad();
    let cap = level_set::CAP_SPACINGS * spacing;
    let old_phi: Option<Vec<f64>> = k.level_set_membership.then(|| {
        let groups: Vec<u32> = src.iter().map(|s| s.group).collect();
        level_set::old_distances(state.level_set.as_ref(), &groups, old, spacing)
    });
    // Held crust (one-way subduction, `subduction::resolve`; none with the switch off): the slab,
    // never on top of the new lattice (ledger: PLT-542).
    let one_way = state.one_way.as_ref();
    let slab = |m: u32| one_way.is_some_and(|ow| ow.holds(m as usize, &src[m as usize]));
    // On top: crust that is not the slab, then visible, then the overriding plate of a subduction
    // contact, then continental, then higher, then the lower group id.
    // Each side is a group and the old sample its crust is judged by (a cover's major corner).
    let rank = |(ga, ma): (u32, u32), (gb, mb): (u32, u32)| {
        let key = |g: u32, m: u32| {
            let held = slab(m);
            let m = &src[m as usize];
            (
                (!held, !m.is_hidden()),
                group_plate[g as usize],
                m.kind == CONTINENTAL,
                m.elevation_m,
                g,
            )
        };
        let (ka, kb) = (key(ga, ma), key(gb, mb));
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
    };
    let (found, mut phi_new): (Vec<Found>, Vec<f32>) = new_pts
        .par_iter()
        .map(|&q| {
            let mut covers: Vec<(u32, Cover)> = Vec::new();
            // With the level set: each cover's distance inside its group's outline, and the
            // groups near `q` that do not hold it, with the old sample their crust is judged by
            // and `q`'s distance outside their outlines.
            let mut inside: Vec<f64> = Vec::new();
            let mut outside: Vec<(u32, u32, f64)> = Vec::new();
            let mut tried: Vec<u32> = Vec::with_capacity(8);
            let near = tree.nearest_n(q, 8);
            for &(j, _) in &near {
                let g = src[j as usize].group;
                if tried.contains(&g) {
                    continue;
                }
                tried.push(g);
                match &old_phi {
                    None => {
                        if let Some(c) = footprint(old, src, g, rots[g as usize], q, j) {
                            covers.push((g, c));
                        }
                    }
                    Some(phi) => {
                        match footprint_level_set(old, src, phi, g, rots[g as usize], q, j) {
                            (Some(c), d) if d >= 0.0 => {
                                covers.push((g, c));
                                inside.push(d);
                            }
                            (Some(c), d) => outside.push((g, c.major(), d)),
                            // No corner of the group in the triangle: judged by its old sample
                            // nearest `q`.
                            (None, d) => outside.push((g, j, d)),
                        }
                    }
                }
            }
            if covers.is_empty() {
                return ((None, covers), 0.0);
            }
            // One-way subduction: where only crust that went down a trench covers the point, the
            // slab took it; the point is a gap and gets new sea floor below (ledger: PLT-542).
            if covers.iter().all(|(_, c)| slab(c.major())) {
                return ((None, covers), 0.0);
            }
            let best = (0..covers.len())
                .max_by(|&a, &b| {
                    rank(
                        (covers[a].0, covers[a].1.major()),
                        (covers[b].0, covers[b].1.major()),
                    )
                })
                .unwrap_or(0);
            // The new sample's distance from its plate's new outline: from the top group's own
            // outline, or from that of a group of another plate that would be on top of it and
            // whose outline `q` lies just outside (a trench: the subducting plate's samples in
            // front of the overriding plate lie the overriding plate's outline away from the
            // boundary).
            let phi_q = if old_phi.is_some() {
                let (tg, tm) = (covers[best].0, covers[best].1.major());
                let tp = group_plate[tg as usize];
                let mut d = inside[best];
                for &(g, m, dg) in &outside {
                    if group_plate[g as usize] != tp
                        && rank((g, m), (tg, tm)) == std::cmp::Ordering::Greater
                    {
                        d = d.min(-dg);
                    }
                }
                d.clamp(0.0, cap) as f32
            } else {
                0.0
            };
            let top = covers.swap_remove(best);
            ((Some(top), covers), phi_q)
        })
        .unzip();

    // Covered samples. Continental crust a collision took across its front was stacked into its
    // belt or went down with the slab (`belts.rs`): it is consumed here, not accreted a second
    // time (ledger: PLT-276, PLT-292). Both are counted, m of rock per new sample.
    let mut plate_new: Vec<u32> = vec![u32::MAX; n];
    // 9 to 11: the floor's sediment reaching a trench, under continental or oceanic crust, or the
    // one-way slab's (measurement only); 12 and 13: the new continental samples whose island-arc
    // class, carried from the nearest corner, the blend of their columns would have lost or given
    // (measurement only, PLT-760: the old reading of the class).
    let mut tally: Vec<[f64; 14]> = vec![[0.0; 14]; n];
    let mature_m = super::subduction::mature_arc_m(k);
    let sediment_share = k.sediment_subduction_share as f32; // ledger: PLT-440
    // By margin type instead of one share (ledger: PLT-660).
    let sourced = k.sediment_subduction_sourced;
    // The accretionary pile's line in rock (ledger: PLT-660, PLT-762).
    let pile_line_m = trench_line_solid_m(k.trench_pile_in_place, ACCRETIONARY_PILE_M);
    // How each new sample was blended, for the gap budget (`gap_budget.rs`) when it is on.
    let mut traces: Vec<Option<gap_budget::Trace>> = vec![None; n];
    // The step's limits (`rate.rs`) on the new lattice: a covered sample's ceiling and top are
    // its corners' blended with its own weights, its floor the blended surface before the
    // accretion (new sea floor's are set with the gaps below).
    let mut limits: Vec<[f32; 3]> = vec![[f32::INFINITY, 0.0, f32::INFINITY]; n];
    // A cover of group `g` interpolated at the new point `q`, its labels from the old sample
    // nearest `q` (ledger: PLT-664).
    let heights = Heights {
        k,
        gravity: env.gravity,
        t_myr: t,
    };
    let interp = |g: u32, c: &Cover, q: DVec3| {
        let rot = rots[g as usize];
        interpolate(src, c, rot, nearest_corner(old, c, rot, q), heights)
    };
    let mut samples: Vec<Sample> = found
        .par_iter()
        .zip(plate_new.par_iter_mut())
        .zip(tally.par_iter_mut())
        .zip(traces.par_iter_mut())
        .zip(limits.par_iter_mut())
        .zip(new_pts.par_iter())
        .map(|((((((top, rest), pl), tally), tr), lim), &q)| match top {
            None => {
                // One-way subduction's gap (the covers are all held crust; none elsewhere): its
                // conserved pools went down with the slab, its sediment counted as subducted
                // sediment too (ledger: PLT-542, PLT-440).
                for (g2, c2) in rest {
                    let u = interp(*g2, c2, q);
                    tally[8] += u.pools[..pool::CONSERVED]
                        .iter()
                        .map(|&x| f64::from(x))
                        .sum::<f64>();
                    let pile = f64::from(u.pools[pool::SEDIMENT].max(0.0))
                        + f64::from(u.pools[pool::SEDIMENTARY].max(0.0));
                    tally[7] += pile;
                    tally[11] += pile;
                }
                Sample::blank(0)
            }
            Some((g, c)) => {
                let mut s = interp(*g, c, q);
                *pl = group_plate[*g as usize];
                if s.kind == CONTINENTAL {
                    let blended = s.basement() > 0.0 && s.arc_oceanic_m >= mature_m;
                    if s.island_arc_marked() && !blended {
                        tally[12] += 1.0;
                    } else if !s.island_arc_marked() && blended {
                        tally[13] += 1.0;
                    }
                }
                if let Some(old) = old_limits {
                    let blend = |v: &[f32]| -> f32 {
                        (0..3)
                            .map(|k| c.w[k] * f64::from(v[c.tri[k] as usize]))
                            .sum::<f64>() as f32
                    };
                    *lim = [blend(&old.ceiling_m), s.elevation_m, blend(&old.top_m)];
                }
                if tracing {
                    *tr = Some(gap_budget::Trace {
                        tri: c.tri,
                        w: c.w,
                        pools: s.pools,
                        z: s.elevation_m,
                    });
                }
                let blended = s.pools;
                for (g2, c2) in rest {
                    let under = &src[c2.major() as usize];
                    let p2 = group_plate[*g2 as usize];
                    if under.is_hidden() || p2 == *pl || overrides(*pl, p2) {
                        let mut u = interp(*g2, c2, q);
                        let conserved: f64 = u.pools[..pool::CONSERVED]
                            .iter()
                            .map(|&x| f64::from(x))
                            .sum();
                        // Crust a collision took across its front is consumed, hidden or
                        // not: the belt stacked its share and the rest went down with the slab.
                        if (under.is_hidden() && under.stacked())
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
                            // accretes. ledger: PLT-440; with `sediment_subduction_sourced`
                            // the share follows the margin's type (PLT-660).
                            let mut subducted = 0.0_f64;
                            let going_under = under.kind != CONTINENTAL
                                && (under.is_hidden() || (p2 != *pl && overrides(*pl, p2)));
                            if going_under {
                                // Measurement only: what reaches the trench, before the share.
                                let pile = f64::from(u.pools[pool::SEDIMENT].max(0.0))
                                    + f64::from(u.pools[pool::SEDIMENTARY].max(0.0));
                                let slot = if s.kind == CONTINENTAL { 9 } else { 10 };
                                tally[slot] += pile;
                            }
                            if (sourced || sediment_share > 0.0) && going_under {
                                let share = if sourced {
                                    let pile = u.pools[pool::SEDIMENT].max(0.0)
                                        + u.pools[pool::SEDIMENTARY].max(0.0);
                                    sediment_subducted_share_at(
                                        f64::from(pile),
                                        super::subduction::orthogonal_convergence_mm_yr(&s),
                                        pile_line_m,
                                    ) as f32
                                } else {
                                    sediment_share
                                };
                                for kp in [pool::SEDIMENT, pool::SEDIMENTARY] {
                                    let down = u.pools[kp].max(0.0) * share;
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
                        let u = interp(*g2, c2, q);
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
    if sourced || sediment_share > 0.0 {
        *world
            .globals
            .entry(SEDIMENT_SUBDUCTED_GLOBAL.to_owned())
            .or_insert(0.0) += sum(7);
    }
    if k.one_way_subduction {
        *world.globals.entry(SLAB_GLOBAL.to_owned()).or_insert(0.0) += sum(8);
    }
    for (name, v) in [
        (SEDIMENT_REACHING_TRENCH_GLOBAL, sum(9) + sum(10) + sum(11)),
        (SEDIMENT_REACHING_TRENCH_CONTINENTAL_GLOBAL, sum(9)),
        (SEDIMENT_REACHING_TRENCH_OCEANIC_GLOBAL, sum(10)),
        (ISLAND_CLASS_KEPT_GLOBAL, sum(12) * 1000.0),
        (ISLAND_CLASS_REFUSED_GLOBAL, sum(13) * 1000.0),
    ] {
        *world.globals.entry(name.to_owned()).or_insert(0.0) += v;
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
    // Volcanic rifted margins (PLT-799): the plume heads' hot mantle, and when each pair of
    // plates began to part (its rift's onset; the gap beside a continent opened after the last
    // resampling, which filled any older one).
    let heads: Vec<HotMantle> = if k.volcanic_margins {
        super::hotspots::hot_mantle(state, env)
    } else {
        Vec::new()
    };
    let opened: BTreeMap<(u32, u32), f64> = if heads.is_empty() {
        BTreeMap::new()
    } else {
        let ids: BTreeMap<EventId, (u32, u32)> = state
            .rift_links
            .iter()
            .map(|l| (l.event, (l.a, l.b)))
            .collect();
        world
            .events
            .iter()
            .filter_map(|e| ids.get(&e.id).map(|&ab| (ab, e.t_start / 1.0e6)))
            .collect()
    };
    let margin_rad = norm.rad(super::hotspots::sourced::MARGIN_WIDTH_KM);
    let (mut margin_m, mut margin_n) = (0.0f64, 0u64);
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
        // The new ridge is the outline between the gap's two plates (ledger: PLT-536): a gap
        // point's distance from it is half the difference of its distances from the two plates'
        // edges on the midline, or its distance from the segment.
        phi_new[i] = if l2 != u32::MAX {
            (0.5 * (d2 - d1)).clamp(0.0, cap) as f32
        } else {
            cap as f32
        };
        if let Some((q, a, to_ridge)) = segment.as_ref().and_then(|s| s[i]) {
            if q != p1 {
                p2 = Some(p1);
                p1 = q;
            }
            alpha = a;
            phi_new[i] = to_ridge.sin().clamp(0.0, cap) as f32;
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
        s.thickness_m = thickness(&s.pools);
        s.heat_flow_mw_m2 = ocean_heat_flow(age) as f32;
        // A volcanic rifted margin (PLT-799): new sea floor of a continent's plate whose cell
        // reaches into the `MARGIN_WIDTH_KM` strip seaward of its continental outline (PLT-802,
        // over its cell's share of it), formed over a plume head's hot mantle (White & McKenzie
        // 1989), is the igneous crust that mantle melts to, continental crust over its ocean
        // basement (PLT-331). It formed when the pair began to part or later (its
        // age, α × the time since the last resampling, at most that), where its plate's motion
        // brings it back to over that time.
        let share = margin_share(d1, margin_rad, norm.spacing_rad());
        if !heads.is_empty()
            && share > 0.0
            && plate_new[l1 as usize] == p1
            && samples[l1 as usize].kind == CONTINENTAL
        {
            let t_open = p2
                .and_then(|q| opened.get(&(p1.min(q), p1.max(q))))
                .copied()
                .unwrap_or(f64::NEG_INFINITY)
                .max(state.resample_myr);
            let t_f = (t - age).max(t_open).min(t);
            let back = super::kinematics::rotation(omega[p1 as usize], -(t - t_f) * 1.0e6);
            if let Some(column) = hot_mantle_at(&heads, back * p, t_f)
                .and_then(|h| volcanic_margin_column(h.excess_k, share))
            {
                let added = thickness(&column) - thickness(&s.pools);
                s.pools = column;
                s.thickness_m = thickness(&s.pools);
                s.make_continental();
                s.regime = regime::PASSIVE_MARGIN;
                // Its own crust stands on the new floor by Airy, under water (settle.rs takes
                // the column from there, ledger ISO-078).
                let wet = if s.elevation_m < 0.0 {
                    super::settle::WATER_LOADED
                } else {
                    1.0
                };
                s.elevation_m = (f64::from(s.elevation_m)
                    + f64::from(added) / super::pools::COLUMN_ISOSTATIC_FACTOR * wet)
                    .min(k.max_elevation_m) as f32;
                margin_m += f64::from(s.own_thickness_m());
                margin_n += 1;
            }
        }
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
    if k.volcanic_margins {
        // ledger: PLT-799 (measurement)
        for (name, v) in [
            (
                VOLCANIC_MARGIN_GLOBAL,
                margin_m * norm.sample_area_km2() / 1000.0,
            ),
            (VOLCANIC_MARGIN_SAMPLES_GLOBAL, margin_n as f64),
        ] {
            *world.globals.entry(name.to_owned()).or_insert(0.0) += v;
        }
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
        phi_new[i] = (level_set::STALE_SPACINGS * spacing) as f32;
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
    // The held crust is consumed or under new crust, and the old lattice's forearc is the next
    // step's to find: the new lattice starts with neither.
    if let Some(ow) = state.one_way.as_mut() {
        ow.held.clear();
        ow.forearc.clear();
    }
    state.limits = new_limits;
    if super::front_trace::wants_resample() {
        // Gap points are new sea floor (as for the gap budget below).
        super::front_trace::stash_resample(
            traces
                .iter()
                .zip(&gap)
                .map(|(t, &g)| if g { None } else { *t })
                .collect(),
        );
    }
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
    if old_phi.is_some() {
        // The new samples' distances, until `level_set::reinit` sets them from the outline the
        // plates have once `bury` is done (ledger: PLT-536).
        state.level_set = Some(level_set::LevelSet {
            phi: phi_new,
            member: state.samples.iter().map(|s| s.group).collect(),
        });
    }
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

/// World globals (the front lane, measurement; PLT-532, only while `plate_fission` is on): plates
/// born of a plate's cut-off pieces at resamplings …
pub const FISSION_GLOBAL: &str = "plates/fission_births";
/// … the samples they took …
pub const FISSION_SAMPLES_GLOBAL: &str = "plates/fission_samples";
/// … and the samples of smaller cut-off pieces the plate around them took (capture).
pub const CAPTURED_SAMPLES_GLOBAL: &str = "plates/captured_samples";

/// A plate is one rigid body (`tectonics.plate_fission`, the front lane, 2026-10-24; ledger
/// PLT-532). Right after a resampling (`geo` the lattice every group shares, each group
/// unrotated), each plate's visible crust that the step's overlaps and the resampling's
/// footprints have cut into pieces meeting only through other plates (`geometry::pieces`) keeps
/// its largest piece as the plate. Every other piece of `min_plate_area_frac` of the planet
/// (PLT-091) or more becomes a plate of its own, with the parent's rotation (fission: Stock & Lee
/// 1994, "if a small fragment breaks off a large plate, then it may lose its slab-pull force and
/// become driven entirely by different conditions"; the Farallon plate's, Lonsdale 2005): the
/// force balance moves it from the next step. Every smaller piece joins the plate most of its
/// lattice neighbours belong to (capture: Stock & Lee 1994, microplates that "became attached to
/// the Pacific plate"; Nicholson et al. 1994), as a plate below that size is absorbed below.
/// Returns (plates born, samples to them, samples captured).
fn split_cut_plates(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
) -> Result<(u32, usize, usize), CoreError> {
    let n = state.samples.len();
    let t = env.t_myr;
    let visible: Vec<bool> = state.samples.iter().map(|s| !s.is_hidden()).collect();
    let mut label: Vec<u32> = (0..n).map(|i| state.plate_of(i)).collect();
    let (comp, sizes, keep) = pieces(geo, &label, |i| visible[i]);
    let min = (env.k.min_plate_area_frac * f64::from(state.n)).ceil() as usize; // ledger: PLT-091
    // The pieces that become plates, largest first (ties by piece).
    let mut big: Vec<u32> = (0..sizes.len() as u32)
        .filter(|&c| {
            let (p, size) = sizes[c as usize];
            keep[&p].1 != c && size >= min
        })
        .collect();
    big.sort_by_key(|&c| (std::cmp::Reverse(sizes[c as usize].1), c));
    let first = state.plates.len() as u32;
    let new_of: BTreeMap<u32, u32> = big
        .iter()
        .enumerate()
        .map(|(k, &c)| (c, first + k as u32))
        .collect();
    for (i, l) in label.iter_mut().enumerate() {
        if let Some(&np) = new_of.get(&comp[i]) {
            *l = np;
        }
    }
    // The rest of the cut-off pieces go to the plate around them.
    let captured = connect_plates(geo, &mut label, |i| visible[i]);
    // New plates: the parent's motion, one unrotated group each.
    for &c in &big {
        let parent = state.plates[sizes[c as usize].0 as usize].clone();
        let id = new_of[&c];
        debug_assert_eq!(id as usize, state.plates.len());
        state.plates.push(Plate {
            id,
            alive: true,
            omega: parent.omega,
            born_myr: t,
            died_myr: None,
            birth_event: None,
            parent: Some(parent.id),
            lip_until_myr: f64::NEG_INFINITY,
            lip_event: None,
            samples: 0,
            centroid: parent.centroid,
            continental_frac: 0.0,
        });
    }
    let mut group_of: BTreeMap<u32, u32> = BTreeMap::new();
    for (g, grp) in state.groups.iter().enumerate() {
        group_of.entry(grp.plate).or_insert(g as u32);
    }
    let mut moved = 0usize;
    for (i, &l) in label.iter().enumerate() {
        if l == state.plate_of(i) {
            continue;
        }
        let rot = state.groups[state.samples[i].group as usize].rot;
        let g = *group_of.entry(l).or_insert_with(|| {
            state.groups.push(Group { plate: l, rot });
            (state.groups.len() - 1) as u32
        });
        state.samples[i].group = g;
        if l >= first {
            moved += 1;
        }
    }
    if big.is_empty() {
        return Ok((0, 0, captured));
    }
    state.recount(pos);
    // The new plates take the boundaries they border (ledger: PLT-534).
    if env.k.split_inheritance {
        let mut splits: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for &c in &big {
            splits
                .entry(sizes[c as usize].0)
                .or_default()
                .push(new_of[&c]);
        }
        let splits: Vec<(u32, Vec<u32>)> = splits.into_iter().collect();
        super::inherit::inherit(state, pos, env, &splits).book(world);
    }
    let norm = &env.norm;
    for &c in &big {
        let id = new_of[&c];
        let p = &state.plates[id as usize];
        let (parent, count, centroid) = (sizes[c as usize].0, p.samples, p.centroid);
        let at = DVec3::from_array(centroid);
        let area = f64::from(count) * norm.sample_area_km2();
        let e = events::event(world, t, PLATE_BIRTH, at)
            .magnitude((area / state.a0_km2) as f32)
            .param("plate", i64::from(id))
            .param("parent", i64::from(parent))
            .param("area_km2", norm.planet_area_km2(area))
            .param("continental_frac", p.continental_frac)
            .param(
                "speed_mm_yr",
                norm.speed_mm_yr(DVec3::from_array(p.omega), at),
            )
            .param("origin", "fission");
        let birth = world.emit(e)?;
        state.plates[id as usize].birth_event = Some(birth);
        state.counters.plate_births += 1;
    }
    Ok((big.len() as u32, moved, captured))
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
        if env.k.plate_fission {
            let (born, moved, captured) = split_cut_plates(state, world, geo, pos, env)?;
            for (key, x) in [
                (FISSION_GLOBAL, f64::from(born)),
                (FISSION_SAMPLES_GLOBAL, moved as f64),
                (CAPTURED_SAMPLES_GLOBAL, captured as f64),
            ] {
                *world.globals.entry(key.to_owned()).or_insert(0.0) += x;
            }
            state.recount(pos);
        }
        let min = (env.k.min_plate_area_frac * f64::from(state.n)).ceil() as u32;
        // A back-arc sliver holding a subducting front is kept below the size (Bird 2003's
        // back-arc microplates, 0.0002 to 0.016 sr; ledger: PLT-715).
        let backarcs = state.backarcs.clone();
        while let Some(small) = state
            .alive()
            .filter(|p| p.samples > 0 && p.samples < min)
            .filter(|p| !backarcs.iter().any(|b| b.plate == p.id))
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

    /// The height follows the rock (PLT-850): ocean floor at its age's floor, interpolated
    /// beside a continental corner 2 km over the sea and an island arc's thick crust, stands
    /// where its own rock and age hold it (`rock_height_m`), not a share of the others' height;
    /// resampled again and again it does not rise. Until 2026-10-31 the height was blended over
    /// all three corners: the bare floor took 30 % of the continent's height at each resampling,
    /// and kept it (the floor's law only sinks it by its age's increments). A floor a trench pulled
    /// 1 km under its law keeps that departure when its own corners carry it.
    #[test]
    fn a_resampled_floor_stands_on_its_own_rock() {
        let k = &crate::params::TectonicParams::default();
        let t = 500.0;
        let h = Heights {
            k,
            gravity: 9.81,
            t_myr: t,
        };
        let floor = |age: f64| {
            let mut s = Sample::blank(0);
            s.pools = oceanic_column();
            s.thickness_m = thickness(&s.pools);
            s.formed_myr = (t - age) as f32;
            s.elevation_m = rock_height_m(&s, k, 9.81, t) as f32;
            s
        };
        let mut continent = Sample::blank(0);
        continent.kind = CONTINENTAL;
        continent.pools = super::super::pools::continental_column();
        continent.thickness_m = thickness(&continent.pools);
        continent.elevation_m = 2000.0;
        let mut arc = floor(60.0);
        arc.pools[pool::MAFIC_PLUTONIC] += 15_000.0;
        arc.thickness_m = thickness(&arc.pools);
        arc.elevation_m = -500.0;
        let cover = Cover {
            t: 0,
            tri: [0, 1, 2],
            w: [0.4, 0.3, 0.3],
        };
        let old = floor(80.0);
        let own = rock_height_m(&old, k, 9.81, t);
        // Beside a continent: the floor's own height to rounding.
        let src = [old, continent, floor(80.0)];
        let s = interpolate(&src, &cover, DQuat::IDENTITY, 0, h);
        assert_eq!(s.kind, OCEANIC);
        assert!(
            (f64::from(s.elevation_m) - own).abs() < 1.0,
            "{} against its rock's {own}",
            s.elevation_m
        );
        // Beside an arc's thick ocean crust (the same class): the arc's departure from its own
        // rock is blended, not its height.
        let src = [old, floor(80.0), arc];
        let s = interpolate(&src, &cover, DQuat::IDENTITY, 0, h);
        let arc_departure = f64::from(arc.elevation_m) - rock_height_m(&arc, k, 9.81, t);
        let want = rock_height_m(&s, k, 9.81, t) + 0.3 * arc_departure;
        assert!(
            (f64::from(s.elevation_m) - want).abs() < 1.0,
            "{} against {want}",
            s.elevation_m
        );
        // Resampled ten times beside the continent: no lift builds up.
        let mut z = old;
        for _ in 0..10 {
            let src = [z, continent, z];
            z = interpolate(&src, &cover, DQuat::IDENTITY, 0, h);
        }
        assert!(
            (f64::from(z.elevation_m) - own).abs() < 1.0,
            "{} after ten resamplings, its rock's {own}",
            z.elevation_m
        );
        // A trench's pull, 1 km under the law at all three corners, is kept.
        let mut pulled = old;
        pulled.elevation_m = (own - 1000.0) as f32;
        let src = [pulled, pulled, pulled];
        let s = interpolate(&src, &cover, DQuat::IDENTITY, 0, h);
        assert!((f64::from(s.elevation_m) - (own - 1000.0)).abs() < 1.0);
    }

    /// PLT-760 through a resampling (PR #53's review): the island-arc class of continental crust
    /// is the nearest corner's, as the kind is, whatever the blend makes of the columns' island-arc
    /// crust and basement. Leak B: a matured arc's edge column (8.5 km of island-arc crust over its
    /// ocean basement) beside its ocean-floor flank keeps its kind and full basement; its arc crust
    /// blended to under the 8 km line while the record was blended over all three corners, and
    /// since 2026-10-31 it follows its class's rock (PLT-851), the edge's own 8.5 km. Leak A: a
    /// continent's own column beside an accreted arc (40 km of island-arc crust) takes a share of
    /// its basement and enough arc crust to pass the line at a 0.2 weight.
    #[test]
    fn the_island_arc_class_survives_a_resampling_from_the_nearest_corner() {
        let k = &crate::params::TectonicParams::default();
        let mature = super::super::subduction::mature_arc_m(k);
        let h = Heights {
            k,
            gravity: 9.81,
            t_myr: 0.0,
        };
        let arc = |crust_m: f32| {
            let mut s = Sample::blank(0);
            s.pools = oceanic_column();
            s.arc_oceanic_m = crust_m;
            s.make_continental();
            s.set_island_arc(true);
            s.pools[pool::FELSIC_PLUTONIC] += crust_m;
            s.thickness_m = thickness(&s.pools);
            s
        };
        let mut floor = Sample::blank(0);
        floor.pools = oceanic_column();
        floor.thickness_m = thickness(&floor.pools);
        let mut continent = Sample::blank(0);
        continent.kind = CONTINENTAL;
        continent.pools = super::super::pools::continental_column();
        continent.thickness_m = thickness(&continent.pools);
        let island = super::super::subduction::island_arc_crust;
        let cover = |w: [f64; 3]| Cover {
            t: 0,
            tri: [0, 1, 2],
            w,
        };
        let edge = arc(8500.0);
        assert!(island(&edge) && !island(&continent) && island(&floor));
        // Leak B, edge weights 0.4 to 0.8.
        for w0 in [0.4, 0.6, 0.8] {
            let w1 = 0.5 * (1.0 - w0);
            let src = [edge, floor, floor];
            let s = interpolate(&src, &cover([w0, w1, w1]), DQuat::IDENTITY, 0, h);
            assert_eq!(s.kind, CONTINENTAL);
            assert!(
                s.basement() > 0.0 && (s.arc_oceanic_m - 8500.0).abs() < 1.0,
                "{w0}: {s:?}"
            );
            assert!(island(&s), "the matured arc's edge lost its class at {w0}");
        }
        // Leak A.
        let src = [continent, arc(40_000.0), continent];
        let s = interpolate(&src, &cover([0.6, 0.2, 0.2]), DQuat::IDENTITY, 0, h);
        assert_eq!(s.kind, CONTINENTAL);
        assert!(s.basement() > 0.0 && s.arc_oceanic_m >= mature, "{s:?}");
        assert!(!island(&s), "the continent took the accreted arc's class");
        // Ocean floor again is no island arc's continental crust.
        let mut back = edge;
        back.make_oceanic();
        assert!(!back.island_arc_marked());
    }

    /// PLT-660, Clift & Vannucchi (2004): an accretionary margin (a pile over 1 km, convergence
    /// under 76 mm/yr) subducts 83 % of the incoming pile; an erosive one (a thinner pile or a
    /// faster convergence) all of it (von Huene & Scholl 1991's type 2). The bounds are strict,
    /// as the sources word them, and the convergence's sign does not enter.
    #[test]
    fn sediment_goes_down_by_the_margins_type() {
        for (pile, conv, want) in [
            (1500.0, 50.0, 0.83),
            (8000.0, 10.0, 0.83),
            (1500.0, -50.0, 0.83),
            (1500.0, 90.0, 1.0),
            (1500.0, 76.0, 1.0),
            (1000.0, 50.0, 1.0),
            (300.0, 20.0, 1.0),
            (0.0, 0.0, 1.0),
        ] {
            assert_eq!(
                sediment_subducted_share(pile, conv),
                want,
                "{pile} m at {conv} mm/yr"
            );
        }
    }

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

    /// A gap `2 half_deg` wide and 35° oblique to the spreading (Earth's envelope, plates.md
    /// "Earth's ridges and transforms") between plate a, turning about ẑ, and plate b, still;
    /// points `step` apart in the pair's frame, the gap points first and one sample of each plate
    /// last.
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
            .into_iter()
            .map(|o| o.map(|(q, a, _)| (q, a)))
            .collect()
        }
    }

    fn oblique_gap(omega_a: DVec3, half_deg: f64) -> Gap {
        use std::f64::consts::FRAC_PI_2;
        let e = DVec3::Z;
        let f = e.any_orthogonal_vector().normalize();
        let g = e.cross(f);
        let at =
            |theta: f64, phi: f64| (f * phi.cos() + g * phi.sin()) * theta.sin() + e * theta.cos();
        let (half, step) = (half_deg.to_radians(), 0.2f64.to_radians());
        let columns = (2.0 * half_deg / 0.2).round() as i32;
        let mid = |theta: f64| 35f64.to_radians().tan() * (theta - FRAC_PI_2);
        let mut grid = Vec::new();
        for r in -50..=50 {
            let theta = FRAC_PI_2 + f64::from(r) * step;
            for c in 0..columns {
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
        let gap = oblique_gap(spread, 3.0);
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
        let back = oblique_gap(-spread, 3.0).segments(&norm, seg);
        assert!(
            back.iter().all(Option::is_none),
            "a converging pair keeps the midline"
        );
        assert!(gap.segments(&norm, 0.0).iter().all(Option::is_none));
    }

    /// The same envelope with a gap 2° wide, narrower than its offset across a 4° band (tan 35° of
    /// 4°, 2.8°): the band's meridian leaves the gap at the band's ends. A segment never hands a
    /// point to one side with the meridian beyond the other plate's edge (a row of the gap whole to
    /// one plate, with no ridge in it); those rows keep the midline, the rest take the segment.
    /// ledger: PLT-502
    #[test]
    fn a_segment_ends_where_its_meridian_leaves_the_gap() {
        let norm = Norm::new(6370.0, 782.0, 100_000);
        let gap = oblique_gap(DVec3::Z * norm.omega_of_speed(50.0), 1.0);
        let m = gap.gap.iter().filter(|&&g| g).count();
        let seg = 4f64.to_radians();
        let out = gap.segments(&norm, seg);
        let e = DVec3::Z;
        let f = e.any_orthogonal_vector().normalize();
        let g = e.cross(f);
        let offset = band_offset(0, 1);
        let frame = |p: DVec3| {
            let theta = p.dot(e).clamp(-1.0, 1.0).acos();
            (theta, p.dot(g).atan2(p.dot(f)))
        };
        // Each band's meridian as `segment_ridges` puts it: the circular mean of its points' φ.
        let mut sums: BTreeMap<i64, (f64, f64)> = BTreeMap::new();
        for p in &gap.pts[..m] {
            let (theta, phi) = frame(*p);
            let s = sums
                .entry((theta / seg + offset).floor() as i64)
                .or_default();
            s.0 += phi.cos();
            s.1 += phi.sin();
        }
        let (mut on, mut off) = (0, 0);
        let mut rows: BTreeMap<i64, [usize; 2]> = BTreeMap::new();
        for (i, p) in gap.pts[..m].iter().enumerate() {
            let (theta, phi) = frame(*p);
            let row = (theta.to_degrees() * 5.0).round() as i64;
            let Some((q, _)) = out[i] else {
                off += 1;
                continue;
            };
            on += 1;
            rows.entry(row).or_default()[q as usize] += 1;
            let (c, s) = sums[&((theta / seg + offset).floor() as i64)];
            let dphi = wrap(phi - s.atan2(c));
            let to_ridge = (theta.sin() * dphi.sin().abs()).asin();
            let [(d1, l1), (d2, _)] = gap.best[i];
            let other = if gap.plate_new[l1 as usize] == q {
                d2
            } else {
                d1
            };
            assert!(
                to_ridge <= other + 1e-12,
                "point {i}: the meridian {to_ridge} rad off, the other plate's edge {other}"
            );
        }
        assert!(off > 0, "some rows lie beyond their band's meridian");
        assert!(on > m / 2, "{on} of {m} points on a segment");
        // Most rows on a segment hold the ridge between both plates' points.
        let both = rows.values().filter(|c| c[0] > 0 && c[1] > 0).count();
        assert!(both * 4 >= rows.len() * 3, "{both} of {} rows", rows.len());
    }

    /// A plate is one rigid body (Gordon 1998: "The central assumption of plate tectonics is that
    /// the plates are rigid"; Bird 2003 [123]: PB2002's 52 plates are 52 segments, each "a
    /// closed curve outlining that plate"; Stock & Lee 1994: a fragment broken off a plate "may
    /// lose its slab-pull force and become driven entirely by different conditions", and
    /// microplates "became attached to the Pacific plate"). Plate 0 is cut in two by a band of
    /// plate 1 (0 < z < 0.3) and keeps one small patch inside the band. With `plate_fission` its
    /// southern piece (the larger) stays plate 0, its northern piece becomes plate 2 with plate
    /// 0's rotation and parent, and the patch joins plate 1; every plate is then one connected
    /// piece and no sample moved. Off, plate 0 keeps all three pieces. ledger: PLT-532
    #[test]
    fn a_plate_cut_in_two_splits_and_a_cut_off_patch_is_captured() {
        use crate::params::TectonicParams;
        use crate::plates::geometry::{cut_samples, positions};
        use crate::plates::rifting::tests::{N, continental, one_plate};
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let centre = DVec3::new(1.0, 0.0, 0.15).normalize();
        let patch = |p: DVec3| p.dot(centre) > 0.06f64.cos();
        let band = |p: DVec3| p.z > 0.0 && p.z < 0.3 && !patch(p);
        let omega = [0.0, 1.0e-3, 2.0e-3];
        let build = || {
            let mut st = one_plate(&geo, 3, |_| continental());
            st.plates[0].omega = omega;
            let mut p1 = st.plates[0].clone();
            p1.id = 1;
            p1.omega = [0.0; 3];
            st.plates.push(p1);
            st.groups.push(Group {
                plate: 1,
                rot: [0.0, 0.0, 0.0, 1.0],
            });
            let pts = geo.points();
            for (i, s) in st.samples.iter_mut().enumerate() {
                s.group = u32::from(band(pts[i]));
            }
            let pos = positions(&st, &geo);
            st.recount(&pos);
            (st, pos)
        };
        let run = |fission: bool| {
            let k = TectonicParams {
                plate_fission: fission,
                ..Default::default()
            };
            let env = Env {
                k: &k,
                norm: Norm::new(6370.0, 6371.0, N),
                dt_myr: 2.0,
                dt_yr: 2.0e6,
                gravity: 9.81,
                step: 1,
                t_myr: 50.0,
            };
            let (mut st, pos) = build();
            let mut world = WorldState::new(1, 16_384, 0.0);
            bury(
                &mut st,
                &mut world,
                Some(&geo),
                &pos,
                &env,
                &BTreeMap::new(),
            )
            .unwrap();
            assert_eq!(positions(&st, &geo), pos, "no sample moved");
            (st, world)
        };
        let pts = geo.points();
        let patch_n = (0..N).filter(|&i| patch(pts[i])).count();
        assert!(
            patch_n > 3
                && (patch_n as f64) < TectonicParams::default().min_plate_area_frac * N as f64,
            "the patch is below the smallest plate: {patch_n}"
        );
        let labels = |st: &PlateState| (0..N).map(|i| st.plate_of(i)).collect::<Vec<u32>>();

        let (off, _) = run(false);
        assert_eq!(off.plates.len(), 2);
        let cut = cut_samples(&geo, &labels(&off), |_| true);
        let north = (0..N).filter(|&i| pts[i].z >= 0.3).count();
        assert_eq!(cut, north + patch_n, "off, plate 0 keeps its pieces");

        let (on, world) = run(true);
        assert_eq!(on.plates.len(), 3, "one plate born");
        let born = &on.plates[2];
        assert!(born.alive);
        assert_eq!(born.parent, Some(0));
        assert_eq!(born.omega, omega, "the parent's rotation");
        assert_eq!(born.born_myr, 50.0);
        assert_eq!(born.samples as usize, north);
        for (i, &q) in pts.iter().enumerate() {
            let want = if patch(q) || band(q) {
                1
            } else if q.z >= 0.3 {
                2
            } else {
                0
            };
            assert_eq!(on.plate_of(i), want, "sample {i} at {q}");
        }
        assert_eq!(cut_samples(&geo, &labels(&on), |_| true), 0);
        let births: Vec<_> = world
            .events
            .iter()
            .filter(|e| e.kind == PLATE_BIRTH)
            .collect();
        assert_eq!(births.len(), 1);
        assert!(matches!(
            births[0].params.get("origin"),
            Some(deeptime_format::ParamValue::Str(o)) if o == "fission"
        ));
        assert!(born.birth_event.is_some());
        assert_eq!(world.globals[FISSION_GLOBAL], 1.0);
        assert_eq!(world.globals[FISSION_SAMPLES_GLOBAL], north as f64);
        assert_eq!(world.globals[CAPTURED_SAMPLES_GLOBAL], patch_n as f64);
        assert_eq!(on.counters.plate_births, 1);
        assert!(on.groups.iter().all(|g| g.rot == [0.0, 0.0, 0.0, 1.0]));
    }

    /// A fission hands the cut-off piece the boundaries it borders (Stock & Lee 1994: "The
    /// fragmentation of a subducting plate creates a new triple junction, two limbs of which are
    /// along the trench"). Plate 0 is cut by a band of plate 1 (0 < z < 0.3) into its southern
    /// half and a northern ring (0.3 ≤ z ≤ 0.9) around plate 2's polar cap; it subducts under
    /// both, each contact in a collision with a slab, the front with plate 1 on the band's
    /// southern edge. With `plate_fission` and `split_inheritance` the ring (plate 3) takes the
    /// trench under plate 2 whole, and both pieces keep subducting under plate 1 with the
    /// collision staying with the southern half at its front. With fission alone every state
    /// stays with plate 0. ledger: PLT-534
    #[test]
    fn a_fission_hands_the_cut_off_piece_the_trench_it_borders() {
        use crate::params::TectonicParams;
        use crate::plates::geometry::positions;
        use crate::plates::inherit::tests::{
            add_plate, assert_inherited, collision_at, contact, find, slab_of,
        };
        use crate::plates::rifting::tests::{N, continental, one_plate};
        use crate::plates::state::SlabTrack;
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let front1 = DVec3::new(1.0, 0.0, 0.0);
        let front2 = DVec3::new(0.0, 0.43, 0.9).normalize();
        let run = |inherit: bool| {
            let k = TectonicParams {
                plate_fission: true,
                split_inheritance: inherit,
                ..Default::default()
            };
            let env = Env {
                k: &k,
                norm: Norm::new(6370.0, 6371.0, N),
                dt_myr: 2.0,
                dt_yr: 2.0e6,
                gravity: 9.81,
                step: 1,
                t_myr: 50.0,
            };
            let mut st = one_plate(&geo, 3, |_| continental());
            add_plate(&mut st, &geo, 1, |p| p.z > 0.0 && p.z < 0.3);
            add_plate(&mut st, &geo, 2, |p| p.z > 0.9);
            for (other, front, km) in [(1, front1, 300.0), (2, front2, 200.0)] {
                let mut c = contact(0, other, front);
                c.collision = Some(collision_at(front));
                st.contacts.push(c);
                st.slabs.push(SlabTrack {
                    under: 0,
                    over: other,
                    km,
                    ..SlabTrack::default()
                });
            }
            let pos = positions(&st, &geo);
            let mut world = WorldState::new(1, 16_384, 0.0);
            bury(
                &mut st,
                &mut world,
                Some(&geo),
                &pos,
                &env,
                &BTreeMap::new(),
            )
            .unwrap();
            assert_eq!(st.plates.len(), 4, "the ring became a plate");
            assert_eq!(st.plates[3].parent, Some(0));
            (st, pos, world)
        };
        let k = TectonicParams::default();
        let env = Env {
            k: &k,
            norm: Norm::new(6370.0, 6371.0, N),
            dt_myr: 2.0,
            dt_yr: 2.0e6,
            gravity: 9.81,
            step: 1,
            t_myr: 50.0,
        };
        let (on, pos, world) = run(true);
        let ring = find(&on, 3, 2).expect("the ring takes the trench under plate 2");
        assert!(ring.collision.is_some());
        assert_eq!(slab_of(&on, 3, 2).map(|s| s.km), Some(200.0));
        assert!(find(&on, 0, 2).is_none() && slab_of(&on, 0, 2).is_none());
        // Under plate 1, without the ring's collision.
        let mut under1 = on.clone();
        under1.contacts.retain(|c| c.over == 1);
        let both = assert_inherited(&under1, &pos, &env, &[0, 3], 1, front1, 300.0);
        assert_eq!(both, vec![0, 3]);
        assert_eq!(
            world.globals[crate::plates::inherit::COLLISIONS_GLOBAL],
            1.0
        );

        let (off, ..) = run(false);
        let pairs: Vec<(u32, u32)> = off.contacts.iter().map(|c| (c.under, c.over)).collect();
        assert_eq!(pairs, vec![(0, 1), (0, 2)]);
        assert!(off.contacts.iter().all(|c| c.collision.is_some()));
        assert_eq!(off.slabs.len(), 2);
    }
}
