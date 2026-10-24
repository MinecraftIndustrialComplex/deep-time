#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Bird 2003's PB2002 plates on the model's lattice: each of the 100,000 Fibonacci points takes the plate
on its side of the nearest boundary step; then the pool's `boundgeom` measure (the same code the model's
maps go through) and the dynamics record's estimator (one spacing per boundary sample over 1.043).

    python3 -I bird_raster.py <bird data dir> <bird scripts dir> <pool tools dir> [n]
"""
import json
import math
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import ConvexHull, cKDTree

data, scripts, tools = (Path(a) for a in sys.argv[1:4])
N = int(sys.argv[4]) if len(sys.argv) > 4 else 100_000
sys.path.insert(0, str(scripts))
sys.path.insert(0, str(tools))
import bird  # noqa: E402
import measure_boundgeom as mg  # noqa: E402

R = 6371.0
segs = bird.read_dig(str(data / "PB2002_boundaries.dig.txt"))
steps = bird.read_steps(str(data / "PB2002_steps.dat.txt"))
dense, owner = [], []
si = 0
step_rows = []
for k, (title, pts) in enumerate(segs):
    t = title[:5]
    left, sym, right = t[0:2], t[2], t[3:5]
    V = [bird.ll2v(*p) for p in pts]
    for i in range(len(V) - 1):
        cls = steps[si]["cls"]
        si += 1
        a, b = np.array(V[i]), np.array(V[i + 1])
        d = bird.gcd(V[i], V[i + 1])
        n = max(2, int(d / 5.0) + 2)
        for s in np.linspace(0.0, 1.0, n):
            q = a + s * (b - a)
            dense.append(q / np.linalg.norm(q))
            owner.append(len(step_rows))
        step_rows.append((a, b, left, right, sym, cls))
dense = np.array(dense)
owner = np.array(owner)

i = np.arange(N)
z = 1.0 - (2 * i + 1) / N
r = np.sqrt(np.maximum(0.0, 1.0 - z * z))
phi = (1.0 + math.sqrt(5.0)) / 2.0
lon = 2.0 * math.pi * np.mod(i / phi, 1.0)
xyz = np.stack([r * np.cos(lon), r * np.sin(lon), z], axis=1)
dist, j = cKDTree(dense).query(xyz)
names = {}
plate = np.zeros(N, dtype=np.int64)
flags = np.zeros(N, dtype=np.int64)
spacing = math.sqrt(4.0 * math.pi / N)
for n_, (x, jj, dd) in enumerate(zip(xyz, j, dist)):
    a, b, left, right, sym, cls = step_rows[owner[jj]]
    side_left = float(np.dot(x, np.cross(a, b))) > 0.0
    name = left if side_left else right
    plate[n_] = names.setdefault(name, len(names))
    near = dd < spacing
    if near and cls == "SUB":
        under = right if sym == "/" else (left if sym == "\\" else None)
        if under == name:
            flags[n_] |= mg.TRENCH
    if near and cls in ("OSR", "CRB"):
        flags[n_] |= mg.DIVERGENT
kind = np.zeros(N, dtype=bool)
table = json.loads((data.parent / "results.json").read_text())["A"]["plates"]
inv = {k: nm for nm, k in names.items()}


def worst_plates(tag):
    ar = {nm: float(np.sum(plate == k)) / N * 4.0 * math.pi for nm, k in names.items()}
    rows = sorted(((abs(ar[nm] - table[nm]["area_sr"]), nm, round(ar[nm], 4), table[nm]["area_sr"])
                   for nm in ar), reverse=True)[:6]
    print(tag, [(nm, a, t) for _, nm, a, t in rows], file=sys.stderr)


worst_plates("nearest step")
# PB2002's plates are each one closed outline (para [123]): samples outside their plate's largest
# connected piece are the side test's misses near corners; each takes its neighbours' commonest plate
# among the samples in their plates' largest pieces, until none is left.
from scipy.sparse import coo_matrix  # noqa: E402
from scipy.sparse.csgraph import connected_components  # noqa: E402

h0 = ConvexHull(xyz).simplices
au = np.concatenate([h0[:, 0], h0[:, 1], h0[:, 2]])
av = np.concatenate([h0[:, 1], h0[:, 2], h0[:, 0]])
nbrs = [[] for _ in range(N)]
for x0, y0 in zip(au.tolist(), av.tolist()):
    nbrs[x0].append(y0)
    nbrs[y0].append(x0)
for _ in range(50):
    same = plate[au] == plate[av]
    g = coo_matrix((np.ones(int(same.sum())), (au[same], av[same])), shape=(N, N))
    nc, lab = connected_components(g, directed=False)
    size = np.bincount(lab, minlength=nc)
    best = {}
    first = np.zeros(nc, dtype=np.int64)
    first[lab[::-1]] = np.arange(N)[::-1]
    comp_plate = plate[first]
    for ci in np.argsort(-size):
        best.setdefault(int(comp_plate[ci]), int(ci))
    bp = np.full(len(names), -1, dtype=np.int64)
    for pl, ci in best.items():
        bp[pl] = ci
    main = bp[plate] == lab
    stray = np.nonzero(~main)[0]
    if len(stray) == 0:
        break
    new = plate.copy()
    for n_ in stray.tolist():
        cand = [int(plate[m_]) for m_ in nbrs[n_] if main[m_]]
        if cand:
            new[n_] = max(set(cand), key=cand.count)
    plate = new
flags[:] = 0
for n_, (jj, dd) in enumerate(zip(j, dist)):
    a, b, left, right, sym, cls = step_rows[owner[jj]]
    if dd >= spacing:
        continue
    under = right if sym == "/" else (left if sym == "\\" else None)
    if cls == "SUB" and under is not None and names.get(under) == plate[n_]:
        flags[n_] |= mg.TRENCH
    if cls in ("OSR", "CRB"):
        flags[n_] |= mg.DIVERGENT
worst_plates("cleaned")
area = {nm: float(np.sum(plate == k)) / N * 4.0 * math.pi for nm, k in names.items()}
worst = max(abs(area[nm] - table[nm]["area_sr"]) for nm in area)
out = {"plates": len(names), "area_worst_sr": worst,
       "area_worst_rel_large": max(abs(area[nm] - table[nm]["area_sr"]) / table[nm]["area_sr"]
                                   for nm in area if table[nm]["area_sr"] > 0.0377)}
# PB2002's plates at or above the model's floor for a plate (PLT-091, 0.003 of the planet), on the
# lattice and in Bird's table: the model's plate count reads against this, not against all 52.
FLOOR_SR = 0.003 * 4.0 * math.pi
out["plates_at_floor"] = sum(1 for a_ in area.values() if a_ >= FLOOR_SR)
out["plates_at_floor_table"] = sum(1 for nm in area if table[nm]["area_sr"] >= FLOOR_SR)
out["floor_area_share"] = sum(a_ for a_ in area.values() if a_ >= FLOOR_SR) / (4.0 * math.pi)
out["floor_differs"] = sorted(f"{nm} {area[nm]:.4f} (table {table[nm]['area_sr']})" for nm in area
                              if (area[nm] >= FLOOR_SR) != (table[nm]["area_sr"] >= FLOOR_SR))
# The dynamics record's estimator: one spacing per sample with a Delaunay neighbour of another plate.
hull = ConvexHull(xyz)
tri = hull.simplices
eu = np.concatenate([tri[:, 0], tri[:, 1], tri[:, 2]])
ev = np.concatenate([tri[:, 1], tri[:, 2], tri[:, 0]])
x_ = plate[eu] != plate[ev]
bnd = np.zeros(N, dtype=bool)
bnd[eu[x_]] = True
bnd[ev[x_]] = True
sp_km = spacing * R
P = {nm: float(np.sum(bnd & (plate == k))) * sp_km / 1.043 for nm, k in names.items()}
pc = {nm: mg.cap_perimeter(area[nm] * R * R) for nm in names}
out["rec_estimator_bnd_km"] = sum(P.values()) / 2.0
out["rec_estimator_excess"] = sum(P.values()) / sum(pc.values())
out["pcap_sum_km"] = sum(pc.values())
out["trench_samples"] = int(np.sum(flags & mg.TRENCH != 0))
out.update(mg.measure(xyz, plate, flags, kind))
print(json.dumps({k: (round(v, 4) if isinstance(v, float) else v) for k, v in out.items()}, indent=1))
