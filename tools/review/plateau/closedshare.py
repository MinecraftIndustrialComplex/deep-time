# SPDX-License-Identifier: LGPL-3.0-or-later
"""The plateau lane: the share of the land whose water ends in a closed basin over a run (VM, cached runs).

Usage: python3 -I closedshare.py <arm folder> [<arm folder> ...]

The summary's `closed_km2_myr` (`surface.closed_basins` on: land cells draining to a lake that passes nothing, km2 x Myr
over the run, `surface/closed.rs`) against the land's km2 x Myr: the final fields' L1 grid area (`surface/elevation_m`'s
cells x the cell's area from `globals.json`'s `surface/cell_m` when present, else 4.8 km) times the mean land share of
the dynamics record's steps (`rec.json`, land over visible samples) times the run's length (the steps' span). Earth:
"Endorheic basins occupy 20% of the Earth's land surface" (Garcia-Castellanos et al. 2003, p. 8-1).
"""
import json
import statistics
import sys
from pathlib import Path


def run(r):
    kv = {}
    for line in open(r / "summary.txt", errors="replace"):
        if "surface_final=" in line:
            kv = dict(t.split("=", 1) for t in line.split() if "=" in t)
            break
    if "closed_km2_myr" not in kv:
        return None
    rec = json.load(open(r / "rec.json"))
    steps = rec.get("steps", [])
    land = statistics.mean(s["land"] / max(s["visible"], 1) for s in steps)
    span = (steps[-1]["t"] - steps[0]["t"]) + float(rec.get("dt_myr", 2.0))
    idx = json.load(open(r / "fields" / "index.json"))
    e = idx["surface/elevation_m"] if "surface/elevation_m" in idx else idx["elevation"]
    g = json.load(open(r / "fields" / "globals.json"))
    g = g.get("globals", g)
    cell_km = float(g.get("surface/cell_m", 4800.0)) / 1000.0
    area = e["w"] * e["h"] * cell_km * cell_km
    return float(kv["closed_km2_myr"]) / (land * area * span), land, span, area


def main():
    for arm in sys.argv[1:]:
        rows = []
        for r in sorted(Path(arm).iterdir()):
            if r.is_dir() and (r / "summary.txt").exists():
                x = run(r)
                if x:
                    rows.append((r.name, x))
        print(f"== closed share {arm} ({len(rows)} runs)")
        for name, (s, land, span, area) in rows:
            print(f"  {name:24s} closed share of the land {s:.3f} (land {land:.3f}, {span:.0f} Myr, {area:.3e} km2)")
        if rows:
            v = [x[0] for _, x in rows]
            print(f"  median {statistics.median(v):.3f} (min {min(v):.3f}, max {max(v):.3f})")


if __name__ == "__main__":
    main()
