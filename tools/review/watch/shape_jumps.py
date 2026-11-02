#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""What the resamplings change against what the steps between change (measurement only, cached pool runs):
shape_jumps.py <arm folder>[@N] ...

The per-step watch (dynamics::Shape, 2026-11-01): each step of the record carries the continental crust's shape at the
step's end (`shape`) and, on a step that resamples, just before the resampling (`shape_pre`): the visible continental
samples and the hyper-extended among them (area), and on the lattice (each point shows its nearest visible sample, a
gap shows ocean floor) the margin's edges raw and smoothed, the continental pieces and the enclosed ocean floor. A resampling should move none of them on average: it remaps the crust, it is no process. Per
quantity and run this splits the run's change into the resamplings' part (shape less shape_pre, summed) and the steps'
part (the rest), and gives the mean change per resampling with its t (mean over its standard error across the run's
resamplings): a |t| of 3 or more, or a resamplings' part of the same size as the whole change, is a numerical effect
to trace (as PLT-664's walk was: edges added at every resampling and never taken back). Medians over runs, per arm.
Runs made before the watch have no shape and are skipped."""
import json
import math
import statistics
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "freeboard"))
import thinning as th  # noqa: E402

KEYS = ["continental", "hyper", "edges", "smooth_edges", "pieces", "enclosed"]


def run_stats(run):
    steps = json.loads((run / "rec.json").read_text()).get("steps") or []
    steps = [s for s in steps if s.get("shape")]
    if len(steps) < 3:
        return None
    out = {}
    for k in KEYS:
        jumps, between = [], 0.0
        for a, b in zip(steps, steps[1:]):
            if b.get("shape_pre"):
                between += b["shape_pre"][k] - a["shape"][k]
                jumps.append(b["shape"][k] - b["shape_pre"][k])
            else:
                between += b["shape"][k] - a["shape"][k]
        total = steps[-1]["shape"][k] - steps[0]["shape"][k]
        n = len(jumps)
        mean = sum(jumps) / n if n else float("nan")
        sd = statistics.stdev(jumps) if n > 1 else float("nan")
        out[k] = {
            "start": steps[0]["shape"][k],
            "end": steps[-1]["shape"][k],
            "at_resamplings": sum(jumps),
            "between": between,
            "total": total,
            "per_resampling": mean,
            "t": mean / (sd / math.sqrt(n)) if n > 1 and sd > 0 else float("nan"),
            "resamplings": n,
        }
    return out


def main(args):
    for arg in args:
        runs = th.runs_of(arg)
        stats = [s for s in (run_stats(r) for r in runs) if s]
        print(f"== {arg} ({len(stats)} of {len(runs)} runs with the watch), medians over runs")
        if not stats:
            continue
        print(f"  {'quantity':<13} {'start':>9} {'end':>9} {'at resamplings':>15} {'between':>10} "
              f"{'per resampling':>15} {'t':>7} {'runs |t|>=3':>11}")
        for k in KEYS:
            col = lambda f: statistics.median(s[k][f] for s in stats if math.isfinite(s[k][f]))
            big = sum(1 for s in stats if math.isfinite(s[k]["t"]) and abs(s[k]["t"]) >= 3)
            print(f"  {k:<13} {col('start'):>9.0f} {col('end'):>9.0f} {col('at_resamplings'):>15.0f} "
                  f"{col('between'):>10.0f} {col('per_resampling'):>15.2f} {col('t'):>7.2f} {big:>8}/{len(stats)}")


if __name__ == "__main__":
    main(sys.argv[1:])
