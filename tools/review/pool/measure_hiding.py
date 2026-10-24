#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `hiding` measure (the front lane, 2026-10-24): what the overlaps' hiding
(`subduction::resolve`) cuts off the plates, from the crust budget's maps of one pool run (made with
+DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps), and what becomes of the plates
fission makes of the pieces, from the run's dynamics record. Printed as a JSON object on stdout.

    python3 -I measure_hiding.py <run folder>

With `plate_fission` on every plate is one piece right after a resampling, so the pieces a snapshot
finds outside their plate's largest were cut since the last resampling, most by the hiding (the cut
trace, plates.md "Step 3c"). The maps hold the visible samples only, so a hidden band shows as the
gap it leaves. Each snapshot after the first 100 Myr that is not at a resampling (one with every
sample visible is skipped): the visible samples' Delaunay triangulation
(their convex hull), each plate's connected pieces through its own samples' edges, and for every
piece outside its plate's largest (a cut piece):

  its size; whether it carries the TRENCH flag (a subducting plate's visible sample beside its own
  hidden ones: the piece borders its own slab); the plates of its Delaunay neighbours; whether one
  other plate encloses it; its distance to its plate's largest piece, in lattice spacings
  (sqrt(4 pi / n) for the record's n samples); and the plates the great circle from it to the
  nearest sample of that piece crosses (the nearest visible sample every third of a spacing).

Classes (shares of the cut samples):

  hole     enclosed by one other plate, 1 to 3 samples
  island   enclosed by one other plate, more
  neck     not enclosed, at most 3 spacings from its plate's largest piece: a thin hidden band
  across   farther, the path crossing one other plate only: its plate went under a plate and
           shows again past it, or a band of one plate cut it off
  far      farther, the path crossing two plates or more

Keys (means over the snapshots; counts per snapshot):

  snaps, cut_share            snapshots measured; cut samples over visible samples
  pieces                      cut pieces per snapshot
  size_p50, size_wmean        a cut piece's samples: median, sample-weighted mean
  trench_share                cut samples in pieces with the TRENCH flag
  hole, island, neck, across, far   the classes' shares of the cut samples
  d_p50, d_wmean              distance to the plate's largest piece, spacings: median over pieces,
                              sample-weighted mean
  width_p50                   spacings of other plates the path crosses, median over pieces
  cont_share                  continental share of the cut samples
  big_pieces, big_share       pieces of min_plate_area_frac or more (fission makes them plates at
                              the next resampling; smaller ones are captured): count, cut share
  small_*, big_*              the classes, trench share and the flags below, apart for the two
  *_nb_sz, *_nb_dv            of the edges from a piece's TRENCH samples to other plates, the
                              share whose far sample is in
                              a subduction zone (SUBDUCTION_ZONE: the plate there has a front
                              near: the piece meets the overriding plate's leading edge) or at a
                              divergent boundary (DIVERGENT: the plate there moves away)
  *_own_sz                    the piece's samples in a subduction zone (the piece overrides)
  main_nb_sz, main_nb_dv      the same two shares for the main pieces' TRENCH samples (the
                              ordinary trench, for comparison)
  fissions                    plates born of fission over the run (the record)
  fission_into_parent         of those that died absorbed, the share absorbed into their parent
  fission_life_p50            their median life, Myr (those that died)
"""
import json
import math
import sys
from pathlib import Path

import numpy as np
from scipy.sparse import coo_matrix
from scipy.sparse.csgraph import connected_components
from scipy.spatial import ConvexHull, cKDTree

SKIP_MYR = 100.0
TRENCH, DIVERGENT, SZ = 2, 16, 64  # plates/state.rs sample flags
BIG = 0.003  # min_plate_area_frac (PLT-091): a piece this share of the samples or more becomes a plate
FIELDS = 11  # crust_budget.rs MAP_FIELDS


def read_snapshot(map_path, plate_path):
    a = np.fromfile(map_path, "<f4").reshape(-1, FIELDS).astype(np.float64)
    if not plate_path.exists():
        return None
    p = np.fromfile(plate_path, "<u4").astype(np.int64)
    if len(p) != len(a):
        return None
    lat, lon = np.radians(a[:, 0]), np.radians(a[:, 1])
    xyz = np.stack([np.cos(lat) * np.cos(lon), np.cos(lat) * np.sin(lon), np.sin(lat)], axis=1)
    return xyz, p, a[:, 10].astype(np.int64), a[:, 4] > 0.5


def edges(xyz):
    tri = ConvexHull(xyz).simplices
    e = np.concatenate([tri[:, [0, 1]], tri[:, [1, 2]], tri[:, [2, 0]]])
    e = np.sort(e, axis=1)
    return np.unique(e, axis=0)


def slerp_path(a, b, step):
    ang = math.acos(max(-1.0, min(1.0, float(np.dot(a, b)))))
    k = max(int(ang / step), 1)
    out = []
    for i in range(k + 1):
        t = i / k
        if ang < 1e-12:
            out.append(a)
            continue
        s = math.sin(ang)
        v = (math.sin((1 - t) * ang) * a + math.sin(t * ang) * b) / s
        out.append(v / np.linalg.norm(v))
    return np.array(out), ang


def measure(xyz, plate, flags, kind, h):
    n = len(plate)
    e = edges(xyz)
    au, av = e[:, 0], e[:, 1]
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
    main_ids = set(main_of.values())
    cut_comps = [ci for ci in range(nc) if ci not in main_ids]
    # Neighbours across plates, per component: their plates and flags.
    diff = ~same
    nb = {}
    nbf = {}
    base = [0, 0, 0]  # main pieces' TRENCH samples' cross-plate edges: count, SZ, DIVERGENT beyond
    for u, v in zip(au[diff].tolist(), av[diff].tolist()):
        nb.setdefault(lab[u], []).append(int(plate[v]))
        nb.setdefault(lab[v], []).append(int(plate[u]))
        if flags[u] & TRENCH:
            nbf.setdefault(lab[u], []).append(int(flags[v]))
        if flags[v] & TRENCH:
            nbf.setdefault(lab[v], []).append(int(flags[u]))
        for a_, b_ in ((u, v), (v, u)):
            if lab[a_] in main_ids and flags[a_] & TRENCH:
                base[0] += 1
                base[1] += bool(flags[b_] & SZ)
                base[2] += bool(flags[b_] & DIVERGENT)
    tree_all = cKDTree(xyz)
    members = {}
    for i, c in enumerate(lab.tolist()):
        if c not in main_ids:
            members.setdefault(c, []).append(i)
    main_trees = {}
    rows = []
    for ci in cut_comps:
        idx = np.array(members[ci])
        p = int(cp[ci])
        mc = main_of[p]
        if mc not in main_trees:
            main_trees[mc] = np.nonzero(lab == mc)[0]
        mains = main_trees[mc]
        mt = cKDTree(xyz[mains])
        d, j = mt.query(xyz[idx])
        k = int(np.argmin(d))
        a, b = xyz[idx[k]], xyz[mains[j[k]]]
        path, ang = slerp_path(a, b, h / 3.0)
        _, near = tree_all.query(path)
        seq = plate[near]
        others = [int(x) for x in seq if x != p]
        crossed = len(set(others))
        width = len(others) * (ang / max(len(path) - 1, 1)) / h
        nbp = nb.get(ci, [])
        enclosed = len(set(nbp)) == 1
        dist = ang / h
        sz = len(idx)
        if enclosed and sz <= 3:
            cls = "hole"
        elif enclosed:
            cls = "island"
        elif dist <= 3.0:
            cls = "neck"
        elif crossed <= 1:
            cls = "across"
        else:
            cls = "far"
        nf = np.array(nbf.get(ci, []), dtype=np.int64)
        rows.append({
            "size": sz,
            "trench": bool(np.any(flags[idx] & TRENCH)),
            "cls": cls,
            "dist": dist,
            "width": width,
            "cont": int(kind[idx].sum()),
            "nb_n": len(nf),
            "nb_sz": int(np.sum(nf & SZ > 0)),
            "nb_dv": int(np.sum(nf & DIVERGENT > 0)),
            "own_sz": float(np.mean(flags[idx] & SZ > 0)),
        })
    cut = sum(r["size"] for r in rows)
    out = {"cut_share": cut / n, "pieces": len(rows)}
    if base[0]:
        out["main_nb_sz"] = base[1] / base[0]
        out["main_nb_dv"] = base[2] / base[0]
    big = [r for r in rows if r["size"] >= BIG * n]
    out["big_pieces"] = len(big)
    out["big_share"] = sum(r["size"] for r in big) / cut if cut else None
    for tag, rs in (("small", [r for r in rows if r["size"] < BIG * n]), ("big", big)):
        w = sum(r["size"] for r in rs)
        if not w:
            continue
        out[f"{tag}_own_sz"] = sum(r["own_sz"] * r["size"] for r in rs) / w
        m = sum(r["nb_n"] for r in rs)
        if m:
            out[f"{tag}_nb_sz"] = sum(r["nb_sz"] for r in rs) / m
            out[f"{tag}_nb_dv"] = sum(r["nb_dv"] for r in rs) / m
        out[f"{tag}_trench"] = sum(r["size"] for r in rs if r["trench"]) / w
        for c in ("hole", "island", "neck", "across", "far"):
            out[f"{tag}_{c}"] = sum(r["size"] for r in rs if r["cls"] == c) / w
    if rows:
        sizes = np.array([r["size"] for r in rows], dtype=float)
        dists = np.array([r["dist"] for r in rows])
        out["size_p50"] = float(np.median(sizes))
        out["size_wmean"] = float((sizes * sizes).sum() / sizes.sum())
        out["trench_share"] = sum(r["size"] for r in rows if r["trench"]) / cut
        for c in ("hole", "island", "neck", "across", "far"):
            out[c] = sum(r["size"] for r in rows if r["cls"] == c) / cut
        out["d_p50"] = float(np.median(dists))
        out["d_wmean"] = float((dists * sizes).sum() / sizes.sum())
        out["width_p50"] = float(np.median([r["width"] for r in rows]))
        out["cont_share"] = sum(r["cont"] for r in rows) / cut
    return out


def fission_stats(rec):
    lives = rec.get("plates", [])
    fis = [p for p in lives if p.get("origin") == "fission"]
    dead = [p for p in fis if p.get("died_myr") is not None]
    absorbed = [p for p in dead if p.get("fate") == "absorbed"]
    into_parent = [p for p in absorbed if p.get("into") is not None and p.get("into") == p.get("parent")]
    life = [p["died_myr"] - p["born_myr"] for p in dead]
    return {
        "fissions": len(fis),
        "fission_into_parent": len(into_parent) / len(absorbed) if absorbed else None,
        "fission_life_p50": float(np.median(life)) if life else None,
    }


def main():
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    run = Path(sys.argv[1])
    rec = json.loads((run / "rec.json").read_text())
    n_all = int(rec.get("samples", 0)) or 100_000
    h = math.sqrt(4.0 * math.pi / n_all)
    maps = run / "maps"
    files = sorted(maps.glob("t*.bin"), key=lambda f: float(f.stem[1:]))
    rows = []
    if files:
        t0 = float(files[0].stem[1:])
        for f in files:
            t = float(f.stem[1:])
            if t <= t0 + SKIP_MYR:
                continue
            snap = read_snapshot(f, maps / "plate" / f.name)
            if snap is None or len(snap[1]) >= n_all:
                # A snapshot at a resampling: nothing hidden, every plate one piece.
                continue
            rows.append(measure(*snap, h))
    out = {"snaps": len(rows)}
    if rows:
        keys = sorted({k for r in rows for k in r})
        for k in keys:
            xs = [r[k] for r in rows if r.get(k) is not None]
            out[k] = sum(xs) / len(xs) if xs else None
    out.update(fission_stats(rec))
    print(json.dumps({k: (round(v, 5) if isinstance(v, float) else v) for k, v in out.items()}))


if __name__ == "__main__":
    main()
