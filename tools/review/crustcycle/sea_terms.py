#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The sea's terms and the crust's additions by kind on cached pool runs (no simulation):

    sea_terms.py <arm folder> [<arm folder> ...]

An arm folder is a pool run set (`.../pool/<key>/<preset>-<length>/<argkey>`); its runs supercontinent-1..4 and
scattered_continents-1..4 are read. The sea is the level a fixed layer of water fills over the ground
(`sea_level::fill_level`), so on the final frame sea = W / A_wet + mean_wet(h): the water over the wet area plus the
wet ground's mean height. That mean splits by the crust under the water: continental, ocean crust of the ocean's own
thickness (7 km, under 10 km here) and thick ocean crust (10 km or more: plateaus, seamount chains, island arcs,
stacked crust). Per run: the sea at the end (rec.json's last `sea_m`), the water as a global layer (a check: it should
be the planet's surface_water_m less its ice), the wet share, each crust's share of the wet area and its mean height
under the sea, the thick ocean crust's rock beyond 7 km as a global layer, and the globals of the crust's additions by
kind (Earth-normalized km^3 over the run). The arm's medians close each block.

From the final frame's dump (fields/: elevation in blocks over the sea, 35 m a block, sea at y 63; plates/crust_type,
plates/thickness_m), rec.json and fields/globals.json. Needs numpy (the VM's .pyenv-numpy, run with -I).
"""
import json
import sys
from pathlib import Path

import numpy as np

M_PER_BLOCK_V = 35.0
SEA_Y = 63.0
OCEAN_M = 7000.0
THICK_M = 10000.0
RUNS = [f"{s}-{n}" for s in ("supercontinent", "scattered_continents") for n in (1, 2, 3, 4)]
GLOBALS = [
    ("cont_vol", "plates/continental_volume_km3"),
    ("tail_magma", "plates/hotspot_magma_km3"),
    ("tail_magma_c", "plates/hotspot_magma_continental_km3"),
    ("tail_rock", "plates/hotspot_tail_rock_km3"),
    ("lip", "plates/lip_km3"),
    ("lip_c", "plates/lip_continental_km3"),
    ("lip_dome_c", "plates/lip_dome_continental_km3"),
    ("lips", "plates/lips"),
    ("vmargin", "plates/volcanic_margin_km3"),
    ("vmargin_n", "plates/volcanic_margin_samples"),
    ("cones_built", "volcanoes/hotspot_cones_built_km3"),
    ("accreted", "plates/resample_accreted_km3"),
    ("accreted_c", "plates/resample_accreted_continent_km3"),
    ("arc_oc", "plates/arc_oceanic_km3"),
    ("arc_c", "plates/arc_continental_km3"),
]


def rows_ok(h, r, w):
    big_r = w * r / (2 * np.pi)
    z = -h * r / 2 + r * np.arange(h)
    return np.abs(z) <= big_r


def one(run):
    d = run / "fields"
    idx = json.loads((d / "index.json").read_text())

    def get(name):
        e = idx[name]
        return np.fromfile(d / e["file"], "<f4").reshape(e["h"], e["w"]).astype(np.float64)

    e = idx["elevation"]
    ok = np.repeat(rows_ok(e["h"], e["r"], e["w"])[:, None], e["w"], 1)
    sea = float(json.loads((run / "rec.json").read_text())["steps"][-1]["sea_m"])
    z = (get("elevation") - SEA_Y) * M_PER_BLOCK_V  # height over the sea, m
    h = z + sea  # over the datum
    kind = get("plates/crust_type")
    thick = get("plates/thickness_m")
    cont = ok & (kind > 0.5)
    ocean = ok & ~(kind > 0.5)
    wet = ok & (z < 0.0)
    n = float(ok.sum())
    nwet = float(wet.sum())
    out = {"sea": sea, "water_m": float(np.where(wet, -z, 0.0).sum()) / n, "wet": nwet / n,
           "cont_area": float(cont.sum()) / n, "land": float((ok & (z >= 0.0)).sum()) / n}
    parts = {"wet_cont": wet & cont, "wet_ocean7": wet & ocean & (thick < THICK_M),
             "wet_thick": wet & ocean & (thick >= THICK_M)}
    for k, m in parts.items():
        out[k + "_share"] = float(m.sum()) / nwet
        out[k + "_h"] = float(h[m].mean()) if m.any() else float("nan")
    out["mean_wet_h"] = float(h[wet].mean())
    out["thick_area"] = float((ocean & (thick >= THICK_M)).sum()) / n
    out["thick_excess_m"] = float(np.where(ocean, np.maximum(thick - OCEAN_M, 0.0), 0.0).sum()) / n
    out["thick_excess15_m"] = float(np.where(ocean & (thick >= 15000.0), thick - OCEAN_M, 0.0).sum()) / n
    g = json.loads((d / "globals.json").read_text())
    g = g.get("globals", g)
    for k, name in GLOBALS:
        out[k] = float(g.get(name, float("nan")))
    return out


def fmt(v):
    if v != v:
        return "nan"
    a = abs(v)
    if a >= 1e5:
        return f"{v:.3e}"
    if a >= 100:
        return f"{v:.0f}"
    if a >= 1:
        return f"{v:.2f}"
    return f"{v:.4f}"


def main():
    for arm in sys.argv[1:]:
        arm = Path(arm)
        rows = [(r, one(arm / r)) for r in RUNS if (arm / r / "fields").exists()]
        keys = list(rows[0][1])
        print(f"== {arm}")
        print("run " + " ".join(keys))
        for r, v in rows:
            print(r + " " + " ".join(fmt(v[k]) for k in keys))
        print("median " + " ".join(fmt(float(np.median([v[k] for _, v in rows]))) for k in keys))


if __name__ == "__main__":
    main()
