#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Earth's crust thickness under its land and the height each thickness stands at, measured the
way the crust budget measures a run (crates/deeptime-core/src/plates/crust_budget.rs: the land is
above 0 m, the thickness bands are its EDGES_KM), from CRUST1.0 (Laske, Masters, Ma & Pasyanos
2013; 1 x 1 degree, the boundaries file crust1.bnds). The land-shape task (build log 2026-10-20):
is the model's thickness distribution too wide, or its height line too steep for thick crust?

    curl -sSfLO http://igppweb.ucsd.edu/~gabi/crust1/crust1.0.tar.gz && tar xzf crust1.0.tar.gz
    python3 -I tools/review/earth-crust.py crust1.bnds

crust1.bnds: 64,800 rows (latitude 89.5 to -89.5, longitude -179.5 to 179.5 within each), nine
columns, the top of each layer in km (positive up): water, ice, upper, middle and lower sediments,
upper, middle and lower crystalline crust, mantle. The surface is the top of the water on land
(the ice surface where there is ice), the rock surface the top of the sediments, the crust is the
sediments and the crystalline crust (the plate model's thickness counts its sediment pools).
Cells are weighted by their area (cos latitude). Needs numpy only. The figures are ledger rows
HYP-040 (the thickness and the heights by band) and HYP-041 (the land above 2 km by its surface
and by its rock).
"""
import sys

import numpy as np

EDGES_KM = [0, 20, 25, 30, 35, 40, 45, 55, 70, 1e9]


def wq(x, w, qs):
    """Weighted quantiles of x."""
    o = np.argsort(x)
    x, w = x[o], w[o]
    c = np.cumsum(w) / w.sum()
    return [float(np.interp(q, c, x)) for q in qs]


def main():
    b = np.loadtxt(sys.argv[1])
    assert b.shape == (64800, 9), b.shape
    lat = np.repeat(np.arange(89.5, -90, -1.0), 360)
    w = np.cos(np.radians(lat))
    top, ice_top, rock = b[:, 0] * 1000, b[:, 1] * 1000, b[:, 2] * 1000
    moho = b[:, 8] * 1000
    water = top - ice_top
    ice = ice_top - rock
    thick = (rock - moho) / 1000.0
    land = (water <= 0.0) & (top > 0.0)
    iced = land & (ice > 0.0)
    print(f"land {w[land].sum() / w.sum():.4f} of the planet; under ice {w[iced].sum() / w[land].sum():.4f} of the land")
    for name, z in [("surface (ice top)", top), ("rock (under the ice)", rock)]:
        hi = land & (z > 2000.0)
        print(f"land above 2 km by its {name}: {w[hi].sum() / w[land].sum():.4f}")
    free = land & ~iced
    hi = free & (top > 2000.0)
    print(f"ice-free land above 2 km: {w[hi].sum() / w[free].sum():.4f} of the ice-free land, "
          f"{w[hi].sum() / w[land].sum():.4f} of all land")
    q = wq(thick[land], w[land], [0.1, 0.25, 0.5, 0.75, 0.9])
    m = np.average(thick[land], weights=w[land])
    sd = np.sqrt(np.average((thick[land] - m) ** 2, weights=w[land]))
    print(f"crust under the land, km: p10/25/50/75/90 {q[0]:.1f}/{q[1]:.1f}/{q[2]:.1f}/{q[3]:.1f}/{q[4]:.1f}, "
          f"mean {m:.1f}, sd {sd:.1f}")
    for sel_name, sel in [("land", land), ("ice-free land", free)]:
        print(f"\n{sel_name}: thickness band, share, rock height p10/50/90 m, surface p50 m, share above 2 km")
        for lo, hi_km in zip(EDGES_KM[:-1], EDGES_KM[1:]):
            s = sel & (thick >= lo) & (thick < hi_km)
            if not s.any():
                continue
            zq = wq(rock[s], w[s], [0.1, 0.5, 0.9])
            sq = wq(top[s], w[s], [0.5])[0]
            above = w[s & (top > 2000.0)].sum() / w[s].sum()
            print(f"  {lo:>3}-{min(hi_km, 999):>3} km  {w[s].sum() / w[sel].sum():.4f}  "
                  f"{zq[0]:>6.0f} {zq[1]:>6.0f} {zq[2]:>6.0f}  {sq:>6.0f}  {above:.3f}")
    # Continental crust as the plate model counts it: everything 25 km and thicker (shelves too).
    cont = thick >= 25.0
    dry = cont & land
    print(f"\ncrust 25 km and thicker: {w[cont].sum() / w.sum():.4f} of the planet, its dry share "
          f"{w[dry].sum() / w[cont].sum():.4f}")
    q = wq(thick[cont], w[cont], [0.1, 0.25, 0.5, 0.75, 0.9])
    m = np.average(thick[cont], weights=w[cont])
    sd = np.sqrt(np.average((thick[cont] - m) ** 2, weights=w[cont]))
    print(f"  its thickness, km: p10/25/50/75/90 {q[0]:.1f}/{q[1]:.1f}/{q[2]:.1f}/{q[3]:.1f}/{q[4]:.1f}, "
          f"mean {m:.1f}, sd {sd:.1f}")
    # The height line: rock height against thickness on the ice-free land, least squares.
    s = free & (thick >= 25.0)
    a, c = np.polyfit(thick[s], rock[s], 1, w=np.sqrt(w[s]))
    print(f"  ice-free land, rock height = {c:.0f} m + {a:.0f} m/km x thickness "
          f"(the model's Airy line: 525 m at 35.1 km, 143 m/km)")


if __name__ == "__main__":
    main()
