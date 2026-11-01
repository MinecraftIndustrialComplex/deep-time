// SPDX-License-Identifier: LGPL-3.0-or-later
//! Plate outlines finer than the lattice, carried through the resampling
//! (`tectonics.level_set_membership`, the front lane, 2026-10-24; ledger PLT-536).
//!
//! A resampling gives every new lattice point the group whose old triangle holds it in the
//! group's own frame (`ridges::footprint`). Without more than the old samples' groups, the old
//! rule takes the group of the corner with the largest barycentric weight, which puts the outline
//! halfway between an old sample of the group and one of another: anywhere within half a
//! spacing of where it was, at random, since every lattice is turned at random. Nothing pulls it
//! back, so a boundary that does not move walks: a static straight boundary's samples spread
//! from 0.44 to 1.73 spacings about it over 40 resamplings, and its lattice edges per spacing of
//! length rise from 2.25 to 4.65 (`plates::tests::how_resamplings_roughen_a_boundary`, plates.md
//! "What still roughens the plates").
//!
//! The level set (Osher & Sethian 1988; the signed distance function of Sussman, Smereka & Osher
//! 1994, "the interface ... is described as the zero level set of a smooth function") keeps the
//! outline between the samples: every sample carries ψ, the sine of its angular distance to its
//! group's outline (`LevelSet::phi`, positive, signed by which side of the outline a group sees
//! it on). Inside one old triangle, each group present weighs its own corners' ψ with the
//! point's barycentric weights against the others' ([`margin`]); the group with the largest
//! weighed ψ holds the point, and the outline is where two weigh the same: the zero line of
//! the interpolated signed distance. The weights are gnomonic (`SphereMesh::barycentric`: the
//! point where the ray towards q meets the triangle's plane), so a signed distance linear in
//! the position, ψ = n·p for the great circle with pole n, interpolates exactly, and a
//! great-circle outline lands where it was whatever the lattice. After the resampling and
//! `ridges::bury`, [`reinit`] sets every sample's ψ again from the new outline (Sussman et al.
//! 1994's reinitialisation): the zero crossings on each edge between groups, joined inside each
//! triangle into great-circle arcs, and each sample's distance to the nearest arc of its group.
//! A plate's outline still moves where the physics moves it (a ridge's new crust, a trench's
//! overriding plate on top, an accretion), and only there.
//!
//! The same outline decides what goes under (`tectonics.outline_cover`, the front lane's step 8,
//! 2026-10-24; ledger PLT-553): [`cover`] says a sample lies on another group's surface where the
//! resampling would give that group the point (a triangle of the group's own lattice, in its own
//! frame, where the group weighs the most by [`margin`]), so a subducting sample goes under the
//! step it crosses the overriding plate's outline, not once a whole triangle of that plate's
//! lattice holds it. The distances are [`distances_now`]: the kept ψ where the neighbours'
//! groups are as they were at the last resampling, half a spacing where not, which is the
//! lattice midpoint rule.
//!
//! # The crust's class (the arc-stock lane, 2026-11-01; ledger PLT-852)
//! A new sample's kind and its class (ocean floor, hyper-extended continental crust, other
//! continental crust: [`class_of`], the classes `ridges::interpolate` blends within, PLT-663)
//! were its nearest old corner's (PLT-664), the rule the level set replaced for the groups: a
//! margin inside one plate (continental crust beside its own plate's ocean floor, and the 10 km
//! line PLT-661's margins end at) walked by up to half a spacing at every resampling, its static
//! samples spreading from 0.44 to 1.64 spacings and its lattice edges per spacing rising from
//! 2.26 to 4.56 in 40 resamplings (`plates::tests::how_resamplings_roughen_a_margin`), as a
//! plate's outline did. The kind and the class each have a level set of their own
//! ([`LevelSet::kind_phi`], [`LevelSet::class_phi`]): each sample's distance from the outline
//! between its kind (class) and another of its group (the group's own outline is not one,
//! [`distances_where`]), kept as the group's is ([`old_kind_distances`],
//! [`old_class_distances`]). The level set alone (the label whose corners weigh the largest
//! Σ w ψ holds the point) drifts a curved outline towards its centre and eats crust one spacing
//! wide, as the corner vote does (`plates::tests::thin_crust_through_resamplings`), so each piece
//! keeps its old count of samples, taking the new points its interpolated signed distance ranks
//! most inside (`ridges::conserved_classes`): each continental piece against the ocean floor,
//! then each piece of crust that is not hyper-extended within the continental crust, the
//! hyper-extended crust taking the rest. An outline stays where the level set puts it, shifted
//! by one distance all round to keep the piece's area.

use glam::{DQuat, DVec3};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::geometry::{Cover, Geo};
use super::state::{OCEANIC, PlateState, Sample};

/// Each sample's distance to its group's outline, as it stood at the last resampling.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LevelSet {
    /// The sine of the angular distance from each sample to the outline of its group (≥ 0, at
    /// most [`CAP_SPACINGS`] spacings).
    pub phi: Vec<f32>,
    /// Each sample's group then. Only which samples shared a group is read: a sample whose
    /// neighbours' membership changed since ([`old_distances`]) has no outline distance to keep.
    pub member: Vec<u32>,
    /// The sine of the angular distance from each sample to the outline of its crust's kind
    /// inside its group, continental crust against ocean floor ([`kind_of`]; ≥ 0, at most
    /// [`CAP_SPACINGS`] spacings). Empty where no reinitialisation set it (ledger: PLT-852).
    #[serde(default)]
    pub kind_phi: Vec<f32>,
    /// Each sample's kind label then ([`kind_of`]), read as `member` is.
    #[serde(default)]
    pub kind_member: Vec<u32>,
    /// The same for its crust's class ([`class_of`]).
    #[serde(default)]
    pub class_phi: Vec<f32>,
    /// Each sample's class label then ([`class_of`]), read as `member` is.
    #[serde(default)]
    pub class_member: Vec<u32>,
}

/// A sample's kind label for the kind level set (ledger: PLT-852): its group's ocean floor or
/// continental crust.
pub fn kind_of(s: &Sample) -> u32 {
    s.group * 2 + u32::from(s.kind != OCEANIC)
}

/// A sample's class label for the class level set (ledger: PLT-852): its group's three classes,
/// ocean floor, hyper-extended continental crust (`transition::hyper_extended`, under 10 km of
/// crystalline crust of its own) and other continental crust, the classes a resampling blends
/// within (PLT-663). Two samples share an outline's side when they share a label.
pub fn class_of(s: &Sample) -> u32 {
    let class = if s.kind == OCEANIC {
        0
    } else if super::transition::hyper_extended(s) {
        1
    } else {
        2
    };
    s.group * 3 + class
}

/// The distance kept, lattice spacings (numerical, ledger PLT-536): only a sample with a
/// neighbour of another group has its distance read at the next resampling (its triangles are
/// the only ones with two groups), and it lies within one edge of the outline; farther samples
/// keep the cap.
pub const CAP_SPACINGS: f64 = 2.0;
/// The distance a sample gets when it has none to keep (a boundary that a rift, a fission
/// elsewhere, a suture or a twin group made since the last resampling, or a run's first
/// resampling), lattice spacings (numerical, ledger PLT-536): half a spacing on every corner of
/// a triangle weighs the groups by their barycentric weights alone, so such a triangle is split
/// as the old rule splits it, at the midpoints of its edges.
pub const STALE_SPACINGS: f64 = 0.5;

/// Every old sample's distance to its group's outline for a resampling: `ls` the level set the
/// last reinitialisation left, `groups` each sample's group now, on the lattice `geo` the
/// samples stand on (spacing `spacing` rad). A sample keeps the distance it was given unless one
/// of its lattice neighbours shares a group with it now and did not then, or the reverse; it gets
/// [`STALE_SPACINGS`] then, and every sample gets it when there is no level set yet.
pub fn old_distances(ls: Option<&LevelSet>, groups: &[u32], geo: &Geo, spacing: f64) -> Vec<f64> {
    kept_distances(
        ls.map(|l| (&l.phi[..], &l.member[..])),
        groups,
        geo,
        STALE_SPACINGS * spacing,
    )
}

/// [`old_distances`] for the kind level set (ledger: PLT-852): `kinds` each sample's kind label
/// now ([`kind_of`]), kept as [`old_class_distances`] keeps a class's.
pub fn old_kind_distances(
    ls: Option<&LevelSet>,
    kinds: &[u32],
    geo: &Geo,
    spacing: f64,
) -> Vec<f64> {
    kept_distances(
        ls.map(|l| (&l.kind_phi[..], &l.kind_member[..])),
        kinds,
        geo,
        STALE_SPACINGS * spacing,
    )
}

/// [`old_distances`] for the class level set (ledger: PLT-852): `classes` each sample's class
/// label now ([`class_of`]); a sample keeps its class distance unless the labels about it changed
/// since the reinitialisation (a margin PLT-661 turned, a column across the 10 km line, a rift);
/// it gets [`STALE_SPACINGS`] then, as a group's does.
pub fn old_class_distances(
    ls: Option<&LevelSet>,
    classes: &[u32],
    geo: &Geo,
    spacing: f64,
) -> Vec<f64> {
    kept_distances(
        ls.map(|l| (&l.class_phi[..], &l.class_member[..])),
        classes,
        geo,
        STALE_SPACINGS * spacing,
    )
}

/// The distances `prev` (each sample's distance and label at the last reinitialisation) keeps
/// for the labels `labels` now: `d0` where a neighbour's sharing of a label with the sample
/// changed, and everywhere when there is none.
fn kept_distances(prev: Option<(&[f32], &[u32])>, labels: &[u32], geo: &Geo, d0: f64) -> Vec<f64> {
    let n = labels.len();
    let Some((phi, member)) = prev.filter(|(p, m)| p.len() == n && m.len() == n) else {
        return vec![d0; n];
    };
    (0..n)
        .into_par_iter()
        .map(|i| {
            let (gi, mi) = (labels[i], member[i]);
            let stale = geo.edges(i as u32).any(|(j, _)| {
                let j = j as usize;
                (member[j] == mi) != (labels[j] == gi)
            });
            if stale { d0 } else { f64::from(phi[i]) }
        })
        .collect()
}

/// Whether, and by how much, group `g` holds the point with gnomonic barycentric weights `w` in
/// the old triangle whose corners stand at `corners` (unit vectors), belong to `groups` and lie
/// `phi` from their groups' outlines: the point's sine distance from the outline (positive
/// inside, negative outside). Each group present weighs its own corners, A = Σ w ψ; the group
/// holds the point when its A is the largest, and the distance is (A_g − max A_other) / |Σ w v|:
/// with ψ the signed distance on each side, the interpolated signed distance Σ w s ψ / |Σ w v|
/// (s = +1 on the group's corners, −1 on the others') on the boundary between two groups, exact
/// for a great-circle outline. With one group only, its interpolated distance, positive when it
/// is `g` and negative when it is not: a point whose triangle has no corner of `g` lies outside
/// `g`'s outline by the others' interpolated distance, which is its distance from `g`'s outline
/// where their outline is the boundary with `g`.
pub fn margin(groups: [u32; 3], w: [f64; 3], phi: [f64; 3], corners: [DVec3; 3], g: u32) -> f64 {
    let mut own = 0.0;
    let mut others: [(u32, f64); 3] = [(u32::MAX, 0.0); 3];
    for k in 0..3 {
        let a = w[k] * phi[k];
        if groups[k] == g {
            own += a;
        } else if let Some(o) = others.iter_mut().find(|o| o.0 == groups[k]) {
            o.1 += a;
        } else if let Some(o) = others.iter_mut().find(|o| o.0 == u32::MAX) {
            *o = (groups[k], a);
        }
    }
    let rival = others
        .iter()
        .filter(|o| o.0 != u32::MAX)
        .map(|o| o.1)
        .reduce(f64::max);
    let len = (corners[0] * w[0] + corners[1] * w[1] + corners[2] * w[2])
        .length()
        .max(1e-12);
    match rival {
        Some(r) => (own - r) / len,
        None => own / len,
    }
}

/// Every sample's distance from its group's outline between resamplings, for the overlaps
/// ([`cover`]): [`old_distances`] with the groups as they stand now, on the lattice `geo` the
/// samples stand on (spacing `spacing` rad).
pub fn distances_now(state: &PlateState, geo: &Geo, spacing: f64) -> Vec<f64> {
    let groups: Vec<u32> = state.samples.iter().map(|s| s.group).collect();
    old_distances(state.level_set.as_ref(), &groups, geo, spacing)
}

/// Where the mantle-frame direction `q` falls inside group `group`'s outline
/// (`tectonics.outline_cover`, ledger PLT-553): the lattice triangle holding `q` in the group's
/// frame (`rot`, located from a triangle of `hint`), when the group holds `q` there by [`margin`]
/// with the corners' distances `phi` ([`distances_now`]), the rule the resampling gives a new
/// point its group by (`ridges::footprint_level_set`). A triangle whose three corners are the
/// group's holds `q` whole, as [`geometry::cover`] has it; one with corners of other groups holds
/// it on the group's side of the zero line, where [`geometry::cover`] holds nothing. The weights
/// are the group's own corners', summing to 1, so [`Cover::major`] is one of its samples. `None`
/// outside the outline or with no corner of the group.
///
/// [`geometry::cover`]: super::geometry::cover
pub fn cover(
    geo: &Geo,
    state: &PlateState,
    phi: &[f64],
    group: u32,
    rot: DQuat,
    q: DVec3,
    hint: u32,
) -> Option<Cover> {
    let local = rot.inverse() * q;
    let h = geo.mesh.vertex_triangle(hint).unwrap_or(0);
    let t = geo.mesh.locate(local, h);
    let tri = geo.mesh.triangles()[t as usize];
    let groups = tri.map(|v| state.samples[v as usize].group);
    let own = groups.map(|g| g == group);
    if !own.contains(&true) {
        return None;
    }
    let mut w = geo.mesh.barycentric(t, local);
    if own.contains(&false) {
        let pts = geo.points();
        let d = margin(
            groups,
            w,
            tri.map(|v| phi[v as usize]),
            tri.map(|v| pts[v as usize]),
            group,
        );
        let sum: f64 = (0..3).filter(|&k| own[k]).map(|k| w[k]).sum();
        if d <= 0.0 || sum <= 0.0 {
            return None;
        }
        for k in 0..3 {
            w[k] = if own[k] { w[k] / sum } else { 0.0 };
        }
    }
    Some(Cover { t, tri, w })
}

/// The sine of the angular distance from `p` to the great-circle arc from `a` to `b` (the
/// shorter one; unit vectors).
pub fn arc_distance(p: DVec3, a: DVec3, b: DVec3) -> f64 {
    let n = a.cross(b);
    let len = n.length();
    if len < 1e-15 {
        return p.cross(a).length();
    }
    let n = n / len;
    let s = p.dot(n);
    let proj = p - n * s;
    if a.cross(proj).dot(n) >= 0.0 && proj.cross(b).dot(n) >= 0.0 && proj.dot(a + b) > 0.0 {
        s.abs()
    } else {
        p.cross(a).length().min(p.cross(b).length())
    }
}

/// Where the outline crosses the edge from `u` (distance `du`) to `v` (`dv`) of two groups: the
/// zero of the signed distance interpolated along it, t = du / (du + dv) (the edge's midpoint
/// when both are 0).
fn crossing(u: DVec3, du: f64, v: DVec3, dv: f64) -> DVec3 {
    let s = du + dv;
    let t = if s > 0.0 { du / s } else { 0.5 };
    (u * (1.0 - t) + v * t).normalize()
}

/// One piece of outline inside a lattice triangle, between the groups `pair`.
struct Arc {
    a: DVec3,
    b: DVec3,
    pair: (u32, u32),
}

/// The outline the groups `groups` and distances `phi` give on `geo`'s lattice: inside every
/// triangle whose corners lie in two groups, the great-circle arc between the zero crossings on
/// its two edges between the groups; inside one with three, the three arcs from the crossings to
/// the point where the three weigh the same ([`margin`]: w_k ∝ 1/ψ_k, or the corner whose ψ is
/// 0). Returns the arcs and, for every sample, the arcs of its triangles (CSR offsets, indices).
fn outline(groups: &[u32], phi: &[f64], geo: &Geo) -> (Vec<Arc>, Vec<u32>, Vec<u32>) {
    let pts = geo.points();
    let tris = geo.mesh.triangles();
    let mut arcs: Vec<Arc> = Vec::new();
    let mut owner: Vec<[u32; 3]> = Vec::new();
    for tri in tris {
        let g = tri.map(|v| groups[v as usize]);
        if g[0] == g[1] && g[1] == g[2] {
            continue;
        }
        let p = tri.map(|v| pts[v as usize]);
        let d = tri.map(|v| phi[v as usize]);
        let cross = |i: usize, j: usize| crossing(p[i], d[i], p[j], d[j]);
        let mut push = |a: DVec3, b: DVec3, x: u32, y: u32| {
            arcs.push(Arc {
                a,
                b,
                pair: (x.min(y), x.max(y)),
            });
            owner.push(*tri);
        };
        if g[0] != g[1] && g[1] != g[2] && g[0] != g[2] {
            let triple = match (0..3).find(|&k| d[k] <= 0.0) {
                Some(k) => p[k],
                None => (0..3)
                    .map(|k| p[k] / d[k])
                    .sum::<DVec3>()
                    .normalize_or(p[0]),
            };
            for (i, j) in [(0, 1), (1, 2), (2, 0)] {
                push(cross(i, j), triple, g[i], g[j]);
            }
        } else {
            // The corner alone in its group, and the two others.
            let k = (0..3)
                .find(|&k| g[k] != g[(k + 1) % 3] && g[k] != g[(k + 2) % 3])
                .unwrap_or(0);
            let (i, j) = ((k + 1) % 3, (k + 2) % 3);
            push(cross(k, i), cross(k, j), g[k], g[i]);
        }
    }
    let n = groups.len();
    let mut count = vec![0u32; n + 1];
    for o in &owner {
        for &v in o {
            count[v as usize + 1] += 1;
        }
    }
    for i in 0..n {
        count[i + 1] += count[i];
    }
    let offsets = count.clone();
    let mut fill = count;
    let mut index = vec![0u32; offsets[n] as usize];
    for (a, o) in owner.iter().enumerate() {
        for &v in o {
            index[fill[v as usize] as usize] = a as u32;
            fill[v as usize] += 1;
        }
    }
    (arcs, offsets, index)
}

/// The distances `groups` and `phi` give every sample of `geo`'s lattice from its group's
/// outline ([`outline`]): the nearest arc of its group among its own triangles and its lattice
/// neighbours', capped at `cap` (the cap where none is that near).
pub fn distances(groups: &[u32], phi: &[f64], geo: &Geo, cap: f64) -> Vec<f32> {
    distances_where(groups, phi, geo, cap, |_, _| true)
}

/// [`distances`] from the arcs between the label pairs `keep` takes only (the class level set's:
/// two classes of one group, not the group's own outline; ledger PLT-852).
pub fn distances_where(
    groups: &[u32],
    phi: &[f64],
    geo: &Geo,
    cap: f64,
    keep: impl Fn(u32, u32) -> bool + Sync,
) -> Vec<f32> {
    let (arcs, offsets, index) = outline(groups, phi, geo);
    let pts = geo.points();
    (0..groups.len())
        .into_par_iter()
        .map(|i| {
            let g = groups[i];
            let p = pts[i];
            let mut best = cap;
            let mut scan = |v: usize| {
                for &a in &index[offsets[v] as usize..offsets[v + 1] as usize] {
                    let arc = &arcs[a as usize];
                    if (arc.pair.0 == g || arc.pair.1 == g) && keep(arc.pair.0, arc.pair.1) {
                        best = best.min(arc_distance(p, arc.a, arc.b));
                    }
                }
            };
            scan(i);
            for (j, _) in geo.edges(i as u32) {
                scan(j as usize);
            }
            best as f32
        })
        .collect()
}

/// Reinitialisation (Sussman et al. 1994): after a resampling and `ridges::bury` (fission,
/// capture and the absorptions under the floor), every sample's distance from its group's
/// outline on the new lattice `geo`, from the distances the resampling gave the new samples
/// (`PlateState::level_set`, set by `ridges::resample`; the stale distance when there are none)
/// and the groups as they stand. `spacing` is the lattice spacing, rad.
pub fn reinit(state: &mut PlateState, geo: &Geo, spacing: f64) {
    let n = state.samples.len();
    let phi: Vec<f64> = match state.level_set.as_ref().filter(|l| l.phi.len() == n) {
        Some(l) => l.phi.iter().map(|&x| f64::from(x)).collect(),
        None => vec![STALE_SPACINGS * spacing; n],
    };
    let groups: Vec<u32> = state.samples.iter().map(|s| s.group).collect();
    let phi = distances(&groups, &phi, geo, CAP_SPACINGS * spacing);
    // The class level set (ledger: PLT-852): the same from the class labels, over the arcs between
    // two classes of one group.
    // The kind and class level sets (ledger: PLT-852): the same from the labels, over the arcs
    // between two labels of one group. A new sample the resampling gave no distance (NaN) crosses
    // the outline at the stale half spacing.
    let stale = STALE_SPACINGS * spacing;
    let given = |v: Option<&Vec<f32>>| -> Vec<f64> {
        match v.filter(|x| x.len() == n) {
            Some(x) => x
                .iter()
                .map(|&d| if d.is_finite() { f64::from(d) } else { stale })
                .collect(),
            None => vec![stale; n],
        }
    };
    let kinds: Vec<u32> = state.samples.iter().map(kind_of).collect();
    let kind_phi = given(state.level_set.as_ref().map(|l| &l.kind_phi));
    let kind_phi = distances_where(&kinds, &kind_phi, geo, CAP_SPACINGS * spacing, |a, b| {
        a / 2 == b / 2
    });
    let classes: Vec<u32> = state.samples.iter().map(class_of).collect();
    let class_phi = given(state.level_set.as_ref().map(|l| &l.class_phi));
    let class_phi = distances_where(&classes, &class_phi, geo, CAP_SPACINGS * spacing, |a, b| {
        a / 3 == b / 3
    });
    state.level_set = Some(LevelSet {
        phi,
        member: groups,
        kind_phi,
        kind_member: kinds,
        class_phi,
        class_member: classes,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plates::init::random_rotation;
    use crate::plates::kinematics::arr;
    use crate::sphere::SplitMix64;

    fn lattice(n: usize, seed: u64) -> Geo {
        let mut rng = SplitMix64::new(seed);
        Geo::build(n, arr(random_rotation(&mut rng))).unwrap()
    }

    /// The signed distance of a great circle is linear in the position, and the gnomonic
    /// weights interpolate it exactly (Sussman et al. 1994: the interface is the zero level set
    /// of the distance function): any point of any triangle across the circle gets its own
    /// distance, whichever side holds the corners.
    #[test]
    fn a_great_circle_interpolates_exactly() {
        let geo = lattice(4000, 3);
        let pole = DVec3::new(0.3, -0.5, 0.8).normalize();
        let pts = geo.points();
        let side = |p: DVec3| u32::from(p.dot(pole) < 0.0);
        let mut checked = 0;
        let mut rng = SplitMix64::new(9);
        for (t, tri) in geo.mesh.triangles().iter().enumerate() {
            let g = tri.map(|v| side(pts[v as usize]));
            if g[0] == g[1] && g[1] == g[2] {
                continue;
            }
            let corners = tri.map(|v| pts[v as usize]);
            let phi = corners.map(|p| p.dot(pole).abs());
            for _ in 0..4 {
                let r = [rng.next_f64(), rng.next_f64(), rng.next_f64()];
                let q = (corners[0] * r[0] + corners[1] * r[1] + corners[2] * r[2]).normalize();
                let w = geo.mesh.barycentric(t as u32, q);
                let d0 = margin(g, w, phi, corners, 0);
                let d1 = margin(g, w, phi, corners, 1);
                let truth = q.dot(pole);
                assert!((d0 - truth).abs() < 1e-12, "{d0} against {truth}");
                assert!((d1 + truth).abs() < 1e-12, "{d1} against {}", -truth);
                checked += 1;
            }
        }
        assert!(checked > 200);
    }

    /// The reinitialisation of a great-circle outline from its exact distances gives every
    /// sample its distance from the circle again (a sample beside it exactly, within the cap
    /// the others), so a resampling that changes nothing leaves the outline where it was.
    #[test]
    fn reinitialisation_keeps_a_great_circle() {
        let geo = lattice(4000, 5);
        let spacing = (4.0 * std::f64::consts::PI / 4000.0).sqrt();
        let pole = DVec3::new(-0.2, 0.7, 0.4).normalize();
        let pts = geo.points();
        let groups: Vec<u32> = pts.iter().map(|p| u32::from(p.dot(pole) < 0.0)).collect();
        let phi: Vec<f64> = pts.iter().map(|p| p.dot(pole).abs()).collect();
        let cap = CAP_SPACINGS * spacing;
        let out = distances(&groups, &phi, &geo, cap);
        let mut beside = 0;
        for i in 0..pts.len() {
            let truth = phi[i];
            let edge = geo
                .edges(i as u32)
                .any(|(j, _)| groups[j as usize] != groups[i]);
            if edge {
                beside += 1;
                assert!(
                    (f64::from(out[i]) - truth).abs() < 1e-6,
                    "sample {i}: {} against {truth}",
                    out[i]
                );
            } else {
                assert!(f64::from(out[i]) >= truth.min(cap) - 1e-6);
            }
        }
        assert!(beside > 100);
    }

    /// At three groups the outline's three arcs meet where the three weigh the same, and each
    /// group holds the points of the triangle on its side of them ([`margin`]).
    #[test]
    fn a_triple_point_is_where_three_weigh_the_same() {
        let corners = [
            DVec3::new(1.0, 0.0, 0.05).normalize(),
            DVec3::new(0.0, 1.0, 0.05).normalize(),
            DVec3::new(0.0, 0.0, 1.0),
        ];
        let groups = [7, 8, 9];
        let phi = [0.02, 0.03, 0.06];
        // The point where w_k ∝ 1/ψ_k.
        let inv: f64 = phi.iter().map(|x| 1.0 / x).sum();
        let w = phi.map(|x| (1.0 / x) / inv);
        let d = groups.map(|g| margin(groups, w, phi, corners, g));
        for x in d {
            assert!(x.abs() < 1e-12, "{d:?}");
        }
        // Near a corner its group holds the point.
        let w = [0.8, 0.1, 0.1];
        assert!(margin(groups, w, phi, corners, 7) > 0.0);
        assert!(margin(groups, w, phi, corners, 8) < 0.0);
        assert!(margin(groups, w, phi, corners, 9) < 0.0);
        // A group with no corner here lies outside its outline by the largest of the others'
        // weighed distances.
        let len = (corners[0] * w[0] + corners[1] * w[1] + corners[2] * w[2]).length();
        let most = (0..3).map(|k| w[k] * phi[k]).fold(0.0, f64::max);
        assert!((margin(groups, w, phi, corners, 3) + most / len).abs() < 1e-12);
    }

    /// A point in a triangle with no corner of the group lies outside the group's outline by the
    /// others' interpolated distance, exact for a great circle as on the side with corners (a
    /// new sample of a subducting plate in front of the overriding plate's outline).
    #[test]
    fn a_triangle_without_the_group_lies_its_distance_outside() {
        let geo = lattice(4000, 7);
        let pole = DVec3::new(0.1, 0.3, -0.9).normalize();
        let pts = geo.points();
        let side = |p: DVec3| u32::from(p.dot(pole) < 0.0);
        let mut rng = SplitMix64::new(4);
        let mut checked = 0;
        for (t, tri) in geo.mesh.triangles().iter().enumerate() {
            let corners = tri.map(|v| pts[v as usize]);
            let g = tri.map(|v| side(pts[v as usize]));
            // Wholly on one side, within about two spacings of the circle.
            if g[0] != g[1] || g[1] != g[2] || corners.iter().any(|p| p.dot(pole).abs() > 0.12) {
                continue;
            }
            let phi = corners.map(|p| p.dot(pole).abs());
            let r = [rng.next_f64(), rng.next_f64(), rng.next_f64()];
            let q = (corners[0] * r[0] + corners[1] * r[1] + corners[2] * r[2]).normalize();
            let w = geo.mesh.barycentric(t as u32, q);
            let truth = q.dot(pole).abs();
            let d = margin(g, w, phi, corners, 1 - g[0]);
            assert!((d + truth).abs() < 1e-12, "{d} against {}", -truth);
            checked += 1;
        }
        assert!(checked > 100);
    }

    /// A sample keeps its distance unless the membership around it changed (a rift line drawn
    /// through its plate since the last resampling): then it gets the stale half spacing.
    #[test]
    fn a_new_boundary_makes_its_samples_stale() {
        let geo = lattice(2000, 11);
        let n = 2000;
        let spacing = (4.0 * std::f64::consts::PI / n as f64).sqrt();
        let pts = geo.points();
        let ls = LevelSet {
            phi: vec![0.01; n],
            member: vec![0; n],
            kind_phi: Vec::new(),
            kind_member: Vec::new(),
            class_phi: Vec::new(),
            class_member: Vec::new(),
        };
        // A rift: the samples with x > 0 go to group 1.
        let groups: Vec<u32> = pts.iter().map(|p| u32::from(p.x > 0.0)).collect();
        let d = old_distances(Some(&ls), &groups, &geo, spacing);
        let mut stale = 0;
        for i in 0..n {
            let edge = geo
                .edges(i as u32)
                .any(|(j, _)| groups[j as usize] != groups[i]);
            stale += usize::from(edge);
            let want = if edge { STALE_SPACINGS * spacing } else { 0.01 };
            assert!(
                (d[i] - want).abs() < 1e-7,
                "sample {i}: {} against {want}",
                d[i]
            );
        }
        assert!(stale > 50);
        // No level set yet: every sample is stale.
        let d = old_distances(None, &groups, &geo, spacing);
        assert!(d.iter().all(|&x| x == STALE_SPACINGS * spacing));
    }
}
