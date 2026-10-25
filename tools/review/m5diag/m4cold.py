#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Earth's cold thick crust by its layers: m4cold.py <crust1.bnds> <crust1.rho>

Why does Earth's thick crust off the orogens stand low (45 to 55 km at 340 m, m4earth.py)? CRUST1.0 (Laske et al.
2013, read as m4earth.py reads it: land above 0 m with no water, ice-free, cells by cos latitude) by region and band
of crustal thickness: the land share, the rock height p50, each layer's thickness p50 (sediments; upper, middle and
lower crystalline crust) and density p50, the crust's mean density p50, the uppermost mantle's density p50, the
own-layer Airy height over a 3,300 mantle p50 (m4earth.py's `h_own`), its excess over the model's one-density line
(2,830 over 3,300, PLT-281), and the rock height less the own-layer Airy height p50 (the part the crust's own
layers do not explain: the mantle's, or density CRUST1.0 does not resolve). Needs numpy only."""
import sys

import numpy as np

REGIONS = [
    ("East European platform", (45, 70, 20, 60)),
    ("Urals", (50, 66, 55, 62)),
    ("Siberian platform", (55, 75, 85, 130)),
    ("Canadian shield", (45, 70, -100, -60)),
    ("Australia, west and centre", (-35, -15, 113, 140)),
    ("Africa", (-35, 15, -18, 52)),
    ("Brazilian shield", (-30, 5, -60, -35)),
    ("Tibet", (27, 40, 75, 100)),
    ("Basin and Range, Cordillera", (32, 50, -122, -105)),
]
BANDS = [(30, 35), (35, 40), (40, 45), (45, 50), (50, 55), (55, 70), (70, 1e9)]


def wq(x, w, q):
    if x.size == 0:
        return float("nan")
    o = np.argsort(x)
    c = np.cumsum(w[o]) / w[o].sum()
    return float(np.interp(q, c, x[o]))


def main():
    b = np.loadtxt(sys.argv[1])
    rho = np.loadtxt(sys.argv[2])
    lat = np.repeat(np.arange(89.5, -90, -1.0), 360)
    lon = np.tile(np.arange(-179.5, 180, 1.0), 180)
    w = np.cos(np.radians(lat))
    top, ice_top, rock = b[:, 0] * 1000, b[:, 1] * 1000, b[:, 2] * 1000
    T = (b[:, 2] - b[:, 8])
    land = (top - ice_top <= 0.0) & (top > 0.0) & (ice_top - rock <= 0.0)
    t = b[:, 2:8] - b[:, 3:9]
    r = rho[:, 2:8]
    sed = t[:, 0:3].sum(axis=1)
    sed_r = np.where(sed > 0, (t[:, 0:3] * r[:, 0:3]).sum(axis=1) / np.maximum(sed, 1e-9), np.nan)
    rbar = (t * r).sum(axis=1) / np.maximum(T, 1e-9)
    h_own = 1000.0 * (t * (3.30 - r)).sum(axis=1) / 3.30
    h_uni = 1000.0 * T * (3.30 - 2.83) / 3.30
    WL = w[land].sum()
    print("region, band km: land share; rock p50 m; sediments km (rho); upper, middle, lower crust km (rho);"
          " crust mean rho; mantle rho; own-layer Airy m (minus the model line's); rock minus own-layer Airy m")
    for name, (la0, la1, lo0, lo1) in REGIONS + [("all ice-free land", (-90, 90, -180, 180))]:
        reg = land & (lat >= la0) & (lat < la1) & (lon >= lo0) & (lon < lo1)
        print(f"{name}:")
        for a, c in BANDS:
            k = reg & (T >= a) & (T < c)
            if w[k].sum() / WL < 0.0005:
                continue
            q = lambda x: wq(x[k], w[k], 0.5)  # noqa: E731
            sr = sed_r[k]
            srp = wq(sr[np.isfinite(sr)], w[k][np.isfinite(sr)], 0.5)
            print(f"  {a:g}-{c:g}: land {w[k].sum() / WL:.4f}  rock {q(rock):5.0f}  sed {q(sed):4.1f} ({1000 * srp:4.0f})"
                  f"  crust {q(t[:, 3]):4.1f} ({1000 * q(r[:, 3]):4.0f}) {q(t[:, 4]):4.1f} ({1000 * q(r[:, 4]):4.0f})"
                  f" {q(t[:, 5]):4.1f} ({1000 * q(r[:, 5]):4.0f})  rho {1000 * q(rbar):4.0f}"
                  f"  mantle {1000 * q(rho[:, 8]):4.0f}  own {q(h_own):5.0f} ({q(h_own - h_uni):5.0f})"
                  f"  rock-own {q(rock - h_own):6.0f}")


if __name__ == "__main__":
    main()
