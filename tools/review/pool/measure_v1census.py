#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `v1census` measure: V1 item 1's seven numbers (decisions log 2026-10-21) and the two
the plates.md table adds, from one pool run folder, printed as a JSON object on stdout.

    python3 -I measure_v1census.py <run folder> [--fields DIR] [--plate FILE] [--rec FILE]

A run folder holds `fields/` (deeptime dump-fields of the final checkpoint, the L1 grid), `plate.json`
(deeptime plate-check --json) and `rec.json` (plate-check --record). The census is
tools/review/continents_census.py's `fields` report, read as numbers (its helpers, not a copy):

  land         cells above the sea level (elevation block 63) over the planet's cells
  largest      the largest landmass (8-connected, joined over the x seam), a share of the planet
  under_sea    continental crust under the sea, a share of the continental crust
  above_2km    land above 2 km, a share of the land
  coast_hi     coast cells (land within 1.6 point spacings of the sea) with land above 2 km within 200 km
               inland, a share of the coast. Version 2 (RLC-052, M5 diagnosis 2026-10-25): measured on
               Fibonacci points of the grid's count (nearest cell; the grid's north-south step is wider
               than the coast band above about 51 degrees), the sea is the world ocean (a closed body of
               sea is land), and the 200 km are over land (a strait wider than 1.6 spacings blocks them).
               coast_hi_old, coast_hi_fib and coast_hi_ocean are the version-1 rule on the grid's cells,
               on the Fibonacci points, and on the Fibonacci points with the closed bodies as land
               (what each fix does); closed_sea is the closed bodies' share of the sea's area
  under_ice    land with 1 m of glacier ice or more, a share of the land
  collisions   continental collisions per 100 Myr (the plate check's collisions_per_100myr)
  crust_net    continental crust's net change over the run, a share of the planet (the record's last step
               less its first: `continental` / `visible`)
  sea_after100_m, sea_end_m
               the sea over the datum: the median of the record's sea_m after the first 100 Myr, and the
               last step's, metres

Needs numpy and scipy (the pool resolves them with nix). Version 2: a change to these definitions
bumps `version` in measures.json, and the pool recomputes the measure over the cached folders.
"""
import argparse
import contextlib
import io
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

import numpy as np  # noqa: E402

import continents_census as cc  # noqa: E402


def census(fields_dir):
    f, _globals, e = cc.load_dump(fields_dir)
    h, w, r = e["h"], e["w"], e["r"]
    lat_r, ok_r = cc.lat_of_rows(h, r, w * r)
    lat = np.repeat(lat_r[:, None], w, 1)
    ok = np.repeat(ok_r[:, None], w, 1)
    lon = np.repeat((360.0 * np.arange(w) / w)[None, :], h, 0)
    ncell = int(ok.sum())
    cell_km2 = cc.EARTH_KM2 / ncell
    z = (f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V
    land = ok & (z > 0)
    cont = ok & (f["plates/crust_type"] > 0.5)
    ice = np.nan_to_num(f.get("climate/ice_thickness_m", np.zeros_like(z)))
    wet_cont = cont & ~land
    lab = cc.label_wrap(land)
    sizes = np.bincount(lab.ravel())
    sizes[0] = 0
    sp = np.sqrt(cell_km2)
    frow, fcol, fp = cc.fib_sample(h, w, r)
    zf = z[frow, fcol]
    with contextlib.redirect_stdout(io.StringIO()):  # coast_metrics also prints a line of its own
        coast = cc.coast_metrics(fp, zf, sp)
        old = cc.coast_metrics(cc.xyz(lat[ok], lon[ok]), z[ok], sp, closed_as_land=False, over_land=False)
        fib_old = cc.coast_metrics(fp, zf, sp, closed_as_land=False, over_land=False)
        fib_ocean = cc.coast_metrics(fp, zf, sp, over_land=False)
    sea_f = zf <= 0
    closed_sea = float((sea_f & ~cc.world_ocean(fp, sea_f, sp)).sum() / max(int(sea_f.sum()), 1))
    n_land = max(int(land.sum()), 1)
    return {
        "land": land.sum() / ncell,
        "largest": float(sizes.max()) / ncell,
        "under_sea": wet_cont.sum() / max(int(cont.sum()), 1),
        "above_2km": float((z[land] > 2000).mean()) if land.any() else 0.0,
        "coast_hi": float(coast["coast_with_hi"]),
        "coast_hi_old": float(old["coast_with_hi"]),
        "coast_hi_fib": float(fib_old["coast_with_hi"]),
        "coast_hi_ocean": float(fib_ocean["coast_with_hi"]),
        "closed_sea": closed_sea,
        "under_ice": float((land & (ice >= cc.ICE_MIN_M)).sum()) / n_land,
        "cont": cont.sum() / ncell,
        "cells": ncell,
    }


def from_plate(path):
    for m in json.loads(Path(path).read_text())["measures"]:
        if m["key"] == "collisions_per_100myr":
            return m["value"]
    return None


def from_record(path):
    rec = json.loads(Path(path).read_text())
    steps = rec["steps"]
    out = {"crust_net": None, "sea_after100_m": None, "sea_end_m": None}
    if not steps:
        return out
    out["crust_net"] = (steps[-1]["continental"] - steps[0]["continental"]) / steps[0]["visible"]
    t0 = rec.get("t0_myr", steps[0]["t"])
    late = [s["sea_m"] for s in steps if s["t"] >= t0 + 100.0 and s.get("sea_m") is not None]
    if late:
        out["sea_after100_m"] = float(np.median(late))
    out["sea_end_m"] = steps[-1].get("sea_m")
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("run")
    ap.add_argument("--fields")
    ap.add_argument("--plate")
    ap.add_argument("--rec")
    a = ap.parse_args()
    run = Path(a.run)
    values = census(a.fields or run / "fields")
    values["collisions"] = from_plate(a.plate or run / "plate.json")
    values.update(from_record(a.rec or run / "rec.json"))
    print(json.dumps({k: (None if v is None else float(v)) for k, v in values.items()}, sort_keys=True))


if __name__ == "__main__":
    main()
