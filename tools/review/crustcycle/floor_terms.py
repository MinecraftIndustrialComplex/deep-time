#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The ocean floor's height by age and thickness on cached pool runs (no simulation):

    floor_terms.py [--t T] <arm folder> [<arm folder> ...]

An arm folder is a pool run set (`.../pool/<key>/<preset>-<length>/<argkey>`); its runs supercontinent-1..4 and
scattered_continents-1..4 are read from the crust budget's maps (`maps/t<T>.bin`, the visible samples' records of
crust_budget::MAP_FIELDS; `maps/age/t<T>.bin`, the sea-floor ages; `maps/pools/t<T>.bin`, crust_budget::POOL_FIELDS),
T 0000 (the end) unless --t. Samples are counted as equal areas (the lattice's).

Per run and the arm's medians: the shares of the samples by crust (continental; oceanic of the ocean's own 7 km
column, under 7.2 km; oceanic 7.2 to 10 km; oceanic 10 km or more), each one's mean height z (the map's, m) and the
oceanic ones' mean age; the ocean's own column's mean height in floor-age bins and its share of the floor in each (so
a higher floor splits into a younger floor and a floor held up at the same age); the rock in oceanic samples beyond
the ocean's 2 km of mafic volcanic rock and 5 km of mafic plutonic rock (the igneous rock the tails, the heads and the
margins add, and arcs), as metres over the whole planet; and the hotspot-flagged share of the floor.
"""
import sys
from pathlib import Path

import numpy as np

NF = 11
NP = 9
HOTSPOT = 8
ARC = 4
RUNS = [f"{s}-{n}" for s in ("supercontinent", "scattered_continents") for n in (1, 2, 3, 4)]
AGE_BINS = [(0, 10), (10, 30), (30, 60), (60, 100), (100, 200), (200, 1e9)]


def one(run, t):
    m = np.fromfile(run / "maps" / f"t{t}.bin", "<f4").reshape(-1, NF).astype(np.float64)
    age = np.fromfile(run / "maps" / "age" / f"t{t}.bin", "<f4").astype(np.float64)
    pools = np.fromfile(run / "maps" / "pools" / f"t{t}.bin", "<f4").reshape(-1, NP).astype(np.float64)
    z, th, kind, flags = m[:, 2], m[:, 3], m[:, 4], m[:, 10].astype(np.int64)
    n = float(len(z))
    cont = kind > 0.5
    oc = ~cont
    classes = {"cont": cont, "oc7": oc & (th < 7200.0), "oc7_10": oc & (th >= 7200.0) & (th < 10000.0),
               "oc10": oc & (th >= 10000.0)}
    out = {}
    for k, sel in classes.items():
        out[k + "_share"] = float(sel.sum()) / n
        out[k + "_z"] = float(z[sel].mean()) if sel.any() else float("nan")
        if k != "cont":
            out[k + "_age"] = float(np.nanmean(age[sel])) if sel.any() else float("nan")
    o7 = classes["oc7"]
    for lo, hi in AGE_BINS:
        sel = o7 & (age >= lo) & (age < hi)
        tag = f"a{lo}"
        out[tag + "_share"] = float(sel.sum()) / max(float(o7.sum()), 1.0)
        out[tag + "_z"] = float(z[sel].mean()) if sel.any() else float("nan")
    extra_v = np.maximum(pools[:, 5] - 2000.0, 0.0)
    extra_p = np.maximum(pools[:, 6] - 5000.0, 0.0)
    out["oc_extra_volc_m"] = float(np.where(oc, extra_v, 0.0).sum()) / n
    out["oc_extra_plut_m"] = float(np.where(oc, extra_p, 0.0).sum()) / n
    out["oc_sediment_m"] = float(np.where(oc, pools[:, 0] + pools[:, 1], 0.0).sum()) / n
    out["oc_hotspot_share"] = float((oc & ((flags & HOTSPOT) != 0)).sum()) / max(float(oc.sum()), 1.0)
    out["oc_arc_share"] = float((oc & ((flags & ARC) != 0)).sum()) / max(float(oc.sum()), 1.0)
    return out


def fmt(v):
    if v != v:
        return "nan"
    a = abs(v)
    if a >= 100:
        return f"{v:.0f}"
    if a >= 1:
        return f"{v:.2f}"
    return f"{v:.4f}"


def main():
    args = sys.argv[1:]
    t = "0000"
    if args[:1] == ["--t"]:
        t, args = args[1], args[2:]
    for arm in args:
        arm = Path(arm)
        rows = [(r, one(arm / r, t)) for r in RUNS if (arm / r / "maps" / f"t{t}.bin").exists()]
        keys = list(rows[0][1])
        print(f"== {arm} t{t}")
        print("run " + " ".join(keys))
        for r, v in rows:
            print(r + " " + " ".join(fmt(v[k]) for k in keys))
        print("median " + " ".join(fmt(float(np.nanmedian([v[k] for _, v in rows]))) for k in keys))


if __name__ == "__main__":
    main()
