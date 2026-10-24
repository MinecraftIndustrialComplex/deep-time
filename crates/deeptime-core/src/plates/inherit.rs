// SPDX-License-Identifier: LGPL-3.0-or-later
//! A boundary belongs to the lithosphere on its two sides (`tectonics.split_inheritance`, the
//! front lane, 2026-10-24; ledger PLT-534).
//!
//! The state of a plate boundary is kept per pair of plates: a contact with its polarity, onset and
//! collision (`PlateState::contacts`, matched by `subduction::resolve` on the pair), and a slab
//! (`PlateState::slabs`, keyed by the subducting and the overriding plate). When a plate splits (a
//! fission at a resampling, `ridges::split_cut_plates`; a rift, `rifting::rift`; a subduction
//! initiation's new plate, `rifting::initiate`), the parent keeps its id and the new pieces get new
//! ones, so without this every boundary the parent had stays keyed to the parent, wherever its
//! lithosphere went: the piece that holds a trench starts a new contact with no slab and no onset,
//! and a collision it carried parts once its front is gone (162 collision states parted within two
//! steps of a fission in arm fl-fission's 8 runs).
//!
//! [`inherit`] hands each such state to the pieces that border the other plate: a piece of the
//! split plate borders a plate where at least `subduction::MIN_CONTACT` of its samples (hidden
//! ones included: a slab's crust lies under the overriding plate) have a sample of that plate
//! within [`BORDER_SPACINGS`] lattice spacings. Every bordering pair keeps the contact with its
//! polarity, onset and confirmation (Stock & Lee 1994: "The fragmentation of a subducting plate
//! creates a new triple junction, two limbs of which are along the trench"; the Cocos and Rivera
//! plates, where "subduction of both plates has been nearly orthogonal to the trench"), and the
//! slab with its length (a length down the dip: each piece's own front pulls on it). The
//! collision goes with its front: the pair whose border lies nearest the collision's front
//! centroid keeps it, the others get the contact without one. A boundary no piece borders any
//! more stays with the parent and closes as before; a pair inside one split plate (the slab a
//! subduction initiation plants under its fracture) is not touched.
//!
//! The slab is one sheet along the trench, and its pull is counted per front edge
//! (`forces.rs`), so the pieces' fronts pull with the parent's force between them. Where the
//! trench lies along a split plate's border cannot be told at a split (a resampling has consumed
//! the hidden samples and reset the flags), so a piece bordering the other plate across a ridge or
//! a transform gets a copy too. A copy of a slab that was at a front is latent
//! (`SlabTrack::latent`): it pulls only at its piece's own fronts and never hangs along the
//! piece's border, so it adds nothing where its piece has no trench. A copy of a slab that hung
//! (no front for a step or more) hangs like it, along its own piece's stretch of the border, so
//! the pieces again pull with the parent's force between them.

use std::collections::{BTreeMap, BTreeSet};

use glam::DVec3;
use rayon::prelude::*;

use super::Env;
use super::geometry::Tree;
use super::state::{PlateState, SlabTrack};
use super::subduction::MIN_CONTACT;
use crate::world::WorldState;

/// How near a sample of the other plate must lie for a sample to stand on the boundary, in lattice
/// spacings: a lattice neighbour lies about 1.07 spacings away on a hexagonal lattice, the next
/// ring about 1.86, so this takes the neighbours of the lattice's irregular cells and not the ring
/// behind them. ledger: PLT-534 (game-choice)
pub const BORDER_SPACINGS: f64 = 1.5;

/// World globals (measurement; PLT-534, only while `split_inheritance` is on): contacts handed
/// to a piece of a split plate whole …
pub const MOVED_GLOBAL: &str = "plates/inherited_contacts_moved";
/// … contacts every other bordering piece took a copy of …
pub const COPIED_GLOBAL: &str = "plates/inherited_contacts_copied";
/// … collision states that went to a piece with their front …
pub const COLLISIONS_GLOBAL: &str = "plates/inherited_collisions";
/// … and slabs handed over whole, and copied.
pub const SLABS_MOVED_GLOBAL: &str = "plates/inherited_slabs_moved";
pub const SLABS_COPIED_GLOBAL: &str = "plates/inherited_slabs_copied";

/// A pair of plates, the subducting one first.
type Pair = (u32, u32);

/// What one call of [`inherit`] did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Inherited {
    pub contacts_moved: u32,
    pub contacts_copied: u32,
    pub collisions_moved: u32,
    pub slabs_moved: u32,
    pub slabs_copied: u32,
}

impl Inherited {
    /// Adds the counts to the world's globals.
    pub fn book(&self, world: &mut WorldState) {
        for (key, x) in [
            (MOVED_GLOBAL, self.contacts_moved),
            (COPIED_GLOBAL, self.contacts_copied),
            (COLLISIONS_GLOBAL, self.collisions_moved),
            (SLABS_MOVED_GLOBAL, self.slabs_moved),
            (SLABS_COPIED_GLOBAL, self.slabs_copied),
        ] {
            *world.globals.entry(key.to_owned()).or_insert(0.0) += f64::from(x);
        }
    }
}

/// Hands the boundary states of plates that just split to the pieces that border the other plate
/// (see the module docs). `splits` lists each parent with the plates split off it (the parent
/// keeps its id and is a piece too); `pos` are the samples' true positions now (mid-step, the
/// lattice's edges between plates are stale). Call it after the pieces' samples are relabelled.
pub fn inherit(
    state: &mut PlateState,
    pos: &[DVec3],
    env: &Env,
    splits: &[(u32, Vec<u32>)],
) -> Inherited {
    let mut out = Inherited::default();
    // Each split plate's family: plate → parent, parent → its pieces (parent first).
    let mut parent_of: BTreeMap<u32, u32> = BTreeMap::new();
    let mut pieces: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (parent, born) in splits {
        if born.is_empty() {
            continue;
        }
        let m = pieces.entry(*parent).or_insert_with(|| vec![*parent]);
        parent_of.insert(*parent, *parent);
        for &p in born {
            if !m.contains(&p) {
                m.push(p);
            }
            parent_of.insert(p, *parent);
        }
    }
    if pieces.is_empty() {
        return out;
    }
    let alive = |p: u32| state.plates.get(p as usize).is_some_and(|q| q.alive);
    let split = |p: u32| parent_of.contains_key(&p);
    let internal = |a: u32, b: u32| split(a) && parent_of.get(&a) == parent_of.get(&b);
    let family = |p: u32| -> Vec<u32> {
        parent_of
            .get(&p)
            .map_or_else(|| vec![p], |parent| pieces[parent].clone())
    };
    let concerns =
        |a: u32, b: u32| (split(a) || split(b)) && !internal(a, b) && alive(a) && alive(b);
    // The plates across a boundary of a split plate.
    let mut across: BTreeSet<u32> = BTreeSet::new();
    let pairs = state
        .contacts
        .iter()
        .map(|c| (c.under, c.over))
        .chain(state.slabs.iter().map(|s| (s.under, s.over)));
    for (a, b) in pairs {
        if !concerns(a, b) {
            continue;
        }
        if split(a) {
            across.extend(family(b));
        }
        if split(b) {
            across.extend(family(a));
        }
    }
    if across.is_empty() {
        return out;
    }
    // The boundary's samples: (piece of a split plate, plate across) → the piece's samples with a
    // sample of that plate within reach, in sample order.
    let n = state.samples.len();
    let plate_of: Vec<u32> = (0..n).map(|i| state.plate_of(i)).collect();
    let tree = Tree::over(pos, |i| across.contains(&plate_of[i]));
    if tree.index.is_empty() {
        return out;
    }
    let reach = BORDER_SPACINGS * env.norm.spacing_rad(); // ledger: PLT-534
    let near: Vec<(u32, Vec<u32>)> = (0..n as u32)
        .into_par_iter()
        .filter(|&i| split(plate_of[i as usize]))
        .filter_map(|i| {
            let p = plate_of[i as usize];
            let mut qs: Vec<u32> = tree
                .within(pos[i as usize], reach)
                .into_iter()
                .map(|(j, _)| plate_of[j as usize])
                .filter(|&q| q != p && !internal(p, q))
                .collect();
            qs.sort_unstable();
            qs.dedup();
            (!qs.is_empty()).then_some((i, qs))
        })
        .collect();
    let mut border: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    for (i, qs) in near {
        for q in qs {
            border.entry((plate_of[i as usize], q)).or_default().push(i);
        }
    }
    // A pair's boundary samples, counted on a split side.
    let side = |a: u32, b: u32| {
        let key = if split(a) { (a, b) } else { (b, a) };
        border.get(&key).map(Vec::as_slice).unwrap_or(&[])
    };
    // The pairs of pieces (subducting family × overriding family) that border each other.
    let bordering = |u: u32, o: u32| -> Vec<(u32, u32)> {
        let mut found = Vec::new();
        for a in family(u) {
            for b in family(o) {
                if alive(a) && alive(b) && side(a, b).len() >= MIN_CONTACT as usize {
                    found.push((a, b));
                }
            }
        }
        found
    };
    let mean = |s: &[u32]| -> DVec3 {
        s.iter()
            .map(|&i| pos[i as usize])
            .sum::<DVec3>()
            .normalize_or(DVec3::Z)
    };
    let key = |a: u32, b: u32| (a.min(b), a.max(b));

    // Contacts.
    let mut taken: BTreeSet<(u32, u32)> = state
        .contacts
        .iter()
        .map(|c| key(c.under, c.over))
        .collect();
    let mut owners: BTreeMap<Pair, (Pair, Vec<Pair>)> = BTreeMap::new();
    let mut copies = Vec::new();
    for k in 0..state.contacts.len() {
        let (u, o) = (state.contacts[k].under, state.contacts[k].over);
        if !concerns(u, o) {
            continue;
        }
        let found = bordering(u, o);
        if found.is_empty() {
            continue;
        }
        let c = &state.contacts[k];
        // The collision goes with its front; a contact without one stays with the pair nearest
        // where its overlap was last seen.
        let anchor = DVec3::from_array(c.collision.as_ref().map_or(c.centroid, |col| col.centroid))
            .normalize_or(DVec3::Z);
        let mut owner = *found
            .iter()
            .min_by(|&&a, &&b| {
                let d = |p: (u32, u32)| {
                    side(p.0, p.1)
                        .iter()
                        .map(|&i| anchor.angle_between(pos[i as usize]))
                        .fold(f64::INFINITY, f64::min)
                };
                d(a).total_cmp(&d(b))
                    .then_with(|| side(b.0, b.1).len().cmp(&side(a.0, a.1).len()))
                    .then_with(|| a.cmp(&b))
            })
            .expect("not empty");
        // The pair itself is free for a copy once its contact moves (a pair held by another
        // contact keeps that one, and this contact stays where it is).
        taken.remove(&key(u, o));
        if !taken.insert(key(owner.0, owner.1)) {
            owner = (u, o);
            taken.insert(key(u, o));
        }
        for &p in &found {
            if p == owner || !taken.insert(key(p.0, p.1)) {
                continue;
            }
            let mut copy = c.clone();
            copy.under = p.0;
            copy.over = p.1;
            copy.collision = None;
            copy.cont_in = 0;
            copy.andean = false;
            copy.max_uplift_mm_yr = 0.0;
            copy.centroid = mean(side(p.0, p.1)).to_array();
            copies.push(copy);
            out.contacts_copied += 1;
        }
        if owner != (u, o) {
            let c = &mut state.contacts[k];
            c.under = owner.0;
            c.over = owner.1;
            out.contacts_moved += 1;
            out.collisions_moved += u32::from(c.collision.is_some());
        }
        owners.insert((u, o), (owner, found));
    }
    state.contacts.extend(copies);

    // Slabs: with their contact's pieces, or, a slab with no contact (one that hangs, or one
    // planted at an initiation), with every bordering pair, the pair itself or the longest border
    // keeping it.
    let mut held: BTreeSet<(u32, u32)> = state.slabs.iter().map(|s| (s.under, s.over)).collect();
    let mut new_slabs: Vec<SlabTrack> = Vec::new();
    for s in &mut state.slabs {
        let (u, o) = (s.under, s.over);
        if !concerns(u, o) {
            continue;
        }
        let (owner, found) = match owners.get(&(u, o)) {
            Some(x) => x.clone(),
            None => {
                let found = bordering(u, o);
                let Some(&longest) = found.iter().max_by(|&&a, &&b| {
                    side(a.0, a.1)
                        .len()
                        .cmp(&side(b.0, b.1).len())
                        .then_with(|| b.cmp(&a))
                }) else {
                    continue;
                };
                let owner = if found.contains(&(u, o)) {
                    (u, o)
                } else {
                    longest
                };
                (owner, found)
            }
        };
        held.remove(&(u, o));
        let owner = if held.insert(owner) {
            owner
        } else {
            held.insert((u, o));
            (u, o)
        };
        for &p in &found {
            if p == owner || !held.insert(p) {
                continue;
            }
            // A slab with no step off a front since it was last fed (`idle_myr` 0) pulls only at
            // fronts, and so does its copy: at its piece's own (ledger: PLT-534).
            new_slabs.push(SlabTrack {
                under: p.0,
                over: p.1,
                latent: s.latent || s.idle_myr == 0.0,
                ..s.clone()
            });
            out.slabs_copied += 1;
        }
        if owner != (u, o) {
            s.under = owner.0;
            s.over = owner.1;
            out.slabs_moved += 1;
        }
    }
    state.slabs.extend(new_slabs);
    out
}

#[cfg(test)]
pub(in crate::plates) mod tests {
    use super::*;
    use crate::params::TectonicParams;
    use crate::plates::geometry::{Geo, positions};
    use crate::plates::normalise::Norm;
    use crate::plates::rifting::tests::{N, old_floor, one_plate};
    use crate::plates::state::{Belt, Collision, Contact, Group};

    fn env(k: &TectonicParams) -> Env<'_> {
        Env {
            k,
            norm: Norm::new(6370.0, 6371.0, N),
            dt_myr: 2.0,
            dt_yr: 2.0e6,
            gravity: 9.81,
            step: 7,
            t_myr: 14.0,
        }
    }

    pub(in crate::plates) fn collision_at(at: DVec3) -> Collision {
        Collision {
            onset_myr: 6.0,
            onset_step: 3,
            last_step: 7,
            last_myr: 14.0,
            shortening_km: 120.0,
            area_km2: 1.0e5,
            convergence_mm_yr: 30.0,
            onset_convergence_mm_yr: 40.0,
            peak_convergence_mm_yr: 40.0,
            later_convergence_mm_yr: None,
            front_km: 900.0,
            peak_front_km: 900.0,
            centroid: at.normalize().to_array(),
            normal: [0.0, 0.0, 1.0],
            continental_over: 0.0,
            stalled_since_myr: None,
            jam_km: None,
            belt: Belt::default(),
        }
    }

    pub(in crate::plates) fn contact(under: u32, over: u32, at: DVec3) -> Contact {
        Contact {
            under,
            over,
            onset_myr: 2.0,
            onset_step: 1,
            onset_event: None,
            onset_slab_age_myr: 80.0,
            onset_convergence_mm_yr: 50.0,
            setting: 0,
            last_step: 7,
            samples: 40,
            collision: None,
            max_uplift_mm_yr: 0.0,
            andean: false,
            centroid: at.normalize().to_array(),
            cont_in: 0,
            reversed: false,
        }
    }

    /// A new plate `id` holding every sample `on` selects (one unrotated group).
    pub(in crate::plates) fn add_plate(
        st: &mut PlateState,
        geo: &Geo,
        id: u32,
        on: impl Fn(DVec3) -> bool,
    ) {
        assert_eq!(st.plates.len(), id as usize);
        let mut p = st.plates[0].clone();
        p.id = id;
        st.plates.push(p);
        st.groups.push(Group {
            plate: id,
            rot: [0.0, 0.0, 0.0, 1.0],
        });
        let g = (st.groups.len() - 1) as u32;
        for (i, &x) in geo.points().iter().enumerate() {
            if on(x) {
                st.samples[i].group = g;
            }
        }
        let pos = positions(st, geo);
        st.recount(&pos);
    }

    fn slab(under: u32, over: u32, km: f64) -> SlabTrack {
        SlabTrack {
            under,
            over,
            km,
            blocked_myr: 0.0,
            idle_myr: 2.0,
            latent: false,
        }
    }

    pub(in crate::plates) fn find(st: &PlateState, under: u32, over: u32) -> Option<&Contact> {
        st.contacts
            .iter()
            .find(|c| c.under == under && c.over == over)
    }

    pub(in crate::plates) fn slab_of(st: &PlateState, under: u32, over: u32) -> Option<&SlabTrack> {
        st.slabs.iter().find(|s| s.under == under && s.over == over)
    }

    /// A split's pieces as [`inherit`] must leave them, judged by brute force: every piece with
    /// at least `MIN_CONTACT` samples within `BORDER_SPACINGS` spacings of a sample of `other`
    /// subducts under it with a contact and a slab of `km`, no other piece does, and the one
    /// collision is with the bordering piece nearest `front`. Returns the bordering pieces.
    pub(in crate::plates) fn assert_inherited(
        st: &PlateState,
        pos: &[DVec3],
        env: &Env,
        pieces: &[u32],
        other: u32,
        front: DVec3,
        km: f64,
    ) -> Vec<u32> {
        let n = st.samples.len();
        let cos = (BORDER_SPACINGS * env.norm.spacing_rad()).cos();
        let of: Vec<u32> = (0..n).map(|i| st.plate_of(i)).collect();
        let across: Vec<DVec3> = (0..n).filter(|&j| of[j] == other).map(|j| pos[j]).collect();
        let mut bordering = Vec::new();
        let mut nearest = (f64::INFINITY, u32::MAX);
        for &p in pieces {
            let border: Vec<usize> = (0..n)
                .filter(|&i| of[i] == p && across.iter().any(|q| pos[i].dot(*q) >= cos))
                .collect();
            let has = border.len() >= MIN_CONTACT as usize;
            assert_eq!(find(st, p, other).is_some(), has, "piece {p}'s contact");
            assert_eq!(
                slab_of(st, p, other).map(|s| s.km),
                has.then_some(km),
                "piece {p}'s slab"
            );
            if has {
                bordering.push(p);
                let d = border
                    .iter()
                    .map(|&i| front.angle_between(pos[i]))
                    .fold(f64::INFINITY, f64::min);
                if d < nearest.0 {
                    nearest = (d, p);
                }
            }
        }
        let with: Vec<u32> = st
            .contacts
            .iter()
            .filter(|c| c.collision.is_some())
            .map(|c| c.under)
            .collect();
        assert_eq!(with, vec![nearest.1], "the collision goes with its front");
        bordering
    }

    /// Stock & Lee 1994: "The fragmentation of a subducting plate creates a new triple junction,
    /// two limbs of which are along the trench", and of the Cocos and Rivera plates, "subduction
    /// of both plates has been nearly orthogonal to the trench". Plate 0 (z ≤ 0.5) subducts under
    /// plate 1 (z > 0.5) and under plate 2 (a cap on the -x side of the equator), each with a
    /// slab; the contact with plate 1 is in a collision whose front is at +x. Plate 0 splits
    /// along x = 0 into itself (+x) and plate 3 (-x): both halves keep subducting under plate 1
    /// with their own copy of the contact and the slab, the onset, polarity and confirmation kept;
    /// the collision stays with the half at its front; the trench under plate 2, which only the
    /// -x half borders, goes to plate 3 whole. With the front at -x the collision goes to plate 3
    /// instead.
    #[test]
    fn both_halves_of_a_split_subducting_plate_keep_the_trench_and_the_collision_goes_with_its_front()
     {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let k = TectonicParams::default();
        let cap2 = DVec3::new(-1.0, 0.0, -0.2).normalize();
        let on2 = move |p: DVec3| p.dot(cap2) > 0.3f64.cos();
        let run = |front: DVec3| {
            let mut st = one_plate(&geo, 5, |_| old_floor());
            add_plate(&mut st, &geo, 1, |p| p.z > 0.5);
            add_plate(&mut st, &geo, 2, |p| p.z <= 0.5 && on2(p));
            let mut c = contact(0, 1, DVec3::new(0.0, 0.8, 0.5));
            c.collision = Some(collision_at(front));
            c.onset_event = Some(deeptime_format::EventId(11));
            st.contacts.push(c);
            st.contacts.push(contact(0, 2, cap2));
            st.slabs.push(slab(0, 1, 300.0));
            st.slabs.push(slab(0, 2, 150.0));
            // The split: plate 0's -x half becomes plate 3.
            let mut p3 = st.plates[0].clone();
            p3.id = 3;
            p3.parent = Some(0);
            st.plates.push(p3);
            st.groups.push(Group {
                plate: 3,
                rot: [0.0, 0.0, 0.0, 1.0],
            });
            let pts = geo.points();
            for (i, p) in pts.iter().enumerate() {
                if st.plate_of(i) == 0 && p.x < 0.0 {
                    st.samples[i].group = 3;
                }
            }
            let pos = positions(&st, &geo);
            st.recount(&pos);
            let got = inherit(&mut st, &pos, &env(&k), &[(0, vec![3])]);
            (st, got)
        };

        let (st, got) = run(DVec3::new(0.87, 0.0, 0.5));
        for under in [0, 3] {
            let c = find(&st, under, 1).expect("both halves subduct under plate 1");
            assert_eq!(c.onset_myr, 2.0);
            assert_eq!(c.onset_event, Some(deeptime_format::EventId(11)));
            assert_eq!(slab_of(&st, under, 1).map(|s| s.km), Some(300.0));
        }
        assert!(
            find(&st, 0, 1).unwrap().collision.is_some(),
            "the front is at +x"
        );
        assert!(find(&st, 3, 1).unwrap().collision.is_none());
        assert!(find(&st, 0, 2).is_none() && slab_of(&st, 0, 2).is_none());
        assert_eq!(find(&st, 3, 2).map(|c| c.onset_slab_age_myr), Some(80.0));
        assert_eq!(slab_of(&st, 3, 2).map(|s| s.km), Some(150.0));
        assert_eq!(st.contacts.len(), 3);
        assert_eq!(st.slabs.len(), 3);
        assert_eq!(
            got,
            Inherited {
                contacts_moved: 1,
                contacts_copied: 1,
                collisions_moved: 0,
                slabs_moved: 1,
                slabs_copied: 1,
            }
        );

        let (st, got) = run(DVec3::new(-0.87, 0.0, 0.5));
        assert!(
            find(&st, 3, 1).unwrap().collision.is_some(),
            "the front is at -x"
        );
        assert!(find(&st, 0, 1).unwrap().collision.is_none());
        assert_eq!(got.collisions_moved, 1);
        assert_eq!(got.contacts_moved, 2);
    }

    /// The overriding plate splits (a back-arc basin opens behind the arc): plate 1 (z > 0.5)
    /// overrides plate 0 and splits into its polar part (z > 0.8, keeping the id) and the arc's
    /// strip beside the trench (plate 3): the trench, its slab and its collision go to the strip,
    /// and the polar part, which no longer borders plate 0, keeps nothing. A plate on its own
    /// that did not split is not touched, nor is a pair inside the split plate.
    #[test]
    fn the_strip_of_a_split_overriding_plate_at_the_trench_takes_it() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let k = TectonicParams::default();
        let mut st = one_plate(&geo, 5, |_| old_floor());
        add_plate(&mut st, &geo, 1, |p| p.z > 0.5);
        add_plate(&mut st, &geo, 2, |p| p.z < -0.9);
        let mut c = contact(0, 1, DVec3::new(0.87, 0.0, 0.5));
        c.collision = Some(collision_at(DVec3::new(0.87, 0.0, 0.5)));
        st.contacts.push(c);
        st.contacts.push(contact(2, 0, DVec3::new(0.0, 0.0, -1.0)));
        st.slabs.push(slab(0, 1, 300.0));
        st.slabs.push(slab(2, 0, 100.0));
        let mut p3 = st.plates[1].clone();
        p3.id = 3;
        p3.parent = Some(1);
        st.plates.push(p3);
        st.groups.push(Group {
            plate: 3,
            rot: [0.0, 0.0, 0.0, 1.0],
        });
        let pts = geo.points();
        for (i, p) in pts.iter().enumerate() {
            if st.plate_of(i) == 1 && p.z <= 0.8 {
                st.samples[i].group = 3;
            }
        }
        // A slab inside the split plate (an initiation's).
        st.slabs.push(slab(3, 1, 50.0));
        let pos = positions(&st, &geo);
        st.recount(&pos);
        let before_other = (find(&st, 2, 0).cloned(), slab_of(&st, 2, 0).cloned());
        let got = inherit(&mut st, &pos, &env(&k), &[(1, vec![3])]);
        let c = find(&st, 0, 3).expect("the arc's strip keeps the trench");
        assert!(c.collision.is_some());
        assert!(
            find(&st, 0, 1).is_none(),
            "the polar part does not border plate 0"
        );
        assert_eq!(slab_of(&st, 0, 3).map(|s| s.km), Some(300.0));
        assert!(slab_of(&st, 0, 1).is_none());
        assert_eq!(slab_of(&st, 3, 1).map(|s| s.km), Some(50.0));
        assert_eq!(
            (find(&st, 2, 0).cloned(), slab_of(&st, 2, 0).cloned()),
            before_other
        );
        assert_eq!(
            got,
            Inherited {
                contacts_moved: 1,
                contacts_copied: 0,
                collisions_moved: 1,
                slabs_moved: 1,
                slabs_copied: 0,
            }
        );
    }

    /// A boundary no piece borders any more (the overlap was elsewhere, or is gone) stays with
    /// the parent, to close as it did before; a split that left a plate whole does nothing.
    #[test]
    fn a_boundary_no_piece_borders_stays_with_the_parent() {
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let k = TectonicParams::default();
        let mut st = one_plate(&geo, 5, |_| old_floor());
        add_plate(&mut st, &geo, 1, |p| p.z > 0.9);
        add_plate(&mut st, &geo, 2, |p| p.z < -0.9);
        // Plate 1 and plate 2 never touch: their contact is a stale one.
        st.contacts.push(contact(2, 1, DVec3::Z));
        st.slabs.push(slab(2, 1, 80.0));
        let mut p3 = st.plates[1].clone();
        p3.id = 3;
        st.plates.push(p3);
        st.groups.push(Group {
            plate: 3,
            rot: [0.0, 0.0, 0.0, 1.0],
        });
        let pts = geo.points();
        for (i, p) in pts.iter().enumerate() {
            if st.plate_of(i) == 1 && p.x < 0.0 {
                st.samples[i].group = 3;
            }
        }
        let pos = positions(&st, &geo);
        st.recount(&pos);
        let before = (st.contacts.clone(), st.slabs.clone());
        let got = inherit(&mut st, &pos, &env(&k), &[(1, vec![3])]);
        assert_eq!(got, Inherited::default());
        assert_eq!((st.contacts.clone(), st.slabs.clone()), before);
        let got = inherit(&mut st, &pos, &env(&k), &[(1, vec![])]);
        assert_eq!(got, Inherited::default());
    }

    /// A split hands the slab's pull to its pieces and adds none: the slab is one sheet along the
    /// trench and pulls per metre of its front (`forces.rs`), so the pull's torque summed over
    /// the pieces of a split subducting plate, and the suction's over those of a split overriding
    /// plate, equal the parent's before the split. Plate 1 (x ≥ 0) turns about the z axis at 50
    /// mm/yr: into plate 0 on the y > 0 half of their border (a trench with a full slab, fed this
    /// step) and away from it on the y < 0 half (a ridge). Each plate splits across the trench
    /// (z = 0: both pieces hold a stretch of it) and along it (y < -0.3 is the new piece, which
    /// borders the other plate only across the ridge, away from the trench's ends at the poles,
    /// and its copy of the slab pulls nothing). A slab that hung with no front hangs on along each
    /// piece's own stretch of the border, with the parent's pull between them. ledger: PLT-534
    #[test]
    fn a_split_plates_pieces_pull_with_the_parents_slab_force() {
        use crate::plates::forces::slab_torques;
        let geo = Geo::build(N, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let k = TectonicParams::default();
        let e = env(&k);
        let w = DVec3::Z * e.norm.omega_of_speed(50.0);
        // (the plate that splits, the new piece's side, the slab hung, the new piece pulls)
        type Case = (u32, fn(DVec3) -> bool, bool, bool);
        let cases: [Case; 5] = [
            (1, |p| p.z < 0.0, false, true),
            (0, |p| p.z < 0.0, false, true),
            (0, |p| p.y < -0.3, false, false),
            (1, |p| p.y < -0.3, false, false),
            (1, |p| p.y < -0.3, true, true),
        ];
        for (i, (parent, side, hung, pulls)) in cases.into_iter().enumerate() {
            let mut st = one_plate(&geo, 5, |_| old_floor());
            add_plate(&mut st, &geo, 1, |p| p.x >= 0.0);
            st.plates[1].omega = w.to_array();
            let mut c = contact(1, 0, DVec3::Y);
            let mut s = slab(1, 0, k.slab_depth_km);
            if hung {
                c.last_step = e.step - 1;
            } else {
                s.idle_myr = 0.0;
            }
            st.contacts.push(c);
            st.slabs.push(s);
            let pos = positions(&st, &geo);
            let before = slab_torques(&st, &geo, &pos, &e);

            let mut p2 = st.plates[parent as usize].clone();
            p2.id = 2;
            p2.parent = Some(parent);
            st.plates.push(p2);
            st.groups.push(Group {
                plate: 2,
                rot: [0.0, 0.0, 0.0, 1.0],
            });
            for (j, &x) in geo.points().iter().enumerate() {
                if st.plate_of(j) == parent && side(x) {
                    st.samples[j].group = 2;
                }
            }
            let pos = positions(&st, &geo);
            st.recount(&pos);
            inherit(&mut st, &pos, &e, &[(parent, vec![2])]);
            let after = slab_torques(&st, &geo, &pos, &e);

            let (under, over) = if parent == 1 {
                (vec![1, 2], vec![0])
            } else {
                (vec![1], vec![0, 2])
            };
            let pull: DVec3 = under.iter().map(|&p| after[p].0).sum();
            let suck: DVec3 = over.iter().map(|&p| after[p].1).sum();
            assert!(before[1].0.length() > 0.0, "case {i}: the slab pulls");
            for (got, want, what) in [(pull, before[1].0, "pull"), (suck, before[0].1, "suction")] {
                assert!(
                    (got - want).length() <= 1e-9 * want.length(),
                    "case {i}: the pieces' {what} {got} against the parent's {want}"
                );
            }
            for &p in &over {
                assert_eq!(after[p].0, DVec3::ZERO, "case {i}: plate {p} is pulled");
            }
            for &p in &under {
                assert_eq!(after[p].1, DVec3::ZERO, "case {i}: plate {p} has suction");
            }
            let piece = if parent == 1 { after[2].0 } else { after[2].1 };
            assert_eq!(
                piece != DVec3::ZERO,
                pulls,
                "case {i}: the new piece's share {piece}"
            );
        }
    }
}
