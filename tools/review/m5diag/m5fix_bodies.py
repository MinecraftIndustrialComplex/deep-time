#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The bodies of sea of a run's final fields, on the census's Fibonacci points (RLC-052): the world
ocean (the largest body) and the closed ones, largest first, with area and share of the coast each
carries, and what a size cut at 1 Mkm2 (the diagnosis note's first choice) would have called closed.

    python3 -I tools/review/m5diag/m5fix_bodies.py <run folder> [<run folder> ...]

Needs numpy and scipy."""
import sys
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

import continents_census as cc  # noqa: E402


def one(folder):
    f, _g, e = cc.load_dump(Path(folder) / "fields")
    h, w, r = e["h"], e["w"], e["r"]
    _lat, ok_r = cc.lat_of_rows(h, r, w * r)
    n = int(ok_r.sum()) * w
    sp = float(np.sqrt(cc.EARTH_KM2 / n))
    row, col, p = cc.fib_sample(h, w, r)
    z = ((f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V)[row, col]
    sea = z <= 0
    lab = cc.components(p, sea, sp)
    sizes = np.bincount(lab[sea])
    order = np.argsort(-sizes)
    km2 = cc.EARTH_KM2 / n
    geo = cc.coast_geometry(p, z, sp, closed_as_land=False, over_land=False)
    coast_n = max(int(geo["coast"].sum()), 1)
    print(f"{Path(folder).name}: {len(sizes)} bodies of sea, largest {sizes[order[0]] * km2 / 1e6:.1f} Mkm2 "
          f"({sizes[order[0]] / sea.sum():.3f} of the sea)")
    # The coast each closed body carries, by the version-1 rule: land within 1.6 spacings of that body.
    from scipy.spatial import cKDTree
    land = ~sea
    for k in order[1:6]:
        body = sea & (lab == k)
        if not body.any():
            continue
        d, _ = cKDTree(p[body]).query(p[land])
        shore = int((d <= cc.chord(1.6 * sp)).sum())
        print(f"   body {sizes[k] * km2 / 1e6:6.2f} Mkm2: shore points {shore} ({shore / coast_n:.3f} of the coast)")
    small = [s for s in sizes[order[1:]] if s * km2 < 1e6]
    print(f"   closed by connectivity {len(sizes) - 1} bodies, {sizes[order[1:]].sum() * km2 / 1e6:.2f} Mkm2;"
          f" the 1 Mkm2 cut would close {len(small)} bodies, {sum(small) * km2 / 1e6:.2f} Mkm2")


if __name__ == "__main__":
    for d in sys.argv[1:]:
        one(d)
