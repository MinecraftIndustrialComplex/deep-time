#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P1-26 per-cell climate on Earth's relief against Earth (docs/design/p1-26-per-cell-climate.md
§7.1): the metrics of the design's target table that need no observed grid, from a
`deeptime climate-dem --dump` folder.

    deeptime climate-dem --dem earth720.f32 --width 720 --dump out/ > out.log
    python3 tools/validation/climate_vs_obs.py out/ [out.log] [label]

Prints one JSON line. Earth's relief is GMT's `earth_relief_30m_p` (SRTM15+, Tozer et al. 2019)
turned into 720 × 360 little-endian f32, north first, from 0° E (`climate-dem` resamples it onto
the planet's projection and writes `z.f32` and the rows' latitudes); it and any observed grid stay out
of the repository (§7.1). The observed values the points are compared with are the design's
(§7.1's table); the grids it also names (Beck et al. 2018 Köppen, Beck et al. 2023 monthly T and
P) are not read here.

- t_global_c: the 2-D model's global mean (meta.json); target 14.1 ± 0.4 °C (Ben, 2026-10-02).
- along_row_std_winter / _annual, eta2_winter / _annual: land temperature at sea level (the
  monthly t_sl fields), rows 30–70°, as `tools/review/percell_metrics.py`; winter is January north
  and July south.
- ice_edge_std_s / _n: the standard deviation over longitudes of the most equatorward latitude
  whose winter-maximum sea-ice cover is ≥ 0.5.
- brest_vladivostok_jan_k, yakutsk_jan_c: January temperature (the lapse rate applied).
- mumbai_mm, dhaka_mm, sahara_mm, atacama_mm: annual precipitation (the Sahara 20–28° N,
  0–25° E and the Atacama 20–27° S, 71–69° W, means over their land cells).
- places_same / places_group: the reference cities `climate-dem` prints (its log).
"""
import json
import os
import re
import sys

import numpy as np

SEA = 31
LAPSE = 6.5e-3  # °C/m, ClimateParams::lapse_c_per_km's default


def cell(lat1, lat, lon, w):
    j = int(np.argmin(np.abs(lat1 - lat)))
    i = int((lon % 360.0) / 360.0 * w) % w
    return j, i


def eta2(v, lat, wgt, mask):
    band = np.floor((lat + 90.0) / 2.0).astype(int)
    vv, ww, bb = v[mask], wgt[mask], band[mask]
    mean = np.average(vv, weights=ww)
    tot = np.sum(ww * (vv - mean) ** 2)
    between = sum(
        ww[bb == b].sum() * (np.average(vv[bb == b], weights=ww[bb == b]) - mean) ** 2
        for b in np.unique(bb)
    )
    return float(between / tot)


def along_row_std(v, lat1, mask):
    s = [
        float(np.std(v[j][mask[j]]))
        for j in range(v.shape[0])
        if 30.0 <= abs(lat1[j]) <= 70.0 and mask[j].sum() >= 5
    ]
    return float(np.mean(s))


def main():
    d = sys.argv[1]
    log = sys.argv[2] if len(sys.argv) > 2 else None
    label = sys.argv[3] if len(sys.argv) > 3 else os.path.basename(os.path.normpath(d))
    meta = json.load(open(os.path.join(d, "meta.json")))
    w, h = meta["w"], meta["h"]
    rd = lambda n: np.fromfile(os.path.join(d, n + ".f32"), dtype="<f4").reshape(h, w)
    k = rd("koppen").astype(int)
    land = k != SEA
    # Row latitudes from meta.json (the planet's projection: Lambert's rows are uniform in sin φ);
    # each row's area weight is cos φ times its latitude span.
    lat1 = np.array(meta["lat"], dtype=float)
    span = np.abs(np.gradient(lat1))
    lat = np.repeat(lat1[:, None], w, axis=1)
    wgt = np.repeat((np.cos(np.radians(lat1)) * span)[:, None], w, axis=1)
    t_sl = [rd(f"t_sl_{m:02d}") for m in range(1, 13)]
    t_ann = sum(t_sl) / 12.0
    north = lat > 0
    winter = np.where(north, t_sl[0], t_sl[6])
    out = {"label": label, "t_global_c": meta["t_global_c"], "net_toa_w_m2": meta["net_toa_w_m2"]}
    out["glaciated_land"] = meta["glaciated_land"]
    out["land_t_mean_c"] = float(np.average(rd("t_mean")[land], weights=wgt[land]))
    out["land_p_mm"] = float(np.average(rd("precip")[land], weights=wgt[land]))
    groups = {}
    codes = meta["koppen_codes"]
    for g in "ABCDE":
        m = land & np.isin(k, [i for i, c in enumerate(codes) if c.startswith(g)])
        groups[g] = round(float(wgt[m].sum() / wgt[land].sum()), 3)
    out["koppen"] = groups
    mid = land & (np.abs(lat) >= 30) & (np.abs(lat) <= 70)
    out["along_row_std_winter"] = along_row_std(winter, lat1, land)
    out["along_row_std_annual"] = along_row_std(t_ann, lat1, land)
    out["eta2_winter"] = eta2(winter, lat, wgt, mid)
    out["eta2_annual"] = eta2(t_ann, lat, wgt, mid)
    p = rd("precip")
    out["eta2_ln_precip"] = eta2(np.log(np.maximum(p, 1.0)), lat, wgt, land)
    ice = rd("sea_ice_max")
    for hemi, rows in (("n", [j for j in range(h) if lat1[j] > 0]), ("s", [j for j in range(h) if lat1[j] < 0])):
        rows = sorted(rows, key=lambda j: abs(lat1[j]))
        edges = []
        for i in range(w):
            for j in rows:
                if not land[j, i] and ice[j, i] >= 0.5:
                    edges.append(abs(lat1[j]))
                    break
        out[f"ice_edge_mean_{hemi}"] = float(np.mean(edges)) if edges else None
        out[f"ice_edge_std_{hemi}"] = float(np.std(edges)) if edges else None
    # January temperature with the lapse rate: the relief is the land's height (≥ 0).
    z = rd("z")
    jan = t_sl[0] - LAPSE * np.maximum(z, 0.0)

    def at(a, la, lo):
        j, i = cell(lat1, la, lo, w)
        if not land[j, i]:
            best = None
            for dj in range(-2, 3):
                for di in range(-2, 3):
                    jj, ii = min(max(j + dj, 0), h - 1), (i + di) % w
                    if land[jj, ii] and (best is None or dj * dj + di * di < best[0]):
                        best = (dj * dj + di * di, jj, ii)
            if best:
                j, i = best[1], best[2]
        return float(a[j, i])

    out["brest_vladivostok_jan_k"] = at(jan, 48.4, -4.5) - at(jan, 43.1, 131.9)
    out["yakutsk_jan_c"] = at(jan, 62.0, 129.7)
    out["mumbai_mm"] = at(p, 19.1, 72.9)
    out["dhaka_mm"] = at(p, 23.7, 90.4)

    def box(la0, la1, lo0, lo1):
        m = land & (lat >= la0) & (lat <= la1)
        lon = (np.arange(w) + 0.5) / w * 360.0
        lon = np.where(lon > 180, lon - 360, lon)
        m &= ((lon >= lo0) & (lon <= lo1))[None, :]
        return float(np.average(p[m], weights=wgt[m]))

    out["sahara_mm"] = box(20, 28, 0, 25)
    out["atacama_mm"] = box(-27, -20, -71, -69)
    if log and os.path.exists(log):
        same = grp = 0
        for line in open(log):
            m = re.search(r"→ (\S+)\s+(same|group|differs)", line)
            if m:
                same += m.group(2) == "same"
                grp += m.group(2) in ("same", "group")
        out["places_same"], out["places_group"] = same, grp
    pc = meta.get("per_cell") or {}
    fc = pc.get("flux_corr") or {}
    out["flux_fit_rms_k"] = fc.get("rms_k")
    print(json.dumps(out))


if __name__ == "__main__":
    main()
