#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The sea's sediment against GlobSed and at the trenches (measurement only, cached pool runs):
seased.py <arm folder>[@N] ...

Subduction erosion (PLT-441, PLT-442: Clift & Vannucchi 2004) acts where the incoming plate's sediment is under 1 km,
and PLT-661 turns ocean floor beside a margin continental for the continents' rock it holds, so both read the sea's
sediment. GlobSed (Straume et al. 2019, Table 3 as bibliography.md quotes it): about 3.37 x 10^8 km3 over about 3.63 x
10^8 km2 (a mean of 927 m); on the margins 1.43 x 10^8 km3 over 4.69 x 10^7 km2; beyond 200 km of the continent-ocean
boundary 1.13 x 10^8 km3 over 2.79 x 10^8 km2. Per snapshot (TIMES), on the visible ocean floor: the sediment and
sedimentary rock (the pools file's first two), km3 (each sample the planet's area over the start's count) and mean
thickness, within 200 km of continental crust and beyond; the same cover on continental crust over an ocean basement
(the converted margins); and on ocean floor at a trench this step (the map's TRENCH flag, the subducting side), the
share with 1 km or more (accretionary by PLT-442's gate) and the median. Medians over runs."""
import os
import statistics
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
import thinning as th  # noqa: E402

R_KM = 6370.0
TIMES = [int(x) for x in os.environ.get("TIMES", "-900,-500,-100,0").split(",")]
TRENCH = 2


def run_stats(run):
    a0, _ = th.read(run, -1000)
    n0 = len(a0)
    cell = 4.0 * np.pi * R_KM**2 / n0
    out = {}
    for t in TIMES:
        try:
            a, pp = th.read(run, t)
        except FileNotFoundError:
            continue
        cont = a[:, 4] > 0.5
        ocean = ~cont
        cover_km = pp[:, 0:2].sum(1) / 1000.0
        p = th.xyz(a[:, 0], a[:, 1])
        r = {}
        if cont.any():
            d, _ = cKDTree(p[cont]).query(p[ocean])
            d_km = 2.0 * np.arcsin(np.clip(d / 2.0, 0.0, 1.0)) * R_KM
            near = np.zeros(len(a), bool)
            near[np.flatnonzero(ocean)[d_km <= 200.0]] = True
        else:
            near = np.zeros(len(a), bool)
        far = ocean & ~near
        r["sea_sed_km3"] = float(cover_km[ocean].sum() * cell)
        r["sea_mean_m"] = float(cover_km[ocean].mean() * 1000.0)
        r["near_sed_km3"] = float(cover_km[near].sum() * cell)
        r["near_mean_m"] = float(cover_km[near].mean() * 1000.0) if near.any() else float("nan")
        r["near_area_km2"] = float(near.sum() * cell)
        r["far_sed_km3"] = float(cover_km[far].sum() * cell)
        r["far_mean_m"] = float(cover_km[far].mean() * 1000.0) if far.any() else float("nan")
        basement = np.clip(np.minimum(pp[:, 7], pp[:, 5] + pp[:, 6]), 0.0, None)
        conv = cont & (basement > 1000.0)
        r["conv_cover_km3"] = float(cover_km[conv].sum() * cell)
        tr = ocean & ((a[:, 10].astype(np.int64) & TRENCH) != 0)
        r["trench_samples"] = int(tr.sum())
        if tr.any():
            r["trench_ge1km"] = float((cover_km[tr] >= 1.0).mean())
            r["trench_p50_m"] = float(np.median(cover_km[tr]) * 1000.0)
            r["trench_near_ge1km"] = float((cover_km[tr & near] >= 1.0).mean()) if (tr & near).any() else float("nan")
            r["trench_far_ge1km"] = float((cover_km[tr & far] >= 1.0).mean()) if (tr & far).any() else float("nan")
        out[t] = r
    return out


def main(args):
    for arg in args:
        runs = th.runs_of(arg)
        stats = [run_stats(r) for r in runs]
        print(f"== {arg} ({len(runs)} runs), medians")
        for t in TIMES:
            rows = [s[t] for s in stats if t in s]
            if not rows:
                continue
            cells = []
            for k in rows[0]:
                xs = [r[k] for r in rows if k in r and np.isfinite(r[k])]
                if xs:
                    cells.append(f"{k} {statistics.median(xs):.4g}")
            print(f"  t{t}: " + "  ".join(cells))


if __name__ == "__main__":
    main(sys.argv[1:])
