#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Earth's deep sea floor, measured the way tools/review/pool/measure_trench.py measures a run:
the share of the sea floor (ground below 0 m) deeper than 6, 7, 8 and 10 km, the depth
percentiles of the floor and the deepest point, area-weighted (cos latitude) on a GMT
earth_relief grid (SRTM15+, Tozer et al. 2019; netCDF-4), and on N Fibonacci points (the plate
model's sampling). Ledger HYP-050, HYP-051 (the trench budget, 2026-10-22).

    curl -sSfLO https://oceania.generic-mapping-tools.org/server/earth/earth_relief/earth_relief_15m_p.grd
    python3 -I tools/review/earth-deep-floor.py earth_relief_15m_p.grd 100000

Needs numpy and h5py (on the VM: nix shell --impure --expr '(builtins.getFlake
"nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy
p.h5py])' -c python3 -I ...).
"""
import sys

import h5py
import numpy as np

THRESHOLDS_M = (5000.0, 6000.0, 7000.0, 8000.0, 10000.0)


def fib(n):
    i = np.arange(n) + 0.5
    lat = np.arcsin(1 - 2 * i / n)
    lon = (np.pi * (1 + 5 ** 0.5) * i) % (2 * np.pi) - np.pi
    return lat, lon


def wpct(x, w, q):
    o = np.argsort(x)
    c = np.cumsum(w[o])
    c /= c[-1]
    return [float(x[o][np.searchsorted(c, p / 100.0)]) for p in q]


def report(tag, depth, w):
    """depth: m below 0 of the sea floor cells, w: their area weights."""
    tot = w.sum()
    p = wpct(depth, w, [50, 90, 99, 99.9])
    shares = " ".join(f">{t / 1000:.0f}km {w[depth > t].sum() / tot:.4f}" for t in THRESHOLDS_M)
    print(f"{tag}: floor share of the grid {tot:.4f}; deeper than: {shares}; "
          f"p50 {p[0]:.0f} p90 {p[1]:.0f} p99 {p[2]:.0f} p99.9 {p[3]:.0f} max {depth.max():.0f} m")


def main():
    with h5py.File(sys.argv[1], "r") as f:
        z = f["z"][()].astype(np.float64)
        lat = f["lat"][()] if "lat" in f else f["y"][()]
        lon = f["lon"][()] if "lon" in f else f["x"][()]
        attrs = {k: f["z"].attrs[k] for k in f["z"].attrs}
    sf = attrs.get("scale_factor", 1.0)
    off = attrs.get("add_offset", 0.0)
    z = z * float(np.ravel(sf)[0]) + float(np.ravel(off)[0])
    print("shape", z.shape, "lat", lat[0], lat[-1], "lon", lon[0], lon[-1], "z", z.min(), z.max())
    w = np.broadcast_to(np.cos(np.radians(lat))[:, None], z.shape)
    sea = z < 0
    w_all = w.sum()
    report("grid, area-weighted", -z[sea], w[sea] / w_all)
    i = np.unravel_index(np.argmin(z), z.shape)
    print(f"deepest cell {z[i]:.0f} m at lat {lat[i[0]]:.3f} lon {lon[i[1]]:.3f}")
    for n in [int(x) for x in sys.argv[2:]]:
        la, lo = fib(n)
        iy = np.clip(np.searchsorted(lat, np.degrees(la)), 0, len(lat) - 1)
        ix = np.clip(np.searchsorted(lon, np.degrees(lo)), 0, len(lon) - 1)
        e = z[iy, ix]
        s = e < 0
        report(f"N={n} Fibonacci points", -e[s], np.full(s.sum(), 1.0 / n))


main()
