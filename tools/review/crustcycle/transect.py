#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The overriding plate behind the ocean-ocean fronts, on the crust-budget maps (no simulation):

    transect.py <arm folder>[@N] ...

For each ocean-ocean front sample at -950 to -50 Myr (the overriding side of a subducting front, MAP_FRONT_SUBDUCTION,
with no continental crust of its own plate within the arc's distance, PLT-370, great circle as oofronts.py), walks
from the front into its own plate along the great circle away from the subducting plate's nearest sample, every 50 km
to 1,500 km, taking the nearest sample at each point, and stops where it meets continental crust or leaves the
plate. Of the walks that meet continental crust it prints the distance, whether that crust is an island arc's
(arc_oceanic_m over arc_continental_m and over 1 km, as oofronts.py), and the floor's age along the oceanic strip
between the front and the continent: at the front, at the strip's middle and just before the continent, and where on
the strip the youngest floor lies. A strip that is youngest at the front grew at the trench; one youngest just before
the continent opened behind the arc (a back-arc basin); one about as old throughout is ocean the trench was set in.
Behind the arc, for every ocean-ocean front sample: the floor's age at the first oceanic sample 200 to 600 km along
the same great circle, whatever plate holds it but the one going under (as earth_fronts.py reads Seton et al. 2020's
floor behind Earth's fronts; young floor there is a back-arc basin's). Of the walks that leave the plate first it prints the distance, whether the plate met is the one going under, and
the floor's age on both sides of that edge (young floor on both is a ridge: the overriding plate is a strip between
its trench and a ridge).
Shares of the ocean-ocean front, pooled over the snapshots and runs (each front sample one lattice spacing).
"""
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
import probe  # noqa: E402
from oofronts import ARC_KM, FRONT, R_KM, arc_km, xyz  # noqa: E402

STEP_KM = 50.0
FAR_KM = 1500.0
DIST_BANDS = [200.0, 300.0, 600.0, 1200.0, FAR_KM]
# Myr: the strip counts as youngest at one end where that end is this much younger than the other.
AGE_EDGE_MYR = 10.0


def walk(p, n, tree, plate, cont, q):
    """The samples met from p along n (unit tangent), every STEP_KM; stops at continental crust or another plate."""
    seen = []
    for d in np.arange(STEP_KM, FAR_KM + STEP_KM / 2, STEP_KM):
        ang = d / R_KM
        x = p * np.cos(ang) + n * np.sin(ang)
        _, k = tree.query(x)
        if plate[k] != q:
            return seen, -1 - k, d
        if cont[k]:
            return seen, k, d
        if not seen or seen[-1] != k:
            seen.append(k)
    return seen, None, None


def snapshot(run, t):
    a, _ = probe.read(run, t)
    nm = probe.name_of(t)
    plate = np.fromfile(run / "maps" / "plate" / nm, "<u4").astype(np.int64)
    age = np.fromfile(run / "maps" / "age" / nm, "<f4").astype(np.float64)
    if len(plate) != len(a) or len(age) != len(a):
        return None
    p = xyz(a)
    cont = a[:, 4] > 0.5
    island = cont & (a[:, 7] > a[:, 8]) & (a[:, 7] > 1000.0)
    idx = np.nonzero((a[:, 10].astype(np.int64) & FRONT) > 0)[0]
    if not len(idx):
        return None
    tree = cKDTree(p)
    rec = {}
    for i in idx:
        q = plate[i]
        own = np.nonzero(cont & (plate == q))[0]
        if len(own):
            dd = arc_km(np.min(np.linalg.norm(p[own] - p[i], axis=1)))
            if dd < ARC_KM:
                continue
        rec["n_oo"] = rec.get("n_oo", 0.0) + 1.0
        _, ks = tree.query(p[i], k=12)
        other = [k for k in np.atleast_1d(ks) if plate[k] != q]
        if not other:
            rec["no_under"] = rec.get("no_under", 0.0) + 1.0
            continue
        n = p[i] - p[other[0]]
        n -= p[i] * p[i].dot(n)
        norm = np.linalg.norm(n)
        if norm == 0.0:
            continue
        n /= norm
        # Behind the arc: the first oceanic floor 200 to 600 km out, on any plate but the subducting one.
        behind = None
        for dist in np.arange(200.0, 600.0 + STEP_KM / 2, STEP_KM):
            ang = dist / R_KM
            _, k = tree.query(p[i] * np.cos(ang) + n * np.sin(ang))
            if cont[k]:
                break
            if plate[k] != plate[other[0]]:
                behind = age[k]
                break
        if behind is None:
            rec["behind_none"] = rec.get("behind_none", 0.0) + 1.0
        else:
            for lo, hi in ((0, 10), (10, 30), (30, 60), (60, 120), (120, 1e9)):
                if lo <= behind < hi:
                    rec[f"behind_age_{lo}"] = rec.get(f"behind_age_{lo}", 0.0) + 1.0
        strip, hit, d = walk(p[i], n, tree, plate, cont, q)
        if hit is None:
            rec["no_continent_1500"] = rec.get("no_continent_1500", 0.0) + 1.0
            continue
        if hit < 0:
            k = -1 - hit
            band = next((f"{lo:.0f}" for lo, hi in zip([0.0] + DIST_BANDS, DIST_BANDS + [FAR_KM + 1]) if lo <= d < hi))
            whose = "under" if plate[k] == plate[other[0]] else "third"
            rec[f"left_{whose}_{band}"] = rec.get(f"left_{whose}_{band}", 0.0) + 1.0
            near = age[strip[-1]] if strip else age[i]
            if not cont[k] and near < 12.0 and age[k] < 12.0:
                rec[f"left_{whose}_at_ridge"] = rec.get(f"left_{whose}_at_ridge", 0.0) + 1.0
            continue
        kind = "arc" if island[hit] else "rest"
        band = next((f"{lo:.0f}" for lo, hi in zip(DIST_BANDS, DIST_BANDS[1:]) if lo <= d < hi), "<200")
        rec[f"hit_{kind}_{band}"] = rec.get(f"hit_{kind}_{band}", 0.0) + 1.0
        ages = [age[i]] + [age[k] for k in strip if k != i]
        front_age, back_age = ages[0], ages[-1]
        mid_age = ages[len(ages) // 2]
        if back_age < front_age - AGE_EDGE_MYR:
            shape = "young_behind"
        elif front_age < back_age - AGE_EDGE_MYR:
            shape = "young_at_front"
        else:
            shape = "even"
        rec[f"shape_{kind}_{shape}"] = rec.get(f"shape_{kind}_{shape}", 0.0) + 1.0
        for name, v in (("front", front_age), ("mid", mid_age), ("back", back_age)):
            rec[f"sum_age_{kind}_{name}"] = rec.get(f"sum_age_{kind}_{name}", 0.0) + v
        rec[f"n_{kind}"] = rec.get(f"n_{kind}", 0.0) + 1.0
        for lo, hi in ((0, 10), (10, 30), (30, 60), (60, 1e9)):
            if lo <= back_age < hi:
                rec[f"back_age_{kind}_{lo}"] = rec.get(f"back_age_{kind}_{lo}", 0.0) + 1.0
    return rec


def main(argv):
    ts = list(range(-950, 0, 50))
    tot, runs, snaps = {}, 0, 0
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
                snaps += 1
                for k, v in rec.items():
                    tot[k] = tot.get(k, 0.0) + v
    oo = tot.get("n_oo", 0.0)
    print(f"# {runs} runs, {snaps} snapshots; {oo:.0f} ocean-ocean front samples (overriding side)")
    print("shares of the ocean-ocean front:")
    for k in sorted(k for k in tot if not k.startswith(("sum_", "n_"))):
        print(f"  {k:30s} {tot[k] / oo:.3f}")
    for kind in ("arc", "rest"):
        n = tot.get(f"n_{kind}", 0.0)
        if n:
            ages = "  ".join(f"{name} {tot[f'sum_age_{kind}_{name}'] / n:5.1f}" for name in ("front", "mid", "back"))
            print(f"mean floor age on the strip before {kind} continental crust, Myr: {ages}")


if __name__ == "__main__":
    main(sys.argv[1:])
