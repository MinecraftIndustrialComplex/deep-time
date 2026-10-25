#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""What the over-land reach takes from M5 and what the lattice stretch would have taken (RLC-052).

    python3 -I tools/review/m5diag/m5fix_stretch.py <run folder> [<run folder> ...]

Per run, on the Fibonacci points of the final fields with the closed bodies as land: the coast that the
straight-line reach (version 1, same landmass) qualifies and the over-land reach (the straight run when
it stays on ground, else the path) does not, as shares of the coast ("lost"), split by the straight-line
distance to the nearest high point: up to 170 km and 170 to 200 km. "path_only" is the share the path
alone (Dijkstra over points joined within 1.6 spacings, no straight run) would qualify: its gap to
"qualified_path" is the lattice's stretch, which the straight run removes. Prints one JSON line per run
and the medians. Needs numpy and scipy."""
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
    from scipy.sparse import coo_matrix
    from scipy.sparse.csgraph import dijkstra
    from scipy.spatial import cKDTree
    ground = path["ground"]
    gi = np.nonzero(ground)[0]
    pg = p[gi]
    pairs = cKDTree(pg).query_pairs(cc.chord(1.6 * sp), output_type="ndarray")
    wt = cc.arc_km(np.linalg.norm(pg[pairs[:, 0]] - pg[pairs[:, 1]], axis=1))
    g = coo_matrix((wt, (pairs[:, 0], pairs[:, 1])), shape=(gi.size, gi.size)).tocsr()
    dist = np.full(z.size, np.inf)
    dist[gi] = dijkstra(g, directed=False, indices=np.nonzero(path["hi"][gi])[0], limit=200.0, min_only=True)
    path_only = path["coast"] & (dist <= 200.0)
    coast = path["coast"]
    n = max(int(coast.sum()), 1)
    lost = straight["has"] & ~path["has"]
    return {"coast": int(coast.sum()), "qualified_straight": straight["has"].sum() / n,
            "qualified_path": path["has"].sum() / n, "lost": lost.sum() / n,
            "lost_to_170": (lost & (straight["reach"] <= 170.0)).sum() / n,
            "lost_170_to_200": (lost & (straight["reach"] > 170.0)).sum() / n,
            "path_only": path_only.sum() / n}


def main():
    rows = [one(d) for d in sys.argv[1:]]
    for d, r in zip(sys.argv[1:], rows):
        print(json.dumps({"run": Path(d).name, **{k: round(v, 5) for k, v in r.items()}}))
    print("median " + json.dumps({k: round(statistics.median(r[k] for r in rows), 5) for k in rows[0]}))


if __name__ == "__main__":
    main()
