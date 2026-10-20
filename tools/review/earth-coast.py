#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Earth's land share, land above 2 km and median distance from land to the coast, measured the
way the crust budget measures a run (crates/deeptime-core/src/plates/crust_budget.rs: land is
above 0 m, the distance is great-circle to the nearest sea point, on N Fibonacci points, the plate
model's sampling), from a GMT earth_relief grid (SRTM15+, Tozer et al. 2019; netCDF-4). Ledger
HYP-035 (build log 2026-10-20, "mountains eat the world").

    curl -sSfLO https://oceania.generic-mapping-tools.org/server/earth/earth_relief/earth_relief_15m_p.grd
    python3 -I tools/review/earth-coast.py earth_relief_15m_p.grd 100000 250000

Needs numpy, scipy and h5py (on the VM: nix shell --impure --expr '(builtins.getFlake
"nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy p.scipy
p.h5py])' -c python3 -I ...). Prints, on earth_relief_15m_p (2026-10-20): N=100000 land 0.2910,
above 2 km 0.1053 of land, coast p50 433 km; N=250000 0.2915, 0.1054, 409 km.
"""
import sys

import h5py
import numpy as np
from scipy.spatial import cKDTree

R = 6371.0


def fib(n):
    i = np.arange(n) + 0.5
    lat = np.arcsin(1 - 2 * i / n)
    lon = (np.pi * (1 + 5 ** 0.5) * i) % (2 * np.pi) - np.pi
    return lat, lon


def main():
    with h5py.File(sys.argv[1], "r") as f:
        keys = list(f.keys())
        z = f["z"][()].astype(np.float64)
        lat = f["lat"][()] if "lat" in f else f["y"][()]
        lon = f["lon"][()] if "lon" in f else f["x"][()]
        attrs = {k: f["z"].attrs[k] for k in f["z"].attrs}
    sf = attrs.get("scale_factor", 1.0)
    off = attrs.get("add_offset", 0.0)
    z = z * float(np.ravel(sf)[0]) + float(np.ravel(off)[0])
    print("keys", keys, "shape", z.shape, "lat", lat[0], lat[-1], "lon", lon[0], lon[-1], "z", z.min(), z.max())
    for n in [int(x) for x in sys.argv[2:]] or [100_000, 250_000]:
        la, lo = fib(n)
        iy = np.clip(np.searchsorted(lat, np.degrees(la)), 0, len(lat) - 1)
        ix = np.clip(np.searchsorted(lon, np.degrees(lo)), 0, len(lon) - 1)
        e = z[iy, ix]
        xyz = np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], axis=1)
        land = e > 0
        tree = cKDTree(xyz[~land])
        d, _ = tree.query(xyz[land])
        dist = 2 * np.arcsin(np.minimum(d / 2, 1)) * R
        print(f"N={n}: land {land.mean():.4f}, above 2 km {np.mean(e[land] > 2000):.4f} of land, "
              f"coast p50 {np.median(dist):.0f} km, p90 {np.percentile(dist, 90):.0f} km, mean {dist.mean():.0f} km")
        # The continental crust's stand-in: the highest 41 % of the planet (Cogley 1984's
        # continental share, HYP-012), and the median distance from it to the deeper floor.
        z41 = np.quantile(e, 0.59)
        up = e > z41
        d, _ = cKDTree(xyz[~up]).query(xyz[up])
        dist = 2 * np.arcsin(np.minimum(d / 2, 1)) * R
        print(f"N={n}: the highest 41 % stands above {z41:.0f} m; its median distance to the deeper floor "
              f"{np.median(dist):.0f} km")


main()
