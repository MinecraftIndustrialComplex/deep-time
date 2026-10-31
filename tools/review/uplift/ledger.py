#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The land ledger (`plates/gap_budget.rs`, `DEEPTIME_GAP_BUDGET=1`): the land's height above the
sea and the columns' thickness changed by each part, km^3 a year (Earth-normalized).

  python3 -I tools/review/uplift/ledger.py [--from T] [--all] <summary.txt> ...

Reads each run's `land_ledger_head` and `land_ledger` lines (sums since the start, km^3, every
50 Myr) and prints per part, over the whole run or with --from T (Myr before the end, e.g. -500)
over the window from then to the end:

  lift     change of H = max(z - sea, 0) on visible samples (rock or ground raised above the sea)
  up       the rises alone
  lift_c   change of H on continental crust
  t_land   change of the columns' thickness where they stood above the sea before the change
  t_shelf  ... on visible continental crust at or under the sea
  t_ocean  ... on visible ocean floor at or under the sea
  t_hidden ... on hidden samples

and the land's volume above the sea at the window's ends, the sea's own term and the closure.
Parts with every number under 0.005 km^3 a year are left out unless --all. The last block is the
mean over the runs.
"""
import re
import sys

NUM = r"(-?[0-9.]+(?:e[+-]?[0-9]+)?|NaN|nan|inf|-inf)"
ROWS = ("lift", "up", "lift_c", "t_land", "t_shelf", "t_ocean", "t_hidden")


def read(path):
    head, snaps = None, []
    with open(path) as f:
        for line in f:
            if line.startswith("land_ledger_head"):
                head = {k: v for k, v in re.findall(r"(\w+)=(\S+)", line)}
            elif line.startswith("land_ledger "):
                d = {}
                for k, v in re.findall(r"(\w+)=(\S+)", line):
                    if k in ROWS:
                        d[k] = [float(x) for x in v.split("/")]
                    else:
                        d[k] = float(v)
                snaps.append(d)
    if head is None or not snaps:
        raise SystemExit(f"{path}: no land_ledger lines (DEEPTIME_GAP_BUDGET=1?)")
    return head, snaps


def window(head, snaps, since):
    end = snaps[-1]
    if since is None:
        start = {k: ([0.0] * len(v) if isinstance(v, list) else 0.0) for k, v in end.items()}
        start["v_km3"] = float(head["v_start_km3"])
        # The first snapshot is 50 Myr after the start (`GAP_SNAPSHOT_MYR`; none at the start).
        start["t"] = snaps[0]["t"] - 50.0
    else:
        t0 = end["t"] + since
        start = min(snaps, key=lambda s: abs(s["t"] - t0))
    yrs = (end["t"] - start["t"]) * 1.0e6
    rates = {r: [(a - b) / yrs for a, b in zip(end[r], start[r])] for r in ROWS}
    other = {
        "years": yrs,
        "v0": start["v_km3"],
        "v1": end["v_km3"],
        "sea0": start.get("sea_m", 0.0),
        "sea1": end["sea_m"],
        "sea_shift": (end["sea_shift_km3"] - start.get("sea_shift_km3", 0.0)) / yrs,
        "dv": (end["v_km3"] - start["v_km3"]) / yrs,
    }
    return rates, other


def show(label, names, rates, other, keep):
    print(f"\n== {label}: {other['years'] / 1e6:.0f} Myr, land above the sea {other['v0']:.3e} to "
          f"{other['v1']:.3e} km^3 (sea {other['sea0']:.0f} to {other['sea1']:.0f} m)")
    print(f"   change {other['dv']:+.3f} km^3/yr; the sea's own term {other['sea_shift']:+.3f}; "
          f"parts {sum(rates['lift']):+.3f}; closure {other['dv'] - other['sea_shift'] - sum(rates['lift']):+.4f}")
    print(f"   {'part':18s}" + "".join(f"{r:>10s}" for r in ROWS))
    for p, name in enumerate(names):
        row = [rates[r][p] for r in ROWS]
        if p in keep:
            print(f"   {name:18s}" + "".join(f"{x:10.3f}" for x in row))
    tot = [sum(rates[r]) for r in ROWS]
    print(f"   {'all':18s}" + "".join(f"{x:10.3f}" for x in tot))
    e, rb = names.index("erosion"), names.index("rebound")
    if rates["lift"][e] != 0:
        print(f"   rebound / erosion's lowering of H: {-rates['lift'][rb] / rates['lift'][e]:.3f}; "
              f"erosion's thinning of land columns {rates['t_land'][e]:+.3f}, "
              f"of land and shelf {rates['t_land'][e] + rates['t_shelf'][e]:+.3f}")


def main(argv):
    since, show_all, paths = None, False, []
    it = iter(argv)
    for a in it:
        if a == "--from":
            since = float(next(it))
        elif a == "--all":
            show_all = True
        else:
            paths.append(a)
    if not paths:
        raise SystemExit(__doc__)
    runs = []
    for p in paths:
        head, snaps = read(p)
        names = head["parts"].split(",")
        rates, other = window(head, snaps, since)
        runs.append((p, names, rates, other))
    names = runs[0][1]
    keep = set()
    for _, _, rates, _ in runs:
        for p in range(len(names)):
            if show_all or any(abs(rates[r][p]) >= 0.005 for r in ROWS):
                keep.add(p)
    for p, nm, rates, other in runs:
        show(p, nm, rates, other, keep)
    if len(runs) > 1:
        n = len(runs)
        mean = {r: [sum(x[2][r][p] for x in runs) / n for p in range(len(names))] for r in ROWS}
        mo = {k: sum(x[3][k] for x in runs) / n for k in runs[0][3]}
        show(f"mean of {n}", names, mean, mo, keep)


if __name__ == "__main__":
    main(sys.argv[1:])
