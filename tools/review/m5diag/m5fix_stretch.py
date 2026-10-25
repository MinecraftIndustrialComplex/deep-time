#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""How much of the over-land reach's fall is the water and how much the lattice's stretch (RLC-052).

    python3 -I tools/review/m5diag/m5fix_stretch.py <run folder> [<run folder> ...]

Per run, on the Fibonacci points of the final fields with the closed bodies as land: the coast the
straight-line reach (version 1, same landmass) qualifies and the over-land reach does not, as shares of
the coast, split by the straight-line distance to the nearest high point: up to 170 km (a strait or a
bay stands in the way; a path stretched by the lattice, at most 1.15, still fits 200 km) and 170 to
200 km (the stretch alone could do it). Prints one JSON line per run and the medians.
Needs numpy and scipy."""
import contextlib
import io
import json
import statistics
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
    sp = float(np.sqrt(cc.EARTH_KM2 / (int(ok_r.sum()) * w)))
    row, col, p = cc.fib_sample(h, w, r)
    z = ((f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V)[row, col]
    with contextlib.redirect_stdout(io.StringIO()):
        straight = cc.coast_geometry(p, z, sp, over_land=False)
        path = cc.coast_geometry(p, z, sp)
    coast = path["coast"]
    n = max(int(coast.sum()), 1)
    lost = straight["has"] & ~path["has"]
    return {"coast": int(coast.sum()), "qualified_straight": straight["has"].sum() / n,
            "qualified_path": path["has"].sum() / n, "lost": lost.sum() / n,
            "lost_to_170": (lost & (straight["reach"] <= 170.0)).sum() / n,
            "lost_170_to_200": (lost & (straight["reach"] > 170.0)).sum() / n,
            "gained": (path["has"] & ~straight["has"]).sum() / n}


def main():
    rows = [one(d) for d in sys.argv[1:]]
    for d, r in zip(sys.argv[1:], rows):
        print(json.dumps({"run": Path(d).name, **{k: round(v, 5) for k, v in r.items()}}))
    print("median " + json.dumps({k: round(statistics.median(r[k] for r in rows), 5) for k in rows[0]}))


if __name__ == "__main__":
    main()
