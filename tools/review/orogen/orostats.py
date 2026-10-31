# SPDX-License-Identifier: LGPL-3.0-or-later
"""The orogen lane: what a run's young collision orogens hold, snapshot by snapshot (VM, cached pool runs).

Usage: python3 -I orostats.py <run folder> [t ...]   (t: the map's time, e.g. -100 for maps/t-100.bin)

From the crust-budget maps (`plates/crust_budget.rs`, MAP_FIELDS): the visible samples stamped with a
collision orogeny (orogeny 2) younger than 30 Myr, by elevation band over the run's sea (from the
dynamics record's step at that time), their thickness quantiles, and the land's high ground (z over
the sea above 2 km) by orogeny. Prints one block per snapshot.
"""
import json
import sys
from pathlib import Path

import numpy as np

F = 11  # lat, lon, z, thickness, kind, orogeny, orogeny_age, arc_oc, arc_co, group, flags


def sea_at(rec, t):
    best = None
    for s in rec.get("steps", []):
        if best is None or abs(s["t"] - t) < abs(best["t"] - t):
            best = s
    return float(best.get("sea_m") or 0.0) if best else 0.0


def main():
    run = Path(sys.argv[1])
    rec = json.load(open(run / "rec.json"))
    ts = sys.argv[2:] or ["-100", "-50", "0000"]
    for t in ts:
        name = f"t{t}.bin" if t.startswith("-") else f"t{t}.bin"
        p = run / "maps" / name
        if not p.exists():
            print(f"{name}: missing")
            continue
        a = np.fromfile(p, "<f4").reshape(-1, F).astype(np.float64)
        tt = float(t) if t != "0000" else 0.0
        sea = sea_at(rec, tt)
        z = a[:, 2] - sea
        thick = a[:, 3] / 1000.0
        cont = a[:, 4] > 0.5
        coll = (a[:, 5] == 2) & (a[:, 6] < 30.0)
        land = z > 0
        n_land = land.sum()
        print(f"== {run.name} {name}: sea {sea:.0f} m, visible {len(a)}, land {n_land}, young collision {coll.sum()}")
        for lo, hi in ((-1e9, 0), (0, 1000), (1000, 2000), (2000, 3000), (3000, 4000), (4000, 1e9)):
            m = coll & (z > lo) & (z <= hi)
            q = np.quantile(thick[m], [0.25, 0.5, 0.75]) if m.any() else [np.nan] * 3
            print(f"  young collision z {lo:>6.0f}..{hi:<6.0f}: {m.sum():6d}  thickness p25/50/75 {q[0]:.1f}/{q[1]:.1f}/{q[2]:.1f} km")
        hi2 = land & (z > 2000)
        print(f"  land above 2 km {hi2.sum()} ({hi2.sum() / max(n_land, 1):.4f} of the land): young collision "
              f"{(hi2 & coll).sum()}, other collision {(hi2 & (a[:, 5] == 2) & ~coll).sum()}, Andean {(hi2 & (a[:, 5] == 1)).sum()}, "
              f"none {(hi2 & (a[:, 5] == 0)).sum()}")
        mt = cont & coll
        if mt.any():
            print(f"  young collision continental thickness p90/p99/max {np.quantile(thick[mt], 0.9):.1f}/"
                  f"{np.quantile(thick[mt], 0.99):.1f}/{thick[mt].max():.1f} km; z p90/p99/max "
                  f"{np.quantile(z[mt], 0.9):.0f}/{np.quantile(z[mt], 0.99):.0f}/{z[mt].max():.0f} m")


if __name__ == "__main__":
    main()
