// SPDX-License-Identifier: LGPL-3.0-or-later
//! The lattice geometry the plate model works on: the spherical Delaunay triangulation of the
//! current lattice (P1-02's [`SphereMesh`], rebuilt at every global resampling), its vertex
//! adjacency with edge lengths, current sample positions, point location inside a group, and
//! multi-source Dijkstra distances over the mesh (geodesic distances within rigid groups).
//!
//! This is derived data: it is rebuilt from `PlateState::lattice` whenever it is missing (a
//! resumed run), so it never goes into a checkpoint.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use glam::{DQuat, DVec3};
use rayon::prelude::*;

use super::kinematics::quat;
use super::state::PlateState;
use crate::error::CoreError;
use crate::sphere::{Adjacency, SampleTree, SphereMesh, fibonacci_sphere};

/// The lattice points: the `n`-point Fibonacci sphere turned by `lattice`.
pub fn lattice_points(n: usize, lattice: [f64; 4]) -> Vec<DVec3> {
    let q = quat(lattice);
    fibonacci_sphere(n)
        .into_par_iter()
        .map(|p| (q * p).normalize())
        .collect()
}

/// The triangulation of one lattice and what the plate model derives from it.
pub struct Geo {
    pub lattice: [f64; 4],
    pub mesh: SphereMesh,
    pub adj: Adjacency,
    /// Great-circle length (radians) of every adjacency entry, parallel to `adj.neighbors`.
    pub edge_rad: Vec<f32>,
}

impl Geo {
    /// Triangulates the lattice (P1-02's hull) and derives the adjacency.
    pub fn build(n: usize, lattice: [f64; 4]) -> Result<Geo, CoreError> {
        let points = lattice_points(n, lattice);
        let mesh = SphereMesh::build(&points).map_err(|e| {
            CoreError::InvalidState(format!("plates: lattice triangulation failed: {e}"))
        })?;
        if !mesh.skipped().is_empty() {
            return Err(CoreError::InvalidState(format!(
                "plates: {} lattice points were not triangulated",
                mesh.skipped().len()
            )));
        }
        let adj = mesh.vertex_neighbors();
        let pts = mesh.points();
        let edge_rad: Vec<f32> = (0..n)
            .into_par_iter()
            .flat_map_iter(|v| {
                let p = pts[v];
                adj.of(v as u32)
                    .iter()
                    .map(move |&u| p.angle_between(pts[u as usize]) as f32)
                    .collect::<Vec<_>>()
            })
            .collect();
        Ok(Geo {
            lattice,
            mesh,
            adj,
            edge_rad,
        })
    }

    pub fn points(&self) -> &[DVec3] {
        self.mesh.points()
    }

    /// Neighbours of `v` with the edge lengths.
    pub fn edges(&self, v: u32) -> impl Iterator<Item = (u32, f32)> + '_ {
        let (a, b) = (
            self.adj.offsets[v as usize] as usize,
            self.adj.offsets[v as usize + 1] as usize,
        );
        self.adj.neighbors[a..b]
            .iter()
            .copied()
            .zip(self.edge_rad[a..b].iter().copied())
    }
}

/// Current positions of all samples: each group's rotation applied to its lattice points.
pub fn positions(state: &PlateState, geo: &Geo) -> Vec<DVec3> {
    let rots: Vec<DQuat> = state.groups.iter().map(|g| quat(g.rot)).collect();
    let pts = geo.points();
    state
        .samples
        .par_iter()
        .enumerate()
        .map(|(i, s)| rots[s.group as usize] * pts[i])
        .collect()
}

/// Where a direction falls inside one group: the lattice triangle whose three vertices are all in
/// `group` and whose cone (in the group's frame) holds the direction, with barycentric weights.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cover {
    /// The triangle's index in the lattice mesh.
    pub t: u32,
    pub tri: [u32; 3],
    pub w: [f64; 3],
}

impl Cover {
    /// The corner with the largest weight.
    pub fn major(&self) -> u32 {
        let k = (0..3).fold(0, |k, i| if self.w[i] > self.w[k] { i } else { k });
        self.tri[k]
    }
}

/// Locates the mantle-frame direction `q` in group `group` (rotation `rot`), walking from a
/// triangle of `hint_vertex`. `None` when the containing lattice triangle is not entirely the
/// group's (the direction is outside the group or on its boundary strip).
pub fn cover(
    geo: &Geo,
    state: &PlateState,
    group: u32,
    rot: DQuat,
    q: DVec3,
    hint_vertex: u32,
) -> Option<Cover> {
    let local = rot.inverse() * q;
    let hint = geo.mesh.vertex_triangle(hint_vertex).unwrap_or(0);
    let t = geo.mesh.locate(local, hint);
    let tri = geo.mesh.triangles()[t as usize];
    if tri
        .iter()
        .all(|&v| state.samples[v as usize].group == group)
    {
        Some(Cover {
            t,
            tri,
            w: geo.mesh.barycentric(t, local),
        })
    } else {
        None
    }
}

/// A k-d tree over the given positions (all samples, or only those `keep` selects; the items of
/// the tree then index `index`).
pub struct Tree {
    pub tree: SampleTree,
    /// Tree item → sample index.
    pub index: Vec<u32>,
}

impl Tree {
    pub fn over(pos: &[DVec3], keep: impl Fn(usize) -> bool + Sync) -> Tree {
        let index: Vec<u32> = (0..pos.len() as u32)
            .into_par_iter()
            .filter(|&i| keep(i as usize))
            .collect();
        let pts: Vec<DVec3> = index.par_iter().map(|&i| pos[i as usize]).collect();
        Tree {
            tree: SampleTree::new(&pts),
            index,
        }
    }

    /// The nearest sample and its squared chord distance.
    pub fn nearest(&self, q: DVec3) -> Option<(u32, f64)> {
        let (item, d2) = self.tree.nearest(q);
        (item != u32::MAX).then(|| (self.index[item as usize], d2))
    }

    /// The `k` nearest samples, nearest first.
    pub fn nearest_n(&self, q: DVec3, k: usize) -> Vec<(u32, f64)> {
        // Mapped in place (P1-17: the L1 sampling asks this for every cell every step).
        let mut found = self.tree.nearest_n(q, k);
        for x in &mut found {
            x.0 = self.index[x.0 as usize];
        }
        found
    }

    /// Samples within `angle` radians of `q`.
    pub fn within(&self, q: DVec3, angle: f64) -> Vec<(u32, f64)> {
        self.tree
            .within(q, angle)
            .into_iter()
            .map(|(item, d2)| (self.index[item as usize], d2))
            .collect()
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Entry {
    d: f64,
    v: u32,
    label: u32,
}

impl Eq for Entry {}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        // A min-heap on distance; ties broken by vertex then label, so results are deterministic.
        other
            .d
            .total_cmp(&self.d)
            .then_with(|| other.v.cmp(&self.v))
            .then_with(|| other.label.cmp(&self.label))
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Distances from a set of labelled sources over the lattice edges that `allow(from, to)` accepts,
/// up to `max_rad`. Returns per vertex the distance (∞ when unreached) and the label of the nearest
/// source (`u32::MAX` when unreached). Serial; deterministic.
pub fn dijkstra(
    geo: &Geo,
    sources: &[(u32, f64, u32)],
    allow: impl Fn(u32, u32) -> bool,
    max_rad: f64,
) -> (Vec<f64>, Vec<u32>) {
    let n = geo.points().len();
    let mut dist = vec![f64::INFINITY; n];
    let mut label = vec![u32::MAX; n];
    let mut heap = BinaryHeap::with_capacity(sources.len() * 2);
    for &(v, d, l) in sources {
        if d < dist[v as usize] {
            dist[v as usize] = d;
            label[v as usize] = l;
            heap.push(Entry { d, v, label: l });
        }
    }
    while let Some(Entry { d, v, label: l }) = heap.pop() {
        if d > dist[v as usize] || label[v as usize] != l {
            continue;
        }
        for (u, len) in geo.edges(v) {
            let nd = d + f64::from(len);
            if nd <= max_rad && nd < dist[u as usize] && allow(v, u) {
                dist[u as usize] = nd;
                label[u as usize] = l;
                heap.push(Entry {
                    d: nd,
                    v: u,
                    label: l,
                });
            }
        }
    }
    (dist, label)
}

/// [`dijkstra`] in arrays kept between runs (one per thread, [`dijkstra_reached`]), listing the
/// vertices it reached; a new run resets only those.
#[derive(Default)]
struct Reach {
    dist: Vec<f64>,
    label: Vec<u32>,
    reached: Vec<u32>,
    heap: BinaryHeap<Entry>,
}

impl Reach {
    /// [`dijkstra`]'s relaxations, in its order, on the kept arrays.
    fn run(
        &mut self,
        geo: &Geo,
        sources: &[(u32, f64, u32)],
        allow: impl Fn(u32, u32) -> bool,
        max_rad: f64,
    ) {
        let n = geo.points().len();
        if self.dist.len() == n {
            for &v in &self.reached {
                self.dist[v as usize] = f64::INFINITY;
                self.label[v as usize] = u32::MAX;
            }
        } else {
            self.dist = vec![f64::INFINITY; n];
            self.label = vec![u32::MAX; n];
        }
        self.reached.clear();
        self.heap.clear();
        let (dist, label, reached) = (&mut self.dist, &mut self.label, &mut self.reached);
        let heap = &mut self.heap;
        for &(v, d, l) in sources {
            if d < dist[v as usize] {
                if dist[v as usize] == f64::INFINITY {
                    reached.push(v);
                }
                dist[v as usize] = d;
                label[v as usize] = l;
                heap.push(Entry { d, v, label: l });
            }
        }
        while let Some(Entry { d, v, label: l }) = heap.pop() {
            if d > dist[v as usize] || label[v as usize] != l {
                continue;
            }
            for (u, len) in geo.edges(v) {
                let nd = d + f64::from(len);
                if nd <= max_rad && nd < dist[u as usize] && allow(v, u) {
                    if dist[u as usize] == f64::INFINITY {
                        reached.push(u);
                    }
                    dist[u as usize] = nd;
                    label[u as usize] = l;
                    heap.push(Entry {
                        d: nd,
                        v: u,
                        label: l,
                    });
                }
            }
        }
        reached.sort_unstable();
    }
}

thread_local! {
    static SCRATCH: std::cell::RefCell<Reach> = std::cell::RefCell::new(Reach::default());
}

/// [`dijkstra`]'s reached vertices (a finite distance), ascending, with their distances and
/// labels: exactly what [`dijkstra`] gives them, from arrays kept per thread (P1-19: the belts
/// and the arcs allocated two arrays over all 250k samples per run, then looped over every
/// sample for the few thousand reached).
///
/// A run to a shorter reach `r` gives the entries of this list with a distance of at most `r`,
/// when every source lies within `r`: Dijkstra pops in order of distance and a path's sum only
/// grows, so every entry beyond `r` is popped after all those within it and never changes one.
pub fn dijkstra_reached(
    geo: &Geo,
    sources: &[(u32, f64, u32)],
    allow: impl Fn(u32, u32) -> bool,
    max_rad: f64,
) -> Vec<(u32, f64, u32)> {
    SCRATCH.with(|r| {
        let mut r = r.borrow_mut();
        r.run(geo, sources, allow, max_rad);
        r.reached
            .iter()
            .map(|&v| (v, r.dist[v as usize], r.label[v as usize]))
            .collect()
    })
}

/// [`dijkstra`] of sources that `part(v)` splits into parts no allowed edge joins (`allow(a, b)`
/// implies `part(a) == part(b)`), each part's run on its own thread (P1-19: the arcs' run from
/// every front of every plate was one serial search). Returns [`dijkstra`]'s distances and
/// labels, and the reached vertices ascending. Each vertex gets what the single run gives it: a
/// part's entries are pushed only by its own pops, and the heap pops them in the same total
/// order (distance, vertex, label) with or without other parts' entries between them.
pub fn dijkstra_parts(
    geo: &Geo,
    sources: &[(u32, f64, u32)],
    part: impl Fn(u32) -> u32,
    allow: impl Fn(u32, u32) -> bool + Sync,
    max_rad: f64,
) -> (Vec<f64>, Vec<u32>, Vec<u32>) {
    let n = geo.points().len();
    let mut parts: std::collections::BTreeMap<u32, Vec<(u32, f64, u32)>> = Default::default();
    for &s in sources {
        parts.entry(part(s.0)).or_default().push(s);
    }
    let parts: Vec<Vec<(u32, f64, u32)>> = parts.into_values().collect();
    let found: Vec<Vec<(u32, f64, u32)>> = parts
        .par_iter()
        .map(|src| dijkstra_reached(geo, src, &allow, max_rad))
        .collect();
    let mut dist = vec![f64::INFINITY; n];
    let mut label = vec![u32::MAX; n];
    let mut reached = Vec::with_capacity(found.iter().map(Vec::len).sum());
    for &(v, d, l) in found.iter().flatten() {
        dist[v as usize] = d;
        label[v as usize] = l;
        reached.push(v);
    }
    reached.par_sort_unstable();
    (dist, label, reached)
}

/// The two nearest distinct classes of source for every vertex (a label-setting Dijkstra where a
/// vertex accepts at most one distance per class and two classes), over edges `allow` accepts.
/// `class(label)` maps a source label to its class (a plate). Per vertex: up to two
/// `(distance, label)` pairs, nearest first.
pub fn dijkstra_two(
    geo: &Geo,
    sources: &[(u32, f64, u32)],
    class: impl Fn(u32) -> u32,
    allow: impl Fn(u32, u32) -> bool,
    max_rad: f64,
) -> Vec<[(f64, u32); 2]> {
    let n = geo.points().len();
    const NONE: (f64, u32) = (f64::INFINITY, u32::MAX);
    let mut best = vec![[NONE; 2]; n];
    let mut settled = vec![0u8; n];
    let mut heap = BinaryHeap::with_capacity(sources.len() * 2);
    for &(v, d, l) in sources {
        heap.push(Entry { d, v, label: l });
    }
    while let Some(Entry { d, v, label: l }) = heap.pop() {
        let slot = &mut best[v as usize];
        let c = class(l);
        if settled[v as usize] >= 2 {
            continue;
        }
        if settled[v as usize] == 1 && class(slot[0].1) == c {
            continue;
        }
        slot[settled[v as usize] as usize] = (d, l);
        settled[v as usize] += 1;
        for (u, len) in geo.edges(v) {
            let nd = d + f64::from(len);
            if nd > max_rad || settled[u as usize] >= 2 || !allow(v, u) {
                continue;
            }
            if settled[u as usize] == 1 && class(best[u as usize][0].1) == c {
                continue;
            }
            heap.push(Entry {
                d: nd,
                v: u,
                label: l,
            });
        }
    }
    best
}

/// Keeps, for every label, only its largest connected piece (over lattice edges between samples
/// with that label) and hands every other piece to the label most of its outside neighbours
/// carry. Only samples `active` selects take part (the rest keep their labels and cast no votes).
/// `plate[i]` is rewritten in place. Returns how many samples moved.
pub fn connect_plates(geo: &Geo, plate: &mut [u32], active: impl Fn(usize) -> bool) -> usize {
    let n = plate.len();
    let mut comp = vec![u32::MAX; n];
    let mut sizes: Vec<(u32, usize)> = Vec::new(); // (plate, size) per component
    let mut stack = Vec::new();
    for s in 0..n {
        if comp[s] != u32::MAX || !active(s) {
            continue;
        }
        let c = sizes.len() as u32;
        let p = plate[s];
        comp[s] = c;
        stack.push(s as u32);
        let mut size = 0;
        while let Some(v) = stack.pop() {
            size += 1;
            for (u, _) in geo.edges(v) {
                if comp[u as usize] == u32::MAX && plate[u as usize] == p && active(u as usize) {
                    comp[u as usize] = c;
                    stack.push(u);
                }
            }
        }
        sizes.push((p, size));
    }
    // The largest component of each plate stays.
    let mut keep: std::collections::BTreeMap<u32, (usize, u32)> = Default::default();
    for (c, &(p, size)) in sizes.iter().enumerate() {
        let e = keep.entry(p).or_insert((0, u32::MAX));
        if size > e.0 {
            *e = (size, c as u32);
        }
    }
    let mut moved = 0;
    // Hand orphan pieces to a neighbour, smallest first, repeating until stable.
    for _ in 0..8 {
        let mut changed = false;
        let mut order: Vec<u32> = (0..sizes.len() as u32)
            .filter(|&c| keep[&sizes[c as usize].0].1 != c && sizes[c as usize].1 > 0)
            .collect();
        order.sort_by_key(|&c| (sizes[c as usize].1, c));
        if order.is_empty() {
            break;
        }
        let mut members: Vec<Vec<u32>> = vec![Vec::new(); sizes.len()];
        for (i, &c) in comp.iter().enumerate() {
            if c != u32::MAX {
                members[c as usize].push(i as u32);
            }
        }
        for c in order {
            let mut votes: std::collections::BTreeMap<u32, usize> = Default::default();
            let own = sizes[c as usize].0;
            for &v in &members[c as usize] {
                for (u, _) in geo.edges(v) {
                    let pu = plate[u as usize];
                    if pu != own && active(u as usize) {
                        *votes.entry(pu).or_default() += 1;
                    }
                }
            }
            if let Some((&to, _)) = votes
                .iter()
                .max_by_key(|(p, n)| (**n, std::cmp::Reverse(**p)))
            {
                let target = keep[&to].1;
                for &v in &members[c as usize] {
                    plate[v as usize] = to;
                    comp[v as usize] = target;
                }
                moved += members[c as usize].len();
                sizes[target as usize].1 += sizes[c as usize].1;
                sizes[c as usize].1 = 0;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    moved
}

/// Continental margin cut off its ocean by a relabelling (a rift's or a subduction initiation's
/// split, a collision's terrane transfer): the samples of `members` that are continental and have
/// a lattice neighbour in `members`, of the same rigid group, that is oceanic and carries another
/// label. Such a coast was a passive margin (crust and ocean on one plate) and is now a plate
/// boundary. `label(i)` is the sample's new plate or fragment.
pub fn coast_split(
    geo: &Geo,
    state: &PlateState,
    members: &[u32],
    member: impl Fn(usize) -> bool,
    label: impl Fn(usize) -> u32,
) -> usize {
    use super::state::CONTINENTAL;
    members
        .iter()
        .filter(|&&i| {
            let s = &state.samples[i as usize];
            s.kind == CONTINENTAL
                && geo.edges(i).any(|(u, _)| {
                    let o = &state.samples[u as usize];
                    member(u as usize)
                        && o.group == s.group
                        && o.kind != CONTINENTAL
                        && label(u as usize) != label(i as usize)
                })
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dijkstra_distances_follow_the_mesh() {
        let geo = Geo::build(2000, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let (d, l) = dijkstra(&geo, &[(0, 0.0, 7)], |_, _| true, 10.0);
        assert!(d.iter().all(|x| x.is_finite()));
        assert!(l.iter().all(|&x| x == 7));
        // Mesh paths are at least the great-circle distance and not much longer.
        let p0 = geo.points()[0];
        for (i, &di) in d.iter().enumerate() {
            let gc = p0.angle_between(geo.points()[i]);
            assert!(
                di >= gc - 1e-6 && di <= gc * 1.25 + 0.1,
                "{i}: {di} vs {gc}"
            );
        }
        let two = dijkstra_two(
            &geo,
            &[(0, 0.0, 0), (1999, 0.0, 1)],
            |l| l,
            |_, _| true,
            10.0,
        );
        assert!(two.iter().all(|b| b[0].1 != u32::MAX && b[1].1 != u32::MAX));
        assert!(two.iter().all(|b| b[0].1 != b[1].1 && b[0].0 <= b[1].0));
    }

    /// The kept-array and per-part runs give exactly the dense run's distances and labels, a
    /// shorter reach the longer run's entries within it, and a reused scratch nothing stale.
    #[test]
    fn sparse_and_split_dijkstras_match_the_dense_one() {
        let geo = Geo::build(3000, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let pts = geo.points();
        // Three bands of latitude no allowed edge crosses, sources in each, labelled in order.
        let part = |v: u32| {
            let z = pts[v as usize].z;
            u32::from(z > -0.2) + u32::from(z > 0.4)
        };
        let allow = |a: u32, b: u32| part(a) == part(b) && b % 7 != 3;
        let sources: Vec<(u32, f64, u32)> = [5u32, 400, 1200, 1201, 2500, 2999, 1700]
            .iter()
            .enumerate()
            .map(|(l, &v)| (v, 0.0, l as u32))
            .collect();
        for max_rad in [0.05, 0.3, 10.0] {
            let (dist, label) = dijkstra(&geo, &sources, allow, max_rad);
            let dense: Vec<(u32, f64, u32)> = (0..pts.len() as u32)
                .filter(|&v| dist[v as usize].is_finite())
                .map(|v| (v, dist[v as usize], label[v as usize]))
                .collect();
            // Twice, so the second run starts from the first's arrays.
            for _ in 0..2 {
                let got = dijkstra_reached(&geo, &sources, allow, max_rad);
                assert_eq!(got.len(), dense.len());
                for (g, d) in got.iter().zip(&dense) {
                    assert_eq!((g.0, g.1.to_bits(), g.2), (d.0, d.1.to_bits(), d.2));
                }
            }
            let (pd, pl, pr) = dijkstra_parts(&geo, &sources, part, allow, max_rad);
            assert_eq!(pr, dense.iter().map(|x| x.0).collect::<Vec<_>>());
            for v in 0..pts.len() {
                assert_eq!(pd[v].to_bits(), dist[v].to_bits(), "{v}");
                assert_eq!(pl[v], label[v], "{v}");
            }
            // A shorter reach: the entries within it.
            let short = dijkstra_reached(&geo, &sources, allow, 0.5 * max_rad);
            let within: Vec<_> = dense.iter().filter(|x| x.1 <= 0.5 * max_rad).collect();
            assert_eq!(short.len(), within.len());
            for (s, w) in short.iter().zip(within) {
                assert_eq!((s.0, s.1.to_bits(), s.2), (w.0, w.1.to_bits(), w.2));
            }
        }
    }

    #[test]
    fn coast_split_counts_continental_edges_cut_off_their_ocean() {
        use crate::plates::state::{CONTINENTAL, Group, PlateState, Sample};
        let geo = Geo::build(2000, [0.0, 0.0, 0.0, 1.0]).unwrap();
        let n = geo.points().len();
        // Continent north of z = 0.2, one plate, one group.
        let samples: Vec<Sample> = geo
            .points()
            .iter()
            .map(|p| {
                let mut s = Sample::blank(0);
                if p.z > 0.2 {
                    s.kind = CONTINENTAL;
                }
                s
            })
            .collect();
        let state = PlateState {
            n: n as u32,
            lattice: [0.0, 0.0, 0.0, 1.0],
            samples,
            groups: vec![Group {
                plate: 0,
                rot: [0.0, 0.0, 0.0, 1.0],
            }],
            plates: Vec::new(),
            contacts: Vec::new(),
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
            gap_budget: None,
            limits: None,
        };
        let all: Vec<u32> = (0..n as u32).collect();
        let coast = (0..n)
            .filter(|&i| {
                state.samples[i].kind == CONTINENTAL
                    && geo
                        .edges(i as u32)
                        .any(|(u, _)| state.samples[u as usize].kind != CONTINENTAL)
            })
            .count();
        assert!(coast > 0);
        // Ocean split off along the coast: every coastal sample counts.
        let by_kind = |i: usize| u32::from(state.samples[i].kind == CONTINENTAL);
        assert_eq!(coast_split(&geo, &state, &all, |_| true, by_kind), coast);
        // One label for everything: nothing is cut.
        assert_eq!(coast_split(&geo, &state, &all, |_| true, |_| 0), 0);
    }

    #[test]
    fn connect_plates_leaves_one_piece_each() {
        let geo = Geo::build(3000, [0.0, 0.0, 0.0, 1.0]).unwrap();
        // Stripes by height with an island of plate 0 inside plate 1.
        let mut plate: Vec<u32> = geo
            .points()
            .iter()
            .map(|p| {
                if p.z > 0.3 || (p.z < -0.5 && p.x > 0.8) {
                    0
                } else {
                    1
                }
            })
            .collect();
        let moved = connect_plates(&geo, &mut plate, |_| true);
        assert!(moved > 0);
        assert!(
            geo.points()
                .iter()
                .zip(&plate)
                .all(|(p, &k)| k == 1 || p.z > 0.0)
        );
    }
}
