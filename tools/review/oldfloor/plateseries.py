#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""plateseries.py <argkey folder> [--seeds=1,2,3,4] [--from=-900]: the plate system against the old floor, from rec.json.

Per run, from the dynamics record's steps after --from (the plate check's floor-age window starts 100 Myr in): the
share of the ocean older than 180 and 250 Myr (old180, old250 over ocean), the plates alive, the largest plate's
share of the visible samples, and the subduction front (the boundary mix's ocean-ocean plus ocean-continent km,
dynamics::mix). Steps are classed by the subduction front: none (under 2,000 km), thin (under 15,000 km) and working
(the rest; Bird 2003's subduction zones are 51,310 km); the old floor's mean over each class shows where the plate
check's floor-age means come from. A stalled step has an area-weighted mean plate speed under 10 mm/yr. Prints one line per run and the pooled shares; plain python, no numpy."""
import json
import statistics
import sys
from pathlib import Path


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    seeds, t_from = None, -900.0
    for a in sys.argv[1:]:
        if a.startswith("--seeds="):
            seeds = {int(s) for s in a.split("=", 1)[1].split(",")}
        if a.startswith("--from="):
            t_from = float(a.split("=", 1)[1])
    pooled = {"none": [0, 0.0, 0.0], "thin": [0, 0.0, 0.0], "working": [0, 0.0, 0.0]}
    for arm in args:
        for run in sorted(p for p in Path(arm).iterdir() if (p / "rec.json").is_file()):
            if seeds is not None and int(run.name.rsplit("-", 1)[1]) not in seeds:
                continue
            r = json.loads((run / "rec.json").read_text())
            steps = [s for s in r["steps"] if s["t"] >= t_from and s.get("ocean")]
            sub = [s["mix_km"][0] + s["mix_km"][1] for s in steps]
            med = statistics.median(sub) if sub else 0.0
            cls = {"none": [], "thin": [], "working": []}
            largest = []
            stalled = 0
            for s, f in zip(steps, sub):
                o180, o250 = s["old180"] / s["ocean"], s["old250"] / s["ocean"]
                c = "none" if f < 2000.0 else "thin" if f < 15000.0 else "working"
                cls[c].append((o180, o250))
                big = max((q[1] for q in s["plates"]), default=0.0)
                largest.append(big / s["visible"])
                vis = sum(q[1] for q in s["plates"])
                stalled += vis > 0 and sum(q[1] * q[3] for q in s["plates"]) / vis < 10.0
            all180 = statistics.mean(s["old180"] / s["ocean"] for s in steps)
            all250 = statistics.mean(s["old250"] / s["ocean"] for s in steps)
            parts = []
            for c, v in cls.items():
                pooled[c][0] += len(v)
                pooled[c][1] += sum(x for x, _ in v)
                pooled[c][2] += sum(y for _, y in v)
                m250 = statistics.mean(y for _, y in v) if v else float("nan")
                parts.append(f"{c} {len(v):3d} steps old250 {m250:.3f}")
            plates = [len(s["plates"]) for s in steps]
            print(f"{run.name:26s} old180 {all180:.3f} old250 {all250:.3f} | plates p50 {statistics.median(plates):4.1f} "
                  f"min {min(plates):2d} | largest p50 {statistics.median(largest):.2f} max {max(largest):.2f} | "
                  f"front p50 {med:7.0f} km | stalled {stalled:3d} | " + " | ".join(parts), flush=True)
    tot = sum(v[0] for v in pooled.values())
    for c, (n, s180, s250) in pooled.items():
        print(f"pooled {c:8s} steps {n:5d} ({n / tot:.3f}) old180 mean {s180 / max(n, 1):.3f} "
              f"old250 mean {s250 / max(n, 1):.3f} share of the old250 sum {s250 / max(sum(v[2] for v in pooled.values()), 1e-9):.3f}")


main()
