#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The collisions' crust budget at the end of each run of an arm: cbfinal.py <run folder> [keys]

Reads the last `cb_globals` line of each run's summary.txt (DEEPTIME_CRUST_BUDGET=1; cumulative
km³ over the run) and prints one line per run and the medians of `keys` (default: what crossed the
collision fronts, what stayed, the wedge's part, the no-room losses and the taper's toe), each also
as a share of what crossed."""
import statistics
import sys
from pathlib import Path

KEYS = ["crossed", "kept", "kept_wedge", "noroom_sliver", "noroom_capped", "noroom_clipped", "taper_toe"]


def last_globals(summary):
    line = None
    for x in summary.read_text().splitlines():
        if x.startswith("cb_globals "):
            line = x
    if line is None:
        return None
    out = {}
    for tok in line.split()[1:]:
        k, _, v = tok.partition("=")
        try:
            out[k] = float(v)
        except ValueError:
            out[k] = v
    return out


def main():
    folder = Path(sys.argv[1])
    keys = sys.argv[2].split(",") if len(sys.argv) > 2 else KEYS
    rows = {}
    for run in sorted(p for p in folder.glob("*-*") if (p / "summary.txt").exists()):
        g = last_globals(run / "summary.txt")
        if g is not None:
            rows[run.name] = g
    print("run t " + " ".join(keys) + " | shares of crossed")
    for name, g in rows.items():
        c = g.get("crossed", 0.0) or float("nan")
        vals = " ".join(f"{g.get(k, float('nan')):.4g}" for k in keys)
        shares = " ".join(f"{g.get(k, float('nan')) / c:.4f}" for k in keys)
        print(f"{name} {g.get('t')} {vals} | {shares}")
    if rows:
        med = " ".join(f"{statistics.median(g.get(k, float('nan')) for g in rows.values()):.4g}" for k in keys)
        shares = " ".join(
            f"{statistics.median(g.get(k, float('nan')) / (g.get('crossed') or float('nan')) for g in rows.values()):.4f}"
            for k in keys)
        print(f"median {len(rows)} runs {med} | {shares}")


if __name__ == "__main__":
    main()
