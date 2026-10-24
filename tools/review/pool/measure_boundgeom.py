#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `boundgeom` measure (the front lane, 2026-10-24): the plate boundary's geometry measured
as Bird 2003 measured PB2002's, from the crust budget's maps of one pool run (made with
+DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps: every 50 Myr the visible samples'
records, `crust_budget.rs` MAP_FIELDS, and their plate ids in maps/plate/), printed as a JSON object on
stdout.

    python3 -I measure_boundgeom.py <run folder>

Each snapshot after the first 100 Myr, except those taken at a resampling (every sample visible, the
record's `samples`: the lattice just rebuilt, nothing hidden and no front flagged, so they measure no
working boundary): the visible samples' Delaunay triangulation (their convex hull);
the boundary is the Voronoi edges between samples of two plates, a chain of the triangles'
circumcentres. A triangle of three plates is a triple junction; a piece is the chain between two
junctions (or a closed loop with none: an island of one plate in another), as Bird's 150 boundaries run
from junction to junction ([111], [112]). Lengths are Bird's divider walks (`bird.py`): a fixed
great-circle chord of 77 km (about the lattice's spacing, 71.4 km: it takes out the Voronoi chain's
zigzag) and 380 km (the shape at the scale of a plate's arcs). Tortuosity is a piece's length over the
great circle between its ends. Bird's values are tools/review/pool/bird/bird.py's over PB2002 (its output is the source
library's Bird2003/PB2002_geometry.json; ledger PLT-531).

  snaps            snapshots measured
  plates           plates with visible samples, per snapshot
  bnd_l77, bnd_l380     the boundary, km (Bird's pieces: 246,541 / 231,009; full resolution 260,487)
  e_l77, e_l380    the sum of the plates' perimeters over the sum of their spherical caps' (Bird 1.418 /
                   1.274; full 1.510)
  ppcap_aw_l380    each plate's perimeter at 380 km over its cap's, area-weighted (Bird 1.324)
  junctions, junctions_euler   triple junctions, and 2 (N - 2) for N plates, what simple plates
                   meeting three at a time have (Bird 100 for 52)
  pieces, loops    pieces between junctions and closed loops (Bird 150 and 0)
  stub_share       the pieces whose ends are under 77 km apart, a share of the pieces
  loop_share       the loops' share of the boundary at 77 km
  piece_p50_l77, piece_wmean_l77   a piece's length at 77 km: median, length-weighted mean (Bird 899 /
                   1,644)
  tau_p50_l77, tau_wmean_l77, tau_sum_l77    tortuosity at 77 km: median, length-weighted mean, the sum
                   of lengths over the sum of chords (Bird 1.100 / 1.290 / 1.220)
  tau_p50_l380, tau_wmean_l380, tau_sum_l380   at 380 km (Bird 1.025 / 1.205 / 1.143)
  tau_front_wmean_l77, tau_front_sum_l77, tau_front_wmean_l380, tau_front_sum_l380   the same over the
                   pieces mostly at a convergent front (more than half their edges are front edges, below);
                   Bird's mostly-SUB pieces (42): 1.172 / 1.145 at 77 km, 1.132 / 1.110 at 380 km
  tau_div_sum_l77, tau_other_sum_l77   the same, sum over sum, over the pieces mostly divergent
                   (the DIVERGENT flag) and the rest
  share_front, share_div   the shares of the boundary at 77 km in mostly-front and mostly-divergent pieces
  front_l77, front_l380    the convergent front's runs of edges, km. A front edge joins two plates and
                   touches a sample the boundary mix (`dynamics::classify_boundary`) put at a front, on
                   the overriding side: the map's MAP_FRONT_SUBDUCTION (256, the subducting sample
                   oceanic) or MAP_FRONT_COLLISION (512, continental) flag (crust_budget.rs, PLT-550).
                   The front is subduction and continental collision together, so it reads against
                   Bird's SUB plus CCB (51,310 + 23,003 = 74,313 km at full resolution; plates.md,
                   "Step 7"). Maps made before 2026-10-24 have no such flags: there a front edge
                   touches a TRENCH sample (on the subducting plate at the trench), which marks
                   collisions' fronts as well
  front_sub_l77, front_sub_l380   of it, the subduction front alone (the 256 flag; Bird SUB 51,310)
  front_trench_l77, front_trench_l380   on flagged maps, the front as older maps give it (TRENCH), to read
                   an arm with the flags against one without
  arcfront_l77     of it, where the overriding sample is oceanic crust (Stern & Scholl 2010's 14,500 km of
                   intra-oceanic arc)
  front_flagged    the share of the snapshots measured whose front came from the 256 / 512 flags
  resampled        snapshots skipped as taken at a resampling
  comps_per_plate  connected pieces of each plate (through its own samples' Delaunay edges), mean
  frag_samples     the share of samples outside their plate's largest piece
  e_l380_first, e_l380_last   e_l380 in the first snapshot after 100 Myr and in the last
  frag_small, frag_mid, frag_big   the plates' cut-off pieces (connected pieces outside each plate's
                   largest) by size, shares of the samples: 1 to 10 samples, 11 to 100, over 100
  frag_cut, frag_own   the same by what they are: mostly of rigid groups that also have samples in
                   the plate's largest piece (cut off it), or of groups of their own (terranes)
  <key>_clean      bnd, e, ppcap_aw_l380, junctions, pieces, loops, tau_sum, tau_front_sum, front,
                   front_sub and arcfront_l77 on the map with every cut-off piece given the plate most of its
                   neighbours in a plate's largest piece carry (what the boundary would be with each
                   plate one connected body; `clean`)
  <key>_k7         bnd, e, front, front_sub, front_trench, arcfront_l77 and the tortuosities on one lattice: the
                   cleaned map put on a fixed Fibonacci lattice of 100,000 points (`k7`), each point the
                   commonest plate of its 7 nearest visible samples and the flags of the nearest
                   sample of that plate. The model's visible samples sit on two lattices or more at a
                   boundary (each plate's own, turned at the last resampling and moved since; where
                   crust overlaps, on two at once), PB2002's raster on one; this is the like-for-like
                   length, and PB2002's raster goes through the same step (bird_raster.py)

Version 2 (2026-10-24: the fragments' classes and the cleaned map's numbers). Version 3 (2026-10-24,
PLT-550): resampling snapshots skipped, the front from the boundary mix's flags (subduction and
collision, against Bird's SUB + CCB), front_sub, and the one-lattice `_k7` numbers.
"""
import json
import math
import sys
from pathlib import Path

import numpy as np
from scipy.sparse import coo_matrix
from scipy.sparse.csgraph import connected_components
from scipy.spatial import ConvexHull

R = 6371.0
SKIP_MYR = 100.0
TRENCH, DIVERGENT = 2, 16  # plates/state.rs sample flags
FRONT_SUB, FRONT_COLL = 256, 512  # crust_budget.rs MAP_FRONT_SUBDUCTION, MAP_FRONT_COLLISION
FRONT = FRONT_SUB | FRONT_COLL
K7_N, K7_K = 100_000, 7  # the one lattice's points and the neighbours a point's plate is voted by
FIELDS = 11  # crust_budget.rs MAP_FIELDS
RES = (77.0, 380.0)


def ang(a, b):
    cx = a[1] * b[2] - a[2] * b[1]
    cy = a[2] * b[0] - a[0] * b[2]
    cz = a[0] * b[1] - a[1] * b[0]
    return math.atan2(math.sqrt(cx * cx + cy * cy + cz * cz), a[0] * b[0] + a[1] * b[1] + a[2] * b[2])


def unit(x, y, z):
    n = math.sqrt(x * x + y * y + z * z)
    return (x / n, y / n, z / n)


def divider(P, L):
    """Bird's divider walk (tools/review/pool/bird/bird.py): the polyline P of unit vectors walked
    with a fixed great-circle chord L km, n L plus the chord from the last divider point to the end."""
    th = L / R
    n = len(P)
    if n < 2:
        return 0.0
    c = P[0]
    j = 0
    s0 = 0.0
    steps = 0
    while j < n - 1:
        A, B = P[j], P[j + 1]
        if ang(c, B) >= th:
            lo, hi = s0, 1.0
            D = (B[0] - A[0], B[1] - A[1], B[2] - A[2])
            for _ in range(40):
                mid = 0.5 * (lo + hi)
                q = unit(A[0] + mid * D[0], A[1] + mid * D[1], A[2] + mid * D[2])
                if ang(c, q) >= th:
                    hi = mid
                else:
                    lo = mid
            s0 = hi
            c = unit(A[0] + s0 * D[0], A[1] + s0 * D[1], A[2] + s0 * D[2])
            steps += 1
        else:
            j += 1
            s0 = 0.0
    return steps * L + R * ang(c, P[-1])


def cap_perimeter(area_km2):
    cos_t = max(-1.0, min(1.0, 1.0 - area_km2 / (2.0 * math.pi * R * R)))
    return 2.0 * math.pi * R * math.sin(math.acos(cos_t))


def wmean(xs, ws):
    s = sum(ws)
    return sum(x * w for x, w in zip(xs, ws)) / s if s > 0 else None


def median(xs):
    xs = sorted(xs)
    return xs[len(xs) // 2] if xs else None


def measure(xyz, plate, flags, kind):
    """The boundary geometry of one snapshot: unit vectors `xyz` (N, 3), plate ids, flags and
    continental (bool) per sample. Returns a dict of numbers. The front is read from the FRONT
    flags when the snapshot has any, else from TRENCH (maps made before the flags)."""
    hull = ConvexHull(xyz)
    tri = hull.simplices
    nbr = hull.neighbors
    m = len(tri)
    a, b, c = xyz[tri[:, 0]], xyz[tri[:, 1]], xyz[tri[:, 2]]
    cc = np.cross(b - a, c - a)
    cc /= np.linalg.norm(cc, axis=1)[:, None]
    cc[np.einsum("ij,ij->i", cc, a) < 0.0] *= -1.0
    t_idx = np.repeat(np.arange(m), 3)
    k = np.tile(np.arange(3), m)
    u = tri[t_idx, (k + 1) % 3]
    v = tri[t_idx, (k + 2) % 3]
    t2 = nbr[t_idx, k]
    once = t_idx < t2
    cross_ = once & (plate[u] != plate[v])
    same = once & (plate[u] == plate[v])
    e1, e2, eu, ev = t_idx[cross_], t2[cross_], u[cross_], v[cross_]
    ne = len(e1)
    fl = flags[eu] | flags[ev]
    flagged = bool(np.any((flags & FRONT) != 0))
    div = (fl & DIVERGENT) != 0
    trench = (fl & TRENCH) != 0
    if flagged:
        front = (fl & FRONT) != 0
        sub = (fl & FRONT_SUB) != 0
        # The overriding sample of a front edge: the one with a FRONT flag.
        over = np.where((flags[eu] & FRONT) != 0, eu, ev)
    else:
        front = (fl & TRENCH) != 0
        sub = None
        # The overriding sample of a front edge: the one without the TRENCH flag.
        over = np.where((flags[eu] & TRENCH) != 0, ev, eu)
    arc = front & ~kind[over]
    # Adjacency of the boundary graph: triangles joined by boundary edges.
    nodes = np.concatenate([e1, e2])
    edges = np.concatenate([np.arange(ne), np.arange(ne)])
    order = np.argsort(nodes, kind="stable")
    nodes_s, edges_s = nodes[order], edges[order]
    deg = np.bincount(nodes, minlength=m)
    start = np.zeros(m + 1, dtype=np.int64)
    start[1:] = np.cumsum(deg)
    pts = [tuple(p) for p in cc]
    e1l, e2l = e1.tolist(), e2.tolist()
    visited = np.zeros(ne, dtype=bool)

    def walk(n0, e):
        path_n, path_e = [n0], []
        n = n0
        while True:
            visited[e] = True
            path_e.append(e)
            nxt = e2l[e] if e1l[e] == n else e1l[e]
            path_n.append(nxt)
            if deg[nxt] != 2:
                break
            s = start[nxt]
            ea, eb = edges_s[s], edges_s[s + 1]
            e_next = ea if ea != e else eb
            if visited[e_next]:
                break
            n, e = nxt, e_next
        return path_n, path_e

    pieces, loops = [], []
    for n0 in np.nonzero(deg >= 3)[0].tolist():
        for s in range(start[n0], start[n0 + 1]):
            e = edges_s[s]
            if not visited[e]:
                pieces.append(walk(n0, e))
    for e in np.nonzero(~visited)[0].tolist():
        if not visited[e]:
            loops.append(walk(e1l[e], e))
    junctions = int(np.sum(deg >= 3))
    # Plates, their areas and perimeters.
    ids, counts = np.unique(plate, return_counts=True)
    a_sample = 4.0 * math.pi * R * R / len(plate)
    area = {int(i): int(cnt) * a_sample for i, cnt in zip(ids, counts)}
    perim = {r: {i: 0.0 for i in area} for r in RES}
    out_pieces = []
    for kind_, group in (("piece", pieces), ("loop", loops)):
        for path_n, path_e in group:
            P = [pts[i] for i in path_n]
            lens = {r: divider(P, r) for r in RES}
            chord = R * ang(P[0], P[-1]) if kind_ == "piece" else 0.0
            pe = np.array(path_e)
            pa, pb = int(plate[eu[pe[0]]]), int(plate[ev[pe[0]]])
            for r in RES:
                perim[r][pa] += lens[r]
                perim[r][pb] += lens[r]
            f_share = float(np.mean(front[pe]))
            d_share = float(np.mean(div[pe]))
            # Runs of front edges within the piece, and of island-arc front edges.
            runs = {}
            masks = [("front", front[pe]), ("arc", arc[pe])]
            if sub is not None:
                masks.append(("sub", sub[pe]))
                masks.append(("trench", trench[pe]))
            for name, mask in masks:
                tot = {r: 0.0 for r in RES}
                i = 0
                ml = mask.tolist()
                while i < len(ml):
                    if not ml[i]:
                        i += 1
                        continue
                    j = i
                    while j + 1 < len(ml) and ml[j + 1]:
                        j += 1
                    Q = P[i:j + 2]
                    for r in RES:
                        tot[r] += divider(Q, r)
                    i = j + 1
                runs[name] = tot
            out_pieces.append({"kind": kind_, "lens": lens, "chord": chord, "front": f_share, "div": d_share,
                               "runs": runs})
    res = {"plates": len(area), "junctions": junctions, "junctions_euler": 2 * (len(area) - 2),
           "pieces": len(pieces), "loops": len(loops), "front_flagged": 1.0 if flagged else 0.0}
    for r in RES:
        tag = f"l{int(r)}"
        res["bnd_" + tag] = sum(p["lens"][r] for p in out_pieces)
        pc = sum(cap_perimeter(x) for x in area.values())
        res["e_" + tag] = sum(perim[r].values()) / pc
        res["front_" + tag] = sum(p["runs"]["front"][r] for p in out_pieces)
        res["arcfront_" + tag] = sum(p["runs"]["arc"][r] for p in out_pieces)
        if sub is not None:
            res["front_sub_" + tag] = sum(p["runs"]["sub"][r] for p in out_pieces)
            res["front_trench_" + tag] = sum(p["runs"]["trench"][r] for p in out_pieces)
    rat = [perim[380.0][i] / cap_perimeter(area[i]) for i in area]
    res["ppcap_aw_l380"] = wmean(rat, [area[i] for i in area])
    pp = [p for p in out_pieces if p["kind"] == "piece"]
    res["stub_share"] = sum(1 for p in pp if p["chord"] < 77.0) / max(len(pp), 1)
    tot77 = res["bnd_l77"]
    res["loop_share"] = sum(p["lens"][77.0] for p in out_pieces if p["kind"] == "loop") / tot77 if tot77 else None
    res["piece_p50_l77"] = median([p["lens"][77.0] for p in pp])
    res["piece_wmean_l77"] = wmean([p["lens"][77.0] for p in pp], [p["lens"][77.0] for p in pp])
    for r in RES:
        tag = f"l{int(r)}"
        ok = [p for p in pp if p["chord"] > 0.0]
        taus = [p["lens"][r] / p["chord"] for p in ok]
        ws = [p["lens"][r] for p in ok]
        res["tau_p50_" + tag] = median(taus)
        res["tau_wmean_" + tag] = wmean(taus, ws)
        res["tau_sum_" + tag] = sum(ws) / sum(p["chord"] for p in ok) if ok else None
        fr = [p for p in ok if p["front"] > 0.5]
        if fr:
            res["tau_front_wmean_" + tag] = wmean([p["lens"][r] / p["chord"] for p in fr], [p["lens"][r] for p in fr])
            res["tau_front_sum_" + tag] = sum(p["lens"][r] for p in fr) / sum(p["chord"] for p in fr)
        if r == 77.0:
            dv = [p for p in ok if p["div"] > 0.5 and p["front"] <= 0.5]
            ot = [p for p in ok if p["div"] <= 0.5 and p["front"] <= 0.5]
            res["tau_div_sum_l77"] = sum(p["lens"][r] for p in dv) / sum(p["chord"] for p in dv) if dv else None
            res["tau_other_sum_l77"] = sum(p["lens"][r] for p in ot) / sum(p["chord"] for p in ot) if ot else None
    res["share_front"] = sum(p["lens"][77.0] for p in out_pieces if p["front"] > 0.5) / tot77 if tot77 else None
    res["share_div"] = sum(p["lens"][77.0] for p in out_pieces if p["div"] > 0.5 and p["front"] <= 0.5) / tot77 \
        if tot77 else None
    # Each plate's connected pieces through its own samples.
    su, sv = u[same], v[same]
    n = len(plate)
    g = coo_matrix((np.ones(len(su)), (su, sv)), shape=(n, n))
    ncomp, lab = connected_components(g, directed=False)
    comp_size = np.bincount(lab, minlength=ncomp)
    comp_plate = np.zeros(ncomp, dtype=np.int64)
    comp_plate[lab] = plate
    largest = {}
    comps = {}
    for ci in range(ncomp):
        p_ = int(comp_plate[ci])
        largest[p_] = max(largest.get(p_, 0), int(comp_size[ci]))
        comps[p_] = comps.get(p_, 0) + 1
    res["comps_per_plate"] = sum(comps.values()) / max(len(comps), 1)
    res["frag_samples"] = 1.0 - sum(largest.values()) / n
    return res


def fib(n):
    """The Fibonacci lattice of `n` unit vectors (bird_raster.py's)."""
    i = np.arange(n)
    z = 1.0 - (2 * i + 1) / n
    r = np.sqrt(np.maximum(0.0, 1.0 - z * z))
    phi = (1.0 + math.sqrt(5.0)) / 2.0
    lon = 2.0 * math.pi * np.mod(i / phi, 1.0)
    return np.stack([r * np.cos(lon), r * np.sin(lon), z], axis=1)


_LATTICE = {}


def k7(xyz, plate, flags, kind, n=K7_N, k=K7_K):
    """The map on one fixed lattice of `n` points: each point takes the commonest plate of its `k`
    nearest samples (ties to the nearest sample's plate if it is among them, else the lowest id)
    and the flags and crust of the nearest sample of that plate. Returns the lattice's (xyz, plate,
    flags, kind)."""
    from scipy.spatial import cKDTree
    lat = _LATTICE.get(n)
    if lat is None:
        lat = _LATTICE[n] = fib(n)
    _, J = cKDTree(xyz).query(lat, k=k)
    P = plate[J]
    cnt = (P[:, :, None] == P[:, None, :]).sum(axis=2)
    top = cnt.max(axis=1)
    lowest = np.where(cnt == top[:, None], P, np.iinfo(np.int64).max).min(axis=1)
    chosen = np.where(cnt[:, 0] == top, P[:, 0], lowest)
    first = np.argmax(P == chosen[:, None], axis=1)
    jj = J[np.arange(n), first]
    return lat, chosen, flags[jj], kind[jj]


K7_KEYS = ["bnd_l77", "bnd_l380", "e_l77", "e_l380", "front_l77", "front_l380", "front_sub_l77",
           "front_sub_l380", "front_trench_l77", "front_trench_l380", "arcfront_l77", "tau_sum_l77",
           "tau_sum_l380", "tau_front_sum_l77", "tau_front_sum_l380"]


def measure_k7(xyz, plate, flags, kind):
    """`measure` of the map put on the one lattice (`k7`), the K7_KEYS as `<key>_k7`."""
    d = measure(*k7(xyz, plate, flags, kind))
    return {key + "_k7": d.get(key) for key in K7_KEYS}


def read_snapshot(map_path, plate_path):
    a = np.fromfile(map_path, "<f4").reshape(-1, FIELDS).astype(np.float64)
    if not plate_path.exists():
        return None
    p = np.fromfile(plate_path, "<u4").astype(np.int64)
    if len(p) != len(a):
        return None
    lat, lon = np.radians(a[:, 0]), np.radians(a[:, 1])
    xyz = np.stack([np.cos(lat) * np.cos(lon), np.cos(lat) * np.sin(lon), np.sin(lat)], axis=1)
    return xyz, p, a[:, 10].astype(np.int64), a[:, 4] > 0.5, a[:, 9].astype(np.int64)


CLEAN_KEYS = ["bnd_l77", "bnd_l380", "e_l77", "e_l380", "ppcap_aw_l380", "junctions", "pieces", "loops",
              "tau_sum_l77", "tau_sum_l380", "tau_front_sum_l77", "tau_front_sum_l380", "front_l77", "front_l380",
              "front_sub_l77", "front_sub_l380", "arcfront_l77"]


def _pieces(plate, au, av, n):
    same = plate[au] == plate[av]
    g = coo_matrix((np.ones(int(same.sum())), (au[same], av[same])), shape=(n, n))
    nc, lab = connected_components(g, directed=False)
    size = np.bincount(lab, minlength=nc)
    first = np.zeros(nc, dtype=np.int64)
    first[lab[::-1]] = np.arange(n)[::-1]
    cp = plate[first]
    main_of = {}
    for ci in np.argsort(-size, kind="stable").tolist():
        main_of.setdefault(int(cp[ci]), ci)
    main = np.isin(lab, list(main_of.values()))
    return lab, size, cp, main_of, main


def clean(xyz, plate, group):
    """The plates' cut-off pieces and the map without them. A fragment is a connected piece of a
    plate (through Delaunay edges between its own samples) outside the plate's largest. Returns the
    map with each fragment sample given the commonest plate among its neighbours in a plate's
    largest piece (repeated inwards until none is left), and the fragments' shares of the samples:
    by size (frag_small 1 to 10 samples, frag_mid 11 to 100, frag_big over 100) and by what they are
    (frag_cut: pieces whose samples are mostly of rigid groups that also have samples in the plate's
    largest piece, so cut off it; the rest, frag_own, crust the plate holds as groups of their own,
    terranes handed to it)."""
    n = len(plate)
    h = ConvexHull(xyz).simplices
    au = np.concatenate([h[:, 0], h[:, 1], h[:, 2], h[:, 1], h[:, 2], h[:, 0]])
    av = np.concatenate([h[:, 1], h[:, 2], h[:, 0], h[:, 0], h[:, 1], h[:, 2]])
    lab, size, cp, main_of, main = _pieces(plate, au, av, n)
    out = {"frag_small": 0.0, "frag_mid": 0.0, "frag_big": 0.0, "frag_cut": 0.0, "frag_own": 0.0}
    if main.all():
        return plate, out
    main_groups = {}
    for p_, ci in main_of.items():
        main_groups[p_] = np.unique(group[lab == ci])
    order = np.argsort(lab, kind="stable")
    bounds = np.searchsorted(lab[order], np.arange(len(size) + 1))
    for ci in range(len(size)):
        p_ = int(cp[ci])
        if main_of[p_] == ci:
            continue
        idx = order[bounds[ci]:bounds[ci + 1]]
        sz = len(idx)
        out["frag_small" if sz <= 10 else ("frag_mid" if sz <= 100 else "frag_big")] += sz / n
        shared = float(np.mean(np.isin(group[idx], main_groups[p_])))
        out["frag_cut" if shared > 0.5 else "frag_own"] += sz / n
    cleaned = plate.copy()
    for _ in range(200):
        stray = ~main
        if not stray.any():
            break
        sel = stray[au] & main[av]
        if not sel.any():
            break
        s_, t_ = au[sel], cleaned[av[sel]]
        pairs, counts = np.unique(np.stack([s_, t_], axis=1), axis=0, return_counts=True)
        # The commonest neighbouring plate per stray sample (ties to the lower plate id).
        best = {}
        for (si, ti), c in zip(pairs.tolist(), counts.tolist()):
            if si not in best or c > best[si][1]:
                best[si] = (ti, c)
        for si, (ti, _) in best.items():
            cleaned[si] = ti
            main[si] = True
    return cleaned, out


def main():
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    maps = Path(sys.argv[1]) / "maps"
    files = sorted(maps.glob("t*.bin"), key=lambda f: float(f.stem[1:]))
    if not files:
        print(json.dumps({}))
        return
    t0 = float(files[0].stem[1:])
    # The planet's samples: a snapshot with all of them visible was taken at a resampling.
    rec_path = Path(sys.argv[1]) / "rec.json"
    n_all = int(json.loads(rec_path.read_text()).get("samples", 0)) if rec_path.exists() else 0
    rows = []
    resampled = 0
    for f in files:
        t = float(f.stem[1:])
        if t <= t0 + SKIP_MYR:
            continue
        snap = read_snapshot(f, maps / "plate" / f.name)
        if snap is None:
            continue
        xyz, plate, flags, kind, group = snap
        if n_all and len(plate) >= n_all:
            resampled += 1
            continue
        row = measure(xyz, plate, flags, kind)
        cleaned, frag = clean(xyz, plate, group)
        row.update(frag)
        if row["frag_samples"] > 0.0:
            cl = measure(xyz, cleaned, flags, kind)
        else:
            cl = row
        for k in CLEAN_KEYS:
            row[k + "_clean"] = cl.get(k)
        row.update(measure_k7(xyz, cleaned, flags, kind))
        rows.append(row)
    out = {"snaps": len(rows), "resampled": resampled}
    if rows:
        keys = sorted({k for r in rows for k in r})
        for k in keys:
            xs = [r[k] for r in rows if r.get(k) is not None]
            out[k] = sum(xs) / len(xs) if xs else None
        out["e_l380_first"] = rows[0]["e_l380"]
        out["e_l380_last"] = rows[-1]["e_l380"]
    print(json.dumps({k: (round(v, 5) if isinstance(v, float) else v) for k, v in out.items()}))


if __name__ == "__main__":
    main()
