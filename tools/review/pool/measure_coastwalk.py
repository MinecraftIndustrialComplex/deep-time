#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `coastwalk` measure (V1 item 1, the docking lane, 2026-10-23): what stands behind a
high coast, from one pool run folder's final fields, printed as a JSON object on stdout.

    python3 -I measure_coastwalk.py <run folder> [--fields DIR]

A coast cell, a high coast and a collision high coast are `measure_collcoast.py`'s (coastset's coast:
land within 1.6 spacings of the sea; high: land above 2 km within 200 km on the same landmass; a
collision high coast has a collision-regime cell within 150 km). From up to 600 cells of each group
(a fixed draw), a walk goes inland along the great circle away from the cell's nearest sea cell in
25 km steps to 2,000 km: the land across is the distance to the first sea cell, the crust behind is
the distance to the first oceanic-crust cell (ledger PLT-513).

Is a collision high coast the back of a narrow strip with sea on both sides (an arc or sliver
standing in the ocean), or the edge of a wide continent? Earth's coasts, by the same walk on
the GMT earth_relief grid at 15 arc-minutes on a 332,800-point Fibonacci sampling (a check of
2026-10-23, not a pool number; ice-covered rock counts as land): 600 km median across and 0.51 of
them 600 km or less (675 km and 0.49 below 60 degrees of latitude).

  chc_share              collision high coasts, share of the high coasts
  hc_other               the other high coasts, share of all coast (v1census's coast_hi with the
                         collision high coasts left out)
  chc_across_km, all_across_km      median land across, km
  chc_across_le600, all_across_le600   share with 600 km or less across
  chc_to_oceanic_km      median distance inland to oceanic crust (2,025 when none in 2,000 km)
  chc_faces_oceanic, all_faces_oceanic   the facing sea cell is oceanic crust
  chc_faces_depth_m      the facing sea cell's median depth, m (negative)
  chc_trench300, all_trench300      a trench or arc regime cell within 300 km

Version 1.
"""
import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

import numpy as np  # noqa: E402
from scipy.spatial import cKDTree  # noqa: E402

import continents_census as cc  # noqa: E402

STEP_KM = 25.0
MAX_KM = 2000.0
DRAW = 600
R_KM = 6371.0


def walk(p, tree, i, j, land, oce):
    c0 = p[i]
    s = p[j] - c0
    t = s - (s @ c0) * c0
    n = np.linalg.norm(t)
    if n < 1e-12:
        return MAX_KM + STEP_KM, MAX_KM + STEP_KM
    t = -t / n
    d_sea = d_oce = None
    for k in range(1, int(MAX_KM / STEP_KM) + 1):
        a = k * STEP_KM / R_KM
        _d, m = tree.query(c0 * np.cos(a) + t * np.sin(a))
        if d_sea is None and not land[m]:
            d_sea = k * STEP_KM
        if d_oce is None and oce[m]:
            d_oce = k * STEP_KM
        if d_sea is not None and d_oce is not None:
            break
    return (d_sea if d_sea is not None else MAX_KM + STEP_KM), (d_oce if d_oce is not None else MAX_KM + STEP_KM)


def from_fields(fields):
    f, _g, e = cc.load_dump(Path(fields))
    h, w, r = e["h"], e["w"], e["r"]
    lat_r, ok_r = cc.lat_of_rows(h, r, w * r)
    lat = np.repeat(lat_r[:, None], w, 1)
    ok = np.repeat(ok_r[:, None], w, 1)
    lon = np.repeat((360.0 * np.arange(w) / w)[None, :], h, 0)
    spacing = np.sqrt(cc.EARTH_KM2 / int(ok.sum()))
    p = cc.xyz(lat[ok], lon[ok])
    z = ((f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V)[ok]
    reg = np.rint(np.nan_to_num(f["regime"][ok], nan=0.0)).astype(int)
    ct = np.rint(np.nan_to_num(f["plates/crust_type"][ok], nan=0.0)).astype(int)
    land = z > 0
    oce = ct == 0
    tree = cKDTree(p)
    sea_idx = np.nonzero(~land)[0]
    ds, js = cKDTree(p[~land]).query(p[land])
    d_sea = np.zeros(z.size)
    d_sea[land] = cc.arc_km(ds)
    near_sea = np.full(z.size, -1)
    near_sea[land] = sea_idx[js]
    coast = land & (d_sea <= 1.6 * spacing)
    hi = land & (z > 2000)
    comp = cc.components(p, land, spacing)
    has = np.zeros(z.size, bool)
    for c in np.unique(comp[hi]):
        hm, cm = np.nonzero(hi & (comp == c))[0], np.nonzero(coast & (comp == c))[0]
        if cm.size == 0:
            continue
        d, _j = cKDTree(p[hm]).query(p[cm], distance_upper_bound=cc.chord(200.0))
        has[cm[np.isfinite(d)]] = True
    coll = reg == 5  # regime::COLLISION
    trench = (reg == 2) | (reg == 3)  # regime::TRENCH, regime::ARC
    hc = np.nonzero(coast & has)[0]
    is_coll = np.zeros(z.size, bool)
    if coll.any() and hc.size:
        dd, _ = cKDTree(p[coll]).query(p[hc], distance_upper_bound=cc.chord(150.0))
        is_coll[hc[np.isfinite(dd)]] = True
    n_coast = max(int(coast.sum()), 1)
    out = {
        "chc_share": float(is_coll[hc].mean()) if hc.size else None,
        "hc_other": float((~is_coll[hc]).sum() / n_coast),
    }
    ttree = cKDTree(p[trench]) if trench.any() else None
    rng = np.random.default_rng(1)
    for name, idx in (("chc", hc[is_coll[hc]]), ("all", np.nonzero(coast)[0])):
        if idx.size == 0:
            continue
        if idx.size > DRAW:
            idx = rng.choice(idx, DRAW, replace=False)
        ws, wo = np.array([walk(p, tree, i, near_sea[i], land, oce) for i in idx]).T
        out[f"{name}_across_km"] = float(np.median(ws))
        out[f"{name}_across_le600"] = float((ws <= 600.0).mean())
        out[f"{name}_faces_oceanic"] = float(oce[near_sea[idx]].mean())
        if ttree is not None:
            dt, _ = ttree.query(p[idx], distance_upper_bound=cc.chord(300.0))
            out[f"{name}_trench300"] = float(np.isfinite(dt).mean())
        if name == "chc":
            out["chc_to_oceanic_km"] = float(np.median(wo))
            out["chc_faces_depth_m"] = float(np.median(z[near_sea[idx]]))
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("run")
    ap.add_argument("--fields")
    a = ap.parse_args()
    values = from_fields(a.fields or Path(a.run) / "fields")
    print(json.dumps({k: (None if v is None else float(v)) for k, v in values.items()}, sort_keys=True))


if __name__ == "__main__":
    main()
