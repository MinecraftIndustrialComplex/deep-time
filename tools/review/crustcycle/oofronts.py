#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The ocean-ocean subduction fronts on the crust-budget maps (no simulation): oofronts.py <arm folder>[@N] ...

Reads each run's `DEEPTIME_CRUST_BUDGET_MAPS` snapshots (maps/t*.bin, the plates in maps/plate/, the floor's age in
maps/age/) at -950 to -50 Myr. The front samples are the overriding side's samples of a subducting front as the
boundary mix classed it (the map flag MAP_FRONT_SUBDUCTION, 256: ocean-ocean or ocean-continent past incipient).
Each one's distance to its own plate's nearest visible continental crust (great circle, Earth-normalized km; the
boundary mix measures it through the plate's lattice, so this is a lower bound) sorts it as the record does
(PLT-370: ocean-ocean where none lies within the arc's distance, 200 km). For the ocean-ocean ones it prints the
distance bands (as COAST_BANDS_KM), what the nearest continental crust is (made by island arcs: arc_oceanic_m over
arc_continental_m and over 1 km; else the rest of the continents), the floor's age at the front sample, whether the
overriding plate holds continental crust at all, and the subducting plate's distance from the front to its own
continental crust (the ocean left before a collision). Shares of the ocean-ocean front, pooled over the snapshots
and runs (each front sample one lattice spacing).
"""
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
import probe  # noqa: E402

FRONT = 256
R_KM = 6371.0
ARC_KM = 200.0  # ledger: PLT-042 (the arc's distance, the class's line, PLT-370)
BANDS = [200.0, 300.0, 600.0, 1200.0, 2400.0]


def xyz(a):
    la, lo = np.radians(a[:, 0]), np.radians(a[:, 1])
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], 1)


def arc_km(chord):
    return 2.0 * np.arcsin(np.clip(chord / 2.0, 0.0, 1.0)) * R_KM


def snapshot(run, t):
    a, _ = probe.read(run, t)
    nm = probe.name_of(t)
    plate = np.fromfile(run / "maps" / "plate" / nm, "<u4").astype(np.int64)
    age = np.fromfile(run / "maps" / "age" / nm, "<f4").astype(np.float64)
    if len(plate) != len(a) or len(age) != len(a):
        return None
    p = xyz(a)
    cont = a[:, 4] > 0.5
    front = (a[:, 10].astype(np.int64) & FRONT) > 0
    idx = np.nonzero(front)[0]
    if not len(idx):
        return None
    d_over = np.full(len(idx), np.inf)
    near = np.full(len(idx), -1)
    d_under = np.full(len(idx), np.inf)
    trees = {}
    for q in np.unique(plate[cont]):
        m = np.nonzero(cont & (plate == q))[0]
        trees[q] = (cKDTree(p[m]), m)
    all_tree = cKDTree(p)
    for j, i in enumerate(idx):
        q = plate[i]
        if q in trees:
            dd, k = trees[q][0].query(p[i])
            d_over[j] = arc_km(dd)
            near[j] = trees[q][1][k]
        # The subducting plate: the nearest sample of another plate.
        _, ks = all_tree.query(p[i], k=12)
        other = [k for k in np.atleast_1d(ks) if plate[k] != q]
        if other:
            u = plate[other[0]]
            if u in trees:
                dd, _ = trees[u][0].query(p[i])
                d_under[j] = arc_km(dd)
    oo = d_over >= ARC_KM
    rec = {"n_front": float(len(idx)), "n_oo": float(oo.sum()), "spacing_km": float(np.sqrt(4 * np.pi * R_KM ** 2 / len(a)))}
    # By the arc's substrate as Earth's lists read it (Gerya 2011: intra-oceanic arcs stand on "ocean crust formed at
    # back-arc spreading centres, earlier intra-oceanic arcs, accretionary complexes and oceanic plateaux"): the
    # distance to the overriding plate's continental crust not made by island arcs (arc_oceanic_m not over
    # arc_continental_m and 1 km).
    old = cont & ~((a[:, 7] > a[:, 8]) & (a[:, 7] > 1000.0))
    d_old = np.full(len(idx), np.inf)
    for q in np.unique(plate[idx]):
        m = np.nonzero(old & (plate == q))[0]
        if not len(m):
            continue
        sel = plate[idx] == q
        dd, _ = cKDTree(p[m]).query(p[idx[sel]])
        d_old[sel] = arc_km(dd)
    rec["n_io_substrate"] = float((d_old >= ARC_KM).sum())
    if not oo.any():
        return rec
    d = d_over[oo]
    for lo, hi in zip(BANDS, BANDS[1:] + [np.inf]):
        rec[f"d_{lo:.0f}"] = float(((d >= lo) & (d < hi)).sum())
    has = near[oo] >= 0
    rec["none"] = float((~has).sum())
    nn = near[oo][has]
    ao, ac = a[nn, 7], a[nn, 8]
    io = (ao > ac) & (ao > 1000.0)
    band = d[has] < 300.0
    rec["near_io"] = float(io.sum())
    rec["near_io_200_300"] = float((io & band).sum())
    rec["near_rest_200_300"] = float((~io & band).sum())
    fa = age[idx[oo]]
    for lo, hi in ((0, 10), (10, 30), (30, 60), (60, 120), (120, 1e9)):
        rec[f"age_{lo}"] = float(((fa >= lo) & (fa < hi)).sum())
    du = d_under[oo]
    for lo, hi in ((0, 500), (500, 1500), (1500, 3000), (3000, np.inf)):
        rec[f"under_{lo}"] = float(((du >= lo) & (du < hi)).sum())
    return rec


def main(argv):
    ts = list(range(-950, 0, 50))
    tot, runs = {}, 0
    for arg in argv:
        for r in probe.runs_of(arg):
            runs += 1
            for t in ts:
                try:
                    rec = snapshot(r, t)
                except (FileNotFoundError, ValueError):
                    rec = None
                if not rec:
                    continue
                sp = rec.pop("spacing_km")
                tot["snapshots"] = tot.get("snapshots", 0) + 1
                tot["km_front"] = tot.get("km_front", 0.0) + rec["n_front"] * sp
                tot["km_oo"] = tot.get("km_oo", 0.0) + rec["n_oo"] * sp
                for k, v in rec.items():
                    tot[k] = tot.get(k, 0.0) + v
    n = tot.get("snapshots", 0)
    print(f"# {runs} runs, {n} snapshots; front samples on the overriding side (one spacing each)")
    print(f"mean front length at a snapshot, km: subducting {tot['km_front'] / n:.0f}, of it ocean-ocean"
          f" {tot['km_oo'] / n:.0f} ({tot['n_oo'] / tot['n_front']:.3f})")
    oo = tot["n_oo"]
    print(f"by the substrate (no continental crust but island arcs' within {ARC_KM:.0f} km): share of the subducting"
          f" front {tot['n_io_substrate'] / tot['n_front']:.3f}")
    print("shares of the ocean-ocean front:")
    for k in sorted(k for k in tot if k not in ("snapshots", "km_front", "km_oo", "n_front", "n_oo", "n_io_substrate")):
        print(f"  {k:22s} {tot[k] / oo:.3f}")


if __name__ == "__main__":
    main(sys.argv[1:])
