#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""How far Earth's trenches lie from the floor's spreading ridges (no simulation):

    earth_ridge.py <PB2002_steps.dat> <crust1.bnds> <Seton2020 .grd> [threshold_km]

Bird 2003's subduction steps (class SUB), classed ocean-ocean or ocean-continent as earth_fronts.py classes them
(CRUST1.0 crystalline crust of at least the threshold, 15 km by default, within 200 km on the overriding side,
PLT-370), with the floor going under (the first dated floor 20 to 200 km out on the subducting side, Seton et al.
2020). For each step: the great-circle distance to the nearest floor younger than 5 Myr on the subducting side of
the step's great circle (Seton's 6-minute cells; the spreading ridges and their flanks, back-arc basins included where
they lie on that side), and the half rate that distance implies with the floor's age at the trench (km / Myr =
mm/yr). Prints, as slabfloor.py prints the model's intra-oceanic fronts, each class's length on floor younger than 55
Myr and older, banded by the ridge's distance and the implied rate, with the length-weighted median distance.

The model's young floor near its ridges is made at resamplings (slabfloor.py reads its ridge as floor within 5 Myr of
the planet's youngest); Earth's grid is the present day, so floor younger than 5 Myr is its ridges' flanks.
"""
import math
import sys

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, __import__("os").path.dirname(__import__("os").path.abspath(__file__)))
import earth_fronts as ef  # noqa: E402

RIDGE_MYR = 5.0
YOUNG_MYR = 55.0
DIST = [0, 500, 1000, 2000, 4000]
RATE = [0, 10, 20, 40, 80]


def band(edges, x):
    if not np.isfinite(x):
        return "none"
    for lo, hi in zip(edges, edges[1:] + [np.inf]):
        if lo <= x < hi:
            return f"{lo:g}-{hi:g}" if np.isfinite(hi) else f">{lo:g}"
    return "none"


def main(argv):
    steps = [s for s in ef.read_steps(argv[0]) if s["cls"] == "SUB"]
    crust = ef.read_crust(argv[1])
    grid = ef.read_age(argv[2])
    th = float(argv[3]) if len(argv) > 3 else 15.0
    lon, lat, z = grid
    ii, jj = np.nonzero(np.isfinite(z) & (z < RIDGE_MYR))
    la, lo = np.radians(lat[ii]), np.radians(lon[jj])
    young = np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], 1)
    tree = cKDTree(young)
    rows = []
    for s in steps:
        a, b = ef.unit(s["lon0"], s["lat0"]), ef.unit(s["lon1"], s["lat1"])
        m = a + b
        m /= np.linalg.norm(m)
        t = b - a
        t -= m * m.dot(t)
        t /= np.linalg.norm(t)
        left = np.cross(m, t)
        over = left if s["bid"][2] == "/" else -left
        cmax = 0.0
        for d in range(20, 201, 20):
            ang = d / ef.R_KM
            p = m * math.cos(ang) + over * math.sin(ang)
            i, j = ef.cell(p)
            cmax = max(cmax, crust[i, j])
        oo = cmax < th
        sa = np.nan
        for d in range(20, 201, 20):
            ang = d / ef.R_KM
            q = m * math.cos(ang) - over * math.sin(ang)
            x = ef.age_at(grid, q)
            if np.isfinite(x):
                sa = x
                break
        dist = np.inf
        k = 64
        while k <= len(young):
            dd, idx = tree.query(m, k=min(k, len(young)))
            side = young[idx] @ over < 0.0
            if side.any():
                chord = dd[np.argmax(side)]
                dist = 2.0 * math.asin(min(1.0, chord / 2.0)) * ef.R_KM
                break
            if k >= len(young):
                break
            k *= 4
        rows.append({"oo": oo, "km": s["length"], "age": sa, "dist": dist,
                     "rate": dist / sa if np.isfinite(dist) and np.isfinite(sa) and sa > 0 else np.inf})
    print(f"# Bird 2003 SUB steps {len(rows)}, {sum(r['km'] for r in rows):.0f} km; ocean-ocean at {th:.0f} km of"
          f" CRUST1.0 crystalline crust; ridge: Seton et al. 2020 floor under {RIDGE_MYR:.0f} Myr on the subducting side")
    for cls, name in ((True, "ocean-ocean"), (False, "ocean-continent")):
        for yv, yname in ((True, f"floor under {YOUNG_MYR:.0f} Myr"), (False, f"floor {YOUNG_MYR:.0f} Myr or older"),
                          (None, "undated floor")):
            sel = [r for r in rows if r["oo"] == cls and (
                (yv is None and not np.isfinite(r["age"])) or
                (yv is not None and np.isfinite(r["age"]) and (r["age"] < YOUNG_MYR) == yv))]
            tot = sum(r["km"] for r in sel)
            if not tot:
                continue
            print(f"  {name}, {yname}: {tot:.0f} km")
            for title, edges, key in (("ridge distance, km", DIST, "dist"), ("implied half rate, mm/yr", RATE, "rate")):
                acc = {}
                for r in sel:
                    bb = band(edges, r[key])
                    acc[bb] = acc.get(bb, 0.0) + r["km"]
                order = [band(edges, e) for e in edges] + ["none"]
                print(f"    {title:26s} " + "  ".join(f"{bb} {acc[bb] / tot:.3f}" for bb in order if bb in acc))
            fin = sorted((r["dist"], r["km"]) for r in sel if np.isfinite(r["dist"]))
            if fin:
                d = np.array([x[0] for x in fin])
                w = np.cumsum([x[1] for x in fin]) / sum(x[1] for x in fin)
                print(f"    ridge distance, km, length-weighted median {d[np.searchsorted(w, 0.5)]:.0f}"
                      f" (p25 {d[np.searchsorted(w, 0.25)]:.0f}, p75 {d[np.searchsorted(w, 0.75)]:.0f})")


if __name__ == "__main__":
    main(sys.argv[1:])
