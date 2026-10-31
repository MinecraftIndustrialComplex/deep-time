#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The crust made continental by part (no simulation): o2c.py <summary.txt> ...

Reads each run's last `cb_moves` line (the crust budget, `DEEPTIME_CRUST_BUDGET=1`, cumulative over the run) and
prints, per part of the plate step that turned ocean floor continental, the mean over the runs of the area turned (km^2
a Myr), the crust it counts as continental (the area times the column's thickness less the ocean
basement kept under it, PLT-331; km^3 a year, Earth-normalized) and the whole column (km^3 a year). The sum of the
counted column is loss-budget.py's "crust made continental" less its "basement" row. The hotspot lane (2026-10-31)
wrote it to see which rule turns the plume tails' and plateaus' ocean-floor basalt into continental crust.
"""

import re
import sys

PLANET_KM2 = 4.0 * 3.141592653589793 * 6371.0**2
YRS = 1.0e9  # the runs are 1000 Myr


def main():
    runs = sys.argv[1:]
    if not runs:
        sys.exit(__doc__)
    tot = {}
    for p in runs:
        last = None
        with open(p) as f:
            for line in f:
                if line.startswith("cb_moves t=0 "):
                    last = line
        if last is None:
            sys.exit(f"{p}: no final cb_moves line")
        for part, body in re.findall(r"(\w+)=\[([^\]]*)\]", last):
            kv = dict(x.split("=") for x in body.split())
            a = float(kv.get("o2c", 0.0)) * PLANET_KM2
            t = float(kv.get("o2c_t_km", 0.0))
            b = float(kv.get("o2c_b_km", 0.0))
            d = tot.setdefault(part, [0.0, 0.0, 0.0])
            d[0] += a / (YRS / 1.0e6)
            d[1] += a * (t - b) / YRS
            d[2] += a * t / YRS
    n = len(runs)
    print(f"{'part':14s} {'area km2/Myr':>16s} {'counted km3/yr':>15s} {'whole km3/yr':>13s}")
    for part, (a, c, w) in sorted(tot.items(), key=lambda kv: -kv[1][1]):
        if a > 0:
            print(f"{part:14s} {a / n:16.3f} {c / n:15.3f} {w / n:13.3f}")
    print(f"{'all':14s} {sum(v[0] for v in tot.values()) / n:16.3f} {sum(v[1] for v in tot.values()) / n:15.3f} "
          f"{sum(v[2] for v in tot.values()) / n:13.3f}")


if __name__ == "__main__":
    main()
