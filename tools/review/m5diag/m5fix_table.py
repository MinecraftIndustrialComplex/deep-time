#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The seven realism checks of an arm's cached runs, version 1 against version 2 of the census (the
M5 measure fixes, RLC-055): medians [IQR] over the runs of the two stored measures, and for M5 what
each fix does (coast_hi_old, coast_hi_fib, coast_hi_ocean, coast_hi).

    python3 -I tools/review/m5diag/m5fix_table.py <arm folder> [old measure] [new measure]

The arm folder is the pool's `<pool>/<key>/<preset>-<length>/<argkey>`; the measures default to
`v1census` (old, version 1) and `v1census2` (new). Standard library only."""
import json
import statistics
import sys
from pathlib import Path

SEVEN = [("land", "M1 land"), ("largest", "M2 largest landmass"), ("under_sea", "M3 crust under the sea"),
         ("above_2km", "M4 land above 2 km"), ("coast_hi", "M5 coast with high ground"),
         ("under_ice", "M6 land under ice"), ("collisions", "M7 collisions per 100 Myr")]
STEPS = [("coast_hi_old", "M5 version 1 rule, census grid"), ("coast_hi_fib", "+ Fibonacci points"),
         ("coast_hi_ocean", "+ closed bodies as land"), ("coast_hi", "+ reach over land (all three)"),
         ("closed_sea", "closed bodies, share of the sea's area")]


def load(folder, name):
    out = {}
    for d in sorted(Path(folder).iterdir()):
        f = d / "measures" / f"{name}.json"
        if f.exists():
            out[d.name] = json.loads(f.read_text())["values"]
    return out


def stat(xs):
    xs = sorted(x for x in xs if x is not None)
    if not xs:
        return "n/a"
    q = statistics.quantiles(xs, n=4, method="inclusive") if len(xs) > 1 else [xs[0]] * 3
    return f"{statistics.median(xs):.4f} [{q[0]:.4f}, {q[2]:.4f}]"


def main():
    folder = sys.argv[1]
    old = load(folder, sys.argv[2] if len(sys.argv) > 2 else "v1census")
    new = load(folder, sys.argv[3] if len(sys.argv) > 3 else "v1census2")
    runs = sorted(set(old) & set(new))
    print(f"{len(runs)} runs; medians [IQR]")
    print(f"{'check':34s} {'old':26s} {'new':26s} runs changed")
    for k, label in SEVEN:
        a, b = [old[r][k] for r in runs], [new[r][k] for r in runs]
        ch = sum(1 for x, y in zip(a, b) if x is not None and y is not None and abs(x - y) > 1e-12)
        print(f"{label:34s} {stat(a):26s} {stat(b):26s} {ch}")
    print()
    print("M5, one fix at a time (new measure's keys), medians [IQR]")
    for k, label in STEPS:
        print(f"{label:42s} {stat([new[r].get(k) for r in runs])}")
    for start in ("supercontinent", "scattered_continents"):
        rs = [r for r in runs if r.startswith(start)]
        print(f"{start:34s} M5 old {stat([old[r]['coast_hi'] for r in rs])}  new {stat([new[r]['coast_hi'] for r in rs])}")


if __name__ == "__main__":
    main()
