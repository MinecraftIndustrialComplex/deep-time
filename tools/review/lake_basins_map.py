#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""A picture of a checkpoint's closed basins (`lake_basins.py`'s rule) by what is under them.

Writes <out>.png: the whole L1 map (sea dark, land grey by height) with the lake cells coloured
by the crust under them (oceanic crust red, continental crust blue; a basin floor below sea level
darker), and <out>-zoom-<k>.png crops of the largest basins with their elevation, crust and the
step's uplift side by side.

Usage: lake_basins_map.py <dump folder> <out prefix> [--zooms 6]
"""
from __future__ import annotations

import argparse
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
import lake_basins as lb  # noqa: E402


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("folder", type=Path)
    ap.add_argument("out")
    ap.add_argument("--zooms", type=int, default=6)
    ap.add_argument("--field", default="surface/elevation_m")
    a = ap.parse_args()
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    f, glob = lb.load(a.folder)
    sea = glob["globals"].get("climate/sea_level_m", 0.0)
    h = f(a.field)
    rows, cols = h.shape
    cell_km2 = lb.PLANET_KM2 / (rows * cols)
    under = h < sea - lb.BLOCK_M
    lab, _ = lb.label(under)
    sizes = np.bincount(lab.ravel())
    big = sizes * cell_km2 >= lb.OPEN_SEA_KM2
    big[0] = False
    open_sea = big[lab]
    fill = lb.priority_flood(h, open_sea)
    depth = fill - h
    land = ~open_sea
    wet = (depth > 0.0) & land
    blab, nb = lb.label(wet)
    dmax = np.zeros(nb + 1)
    np.maximum.at(dmax, blab.ravel(), depth.ravel())
    lake = wet & (dmax[blab] >= lb.BLOCK_M)
    crust = f("plates/crust_type")
    uplift = f("plates/uplift_mm_yr")
    rgb = np.zeros((rows, cols, 3))
    rgb[open_sea] = (0.10, 0.14, 0.22)
    g = np.clip((h - sea) / 3000.0, 0, 1)
    shade = 0.55 + 0.4 * g
    rgb[land] = np.stack([shade, shade, shade], -1)[land]
    oce = lake & (crust < 0.5)
    con = lake & (crust >= 0.5)
    deep = h < sea
    rgb[oce & ~deep] = (0.95, 0.35, 0.30)
    rgb[oce & deep] = (0.60, 0.10, 0.10)
    rgb[con & ~deep] = (0.35, 0.70, 0.98)
    rgb[con & deep] = (0.10, 0.30, 0.70)
    fig = plt.figure(figsize=(cols / 100 * 2, rows / 100 * 2), dpi=100)
    ax = fig.add_axes([0, 0, 1, 1])
    ax.imshow(rgb, interpolation="nearest")
    ax.axis("off")
    fig.savefig(f"{a.out}.png")
    plt.close(fig)
    # The largest basins.
    counts = np.bincount(blab.ravel(), weights=lake.ravel().astype(float))
    counts[0] = 0
    for k, b in enumerate(np.argsort(-counts)[: a.zooms]):
        if counts[b] == 0:
            break
        rr, cc = np.nonzero(blab == b)
        r0, c0 = int(np.median(rr)), int(np.median(cc))
        half_r, half_c = 40, 70
        rs = np.clip(np.arange(r0 - half_r, r0 + half_r), 0, rows - 1)
        cs = np.arange(c0 - half_c, c0 + half_c) % cols
        win = np.ix_(rs, cs)
        fig, axs = plt.subplots(1, 3, figsize=(15, 4.6))
        im = axs[0].imshow(h[win] - sea, cmap="terrain", vmin=-3000, vmax=3000)
        axs[0].contour(blab[win] == b, levels=[0.5], colors="k", linewidths=0.8)
        axs[0].set_title(f"elevation above sea, m (basin {int(counts[b])} cells, max depth {dmax[b]:.0f} m)")
        fig.colorbar(im, ax=axs[0], shrink=0.8)
        im = axs[1].imshow(crust[win], cmap="coolwarm_r", vmin=0, vmax=1)
        axs[1].contour(blab[win] == b, levels=[0.5], colors="k", linewidths=0.8)
        axs[1].set_title("crust (0 oceanic, 1 continental)")
        up = uplift[win] * 2000.0
        im = axs[2].imshow(up, cmap="RdBu", vmin=-500, vmax=500)
        axs[2].contour(blab[win] == b, levels=[0.5], colors="k", linewidths=0.8)
        axs[2].set_title("plate uplift of the last step, m")
        fig.colorbar(im, ax=axs[2], shrink=0.8)
        for x in axs:
            x.set_xticks([])
            x.set_yticks([])
        fig.tight_layout()
        fig.savefig(f"{a.out}-zoom-{k}.png", dpi=80)
        plt.close(fig)
    return 0


if __name__ == "__main__":
    sys.exit(main())
