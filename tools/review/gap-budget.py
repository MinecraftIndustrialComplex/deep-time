#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Tabulate the land's gap budget (crates/deeptime-core/src/plates/gap_budget.rs, ledger PLT-283)
from run summaries made with DEEPTIME_GAP_BUDGET=1, as the mean over the runs.

  python3 tools/review/gap-budget.py [--t T]...  [--bands 30-40km,40-55km] <summary.txt> ...

For each snapshot time T (default: the middle and the last snapshot of the first run): one line
per band with the share of the continental samples, the mean elevation, Airy height and gap (with
the runs' lowest and highest), then one table per band with every part of the gap that is not zero
in every run:
  stock      m of the band's mean gap the part made (since the sample's birth or the run's start),
             with the runs' lowest and highest;
  ocean      of it, what was made while the sample was oceanic crust;
  continent  the rest, made on the continent;
  gap/Myr    the part's rate over the 50 Myr before the snapshot, m/Myr per column of the band;
  surface/Myr and Airy/Myr  the same for the elevation and for the Airy height (gap = surface -
             Airy).
"""
import argparse
import re
import sys

STOCK = re.compile(
    r"^gap_stock t=(-?\d+) band=(\S+) n=(\S+) z=(\S+) iso=(\S+) gap=(\S+) gap_p50=(\S+) "
    r"ocean=(\S+) parts=(\S+)(?: ocean_parts=(\S+))?"
)
RATE = re.compile(r"^gap_rate t=(-?\d+) band=(\S+) m_per_myr=(\S+)(?: lift_m_per_myr=(\S+))?")
HEAD = re.compile(r"^gap_budget z0_m=(\S+) restarts=(\S+) worst_residual_m=(\S+) parts=(\S+)")


def floats(text):
    return [float(x) for x in text.split("/")] if text else None


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
                key = (int(m.group(1)), m.group(2))
                vals = [float(x) for x in m.group(3, 4, 5, 6, 7, 8)]
                stock[key] = (vals, floats(m.group(9)), floats(m.group(10)))
                continue
            m = RATE.match(line)
            if m:
                rate[(int(m.group(1)), m.group(2))] = (floats(m.group(3)), floats(m.group(4)))
    return names, head, stock, rate


def mean(xs):
    return sum(xs) / len(xs) if xs else float("nan")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--t", type=int, action="append")
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
        print(f"\n## t = {t} Myr")
        print("| band | runs | share of continental samples | elevation, m | Airy height, m "
              "| gap, m (median) | of it made on the ocean floor, m |")
        print("|---|---|---|---|---|---|---|")
        for b in bands:
            have = [r for r in runs if (t, b) in r[3]]
            v = [[r[3][(t, b)][0][k] for r in have] for k in range(6)]
            rng = lambda xs: f"{mean(xs):.0f} [{min(xs):.0f}, {max(xs):.0f}]"
            print(f"| {b} | {len(have)} | {mean(v[0]):.3f} [{min(v[0]):.3f}, {max(v[0]):.3f}] "
                  f"| {rng(v[1])} | {rng(v[2])} | {rng(v[3])} ({mean(v[4]):.0f}) | {mean(v[5]):.0f} |")
        for b in bands:
            have = [r for r in runs if (t, b) in r[3] and (t, b) in r[4]]
            if not have:
                continue
            print(f"\n### {b}, t = {t} Myr ({len(have)} runs)")
            print("| part | stock, m [lowest, highest] | ocean | continent | gap/Myr | surface/Myr "
                  "| Airy/Myr |")
            print("|---|---|---|---|---|---|---|")
            tot = [0.0] * 6
            for i, name in enumerate(names):
                st = [r[3][(t, b)][1][i] for r in have]
                oc = [r[3][(t, b)][2][i] for r in have if r[3][(t, b)][2]]
                gr = [r[4][(t, b)][0][i] for r in have]
                lf = [r[4][(t, b)][1][i] for r in have if r[4][(t, b)][1]]
                if all(abs(x) < 0.5 for x in st + oc) and all(abs(x) < 0.005 for x in gr + lf):
                    continue
                row = [mean(st), mean(oc), mean(st) - mean(oc), mean(gr), mean(lf),
                       mean(lf) - mean(gr)]
                tot = [x + (y if y == y else 0.0) for x, y in zip(tot, row)]
                print(f"| {name} | {row[0]:.0f} [{min(st):.0f}, {max(st):.0f}] | {row[1]:.0f} "
                      f"| {row[2]:.0f} | {row[3]:.2f} | {row[4]:.2f} | {row[5]:.2f} |")
            print(f"| sum | {tot[0]:.0f} | {tot[1]:.0f} | {tot[2]:.0f} | {tot[3]:.2f} "
                  f"| {tot[4]:.2f} | {tot[5]:.2f} |")


if __name__ == "__main__":
    main()
