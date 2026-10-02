#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Cell-scale artifacts of a run's deep-time surface (the flexure follow-up, build log 2026-10-02).

    deeptime dump-fields --in run.dtcp --out fields --fields surface/elevation_m,surface/flexure_m,\\
        surface/deflection_m,plates/plate_id
    python3 tools/review/flexure_artifacts.py --label NAME --sea 0 fields [fields ...]

For each folder: the land's cell-to-cell roughness (|z - mean of the four neighbours|: median and
99th percentile), and the zigzag share (the cell and its next neighbour along x or y are both strict
extrema of opposite kind, each by more than 20 m: ... low, high, low, high ...) of the elevation and
of the flexure field, over margin cells (within 3 cells of a plate boundary), plate interiors, and
the polar band (|latitude| > 70 degrees, the stripes). Prints one JSON line per folder.
Needs numpy.
"""

import argparse
import json
from pathlib import Path

import numpy as np


def load(folder: Path) -> dict:
    index = json.loads((folder / "index.json").read_text())
    return {k: np.fromfile(folder / e["file"], dtype="<f4").reshape(e["h"], e["w"]).astype(float)
            for k, e in index.items()}


def zigzag(v: np.ndarray, t: float = 20.0) -> np.ndarray:
    """Boolean map of the cells that zigzag with their next neighbour along x or y."""
    def ext_x(a):
        l, r = np.roll(a, 1, 1), np.roll(a, -1, 1)
        return ((l - a) * (a - r) < 0) & (np.abs(l - a) > t) & (np.abs(a - r) > t)

    def ext_y(a):
        e = np.zeros_like(a, dtype=bool)
        l, m, r = a[:-2], a[1:-1], a[2:]
        e[1:-1] = ((l - m) * (m - r) < 0) & (np.abs(l - m) > t) & (np.abs(m - r) > t)
        return e

    ex, ey = ext_x(v), ext_y(v)
    zx = ex & np.roll(ex, -1, 1)
    zy = np.zeros_like(ey)
    zy[:-1] = ey[:-1] & ey[1:]
    return zx | zy


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--label", default="")
    ap.add_argument("--sea", type=float, default=0.0)
    ap.add_argument("folders", nargs="+", type=Path)
    a = ap.parse_args()
    for folder in a.folders:
        f = load(folder)
        z = f["surface/elevation_m"] - a.sea
        h, w = z.shape
        lat = 90 - (np.arange(h) + 0.5) * 180 / h
        polar = np.repeat((np.abs(lat) > 70)[:, None], w, 1)
        pid = f.get("plates/plate_id")
        margin = np.zeros_like(z, dtype=bool)
        if pid is not None:
            b = (pid != np.roll(pid, 1, 1)) | (pid != np.roll(pid, -1, 1))
            b[1:] |= pid[1:] != pid[:-1]
            b[:-1] |= pid[:-1] != pid[1:]
            m = b.copy()
            for _ in range(3):
                g = m.copy()
                g |= np.roll(m, 1, 1) | np.roll(m, -1, 1)
                g[1:] |= m[:-1]
                g[:-1] |= m[1:]
                m = g
            margin = m
        mean4 = (np.roll(z, 1, 1) + np.roll(z, -1, 1) + np.vstack([z[:1], z[:-1]]) + np.vstack([z[1:], z[-1:]])) / 4
        dev = np.abs(z - mean4)
        land = (z > 0) & ~polar
        out = {"run": a.label or folder.name,
               "roughness_land_p50_m": round(float(np.median(dev[land])), 1),
               "roughness_land_p99_m": round(float(np.percentile(dev[land], 99)), 1),
               "roughness_sea_p99_m": round(float(np.percentile(dev[(z <= 0) & ~polar], 99)), 1)}
        # Land whose four neighbours are land too: the coast (an island cell beside deep sea)
        # left out.
        inland = land.copy()
        inland &= np.roll(z, 1, 1) > 0
        inland &= np.roll(z, -1, 1) > 0
        inland[1:] &= z[:-1] > 0
        inland[:-1] &= z[1:] > 0
        out["roughness_inland_p99_m"] = round(float(np.percentile(dev[inland], 99)), 1)
        out["roughness_inland_margin_p99_m"] = round(float(np.percentile(dev[inland & margin], 99)), 1) if (inland & margin).any() else None
        coast = land & ~inland
        out["coast_cells_on_margin"] = int((coast & margin).sum())
        out["coast_cells"] = int(coast.sum())
        out["roughness_land_margin_p99_m"] = round(float(np.percentile(dev[land & margin], 99)), 1)
        out["roughness_land_interior_p99_m"] = round(float(np.percentile(dev[land & ~margin], 99)), 1)
        out["land_margin_share"] = round(float((land & margin).sum() / max(land.sum(), 1)), 3)
        # The roughest 1 % of land cells: how many of them are zigzag cells.
        rough = land & (dev >= np.percentile(dev[land], 99))
        out["roughest_land_zigzag_share"] = round(float(zigzag(z)[rough].mean()), 3)
        out["roughest_land_on_margin_share"] = round(float(margin[rough].mean()), 3)
        for name in ("surface/elevation_m", "surface/flexure_m"):
            if name not in f:
                continue
            zz = zigzag(f[name])
            key = name.split("/")[1].split("_")[0]
            for region, sel in (("margin", margin & ~polar), ("interior", ~margin & ~polar), ("polar", polar)):
                out[f"zigzag_{key}_{region}"] = round(float(zz[sel].mean()), 5)
        print(json.dumps(out))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
