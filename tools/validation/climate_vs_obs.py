#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P1-26 per-cell climate on Earth's relief against Earth (docs/design/p1-26-per-cell-climate.md
§7.1): the metrics of the design's target table that need no observed grid, from a
`deeptime climate-dem --dump` folder.

    deeptime climate-dem --dem earth720.f32 --width 720 --dump out/ > out.log
    python3 tools/validation/climate_vs_obs.py out/ [out.log] [label] [--obs OBS_DIR]

Prints one JSON line. Earth's relief is GMT's `earth_relief_30m_p` (SRTM15+, Tozer et al. 2019)
turned into 720 × 360 little-endian f32, north first, from 0° E (`climate-dem` resamples it onto
the planet's projection and writes `z.f32` and the rows' latitudes); it and any observed grid stay out
of the repository (§7.1). The observed values the points are compared with are the design's
(§7.1's table).

With `--obs OBS_DIR` it also scores the run against the observed grids (CC BY 4.0, validation
only, never committed or shipped), which OBS_DIR must hold:

- `Beck_KG_V1_present_0p5.tif`: Beck et al. 2018's present Köppen–Geiger classes at 0.5°
  (figshare 6396959, `Beck_KG_V1.zip`; codes 1–30 in the paper's order, 0 sea);
- `1961_1990/ensemble_mean_0p5.nc`: Beck et al. 2023's monthly air temperature (°C) and
  precipitation (mm/month) at 0.5° (figshare 21789074, `climate_data_0p5.zip`).

Each model cell takes the observed 0.5° cell under its centre; land is where both call it land
(the observed land is Beck 2018's). The `obs_*` keys:

- obs_land_p_mm / land_p_mm_on_obs: the observed and the model's annual land precipitation on
  that common land (area-weighted), also without Antarctica (`_no_ant`, north of 60° S);
- ln_p_rmse: RMSE of ln annual precipitation (both ≥ 1 mm) over the common land; eta2_ln_p_obs;
- t_month_rmse / t_month_bias: monthly air temperature over the common land, the model's at its
  height (its sea-level field less 6.5 °C/km) against the observed;
- koppen_exact / koppen_group: shares of the common land in the same class and main group;
- along_row_std_winter_obs, eta2_winter_obs: the observed winter spread on the model's grid
  (observed temperatures brought to sea level with 6.5 °C/km on the model's relief);
  along_row_std_winter_height(_obs): the model's and the observed spread at the ground's height;
- regions: model and observed annual precipitation over named land boxes.

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


# Beck et al. 2018's legend (Table 2), codes 1–30.
BECK_CODES = [
    "Af", "Am", "Aw", "BWh", "BWk", "BSh", "BSk", "Csa", "Csb", "Csc", "Cwa", "Cwb", "Cwc", "Cfa",
    "Cfb", "Cfc", "Dsa", "Dsb", "Dsc", "Dsd", "Dwa", "Dwb", "Dwc", "Dwd", "Dfa", "Dfb", "Dfc",
    "Dfd", "ET", "EF",
]

# Land boxes for the regional rain: (name, lat0, lat1, lon0, lon1), longitudes −180…180.
REGIONS = [
    ("sahara", 18, 30, -10, 30),
    ("amazon", -10, 2, -70, -50),
    ("congo", -5, 5, 15, 28),
    ("india", 15, 28, 72, 88),
    ("se_china", 22, 32, 105, 120),
    ("w_europe", 43, 55, -5, 15),
    ("siberia", 55, 70, 90, 130),
    ("central_asia", 38, 50, 55, 80),
    ("australia_interior", -30, -20, 120, 145),
    ("e_usa", 32, 42, -90, -75),
    ("w_usa_interior", 35, 45, -118, -105),
    ("sahel", 10, 16, -10, 30),
]


def load_obs(od):
    """The observed grids, cached as an .npz beside them."""
    cache = os.path.join(od, "obs_cache.npz")
    if os.path.exists(cache):
        z = np.load(cache)
        return z["kg"], z["t"], z["p"]
    from PIL import Image
    import netCDF4

    kg = np.array(Image.open(os.path.join(od, "Beck_KG_V1_present_0p5.tif"))).astype(np.int16)
    ds = netCDF4.Dataset(os.path.join(od, "1961_1990", "ensemble_mean_0p5.nc"))
    lat = np.array(ds["lat"][:])
    lon = np.array(ds["lon"][:])
    t = np.ma.filled(ds["air_temperature"][:].astype(np.float32), np.nan)
    p = np.ma.filled(ds["precipitation"][:].astype(np.float32), np.nan)
    # North first, from 180° W (Beck 2018's GeoTIFF is laid out the same way).
    if lat[0] < lat[-1]:
        t, p = t[:, ::-1, :], p[:, ::-1, :]
    if lon[0] > 0:
        raise SystemExit("climate_vs_obs: expected longitudes from 180° W")
    np.savez_compressed(cache, kg=kg, t=t, p=p)
    return kg, t, p


def obs_metrics(out, od, lat1, w, h, land, wgt, k, codes, t_sl, p, z):
    kg, t_obs, p_obs = load_obs(od)
    lon = (np.arange(w) + 0.5) / w * 360.0
    lon = np.where(lon >= 180.0, lon - 360.0, lon)
    io = np.clip(((lon + 180.0) / 0.5).astype(int), 0, 719)
    jo = np.clip(((90.0 - lat1) / 0.5).astype(int), 0, 359)
    kgo = kg[jo][:, io]
    to = t_obs[:, jo][:, :, io]
    po = p_obs[:, jo][:, :, io]
    po_ann = np.nansum(po, axis=0)
    both = land & (kgo > 0) & np.isfinite(to).all(axis=0) & np.isfinite(po).all(axis=0)
    lat = np.repeat(lat1[:, None], w, axis=1)
    noant = both & (lat > -60.0)
    ww = wgt
    out["obs_land_p_mm"] = float(np.average(po_ann[both], weights=ww[both]))
    out["land_p_mm_on_obs"] = float(np.average(p[both], weights=ww[both]))
    out["obs_land_p_mm_no_ant"] = float(np.average(po_ann[noant], weights=ww[noant]))
    out["land_p_mm_on_obs_no_ant"] = float(np.average(p[noant], weights=ww[noant]))
    lm, lo = np.log(np.maximum(p, 1.0)), np.log(np.maximum(po_ann, 1.0))
    out["ln_p_rmse"] = float(np.sqrt(np.average((lm - lo)[both] ** 2, weights=ww[both])))
    out["ln_p_bias"] = float(np.average((lm - lo)[both], weights=ww[both]))
    out["eta2_ln_p_obs"] = eta2(lo, lat, ww, both)
    zl = np.maximum(z, 0.0)
    dt = np.stack([t_sl[m] - LAPSE * zl - to[m] for m in range(12)])
    wm = np.repeat(ww[None], 12, axis=0)
    bm = np.repeat(both[None], 12, axis=0)
    out["t_month_rmse"] = float(np.sqrt(np.average(dt[bm] ** 2, weights=wm[bm])))
    out["t_month_bias"] = float(np.average(dt[bm], weights=wm[bm]))
    # Köppen: the model's codes by name against Beck's.
    model_names = np.array(codes, dtype=object)[np.clip(k, 0, len(codes) - 1)]
    obs_names = np.array([""] + BECK_CODES, dtype=object)[np.clip(kgo, 0, 30)]
    same = both & (model_names == obs_names)
    grp = both & (np.vectorize(lambda a: a[:1])(model_names) == np.vectorize(lambda a: a[:1])(obs_names))
    out["koppen_exact"] = float(ww[same].sum() / ww[both].sum())
    out["koppen_group"] = float(ww[grp].sum() / ww[both].sum())
    obs_groups = {}
    for g in "ABCDE":
        m = both & np.vectorize(lambda a: a.startswith(g))(obs_names)
        obs_groups[g] = round(float(ww[m].sum() / ww[both].sum()), 3)
    out["koppen_obs"] = obs_groups
    winter_obs = np.where(lat > 0, to[0], to[6]) + LAPSE * zl
    mid = both & (np.abs(lat) >= 30) & (np.abs(lat) <= 70)
    out["along_row_std_winter_obs"] = along_row_std(winter_obs, lat1, both)
    # The same at the ground's height (no lapse rate), model and observed.
    winter_h = np.where(lat > 0, t_sl[0], t_sl[6]) - LAPSE * zl
    out["along_row_std_winter_height"] = along_row_std(winter_h, lat1, both)
    out["along_row_std_winter_height_obs"] = along_row_std(np.where(lat > 0, to[0], to[6]), lat1, both)
    out["eta2_winter_obs"] = eta2(winter_obs, lat, ww, mid)
    lonm = np.repeat(lon[None, :], h, axis=0)
    reg = {}
    for name, a0, a1, b0, b1 in REGIONS:
        m = both & (lat >= a0) & (lat <= a1) & (lonm >= b0) & (lonm <= b1)
        if m.sum() == 0:
            continue
        reg[name] = [round(float(np.average(p[m], weights=ww[m]))), round(float(np.average(po_ann[m], weights=ww[m])))]
    out["regions"] = reg
    # Zonal land means by 10° bands, model / observed.
    zon = {}
    for a in range(-60, 90, 10):
        m = both & (lat >= a) & (lat < a + 10)
        if m.sum() > 20:
            zon[str(a)] = [round(float(np.average(p[m], weights=ww[m]))), round(float(np.average(po_ann[m], weights=ww[m])))]
    out["zonal_land_p"] = zon


def main():
    args = sys.argv[1:]
    od = None
    if "--obs" in args:
        i = args.index("--obs")
        od = args[i + 1]
        del args[i : i + 2]
    d = args[0]
    log = args[1] if len(args) > 1 else None
    label = args[2] if len(args) > 2 else os.path.basename(os.path.normpath(d))
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
    if od:
        obs_metrics(out, od, lat1, w, h, land, wgt, k, codes, t_sl, p, z)
    pc = meta.get("per_cell") or {}
    fc = pc.get("flux_corr") or {}
    out["flux_fit_rms_k"] = fc.get("rms_k")
    print(json.dumps(out))


if __name__ == "__main__":
    main()
