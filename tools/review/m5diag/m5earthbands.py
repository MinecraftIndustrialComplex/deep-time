#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Earth's continental crust by thickness band, the drowned part included: m5earthbands.py <crust1.bnds>

The drowning diagnosis (M2 and M3 after the motion fix) asks whether the model's 35 to 45 km crust stands too low for its
thickness or is too thin. m4earth.py reads CRUST1.0's land only; this reads all of it. CRUST1.0 (Laske, Masters, Ma &
Pasyanos 2013; 1 x 1 degree boundaries file, read as m4earth.py reads it: column 0 the top of the water, 1 the top of
the ice, 2 the rock (the top of the sediments), 8 the Moho; cells weighted by cos latitude; heights over Earth's sea).

Prints:
1. the whole planet: the share under water, the land's mean height (the surface, ice included) and its mean rock
   height, the ocean's mean depth and volume over the 1-degree cells;
2. the share of the planet whose crust (sediments and crystalline crust) is at least T_min thick, for several T_min,
   with the share of it under water, so the continental share is read at the threshold that puts Earth's margins in;
3. at T_min = 25 km, by band of thickness: the band's share of the continental crust, the land share in the band, the
   rock height p25 / p50 / p75 over the sea, the share above 2 km, and the model's own Airy line at the same thickness
   (z_0 525 m at 35.1 km, 7.02 m of crust per m: PLT-281's 2,830 over 3,300) with the sea at the datum.
Needs numpy only."""
import sys

import numpy as np

BANDS = [(25, 30), (30, 35), (35, 40), (40, 45), (45, 50), (50, 55), (55, 70), (70, 1e9), (35, 45)]
T_MINS = [15, 20, 25, 28, 30, 32]
OC_BANDS = [(0, 8), (8, 10), (10, 15), (15, 25), (25, 1e9)]
EARTH_R_KM = 6371.0


def wq(x, w, q):
    if x.size == 0:
        return float("nan")
    o = np.argsort(x)
    c = np.cumsum(w[o]) / w[o].sum()
    return float(np.interp(q, c, x[o]))


def model_line(t_km):
    """The model's continental Airy height over the datum (PLT-281): 525 m at 35.1 km, 1 m per 7.02 m of crust."""
    return 525.0 + (t_km - 35.1) * 1000.0 / 7.02


def main():
    b = np.loadtxt(sys.argv[1])
    assert b.shape == (64800, 9), b.shape
    lat = np.repeat(np.arange(89.5, -90, -1.0), 360)
    w = np.cos(np.radians(lat))
    top, ice_top, rock, moho = b[:, 0] * 1000, b[:, 1] * 1000, b[:, 2] * 1000, b[:, 8] * 1000
    water = top - ice_top
    T = (rock - moho) / 1000.0
    wet = water > 0.0
    W = w.sum()
    area_km2 = 4 * np.pi * EARTH_R_KM ** 2
    print("1. The whole planet (CRUST1.0, 1 x 1 degree, cos-latitude weights)")
    print(f"   under water {w[wet].sum() / W:.3f}; land {w[~wet].sum() / W:.3f}")
    print(f"   land: mean surface height {np.average(top[~wet], weights=w[~wet]):.0f} m (ice included),"
          f" mean rock height {np.average(rock[~wet], weights=w[~wet]):.0f} m")
    depth = np.average(water[wet], weights=w[wet])
    vol = depth / 1000.0 * area_km2 * w[wet].sum() / W
    print(f"   ocean: mean depth {depth:.0f} m, volume {vol:.3e} km3 (cells under water)")
    print("2. The share of the planet with crust at least T_min thick, and the part of it under water")
    for tm in T_MINS:
        m = T >= tm
        print(f"   T >= {tm:2d} km: {w[m].sum() / W:.3f} of the planet, {w[m & wet].sum() / max(w[m].sum(), 1e-9):.3f}"
              f" of it under water; its mean rock height {np.average(rock[m], weights=w[m]):.0f} m")
    print("2b. The sea floor by the crust's thickness under it (sediments included): share of the sea's area, mean"
          " depth of the rock below the sea m")
    Wt = w[wet].sum()
    for a, z in OC_BANDS:
        m = wet & (T >= a) & (T < z)
        if m.any():
            print(f"   {a:g}-{z:g} km: share {w[m].sum() / Wt:.3f}, rock depth mean {-np.average(rock[m], weights=w[m]):.0f}"
                  f" m, its sediment mean {np.average((rock[m] - b[m, 5] * 1000), weights=w[m]):.0f} m")
    oc = wet & (T < 15.0)
    print(f"   crust under 15 km: mean rock depth {-np.average(rock[oc], weights=w[oc]):.0f} m, mean water depth"
          f" {np.average(water[oc], weights=w[oc]):.0f} m")
    cont = T >= 25.0
    C = w[cont].sum()
    print("3. Continental crust (T >= 25 km) by band: share of it, land share in the band, rock height p25 / p50 / p75 m"
          " over the sea, share above 2 km, the model's Airy line over the datum")
    for a, z in BANDS:
        m = cont & (T >= a) & (T < z)
        if not m.any():
            continue
        r, ww = rock[m], w[m]
        mid = min(0.5 * (a + z), a + 7.5)
        print(f"   {a:g}-{z:g} km: share {ww.sum() / C:.3f}, land {ww[~wet[m]].sum() / ww.sum():.3f},"
              f" rock {wq(r, ww, 0.25):.0f} / {wq(r, ww, 0.5):.0f} / {wq(r, ww, 0.75):.0f},"
              f" above 2 km {ww[r > 2000].sum() / ww.sum():.3f},"
              f" model line at T p50 {model_line(wq(T[m], ww, 0.5)):.0f} (band middle {model_line(mid):.0f})")
    m = cont & ~wet
    sl, ic = np.polyfit(T[m], rock[m], 1, w=w[m])
    print(f"   land on continental crust: rock = {ic:.0f} m + {sl:.0f} m/km x T; all continental crust:"
          f" rock = {np.polyfit(T[cont], rock[cont], 1, w=w[cont])[1]:.0f} m"
          f" + {np.polyfit(T[cont], rock[cont], 1, w=w[cont])[0]:.0f} m/km x T")
    print(f"   continental crust's thickness p10 / p50 / p90 {wq(T[cont], w[cont], 0.1):.1f} / {wq(T[cont], w[cont], 0.5):.1f}"
          f" / {wq(T[cont], w[cont], 0.9):.1f} km, mean {np.average(T[cont], weights=w[cont]):.1f};"
          f" on land {np.average(T[m], weights=w[m]):.1f}")


if __name__ == "__main__":
    main()
