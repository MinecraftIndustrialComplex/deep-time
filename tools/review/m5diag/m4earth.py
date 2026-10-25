#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Earth's control for the M4 diagnosis: m4earth.py <crust1.bnds> [crust1.rho]

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
    # Earth's M4 by region (boxes of latitude and longitude, the agent's: a stand-in for the model's regime classes).
    lon = np.tile(np.arange(-179.5, 180, 1.0), 180)
    boxes = [
        ("Tibet, Himalaya, Pamir, Karakoram (collision)", (26, 40, 68, 105)),
        ("Alps to Zagros and Iran (collision)", (28, 48, -10, 68)),
        ("Tien Shan, Altai, Mongolia (far-field of India-Asia)", (40, 56, 68, 125)),
        ("Andes (Andean margin)", (-56, 12, -82, -60)),
        ("North American Cordillera and Mexico", (14, 70, -170, -100)),
        ("Africa and Arabia (plateaus, rifts)", (-35, 32, -18, 60)),
        ("Antarctica", (-90, -60, -180, 180)),
        ("Greenland", (59, 84, -75, -10)),
    ]
    taken = np.zeros_like(land)
    regions = []
    for name, (la0, la1, lo0, lo1) in boxes:
        k = land & ~taken & (lat >= la0) & (lat < la1) & (lon >= lo0) & (lon < lo1)
        taken |= k
        regions.append((name, k))
    regions.append(("the rest", land & ~taken))
    print("Earth's M4 by region (share of all land): M4, of it ice-free; the region's land; crust p50 under its high"
          " ground; the region's crust of 45 to 55 km (share of all land) and its rock p50")
    for name, k in regions:
        kh = k & hi
        kb = k & (T >= 45) & (T < 55)
        print(f"  {name:55s} m4 {w[kh].sum() / WL:.4f} ({w[kh & ~iced].sum() / WL:.4f})  land {w[k].sum() / WL:.4f}"
              f"  T p50 {wq(T[kh], w[kh], 0.5):.1f}  45-55 km {w[kb].sum() / WL:.4f} at {wq(rock[kb], w[kb], 0.5):.0f} m")
    if len(sys.argv) > 2:
        densities(b, np.loadtxt(sys.argv[2]), w, land & ~iced, iced, rock, T, WL, w[hi].sum() / WL)


def fit(x, y, w):
    A = np.vstack([x, np.ones_like(x)]).T * np.sqrt(w)[:, None]
    s, c = np.linalg.lstsq(A, y * np.sqrt(w), rcond=None)[0]
    return float(s), float(c)


def densities(b, rho, w, m, iced_land, rock, T, WL, m4):
    """Earth's ice-free land against its own crust's buoyancy (CRUST1.0's layer densities, crust1.rho): the crust's
    mean density by band, the Airy height of each column with its own layers over the model's mantle (3,300) and over
    CRUST1.0's own mantle, and the rock height against both; then Earth's M4 with every column moved from its own
    layers' Airy height to the model's (one crustal density 2,830 kg/m3), its residual kept."""
    assert rho.shape == b.shape, rho.shape
    t = (b[:, 2:8] - b[:, 3:9])                      # km: three sediment and three crystalline layers
    r = rho[:, 2:8]                                   # g/cm3
    Tk = t.sum(axis=1)
    rbar = np.where(Tk > 0, (t * r).sum(axis=1) / np.maximum(Tk, 1e-9), np.nan)
    h_own = 1000.0 * (t * (3.30 - r)).sum(axis=1) / 3.30          # m of buoyancy over a 3,300 mantle
    rm = rho[:, 8]
    h_own_m = 1000.0 * (t * (rm[:, None] - r)).sum(axis=1) / rm   # over CRUST1.0's own uppermost mantle
    h_uni = 1000.0 * Tk * (3.30 - 2.83) / 3.30                    # the model's line, PLT-281
    s_rT, c_rT = fit(T[m], rock[m], w[m])
    s_hT, _ = fit(T[m], h_own[m], w[m])
    s_hmT, _ = fit(T[m], h_own_m[m], w[m])
    s_rh, c_rh = fit(h_own[m], rock[m], w[m])
    print("\nEarth's ice-free land and its own crust's buoyancy (CRUST1.0 densities):")
    print(f"  rock height on T: {c_rT:.0f} m + {s_rT:.0f} m/km; the own-layer Airy height on T: {s_hT:.0f} m/km"
          f" over 3,300 ({s_hmT:.0f} over CRUST1.0's mantle); the model's line 142 m/km")
    print(f"  rock height on the own-layer Airy height: slope {s_rh:.2f}, intercept {c_rh:.0f} m")
    print("  band km: land share, crust mean density p50 (p10, p90) kg/m3, mantle p50, rock p50, own-layer Airy p50"
          " (minus the model line's at the same T), rock minus own-layer Airy p50")
    for a, c in BANDS + FINE:
        k = m & (T >= a) & (T < c)
        if w[k].sum() == 0:
            continue
        print(f"  {a:g}-{c:g}: land {w[k].sum() / WL:.4f}  rho {1000 * wq(rbar[k], w[k], 0.5):.0f}"
              f" ({1000 * wq(rbar[k], w[k], 0.1):.0f}, {1000 * wq(rbar[k], w[k], 0.9):.0f})"
              f"  mantle {1000 * wq(rm[k], w[k], 0.5):.0f}  rock {wq(rock[k], w[k], 0.5):.0f}"
              f"  own {wq(h_own[k], w[k], 0.5):.0f} ({wq(h_own[k] - h_uni[k], w[k], 0.5):.0f})"
              f"  rock-own {wq(rock[k] - h_own[k], w[k], 0.5):.0f}")
    # Earth's M4 with the model's one density: each ice-free column moves by (h_uni - h_own); iced cells as they are.
    hi_iced = w[iced_land & (rock > 2000.0)].sum() / WL
    for name, sh in (("over 3,300", h_uni - h_own), ("over CRUST1.0's mantle", h_uni - h_own_m)):
        hi2 = m & (rock + sh > 2000.0)
        print(f"  Earth's M4 with the model's one crustal density ({name}): ice-free land {w[hi2].sum() / WL:.4f}"
              f" + iced as observed {hi_iced:.4f} = {w[hi2].sum() / WL + hi_iced:.4f}"
              f" (observed {m4:.4f}, of it ice-free {w[m & (rock > 2000.0)].sum() / WL:.4f})")


if __name__ == "__main__":
    main()
