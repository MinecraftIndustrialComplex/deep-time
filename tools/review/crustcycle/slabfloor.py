#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""What floor the subducting plates bring to the intra-oceanic trenches (no simulation): slabfloor.py <arm folder>[@N] ...

Reads each run's crust-budget maps (`DEEPTIME_CRUST_BUDGET_MAPS`: maps/t*.bin, maps/plate/, maps/age/) at -950 to -50
Myr and rec.json's plate lives. The intra-oceanic front samples are oofronts.py's (the overriding side of a subducting
front past incipient, no continental crust of the overriding plate within 200 km, PLT-370); the subducting plate is the
plate of the nearest sample of another plate among the 12 nearest, and the floor going under is that sample's age.
For each front sample it reads the subducting plate at the map's time:

  its age since its birth (rec.json `born_myr`) and how it was born (`origin`);
  its visible ocean floor's median and 90th-percentile age, and its share of the planet's samples;
  the distance from the front sample to the plate's nearest floor within 5 Myr of the planet's youngest floor (its
  spreading ridge's flank: new floor is made only at resamplings, so between them the youngest floor ages; great
  circle, Earth-normalized km; none when the plate has no floor that young: no ridge spread on it at the last
  resampling), and the plate's own youngest floor's age over the planet's (its ridge's age); and the spreading rate that
  distance implies with the floor's age at the front (km / Myr = mm/yr, a half rate: the floor's age over its path
  from the ridge, where the plate's ridge has not moved);

and prints, pooled over the snapshots and runs, the intra-oceanic front on floor younger than 55 Myr and on older
floor by each of these bands (shares of each), with the arm's mean front length. Where the run records its onsets
(the onset trace, branch oceanic-fronts), each front sample is linked to its contact's onset as gapfronts.py links it
(the pair's latest onset still active, or one side for a split's copy), and the front is banded by the contact's age
(the map's time less the onset's) and by the floor that first went down at the onset (`slab_age_myr`), and the floor
going under now is given by the contact's age: over a contact's life, does its trench reach older floor?
"""
import json
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gapfronts  # noqa: E402
import oofronts  # noqa: E402
import probe  # noqa: E402

YOUNG_MYR = 55.0
RIDGE_MYR = 5.0
BANDS = {
    "plate age, Myr": ([0, 25, 55, 100, 200], lambda r: r["plate_age"]),
    "plate's floor median, Myr": ([0, 10, 25, 55, 100], lambda r: r["plate_p50"]),
    "plate's floor p90, Myr": ([0, 25, 55, 100, 150], lambda r: r["plate_p90"]),
    "plate's share of the planet": ([0, 0.01, 0.03, 0.06, 0.12], lambda r: r["plate_share"]),
    "ridge distance, km": ([0, 500, 1000, 2000, 4000], lambda r: r["ridge_km"]),
    "own youngest over planet's, Myr": ([0, 5, 20, 55, 100], lambda r: r["own_young"]),
    "implied half rate, mm/yr": ([0, 10, 20, 40, 80], lambda r: r["rate"]),
    "contact age, Myr": ([0, 10, 25, 55, 100], lambda r: r["contact_age"]),
    "first floor down, Myr": ([0, 6, 12, 25, 55], lambda r: r["first_floor"]),
}
ORIGINS = ["initial", "rift", "fission", "back_arc", "subduction_initiation", "margin_propagation", "other"]


def snapshot(run, t, lives, onsets=None, by_pair=None):
    a, _ = probe.read(run, t)
    nm = probe.name_of(t)
    plate = np.fromfile(run / "maps" / "plate" / nm, "<u4").astype(np.int64)
    age = np.fromfile(run / "maps" / "age" / nm, "<f4").astype(np.float64)
    if len(plate) != len(a) or len(age) != len(a):
        return None, 0.0
    p = oofronts.xyz(a)
    cont = a[:, 4] > 0.5
    front = (a[:, 10].astype(np.int64) & oofronts.FRONT) > 0
    idx = np.nonzero(front)[0]
    spacing = float(np.sqrt(4 * np.pi * oofronts.R_KM ** 2 / len(a)))
    if not len(idx):
        return [], spacing
    trees = {}
    for q in np.unique(plate[cont]):
        m = np.nonzero(cont & (plate == q))[0]
        trees[q] = cKDTree(p[m])
    all_tree = cKDTree(p)
    ocean = ~cont & np.isfinite(age)
    youngest = float(age[ocean].min()) if ocean.any() else 0.0
    stats, ridges = {}, {}
    out = []
    for i in idx:
        q = int(plate[i])
        if q in trees:
            dd, _ = trees[q].query(p[i])
            if oofronts.arc_km(dd) < oofronts.ARC_KM:
                continue
        _, ks = all_tree.query(p[i], k=12)
        other = [k for k in np.atleast_1d(ks) if plate[k] != q]
        if not other:
            continue
        k = other[0]
        u = int(plate[k])
        if u not in stats:
            m = ocean & (plate == u)
            ages = age[m]
            stats[u] = (float(np.median(ages)) if len(ages) else np.nan,
                        float(np.percentile(ages, 90)) if len(ages) else np.nan,
                        float((plate == u).sum()) / len(a),
                        float(ages.min()) - youngest if len(ages) else np.nan)
            young = np.nonzero(m & (age < youngest + RIDGE_MYR))[0]
            ridges[u] = cKDTree(p[young]) if len(young) else None
        p50, p90, share, own_young = stats[u]
        ridge_km = np.inf
        if ridges[u] is not None:
            dd, _ = ridges[u].query(p[i])
            ridge_km = oofronts.arc_km(dd)
        slab_age = float(age[k])
        life = lives.get(u, {})
        contact_age, first_floor = np.nan, np.nan
        if onsets:
            j, _ = gapfronts.link(by_pair, onsets, u, q, t)
            if j is not None:
                contact_age = t - onsets[j]["t"]
                first_floor = onsets[j]["slab_age_myr"]
        out.append({
            "slab_age": slab_age,
            "contact_age": contact_age,
            "first_floor": first_floor,
            "young": slab_age < YOUNG_MYR,
            "plate_age": t - float(life.get("born_myr", -1000.0)),
            "origin": life.get("origin", "other") if life.get("origin") in ORIGINS else "other",
            "plate_p50": p50,
            "plate_p90": p90,
            "plate_share": share,
            "own_young": own_young,
            "ridge_km": ridge_km,
            "rate": ridge_km / slab_age if np.isfinite(ridge_km) and slab_age > 0 else np.inf,
        })
    return out, spacing


def band_name(edges, x):
    if not np.isfinite(x):
        return "none"
    for lo, hi in zip(edges, edges[1:] + [np.inf]):
        if lo <= x < hi:
            return f"{lo:g}-{hi:g}" if np.isfinite(hi) else f">{lo:g}"
    return f"<{edges[0]:g}"


def main(argv):
    ts = list(range(-950, 0, 50))
    for arg in argv:
        rows, snaps, km = [], 0, 0.0
        runs = probe.runs_of(arg)
        for r in runs:
            rec = json.loads((r / "rec.json").read_text())
            lives = {int(x["id"]): x for x in rec.get("plates") or []}
            onsets = rec.get("onsets") or []
            by_pair = {}
            for i, o in enumerate(onsets):
                by_pair.setdefault((o["under"], o["over"]), []).append(i)
            for t in ts:
                try:
                    got, sp = snapshot(r, t, lives, onsets, by_pair)
                except (FileNotFoundError, ValueError):
                    got = None
                if got is None:
                    continue
                snaps += 1
                for x in got:
                    x["km"] = sp
                    km += sp
                rows.extend(got)
        print(f"# {arg}: {len(runs)} runs, {snaps} snapshots; intra-oceanic front {km / max(snaps, 1):.0f} km at a"
              f" snapshot")
        for yv, name in ((True, f"floor under {YOUNG_MYR:.0f} Myr"), (False, f"floor {YOUNG_MYR:.0f} Myr or older")):
            sel = [x for x in rows if x["young"] == yv]
            tot = sum(x["km"] for x in sel)
            print(f"  {name}: {tot / max(snaps, 1):.0f} km ({tot / max(km, 1e-9):.3f})")
            for title, (edges, key) in BANDS.items():
                acc = {}
                for x in sel:
                    b = band_name(edges, key(x))
                    acc[b] = acc.get(b, 0.0) + x["km"]
                order = [band_name(edges, e) for e in edges] + ["none"]
                print(f"    {title:28s} " + "  ".join(f"{b} {acc.get(b, 0.0) / max(tot, 1e-9):.3f}"
                                                     for b in order if b in acc))
            acc = {}
            for x in sel:
                acc[x["origin"]] = acc.get(x["origin"], 0.0) + x["km"]
            print(f"    {'plate origin':28s} " + "  ".join(f"{o} {acc.get(o, 0.0) / max(tot, 1e-9):.3f}"
                                                         for o in ORIGINS if o in acc))
            fin = [x for x in sel if np.isfinite(x["ridge_km"])]
            if fin:
                w = np.array([x["km"] for x in fin])
                d = np.array([x["ridge_km"] for x in fin])
                order = np.argsort(d)
                c = np.cumsum(w[order]) / w.sum()
                print(f"    ridge distance, km, length-weighted median {d[order][np.searchsorted(c, 0.5)]:.0f}"
                      f" (p25 {d[order][np.searchsorted(c, 0.25)]:.0f}, p75 {d[order][np.searchsorted(c, 0.75)]:.0f})")
        by_contact_age(rows)
    return 0


def by_contact_age(rows):
    linked = [x for x in rows if np.isfinite(x["contact_age"])]
    if not linked:
        return
    print("    the floor going under by the contact's age (all intra-oceanic front linked): km share, floor age p25/p50/p75,"
          " share under 55 Myr")
    edges = [0, 10, 25, 55, 100, 200]
    tot = sum(x["km"] for x in linked)
    for lo, hi in zip(edges, edges[1:] + [np.inf]):
        sel = [x for x in linked if lo <= x["contact_age"] < hi]
        if not sel:
            continue
        a = np.array([x["slab_age"] for x in sel])
        w = sum(x["km"] for x in sel)
        print(f"      {lo:g}-{hi:g} Myr: {w / tot:.3f}  {np.percentile(a, 25):.0f}/{np.percentile(a, 50):.0f}/"
              f"{np.percentile(a, 75):.0f}  {np.mean(a < YOUNG_MYR):.3f}")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
