#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The closed basins of a checkpoint's surface, and what made them.

Reads a `deeptime dump-fields` folder (index.json, globals.json, <field>.f32) of a plate-era
checkpoint and finds every closed depression of the surface stage's L1 elevation the way the
"Rivers & lakes" view does (`hydro::network`): the open sea is a connected body at least a block
below sea level and at least 5,000 km² in area; a Priority-Flood from it fills every depression to
its spill; a lake is a basin at least a block deep (35 m) and at least one cell. It then reports,
for the lake cells and the basins:

- the share of land, of ice-free land, and the largest basin as a share of the planet;
- the basins' depth, area and the crust under them (continental or oceanic, the plates' regime);
- the step's uplift at each basin's floor against its spill: how much of the depth the plate
  model's differential uplift (settling, collisions, thermal subsidence) makes in one step;
- the deposition in the basin against its depth: how long the sediment would take to fill it;
- the surface's flexure and ice there.

Usage: lake_basins.py <dump folder> [--step-myr 2.0] [--json out.json]
"""
from __future__ import annotations

import argparse
import heapq
import json
import sys
from pathlib import Path

import numpy as np

BLOCK_M = 35.0
OPEN_SEA_KM2 = 5000.0
PLANET_KM2 = 4.0 * np.pi * 782.278376285284 ** 2


def load(folder: Path):
    index = json.loads((folder / "index.json").read_text())
    glob = json.loads((folder / "globals.json").read_text())

    def f(name: str) -> np.ndarray | None:
        e = index.get(name)
        if e is None:
            return None
        a = np.fromfile(folder / e["file"], dtype="<f4")
        return a.reshape(e["h"], e["w"]).astype(np.float64)

    return f, glob


def label(mask: np.ndarray) -> tuple[np.ndarray, int]:
    """4-connected components with the x seam joined."""
    from scipy import ndimage

    lab, n = ndimage.label(mask)
    # Join across the seam with a union-find on the labels.
    parent = np.arange(n + 1)

    def find(a):
        while parent[a] != a:
            parent[a] = parent[parent[a]]
            a = parent[a]
        return a

    left, right = lab[:, 0], lab[:, -1]
    for a, b in zip(left, right):
        if a and b:
            ra, rb = find(a), find(b)
            if ra != rb:
                parent[max(ra, rb)] = min(ra, rb)
    roots = np.array([find(i) for i in range(n + 1)])
    return roots[lab], n


def priority_flood(h: np.ndarray, seed: np.ndarray) -> np.ndarray:
    """The spill level of every cell (4-connected, x wraps), from the seed cells."""
    rows, cols = h.shape
    fill = np.full(h.shape, np.inf)
    done = np.zeros(h.shape, dtype=bool)
    heap: list[tuple[float, int]] = []
    for idx in np.flatnonzero(seed):
        r, c = divmod(int(idx), cols)
        fill[r, c] = h[r, c]
        done[r, c] = True
        heap.append((h[r, c], int(idx)))
    heapq.heapify(heap)
    hv = h.ravel()
    fv = fill.ravel()
    dv = done.ravel()
    while heap:
        lvl, idx = heapq.heappop(heap)
        r, c = divmod(idx, cols)
        for rr, cc in ((r - 1, c), (r + 1, c), (r, (c - 1) % cols), (r, (c + 1) % cols)):
            if rr < 0 or rr >= rows:
                continue
            j = rr * cols + cc
            if dv[j]:
                continue
            dv[j] = True
            v = max(hv[j], lvl)
            fv[j] = v
            heapq.heappush(heap, (v, j))
    return fv.reshape(h.shape)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("folder", type=Path)
    ap.add_argument("--field", default="surface/elevation_m")
    ap.add_argument("--step-myr", type=float, default=2.0)
    ap.add_argument("--json", type=Path)
    a = ap.parse_args()
    f, glob = load(a.folder)
    sea = glob["globals"].get("climate/sea_level_m", 0.0)
    h = f(a.field)
    rows, cols = h.shape
    cell_km2 = PLANET_KM2 / (rows * cols)
    # The open sea.
    under = h < sea - BLOCK_M
    lab, _ = label(under)
    sizes = np.bincount(lab.ravel())
    big = sizes * cell_km2 >= OPEN_SEA_KM2
    big[0] = False
    open_sea = big[lab]
    fill = priority_flood(h, open_sea)
    depth = fill - h
    land = ~open_sea
    wet = (depth > 0.0) & land
    blab, nb = label(wet)
    # Per basin: its cells, max depth, floor and spill.
    flat = blab.ravel()
    order = np.argsort(flat, kind="stable")
    starts = np.searchsorted(flat[order], np.arange(nb + 2))
    uplift = f("plates/uplift_mm_yr")
    dep = f("surface/deposition_m_myr")
    ero = f("surface/erosion_m_myr")
    flex = f("surface/flexure_m")
    ice = f("climate/ice_thickness_m")
    crust = f("plates/crust_type")
    regime = f("regime")
    glac = f("surface/glacial_erosion_m_myr")
    tmean = f("climate/t_mean_c")
    precip = f("climate/precip_mm")
    hv, dv, fv = h.ravel(), depth.ravel(), fill.ravel()
    basins = []
    for b in range(1, nb + 1):
        cells = order[starts[b]:starts[b + 1]]
        if len(cells) == 0:
            continue
        dmax = float(dv[cells].max())
        if dmax < BLOCK_M:
            continue
        floor = int(cells[np.argmax(dv[cells])])
        level = float(fv[floor])
        # The spill: the lowest cell on the basin's rim (a neighbour outside the basin at the
        # level, or the cell next to it).
        r0, c0 = divmod(floor, cols)
        rim = []
        for idx in cells:
            r, c = divmod(int(idx), cols)
            for rr, cc in ((r - 1, c), (r + 1, c), (r, (c - 1) % cols), (r, (c + 1) % cols)):
                if 0 <= rr < rows and blab[rr, cc] != b:
                    rim.append(rr * cols + cc)
        rim = np.unique(np.array(rim, dtype=np.int64))
        spill = int(rim[np.argmin(hv[rim])]) if len(rim) else floor
        rec = {
            "cells": int(len(cells)),
            "km2": float(len(cells) * cell_km2),
            "max_depth_m": dmax,
            "mean_depth_m": float(dv[cells].mean()),
            "level_m": level - sea,
            "lat_row": r0,
            "col": c0,
        }
        if uplift is not None:
            uv = uplift.ravel()
            # mm/yr = m/kyr; a step of `step_myr` Myr.
            rec["dam_uplift_m"] = float((uv[spill] - uv[floor]) * a.step_myr * 1000.0)
            rec["floor_uplift_m"] = float(uv[floor] * a.step_myr * 1000.0)
            rec["spill_uplift_m"] = float(uv[spill] * a.step_myr * 1000.0)
            rec["mean_uplift_m"] = float(uv[cells].mean() * a.step_myr * 1000.0)
        if dep is not None:
            rec["deposition_m_myr"] = float(dep.ravel()[cells].mean())
        if ero is not None:
            rec["spill_erosion_m_myr"] = float(ero.ravel()[spill])
        if flex is not None:
            rec["flexure_floor_m"] = float(flex.ravel()[floor])
            rec["flexure_spill_m"] = float(flex.ravel()[spill])
        if ice is not None:
            rec["ice_m"] = float(ice.ravel()[cells].mean())
        if crust is not None:
            rec["crust"] = float(np.median(crust.ravel()[cells]))
        if regime is not None:
            rec["regime"] = float(np.median(regime.ravel()[cells]))
        if glac is not None:
            rec["glacial_erosion_m_myr"] = float(glac.ravel()[cells].mean())
        if tmean is not None:
            rec["t_mean_c"] = float(tmean.ravel()[cells].mean())
        if precip is not None:
            rec["precip_mm"] = float(precip.ravel()[cells].mean())
        basins.append(rec)
    lake_cells = sum(b["cells"] for b in basins)
    land_cells = int(land.sum())
    out = {
        "field": a.field,
        "t_years": glob["t_years"],
        "sea_m": sea,
        "cell_km2": cell_km2,
        "land_share_of_planet": land_cells / (rows * cols),
        "lake_share_of_land": lake_cells / land_cells,
        "basins": len(basins),
        "shallow_basin_cells_share_of_land": float(wet.sum() - lake_cells) / land_cells,
        "largest_share_of_planet": max((b["cells"] for b in basins), default=0) / (rows * cols),
    }
    if ice is not None:
        free = land & (ice < 1.0)
        out["ice_free_land_share"] = float(free.sum()) / land_cells
    arr = lambda k: np.array([b[k] for b in basins if k in b])
    w = arr("cells").astype(float)

    def wq(k, qs=(0.1, 0.5, 0.9)):
        v = arr(k)
        if len(v) != len(w) or len(v) == 0:
            return None
        o = np.argsort(v)
        cw = np.cumsum(w[o])
        return {f"p{int(q * 100)}": float(v[o][np.searchsorted(cw, q * cw[-1])]) for q in qs}

    for k in ("max_depth_m", "mean_depth_m", "km2", "dam_uplift_m", "floor_uplift_m",
              "spill_uplift_m", "mean_uplift_m", "deposition_m_myr", "spill_erosion_m_myr",
              "flexure_floor_m", "flexure_spill_m", "ice_m", "glacial_erosion_m_myr", "t_mean_c",
              "precip_mm", "level_m"):
        out[f"{k}_by_area"] = wq(k)
    if len(basins):
        dam = arr("dam_uplift_m")
        md = arr("max_depth_m")
        if len(dam) == len(md):
            out["area_share_dam_over_half_depth"] = float(w[dam > 0.5 * md].sum() / w.sum())
            out["area_share_dam_positive"] = float(w[dam > 0].sum() / w.sum())
        dep_b = arr("deposition_m_myr")
        if len(dep_b) == len(md):
            fill_myr = md / np.maximum(dep_b, 1e-6)
            o = np.argsort(fill_myr)
            cw = np.cumsum(w[o])
            out["fill_time_myr_by_area"] = {f"p{q}": float(fill_myr[o][np.searchsorted(cw, q / 100 * cw[-1])]) for q in (10, 50, 90)}
        # Per lake cell: the crust and the regime under it.
        in_lake = np.zeros(rows * cols, dtype=bool)
        for b in range(1, nb + 1):
            cells = order[starts[b]:starts[b + 1]]
            if len(cells) and dv[cells].max() >= BLOCK_M:
                in_lake[cells] = True
        if crust is not None:
            cv = crust.ravel()[in_lake]
            out["lake_cells_crust"] = {str(k): float(np.mean(cv == k)) for k in np.unique(cv[np.isfinite(cv)])}
            out["lake_cells_crust"]["nan"] = float(np.mean(~np.isfinite(cv)))
        if regime is not None:
            rv = regime.ravel()[in_lake]
            out["lake_cells_regime"] = {str(k): float(np.mean(rv == k)) for k in np.unique(rv[np.isfinite(rv)])}
        # Lake cells whose ground is under the sea level (a basin below the sea, cut off from it).
        out["lake_cells_ground_below_sea"] = float(np.mean(hv[in_lake] < sea))
        out["lake_cells_depth_over_500m"] = float(np.mean(dv[in_lake] > 500.0))
        if uplift is not None:
            uv = uplift.ravel()[in_lake] * a.step_myr * 1000.0
            out["lake_cells_step_uplift_m"] = {f"p{q}": float(np.percentile(uv, q)) for q in (5, 25, 50, 75, 95)}
        out["land_cells_step_uplift_m"] = {f"p{q}": float(np.percentile(uplift.ravel()[land.ravel()] * a.step_myr * 1000.0, q)) for q in (5, 25, 50, 75, 95)} if uplift is not None else None
        lv = arr("level_m")
        out["area_share_level_below_100m"] = float(w[lv < 100.0].sum() / w.sum())
    out["largest"] = sorted(basins, key=lambda b: -b["cells"])[:8]
    print(json.dumps(out, indent=1))
    if a.json:
        a.json.write_text(json.dumps({"summary": out, "basins": basins}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
