#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The gap budget's rates (`plates/gap_budget.rs`, `DEEPTIME_GAP_BUDGET=1`) over all the visible
continental crust: per part, the change of the gap (elevation less the column's Airy height) and of
the elevation, m/Myr per column, each band's rate weighed by its share of the columns.

  python3 -I tools/review/uplift/gapsplit.py [--from T] <summary.txt> ...

Reads each run's `gap_stock` (the band's share, n) and `gap_rate` lines (every 50 Myr, the rates
of the window that ends there) and prints the mean over the windows (from T Myr before the end
with --from), per run and over the runs. A part that lowers the surface by less than it lowers
the Airy height (erosion of light rock with the surface's 2,640 kg/m3 rebound against the column's
mean 2,830) opens a gap that the settling (`settle.rs`) closes by lifting.
"""
import re
import sys

NUM = r"(-?[0-9.]+(?:e[+-]?[0-9]+)?|NaN|nan|inf|-inf)"


def read(path):
    names, rows = None, {}
    with open(path) as f:
        for line in f:
            if line.startswith("gap_budget "):
                m = re.search(r"parts=(\S+)", line)
                names = m.group(1).split(",")
            elif line.startswith("gap_stock "):
                t = float(re.search(r"t=(-?[0-9.]+)", line).group(1))
                band = re.search(r"band=(\S+)", line).group(1)
                n = float(re.search(r" n=" + NUM, line).group(1))
                rows.setdefault(t, {}).setdefault(band, {})["n"] = n
            elif line.startswith("gap_rate "):
                t = float(re.search(r"t=(-?[0-9.]+)", line).group(1))
                band = re.search(r"band=(\S+)", line).group(1)
                gap = [float(x) for x in re.search(r"m_per_myr=(\S+)", line).group(1).split("/")]
                lift = [float(x) for x in re.search(r"lift_m_per_myr=(\S+)", line).group(1).split("/")]
                d = rows.setdefault(t, {}).setdefault(band, {})
                d["gap"], d["lift"] = gap, lift
    return names, rows


def mean(names, rows, since):
    ts = sorted(rows)
    if since is not None:
        ts = [t for t in ts if t > ts[-1] + since]
    k = len(names)
    gap, lift = [0.0] * k, [0.0] * k
    for t in ts:
        for d in rows[t].values():
            if "gap" not in d:
                continue
            for p in range(k):
                gap[p] += d["n"] * d["gap"][p] / len(ts)
                lift[p] += d["n"] * d["lift"][p] / len(ts)
    return gap, lift


def show(label, names, gap, lift):
    print(f"\n== {label}: m/Myr per visible continental column (gap = elevation less Airy height)")
    print(f"   {'part':18s}{'gap':>9s}{'lift':>9s}{'Airy':>9s}")
    order = sorted(range(len(names)), key=lambda p: -abs(gap[p]) - abs(lift[p]))
    for p in order:
        if abs(gap[p]) < 0.05 and abs(lift[p]) < 0.05:
            continue
        print(f"   {names[p]:18s}{gap[p]:9.2f}{lift[p]:9.2f}{lift[p] - gap[p]:9.2f}")
    print(f"   {'all':18s}{sum(gap):9.2f}{sum(lift):9.2f}{sum(lift) - sum(gap):9.2f}")
    e, r = names.index("erosion"), names.index("rebound")
    if lift[e] != 0:
        print(f"   the surface's rebound over erosion's lowering {-lift[r] / lift[e]:.3f}; erosion and its "
              f"rebound open a gap of {gap[e] + gap[r]:.2f} m/Myr, {(gap[e] + gap[r]) / lift[e]:.3f} of the "
              f"lowering; the settling closes {gap[names.index('settle')]:.2f}")


def main(argv):
    since, paths = None, []
    it = iter(argv)
    for a in it:
        if a == "--from":
            since = float(next(it))
        else:
            paths.append(a)
    if not paths:
        raise SystemExit(__doc__)
    got = []
    for p in paths:
        names, rows = read(p)
        g, l = mean(names, rows, since)
        show(p, names, g, l)
        got.append((g, l))
    if len(got) > 1:
        k = len(names)
        g = [sum(x[0][p] for x in got) / len(got) for p in range(k)]
        l = [sum(x[1][p] for x in got) / len(got) for p in range(k)]
        show(f"mean of {len(got)}", names, g, l)


if __name__ == "__main__":
    main(sys.argv[1:])
