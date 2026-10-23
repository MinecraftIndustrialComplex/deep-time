#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `fill` measure: how fast the surface's own sediment fills the closed basins of the
final L1 surface (the release's pace, ISO-099, 2026-10-23), from one pool run folder, printed as a
JSON object on stdout.

    python3 -I measure_fill.py <run folder> [--fields DIR] [--list N]

The basins are those of the `basins` measure (measure_basins.py: the hydrology's open sea, a
priority flood, closed bodies the flood raised by half a block or more) deeper than 10 blocks. In
them it reads the surface stage's last step: `surface/deposition_m_myr` (net deposition, m/Myr)
and `surface/sediment_m` (the strata log's thickness, m).

  fill_dep_med_m_myr   median over the basins of their mean deposition rate, m/Myr
  fill_dep_km3_myr     the sediment the basins take a Myr, all of them, km3
  fill_room_km3        their room (as the basins measure's closed_volume_km3), km3
  fill_myr             room / sediment a Myr: the time the deposition of the last step would
                       take to fill them all, Myr
  fill_myr_med         the median basin's own fill time, Myr (a basin with no deposition: 1e6)
  fill_fast_share      of the basins, the share that would fill within 10 Myr
  fill_sed_med_m       median over the basins of their mean sediment thickness, m

`--list N` prints the N deepest basins with their deposition and fill time before the JSON line.
Needs numpy and scipy. Version 1.
"""
import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(HERE.parent))

import numpy as np  # noqa: E402

import continents_census as cc  # noqa: E402
import measure_basins as mb  # noqa: E402

FAST_MYR = 10.0
NEVER_MYR = 1e6


def fill(fields_dir, top=0):
    a, r = mb.load(fields_dir, "elevation")
    dep, rd = mb.load(fields_dir, "surface/deposition_m_myr")
    sed, rs = mb.load(fields_dir, "surface/sediment_m")
    if dep is None or sed is None or rd != r or rs != r:
        raise SystemExit("the dump lacks surface/deposition_m_myr or surface/sediment_m on the elevation's grid")
    h, w = a.shape
    _lat, ok_r = cc.lat_of_rows(h, r, w * r)
    ok = np.repeat(ok_r[:, None], w, 1) & np.isfinite(a)
    z = np.where(ok, a - cc.SEA_Y, np.nan)
    cell_km2 = (r * mb.M_PER_BLOCK_H / 1000.0) ** 2
    min_sea = max(1, int(np.ceil(mb.OPEN_SEA_KM2 / cell_km2)))
    deep = ok & (z <= -1.0)
    lab = mb.label_wrap(deep)
    sizes = np.bincount(lab.ravel())
    sizes[0] = 0
    sea = deep & (sizes[lab] >= min_sea)
    filled, reached = mb.flood(np.nan_to_num(z, nan=1e9), sea, ok)
    depth = np.where(ok & reached, filled - np.nan_to_num(z), 0.0)
    basin = ok & ~sea & (depth >= 0.5)
    blab = mb.label_wrap(basin)
    dep = np.nan_to_num(dep)
    sed = np.nan_to_num(sed)
    rows = []
    for b in np.unique(blab[basin]):
        m = blab == b
        d = float(depth[m].max())
        if d <= 10.0:
            continue
        room = float(depth[m].sum()) * cc.M_PER_BLOCK_V / 1000.0 * cell_km2
        rate = float(np.maximum(dep[m], 0.0).sum()) / 1000.0 * cell_km2
        k = int(np.argmax(depth[m]))
        jj, ii = np.nonzero(m)
        rows.append({"depth": d, "cells": int(m.sum()), "room": room, "rate": rate,
                     "dep": float(dep[m].mean()), "sed": float(sed[m].mean()),
                     "myr": room / rate if rate > 0 else NEVER_MYR, "at": [int(ii[k]), int(jj[k])]})
    rows.sort(key=lambda x: -x["depth"])
    for x in rows[:top]:
        print("basin depth %6.1f b cells %4d room %8.0f km3 deposition %7.1f m/Myr (%7.2f km3/Myr) fill %9.1f Myr sediment %6.0f m floor cell %s" % (
            x["depth"], x["cells"], x["room"], x["dep"], x["rate"], x["myr"], x["sed"], x["at"]))
    room = sum(x["room"] for x in rows)
    rate = sum(x["rate"] for x in rows)
    med = (lambda v: float(np.median(v)) if v else 0.0)
    return {
        "fill_dep_med_m_myr": med([x["dep"] for x in rows]),
        "fill_dep_km3_myr": rate,
        "fill_room_km3": room,
        "fill_myr": room / rate if rate > 0 else NEVER_MYR,
        "fill_myr_med": med([x["myr"] for x in rows]),
        "fill_fast_share": (sum(1 for x in rows if x["myr"] <= FAST_MYR) / len(rows)) if rows else 0.0,
        "fill_sed_med_m": med([x["sed"] for x in rows]),
    }


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("run")
    ap.add_argument("--fields")
    ap.add_argument("--list", type=int, default=0)
    a = ap.parse_args()
    fields = Path(a.fields) if a.fields else Path(a.run) / "fields"
    print(json.dumps(fill(fields, a.list)))


if __name__ == "__main__":
    main()
