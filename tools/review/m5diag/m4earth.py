#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Earth's control for the M4 diagnosis: m4earth.py <crust1.bnds>

CRUST1.0 (Laske, Masters, Ma & Pasyanos 2013; 1 x 1 degree, the boundaries file), read as
tools/review/earth-crust.py reads it (HYP-040, HYP-041): land is above 0 m with no water on it, the
rock surface the top of the sediments, the crust the sediments and the crystalline crust, cells
weighted by cos latitude. Prints, for the model's thickness bands (m4split.py's BANDS and FINE):
the share of the land, the land above 2 km by rock from the band (a share of all land), the rock
height p50 and the iced share; then the land above 2 km by rock split at the thickness T2 whose
Airy height on the model's line (z_0 525 m at 35.1 km, 7.02 m of crust per m, PLT-281) is 2 km
over a sea S over the datum, for the seas of the model's runs, and the share of the crust thicker
than T2 that stands above 2 km. Needs numpy only."""
import sys

import numpy as np

BANDS = [(0, 30), (30, 35), (35, 40), (40, 45), (45, 55), (55, 70), (70, 1e9)]
FINE = [(44, 46), (46, 48), (48, 49.5), (49.5, 50.5), (50.5, 52), (52, 55), (55, 60), (60, 70)]
FACTOR = 3300.0 / (3300.0 - 2830.0)


def wq(x, w, q):
    if x.size == 0:
        return float("nan")
    o = np.argsort(x)
    c = np.cumsum(w[o]) / w[o].sum()
    return float(np.interp(q, c, x[o]))


def main():
    b = np.loadtxt(sys.argv[1])
    assert b.shape == (64800, 9), b.shape
    lat = np.repeat(np.arange(89.5, -90, -1.0), 360)
    w = np.cos(np.radians(lat))
    top, ice_top, rock = b[:, 0] * 1000, b[:, 1] * 1000, b[:, 2] * 1000
    moho = b[:, 8] * 1000
    water = top - ice_top
    ice = ice_top - rock
    T = (rock - moho) / 1000.0
    land = (water <= 0.0) & (top > 0.0)
    iced = land & (ice > 0.0)
    hi = land & (rock > 2000.0)
    WL = w[land].sum()
    print(f"land above 2 km by rock {w[hi].sum() / WL:.4f} of the land (HYP-041: 0.047); iced land {w[iced].sum() / WL:.4f}")
    print("band km: land share, M4 from the band (of all land), rock p50 m (ice-free p50), iced share")
    for a, c in BANDS + FINE + [(35, 45)]:
        m = land & (T >= a) & (T < c)
        mf = m & ~iced
        print(f"  {a:g}-{c:g}: land {w[m].sum() / WL:.4f}  m4 {w[m & hi].sum() / WL:.4f}"
              f"  rock p50 {wq(rock[m], w[m], 0.5):.0f} ({wq(rock[mf], w[mf], 0.5):.0f})"
              f"  iced {w[m & iced].sum() / max(w[m].sum(), 1e-12):.2f}")
    for S in (227.0, 500.0, 800.0, 1100.0):
        t2 = 35.1 + FACTOR * (2000.0 + S - 525.0) / 1000.0
        thick = land & (T >= t2)
        print(f"sea {S:.0f} m over the datum: T2 {t2:.1f} km; Earth's M4 on crust >= T2 {w[hi & thick].sum() / WL:.4f},"
              f" on thinner crust {w[hi & ~thick].sum() / WL:.4f}; crust >= T2 {w[thick].sum() / WL:.4f} of the land,"
              f" {w[thick & hi].sum() / max(w[thick].sum(), 1e-12):.2f} of it above 2 km"
              f" (ice-free: {w[thick & hi & ~iced].sum() / max(w[thick & ~iced].sum(), 1e-12):.2f})")
    # Earth's high ground by its crust: the thickness p10/p50/p90 under the land above 2 km by rock.
    print(f"crust under the land above 2 km by rock, km: p10 {wq(T[hi], w[hi], 0.1):.1f} p50 {wq(T[hi], w[hi], 0.5):.1f}"
          f" p90 {wq(T[hi], w[hi], 0.9):.1f}; ice-free {wq(T[hi & ~iced], w[hi & ~iced], 0.5):.1f}")


if __name__ == "__main__":
    main()
