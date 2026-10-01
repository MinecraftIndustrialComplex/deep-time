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
    mv = scale.get("m_per_block_v", 20.0)
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
        diff = (h_on - h_off)[land] * scale.get("m_per_block_v", 20.0)
        moved = np.abs(diff) >= 20.0
        print("difference =", json.dumps({
            "land_cells": int(land.sum()),
            "cells_a_block_or_more_apart": float(moved.mean()),
            "mean_abs_m_where_apart": float(np.abs(diff[moved]).mean()) if moved.any() else 0.0,
            "max_up_m": float(diff.max()), "max_down_m": float(-diff.min()),
            "land_became_sea": int(((h_off > sea) & (h_on <= sea)).sum()),
            "sea_became_land": int(((h_off <= sea) & (h_on > sea)).sum()),
        }))


if __name__ == "__main__":
    main()
