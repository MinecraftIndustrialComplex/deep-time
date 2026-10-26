#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The continental crust's growth by steps since the last resampling: m5resample.py <folder>[@N] [<folder>[@N] ...]

For the M5 stage (a)+(b) arm: if the continents grow because a resampling blends their own crust into the margins
turned continental (the blend of `ridges.rs` takes a continental sample's pools from its continental corners only, so
a converted margin is blended with the core), the next steps' taper finds a new boundary and turns the next ring: the
growth comes in the first steps after each resampling. Without that, it is spread over the steps. From each run's
record (rec.json: `visible` jumps back to the lattice's full count at a resampling, `continental` per step), the mean
change of the continental sample count a step, as a share of the planet per 100 steps, by steps since the resampling
(0 is the resampling's own step), and the share of the run's net growth in the first 3 steps after one. Means over
runs. Needs nothing beyond the standard library."""
import json
import statistics
import sys
from pathlib import Path

BINS = [0, 1, 2, 3, 5, 10, 20, 10**9]


def one(run):
    steps = json.loads((run / "rec.json").read_text())["steps"]
    full = max(s["visible"] for s in steps)
    since, last = [], None
    for i, s in enumerate(steps):
        if s["visible"] == full and (i == 0 or steps[i - 1]["visible"] < full):
            last = i
        since.append(i - last if last is not None else 10**9)
    d = {b: [] for b in BINS[:-1]}
    early, total = 0.0, 0.0
    for i in range(1, len(steps)):
        dc = (steps[i]["continental"] - steps[i - 1]["continental"]) / steps[i]["visible"]
        total += dc
        if since[i] <= 3:
            early += dc
        for lo, hi in zip(BINS, BINS[1:]):
            if lo <= since[i] < hi:
                d[lo].append(dc)
    out = {f"{lo}-{hi - 1 if hi < 10**9 else 'end'}": (statistics.mean(v) * 100 if v else float("nan"), len(v))
           for (lo, v), hi in zip(d.items(), BINS[1:])}
    return out, early, total, sum(1 for x in since if x == 0)


def main():
    for arg in sys.argv[1:]:
        folder, _, top = arg.partition("@")
        rows = []
        for run in sorted(q for q in Path(folder).iterdir() if (q / "rec.json").exists()):
            if top and int(run.name.rsplit("-", 1)[1]) > int(top):
                continue
            rows.append(one(run))
        if not rows:
            print(f"{folder}: no runs")
            continue
        print(f"{folder}: {len(rows)} runs, resamplings a run {statistics.mean(r[3] for r in rows):.1f}")
        for k in rows[0][0]:
            vals = [r[0][k][0] for r in rows if r[0][k][0] == r[0][k][0]]
            n = statistics.mean(r[0][k][1] for r in rows)
            if not vals:
                continue
            print(f"  steps since {k:>7}: continental change {statistics.mean(vals):+.4f} of the planet per 100 steps"
                  f" ({n:.0f} steps a run)")
        print(f"  net growth over the run {statistics.mean(r[2] for r in rows):+.4f}, of it in the first 3 steps after a"
              f" resampling {statistics.mean(r[1] for r in rows):+.4f}")


if __name__ == "__main__":
    main()
