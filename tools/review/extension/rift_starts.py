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
    # Cold and not thinned before: a side under 60 mW/m2 whose edge crust is 30 km or more (normal lithosphere, which
    # the sources say the available forces cannot extend without heat, magma or an inherited weakness).
    cold_normal = 0
    for key, r in first.items():
        if any(
            int(r[f"{s}_mode"]) > 0 and q_of(float(r[f"{s}_lith_km"])) < 60.0 and float(r[f"{s}_crust_km"]) >= 30.0
            for s in "ab"
        ):
            cold_normal += 1
    stretched = {}
    for r in rows:
        key = (r["a"], r["b"])
        stretched[key] = stretched.get(key, 0.0) + float(r["stretched_m"])
    total = sum(stretched.values())
    cold_keys = [
        key for key, r in first.items()
        if any(int(r[f"{s}_mode"]) > 0 and q_of(float(r[f"{s}_lith_km"])) < 60.0 and float(r[f"{s}_crust_km"]) >= 30.0 for s in "ab")
    ]
    print(
        f"pairs starting with a cold side of normal crust (under 60 mW/m2, 30 km or more): {cold_normal} "
        f"({cold_normal / max(len(first), 1):.2f}); their share of the stretched crust "
        f"{sum(stretched[k] for k in cold_keys) / max(total, 1e-9):.2f}"
    )
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
    # Per step: the rifts' total boundary length and the area they open, against Bird 2003's continental rift
    # boundaries (PB2002 Table 3: 27,472 km, area production 0.011502 m2/s = 0.363 km2/yr).
    by_step = defaultdict(lambda: [0.0, 0.0, 0])
    for r in rows:
        b = by_step[r["step"]]
        b[0] += float(r["length_km"])
        b[1] += float(r["length_km"]) * float(r["speed_mm_yr"]) * 1.0e-6  # km x mm/yr -> km2/yr
        b[2] += 1
    lengths = [b[0] for b in by_step.values()]
    areas = [b[1] for b in by_step.values()]
    pairs_now = [b[2] for b in by_step.values()]
    print(
        "per step: rift boundary km p10/p50/p90",
        [round(quant(lengths, x)) for x in (0.1, 0.5, 0.9)],
        "| opening km2/yr p10/p50/p90",
        [round(quant(areas, x), 3) for x in (0.1, 0.5, 0.9)],
        "| pairs p10/p50/p90",
        [quant(pairs_now, x) for x in (0.1, 0.5, 0.9)],
        "| pairs of 500 km or more p50",
        quant([sum(1 for r in rows if r["step"] == st and float(r["length_km"]) >= 500) for st in by_step], 0.5),
    )
    lives = sorted(count.values())
    print("steps per pair p10/p50/p90", [quant(lives, x) for x in (0.1, 0.5, 0.9)], "| one-step pairs", sum(1 for x in lives if x == 1))
    # What the cold starts start on (traces with the origin columns, since 2026-10-31): a split (one side's plate the
    # other's parent, or both one parent's, born within 10 Myr of the start), an old suture or belt at the edge (a
    # collision stamp on any edge column), magma (a large igneous province boosting either plate), or none of these.
    if rows and "a_belt" in rows[0]:
        def kinds(r):
            t = float(r["t_myr"])
            a, b = r["a"], r["b"]
            out = set()
            pa, pb = r["a_parent"], r["b_parent"]
            ba, bb = float(r["a_born_myr"]), float(r["b_born_myr"])
            if (pa == b or pb == a or (pa == pb and pa != "-1")) and min(t - ba, t - bb) <= 10.0:
                out.add("split")
            if any(int(r[f"{s}_mode"]) > 0 and float(r[f"{s}_belt"]) > 0.0 for s in "ab"):
                out.add("belt")
            if any(int(r[f"{s}_lip"]) > 0 for s in "ab"):
                out.add("magma")
            return out
        for label, keys in (("all", list(first)), ("cold normal", cold_keys)):
            tally = defaultdict(int)
            weak = 0
            for key in keys:
                ks = kinds(first[key])
                for k in ks:
                    tally[k] += 1
                weak += bool(ks - {"split"})
            m = max(len(keys), 1)
            share = {k: round(v / m, 2) for k, v in sorted(tally.items())}
            print(
                f"{label} starts ({len(keys)}): {share}; with a belt or magma {weak / m:.2f}; "
                f"stretched {sum(stretched[k] for k in keys) / max(total, 1e-9):.2f} of the crust"
            )
        none_keys = [k for k in cold_keys if not (kinds(first[k]) - {"split"})]
        print(
            f"cold normal starts with no belt and no magma: {len(none_keys)} ({len(none_keys) / max(len(first), 1):.2f} "
            f"of the pairs), stretched {sum(stretched[k] for k in none_keys) / max(total, 1e-9):.2f} of the crust"
        )
