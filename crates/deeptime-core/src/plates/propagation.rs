// SPDX-License-Identifier: LGPL-3.0-or-later
//! Law 2 of the floor-age lane: a subduction zone propagates along a passive margin
//! (`tectonics.margin_propagation`, ledger PLT-656; docs/science/plates.md, "Law 2").
//!
//! Zhou, Li, Gerya & Stern 2020: "With increasing slab pull, the subduction zone propagates
//! laterally", and "slab pull is partially applied to bend the adjacent region"; Govers & Wortel
//! 2005: the end of a subducting slab tears the plate it hangs from (a STEP fault). Where a
//! plate U subducts at a trench that reaches U's own continental margin, the floor between the
//! trench and the margin is pulled by U's slab across the margin. When that pull beats the
//! margin's strength, the margin breaks: the continent C and the floor beyond a tear from the end
//! of the broken stretch to U's next boundary become a new plate K', and U, which keeps the slab
//! that pulls, converges on K' along the broken stretch. Each further critical convergence there
//! (`forces::INCIPIENT_SLAB_KM`, 100 km), under the same force test with U's slab beside the tip
//! (the new margin's own, or U's slab under the plate that took U's floor at the tip down),
//! breaks the next stretch of K''s margin floor and moves it to U.
//!
//! # The force test
//! The pull across the margin ([`forces::margin_pull_n_m`]) is the slab pull per metre of the
//! slab that hangs beside the stretch (FRC-002, FRC-004: its length, its floor's age) times the
//! cosine between the direction it pulls (across U's trench, towards the plate over the slab)
//! and the inward normal of the stretch to break. The strength
//! ([`forces::margin_strength_n_m`]) is the fallback of PLT-658: PLT-517's 3 × 10¹² N/m scaled
//! with the floor's elastic thickness by law 3 (PLT-657). No source read gives a verified
//! integrated strength of a passive margin, and under this one the propagation does not depend
//! on the floor's age the way Zhou et al.'s does.
//!
//! # The lattice
//! Everything runs on the L0 lattice: "near" is [`BORDER_SPACINGS`] lattice spacings (PLT-534);
//! the coast floor is a plate's visible ocean floor with a lattice neighbour of the plate's own
//! visible continental crust that near; the tear is a great circle from the end of the stretch
//! in the direction away from the continent, marched in half spacings to the first sample of
//! another plate, and a lattice edge whose ends lie on its two sides within its length is cut.
//! A step moves one lattice spacing of margin, so the propagation's speed is set by the
//! critical convergence and the lattice (about one spacing per 100 km of convergence), not
//! resolved below it (PLT-659).

use std::collections::BTreeMap;
use std::f64::consts::PI;

use glam::{DQuat, DVec3};

use super::Env;
use super::events::{self, PLATE_BIRTH};
use super::forces::{self, INCIPIENT_SLAB_KM, SLAB_BUOYANT_MYR};
use super::geometry::{Geo, Tree, connect_plates, pieces};
use super::inherit::BORDER_SPACINGS;
use super::kinematics::tangent;
use super::state::{CONTINENTAL, Group, Plate, PlateState, Propagation, TRENCH};
use crate::error::CoreError;
use crate::sphere::kdtree::angle_of_chord2;
use crate::world::WorldState;

/// The plate_birth `origin` of a margin propagation's new plate.
pub const ORIGIN: &str = "margin_propagation";
/// World globals (measurement, PLT-656): the margin broken by propagations, planet km …
pub const KM_GLOBAL: &str = "plates/propagation_km";
/// … and the time the propagations were alive, Myr summed over them (the mean speed is the
/// ratio).
pub const MYR_GLOBAL: &str = "plates/propagation_myr";

/// Diagnosis (tests only, `plates::tests::why_law_2_propagations_end`): why starts stall, why
/// live propagations wait or end, why advances stall. Indices into [`TALLY_NAMES`].
const T_START_STALL: usize = 0;
const T_START_TEAR: usize = 8;
const T_START_NO_FLOOR: usize = 9;
const T_END_DEAD: usize = 10;
#[cfg(test)]
const T_END_NO_KP: usize = 11;
#[cfg(test)]
const T_END_NO_COAST: usize = 12;
#[cfg(test)]
const T_END_NO_U: usize = 13;
const T_END_NORMAL: usize = 14;
const T_END_U_NEAR: usize = 15;
const T_WAIT_CONV: usize = 16;
const T_WAIT_PULL: usize = 17;
const T_ADV_NO_END: usize = 18;
const T_ADV_NORMAL: usize = 19;
const T_ADV_TEAR: usize = 20;
const T_ADV_STALL: usize = 21;
const T_ADV_GO: usize = 27;
const T_VISIT: usize = 28;
const T_START_GEOMETRY: usize = 29;
const T_START_SMALL_C: usize = 30;
#[cfg(test)]
pub(super) const TALLY_NAMES: [&str; 31] = [
    "start stall: e or P not on U",
    "start stall: a continent sample not on K'",
    "start stall: K' in more than one piece",
    "start stall: more pieces than before plus one",
    "start stall: K' below the minimum",
    "start stall: U below the minimum",
    "start stall: no stretch",
    "start stall: the stretch reaches past P",
    "start stall: no tear",
    "start made a K' with no visible floor",
    "end: U or K' died",
    "end: no K' sample within 2 spacings of the carried tip",
    "end: K' there, but no K' coast floor",
    "end: K' coast floor there, but none with U's crust within reach",
    "end: no normal at the tip",
    "end: no U crust within reach at the force test",
    "wait: convergence under 100 km",
    "wait: pull under the strength",
    "advance stall: no stretch end",
    "advance stall: no normal at the end",
    "advance stall: no tear",
    "advance stall: the tip not moved",
    "advance stall: K' below the minimum",
    "advance stall: K' in more pieces",
    "advance stall: U in more pieces",
    "advance stall: no stretch",
    "advance stall: the stretch reaches past P'",
    "advances",
    "visits (tip found)",
    "start none: the trench does not run on along the margin (Zhou's geometry)",
    "start none: the continent under a plate's worth of samples",
];
#[cfg(test)]
pub(super) static TALLY: [std::sync::atomic::AtomicU32; 31] =
    [const { std::sync::atomic::AtomicU32::new(0) }; 31];

fn tally(_i: usize) {
    #[cfg(test)]
    TALLY[_i].fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// Diagnosis (tests only): the samples within about 7 spacings of `c`, visible ones left and
/// hidden ones right, in a tangent frame (east right, north up). U: floor `u` (a trench `T`),
/// land `U`, hidden `h`; K': floor `k`, land `C`, hidden `x`; others: floor `o`, land `O`,
/// hidden `.`; the centre `@`.
#[cfg(test)]
fn sketch(st: &PlateState, pos: &[DVec3], c: DVec3, sp: f64, u: u32, kp: u32) -> String {
    const R: i32 = 9;
    const CELL: f64 = 0.8;
    let east = DVec3::Z.cross(c).normalize_or(DVec3::X);
    let north = c.cross(east);
    let w = (2 * R + 1) as usize;
    let mut grid = vec![vec![(f64::INFINITY, ' '); 2 * w]; w];
    for (i, &p) in pos.iter().enumerate() {
        if p.angle_between(c) > (R as f64 + 1.0) * CELL * sp {
            continue;
        }
        let (x, y) = (
            (p - c).dot(east) / sp / CELL,
            (p - c).dot(north) / sp / CELL,
        );
        let (cx, cy) = (x.round() as i32, y.round() as i32);
        if cx.abs() > R || cy.abs() > R {
            continue;
        }
        let s = &st.samples[i];
        let pl = st.plate_of(i);
        let land = s.kind == CONTINENTAL;
        let ch = match (pl == u, pl == kp, s.is_hidden(), land) {
            (true, _, true, _) => 'h',
            (true, _, false, true) => 'U',
            (true, _, false, false) if s.flags & TRENCH != 0 => 'T',
            (true, _, false, false) => 'u',
            (_, true, true, _) => 'x',
            (_, true, false, true) => 'C',
            (_, true, false, false) => 'k',
            (_, _, true, _) => '.',
            (_, _, false, true) => 'O',
            _ => 'o',
        };
        let d = (x - cx as f64).hypot(y - cy as f64);
        let col = (cx + R) as usize + if s.is_hidden() { w } else { 0 };
        let cell = &mut grid[(R - cy) as usize][col];
        if d < cell.0 {
            *cell = (d, ch);
        }
    }
    grid[R as usize][R as usize].1 = '@';
    grid.iter()
        .map(|row| {
            let (a, b) = row.split_at(w);
            format!(
                "    {} | {}\n",
                a.iter().map(|c| c.1).collect::<String>(),
                b.iter().map(|c| c.1).collect::<String>()
            )
        })
        .collect()
}

/// Diagnosis (tests only): how many propagations have been sketched.
#[cfg(test)]
static SKETCHED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// How far beyond "near" the start's stretch may reach from the trench, in lattice spacings:
/// the broken stretch at a start is the coast floor within this of the trench sample, one
/// lattice step along the margin. ledger: PLT-659 (game-choice: the lattice's resolution)
const START_REACH_SPACINGS: f64 = BORDER_SPACINGS + 0.1;
/// How far the next step looks for the tip it carried, in lattice spacings. ledger: PLT-659
/// (game-choice: the lattice's resolution)
const TIP_SEARCH_SPACINGS: f64 = 2.0;
/// The tear's step and its overhang at both ends, in lattice spacings. ledger: PLT-659
/// (game-choice: the lattice's resolution)
const TEAR_STEP_SPACINGS: f64 = 0.5;
/// How far the margin's strike is read along its coast floor, in lattice spacings: within the
/// border's 1.5 spacings a point of the margin sees one or two samples of its continent, too few
/// for a direction. ledger: PLT-659 (game-choice: the lattice's resolution)
const STRIKE_SPACINGS: f64 = 2.0;
/// How far a trench's strike is read along its trench samples from the trench sample nearest the
/// margin, in lattice spacings: twice the margin's [`STRIKE_SPACINGS`], since that sample is at
/// the trench's end and sees its trench on one side only. ledger: PLT-659 (game-choice: the
/// lattice's resolution)
const TRENCH_STRIKE_SPACINGS: f64 = 2.0 * STRIKE_SPACINGS;
/// The coast floor reads as a line (a strike) when its variance along its principal direction is
/// at least this many times its variance across it; a corner or a cape does not. ledger: PLT-659
/// (game-choice: the lattice's resolution)
const STRIKE_ANISOTROPY: f64 = 2.0;

/// What a planned cut or advance found.
enum Plan<T> {
    /// Nothing to do: no candidate, or the pull does not beat the margin's strength.
    None,
    /// The force test passed but the lattice gave no clean cut (a stall, PLT-656).
    Stall,
    Go(T),
}

/// A start's cut: plate `under` (U) keeps everything but `split`, which becomes K'.
struct Cut {
    under: u32,
    /// The broken stretch's end (U's coast floor, staying on U).
    tip: usize,
    split: Vec<u32>,
    /// Coast floor samples of the broken stretch.
    stretch: usize,
    pull_n_m: f64,
    strength_n_m: f64,
    age_myr: f64,
    tear_rad: f64,
}

/// An advance: `moved` goes from K' to U; `tip` is the stretch's end.
struct Advance {
    tip: usize,
    moved: Vec<u32>,
    stretch: usize,
}

/// The lattice as this call sees it.
struct View<'a> {
    st: &'a PlateState,
    geo: &'a Geo,
    pos: &'a [DVec3],
    plate: Vec<u32>,
    /// Visible samples.
    vis: &'a Tree,
    /// A visible ocean floor sample with a lattice neighbour of its own plate's visible
    /// continental crust within reach.
    coast: Vec<bool>,
    sp: f64,
    reach: f64,
}

impl<'a> View<'a> {
    fn new(st: &'a PlateState, geo: &'a Geo, pos: &'a [DVec3], vis: &'a Tree, sp: f64) -> Self {
        let n = st.samples.len();
        let plate: Vec<u32> = (0..n).map(|i| st.plate_of(i)).collect();
        let reach = BORDER_SPACINGS * sp;
        let mut v = View {
            st,
            geo,
            pos,
            plate,
            vis,
            coast: Vec::new(),
            sp,
            reach,
        };
        v.coast = (0..n)
            .map(|i| {
                let pl = v.plate[i];
                v.floor(i, pl)
                    && geo.edges(i as u32).any(|(j, _)| {
                        v.land(j as usize, pl) && pos[i].angle_between(pos[j as usize]) < reach
                    })
            })
            .collect();
        v
    }

    fn visible(&self, i: usize, pl: u32) -> bool {
        self.plate[i] == pl && !self.st.samples[i].is_hidden()
    }

    fn floor(&self, i: usize, pl: u32) -> bool {
        self.visible(i, pl) && self.st.samples[i].kind != CONTINENTAL
    }

    fn land(&self, i: usize, pl: u32) -> bool {
        self.visible(i, pl) && self.st.samples[i].kind == CONTINENTAL
    }

    /// The unit tangent at `x` towards the visible samples `of` selects within reach, if any.
    fn toward(&self, x: DVec3, of: impl Fn(usize) -> bool) -> Option<DVec3> {
        let mut sum = DVec3::ZERO;
        for (j, _) in self.vis.within(x, self.reach) {
            let j = j as usize;
            if !of(j) {
                continue;
            }
            let t = tangent(x, self.pos[j] - x);
            if t.length_squared() > 1e-24 {
                sum += t.normalize();
            }
        }
        (sum.length_squared() > 1e-24).then(|| sum.normalize())
    }

    /// The margin's inward normal at `x`, a unit tangent towards plate `pl`'s continent: across
    /// the strike of `pl`'s coast floor ([`View::across`]), signed towards the continent within
    /// reach ([`View::toward`]). ledger: PLT-659
    fn normal(&self, x: DVec3, pl: u32) -> Option<DVec3> {
        let toward = self.toward(x, |j| self.land(j, pl))?;
        Some(self.across(
            x,
            STRIKE_SPACINGS,
            |j| self.coast[j] && self.plate[j] == pl,
            toward,
        ))
    }

    /// The unit tangent at `x` across the principal direction (the strike) of the visible
    /// samples `line` selects within `window` lattice spacings, signed towards `toward` (a unit
    /// tangent); `toward` itself where they are no line (fewer than three samples, or a variance
    /// along under [`STRIKE_ANISOTROPY`] times the variance across). ledger: PLT-659
    fn across(&self, x: DVec3, window: f64, line: impl Fn(usize) -> bool, toward: DVec3) -> DVec3 {
        let Some(across) = self.line_normal(x, window, line) else {
            return toward;
        };
        let side = across.dot(toward);
        if side.abs() < 1e-9 {
            return toward;
        }
        if side > 0.0 { across } else { -across }
    }

    /// The unit tangent at `x` across the strike of the visible samples `line` selects within
    /// `window` lattice spacings, unsigned; None where they are no line (as [`View::across`]).
    /// ledger: PLT-659
    fn line_normal(&self, x: DVec3, window: f64, line: impl Fn(usize) -> bool) -> Option<DVec3> {
        let near: Vec<DVec3> = self
            .vis
            .within(x, window * self.sp)
            .into_iter()
            .map(|(j, _)| j as usize)
            .filter(|&j| line(j))
            .map(|j| tangent(x, self.pos[j] - x))
            .collect();
        if near.len() < 3 {
            return None;
        }
        let e1 = tangent(x, if x.x.abs() < 0.9 { DVec3::X } else { DVec3::Y }).normalize();
        let e2 = x.cross(e1);
        let n = near.len() as f64;
        let (mx, my) = near.iter().fold((0.0, 0.0), |a, t| {
            (a.0 + t.dot(e1) / n, a.1 + t.dot(e2) / n)
        });
        let (mut sxx, mut syy, mut sxy) = (0.0, 0.0, 0.0);
        for t in &near {
            let (a, b) = (t.dot(e1) - mx, t.dot(e2) - my);
            sxx += a * a;
            syy += b * b;
            sxy += a * b;
        }
        let half = 0.5 * (sxx + syy);
        let dev = (0.25 * (sxx - syy) * (sxx - syy) + sxy * sxy).sqrt();
        if half + dev < STRIKE_ANISOTROPY * (half - dev) {
            return None;
        }
        let th = 0.5 * (2.0 * sxy).atan2(sxx - syy);
        let strike = e1 * th.cos() + e2 * th.sin();
        let across = x.cross(strike).normalize_or_zero();
        (across != DVec3::ZERO).then_some(across)
    }

    /// The nearest visible sample within `angle` of `x` that `of` selects (ties: the lower
    /// index).
    fn nearest(&self, x: DVec3, angle: f64, of: impl Fn(usize) -> bool) -> Option<usize> {
        self.vis
            .within(x, angle)
            .into_iter()
            .filter(|&(j, _)| of(j as usize))
            .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)))
            .map(|(j, _)| j as usize)
    }

    /// The continent at `coast`: the visible continental crust of plate `pl` connected over
    /// lattice edges to the nearest such lattice neighbour of `coast`.
    fn continent(&self, coast: usize, pl: u32) -> Vec<bool> {
        let n = self.plate.len();
        let mut seen = vec![false; n];
        let start = self
            .geo
            .edges(coast as u32)
            .map(|(j, _)| j as usize)
            .filter(|&j| self.land(j, pl))
            .min_by(|&a, &b| {
                let x = self.pos[coast];
                x.angle_between(self.pos[a])
                    .total_cmp(&x.angle_between(self.pos[b]))
                    .then(a.cmp(&b))
            });
        let Some(start) = start else {
            return seen;
        };
        flood(self.geo, &mut seen, &[start], |_, j| self.land(j, pl));
        seen
    }

    /// The tear from the stretch's end `p` in the direction `d` (a unit tangent at `p`, away
    /// from the continent) to the first visible sample not of plate `own`; `p` counts on the
    /// side of `side_of`. None when it does not leave the plate within half the globe.
    fn tear(&self, p: usize, d: DVec3, own: u32, side_of: usize) -> Option<Tear> {
        let x = self.pos[p];
        let step = TEAR_STEP_SPACINGS * self.sp;
        let m = x.cross(d).normalize_or_zero();
        if m == DVec3::ZERO {
            return None;
        }
        let mut s = step;
        while s <= PI {
            let q = x * s.cos() + d * s.sin();
            let (j, _) = self.vis.nearest(q)?;
            if self.plate[j as usize] != own {
                return Some(Tear {
                    p,
                    x,
                    d,
                    m,
                    p_side: m.dot(self.pos[side_of]) > 0.0,
                    end: s,
                    pad: step,
                });
            }
            s += step;
        }
        None
    }
}

/// A great-circle tear through the lattice (a STEP fault, Govers & Wortel 2005).
struct Tear {
    p: usize,
    x: DVec3,
    d: DVec3,
    /// The pole of its great circle.
    m: DVec3,
    p_side: bool,
    /// Its length, rad.
    end: f64,
    /// Its overhang at both ends, rad.
    pad: f64,
}

impl Tear {
    fn side(&self, i: usize, pos: &[DVec3]) -> bool {
        if i == self.p {
            self.p_side
        } else {
            self.m.dot(pos[i]) > 0.0
        }
    }

    /// Whether the lattice edge `a`–`b` crosses the tear.
    fn cuts(&self, a: usize, b: usize, pos: &[DVec3]) -> bool {
        if self.side(a, pos) == self.side(b, pos) {
            return false;
        }
        let mid = (pos[a] + pos[b]).normalize_or_zero();
        let s = mid.dot(self.d).atan2(mid.dot(self.x));
        (-self.pad..=self.end + self.pad).contains(&s)
    }
}

/// Marks in `seen` every sample reachable from `from` over lattice edges `cross` allows
/// (`cross(from, to)`; a start is taken only if `cross` allows it from itself).
fn flood(geo: &Geo, seen: &mut [bool], from: &[usize], cross: impl Fn(usize, usize) -> bool) {
    let mut stack: Vec<usize> = Vec::new();
    for &f in from {
        if !seen[f] && cross(f, f) {
            seen[f] = true;
            stack.push(f);
        }
    }
    while let Some(v) = stack.pop() {
        for (u, _) in geo.edges(v as u32) {
            let u = u as usize;
            if !seen[u] && cross(v, u) {
                seen[u] = true;
                stack.push(u);
            }
        }
    }
}

/// For every hidden sample of plate `pl`, `flag` of the nearest visible sample of the plate
/// with the same crust (ocean floor follows ocean floor: a slab goes with the floor it hangs
/// from, not with a continent beside the trench); false elsewhere.
fn hidden_follow(v: &View, pl: u32, flag: &[bool]) -> Vec<bool> {
    let (st, pos) = (v.st, v.pos);
    let land = Tree::over(pos, |i| v.land(i, pl));
    let floor = Tree::over(pos, |i| v.floor(i, pl));
    (0..st.samples.len())
        .map(|i| {
            let s = &st.samples[i];
            if v.plate[i] != pl || !s.is_hidden() {
                return false;
            }
            let (a, b) = if s.kind == CONTINENTAL {
                (&land, &floor)
            } else {
                (&floor, &land)
            };
            a.nearest(pos[i])
                .or_else(|| b.nearest(pos[i]))
                .is_some_and(|(j, _)| flag[j as usize])
        })
        .collect()
}

/// The number of lattice pieces of label `of` (`pieces` over the samples `active` selects).
fn count_pieces(geo: &Geo, lab: &[u32], of: u32, active: impl Fn(usize) -> bool) -> usize {
    let (_, sizes, _) = pieces(geo, lab, active);
    sizes.iter().filter(|&&(p, s)| p == of && s > 0).count()
}

/// The fewest samples a cut leaves on each side: a plate the burial would not delete at once
/// (`min_plate_area_frac`), and at least a subduction initiation's 32. ledger: PLT-659
fn min_samples(env: &Env, n: usize) -> usize {
    ((env.k.min_plate_area_frac * n as f64).ceil() as usize).max(32)
}

/// Law 2's step (PLT-656): carries on every live propagation (an advance where its convergence
/// and its pull allow), then starts one at every plate whose trench reaches its own margin
/// where the pull beats the margin's strength. Runs after the step's subduction initiations;
/// `pos` are this step's positions, the plates' rotations the next step's.
pub fn step(
    state: &mut PlateState,
    world: &mut WorldState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
) -> Result<(), CoreError> {
    let k = env.k;
    if !k.margin_propagation {
        state.propagations.clear();
        return Ok(());
    }
    let sp = env.norm.spacing_rad();
    let vis = Tree::over(pos, |i| !state.samples[i].is_hidden());
    let mut changed = false;
    let live: Vec<Propagation> = std::mem::take(&mut state.propagations);
    let mut kept: Vec<Propagation> = Vec::new();
    for mut pr in live {
        let alive = |p: u32| state.plates.get(p as usize).is_some_and(|x| x.alive);
        if !alive(pr.under) || !alive(pr.over) {
            tally(T_END_DEAD);
            continue;
        }
        let (plan, tip) = {
            let v = View::new(state, geo, pos, &vis, sp);
            match carry_on(&v, env, &mut pr) {
                None => (Plan::None, None),
                Some((plan, tip)) => (plan, Some(tip)),
            }
        };
        let Some(tip) = tip else {
            continue;
        };
        *world.globals.entry(MYR_GLOBAL.to_owned()).or_insert(0.0) += env.dt_myr;
        tally(T_VISIT);
        pr.tip = tip.to_array();
        match plan {
            Plan::None => {}
            Plan::Stall => {
                state.counters.propagation_stalls += 1;
                continue;
            }
            Plan::Go(adv) => {
                tally(T_ADV_GO);
                // The tip moves to the broken stretch's end.
                apply_advance(state, world, pos, env, &mut pr, &adv);
                changed = true;
            }
        }
        kept.push(pr);
    }
    state.propagations = kept;
    // New propagations, one per plate with a trench (none where the plate already has one).
    let mut ids: Vec<u32> = (0..state.samples.len())
        .filter(|&i| {
            let s = &state.samples[i];
            s.flags & TRENCH != 0 && !s.is_hidden() && s.kind != CONTINENTAL
        })
        .map(|i| state.plate_of(i))
        .collect();
    ids.sort_unstable();
    ids.dedup();
    for u in ids {
        if state.alive().count() as u32 >= k.max_plates {
            break;
        }
        if !state.plates[u as usize].alive || state.propagations.iter().any(|p| p.under == u) {
            continue;
        }
        let plan = {
            let v = View::new(state, geo, pos, &vis, sp);
            plan_start(&v, env, u)
        };
        match plan {
            Plan::None => {}
            Plan::Stall => state.counters.propagation_stalls += 1,
            Plan::Go(cut) => {
                apply_start(state, world, pos, env, &cut)?;
                changed = true;
            }
        }
    }
    if changed {
        state.recount(pos);
    }
    // Each tip rides K' through the next step.
    for pr in &mut state.propagations {
        let w = DVec3::from_array(state.plates[pr.over as usize].omega) * env.dt_yr;
        let q = DQuat::from_scaled_axis(w);
        pr.tip = (q * DVec3::from_array(pr.tip)).normalize().to_array();
    }
    Ok(())
}

/// Plans a start at plate `u` (PLT-656): its trench sample nearest its own coast floor, within
/// reach of it, whose overriding plate's slab is past the critical convergence; the stretch's
/// end, U's coast floor within [`START_REACH_SPACINGS`] of it farthest from U's trench; the pull
/// of that slab across the margin there against the margin's strength; then the cut.
fn plan_start(v: &View, env: &Env, u: u32) -> Plan<Cut> {
    let k = env.k;
    let t = env.t_myr;
    let (st, pos) = (v.st, v.pos);
    let n = st.samples.len();
    let trench: Vec<usize> = (0..n)
        .filter(|&i| v.floor(i, u) && st.samples[i].flags & TRENCH != 0)
        .collect();
    if trench.is_empty() || !(0..n).any(|i| v.coast[i] && v.plate[i] == u) {
        return Plan::None;
    }
    let coast_tree = Tree::over(pos, |i| v.coast[i] && v.plate[i] == u);
    let is_trench = |i: usize| st.samples[i].flags & TRENCH != 0 && v.floor(i, u);
    let trench_tree = Tree::over(pos, is_trench);
    let mut cands: Vec<(f64, usize)> = trench
        .iter()
        .filter_map(|&e| {
            let (_, d2) = coast_tree.nearest(pos[e])?;
            let d = angle_of_chord2(d2);
            (d <= v.reach).then_some((d, e))
        })
        .collect();
    cands.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    let found = cands.iter().find_map(|&(_, e)| {
        if st.samples[e].age_myr(t) < SLAB_BUOYANT_MYR {
            return None;
        }
        let o_at = v.nearest(pos[e], 2.0 * v.reach, |j| v.plate[j] != u)?;
        let o = v.plate[o_at];
        let slab = st
            .slabs
            .iter()
            .find(|s| s.under == u && s.over == o && !s.latent && s.km >= INCIPIENT_SLAB_KM)?;
        Some((e, o, o_at, slab.km))
    });
    let Some((e, o, o_at, slab_km)) = found else {
        return Plan::None;
    };
    let reach_p = START_REACH_SPACINGS * v.sp;
    let to_trench = |i: usize| {
        trench_tree
            .nearest(pos[i])
            .map_or(f64::INFINITY, |(_, d2)| angle_of_chord2(d2))
    };
    let tip = coast_tree
        .within(pos[e], reach_p)
        .into_iter()
        .map(|(i, _)| i as usize)
        .filter(|&i| st.samples[i].flags & TRENCH == 0)
        .max_by(|&a, &b| to_trench(a).total_cmp(&to_trench(b)).then(b.cmp(&a)));
    let Some(p) = tip else {
        return Plan::None;
    };
    let Some(inward) = v.normal(pos[p], u) else {
        return Plan::None;
    };
    // The slab pulls U's floor across U's trench towards O: across the trench's strike at e,
    // signed towards O's visible samples there (alone, those lean towards the side past the
    // trench's end, where no hidden slab lies over O's edge). ledger: PLT-659
    let to_o = v
        .toward(pos[e], |j| v.visible(j, o))
        .unwrap_or_else(|| tangent(pos[e], pos[o_at] - pos[e]).normalize_or_zero());
    let drive = v.across(pos[e], TRENCH_STRIKE_SPACINGS, is_trench, to_o);
    // Zhou et al. 2020's geometry (ledger: PLT-717): the trench runs on along the margin, so the
    // tear across the margin is a STEP at the trench's end. The trench (U's trench samples within
    // `TRENCH_STRIKE_SPACINGS` of e) and the margin (U's coast floor within `STRIKE_SPACINGS` of
    // p) each read as a line, their strikes lie within atan(1.5 / 4), about 20°, of each other (a
    // border's offset, PLT-534, over the trench's window: the angle at which the lattice can
    // still tell the trench from the margin's own line), and e lies within the border of the
    // margin's line through p. Where the trench meets the margin at an angle, U moves along the
    // margin and the start is half a process (arm fl-law2: 116 starts a run, no advance).
    let strike_cos = TRENCH_STRIKE_SPACINGS / TRENCH_STRIKE_SPACINGS.hypot(BORDER_SPACINGS);
    let trench_n = v.line_normal(pos[e], TRENCH_STRIKE_SPACINGS, is_trench);
    let margin_n = v.line_normal(pos[p], STRIKE_SPACINGS, |j| v.coast[j] && v.plate[j] == u);
    let off_line = pos[e].dot(inward).abs().asin();
    let zhou = match (trench_n, margin_n) {
        (Some(tn), Some(mn)) => tn.dot(mn).abs() >= strike_cos && off_line <= v.reach,
        _ => false,
    };
    if !zhou {
        tally(T_START_GEOMETRY);
        return Plan::None;
    }
    let cos = drive.dot(inward);
    let age_e = st.samples[e].age_myr(t);
    let age_p = st.samples[p].age_myr(t);
    let pull = forces::margin_pull_n_m(k, slab_km, age_e, cos, env.gravity);
    let strength = forces::margin_strength_n_m(k, age_p, env.gravity);
    if pull < strength {
        return Plan::None;
    }
    // The cut: the continent and the floor beyond the tear go to K'. A continent of fewer
    // samples than a plate may hold (`min_samples`) is a speck, no passive margin (arm fl-law2's
    // starts at "continents" of 1 to 11 samples). ledger: PLT-717
    let cont = v.continent(p, u);
    if cont.iter().filter(|&&c| c).count() < min_samples(env, n) {
        tally(T_START_SMALL_C);
        return Plan::None;
    }
    let Some(tear) = v.tear(p, -inward, u, e) else {
        tally(T_START_TEAR);
        return Plan::Stall;
    };
    let mut on_u = vec![false; n];
    flood(v.geo, &mut on_u, &[e], |a, b| {
        v.visible(b, u) && !cont[b] && (a == b || !tear.cuts(a, b, pos))
    });
    on_u[p] = true;
    let members: Vec<bool> = (0..n).map(|i| v.plate[i] == u).collect();
    let lab_hidden = hidden_follow(v, u, &on_u);
    let lab: Vec<u32> = (0..n)
        .map(|i| {
            if !members[i] {
                u32::MAX
            } else if st.samples[i].is_hidden() {
                u32::from(lab_hidden[i])
            } else {
                u32::from(on_u[i])
            }
        })
        .collect();
    let mut lab = lab;
    let before = count_pieces(v.geo, &v.plate, u, |i| members[i]);
    connect_plates(v.geo, &mut lab, |i| members[i]);
    let split: Vec<u32> = (0..n as u32).filter(|&i| lab[i as usize] == 0).collect();
    let stays = members.iter().filter(|&&m| m).count() - split.len();
    let min = min_samples(env, n);
    let n0 = count_pieces(v.geo, &lab, 0, |i| members[i]);
    let n1 = count_pieces(v.geo, &lab, 1, |i| members[i]);
    let reach_stretch = pos[p].angle_between(pos[e]) + 0.75 * v.sp;
    let stretch: Vec<usize> = (0..n)
        .filter(|&i| {
            lab[i] == 1
                && v.floor(i, u)
                && v.geo.edges(i as u32).any(|(j, _)| {
                    cont[j as usize] && pos[i].angle_between(pos[j as usize]) < v.reach
                })
        })
        .collect();
    let fails = [
        lab[e] != 1 || lab[p] != 1,
        (0..n).any(|i| cont[i] && lab[i] != 0),
        n0 != 1,
        n0 + n1 > before + 1,
        split.len() < min,
        stays < min,
        stretch.is_empty(),
        stretch
            .iter()
            .any(|&i| pos[i].angle_between(pos[e]) > reach_stretch),
    ];
    if let Some(f) = fails.iter().position(|&x| x) {
        tally(T_START_STALL + f);
        return Plan::Stall;
    }
    Plan::Go(Cut {
        under: u,
        tip: p,
        split,
        stretch: stretch.len(),
        pull_n_m: pull,
        strength_n_m: strength,
        age_myr: age_p,
        tear_rad: tear.end,
    })
}

/// Carries on propagation `pr` (PLT-656): finds its tip (K''s coast floor beside U's crust,
/// visible or hidden, nearest the tip it carried), adds the step's convergence of U on K' across
/// the margin there, and plans an advance once that reaches the critical convergence and U's slab
/// beside the tip pulls across the next stretch harder than its strength. None: the propagation
/// has ended (no tip).
///
/// U's crust beside the tip counts hidden too: where the trench meets the margin at an angle,
/// U keeps a wedge of floor between the trench and the tear that is narrow at the tip, and its
/// slab takes that floor down within a step or two; the slab still hangs from U there. The slab
/// that pulls is U's under the plate that covers U's crust nearest the tip: K' (the new margin's
/// own slab) where that crust is visible, the plate over it where it is hidden. ledger: PLT-656
fn carry_on(v: &View, env: &Env, pr: &mut Propagation) -> Option<(Plan<Advance>, DVec3)> {
    let k = env.k;
    let t = env.t_myr;
    let (st, pos) = (v.st, v.pos);
    let (u, kp) = (pr.under, pr.over);
    let carried = DVec3::from_array(pr.tip);
    let u_crust = Tree::over(pos, |i| v.plate[i] == u);
    let u_beside = |x: DVec3| {
        u_crust
            .nearest(x)
            .filter(|&(_, d2)| angle_of_chord2(d2) <= v.reach)
            .map(|(j, _)| j as usize)
    };
    let tip = v
        .vis
        .within(carried, TIP_SEARCH_SPACINGS * v.sp)
        .into_iter()
        .filter(|&(i, _)| {
            let i = i as usize;
            v.coast[i] && v.plate[i] == kp && u_beside(pos[i]).is_some()
        })
        .min_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)))
        .map(|(i, _)| i as usize);
    #[cfg(test)]
    if std::env::var_os("LAW2_DIAG_TRACE").is_some() {
        // In spacings from the carried tip: the nearest sample of each kind (all samples).
        let d = |of: &dyn Fn(usize) -> bool| {
            (0..pos.len())
                .filter(|&j| of(j))
                .map(|j| pos[j].angle_between(carried) / v.sp)
                .fold(f64::INFINITY, f64::min)
        };
        let hid = |j: usize| st.samples[j].is_hidden();
        eprintln!(
            "law2 t {t:.0} U {u} K' {kp} age {:.0} conv {:.0} km, step {} resampled {}: tip {} | U vis {:.2} hid {:.2}, K' coast {:.2} floor {:.2} land {:.2} hid {:.2}, other {:.2}",
            t - pr.born_myr,
            pr.converged_km,
            st.step,
            st.resample_step,
            tip.is_some(),
            d(&|j| v.plate[j] == u && !hid(j)),
            d(&|j| v.plate[j] == u && hid(j)),
            d(&|j| v.plate[j] == kp && v.coast[j]),
            d(&|j| v.floor(j, kp)),
            d(&|j| v.land(j, kp)),
            d(&|j| v.plate[j] == kp && hid(j)),
            d(&|j| v.plate[j] != kp && v.plate[j] != u && !hid(j)),
        );
        if SKETCHED.fetch_add(1, std::sync::atomic::Ordering::Relaxed) < 24 {
            eprint!("{}", sketch(st, pos, carried, v.sp, u, kp));
        }
    }
    let Some(tip) = tip else {
        #[cfg(test)]
        {
            let near = v.vis.within(carried, TIP_SEARCH_SPACINGS * v.sp);
            let of_kp = |i: u32| v.plate[i as usize] == kp;
            tally(if !near.iter().any(|&(i, _)| of_kp(i)) {
                T_END_NO_KP
            } else if !near.iter().any(|&(i, _)| of_kp(i) && v.coast[i as usize]) {
                T_END_NO_COAST
            } else {
                T_END_NO_U
            });
        }
        return None;
    };
    let x = pos[tip];
    let Some(inward) = v.normal(x, kp) else {
        tally(T_END_NORMAL);
        return None;
    };
    let omega = |p: u32| DVec3::from_array(st.plates[p as usize].omega);
    let conv = env.norm.velocity_mm_yr(omega(u) - omega(kp), x).dot(inward);
    pr.converged_km += conv.max(0.0) * env.dt_myr;
    if pr.converged_km < INCIPIENT_SLAB_KM {
        tally(T_WAIT_CONV);
        return Some((Plan::None, x));
    }
    let Some(u_near) = u_beside(x) else {
        tally(T_END_U_NEAR);
        return None;
    };
    let over = if st.samples[u_near].is_hidden() {
        v.nearest(pos[u_near], PI, |j| v.plate[j] != u)
            .map_or(kp, |j| v.plate[j])
    } else {
        kp
    };
    let slab_km = st
        .slabs
        .iter()
        .find(|s| s.under == u && s.over == over && !s.latent)
        .map_or(0.0, |s| s.km);
    // Under K' it pulls across the broken stretch, towards K''s continent; under another plate,
    // across U's trench there towards that plate, as at the start.
    let drive = if over == kp {
        v.toward(pos[u_near], |j| v.land(j, kp))
    } else {
        v.toward(pos[u_near], |j| v.visible(j, over)).map(|w| {
            let is_trench = |j: usize| st.samples[j].flags & TRENCH != 0 && v.floor(j, u);
            v.across(pos[u_near], TRENCH_STRIKE_SPACINGS, is_trench, w)
        })
    };
    let cos = drive.map_or(0.0, |w| w.dot(inward));
    let pull = forces::margin_pull_n_m(k, slab_km, st.samples[u_near].age_myr(t), cos, env.gravity);
    let strength = forces::margin_strength_n_m(k, st.samples[tip].age_myr(t), env.gravity);
    #[cfg(test)]
    if std::env::var_os("LAW2_DIAG_TRACE").is_some() {
        eprintln!(
            "law2 FORCE U {u} K' {kp}: U crust at {:.2} sp (hidden {}), over {over}, slab {slab_km:.0} km, cos {cos:.2}, pull {pull:.2e}, strength {strength:.2e}",
            pos[u_near].angle_between(x) / v.sp,
            st.samples[u_near].is_hidden()
        );
    }
    if pull < strength {
        tally(T_WAIT_PULL);
        return Some((Plan::None, x));
    }
    Some((plan_advance(v, env, kp, u, tip, u_near, &u_crust), x))
}

/// Plans an advance: the stretch's end P' is K''s coast floor within [`START_REACH_SPACINGS`]
/// of the tip, the farthest from U's crust, visible or hidden (`u_crust`; one lattice step along
/// the margin, as at the start); K''s floor from the tip to a tear from P' away from the
/// continent goes to U, P' with it (PLT-656).
fn plan_advance(
    v: &View,
    env: &Env,
    kp: u32,
    u: u32,
    tip: usize,
    u_near: usize,
    u_crust: &Tree,
) -> Plan<Advance> {
    let (st, pos) = (v.st, v.pos);
    let n = st.samples.len();
    let to_u = |i: usize| {
        u_crust
            .nearest(pos[i])
            .map_or(f64::INFINITY, |(_, d2)| angle_of_chord2(d2))
    };
    let end = v
        .vis
        .within(pos[tip], START_REACH_SPACINGS * v.sp)
        .into_iter()
        .map(|(i, _)| i as usize)
        .filter(|&i| v.coast[i] && v.plate[i] == kp)
        .max_by(|&a, &b| to_u(a).total_cmp(&to_u(b)).then(b.cmp(&a)));
    let Some(p) = end else {
        tally(T_ADV_NO_END);
        return Plan::Stall;
    };
    let Some(inward) = v.normal(pos[p], kp) else {
        tally(T_ADV_NORMAL);
        return Plan::Stall;
    };
    let cont = v.continent(p, kp);
    let Some(tear) = v.tear(p, -inward, kp, u_near) else {
        tally(T_ADV_TEAR);
        return Plan::Stall;
    };
    let mut moved = vec![false; n];
    flood(v.geo, &mut moved, &[tip], |a, b| {
        v.visible(b, kp) && !cont[b] && (a == b || !tear.cuts(a, b, pos))
    });
    moved[p] = true;
    let follow = hidden_follow(v, kp, &moved);
    for (i, m) in moved.iter_mut().enumerate() {
        if v.plate[i] == kp && st.samples[i].is_hidden() {
            *m = follow[i];
        }
    }
    let mut lab = v.plate.clone();
    for (i, m) in moved.iter().enumerate() {
        if *m {
            lab[i] = u;
        }
    }
    let left = lab.iter().filter(|&&p| p == kp).count();
    let stretch: Vec<usize> = (0..n)
        .filter(|&i| {
            moved[i]
                && v.floor(i, kp)
                && v.geo.edges(i as u32).any(|(j, _)| {
                    cont[j as usize] && pos[i].angle_between(pos[j as usize]) < v.reach
                })
        })
        .collect();
    let k_before = count_pieces(v.geo, &v.plate, kp, |i| v.plate[i] == kp);
    let u_before = count_pieces(v.geo, &v.plate, u, |i| v.plate[i] == u);
    let k_after = count_pieces(v.geo, &lab, kp, |i| lab[i] == kp);
    let u_after = count_pieces(v.geo, &lab, u, |i| lab[i] == u);
    let reach_stretch = pos[p].angle_between(pos[tip]) + 0.75 * v.sp;
    let fails = [
        !moved[tip],
        left < min_samples(env, n),
        k_after > k_before,
        u_after > u_before,
        stretch.is_empty(),
        stretch
            .iter()
            .any(|&i| pos[i].angle_between(pos[tip]) > reach_stretch),
    ];
    if let Some(f) = fails.iter().position(|&x| x) {
        tally(T_ADV_STALL + f);
        return Plan::Stall;
    }
    Plan::Go(Advance {
        tip: p,
        moved: (0..n as u32).filter(|&i| moved[i as usize]).collect(),
        stretch: stretch.len(),
    })
}

/// Hands `samples` to plate `to`, each into a group of its own group's rotation (so nothing
/// moves).
fn hand_over(state: &mut PlateState, samples: &[u32], to: u32) {
    let mut twin: BTreeMap<u32, u32> = BTreeMap::new();
    for &i in samples {
        let g = state.samples[i as usize].group;
        let new = *twin.entry(g).or_insert_with(|| {
            let rot = state.groups[g as usize].rot;
            state.groups.push(Group { plate: to, rot });
            (state.groups.len() - 1) as u32
        });
        state.samples[i as usize].group = new;
    }
}

/// The planet length of `stretch` coast floor samples, km.
fn stretch_km(env: &Env, stretch: usize) -> f64 {
    env.norm
        .planet_len_km(stretch as f64 * env.norm.km(env.norm.spacing_rad()))
}

fn apply_start(
    state: &mut PlateState,
    world: &mut WorldState,
    pos: &[DVec3],
    env: &Env,
    cut: &Cut,
) -> Result<(), CoreError> {
    let t = env.t_myr;
    let norm = env.norm;
    let u = cut.under;
    let new_id = state.plates.len() as u32;
    let parent = state.plates[u as usize].clone();
    hand_over(state, &cut.split, new_id);
    let (mut c, mut vis, mut cont, mut ages, mut oceanic) = (DVec3::ZERO, 0usize, 0usize, 0.0, 0);
    for &i in &cut.split {
        let s = &state.samples[i as usize];
        if s.is_hidden() {
            continue;
        }
        vis += 1;
        c += pos[i as usize];
        if s.kind == CONTINENTAL {
            cont += 1;
        } else {
            ages += s.age_myr(t);
            oceanic += 1;
        }
    }
    if oceanic == 0 {
        tally(T_START_NO_FLOOR);
    }
    #[cfg(test)]
    if std::env::var_os("LAW2_DIAG_TRACE").is_some() {
        eprintln!(
            "law2 START t {t:.0} U {u} K' {new_id}: split {} (visible {vis}, land {cont}, floor {oceanic}), U had {}, stretch {}, tear {:.2} sp",
            cut.split.len(),
            parent.samples,
            cut.stretch,
            cut.tear_rad / norm.spacing_rad()
        );
        if SKETCHED.load(std::sync::atomic::Ordering::Relaxed) < 24 {
            eprint!(
                "{}",
                sketch(state, pos, pos[cut.tip], norm.spacing_rad(), u, new_id)
            );
        }
    }
    let c = c.normalize_or(pos[cut.tip]);
    let omega = DVec3::from_array(parent.omega);
    let count = cut.split.len();
    let frac = cont as f64 / vis.max(1) as f64;
    state.plates.push(Plate {
        id: new_id,
        alive: true,
        omega: omega.to_array(),
        born_myr: t,
        died_myr: None,
        birth_event: None,
        parent: Some(u),
        lip_until_myr: f64::NEG_INFINITY,
        lip_event: None,
        samples: count as u32,
        centroid: c.to_array(),
        continental_frac: frac,
    });
    let p = &mut state.plates[u as usize];
    p.samples = p.samples.saturating_sub(count as u32);
    let mean_age = if oceanic > 0 {
        ages / oceanic as f64
    } else {
        0.0
    };
    let e = events::event(world, t, PLATE_BIRTH, c)
        .magnitude((count as f64 * norm.sample_area_km2() / state.a0_km2) as f32)
        .param("plate", i64::from(new_id))
        .param("parent", i64::from(u))
        .param("origin", ORIGIN)
        .param(
            "area_km2",
            norm.planet_area_km2(count as f64 * norm.sample_area_km2()),
        )
        .param("mean_ocean_age_myr", mean_age)
        .param("continental_frac", frac)
        .param("speed_mm_yr", norm.speed_mm_yr(omega, c))
        .param("margin_age_myr", cut.age_myr)
        .param("pull_n_m", cut.pull_n_m)
        .param("strength_n_m", cut.strength_n_m)
        .param("tear_km", norm.planet_len_km(norm.km(cut.tear_rad)));
    let birth = world.emit(e)?;
    state.plates[new_id as usize].birth_event = Some(birth);
    state.counters.plate_births += 1;
    state.counters.propagation_starts += 1;
    state.counters.propagations += 1;
    *world.globals.entry(KM_GLOBAL.to_owned()).or_insert(0.0) += stretch_km(env, cut.stretch);
    // K' takes the boundaries of U it borders (ledger: PLT-534).
    if env.k.split_inheritance {
        super::inherit::inherit(state, pos, env, &[(u, vec![new_id])]).book(world);
    }
    state.propagations.push(Propagation {
        under: u,
        over: new_id,
        tip: pos[cut.tip].to_array(),
        converged_km: 0.0,
        advances: 0,
        born_myr: t,
        last_myr: t,
    });
    Ok(())
}

fn apply_advance(
    state: &mut PlateState,
    world: &mut WorldState,
    pos: &[DVec3],
    env: &Env,
    pr: &mut Propagation,
    adv: &Advance,
) {
    hand_over(state, &adv.moved, pr.under);
    let n = adv.moved.len() as u32;
    let k = &mut state.plates[pr.over as usize];
    k.samples = k.samples.saturating_sub(n);
    state.plates[pr.under as usize].samples += n;
    state.counters.propagations += 1;
    pr.advances += 1;
    pr.converged_km = 0.0;
    pr.last_myr = env.t_myr;
    pr.tip = pos[adv.tip].to_array();
    *world.globals.entry(KM_GLOBAL.to_owned()).or_insert(0.0) += stretch_km(env, adv.stretch);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::TectonicParams;
    use crate::plates::geometry::{cut_samples, positions};
    use crate::plates::normalise::Norm;
    use crate::plates::rifting::tests::{N, continental, old_floor, one_plate};
    use crate::plates::state::{HIDDEN, Sample, SlabTrack};

    /// Which plate and crust a test layout puts at a latitude and longitude, degrees.
    #[derive(Clone, Copy, PartialEq)]
    enum Cell {
        /// U's ocean floor.
        Floor,
        /// U's continent C.
        Land,
        /// U's slab under O (hidden).
        Slab,
        /// The overriding plate O.
        Over,
        /// Everything else, plate X.
        Rest,
    }

    const U: u32 = 0;
    const O: u32 = 1;
    const X: u32 = 2;
    /// The plate a start makes.
    const KP: u32 = 3;
    /// The age of the test floor, Myr.
    const AGE: f64 = 80.0;
    const T: f64 = 2.0;

    /// Law 2 on, with law 3 inside it (both off by default since 2026-10-26, PLT-656).
    fn law2() -> TectonicParams {
        TectonicParams {
            margin_propagation: true,
            incipient_resistance_by_thickness: true,
            ..TectonicParams::default()
        }
    }

    fn env(k: &TectonicParams) -> Env<'_> {
        Env {
            k,
            norm: Norm::new(6370.0, 6371.0, N),
            dt_myr: 2.0,
            dt_yr: 2.0e6,
            gravity: 9.81,
            step: 1,
            t_myr: T,
        }
    }

    fn lat_lon(p: DVec3) -> (f64, f64) {
        (
            p.z.clamp(-1.0, 1.0).asin().to_degrees(),
            p.y.atan2(p.x).to_degrees(),
        )
    }

    fn floor() -> Sample {
        let mut s = old_floor();
        s.formed_myr = (T - AGE) as f32;
        s
    }

    /// Plates U, O and X as `layout` says; U's floor beside its slab, or beside O where there is
    /// no slab, is its trench, and its slab under O is `slab_km` long.
    fn build(geo: &Geo, layout: impl Fn(f64, f64) -> Cell, slab_km: f64) -> PlateState {
        let cells: Vec<Cell> = geo
            .points()
            .iter()
            .map(|&p| {
                let (la, lo) = lat_lon(p);
                layout(la, lo)
            })
            .collect();
        let mut st = one_plate(geo, 1, |_| floor());
        for (pl, g) in [(O, 1u32), (X, 2)] {
            let mut p = st.plates[0].clone();
            p.id = pl;
            st.plates.push(p);
            st.groups.push(Group {
                plate: pl,
                rot: st.groups[0].rot,
            });
            assert_eq!(st.groups.len() as u32 - 1, g);
        }
        for (s, c) in st.samples.iter_mut().zip(&cells) {
            match c {
                Cell::Floor => {}
                Cell::Land => *s = continental(),
                Cell::Slab => s.flags |= HIDDEN,
                Cell::Over => s.group = 1,
                Cell::Rest => s.group = 2,
            }
        }
        for (i, c) in cells.iter().enumerate() {
            if *c == Cell::Floor
                && geo
                    .edges(i as u32)
                    .any(|(j, _)| matches!(cells[j as usize], Cell::Slab | Cell::Over))
            {
                st.samples[i].flags |= TRENCH;
            }
        }
        st.slabs.push(SlabTrack {
            under: U,
            over: O,
            km: slab_km,
            ..SlabTrack::default()
        });
        let pos = positions(&st, geo);
        st.recount(&pos);
        st
    }

    /// The trench runs along the equator west of 0° (U's floor south, O north beyond U's slab),
    /// and U's continent C lies north of the equator from 0° to `c_east`: the trench meets the
    /// margin end on, so the slab pulls U's floor straight across the margin. U's box ends at
    /// `u_east` and 40° S.
    fn line(la: f64, lo: f64, c_east: f64, u_east: f64) -> Cell {
        let slab = 3.0; // about two spacings of the 16,000-sample lattice
        if !(-60.0..u_east).contains(&lo) || !(-40.0..30.0).contains(&la) {
            Cell::Rest
        } else if la < 0.0 {
            Cell::Floor
        } else if (0.0..c_east).contains(&lo) {
            Cell::Land
        } else if lo < 0.0 && la < slab {
            Cell::Slab
        } else if lo < 0.0 {
            Cell::Over
        } else {
            Cell::Rest
        }
    }

    fn in_line(la: f64, lo: f64) -> Cell {
        line(la, lo, 40.0, 80.0)
    }

    /// As [`in_line`] with a narrow continent and box, so K' is the smallest plate.
    fn in_line_small(la: f64, lo: f64) -> Cell {
        line(la, lo, 12.0, 16.0)
    }

    /// The trench runs along the equator west of 2° E (U's floor south, O north), and C lies
    /// south of the equator east of 2° E: the trench meets the margin at a right angle, and the
    /// slab pulls U's floor north, along the coast, not across it.
    fn across(la: f64, lo: f64) -> Cell {
        if !(-60.0..80.0).contains(&lo) || !(-40.0..30.0).contains(&la) {
            Cell::Rest
        } else if la < 0.0 {
            if lo >= 2.0 { Cell::Land } else { Cell::Floor }
        } else {
            Cell::Over
        }
    }

    /// As [`across`], with U's slab hidden under O's edge along the trench (to 3° N) and none
    /// past the trench's end: O's visible samples near the end lie past it, to the north-east,
    /// but the slab pulls U's floor north, across its trench, along the coast.
    fn across_slab(la: f64, lo: f64) -> Cell {
        match across(la, lo) {
            Cell::Over if lo < 2.0 && la < 3.0 => Cell::Slab,
            c => c,
        }
    }

    fn start(geo: &Geo, layout: fn(f64, f64) -> Cell, k: &TectonicParams, slab: f64) -> PlateState {
        let mut st = build(geo, layout, slab);
        let pos = positions(&st, geo);
        let mut world = WorldState::new(1, 16_384, 0.0);
        step(&mut st, &mut world, geo, &pos, &env(k)).unwrap();
        st
    }

    /// Each plate is one lattice piece and every sample's plate is alive.
    fn clean(st: &PlateState, geo: &Geo) {
        let plate: Vec<u32> = (0..N).map(|i| st.plate_of(i)).collect();
        assert_eq!(cut_samples(geo, &plate, |_| true), 0, "orphan samples");
        for (i, &p) in plate.iter().enumerate() {
            assert!(st.plates[p as usize].alive, "sample {i} on a dead plate");
        }
    }

    /// The start (Zhou et al. 2020; Govers & Wortel 2005): with the slab under O past the
    /// critical convergence and pulling straight across the margin, C and the floor beyond a
    /// tear from the end of the broken stretch become a new plate K'; the trench and the slab
    /// stay on U, the tear runs south to U's boundary, and no plate is cut into pieces.
    #[test]
    fn a_trench_that_reaches_its_margin_cuts_it_off() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let k = law2();
        let mut st = build(&geo, in_line, 300.0);
        let pos = positions(&st, &geo);
        let mut world = WorldState::new(1, 16_384, 0.0);
        step(&mut st, &mut world, &geo, &pos, &env(&k)).unwrap();
        assert_eq!(st.plates.len(), 4, "a new plate");
        assert_eq!(st.counters.propagation_starts, 1);
        assert_eq!(st.counters.propagations, 1);
        assert_eq!(st.counters.plate_births, 1);
        assert_eq!(st.counters.propagation_stalls, 0);
        assert_eq!(st.propagations.len(), 1);
        assert_eq!((st.propagations[0].under, st.propagations[0].over), (U, KP));
        let sp = Norm::new(6370.0, 6371.0, N).spacing_rad().to_degrees();
        let mut tear_lon: Vec<f64> = Vec::new();
        for (i, &x) in pos.iter().enumerate() {
            let (la, lo) = lat_lon(x);
            let s = &st.samples[i];
            let pl = st.plate_of(i);
            if s.kind == CONTINENTAL {
                assert_eq!(pl, KP, "all of C goes to K'");
            }
            // Far west of the junction U's floor stays; far east it went with C.
            if !s.is_hidden() && s.kind != CONTINENTAL && la < 0.0 && la > -38.0 {
                if lo < -2.0 * sp && lo > -58.0 {
                    assert_eq!(pl, U, "floor west of the tear at {la:.1} {lo:.1}");
                }
                if lo > 8.0 * sp && lo < 78.0 {
                    assert_eq!(pl, KP, "floor east of the tear at {la:.1} {lo:.1}");
                }
            }
            if s.flags & TRENCH != 0 {
                assert_eq!(pl, U, "the trench stays on U");
            }
            if s.is_hidden() {
                assert_eq!(pl, U, "U keeps its slab");
            }
            if pl == U
                && !s.is_hidden()
                && la < -2.0 * sp
                && geo
                    .edges(i as u32)
                    .any(|(j, _)| st.plate_of(j as usize) == KP)
            {
                tear_lon.push(lo);
            }
        }
        // The tear: a line of U–K' edges from the margin down to U's southern boundary, near one
        // meridian.
        let (lo_min, lo_max) = tear_lon
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |a, &x| {
                (a.0.min(x), a.1.max(x))
            });
        eprintln!(
            "tear: {} samples on U's side, longitudes {lo_min:.2} to {lo_max:.2}",
            tear_lon.len()
        );
        assert!(tear_lon.len() >= 15, "the tear reaches U's boundary");
        assert!(lo_max - lo_min < 4.0 * sp, "the tear is straight");
        let birth = world
            .events
            .iter()
            .find(|e| e.kind == PLATE_BIRTH)
            .expect("a plate_birth");
        assert!(matches!(
            birth.params.get("origin"),
            Some(deeptime_format::ParamValue::Str(o)) if o == ORIGIN
        ));
        assert!(world.globals[KM_GLOBAL] > 0.0);
        clean(&st, &geo);
    }

    /// The pull acts across the margin only (cos θ, PLT-656): where the trench meets the margin
    /// at a right angle and the slab pulls U's floor along the coast, nothing starts, also where
    /// the slab hides O's edge along the trench, so O's visible samples near its end lie past it
    /// (the slab pulls across its trench's strike, PLT-659). The switch off, nothing starts; a slab short of the critical convergence (PLT-611) starts nothing;
    /// and a slab past it starts only once its pull beats the margin (PLT-658 with law 3 at 80
    /// Myr: 5.9e12 N/m, which a full 3e13 × L / 700 km pull reaches at L = 138 km).
    #[test]
    fn a_pull_along_the_margin_or_a_short_slab_starts_nothing() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let k = law2();
        for layout in [across, across_slab] {
            let side = start(&geo, layout, &k, 600.0);
            assert!(
                side.plates.len() == 3 && side.counters.propagation_starts == 0,
                "a pull along the margin started a propagation"
            );
        }
        let off = TectonicParams {
            margin_propagation: false,
            ..law2()
        };
        assert_eq!(start(&geo, in_line, &off, 300.0).plates.len(), 3);
        assert_eq!(start(&geo, in_line, &k, 60.0).plates.len(), 3);
        assert_eq!(start(&geo, in_line, &k, 120.0).plates.len(), 3);
        assert_eq!(start(&geo, in_line, &k, 160.0).plates.len(), 4);
    }

    /// The trench runs along the equator west of 0° (U's floor south, O north beyond U's slab),
    /// and C lies east of 0°, north-east of the line from 0° to the south-east at 45°: the trench
    /// meets the margin at 45°, where arm fl-law2's starts fired, the trench is straight to its
    /// end, and the slab pulls U's floor across the margin at cos 45°.
    fn oblique(la: f64, lo: f64) -> Cell {
        if !(-60.0..80.0).contains(&lo) || !(-40.0..30.0).contains(&la) {
            Cell::Rest
        } else if lo >= 0.0 {
            if la > -lo { Cell::Land } else { Cell::Floor }
        } else if la < 0.0 {
            Cell::Floor
        } else if la < 3.0 {
            Cell::Slab
        } else {
            Cell::Over
        }
    }

    /// Zhou et al. 2020's geometry only (ledger PLT-717): a trench meeting the margin at 45° with
    /// a full 700 km slab, whose pull across the margin (cos 45° of 3 × 10¹³ N/m) beats the
    /// margin's 5.9 × 10¹² N/m at 80 Myr, starts nothing, because the trench does not run on along
    /// the margin's strike (on the 16,000-sample lattice the two strikes read 42° apart, cos 0.74,
    /// against the gate's atan(1.5 / 4) = 20.6°; in [`in_line`] 0.6°); the same slab where the
    /// trench does run on ([`in_line`]) starts. A continent of fewer samples than a plate may hold
    /// starts nothing either.
    #[test]
    fn a_trench_meeting_the_margin_at_an_angle_starts_nothing() {
        use std::sync::atomic::Ordering;
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let k = law2();
        let refused = TALLY[T_START_GEOMETRY].load(Ordering::Relaxed);
        let st = start(&geo, oblique, &k, 700.0);
        assert!(
            st.plates.len() == 3 && st.counters.propagation_starts == 0,
            "a trench at 45° to the margin started a propagation"
        );
        assert!(TALLY[T_START_GEOMETRY].load(Ordering::Relaxed) > refused);
        assert_eq!(start(&geo, in_line, &k, 700.0).plates.len(), 4);
        // A narrow continent (12° by 30°, about 135 samples) under a plate's worth of samples.
        let big = TectonicParams {
            min_plate_area_frac: 200.0 / N as f64,
            ..law2()
        };
        let small = TALLY[T_START_SMALL_C].load(Ordering::Relaxed);
        assert_eq!(start(&geo, in_line_small, &big, 700.0).plates.len(), 3);
        assert!(TALLY[T_START_SMALL_C].load(Ordering::Relaxed) > small);
    }

    /// The advance: with U converging north on K' at about 40 mm/yr and the new margin's slab
    /// hanging, the second step's convergence passes the critical convergence (2 × 76 km), and
    /// the next stretch of K''s margin floor, cut off by a tear of its own, goes to U; C stays on
    /// K', no plate is cut into pieces, and the tip has moved east along the margin. Diverging,
    /// nothing advances.
    #[test]
    fn the_convergence_moves_the_tip_along_the_margin() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let k = law2();
        let e = env(&k);
        let mut st = build(&geo, in_line, 300.0);
        let pos = positions(&st, &geo);
        let mut world = WorldState::new(1, 16_384, 0.0);
        step(&mut st, &mut world, &geo, &pos, &e).unwrap();
        assert_eq!(st.plates.len(), 4);
        let speed = e.norm.omega_of_speed(40.0);
        let l0 = 20f64.to_radians();
        st.plates[U as usize].omega = [speed * l0.sin(), -speed * l0.cos(), 0.0];
        for p in [O, X, KP] {
            st.plates[p as usize].omega = [0.0; 3];
        }
        st.slabs.push(SlabTrack {
            under: U,
            over: KP,
            km: 300.0,
            ..SlabTrack::default()
        });
        let tip0 = DVec3::from_array(st.propagations[0].tip);
        let k_before = st.plates[KP as usize].samples;
        step(&mut st, &mut world, &geo, &pos, &e).unwrap();
        let pr = &st.propagations[0];
        assert_eq!(
            pr.advances, 0,
            "one step is short of the critical convergence"
        );
        assert!((pr.converged_km - 76.0).abs() < 8.0, "{}", pr.converged_km);
        step(&mut st, &mut world, &geo, &pos, &e).unwrap();
        assert_eq!(st.propagations.len(), 1, "the propagation goes on");
        let pr = &st.propagations[0];
        assert_eq!(pr.advances, 1, "advanced once");
        assert_eq!(pr.converged_km, 0.0);
        assert_eq!(st.counters.propagations, 2);
        assert_eq!(st.counters.propagation_stalls, 0);
        assert_eq!(st.plates.len(), 4, "no new plate");
        let k_after = st.plates[KP as usize].samples;
        // A strip about a spacing wide from the margin to U's southern boundary, 40° away.
        assert!(
            k_before - k_after >= 15,
            "K' gave U a strip of floor: {k_before} → {k_after}"
        );
        let tip1 = DVec3::from_array(pr.tip);
        let (_, lo0) = lat_lon(tip0);
        let (_, lo1) = lat_lon(tip1);
        eprintln!("tip from {lo0:.2}° to {lo1:.2}° E; K' {k_before} → {k_after} samples");
        let sp = e.norm.spacing_rad().to_degrees();
        assert!(
            lo1 - lo0 > 0.5 * sp && lo1 - lo0 < 2.5 * sp,
            "the tip moved about a spacing east along the margin"
        );
        for (i, s) in st.samples.iter().enumerate() {
            if s.kind == CONTINENTAL {
                assert_eq!(st.plate_of(i), KP);
            }
        }
        clean(&st, &geo);
        let mut away = st.clone();
        away.plates[U as usize].omega = [-speed * l0.sin(), speed * l0.cos(), 0.0];
        let before = away.counters.propagations;
        for _ in 0..4 {
            step(&mut away, &mut world, &geo, &pos, &e).unwrap();
        }
        assert_eq!(away.counters.propagations, before);
        assert_eq!(away.propagations.len(), 1);
    }

    /// U's floor beside the tip gone down (hidden, as where the trench meets the margin at an
    /// angle and U's slab takes the narrow wedge of floor at the tip down within a step): the tip
    /// still finds U's crust beside it, and the propagation carries on and advances under U's
    /// slab there as it does with that floor visible (PLT-656).
    #[test]
    fn a_tip_beside_u_s_slab_carries_on() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let k = law2();
        let e = env(&k);
        let mut st = build(&geo, in_line, 300.0);
        let pos = positions(&st, &geo);
        let mut world = WorldState::new(1, 16_384, 0.0);
        step(&mut st, &mut world, &geo, &pos, &e).unwrap();
        assert_eq!(st.plates.len(), 4);
        let speed = e.norm.omega_of_speed(40.0);
        let l0 = 20f64.to_radians();
        st.plates[U as usize].omega = [speed * l0.sin(), -speed * l0.cos(), 0.0];
        for p in [O, X, KP] {
            st.plates[p as usize].omega = [0.0; 3];
        }
        st.slabs.push(SlabTrack {
            under: U,
            over: KP,
            km: 300.0,
            ..SlabTrack::default()
        });
        let tip0 = DVec3::from_array(st.propagations[0].tip);
        let sp = e.norm.spacing_rad();
        let plate: Vec<u32> = (0..N).map(|i| st.plate_of(i)).collect();
        let mut hidden = 0;
        for (i, s) in st.samples.iter_mut().enumerate() {
            if plate[i] == U && !s.is_hidden() && pos[i].angle_between(tip0) < 2.5 * sp {
                s.flags |= HIDDEN;
                hidden += 1;
            }
        }
        assert!(hidden >= 5, "{hidden} samples of U's floor hidden");
        let nearest_visible_u = (0..N)
            .filter(|&i| plate[i] == U && !st.samples[i].is_hidden())
            .map(|i| pos[i].angle_between(tip0) / sp)
            .fold(f64::INFINITY, f64::min);
        assert!(nearest_visible_u > 2.4, "{nearest_visible_u}");
        let k_before = st.plates[KP as usize].samples;
        step(&mut st, &mut world, &geo, &pos, &e).unwrap();
        assert_eq!(st.propagations.len(), 1, "the propagation carries on");
        assert_eq!(st.propagations[0].advances, 0);
        step(&mut st, &mut world, &geo, &pos, &e).unwrap();
        assert_eq!(st.propagations.len(), 1, "the propagation goes on");
        let pr = &st.propagations[0];
        assert_eq!(pr.advances, 1, "advanced once");
        assert_eq!(st.counters.propagation_stalls, 0);
        let k_after = st.plates[KP as usize].samples;
        assert!(
            k_before - k_after >= 15,
            "K' gave U a strip of floor: {k_before} → {k_after}"
        );
        let (_, lo0) = lat_lon(tip0);
        let (_, lo1) = lat_lon(DVec3::from_array(pr.tip));
        let sp_deg = sp.to_degrees();
        eprintln!("tip from {lo0:.2}° to {lo1:.2}° E; K' {k_before} → {k_after} samples");
        assert!(
            lo1 - lo0 > 0.5 * sp_deg && lo1 - lo0 < 3.5 * sp_deg,
            "the tip moved east along the margin"
        );
        clean(&st, &geo);
    }

    /// K' absorbed (the burial at a resampling, `ridges::bury`, below the minimum plate area):
    /// the propagation ends with it, and no sample is left on a dead plate or cut off its plate.
    #[test]
    fn an_absorbed_sliver_ends_its_propagation_cleanly() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let k = law2();
        let mut st = build(&geo, in_line_small, 300.0);
        let pos = positions(&st, &geo);
        let mut world = WorldState::new(1, 16_384, 0.0);
        step(&mut st, &mut world, &geo, &pos, &env(&k)).unwrap();
        assert_eq!(st.propagations.len(), 1);
        let sizes: Vec<u32> = st.plates.iter().map(|p| p.samples).collect();
        eprintln!("plate sizes after the start: {sizes:?}");
        assert!(
            (0..KP as usize).all(|p| sizes[p] > sizes[KP as usize]),
            "K' is the smallest plate"
        );
        let big = TectonicParams {
            min_plate_area_frac: f64::from(sizes[KP as usize]) / N as f64 * 1.01,
            plate_fission: false,
            ..law2()
        };
        crate::plates::ridges::bury(
            &mut st,
            &mut world,
            Some(&geo),
            &pos,
            &env(&big),
            &BTreeMap::new(),
        )
        .unwrap();
        assert!(!st.plates[KP as usize].alive, "K' absorbed");
        assert!((0..KP as usize).all(|p| st.plates[p].alive));
        step(&mut st, &mut world, &geo, &pos, &env(&k)).unwrap();
        assert!(
            st.propagations.iter().all(|p| p.over != KP),
            "the propagation ended"
        );
        clean(&st, &geo);
    }
}
