#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `coastset` measure: which plate setting the coast with high ground sits in (gap 4's
diagnosis, docs/science/lithosphere-thermal-state.md "Where the high coasts are"), from one pool run
folder's final fields, printed as a JSON object on stdout.

    python3 -I measure_coastset.py <run folder> [--fields DIR] [--radius-km 150]

The coast and the high ground are v1census's `coast_hi` (continents_census.coast_metrics): coast cells
are land within 1.6 cell spacings of the sea, and a coast cell has high ground when land above 2 km of
the same landmass lies within 200 km. Each coast cell gets a setting from the `regime` codes of the
cells (land and sea) within `--radius-km` of it (Earth-normalized km), the first that holds of:

  collision  a collision belt (regime 5)
  arc        a trench, arc or back-arc (regimes 2, 3, 4)
  rifted     a rift, a ridge or a passive margin (regimes 6, 1, 9)
  hotspot    a hotspot (regime 7)
  other      the rest (craton and ocean basin only: an inland sea, a transform or a margin the regime
             does not mark)

and the high ground it counts gets an origin from the nearest cell above 2 km: `him` (a Himalayan
orogeny), `and` (an Andean one) or `none`.

Keys (shares are of the coast with high ground unless named otherwise):
  coast_hi                the share of the coast with high ground (equals v1census's)
  hi_<set>                share of the coast with high ground in each setting (they sum to 1)
  coast_<set>             share of the coast in each setting
  rate_<set>              the coast with high ground among the coast of that setting
  without_<set>           coast_hi if that setting's high coasts had none (what removing them could reach)
  hg_<origin>             share of the coast with high ground by its high ground's origin
  <set>_hg_<origin>       the same within a setting, a share of that setting's high coasts
  hg_age_med_<set>        median orogeny age of the high ground of a setting's high coasts, Myr (NaN: none)
  hg_dist_med_<set>       median distance from a setting's high coast to its high ground, km
  hg_thick_med_<set>      median crust thickness under that high ground, km
  coast_thick_med_<set>   median crust thickness under a setting's high coasts, km

Needs numpy and scipy (the pool resolves them with nix). Version 1.
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

SETS = ("collision", "arc", "rifted", "hotspot", "other")
SET_REGIMES = {"collision": (5,), "arc": (2, 3, 4), "rifted": (6, 1, 9), "hotspot": (7,)}
ORIGINS = ("him", "and", "none")


def coastset(fields_dir, radius_km):
    f, _globals, e = cc.load_dump(fields_dir)
    h, w, r = e["h"], e["w"], e["r"]
    lat_r, ok_r = cc.lat_of_rows(h, r, w * r)
    lat = np.repeat(lat_r[:, None], w, 1)
    ok = np.repeat(ok_r[:, None], w, 1)
    lon = np.repeat((360.0 * np.arange(w) / w)[None, :], h, 0)
    ncell = int(ok.sum())
    spacing = np.sqrt(cc.EARTH_KM2 / ncell)
    p = cc.xyz(lat[ok], lon[ok])
    z = ((f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V)[ok]
    reg = np.rint(np.nan_to_num(f["regime"][ok], nan=0.0)).astype(int)
    oro = np.rint(np.nan_to_num(f["plates/orogeny_type"][ok], nan=0.0)).astype(int)
    oro_age = f["plates/orogeny_age_myr"][ok].astype(float)
    thick = f["plates/thickness_m"][ok].astype(float) / 1000.0

    # coast_metrics' coast and high ground, keeping the nearest high cell of each high coast.
    land = z > 0
    d_sea = np.zeros(z.size)
    ds, _ = cKDTree(p[~land]).query(p[land])
    d_sea[land] = cc.arc_km(ds)
    coast = land & (d_sea <= 1.6 * spacing)
    hi = land & (z > 2000)
    comp = cc.components(p, land, spacing)
    has = np.zeros(z.size, bool)
    near_hi = np.full(z.size, -1)
    near_d = np.full(z.size, np.nan)
    for c in np.unique(comp[hi]):
        hm, cm = np.nonzero(hi & (comp == c))[0], np.nonzero(coast & (comp == c))[0]
        if cm.size == 0:
            continue
        d, j = cKDTree(p[hm]).query(p[cm], distance_upper_bound=cc.chord(200.0))
        found = np.isfinite(d)
        has[cm[found]] = True
        near_hi[cm[found]] = hm[j[found]]
        near_d[cm[found]] = cc.arc_km(d[found])

    # Each coast cell's setting from the regimes within the radius.
    ci = np.nonzero(coast)[0]
    setting = np.full(z.size, -1)
    tree = cKDTree(p)
    nbrs = tree.query_ball_point(p[ci], cc.chord(radius_km))
    for k, i in enumerate(ci):
        seen = set(reg[nbrs[k]].tolist())
        s = len(SETS) - 1
        for si, name in enumerate(SETS[:-1]):
            if seen.intersection(SET_REGIMES[name]):
                s = si
                break
        setting[i] = s

    n_coast = max(int(coast.sum()), 1)
    hc = coast & has
    n_hc = max(int(hc.sum()), 1)
    origin = np.full(z.size, -1)
    hj = near_hi[hc]
    origin[hc] = np.where(oro[hj] == 2, 0, np.where(oro[hj] == 1, 1, 2))
    out = {"coast_hi": hc.sum() / n_coast}
    for si, name in enumerate(SETS):
        cs = coast & (setting == si)
        hs = hc & (setting == si)
        out[f"hi_{name}"] = hs.sum() / n_hc
        out[f"coast_{name}"] = cs.sum() / n_coast
        out[f"rate_{name}"] = hs.sum() / cs.sum() if cs.any() else None
        out[f"without_{name}"] = (hc.sum() - hs.sum()) / n_coast
        idx = np.nonzero(hs)[0]
        for oi, oname in enumerate(ORIGINS):
            out[f"{name}_hg_{oname}"] = float((origin[idx] == oi).mean()) if idx.size else None
        if idx.size:
            ages = oro_age[near_hi[idx]]
            ages = ages[np.isfinite(ages)]
            out[f"hg_age_med_{name}"] = float(np.median(ages)) if ages.size else None
            out[f"hg_dist_med_{name}"] = float(np.median(near_d[idx]))
            out[f"hg_thick_med_{name}"] = float(np.median(thick[near_hi[idx]]))
            out[f"coast_thick_med_{name}"] = float(np.median(thick[idx]))
        else:
            for k in ("hg_age_med_", "hg_dist_med_", "hg_thick_med_", "coast_thick_med_"):
                out[k + name] = None
    for oi, oname in enumerate(ORIGINS):
        out[f"hg_{oname}"] = float((origin[hc] == oi).mean()) if hc.any() else None
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("run")
    ap.add_argument("--fields")
    ap.add_argument("--radius-km", type=float, default=150.0)
    a = ap.parse_args()
    run = Path(a.run)
    values = coastset(a.fields or run / "fields", a.radius_km)
    print(json.dumps({k: (None if v is None else float(v)) for k, v in values.items()}, sort_keys=True))


if __name__ == "__main__":
    main()
