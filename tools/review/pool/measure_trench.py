#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `trench` measure (the trench budget, 2026-10-22): the deep sea floor and the losses at
subduction zones, from one pool run folder, printed as a JSON object on stdout.

    python3 -I measure_trench.py <run folder> [--fields DIR] [--rec FILE]

The deep floor, on the final frame's L1 grid (rows uniform in sin(latitude), so every planet cell has
the same area):

  deep6, deep8, deep10   the share of the sea floor (cells below the sea) deeper than 6, 8, 10 km
  deepest_m              the deepest cell, m below the sea
  p50_m, p99_m           the sea floor's median and 99th-percentile depth, m
  plate_deep6, plate_deepest_m, surface_deep6, surface_deepest_m
                         the same two from the plate model's elevation (`plates/elevation_m`) and the
                         surface stage's (`surface/elevation_m`), both over the datum, against the sea
                         over the datum (`climate/sea_level_m`): which stage digs the deep floor

Earth (tools/review/earth-deep-floor.py on SRTM15+ at 15'; ledger HYP-050, HYP-051): deep6 0.0073
(0.0076 on 100,000 Fibonacci points; Harris et al. 2014 Table 5's hadal floor, SRTM30_PLUS: 0.95 %),
deep8 0.0003, deep10 0, deepest 9,242 m at 15' (Challenger Deep 10,920 m, Stern 2021 Table 1), median
4,098 m, p99 5,953 m.

The losses at subduction zones, km³/yr over the run (the world globals in `fields/globals.json`
accumulate from the run's start; the length is the record's):

  sed_subducted   sediment subduction at resamplings (PLT-440; Stern & Scholl 2010 Table 1: 1.1 at
                  ocean margins and 0.1 at suturing zones)
  forearc_eroded  subduction erosion, continental and other crust (PLT-441; Table 1: 1.4 and 0.3)
  erosive_km      the erosive fronts' mean length, km (Clift & Vannucchi 2004's tables: 22,925 km of
                  their 40,225; 57 % of the margins)
  front_km        the oceanic subduction fronts' mean length, km (Stern & Scholl 2010: 43,000 km of
                  convergent margin)
  arcs            arc crust added, continental and island arcs (Table 1: 2.5)
  accreted        the conserved crust a resampling accreted to the column on top (no Earth value: the
                  model's own sediment's and slab's crust that did not go down)

Needs numpy (the pool resolves it with nix). Version 1.
"""
import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent))

import numpy as np  # noqa: E402

import continents_census as cc  # noqa: E402

KM = (6000.0, 8000.0, 10000.0)


def floor(depth):
    """depth: m below the sea of the sea-floor cells."""
    out = {f"deep{int(t / 1000)}": float((depth > t).mean()) if depth.size else 0.0 for t in KM}
    out["deepest_m"] = float(depth.max()) if depth.size else 0.0
    p = np.percentile(depth, [50, 99]) if depth.size else [0.0, 0.0]
    out["p50_m"], out["p99_m"] = float(p[0]), float(p[1])
    return out


def deep_floor(fields_dir):
    f, g, e = cc.load_dump(fields_dir)
    h, w, r = e["h"], e["w"], e["r"]
    _lat, ok_r = cc.lat_of_rows(h, r, w * r)
    ok = np.repeat(ok_r[:, None], w, 1)
    z = (f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V
    sea = ok & (z < 0)
    out = floor(-z[sea])
    sea_m = g["globals"].get("climate/sea_level_m")
    for tag, name in (("plate", "plates/elevation_m"), ("surface", "surface/elevation_m")):
        if name in f and sea_m is not None:
            d = sea_m - f[name][ok]
            d = d[np.isfinite(d) & (d > 0)]
            s = floor(d)
            out[f"{tag}_deep6"] = s["deep6"]
            out[f"{tag}_deepest_m"] = s["deepest_m"]
    return out, g["globals"]


def budget(globals_, rec_path):
    rec = json.loads(Path(rec_path).read_text())
    steps = rec["steps"]
    myr = steps[-1]["t"] - rec.get("t0_myr", steps[0]["t"]) if steps else 0.0
    if myr <= 0:
        return {}
    yr = myr * 1.0e6
    g = lambda k: globals_.get(k, 0.0)  # noqa: E731
    return {
        "sed_subducted": g("plates/sediment_subducted_km3") / yr,
        "forearc_eroded": (g("plates/forearc_eroded_continental_km3") + g("plates/forearc_eroded_other_km3")) / yr,
        "forearc_continental": g("plates/forearc_eroded_continental_km3") / yr,
        "forearc_short": g("plates/forearc_short_km3") / yr,
        "erosive_km": (g("plates/forearc_front_continental_km_myr") + g("plates/forearc_front_oceanic_km_myr")) / myr,
        "front_km": (g("plates/arc_front_continental_km_myr") + g("plates/arc_front_oceanic_km_myr")) / myr,
        "arcs": (g("plates/arc_continental_km3") + g("plates/arc_oceanic_km3")) / yr,
        "accreted": g("plates/resample_accreted_km3") / yr,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("run")
    ap.add_argument("--fields")
    ap.add_argument("--rec")
    a = ap.parse_args()
    run = Path(a.run)
    values, globals_ = deep_floor(a.fields or run / "fields")
    values.update(budget(globals_, a.rec or run / "rec.json"))
    print(json.dumps({k: (None if v is None else float(v)) for k, v in values.items()}, sort_keys=True))


if __name__ == "__main__":
    main()
