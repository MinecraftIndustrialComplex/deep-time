#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P2-01: what a planet run of tools/review/p2-01-planet.sh `world` says about its folds and faults.

    python3 tools/review/p2-01-report.py <runs-folder> <name> [<name-without-relief>]

Prints the structure stage's deformation note, the amplifier's relief note, the strata-check's
structure census and, given a second run with `structure.deformation.relief=0`, the hypsometry of
the two baked terrains side by side (land share, quantiles of land height above the sea in metres,
the share of land cells that differ and by how much), read from each world's `elevation` layer
through `deeptime heightmap` at 8 blocks a sample.
"""
import json
import subprocess
import sys
from pathlib import Path

import numpy as np


def notes(world: Path) -> dict:
    m = json.loads((world / "manifest.json").read_text())
    out = {}
    for k, v in m.get("notes", {}).items():
        try:
            out[k] = json.loads(v) if isinstance(v, str) else v
        except ValueError:
            out[k] = v
    out["_scale"] = m["scale"]
    return out


def heights(runs: Path, name: str, scale: dict, step: int = 8) -> np.ndarray:
    c = scale["circumference_blocks"]
    out = runs / f"{name}-height.npy"
    if not out.exists():
        subprocess.run(
            [str(runs / f"deeptime-{name}"), "heightmap", "--world-data", str(runs / f"{name}-world"),
             f"--box={-c // 2},{-c // 4},{c // 2},{c // 4}", "--step", str(step), "--out", str(out)],
            check=True, stdout=subprocess.DEVNULL)
    a = np.load(out)
    return a[:, :, 0] if a.ndim == 3 else a


def hypsometry(h: np.ndarray, scale: dict) -> dict:
    sea = scale["sea_level_y"]
    mv = scale.get("m_per_block_v", 35.0)
    # Rows are latitudes of an equirectangular grid: weight by cos(latitude).
    rows = h.shape[0]
    lat = (np.arange(rows) + 0.5) / rows * np.pi - np.pi / 2
    wgt = np.repeat(np.cos(lat)[:, None], h.shape[1], axis=1)
    land = h > sea
    z = (h[land] - sea) * mv
    w = wgt[land]
    order = np.argsort(z)
    cw = np.cumsum(w[order]) / w.sum()
    q = lambda p: float(z[order][np.searchsorted(cw, p)])
    return {
        "land_share": float(wgt[land].sum() / wgt.sum()),
        "land_median_m": q(0.5), "land_p10_m": q(0.1), "land_p90_m": q(0.9), "land_p99_m": q(0.99),
        "land_mean_m": float((z * w).sum() / w.sum()), "highest_m": float(z.max()),
    }


def main() -> None:
    runs, name = Path(sys.argv[1]), sys.argv[2]
    n = notes(runs / f"{name}-world")
    d = n.get("structure", {}).get("deformation", {})
    for key in ["structures", "formed", "median", "lattice", "coverage", "traps", "events", "with_relief",
                "saturated", "overprinted", "subducted", "noddy_check"]:
        print(f"deformation.{key} = {json.dumps(d.get(key))}")
    print("amplify.structure_relief =", json.dumps(n.get("amplify", {}).get("structure_relief")))
    print("bake.structure =", json.dumps(n.get("bake", {}).get("structure")))
    strata = runs / f"{name}-strata.json"
    if strata.exists():
        s = json.loads(strata.read_text())
        print("strata-check.structure =", json.dumps(s.get("structure")))
        for k in ["columns", "land_columns"]:
            print(f"strata-check.{k} =", json.dumps(s.get(k)))
    scale = n["_scale"]
    h_on = heights(runs, name, scale)
    print(f"hypsometry[{name}] =", json.dumps(hypsometry(h_on, scale)))
    if len(sys.argv) > 3:
        other = sys.argv[3]
        h_off = heights(runs, other, notes(runs / f"{other}-world")["_scale"])
        print(f"hypsometry[{other}] =", json.dumps(hypsometry(h_off, scale)))
        sea = scale["sea_level_y"]
        land = (h_on > sea) | (h_off > sea)
        diff = (h_on - h_off)[land] * scale.get("m_per_block_v", 35.0)
        moved = np.abs(diff) >= 20.0
        relief_picture(runs, name, other, h_on, h_off, scale, Path("review/out/p2-01/relief.png"))
        print("difference =", json.dumps({
            "land_cells": int(land.sum()),
            "cells_a_block_or_more_apart": float(moved.mean()),
            "mean_abs_m_where_apart": float(np.abs(diff[moved]).mean()) if moved.any() else 0.0,
            "max_up_m": float(diff.max()), "max_down_m": float(-diff.min()),
            "land_became_sea": int(((h_off > sea) & (h_on <= sea)).sum()),
            "sea_became_land": int(((h_off <= sea) & (h_on > sea)).sum()),
        }))


def relief_picture(runs: Path, on: str, off: str, h_on: np.ndarray, h_off: np.ndarray, scale: dict,
                   out: Path, step: int = 8, side: int = 640) -> None:
    """Hillshades of the terrain with and without the structures' relief, and their difference, over
    the `side`-block square where the two differ most (found on the coarse arrays, redrawn at one
    block a pixel)."""
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    c = scale["circumference_blocks"]
    sea = scale["sea_level_y"]
    mv = scale.get("m_per_block_v", 35.0)
    d = np.abs(h_on - h_off) * ((h_on > sea) & (h_off > sea))
    k = side // step
    rows, cols = d.shape
    # Away from the poles (the map is stretched there).
    lo, hi = rows // 6, rows - rows // 6
    best, at = -1.0, (0, 0)
    for j in range(lo, hi - k, k // 2):
        for i in range(0, cols - k, k // 2):
            v = float(d[j:j + k, i:i + k].sum())
            if v > best:
                best, at = v, (j, i)
    x0 = -c // 2 + at[1] * step
    z0 = -c // 4 + at[0] * step
    box = f"--box={x0},{z0},{x0 + side},{z0 + side}"
    tiles = {}
    for name in (on, off):
        f = runs / f"{name}-relief.npy"
        subprocess.run([str(runs / f"deeptime-{on}"), "heightmap", "--world-data", str(runs / f"{name}-world"),
                        box, "--step", "1", "--out", str(f)], check=True, stdout=subprocess.DEVNULL)
        tiles[name] = np.load(f)[:, :, 0]

    def shade(h: np.ndarray) -> np.ndarray:
        gy, gx = np.gradient(h)
        return np.clip(0.5 + 0.35 * (gx - gy), 0.0, 1.0)

    fig, ax = plt.subplots(1, 3, figsize=(18, 6.4), dpi=110)
    ax[0].imshow(shade(tiles[off]), cmap="gray", vmin=0, vmax=1)
    ax[0].set_title("Without the structures' relief")
    ax[1].imshow(shade(tiles[on]), cmap="gray", vmin=0, vmax=1)
    ax[1].set_title("With fault scarps, rift valleys and fold ridges")
    diff = (tiles[on] - tiles[off]) * mv
    lim = max(20.0, float(np.percentile(np.abs(diff), 99.5)))
    im = ax[2].imshow(diff, cmap="RdBu", vmin=-lim, vmax=lim)
    ax[2].set_title("Difference, m (blue: higher with the structures)")
    fig.colorbar(im, ax=ax[2], fraction=0.046)
    for a in ax:
        a.set_xticks([])
        a.set_yticks([])
    fig.suptitle(f"Blocks x {x0}..{x0 + side}, z {z0}..{z0 + side} (one block a pixel; north up)")
    fig.tight_layout()
    fig.savefig(out, facecolor="white")
    print("relief picture =", json.dumps({"file": str(out), "x0": x0, "z0": z0, "side": side,
                                          "max_up_m": float(diff.max()), "max_down_m": float(-diff.min())}))


if __name__ == "__main__":
    main()
