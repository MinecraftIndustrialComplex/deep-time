#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Earth's land across its coasts, the Earth reference of the pool's `coastwalk` measure
(tools/review/pool/measure_coastwalk.py, ledger PLT-513): from each coast point (land within 1.6
spacings of the sea, as continents_census.py), walk inland along the great circle away from its
nearest sea point in 25 km steps to 2,000 km, to the first sea point.

    curl -sSfLO https://oceania.generic-mapping-tools.org/server/earth/earth_relief/earth_relief_15m_p.grd
    python3 -I tools/review/earth-across.py earth_relief_15m_p.grd [N]

Needs numpy, scipy and h5py (on the VM through nix shell, as earth-coast.py). Ice-covered rock
counts as land (surface relief). Printed on earth_relief_15m_p, N = 332,800 (2026-10-23): all
coast 600 km median across and 0.51 of it 600 km or less; below 60 degrees of latitude 675 km and
0.49.
"""
import importlib.util
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import numpy as np  # noqa: E402
from scipy.spatial import cKDTree  # noqa: E402

import continents_census as cc  # noqa: E402

spec = importlib.util.spec_from_file_location("epd", HERE / "earth-paleodem.py")
epd = importlib.util.module_from_spec(spec)
spec.loader.exec_module(epd)

STEP, MAX = 25.0, 2000.0


def main():
    z, lat, lon = epd.read_grid(sys.argv[1])
    n = int(sys.argv[2]) if len(sys.argv) > 2 else 332_800
    plat, plon = epd.fib(n)
    iy = np.clip(np.searchsorted(lat, plat), 0, lat.size - 1)
    ix = np.clip(np.searchsorted(lon, plon), 0, lon.size - 1)
    zp = z[iy, ix]
    p = cc.xyz(plat, plon)
    sp = float(np.sqrt(cc.EARTH_KM2 / n))
    land = zp > 0
    sea_idx = np.nonzero(~land)[0]
    ds, js = cKDTree(p[~land]).query(p[land])
    d_sea = np.zeros(zp.size)
    d_sea[land] = cc.arc_km(ds)
    near = np.full(zp.size, -1)
    near[land] = sea_idx[js]
    coast = np.nonzero(land & (d_sea <= 1.6 * sp))[0]
    tree = cKDTree(p)
    rng = np.random.default_rng(1)
    idx = rng.choice(coast, min(4000, coast.size), replace=False)
    ws = []
    for i in idx:
        c0 = p[i]
        s = p[near[i]] - c0
        t = s - (s @ c0) * c0
        t = -t / max(np.linalg.norm(t), 1e-12)
        d = MAX + STEP
        for k in range(1, int(MAX / STEP) + 1):
            a = k * STEP / 6371.0
            _d, m = tree.query(c0 * np.cos(a) + t * np.sin(a))
            if not land[m]:
                d = k * STEP
                break
        ws.append(d)
    ws = np.array(ws)
    polar = np.abs(plat[idx]) > 60
    print(f"N {n} spacing {sp:.1f} km coast points {coast.size}")
    for name, m in (("all", np.ones(idx.size, bool)), ("lat<=60", ~polar)):
        v = ws[m]
        print(
            f"{name:8s} across_land_med_km {np.median(v):7.1f} le600 {np.mean(v <= 600):.3f} "
            f"le1000 {np.mean(v <= 1000):.3f} n {v.size}"
        )


if __name__ == "__main__":
    main()
