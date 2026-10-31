#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The volcano stage's cones handed to the plate pools, by what nets them (no simulation): cones.py <run> ...

Each <run> is a pool run folder (its `fields/globals.json`, the final world's globals, and `summary.txt`).
Prints per run and as the mean [lowest, highest] over the runs, in km^3 a year Earth-normalized over the run:

  handed      the cones' rock the plate model took into its pools (PLT-286: `plates/cone_felsic_km3` +
              `plates/cone_mafic_km3`), and of it the mafic
  arcs        the cones an arc counted inside its rate (`arc_cones_continental` + `arc_cones_oceanic`,
              PLT-761; runs from 2026-10-31 on, else nan)
  hotspots    the cones a hotspot counted inside its rate within its 150 km kernel (`hotspot_cones`);
              with the cone budget (PLT-803) not netted, so not counted below
  plateau     the flood basalts' plateau beyond the plate model's dome (the volcano stage lays th x
              (plateau - dome), VOL-035): with the sourced heads the LIP global's whole volume times
              0.486 (0.315 / 0.648, the plateau's and the dome's areas over pi R^2); else the stage's
              flood-basalt count times the preset's 1,500 m x 0.315 x pi x (900 km)^2 (the agent's
              arithmetic)
  by id       the cones the hotspots' tails netted by the cone's own hotspot (`hotspot_cones_by_id`, the
              cone budget, PLT-803; nan before). Runs of the budget before its review fixes (2026-10-31)
              count a continental hotspot's felsic cones within an arc's reach under arcs too, so
              unbudgeted reads low there by them (0.004 to 0.007 on hs-budget)
  unnetted    what dead tails' cones handed that no tail netted (`hotspot_cones_unnetted`, PLT-804; nan
              before the review fixes), inside unbudgeted
  vh built    the hotspots' cones at full, as the volcano stage built them (`volcanoes/hotspot_cones_
              built_km3`, PLT-804), and vh handed what they handed (the same without the budget)
  unbudgeted  handed less every counted part: rock no rate made
  on cont.    the cones handed to continental crust (`plates/cone_continental_km3`, PLT-803's
              measurement; nan before)
  own basalt  the hotspots' own basalt (`plates/hotspot_magma_km3`), for scale

The hotspot lane (2026-10-31) wrote it to measure the cones that count against no rate.
"""

import json
import math
import os
import sys

YEARS = 1.0e9  # the runs are 1000 Myr
PLATEAU_LESS_DOME = 0.315  # x pi R^2
PLATEAU = 0.648


def stage_count(summary, key):
    try:
        with open(summary) as f:
            for line in f:
                if "volcanoes_final=" in line:
                    for tok in line.split():
                        if tok.startswith(key + "="):
                            return float(tok.split("=", 1)[1])
    except OSError:
        pass
    return math.nan


def one(run):
    with open(os.path.join(run, "fields", "globals.json")) as f:
        g = json.load(f)
    g = g.get("globals", g)
    nan = math.nan

    def per(*keys):
        if not all(("plates/" + k) in g for k in keys):
            return nan
        return sum(g["plates/" + k] for k in keys) / YEARS

    handed = per("cone_felsic_km3", "cone_mafic_km3")
    mafic = per("cone_mafic_km3")
    arcs = per("arc_cones_continental_km3", "arc_cones_oceanic_km3")
    hot = per("hotspot_cones_km3")
    if "plates/lip_km3" in g:
        plateau = g["plates/lip_km3"] * PLATEAU_LESS_DOME / PLATEAU / YEARS
    else:
        n = stage_count(os.path.join(run, "summary.txt"), "flood_basalts")
        plateau = n * 1.5 * PLATEAU_LESS_DOME * math.pi * 900.0**2 / YEARS
    by_id = per("hotspot_cones_by_id_km3")
    netted_hot = by_id if not math.isnan(by_id) else hot
    counted = sum(x for x in (arcs, netted_hot, plateau) if not math.isnan(x))
    vol = lambda k: g.get("volcanoes/" + k, nan) / YEARS
    return {
        "handed": handed,
        "mafic": mafic,
        "arcs": arcs,
        "hotspots": hot,
        "plateau": plateau,
        "by id": by_id,
        "unnetted": per("hotspot_cones_unnetted_km3"),
        "unbudgeted": handed - counted if not math.isnan(arcs) else nan,
        "on cont.": per("cone_continental_km3"),
        "vh built": vol("hotspot_cones_built_km3"),
        "vh handed": vol("hotspot_cones_handed_km3"),
        "own basalt": per("hotspot_magma_km3"),
        "shields": stage_count(os.path.join(run, "summary.txt"), "shield_born"),
    }


def main():
    runs = sys.argv[1:]
    if not runs:
        sys.exit(__doc__)
    rows = [(os.path.basename(r.rstrip("/")), one(r)) for r in runs]
    keys = list(rows[0][1])
    print(f"{'run':24s} " + " ".join(f"{k:>10s}" for k in keys))
    for name, r in rows:
        print(f"{name:24s} " + " ".join(f"{r[k]:10.3f}" for k in keys))
    out = []
    for k in keys:
        xs = [r[k] for _, r in rows if not math.isnan(r[k])]
        out.append(f"{sum(xs) / len(xs):10.3f}" if xs else f"{'nan':>10s}")
    print(f"{'mean of ' + str(len(rows)):24s} " + " ".join(out))
    lo = []
    for k in keys:
        xs = [r[k] for _, r in rows if not math.isnan(r[k])]
        lo.append(f"{min(xs):4.2f}-{max(xs):<5.2f}" if xs else f"{'nan':>10s}")
    print(f"{'range':24s} " + " ".join(f"{x:>10s}" for x in lo))


if __name__ == "__main__":
    main()
