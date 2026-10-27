#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""oldfloor_read.py <oldfloor.py output, JSON lines> [--working=15000]: pooled tables of where the old floor sits.

Snapshots are split by the step's subduction front (rec's boundary mix, ocean-ocean plus ocean-continent): working
(at least --working km, and the map's flagged front at least half of it, so the fronts are on the map) and stalled
(under 2,000 km). For each class and each age limit (180, 250 Myr): the old floor's share of the ocean, and of the
old floor the shares in the margin zone, on giant plates, in closed basins, in a component with no active front, by
the distance through the ocean to the nearest active front, by the distance to its own plate's trench, and on plates
with no trench. Then the plate check's floor-age measures per run and the passive margins' ages."""
import json
import statistics
import sys

REACH = ["0-500", "500-1000", "1000-2000", "2000-4000", "4000+", "none"]


def main():
    path = [a for a in sys.argv[1:] if not a.startswith("--")][0]
    working = 15000.0
    for a in sys.argv[1:]:
        if a.startswith("--working="):
            working = float(a.split("=", 1)[1])
    runs = [json.loads(line) for line in open(path) if line.strip()]
    print(f"{len(runs)} runs")
    keys = ("ocean_age_p50_myr", "ocean_age_p90_mean_myr", "ocean_old180_pct", "ocean_old250_pct", "active_margin_mean")
    for k in keys:
        v = [r["checks"][k][0] for r in runs if k in r["checks"]]
        f = sum(r["checks"][k][1] == "FAIL" for r in runs if k in r["checks"])
        if v:
            print(f"check {k:24s} median {statistics.median(v):8.2f}  range {min(v):.2f} to {max(v):.2f}  fail {f}/{len(v)}")
    cls = {"working": [], "stalled": [], "between": []}
    for r in runs:
        for s in r["snaps"]:
            f = s["front_km_rec"]
            if f >= working and s["front_km_map"] >= 0.5 * f:
                cls["working"].append(s)
            elif f < 2000.0:
                cls["stalled"].append(s)
            else:
                cls["between"].append(s)
    for name, snaps in cls.items():
        if not snaps:
            continue
        ocean = sum(s["ocean"] for s in snaps)
        print(f"\n== {name}: {len(snaps)} snapshots, mean speed {statistics.mean(s['speed_mean_rec'] for s in snaps):.1f} "
              f"mm/yr, plates p50 {statistics.median(s['plates'] for s in snaps)}, ocean in the main component "
              f"{sum(s['main_share'] * s['ocean'] for s in snaps) / ocean:.3f}, margin zone {sum(s['ocean_margin'] for s in snaps) / ocean:.3f} of the ocean")
        oreach = [sum(s["ocean_reach"][i] for s in snaps) / ocean for i in range(6)]
        print("   the whole ocean by reach: " + ", ".join(f"{REACH[i]} {oreach[i]:.3f}" for i in range(6)))
        for lim in ("180", "250"):
            old = sum(s[f"old{lim}"] for s in snaps)
            if not old:
                continue
            sh = lambda k: sum(s[k] for s in snaps) / old  # noqa: E731
            print(f"   older than {lim} Myr: {old / ocean:.3f} of the ocean; of it margin {sh(f'old{lim}_margin'):.3f}, "
                  f"giant plates {sh(f'old{lim}_giant'):.3f}, closed basins {sh(f'old{lim}_closed_basin'):.3f} "
                  f"(no front in the basin {sh(f'old{lim}_closed_nofront'):.3f}), main ocean with no front "
                  f"{sh(f'old{lim}_main_nofront'):.3f}, on a plate with no trench {sh(f'old{lim}_notrench_plate'):.3f}")
            for k, label in ((f"old{lim}_reach", "reach (through the ocean)"),
                             (f"old{lim}_reach_margin", "reach, margin floor only"),
                             (f"old{lim}_own", "own plate's trench (straight)"),
                             (f"old{lim}_own_margin", "own trench, margin floor only")):
                tot = sum(sum(s[k]) for s in snaps) or 1
                b = [sum(s[k][i] for s in snaps) / tot for i in range(6)]
                print(f"      {label:32s} " + ", ".join(f"{REACH[i]} {b[i]:.3f}" for i in range(6)))
        ma = [s for s in snaps if s.get("margin_age_mean") is not None]
        if ma:
            edges = ["0-50", "50-100", "100-150", "150-180", "180-250", "250-350", "350-500", "500+", "-"]
            tot = sum(sum(s["margin_age"]) for s in ma)
            b = [sum(s["margin_age"][i] for s in ma) / tot for i in range(len(edges))]
            print(f"   passive margins: {sum(s['passive_coast'] for s in ma) / max(sum(s['coast'] for s in ma), 1):.3f} of the "
                  f"coast samples; age mean {statistics.mean(s['margin_age_mean'] for s in ma):.0f} Myr, p50 of snapshot "
                  f"medians {statistics.median(s['margin_age_p50'] for s in ma):.0f}, max {max(s['margin_age_max'] for s in ma):.0f}")
            print("      by age: " + ", ".join(f"{edges[i]} {b[i]:.3f}" for i in range(len(edges) - 1)))


main()
