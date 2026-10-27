#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""oldfloor.py <argkey folder> [--seeds=1,2,3,4] [--from=-900]: where the old sea floor sits, and what could reach it.

The old-floor lane's diagnosis (2026-10-27). Reads each run's plate check (plate.json), the crust-budget maps
(maps/t*.bin, maps/plate/, maps/age/; crust_budget.rs MAP_FIELDS, write_plates, write_ages) and rec.json, and prints
one JSON object per run on stdout (numpy and scipy: the VM's pool numpy env).

Per snapshot from --from on (the plate check's floor-age measures start 100 Myr in; snapshots taken at a resampling,
with no hidden sample, are skipped), the visible oceanic samples older than 180 and older than 250 Myr, classed by:
  margin      within the margin of its own plate's continent (PLT-503's 260 km plus one spacing), the passive-margin
              floor that no rule converts;
  basin       the oceanic connected component it lies in (the lattice graph: neighbours within 1.7 spacings), as the
              main ocean (the largest component) or a closed basin (any other), and whether that component reaches an
              active subduction front at all;
  reach       the distance through the ocean (Dijkstra over oceanic samples) to the nearest active subduction front:
              a sample flagged MAP_FRONT_SUBDUCTION (the overriding side of an ocean-ocean or ocean-continent front, as
              the dynamics record classes it) or an oceanic TRENCH sample next to one; banded 0-500, 500-1000,
              1000-2000, 2000-4000, 4000+ km, none. An invading trench (Duarte et al. 2013) has to cross this distance;
  own         the straight distance to its own plate's nearest TRENCH sample (where its plate goes down), or none;
  giant       on a plate over 0.2 of the planet.
And the passive margins' ages: every continental sample with an oceanic graph neighbour of its own plate is a passive
coast sample; its margin age is the oldest such neighbour's age (the floor beside a passive margin is as old as the
margin, Bradley 2008). Coast samples with an oceanic neighbour of another plate are active or transform; the share of
the coast that is passive is reported beside the plate check's active_margin_mean."""
import json
import math
import sys
from pathlib import Path

import numpy as np
from scipy.sparse import coo_matrix
from scipy.sparse.csgraph import connected_components, dijkstra
from scipy.spatial import cKDTree

FIELDS = 11
R_KM = 6371.0
TRENCH = 2
FRONT_SUB = 256
MARGIN_KM = 260.0  # PLT-503: arc distance + width; one spacing added per run
GIANT = 0.2
OLD = (180.0, 250.0)
REACH_EDGES = [0.0, 500.0, 1000.0, 2000.0, 4000.0]
MARGIN_AGE_EDGES = [0.0, 50.0, 100.0, 150.0, 180.0, 250.0, 350.0, 500.0]
CHECKS = ("ocean_age_p50_myr", "ocean_age_p90_mean_myr", "ocean_old180_pct", "ocean_old250_pct",
          "active_margin_mean", "speed_rms_mm_yr", "boundary_subduction_pct", "plates_alive_p50")


def xyz(lat, lon):
    la, lo = np.radians(lat), np.radians(lon)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], 1)


def km(chord):
    return 2.0 * R_KM * np.arcsin(np.minimum(chord / 2.0, 1.0))


def bands(x, edges):
    """Counts of x in [edges[i], edges[i+1]) and a last band [edges[-1], inf), then the infinite ones."""
    fin = np.isfinite(x)
    idx = np.searchsorted(edges, x[fin], side="right") - 1
    out = np.bincount(np.clip(idx, 0, len(edges) - 1), minlength=len(edges)).astype(int).tolist()
    return out + [int((~fin).sum())]


def snapshot(run, path, N, step):
    t = float(path.stem[1:])
    a = np.fromfile(path, "<f4").reshape(-1, FIELDS).astype(np.float64)
    n = len(a)
    if n >= N:
        return None  # a resampling's snapshot: no hidden sample, no front flagged
    pp, ap = run / "maps" / "plate" / path.name, run / "maps" / "age" / path.name
    if not (pp.is_file() and ap.is_file()):
        return None
    plate = np.fromfile(pp, "<u4").astype(np.int64)
    age = np.fromfile(ap, "<f4").astype(np.float64)
    if len(plate) != n or len(age) != n:
        return None
    cont = a[:, 4] > 0.5
    oc = ~cont
    flags = np.rint(a[:, 10]).astype(np.int64)
    p = xyz(a[:, 0], a[:, 1])
    spacing = math.sqrt(4.0 * math.pi / N) * R_KM
    margin_km = MARGIN_KM + spacing

    # The lattice graph: each visible sample's neighbours within 1.7 spacings (at most 8).
    tree = cKDTree(p)
    d, j = tree.query(p, k=9)
    dkm = km(d)
    i = np.repeat(np.arange(n), 9)
    j, dkm = j.ravel(), dkm.ravel()
    keep = (j != i) & (j < n) & (dkm <= 1.7 * spacing)
    i, j, dkm = i[keep], j[keep], dkm[keep]

    # Active subduction fronts: the overriding side's flag, and oceanic TRENCH samples beside one.
    sub = (flags & FRONT_SUB) != 0
    near_sub = np.zeros(n, bool)
    near_sub[i[sub[j]]] = True
    src = sub | (oc & ((flags & TRENCH) != 0) & near_sub)

    # Ocean components.
    oo = oc[i] & oc[j]
    g = coo_matrix((np.ones(oo.sum()), (i[oo], j[oo])), shape=(n, n)).tocsr()
    ncomp, comp = connected_components(g, directed=False)
    comp_size = np.bincount(comp[oc], minlength=ncomp)
    main = int(np.argmax(comp_size))
    comp_has_src = np.zeros(ncomp, bool)
    comp_has_src[comp[src & oc]] = True
    # a front sample on continental crust (an Andean margin) serves the components it touches
    touch = src[i] & oc[j]
    comp_has_src[comp[j[touch]]] = True

    # Distance through the ocean to the nearest front: edges ocean-ocean and ocean-front.
    w = oo | (src[i] & oc[j]) | (oc[i] & src[j])
    gw = coo_matrix((dkm[w], (i[w], j[w])), shape=(n, n)).tocsr()
    srcs = np.flatnonzero(src)
    reach = dijkstra(gw, directed=False, indices=srcs, min_only=True) if len(srcs) else np.full(n, np.inf)

    # Own plate: continent distance (margin zone), own trench distance.
    ids, inv = np.unique(plate, return_inverse=True)
    share = np.bincount(inv, minlength=len(ids)) / N
    dco = np.full(n, np.inf)
    dto = np.full(n, np.inf)
    trench = (flags & TRENCH) != 0
    for k in range(len(ids)):
        m = inv == k
        mo = m & oc
        if not mo.any():
            continue
        for sel, out in ((m & cont, dco), (m & trench, dto)):
            if sel.any():
                dd, _ = cKDTree(p[sel]).query(p[mo])
                out[mo] = km(dd)
    margin = oc & (dco <= margin_km)
    giant = oc & (share[inv] > GIANT)

    vis = sum(q[1] for q in step["plates"]) or 1.0
    res = {"t": t, "ocean": int(oc.sum()), "ocean_components": int(len(np.unique(comp[oc]))),
           "main_share": float(comp_size[main] / oc.sum()), "front_km_map": float(sub.sum() * spacing),
           "front_km_rec": float(step["mix_km"][0] + step["mix_km"][1]),
           "speed_mean_rec": float(sum(q[1] * q[3] for q in step["plates"]) / vis), "plates": len(ids)}
    for lim in OLD:
        old = oc & (age > lim)
        tag = f"{lim:g}"
        in_main = comp == main
        closed = old & ~in_main
        res[f"old{tag}"] = int(old.sum())
        res[f"old{tag}_margin"] = int((old & margin).sum())
        res[f"old{tag}_giant"] = int((old & giant).sum())
        res[f"old{tag}_closed_basin"] = int(closed.sum())
        res[f"old{tag}_closed_nofront"] = int((closed & ~comp_has_src[comp]).sum())
        res[f"old{tag}_main_nofront"] = int((old & in_main & ~comp_has_src[comp]).sum())
        res[f"old{tag}_reach"] = bands(reach[old], REACH_EDGES)
        res[f"old{tag}_reach_margin"] = bands(reach[old & margin], REACH_EDGES)
        res[f"old{tag}_own"] = bands(dto[old], REACH_EDGES)
        res[f"old{tag}_own_margin"] = bands(dto[old & margin], REACH_EDGES)
        res[f"old{tag}_notrench_plate"] = int((old & ~np.isfinite(dto)).sum())
    res["ocean_reach"] = bands(reach[oc], REACH_EDGES)
    res["ocean_margin"] = int(margin.sum())

    # Passive margins: continental samples with an oceanic neighbour of their own plate, and its age.
    same = plate[i] == plate[j]
    pm = cont[i] & oc[j] & same
    coast_any = np.zeros(n, bool)
    coast_any[i[cont[i] & oc[j]]] = True
    mage = np.full(n, -np.inf)
    np.maximum.at(mage, i[pm], age[j[pm]])
    passive = np.isfinite(mage)
    res["coast"] = int(coast_any.sum())
    res["passive_coast"] = int(passive.sum())
    res["margin_age"] = bands(mage[passive], MARGIN_AGE_EDGES)
    res["margin_age_mean"] = float(mage[passive].mean()) if passive.any() else None
    res["margin_age_p50"] = float(np.median(mage[passive])) if passive.any() else None
    res["margin_age_max"] = float(mage[passive].max()) if passive.any() else None
    return res


def checks(run):
    pj = run / "plate.json"
    if not pj.is_file():
        return {}
    out = {}
    for m in json.loads(pj.read_text()).get("measures", []):
        if m.get("key") in CHECKS:
            out[m["key"]] = [m.get("value"), m.get("status")]
    return out


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    seeds, t_from = None, -900.0
    for a in sys.argv[1:]:
        if a.startswith("--seeds="):
            seeds = {int(s) for s in a.split("=", 1)[1].split(",")}
        if a.startswith("--from="):
            t_from = float(a.split("=", 1)[1])
    for arm in args:
        for run in sorted(p for p in Path(arm).iterdir() if (p / "rec.json").is_file()):
            if seeds is not None and int(run.name.rsplit("-", 1)[1]) not in seeds:
                continue
            r = json.loads((run / "rec.json").read_text())
            N = r["samples"]
            steps = r["steps"]
            step_t = np.array([s["t"] for s in steps])
            snaps = []
            if (run / "maps" / "age").is_dir():
                for path in sorted((run / "maps").glob("t*.bin")):
                    if float(path.stem[1:]) < t_from:
                        continue
                    s = snapshot(run, path, N, steps[int(np.argmin(np.abs(step_t - float(path.stem[1:]))))])
                    if s is not None:
                        snaps.append(s)
            print(json.dumps({"arm": arm, "run": run.name, "samples": N, "checks": checks(run), "snaps": snaps}),
                  flush=True)


main()
