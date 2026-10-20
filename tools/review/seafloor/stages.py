#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Sea-floor slopes stage by stage (the sea-floor spikes study, 2026-10-20), on `deeptime dump-fields`
output: the plate model's, the surface stage's (L1) and the young landscape's (L2) elevation on the
simulation's Lambert grids, in ground metres.

    stages.py <dumpdir> <out.json> [--png out.png --boxes lat0,lat1,lon0,lon1;...]

Deep sea floor = elevation below -210 m, |lat| <= 75. Slopes from centred differences in ground
metres (east-west cell r*300*cos(lat), north-south r*300/cos(lat)); percentiles are plain (the grid
is equal-area). "At L1" = the field block-averaged to 16 sim blocks per cell first (4.8 km).
"""

import argparse
import json
import math
from pathlib import Path

import numpy as np

C = 16384
R = C / (2 * math.pi)
M = 300.0  # metres per sim block


def zo():
    q = 64
    return int(math.ceil(R / q) * q)


def load(d, name, index):
    e = index[name]
    a = np.fromfile(Path(d) / e["file"], dtype="<f4").reshape(e["h"], e["w"]).astype(np.float64)
    return a, e["r"]


def lat_rows(h, r):
    z = -zo() + (np.arange(h) + 0.5) * r
    return np.degrees(np.arcsin(np.clip(-z / R, -1, 1)))


def slopes(a, r):
    lat = lat_rows(a.shape[0], r)
    c = np.cos(np.radians(lat))
    dx = (r * M * np.maximum(c, 1e-3))[:, None]
    dy = (r * M / np.maximum(c, 1e-3))[:, None]
    gx = (np.roll(a, -1, axis=1) - np.roll(a, 1, axis=1)) / (2 * dx)
    gy = np.gradient(a, axis=0) / dy
    return np.degrees(np.arctan(np.hypot(gx, gy))), lat


def coarsen(a, k):
    h, w = (a.shape[0] // k) * k, (a.shape[1] // k) * k
    return a[:h, :w].reshape(h // k, k, w // k, k).mean(axis=(1, 3))


def stats(a, r):
    deg, lat = slopes(a, r)
    m = (a < -210.0) & (np.abs(lat)[:, None] <= 75.0)
    d = deg[m]
    # The biggest drop to a neighbour (m), the cliff measure.
    drops = np.zeros_like(a)
    for sy, sx in ((0, 1), (0, -1), (1, 0), (-1, 0)):
        drops = np.maximum(drops, a - np.roll(np.roll(a, sy, 0), sx, 1))
    return {
        "cells": int(m.sum()),
        "deg_pct": {str(p): float(np.percentile(d, p)) for p in (50, 75, 90, 95, 99, 99.9)},
        "share_over": {str(t): float((d > t).mean()) for t in (5, 10, 15, 20, 30)},
        "share_drop_over_1km": float((drops[m] > 1000).mean()),
        "share_drop_over_2km": float((drops[m] > 2000).mean()),
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dump")
    ap.add_argument("out")
    ap.add_argument("--fields", default="plates/elevation_m,surface/elevation_m,young/elevation_m")
    ap.add_argument("--png")
    ap.add_argument("--boxes", default="")
    args = ap.parse_args()
    index = json.loads((Path(args.dump) / "index.json").read_text())
    res = {}
    fields = {}
    for name in args.fields.split(","):
        if name not in index:
            print("missing", name)
            continue
        a, r = load(args.dump, name, index)
        fields[name] = (a, r)
        s = {"r": r, "native": stats(a, r)}
        if r < 16:
            s["at_L1"] = stats(coarsen(a, 16 // r), 16)
        res[name] = s
        print(name, json.dumps(s))
    Path(args.out).write_text(json.dumps(res, indent=1))
    if args.png and args.boxes:
        import matplotlib
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
        from matplotlib.colors import LightSource
        boxes = [tuple(float(v) for v in b.split(",")) for b in args.boxes.split(";") if b]
        ls = LightSource(azdeg=315, altdeg=35)
        fig, axs = plt.subplots(len(boxes), len(fields), figsize=(6 * len(fields), 5 * len(boxes)), squeeze=False)
        for bi, (la0, la1, lo0, lo1) in enumerate(boxes):
            for fi, (name, (a, r)) in enumerate(fields.items()):
                lat = lat_rows(a.shape[0], r)
                lon = (np.arange(a.shape[1]) + 0.5) * r / C * 360.0
                rows = np.where((lat >= la0) & (lat <= la1))[0]
                cols = np.where((lon >= lo0) & (lon <= lo1))[0]
                if len(rows) < 2 or len(cols) < 2:
                    continue
                sub = a[rows[0]:rows[-1] + 1, cols[0]:cols[-1] + 1]
                ax = axs[bi][fi]
                ax.imshow(ls.shade(sub, cmap=plt.cm.gist_earth, vert_exag=0.02, blend_mode="soft", vmin=-9000, vmax=2000),
                          extent=[lon[cols[0]], lon[cols[-1]], lat[rows[-1]], lat[rows[0]]], aspect="auto", interpolation="nearest")
                ax.set_title(f"{name} r{r}  box {bi}: {sub.min():.0f}..{sub.max():.0f} m", fontsize=9)
        fig.savefig(args.png, dpi=70, bbox_inches="tight")


if __name__ == "__main__":
    main()
