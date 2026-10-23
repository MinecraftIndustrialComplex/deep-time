#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `basins` measure: the closed basins of the final L1 surface (the savanna pit task,
2026-10-22), from one pool run folder, printed as a JSON object on stdout.

    python3 -I measure_basins.py <run folder> [--fields DIR] [--field elevation] [--list N]

The surface is the dump's `elevation` field (blocks; sea level at block 63). The open sea is what
the hydrology calls it (HYD-012, HYD-100): every cell a block or more under sea level in an
8-connected body (x wraps) of at least 5,000 km2 (cells of r blocks at 300 m a block, the quick
16k planet's scale). A priority flood from the open sea gives every cell its spill level; a
closed basin is an 8-connected body of cells the flood raised by half a block or more, and its
depth is its spill level less its floor.

  basins_10, basins_30   closed basins deeper than 10 and 30 blocks (350 m and 1,050 m)
  below_sea_10           of basins_10, those whose floor lies under sea level
  dry_rim_10             of basins_10, those whose rim (the land cells round it) is mostly
                         Koppen B (arid and semi-arid); needs climate/koppen, else -1
  deepest_blocks         the deepest closed basin's depth, blocks
  closed_below_sea_kb2   ground under sea level the open sea does not reach, 1,000 blocks2
  closed_volume_km3      the room of the closed basins deeper than 10 blocks, km3 (35 m and
                         300 m a block)

`--list N` prints the N deepest basins (depth, floor, spill, area, the cell of the floor) before
the JSON line. Needs numpy and scipy. Version 1.
"""
import argparse
import heapq
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

import numpy as np  # noqa: E402
from scipy import ndimage  # noqa: E402

import continents_census as cc  # noqa: E402

# The hydrology's open sea (crates/deeptime-core/src/hydro/mod.rs OPEN_SEA_KM2). ledger: HYD-012
OPEN_SEA_KM2 = 5000.0
# The quick 16k planet's horizontal scale, m a block (earthlike_quick_16k scale.m_per_block_h).
M_PER_BLOCK_H = 300.0
KOPPEN_B = (4, 5, 6, 7)  # BWh, BWk, BSh, BSk (climate/koppen.rs CODES)
KOPPEN_SEA = 31


def label_wrap(mask):
    return cc.label_wrap(mask)


def flood(z, seed, ok):
    """Spill levels: a priority flood from the `seed` cells over the `ok` cells (8 neighbours, x
    wraps, rows closed). Cells outside `ok` are walls."""
    h, w = z.shape
    filled = np.where(seed, z, np.inf)
    done = seed.copy()
    heap = [(float(z[j, i]), j, i) for j, i in zip(*np.nonzero(seed))]
    heapq.heapify(heap)
    nb = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)]
    while heap:
        lv, j, i = heapq.heappop(heap)
        for dj, di in nb:
            jj = j + dj
            if jj < 0 or jj >= h:
                continue
            ii = (i + di) % w
            if done[jj, ii] or not ok[jj, ii]:
                continue
            done[jj, ii] = True
            f = max(lv, float(z[jj, ii]))
            filled[jj, ii] = f
            heapq.heappush(heap, (f, jj, ii))
    return filled, done


def load(fields_dir, name):
    """One dumped field as (array, blocks a cell), or (None, 0) when the dump lacks it."""
    idx = json.loads((Path(fields_dir) / "index.json").read_text())
    e = idx.get(name)
    if e is None:
        return None, 0
    return np.fromfile(Path(fields_dir) / e["file"], "<f4").reshape(e["h"], e["w"]), e["r"]


def census(fields_dir, field="elevation", top=0):
    a, r = load(fields_dir, field)
    h, w = a.shape
    _lat, ok_r = cc.lat_of_rows(h, r, w * r)
    ok = np.repeat(ok_r[:, None], w, 1) & np.isfinite(a)
    z = np.where(ok, a - cc.SEA_Y, np.nan)  # blocks over sea level
    cell_km2 = (r * M_PER_BLOCK_H / 1000.0) ** 2
    min_sea = max(1, int(np.ceil(OPEN_SEA_KM2 / cell_km2)))
    deep = ok & (z <= -1.0)
    lab = label_wrap(deep)
    sizes = np.bincount(lab.ravel())
    sizes[0] = 0
    sea = deep & (sizes[lab] >= min_sea)
    filled, reached = flood(np.nan_to_num(z, nan=1e9), sea, ok)
    depth = np.where(ok & reached, filled - np.nan_to_num(z), 0.0)
    basin = ok & ~sea & (depth >= 0.5)
    blab = label_wrap(basin)
    ids = np.unique(blab[basin])
    kop, kr = load(fields_dir, "climate/koppen")
    if kop is not None and kr != r:
        # A finer surface than the climate's grid: each climate cell over its fine cells.
        kop = np.repeat(np.repeat(kop, kr // r, 0), kr // r, 1)[:h, :w]
    rows = []
    ring_struct = np.ones((3, 3), bool)
    for b in ids:
        m = blab == b
        dz = depth[m]
        k = int(np.argmax(dz))
        jj, ii = np.nonzero(m)
        floor = float(z[m].min())
        spill = float(filled[m].max())
        row = {"depth": float(dz.max()), "floor": floor, "spill": spill, "cells": int(m.sum()),
               "kb2": m.sum() * r * r / 1000.0, "at": [int(ii[k]), int(jj[k])],
               "room_km3": float(dz.sum()) * 35.0 / 1000.0 * cell_km2}
        if kop is not None and row["depth"] > 10.0:
            ring = ndimage.binary_dilation(m, ring_struct) & ~m & ok
            kk = kop[ring]
            kk = kk[np.isfinite(kk) & (kk != KOPPEN_SEA)].astype(int)
            row["dry_rim"] = bool(kk.size and np.isin(kk, KOPPEN_B).mean() > 0.5)
        rows.append(row)
    rows.sort(key=lambda x: -x["depth"])
    for x in rows[:top]:
        print("basin depth %6.1f b floor %7.1f spill %6.1f cells %4d (%.0f kb2) floor cell %s room %.0f km3%s" % (
            x["depth"], x["floor"], x["spill"], x["cells"], x["kb2"], x["at"], x["room_km3"],
            "" if "dry_rim" not in x else (" dry rim" if x["dry_rim"] else " wet rim")))
    d10 = [x for x in rows if x["depth"] > 10.0]
    closed_low = ok & ~sea & (np.nan_to_num(z) < 0.0) & (depth > 0.0)
    return {
        "basins_10": len(d10),
        "basins_30": sum(1 for x in rows if x["depth"] > 30.0),
        "below_sea_10": sum(1 for x in d10 if x["floor"] < 0.0),
        "dry_rim_10": sum(1 for x in d10 if x.get("dry_rim")) if kop is not None else -1,
        "deepest_blocks": rows[0]["depth"] if rows else 0.0,
        "closed_below_sea_kb2": float(closed_low.sum()) * r * r / 1000.0,
        "closed_volume_km3": sum(x["room_km3"] for x in d10),
    }


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("run")
    ap.add_argument("--fields")
    ap.add_argument("--field", default="elevation")
    ap.add_argument("--list", type=int, default=0)
    a = ap.parse_args()
    fields = Path(a.fields) if a.fields else Path(a.run) / "fields"
    print(json.dumps(census(fields, a.field, a.list)))


if __name__ == "__main__":
    main()
