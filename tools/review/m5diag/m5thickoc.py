#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""What the thick oceanic-labelled crust is: m5thickoc.py <arm folder>[@N] [map name, default t0000.bin]

The drowning diagnosis (m5sea.py) finds oceanic-labelled crust (kind 0) 25 km and more thick on 0.05 (base14) to 0.10
(m5-unfreeze) of the ocean floor, standing near the datum, so it holds sea water up like continental crust would.
From each run's crust-budget map (maps/t*.bin, `DEEPTIME_CRUST_BUDGET_MAPS`; plates/crust_budget.rs MAP_FIELDS:
latitude, longitude, elevation m, thickness m, kind, orogeny, its age, island-arc and continental-arc magma m, the
plate group, the flags), the oceanic samples by thickness band: their share, the mean elevation, the island-arc and
continental-arc magma's share of the thickness, the share with an orogeny tag, the flags' shares (arc, hotspot,
divergent, subduction zone, forearc) and the distance to the nearest continental sample p50 (km). Medians over runs.
Needs numpy and scipy."""
import statistics
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

FIELDS = 11
BANDS = [(0, 8), (8, 10), (10, 15), (15, 25), (25, 1e9)]
FLAGS = {"arc": 4, "hot": 8, "div": 16, "sz": 64, "fore": 128}
R_KM = 6371.0


def xyz(lat, lon):
    la, lo = np.radians(lat), np.radians(lon)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], 1)


def one(path):
    a = np.fromfile(path, "<f4").reshape(-1, FIELDS).astype(np.float64)
    p = xyz(a[:, 0], a[:, 1])
    T = a[:, 3] / 1000.0
    cont = a[:, 4] > 0.5
    flags = np.rint(a[:, 10]).astype(np.int64)
    d, _ = cKDTree(p[cont]).query(p[~cont])
    dist = np.zeros(len(a))
    dist[~cont] = 2 * R_KM * np.arcsin(np.minimum(d / 2, 1.0))
    out = {}
    O = max(int((~cont).sum()), 1)
    for lo, hi in BANDS:
        m = ~cont & (T >= lo) & (T < hi)
        n = int(m.sum())
        if not n:
            continue
        o = {"share": n / O, "z": float(a[m, 2].mean()), "T": float(T[m].mean()),
             "arc_o": float((a[m, 7] / 1000.0 / T[m]).mean()), "arc_c": float((a[m, 8] / 1000.0 / T[m]).mean()),
             "oro": float((a[m, 5] > 0.5).mean()), "dist_p50": float(np.median(dist[m]))}
        for k, b in FLAGS.items():
            o[k] = float(((flags[m] & b) != 0).mean())
        out[f"{lo:g}-{hi:g}"] = o
    return out


def series(folder, top):
    """The thick oceanic crust over the run, from every map: its share of the oceanic crust at 15 km and more and at
    25 km and more, their mean height, and the volume beyond a normal 7 km floor (planet-mean m), medians over runs."""
    by_t = {}
    for run in sorted(p for p in Path(folder).iterdir() if (p / "maps").is_dir()):
        if top and int(run.name.rsplit("-", 1)[1]) > int(top):
            continue
        for m in (run / "maps").glob("t*.bin"):
            a = np.fromfile(m, "<f4").reshape(-1, FIELDS).astype(np.float64)
            T, oc = a[:, 3] / 1000.0, a[:, 4] <= 0.5
            O = max(int(oc.sum()), 1)
            row = {"oc": oc.mean()}
            for lo in (15, 25):
                k = oc & (T >= lo)
                row[f"s{lo}"] = k.sum() / O
                row[f"z{lo}"] = float(a[k, 2].mean()) if k.any() else float("nan")
            row["excess_m"] = float(np.where(oc, np.maximum(T - 7.0, 0.0), 0.0).mean() * 1000.0)
            by_t.setdefault(int(m.stem[1:]), []).append(row)
    print("t Myr, runs, oceanic share of the planet, share of the oceanic crust >= 15 km and its mean z, >= 25 km and its"
          " mean z, oceanic crust beyond 7 km as planet-mean m (medians)")
    for t in sorted(by_t):
        rs = by_t[t]
        m = {x: statistics.median(r[x] for r in rs) for x in rs[0]}
        print(f"  {t:6d} {len(rs)}  oc {m['oc']:.3f}  >=15 {m['s15']:.3f} z {m['z15']:6.0f}  >=25 {m['s25']:.3f}"
              f" z {m['z25']:6.0f}  excess {m['excess_m']:5.0f} m")


def main():
    folder, _, top = sys.argv[1].partition("@")
    name = sys.argv[2] if len(sys.argv) > 2 else "t0000.bin"
    if name == "series":
        series(folder, top)
        return
    rows = []
    for run in sorted(p for p in Path(folder).iterdir() if (p / "maps" / name).exists()):
        if top and int(run.name.rsplit("-", 1)[1]) > int(top):
            continue
        rows.append(one(run / "maps" / name))
    print(f"{len(rows)} runs, map {name}: oceanic samples by thickness band (medians over runs)")
    for lo, hi in BANDS:
        k = f"{lo:g}-{hi:g}"
        rs = [r[k] for r in rows if k in r]
        if not rs:
            continue
        m = {x: statistics.median(r[x] for r in rs) for x in rs[0]}
        print(f"  {k:>8s} km: share {m['share']:.3f}, z {m['z']:6.0f} m, T {m['T']:5.1f} km, island-arc magma"
              f" {m['arc_o']:.2f} and continental-arc {m['arc_c']:.2f} of T, orogeny-tagged {m['oro']:.2f},"
              f" flags arc {m['arc']:.2f} hot {m['hot']:.2f} div {m['div']:.2f} sz {m['sz']:.2f} fore {m['fore']:.2f},"
              f" to continental crust p50 {m['dist_p50']:.0f} km")


if __name__ == "__main__":
    main()
