#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Where the rift trace's pairs start: each pair's first step, its edges' crust, lithosphere and heat flow (the model's
own law, q = 26 + 3,375 / L mW/m2), against Kusznir & Park's 60 to 65 mW/m2 (significant extension of normal
lithosphere needs that much under 3 TN/m) and Buck's wide band.

    rift_starts.py <trace folder> [...]
"""
import csv
import sys
from collections import defaultdict
from pathlib import Path


def q_of(lith):
    return 26.0 + 3375.0 / max(lith, 1.0)


def quant(xs, p):
    xs = sorted(xs)
    if not xs:
        return float("nan")
    return xs[min(len(xs) - 1, max(0, round(p * (len(xs) - 1))))]


for folder in sys.argv[1:]:
    rows = list(csv.DictReader(open(Path(folder) / "steps.tsv"), delimiter="\t"))
    first = {}
    count = defaultdict(int)
    for r in rows:
        key = (r["a"], r["b"])
        count[key] += 1
        if key not in first:
            first[key] = r
    qs, crusts, cold_pairs, hot_pairs = [], [], 0, 0
    q_all = []
    for key, r in first.items():
        sides = [s for s in "ab" if int(r[f"{s}_mode"]) > 0]
        qq = [q_of(float(r[f"{s}_lith_km"])) for s in sides]
        cc = [float(r[f"{s}_crust_km"]) for s in sides]
        qs.extend(qq)
        crusts.extend(cc)
        if qq and min(qq) < 65.0:
            cold_pairs += 1
        else:
            hot_pairs += 1
    for r in rows:
        for s in "ab":
            if int(r[f"{s}_mode"]) > 0:
                q_all.append(q_of(float(r[f"{s}_lith_km"])))
    n = len(first)
    print(f"== {folder}: {n} pairs")
    print(
        "first step's edges: heat flow mW/m2 p10/p50/p90",
        [round(quant(qs, x)) for x in (0.1, 0.5, 0.9)],
        "| crust km p10/p50/p90",
        [round(quant(crusts, x), 1) for x in (0.1, 0.5, 0.9)],
        f"| pairs with a side under 65 mW/m2 at the start {cold_pairs / max(n, 1):.2f}",
        f"| sides under 60 {sum(1 for x in qs if x < 60) / max(len(qs), 1):.2f}, under 65 {sum(1 for x in qs if x < 65) / max(len(qs), 1):.2f}",
    )
    print(
        "all side-steps: heat flow p10/p50/p90",
        [round(quant(q_all, x)) for x in (0.1, 0.5, 0.9)],
        f"| under 65 {sum(1 for x in q_all if x < 65) / max(len(q_all), 1):.2f}",
    )
    lives = sorted(count.values())
    print("steps per pair p10/p50/p90", [quant(lives, x) for x in (0.1, 0.5, 0.9)], "| one-step pairs", sum(1 for x in lives if x == 1))
