#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""oldfloor_welds.py <argkey folder> [--seeds=1,2,3,4]: how much ocean floor the welds take into other plates.

The coordinator's diagnosis for the old floor (2026-10-27, plates.md Step 18). A weld (PLT-246) makes two plates in a
stalled collision one plate; the floor of the plate that goes (fate "sutured" in the dynamics record's plates) stays,
now inside the other plate, and no boundary is left between it and the other plate's continents. For every sutured
plate, its row on the record's last step before the weld (rec.json steps[].plates: id, visible, continental, ...) gives
its share of the planet and its oceanic share. Prints, per run and pooled: the welds, the welded plates' share of the
planet and oceanic share (medians, 90th percentiles), and the ocean welded, the sum of the welded plates' oceanic
area in planets (visible samples over the record's sample count). Standard library only.
"""
import json
import statistics
import sys
from pathlib import Path


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    seeds = None
    for a in sys.argv[1:]:
        if a.startswith("--seeds="):
            seeds = {int(s) for s in a.split("=", 1)[1].split(",")}
    pooled, per_run = [], {}
    for arm in args:
        for run in sorted(p for p in Path(arm).iterdir() if (p / "rec.json").is_file()):
            if seeds is not None and int(run.name.rsplit("-", 1)[1]) not in seeds:
                continue
            r = json.loads((run / "rec.json").read_text())
            n, steps = r["samples"], r["steps"]
            welded = []
            for p in r["plates"]:
                if p.get("fate") != "sutured":
                    continue
                before = [s for s in steps if s["t"] < p["died_myr"]]
                row = next((q for q in before[-1]["plates"] if int(q[0]) == p["id"]), None) if before else None
                if row is None:
                    continue
                vis, cont = row[1], row[2]
                welded.append((vis / n, 1.0 - cont / max(vis, 1.0)))
            per_run[run.name] = (len(welded), sum(s * o for s, o in welded))
            pooled += welded
    if not pooled:
        print("no welds found")
        return

    def p90(xs):
        xs = sorted(xs)
        return xs[min(len(xs) - 1, int(0.9 * len(xs)))]

    shares, oceans = [s for s, _ in pooled], [o for _, o in pooled]
    ocean = [v[1] for v in per_run.values()]
    print(json.dumps({
        "runs": len(per_run),
        "welds": len(pooled),
        "welds_per_run_median": statistics.median(v[0] for v in per_run.values()),
        "welded_share_p50": round(statistics.median(shares), 4),
        "welded_share_p90": round(p90(shares), 4),
        "welded_oceanic_share_p50": round(statistics.median(oceans), 3),
        "ocean_welded_planets_per_run_median": round(statistics.median(ocean), 3),
        "ocean_welded_planets_per_run_min_max": [round(min(ocean), 3), round(max(ocean), 3)],
        "welds_over_0.1_of_planet": sum(1 for s in shares if s >= 0.1),
    }, indent=1))


main()
