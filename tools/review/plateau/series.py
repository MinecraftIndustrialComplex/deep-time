# SPDX-License-Identifier: LGPL-3.0-or-later
"""The plateau lane: the high ground through a run, and the run's extremes (VM, cached runs, no simulation).

Usage: python3 -I series.py <arm folder> [<arm folder> ...]

From every crust-budget map (`maps/t<Myr>.bin`, every 50 Myr, `plates/crust_budget.rs` MAP_FIELDS; z over the step's sea
from the dynamics record, as `tools/review/orogen/hiseries.py`): the share of the land above 2, 3 and 4 km, and the
highest and deepest visible sample (the plates' elevation). Printed per arm: for each snapshot time the median over the
runs of the three shares; per run the mean over the snapshots after the first 100 Myr and the largest; then the medians.
Per run also the summary's `guard_worst_m` (the farthest any L1 cell stood from the sea after a step, `surface/guard.rs`)
and `max_elevation_m`, and the final surface's highest and deepest cell (`fields/surface/elevation_m` against
`climate/sea_level_m`), so the guard's worst can be told a peak or a trench at the run's end.
"""
import json
import statistics
import sys
from pathlib import Path

import numpy as np

F = 11
HI = (2000.0, 3000.0, 4000.0)


def sea_of(rec):
    return {round(s["t"]): float(s.get("sea_m") or 0.0) for s in rec.get("steps", [])}


def near(seas, t):
    k = min(seas, key=lambda x: abs(x - t))
    return seas[k]


def summary_kv(run):
    out = {}
    for line in open(run / "summary.txt", errors="replace"):
        if "surface_final=" in line:
            for t in line.split():
                if "=" in t:
                    k, v = t.split("=", 1)
                    out[k] = v
            break
    return out


def final_extremes(run):
    f = run / "fields"
    try:
        idx = json.load(open(f / "index.json"))
        g = json.load(open(f / "globals.json"))
    except OSError:
        return float("nan"), float("nan")
    g = g.get("globals", g)
    sea = float(g.get("climate/sea_level_m", 0.0))
    name = "surface/elevation_m" if "surface/elevation_m" in idx else "elevation"
    e = idx[name]
    z = np.fromfile(f / e["file"], "<f4").astype(np.float64) - sea
    return float(z.max()), float(-z.min())


def run_series(run):
    rec = json.load(open(run / "rec.json"))
    seas = sea_of(rec)
    out = []
    for p in sorted((run / "maps").glob("t*.bin")):
        t = float(p.stem[1:])
        a = np.fromfile(p, "<f4").reshape(-1, F).astype(np.float64)
        z = a[:, 2] - near(seas, t)
        land = z > 0
        n = max(int(land.sum()), 1)
        out.append((t, [float((z > h).sum() / n) for h in HI], float(z.max()), float(-z.min())))
    return out


def med(v):
    v = [x for x in v if x == x]
    return statistics.median(v) if v else float("nan")


def main():
    for arm in sys.argv[1:]:
        rows = []
        for r in sorted(Path(arm).iterdir()):
            if r.is_dir() and (r / "maps").is_dir() and (r / "rec.json").exists():
                s = run_series(r)
                if s:
                    rows.append((r, s))
        print(f"== {arm} ({len(rows)} runs)")
        times = sorted({x[0] for _, s in rows for x in s})
        print("  t (Myr): median share of the land above 2 / 3 / 4 km over the runs")
        for t in times:
            per =[[x[1][k] for _, s in rows for x in s if x[0] == t] for k in range(3)]
            print(f"    {t:7.0f}: {med(per[0]):.4f} / {med(per[1]):.5f} / {med(per[2]):.5f}")
        agg = {k: [] for k in ("m2", "x2", "m3", "x3", "m4", "x4", "worst", "maxel", "fpeak", "fdeep", "speak", "sdeep")}
        for r, s in rows:
            late = [x for x in s if x[0] >= times[0] + 100.0] or s
            for k, h in enumerate(("2", "3", "4")):
                agg["m" + h].append(statistics.mean(x[1][k] for x in late))
                agg["x" + h].append(max(x[1][k] for x in s))
            kv = summary_kv(r)
            agg["worst"].append(float(kv.get("guard_worst_m", "nan")))
            agg["maxel"].append(float(kv.get("max_elevation_m", "nan")))
            fp, fd = final_extremes(r)
            agg["fpeak"].append(fp)
            agg["fdeep"].append(fd)
            agg["speak"].append(max(x[2] for x in s))
            agg["sdeep"].append(max(x[3] for x in s))
            print(f"  {r.name:24s} above 2 km mean {agg['m2'][-1]:.4f} max {agg['x2'][-1]:.4f}; 3 km mean "
                  f"{agg['m3'][-1]:.5f} max {agg['x3'][-1]:.4f}; 4 km mean {agg['m4'][-1]:.5f} max {agg['x4'][-1]:.4f}; "
                  f"guard worst {agg['worst'][-1]:.0f}, max elevation {agg['maxel'][-1]:.0f}, final surface peak "
                  f"{fp:.0f} deep {fd:.0f}; maps' highest sample {agg['speak'][-1]:.0f} deepest {agg['sdeep'][-1]:.0f}")
        for k, label in (("m2", "above 2 km, mean over time"), ("x2", "above 2 km, largest"),
                         ("m3", "above 3 km, mean over time"), ("x3", "above 3 km, largest"),
                         ("m4", "above 4 km, mean over time"), ("x4", "above 4 km, largest"),
                         ("worst", "guard worst, m"), ("maxel", "max elevation at the end, m"),
                         ("fpeak", "final surface peak over sea, m"), ("fdeep", "final surface deepest under sea, m"),
                         ("speak", "maps' highest sample, m"), ("sdeep", "maps' deepest sample, m")):
            v = agg[k]
            print(f"  median {label:34s} {med(v):.5f}  (min {min(v):.5f}, max {max(v):.5f})")


if __name__ == "__main__":
    main()
