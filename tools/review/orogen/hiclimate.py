# SPDX-License-Identifier: LGPL-3.0-or-later
"""The orogen lane: the climate class of the high ground at a run's end (VM, cached pool runs).

Usage: python3 -I hiclimate.py <arm folder> [<arm folder> ...]

From the dumped final fields (L1 grid): the surface elevation over the sea (`surface/elevation_m`,
the sea `climate/sea_level_m` in globals.json) and the Köppen class (`climate/koppen`, codes as
`climate/koppen.rs`). For the land above 2 km, and for its flat part (the 3 x 3 neighbourhood
spanning under 500 m, as `surface::erosion_bands`), the share that is arid (B), polar (E) and
the rest. Medians over the runs.
"""
import json
import statistics
import sys
from pathlib import Path

import numpy as np

CODES = ["Af", "Am", "Aw", "As", "BWh", "BWk", "BSh", "BSk", "Csa", "Csb", "Csc", "Cwa", "Cwb", "Cwc",
         "Cfa", "Cfb", "Cfc", "Dsa", "Dsb", "Dsc", "Dsd", "Dwa", "Dwb", "Dwc", "Dwd", "Dfa", "Dfb",
         "Dfc", "Dfd", "ET", "EF", "sea"]


def field(f, idx, name):
    e = idx[name]
    return np.fromfile(f / e["file"], "<f4").reshape(e["h"], e["w"]).astype(np.float64)


def run(r):
    f = r / "fields"
    idx = json.load(open(f / "index.json"))
    g = json.load(open(f / "globals.json"))
    g = g.get("globals", g)
    sea = float(g.get("climate/sea_level_m", 0.0))
    name = "surface/elevation_m" if "surface/elevation_m" in idx else "elevation"
    z = field(f, idx, name) - sea
    k = field(f, idx, "climate/koppen").round().astype(int)
    hi = z > 2000.0
    pad = np.pad(z, 1, mode="edge")
    win = np.stack([pad[1 + dy:pad.shape[0] - 1 + dy, 1 + dx:pad.shape[1] - 1 + dx]
                    for dy in (-1, 0, 1) for dx in (-1, 0, 1)])
    flat = hi & ((win.max(0) - win.min(0)) < 500.0)
    out = {}
    for label, m in (("hi", hi), ("flat", flat)):
        n = int(m.sum())
        if n == 0:
            continue
        cls = k[m]
        arid = np.isin(cls, [4, 5, 6, 7]).sum() / n
        polar = np.isin(cls, [29, 30]).sum() / n
        out[label + "_n"] = n
        out[label + "_arid"] = arid
        out[label + "_polar"] = polar
        out[label + "_rest"] = 1.0 - arid - polar
    land = z > 0
    out["land_arid"] = np.isin(k[land], [4, 5, 6, 7]).sum() / max(int(land.sum()), 1)
    return out


def main():
    for arm in sys.argv[1:]:
        rows = [run(r) for r in sorted(Path(arm).iterdir()) if (r / "fields" / "index.json").exists()]
        print(f"== {arm} ({len(rows)} runs)")
        keys = sorted({k for r in rows for k in r})
        for k in keys:
            v = [r[k] for r in rows if k in r]
            print(f"  median {k:12s} {statistics.median(v):.3f}  (n {len(v)}, min {min(v):.3f}, max {max(v):.3f})")


if __name__ == "__main__":
    main()
