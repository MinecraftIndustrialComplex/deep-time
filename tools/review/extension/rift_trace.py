#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Reads the rift trace (crates/deeptime-core/src/plates/rift_trace.rs) of one or more runs.

    rift_trace.py <trace folder> [<trace folder> ...] [--pairs N]

Each folder holds steps.tsv and lays.tsv (DEEPTIME_RIFT_TRACE). Per run: the rifts' totals, the
stretched crust by the side's mode (narrow or wide), and per pair of plates (the N that stretched
most) the opening before break-up (the sum of u dt, asked_km2 / length_km, over the pair's steps
up to its first resampling with fewer gap samples laid than the gap holds), the share of its
side-steps that were wide, its edge columns' crust and Moho at its first and last steps, how far
it reached and thinned, and the area it thinned a step (columns thinned x a sample's area).
"""

import csv
import statistics
import sys
from collections import defaultdict
from pathlib import Path


def rows(path):
    if not path.exists():
        return []
    with path.open() as f:
        return [
            {k: float(v) for k, v in r.items()}
            for r in csv.DictReader(f, delimiter="\t")
        ]


def q(xs, p):
    xs = sorted(xs)
    if not xs:
        return float("nan")
    i = min(len(xs) - 1, max(0, round(p * (len(xs) - 1))))
    return xs[i]


def run(folder, n_pairs):
    folder = Path(folder)
    steps = rows(folder / "steps.tsv")
    lays = rows(folder / "lays.tsv")
    # A resumed run may write a step twice: keep the last row per (step, a, b).
    seen = {}
    for r in steps:
        seen[(r["step"], r["a"], r["b"])] = r
    steps = sorted(seen.values(), key=lambda r: (r["step"], r["a"], r["b"]))
    print(f"== {folder}")
    if not steps:
        print("no rift rows")
        return
    area = statistics.median(r["sample_km2"] for r in steps)
    t0, t1 = steps[0]["t_myr"], steps[-1]["t_myr"]
    n_steps = len({r["step"] for r in steps})
    asked = sum(r["asked_km2"] for r in steps)
    taken = sum(r["taken_km2"] for r in steps)
    stretched = sum(r["stretched_m"] for r in steps) * area / 1000.0
    by_mode = defaultdict(float)
    side_modes = defaultdict(int)
    for r in steps:
        for s in "ab":
            mode = int(r[f"{s}_mode"])
            side_modes[mode] += 1
            by_mode[mode] += r[f"{s}_given_m"] * area / 1000.0
    span = max(t1 - t0, 1e-9)
    print(
        f"rift rows {len(steps)} over {n_steps} steps ({t0:.0f} to {t1:.0f} Myr), "
        f"{len(steps) / n_steps:.1f} pairs a step; sample {area:.0f} km2"
    )
    print(
        f"opening's area asked {asked:.4g} km2, taken {taken:.4g} ({taken / asked:.3f}); "
        f"stretched {stretched:.4g} km3 ({stretched / span:.4g} km3/Myr)"
    )
    names = {0: "none", 1: "narrow", 2: "wide"}
    for m in sorted(side_modes):
        share = by_mode[m] / stretched if stretched > 0 else float("nan")
        print(
            f"  {names[m]:6s} side-steps {side_modes[m]:6d}, stretched {by_mode[m]:.4g} km3 ({share:.3f})"
        )
    breakups = [r for r in lays if r["breakup"] >= 1]
    laid = [r for r in lays if r["gap"] > 0]
    print(
        f"lays {len(lays)} (with a gap {len(laid)}, break-ups {len(breakups)}, waiting {sum(1 for r in lays if r['gap'] == 0 and r['waits_m'] > 0)}); "
        f"column km median {q([r['column_m'] / 1000 for r in laid], 0.5):.1f}, "
        f"crystalline km median {q([r['crystalline_m'] / 1000 for r in laid], 0.5):.1f}, "
        f"gap samples median {q([r['gap'] for r in laid], 0.5):.0f}"
    )
    # Per pair.
    pairs = defaultdict(list)
    for r in steps:
        pairs[(int(r["a"]), int(r["b"]))].append(r)
    first_break = {}
    for r in sorted(lays, key=lambda r: r["t_myr"]):
        key = (int(r["a"]), int(r["b"]))
        if r["breakup"] >= 1 and key not in first_break:
            first_break[key] = r["t_myr"]
    order = sorted(pairs, key=lambda k: -sum(r["stretched_m"] for r in pairs[k]))
    print(
        "pair      life_myr  steps  open_km  open_to_break  break_t  wide  "
        "crust_km first>last  moho_c first>last  reach_km  far_km  mean_km  thinned_km2 med/max  km3"
    )
    never = 0
    for key in order:
        rs = pairs[key]
        opening = sum(r["asked_km2"] / r["length_km"] for r in rs if r["length_km"] > 0)
        bt = first_break.get(key)
        if bt is None:
            never += 1
            to_break = "never"
            bts = "-"
        else:
            to_break = f"{sum(r['asked_km2'] / r['length_km'] for r in rs if r['length_km'] > 0 and r['t_myr'] <= bt):.0f}"
            bts = f"{bt:.0f}"
        sides = [(r, s) for r in rs for s in "ab" if r[f"{s}_mode"] > 0]
        wide = sum(1 for r, s in sides if r[f"{s}_mode"] == 2) / max(len(sides), 1)

        def edge(r, f):
            xs = [r[f"{s}_{f}"] for s in "ab" if r[f"{s}_mode"] > 0]
            return sum(xs) / len(xs) if xs else float("nan")

        thinned = [(r["a_thinned"] + r["b_thinned"]) * area for r in rs]
        reach = [r[f"{s}_reach_km"] for r, s in sides]
        far = [r[f"{s}_far_km"] for r, s in sides if r[f"{s}_thinned"] > 0]
        mean = [r[f"{s}_mean_km"] for r, s in sides if r[f"{s}_thinned"] > 0]
        km3 = sum(r["stretched_m"] for r in rs) * area / 1000.0
        line = (
            f"{key[0]:3d}-{key[1]:<3d} {rs[-1]['t_myr'] - rs[0]['t_myr']:9.0f} {len(rs):6d} {opening:8.0f} "
            f"{to_break:>13s} {bts:>8s} {wide:5.2f}  {edge(rs[0], 'crust_km'):6.1f} > {edge(rs[-1], 'crust_km'):5.1f}"
            f"      {edge(rs[0], 'moho_c'):5.0f} > {edge(rs[-1], 'moho_c'):5.0f}  {q(reach, 0.5):8.0f} {q(far, 0.5):7.0f} "
            f"{q(mean, 0.5):8.0f}  {q(thinned, 0.5):9.3g}/{max(thinned):9.3g}  {km3:.3g}"
        )
        if order.index(key) < n_pairs:
            print(line)
    print(f"pairs {len(pairs)}, never broke up {never}")
    opens, lives, broke_open = [], [], []
    for key, rs in pairs.items():
        opening = sum(r["asked_km2"] / r["length_km"] for r in rs if r["length_km"] > 0)
        opens.append(opening)
        lives.append(rs[-1]["t_myr"] - rs[0]["t_myr"] + 2.0)
        bt = first_break.get(key)
        if bt is not None:
            broke_open.append(
                sum(r["asked_km2"] / r["length_km"] for r in rs if r["length_km"] > 0 and r["t_myr"] <= bt)
            )
    print(
        "pair opening km p10/p50/p90", [round(q(opens, x)) for x in (0.1, 0.5, 0.9)],
        "| opening before break-up p10/p50/p90", [round(q(broke_open, x)) for x in (0.1, 0.5, 0.9)],
        f"| broke within 100 km {sum(1 for x in broke_open if x <= 100) / max(len(broke_open), 1):.2f}",
        "| life Myr p10/p50/p90", [round(q(lives, x)) for x in (0.1, 0.5, 0.9)],
    )
    for name, mode in (("wide", 2), ("narrow", 1)):
        xs = [(r, s) for r in steps for s in "ab" if r[f"{s}_mode"] == mode and r[f"{s}_thinned"] > 0]
        print(
            f"{name} sides that thinned: {len(xs)}; reach km p50 {q([r[f'{s}_reach_km'] for r, s in xs], 0.5):.0f}, "
            f"far km p50/p90 {q([r[f'{s}_far_km'] for r, s in xs], 0.5):.0f}/{q([r[f'{s}_far_km'] for r, s in xs], 0.9):.0f}, "
            f"thinned km2 per step p50/p90 {q([r[f'{s}_thinned'] * area for r, s in xs], 0.5):.3g}/"
            f"{q([r[f'{s}_thinned'] * area for r, s in xs], 0.9):.3g}, "
            f"edge crust km p50 {q([r[f'{s}_crust_km'] for r, s in xs], 0.5):.1f}"
        )
    lens = [r["length_km"] for r in steps]
    print(
        f"boundary length km p10/p50/p90 {q(lens, 0.1):.0f}/{q(lens, 0.5):.0f}/{q(lens, 0.9):.0f}; "
        f"speed mm/yr p50 {q([r['speed_mm_yr'] for r in steps], 0.5):.1f}"
    )
    # The mode against the edge crust: does a side's mode change as its edge thins?
    bins = [(0, 15), (15, 20), (20, 25), (25, 30), (30, 35), (35, 45), (45, 1000)]
    print("edge crust km   side-steps  wide share  moho_c median  lith_km median")
    for lo, hi in bins:
        xs = [(r, s) for r, s in ((r, s) for r in steps for s in "ab") if r[f"{s}_mode"] > 0 and lo <= r[f"{s}_crust_km"] < hi]
        if not xs:
            continue
        w = sum(1 for r, s in xs if r[f"{s}_mode"] == 2) / len(xs)
        print(
            f"  {lo:3d}-{hi:<4d}      {len(xs):9d}  {w:10.3f}  {q([r[f'{s}_moho_c'] for r, s in xs], 0.5):13.0f}"
            f"  {q([r[f'{s}_lith_km'] for r, s in xs], 0.5):14.0f}"
        )


def main(argv):
    n_pairs = 20
    folders = []
    i = 0
    while i < len(argv):
        if argv[i] == "--pairs":
            n_pairs = int(argv[i + 1])
            i += 2
            continue
        folders.append(argv[i])
        i += 1
    if not folders:
        print(__doc__)
        return 2
    for f in folders:
        run(f, n_pairs)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
