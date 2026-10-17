#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Tabulate the land's gap budget (crates/deeptime-core/src/plates/gap_budget.rs, ledger PLT-283)
from run summaries made with DEEPTIME_GAP_BUDGET=1, mean over runs (min and max beside it).

  python3 tools/review/gap-budget.py [--t T ...] [--bands 30-40km,40-55km] <summary.txt> ...

For each snapshot time T (default: the middle and the last snapshot of the first run) and each
band: the share of continental samples, the mean elevation, Airy height and gap, then every part
of the gap two ways: the stock (m of the band's mean gap that the part made, since each sample's
birth or the run's start) and the rate (m/Myr per column over the 50 Myr before the snapshot).
"""
import argparse
import re
import sys
from collections import defaultdict

STOCK = re.compile(
    r"^gap_stock t=(-?\d+) band=(\S+) n=(\S+) z=(\S+) iso=(\S+) gap=(\S+) gap_p50=(\S+) "
    r"ocean=(\S+) parts=(\S+)$"
)
RATE = re.compile(r"^gap_rate t=(-?\d+) band=(\S+) m_per_myr=(\S+)")
HEAD = re.compile(r"^gap_budget z0_m=(\S+) restarts=(\S+) worst_residual_m=(\S+) parts=(\S+)$")


def read(path):
    names, head, stock, rate = None, None, {}, {}
    with open(path) as f:
        for line in f:
            line = line.strip()
            m = HEAD.match(line)
            if m:
                names = m.group(4).split(",")
                head = (float(m.group(1)), int(m.group(2)), float(m.group(3)))
                continue
            m = STOCK.match(line)
            if m:
                t, band = int(m.group(1)), m.group(2)
                vals = [float(x) for x in m.group(3, 4, 5, 6, 7, 8)]
                parts = [float(x) for x in m.group(9).split("/")]
                stock[(t, band)] = (vals, parts)
                continue
            m = RATE.match(line)
            if m:
                rate[(int(m.group(1)), m.group(2))] = [float(x) for x in m.group(3).split("/")]
    return names, head, stock, rate


def stats(xs):
    return sum(xs) / len(xs), min(xs), max(xs)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--t", type=int, nargs="*")
    ap.add_argument("--bands", default="30-40km,40-55km,55-70km")
    ap.add_argument("runs", nargs="+")
    a = ap.parse_args()
    runs = [(p, *read(p)) for p in a.runs]
    runs = [r for r in runs if r[1]]
    if not runs:
        sys.exit("no gap_budget lines in the summaries")
    names = runs[0][1]
    for p, _, head, _, _ in runs:
        print(f"# {p}: z0 {head[0]:.0f} m, restarts {head[1]}, worst residual {head[2]:.1f} m")
    times = sorted({t for (t, _) in runs[0][3]})
    ts = a.t or [times[len(times) // 2 - 1], times[-1]]
    bands = a.bands.split(",")
    for t in ts:
        have = [r for r in runs if (t, bands[0]) in r[3]]
        print(f"\n## t = {t} Myr ({len(have)} runs; mean [min, max])")
        print("| | " + " | ".join(bands) + " |")
        print("|---" * (len(bands) + 1) + "|")
        for k, label in enumerate(["share of continental samples", "elevation, m", "Airy height, m",
                                   "gap, m", "gap median, m", "of it made while oceanic, m"]):
            cells = []
            for b in bands:
                xs = [r[3][(t, b)][0][k] for r in have if (t, b) in r[3]]
                m, lo, hi = stats(xs)
                cells.append(f"{m:.3f} [{lo:.3f}, {hi:.3f}]" if k == 0 else f"{m:.0f} [{lo:.0f}, {hi:.0f}]")
            print(f"| {label} | " + " | ".join(cells) + " |")
        print("\nStock by part, m (mean [min, max]); rate, m/Myr per column (mean):")
        print("| part | " + " | ".join(f"{b} stock | {b} rate" for b in bands) + " |")
        print("|---" * (2 * len(bands) + 1) + "|")
        totals = defaultdict(float)
        for i, name in enumerate(names):
            cells = []
            for b in bands:
                xs = [r[3][(t, b)][1][i] for r in have if (t, b) in r[3]]
                rs = [r[4][(t, b)][i] for r in have if (t, b) in r[4]]
                m, lo, hi = stats(xs)
                rm = stats(rs)[0]
                totals[(b, 0)] += m
                totals[(b, 1)] += rm
                cells.append(f"{m:.0f} [{lo:.0f}, {hi:.0f}] | {rm:.2f}")
            print(f"| {name} | " + " | ".join(cells) + " |")
        print("| sum | " + " | ".join(f"{totals[(b, 0)]:.0f} | {totals[(b, 1)]:.2f}" for b in bands) + " |")


if __name__ == "__main__":
    main()
