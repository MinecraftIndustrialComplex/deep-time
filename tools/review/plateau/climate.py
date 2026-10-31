# SPDX-License-Identifier: LGPL-3.0-or-later
"""The plateau lane: the high ground's climate at a run's end, as the surface's discharge reads it (VM, cached runs).

Usage: python3 -I climate.py <arm folder> [<arm folder> ...]

The surface stage's runoff (`surface/climate.rs`, ERO-010) is P - Fu's ET with PET = 60 mm/degC x the annual mean
temperature, 0 below 0 degC: on ground whose mean is below freezing nothing evaporates and all of P runs off and
counts in the stream power's drainage area. The bake's water budget (`hydro::HydroParams::budget`, HYD-029 to HYD-037)
takes PET from the biotemperature T+ (the year's mean of the temperature above freezing, a sinusoidal year between the
cold and warm seasons) and lets only melted snow run off. From the dumped final fields (`surface/elevation_m`,
`climate/t_mean_c`, `climate/t_warm_c`, `climate/koppen`; the cold season taken as 2 t_mean - t_warm, the sinusoid's
other extreme): for the land above 2 km and its flat part (3 x 3 relief under 500 m), the share below 0 degC, the
medians of t_mean, T+ and the thawed share of the year, and the Koppen arid (B) and polar (E) shares. Medians over runs.
Precipitation is not in the dump, so the runoff itself is not computed here.
"""
import json
import math
import statistics
import sys
from pathlib import Path

import numpy as np


def field(f, idx, name):
    e = idx[name]
    return np.fromfile(f / e["file"], "<f4").reshape(e["h"], e["w"]).astype(np.float64)


def thaw(t_mean, t_warm):
    """Biotemperature and thawed share of a sinusoidal year (hydro::Climate::thaw)."""
    t_cold = 2.0 * t_mean - t_warm
    a = 0.5 * np.maximum(t_warm - t_cold, 0.0)
    m = 0.5 * (t_warm + t_cold)
    tp = np.where(m - a >= 0, np.maximum(t_mean, 0.0), 0.0)
    sh = np.where(m - a >= 0, 1.0, 0.0)
    mid = (m - a < 0) & (m + a > 0)
    with np.errstate(invalid="ignore", divide="ignore"):
        th = np.arccos(np.clip(-m / np.where(a > 0, a, 1.0), -1.0, 1.0))
    tp = np.where(mid, np.maximum((m * th + a * np.sin(th)) / math.pi, 0.0), tp)
    sh = np.where(mid, th / math.pi, sh)
    return tp, sh


def run(r):
    f = r / "fields"
    idx = json.load(open(f / "index.json"))
    g = json.load(open(f / "globals.json"))
    g = g.get("globals", g)
    sea = float(g.get("climate/sea_level_m", 0.0))
    name = "surface/elevation_m" if "surface/elevation_m" in idx else "elevation"
    z = field(f, idx, name) - sea
    tm = field(f, idx, "climate/t_mean_c")
    tw = field(f, idx, "climate/t_warm_c")
    k = field(f, idx, "climate/koppen").round().astype(int)
    hi = z > 2000.0
    pad = np.pad(z, 1, mode="edge")
    win = np.stack([pad[1 + dy:pad.shape[0] - 1 + dy, 1 + dx:pad.shape[1] - 1 + dx]
                    for dy in (-1, 0, 1) for dx in (-1, 0, 1)])
    flat = hi & ((win.max(0) - win.min(0)) < 500.0)
    tp, sh = thaw(tm, tw)
    out = {}
    for label, msk in (("hi", hi), ("flat", flat)):
        n = int(msk.sum())
        if n == 0:
            continue
        out[label + "_n"] = n
        out[label + "_below0"] = float((tm[msk] < 0).mean())
        out[label + "_t_mean"] = float(np.median(tm[msk]))
        out[label + "_t_plus"] = float(np.median(tp[msk]))
        out[label + "_thawed_share"] = float(np.median(sh[msk]))
        out[label + "_arid"] = float(np.isin(k[msk], [4, 5, 6, 7]).mean())
        out[label + "_polar"] = float(np.isin(k[msk], [29, 30]).mean())
        cold = msk & (tm < 0)
        if cold.any():
            # Below 0 degC the surface's PET is 0; the bake's is 60 x T+ (mm/yr) on the melted share.
            out[label + "_below0_pet_hydro_mm"] = float(np.median(60.0 * tp[cold]))
            out[label + "_below0_melt_cap_mm"] = float(np.median(4.0 * 365.25 * tp[cold]))
    return out


def main():
    for arm in sys.argv[1:]:
        rows = [run(r) for r in sorted(Path(arm).iterdir()) if (r / "fields" / "index.json").exists()]
        print(f"== climate {arm} ({len(rows)} runs)")
        for key in sorted({k for r in rows for k in r}):
            v = [r[key] for r in rows if key in r]
            print(f"  median {key:28s} {statistics.median(v):9.3f}  (n {len(v)}, min {min(v):.3f}, max {max(v):.3f})")


if __name__ == "__main__":
    main()
