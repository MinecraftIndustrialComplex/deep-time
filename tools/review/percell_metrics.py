#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P1-26 per-cell climate: the design's stripe and variety metrics (docs/design/p1-26-per-cell-
climate.md §1.1, §7.4) on a run's final fields.

    deeptime stage climate --preset earthlike_quick_16k ... --out run.dtcp
    deeptime dump-fields --in run.dtcp --out dump/
    python3 tools/review/percell_metrics.py dump/ [label]

Prints one JSON line. Everything is area-weighted (cos φ of the row; the rows are taken as evenly
spaced in latitude, the equirectangular projection's: change `row_lat` when the projection
changes). Land is where the Köppen class is not the sea's (31).

- along_row_std_winter / _annual: the mean over rows at 30–70° of the standard deviation of the
  land's sea-level temperature along the row (winter: January north, July south).
- eta2_*: the share of the land's variance explained by 2° latitude bands (1 = purely banded).
- sea_ice_eta2: the same for the annual sea ice over the sea; ice_edge_std_n/_s: the standard
  deviation over longitudes of the most equatorward latitude whose (winter-maximum) cover is ≥ 0.5.
- koppen_*: the main groups' land shares; koppen_longest_row_run_deg: the longest stretch of
  longitude over which a class boundary follows one grid row (§7.4 item 4).
- polar_*: the polar continent (land poleward of 60° in the hemisphere with more of it): its
  winter along-row std and the classes holding ≥ 5 % of it.
"""
import json
import os
import sys

import numpy as np

SEA = 31


def load(d):
    idx = json.load(open(os.path.join(d, "index.json")))
    out = {}
    for name, e in idx.items():
        a = np.fromfile(os.path.join(d, e["file"]), dtype="<f4").reshape(e["h"], e["w"])
        out[name] = a
    return out


def row_lat(h):
    return 90.0 - 180.0 * (np.arange(h) + 0.5) / h


def eta2(v, lat, wgt, mask):
    band = np.floor((lat + 90.0) / 2.0).astype(int)
    vv, ww, bb = v[mask], wgt[mask], band[mask]
    if ww.sum() == 0:
        return float("nan")
    mean = np.average(vv, weights=ww)
    tot = np.sum(ww * (vv - mean) ** 2)
    between = 0.0
    for b in np.unique(bb):
        s = bb == b
        between += ww[s].sum() * (np.average(vv[s], weights=ww[s]) - mean) ** 2
    return float(between / tot) if tot > 0 else float("nan")


def along_row_std(v, lat, mask, lo=30.0, hi=70.0):
    stds = []
    for j in range(v.shape[0]):
        if lo <= abs(lat[j]) <= hi:
            m = mask[j]
            if m.sum() >= 5:
                stds.append(float(np.std(v[j][m])))
    return float(np.mean(stds)) if stds else float("nan")


def main():
    d = sys.argv[1]
    label = sys.argv[2] if len(sys.argv) > 2 else os.path.basename(os.path.normpath(d))
    f = load(d)
    k = f["climate/koppen"].astype(int)
    h, w = k.shape
    lat1 = row_lat(h)
    lat = np.repeat(lat1[:, None], w, axis=1)
    wgt = np.cos(np.radians(lat))
    land = k != SEA
    sea = ~land
    north = lat > 0
    out = {"label": label, "w": w, "h": h}
    t_mean = f["climate/t_mean_c"]
    out["t_surface_c"] = float(np.average(t_mean, weights=wgt))
    out["t_land_c"] = float(np.average(t_mean[land], weights=wgt[land]))
    out["p_land_mm"] = float(np.average(f["climate/precip_mm"][land], weights=wgt[land]))
    have_months = "climate/month/t_sl_c_01" in f
    if have_months:
        jan, jul = f["climate/month/t_sl_c_01"], f["climate/month/t_sl_c_07"]
        winter = np.where(north, jan, jul)
        annual = sum(f["climate/month/t_sl_c_%02d" % m] for m in range(1, 13)) / 12.0
        out["along_row_std_winter"] = along_row_std(winter, lat1, land)
        out["along_row_std_annual"] = along_row_std(annual, lat1, land)
        out["eta2_winter"] = eta2(winter, lat, wgt, land)
        out["eta2_annual"] = eta2(annual, lat, wgt, land)
        out["eta2_ln_precip"] = eta2(np.log(np.maximum(f["climate/precip_mm"], 1e-3)), lat, wgt, land)
    ice = f.get("climate/sea_ice")
    if ice is not None:
        out["sea_ice_eta2"] = eta2(ice, lat, wgt, sea)
        out["sea_ice_frac"] = float(np.average(ice[sea], weights=wgt[sea]))
        top = f.get("climate/sea_ice_max", ice)
        for hemi, name in ((1, "n"), (-1, "s")):
            edges = []
            for i in range(w):
                rows = [j for j in range(h) if hemi * lat1[j] > 0 and sea[j, i] and top[j, i] >= 0.5]
                if rows:
                    edges.append(min(abs(lat1[j]) for j in rows))
            out["ice_edge_n_cols_" + name] = len(edges)
            out["ice_edge_mean_" + name] = float(np.mean(edges)) if edges else float("nan")
            out["ice_edge_std_" + name] = float(np.std(edges)) if edges else float("nan")
    groups = {}
    codes = None
    for g in "ABCDE":
        groups[g] = 0.0
    # Köppen codes in the order of koppen::CODES (index = class).
    codes = ["Af", "Am", "Aw", "As", "BWh", "BWk", "BSh", "BSk", "Csa", "Csb", "Csc", "Cwa", "Cwb",
             "Cwc", "Cfa", "Cfb", "Cfc", "Dsa", "Dsb", "Dsc", "Dsd", "Dwa", "Dwb", "Dwc", "Dwd", "Dfa",
             "Dfb", "Dfc", "Dfd", "ET", "EF", "sea"]
    lw = wgt[land].sum()
    for c in np.unique(k[land]):
        g = codes[c][0] if c < len(codes) else "?"
        groups[g] = groups.get(g, 0.0) + float(wgt[land & (k == c)].sum() / lw)
    out["koppen"] = {g: round(v, 3) for g, v in groups.items()}
    # The longest run of a class boundary along one row (between row j and j + 1), degrees.
    longest = 0
    for j in range(h - 1):
        diff = (k[j] != k[j + 1]) & land[j] & land[j + 1]
        run = best = 0
        for x in np.concatenate([diff, diff]):  # wraps
            run = run + 1 if x else 0
            best = max(best, run)
        longest = max(longest, min(best, w))
    out["koppen_longest_row_run_deg"] = longest * 360.0 / w
    if have_months:
        pn = (land & (lat >= 60)).sum()
        ps = (land & (lat <= -60)).sum()
        hemi = 1 if pn >= ps else -1
        pm = land & (hemi * lat >= 60)
        if pm.sum() > 0:
            out["polar_hemisphere"] = hemi
            out["polar_land_share"] = float(wgt[pm].sum() / lw)
            out["polar_along_row_std_winter"] = along_row_std(winter, lat1, pm, 60.0, 90.0)
            cls, cnt = np.unique(k[pm], return_counts=True)
            sh = {codes[c]: round(float(wgt[pm & (k == c)].sum() / wgt[pm].sum()), 3) for c in cls}
            out["polar_classes_ge_5pct"] = {c: s for c, s in sh.items() if s >= 0.05}
    print(json.dumps(out))


if __name__ == "__main__":
    main()
