# SPDX-License-Identifier: LGPL-3.0-or-later
"""The orogen lane: the land's high ground through a run, from every crust-budget map (VM, cached runs).

Usage: python3 -I hiseries.py <arm folder> [<arm folder> ...]

A run's end is one frame; whether a large collision is active then decides its land above 2 km. Every
50 Myr snapshot (`maps/t<Myr>.bin`, after the first 100 Myr) gives the share of the land (z over the
step's sea, the dynamics record's) above 2 km and above 4 km, and the share of the land above 2 km
that is a collision orogeny younger than 30 Myr. Per run: the mean over the snapshots and the
largest; per arm: the medians of those.
"""
import json
import statistics
import sys
from pathlib import Path

import numpy as np

F = 11


def sea_of(rec):
    return {round(s["t"]): float(s.get("sea_m") or 0.0) for s in rec.get("steps", [])}


def near(seas, t):
    k = min(seas, key=lambda x: abs(x - t))
    return seas[k]


def run_series(run):
    rec = json.load(open(run / "rec.json"))
    seas = sea_of(rec)
    out = []
    for p in sorted((run / "maps").glob("t*.bin")):
        t = float(p.stem[1:])
        if t < -900.0:
            continue
        a = np.fromfile(p, "<f4").reshape(-1, F).astype(np.float64)
        z = a[:, 2] - near(seas, t)
        land = z > 0
        n = max(int(land.sum()), 1)
        hi2 = z > 2000
        hi4 = z > 4000
        young = (a[:, 5] == 2) & (a[:, 6] < 30.0)
        out.append((t, hi2.sum() / n, hi4.sum() / n, (hi2 & young).sum() / max(int(hi2.sum()), 1)))
    return out


def main():
    for arm in sys.argv[1:]:
        rows = []
        for r in sorted(Path(arm).iterdir()):
            if r.is_dir() and (r / "maps").is_dir() and (r / "rec.json").exists():
                s = run_series(r)
                if s:
                    rows.append((r.name, s))
        print(f"== {arm} ({len(rows)} runs)")
        mean2, max2, mean4, max4, young = [], [], [], [], []
        for name, s in rows:
            h2 = [x[1] for x in s]
            h4 = [x[2] for x in s]
            yg = [x[3] for x in s]
            mean2.append(statistics.mean(h2))
            max2.append(max(h2))
            mean4.append(statistics.mean(h4))
            max4.append(max(h4))
            young.append(statistics.mean(yg))
            print(f"  {name:24s} above 2 km mean {mean2[-1]:.4f} max {max2[-1]:.4f} end {h2[-1]:.4f}; above 4 km mean "
                  f"{mean4[-1]:.5f} max {max4[-1]:.4f}; young collision share of it {young[-1]:.2f} ({len(s)} maps)")
        for label, v in (("above 2 km, mean over time", mean2), ("above 2 km, largest", max2),
                         ("above 4 km, mean over time", mean4), ("above 4 km, largest", max4),
                         ("young collision share", young)):
            print(f"  median {label:28s} {statistics.median(v):.4f}  (min {min(v):.4f}, max {max(v):.4f})")


if __name__ == "__main__":
    main()
