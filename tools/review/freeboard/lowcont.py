#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Why base35's continents stand low, against Earth: lowcont.py <mode> ...

base35 (master 9d9e4c94) against base34: land 0.227 to 0.209, continental crust under the sea 0.540 to 0.602, the sea
at the end 154 to 316 m (sea-level.md, "Why base35's continents stand low"). This splits a continental column's height
over the sea into its thickness, the height line it stands on at that thickness, and the sea, against CRUST1.0 and
SRTM15+, on the pool's cached runs only.

Modes:
  series <arm folder> ...
      summary.txt per run: the crust budget's cb_t lines (continental share, volume, mean thickness, land, the sea,
      the dry share of the continental crust) at every 50 Myr, the surface's hypsometry line and its erosion_bands
      line (the land's net lowering and its parts, m/Myr, by height band, window relief wz* and cell relief rz*).
      Medians over the runs.
  earth <crust1.bnds> <earth_relief_15m_p.grd>
      Earth: CRUST1.0 (Laske et al. 2013, 1 x 1 degree, read as m5earthbands.py reads it: column 1 the top of the ice
      or rock, 2 the rock, 8 the Moho, cells by cos latitude) by crust thickness: the share of the planet, the share
      above the sea, the solid surface's mean and p50 over the sea; the crust at least 20 km thick (the census's
      like-for-like continental crust, RLC-030); the cratons' boxes (m4cold.py's); and SRTM15+ at 15' (Tozer et al.
      2019) hypsometry: the land's mean and quantiles, its shares above 1 to 4 km, the shelf (0 to 200 m deep) and
      the slope (200 to 2,000 m) as shares of the planet, with and without the two ice sheets (cc.ice_sheets).
      Writes earth.json beside nothing: prints JSON on its last line.
  model <earth.json> <arm folder>[@N] ...
      The same on each run's final frame (fields/ on the L1 grid, equal-area rows; the surface's elevation, the
      plate model's thickness and kind, the climate's sea), medians over runs, and the split against Earth:
        D = sum_b s_b d_b (the dry share of crust >= 20 km), H = sum_b s_b m_b (its mean height over the sea),
        s_b the band's share of that crust, d_b its dry share, m_b its mean height over the sea, 1-km bands;
        thickness term  sum (s_M - s_E) x_E,
        line term       sum s_M (x_M0 - x_E)   (x_M0 the model's band read against its datum, not its sea),
        sea term        sum s_M (x_M - x_M0),
      which add up to D_M - D_E (or H_M - H_E) exactly; and the sea's fill on four factors against Earth's (Shapley,
      m5sea.fill): the water, the share of crust >= 20 km, its height histogram, and the thinner crust's.
Needs numpy (the VM's .pyenv-numpy, run with -I); earth needs h5py."""
import itertools
import json
import re
import statistics
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "m5diag"))
import continents_census as cc  # noqa: E402

T_MIN = 20.0  # km: the census's like-for-like continental crust (RLC-030, PLT-261's 20 km boundary)
BIN_EDGES = np.concatenate([np.arange(20.0, 70.0, 1.0), [70.0, 1e9]])
COARSE = [(0, 10), (10, 15), (15, 20), (20, 25), (25, 30), (30, 35), (35, 40), (40, 45), (45, 50), (50, 60),
          (60, 1e9)]
HIST_EDGES = np.arange(-12000.0, 9000.0 + 1.0, 25.0)
WATER_LAYER_M = 2664.0  # ledger: ISO-085
RHO_ICE_OVER_SEA = 917.0 / 1028.0  # ledger: CLM-210
CRATONS = [
    ("East European platform", (45, 70, 20, 60)),
    ("Siberian platform", (55, 75, 85, 130)),
    ("Canadian shield", (45, 70, -100, -60)),
    ("Australia, west and centre", (-35, -15, 113, 140)),
    ("Africa", (-35, 15, -18, 52)),
    ("Brazilian shield", (-30, 5, -60, -35)),
]  # m4cold.py's boxes


def med(xs):
    xs = [x for x in xs if x is not None and np.isfinite(x)]
    return statistics.median(xs) if xs else float("nan")


def wq(x, w, q):
    if x.size == 0:
        return float("nan")
    o = np.argsort(x)
    c = np.cumsum(w[o]) / w[o].sum()
    return float(np.interp(q, c, x[o]))


def wmean(x, w):
    return float((x * w).sum() / w.sum()) if w.sum() > 0 else float("nan")


def hist(h, w):
    c, _ = np.histogram(np.clip(h, HIST_EDGES[0], HIST_EDGES[-1] - 1), HIST_EDGES, weights=w)
    return c / max(c.sum(), 1e-30)


def fill(W, f, hc, ho):
    """m5sea.fill: the level at which water W (m over the planet) fills every cell of the two histograms below it."""
    c = 0.5 * (HIST_EDGES[1:] + HIST_EDGES[:-1])
    lo, hi = HIST_EDGES[0], HIST_EDGES[-1]
    for _ in range(60):
        s = 0.5 * (lo + hi)
        v = f * (hc * np.maximum(s - c, 0.0)).sum() + (1.0 - f) * (ho * np.maximum(s - c, 0.0)).sum()
        lo, hi = (s, hi) if v < W else (lo, s)
    return 0.5 * (lo + hi)


def shapley(base, arm):
    keys = list(base)
    part = {k: 0.0 for k in keys}
    perms = list(itertools.permutations(keys))
    for p in perms:
        cur = dict(base)
        s0 = fill(cur["W"], cur["f"], cur["hc"], cur["ho"])
        for k in p:
            cur[k] = arm[k]
            s1 = fill(cur["W"], cur["f"], cur["hc"], cur["ho"])
            part[k] += (s1 - s0) / len(perms)
            s0 = s1
    return part


def profile(T, h, w, sea):
    """Per 1-km band of crust >= T_MIN: share s, dry share d (at `sea`), mean height over `sea` m; and the totals."""
    sel = T >= T_MIN
    tot = w[sel].sum()
    s, d, m = [], [], []
    for a, b in zip(BIN_EDGES[:-1], BIN_EDGES[1:]):
        k = sel & (T >= a) & (T < b)
        wk = w[k].sum()
        s.append(wk / tot if tot > 0 else 0.0)
        d.append(float((w[k] * (h[k] > sea)).sum() / wk) if wk > 0 else float("nan"))
        m.append(float(((h[k] - sea) * w[k]).sum() / wk) if wk > 0 else float("nan"))
    return np.array(s), np.array(d), np.array(m)


def coarse(T, h, w, sea, label=None):
    out = []
    W = w.sum()
    for a, b in COARSE:
        k = (T >= a) & (T < b)
        wk = w[k].sum()
        row = {"band": f"{a:g}-{b:g}" if b < 1e8 else f"{a:g}+", "share": float(wk / W)}
        if wk > 0:
            row.update({"dry": float((w[k] * (h[k] > sea)).sum() / wk), "mean": wmean(h[k] - sea, w[k]),
                        "p50": wq(h[k] - sea, w[k], 0.5)})
            if label is not None:
                row["cont"] = float((w[k] * label[k]).sum() / wk)
        out.append(row)
    return out


def hypso(z, w, sheets=None):
    """z: surface over the sea. The land's mean, quantiles and shares above 1 to 4 km; the shelf and slope."""
    keep = np.ones(z.shape, bool) if sheets is None else ~sheets
    z, w = z[keep], w[keep]
    land = z > 0
    W, WL = w.sum(), w[land].sum()
    zl, wl = z[land], w[land]
    out = {"land": float(WL / W), "land_mean": wmean(zl, wl)}
    for q in (0.1, 0.25, 0.5, 0.75, 0.9, 0.99):
        out[f"land_p{int(q * 100)}"] = wq(zl, wl, q)
    for k in (0.2, 0.5, 1, 2, 3, 4):
        out[f"land_above_{k:g}km"] = float(wl[zl > k * 1000].sum() / WL)
    out["shelf_0_200"] = float(w[(z <= 0) & (z > -200)].sum() / W)
    out["slope_200_2000"] = float(w[(z <= -200) & (z > -2000)].sum() / W)
    out["below_2000"] = float(w[z <= -2000].sum() / W)
    return out


# ------------------------------------------------------------------------------------------------ series

NUM = re.compile(r"([a-z_0-9]+)=(-?[0-9.]+(?:e-?[0-9]+)?)")


def runs_of(arg):
    folder, _, n = arg.partition("@")
    out = []
    for start in ("supercontinent", "scattered_continents"):
        for seed in range(1, (int(n) if n else 12) + 1):
            r = Path(folder) / f"{start}-{seed}"
            if (r / "fields" / "index.json").exists():
                out.append(r)
    return out


def parse_summary(run):
    cb, hyp, bands = {}, {}, {}
    for line in (run / "summary.txt").read_text().splitlines():
        if line.startswith("cb_t "):
            d = {k: float(v) for k, v in NUM.findall(line)}
            cb[int(d["t"])] = d
        elif "hypsometry land_frac_area" in line:
            hyp = {k: float(v) for k, v in NUM.findall(line.split("hypsometry", 1)[1])}
        elif "erosion_bands" in line:
            for tok in line.split("erosion_bands", 1)[1].split():
                name, _, vals = tok.partition(":")
                if name == "wrate":
                    continue
                try:
                    bands[name] = [float(x) for x in vals.split("/")]
                except ValueError:
                    pass
    return cb, hyp, bands


def main_series(args):
    for arg in args:
        rs = runs_of(arg)
        parsed = [parse_summary(r) for r in rs]
        print(f"== series {arg} ({len(rs)} runs)")
        cols = ["cont", "vol_km3", "mean_t_km", "land", "sea_m", "dry", "dry_sl", "above_2km", "felsic_km3",
                "felsic_ocean_km3"]
        print(f"{'t':>6} " + " ".join(f"{c:>12}" for c in cols))
        ts = sorted({t for p in parsed for t in p[0]})
        for t in ts:
            rows = [p[0][t] for p in parsed if t in p[0]]
            print(f"{t:>6} " + " ".join(f"{med([r.get(c) for r in rows]):>12.4g}" for c in cols))
        print("-- hypsometry (surface, final)")
        keys = sorted({k for p in parsed for k in p[1]})
        for k in keys:
            print(f"  {k} {med([p[1].get(k) for p in parsed]):.4g}")
        print("-- erosion_bands: share / net / stream power / of it the critical-slope cut / diffusion / glacial, m/Myr")
        names = list(parsed[0][2]) if parsed else []
        for nm in names:
            v = [med([p[2][nm][i] for p in parsed if nm in p[2] and len(p[2][nm]) > i]) for i in range(6)]
            print(f"  {nm:>8} " + " ".join(f"{x:>8.4g}" for x in v))


# ------------------------------------------------------------------------------------------------ earth

def earth_crust(bnds):
    b = np.loadtxt(bnds)
    assert b.shape == (64800, 9), b.shape
    lat = np.repeat(np.arange(89.5, -90, -1.0), 360)
    lon = np.tile(np.arange(-179.5, 180, 1.0), 180)
    w = np.cos(np.radians(lat))
    top, solid, rock, moho = b[:, 0] * 1000, b[:, 1] * 1000, b[:, 2] * 1000, b[:, 8] * 1000
    T = (rock - moho) / 1000.0
    return lat, lon, w, top, solid, rock, T


def main_earth(bnds, relief):
    lat, lon, w, top, solid, rock, T = earth_crust(bnds)
    sheets = cc.ice_sheets(lat, lon)
    out = {"source": "CRUST1.0 (Laske et al. 2013) and SRTM15+ 15' (Tozer et al. 2019)"}
    # The solid surface (ice or rock under the water) over the sea: what the sea fills.
    h = solid
    out["coarse"] = coarse(T, h, w, 0.0)
    s, d, m = profile(T, h, w, 0.0)
    out["profile"] = {"s": s.tolist(), "d": d.tolist(), "m": m.tolist()}
    c = T >= T_MIN
    out["cont20"] = {"share": float(w[c].sum() / w.sum()), "T_mean": wmean(T[c], w[c]),
                     "T_p10": wq(T[c], w[c], 0.1), "T_p50": wq(T[c], w[c], 0.5), "T_p90": wq(T[c], w[c], 0.9),
                     "dry": float((w[c] * (h[c] > 0)).sum() / w[c].sum()), "h_mean": wmean(h[c], w[c]),
                     "h_p50": wq(h[c], w[c], 0.5),
                     "vol_planet_km": float((w[c] * T[c]).sum() / w.sum())}
    c2 = c & ~sheets
    out["cont20_no_sheets"] = {"T_mean": wmean(T[c2], w[c2]), "dry": float((w[c2] * (h[c2] > 0)).sum() / w[c2].sum()),
                               "h_mean": wmean(h[c2], w[c2]), "h_p50": wq(h[c2], w[c2], 0.5)}
    # The rock height at the sea: the thickness at which the band's p50 crosses 0, by interpolation on 1-km bands.
    p50 = []
    for a in np.arange(20.0, 60.0, 1.0):
        k = (T >= a) & (T < a + 1)
        p50.append(wq(h[k], w[k], 0.5) if k.any() else float("nan"))
    out["p50_by_km"] = dict(zip([float(a) for a in np.arange(20.0, 60.0, 1.0)], p50))
    # The cratons' boxes, land only.
    land = (top - solid <= 0.0) & (top > 0.0)
    cr = {}
    for name, (la0, la1, lo0, lo1) in CRATONS:
        k = land & (lat >= la0) & (lat <= la1) & (lon >= lo0) & (lon <= lo1)
        cr[name] = {"T_p50": wq(T[k], w[k], 0.5), "T_mean": wmean(T[k], w[k]), "h_mean": wmean(h[k], w[k]),
                    "h_p50": wq(h[k], w[k], 0.5), "h_p25": wq(h[k], w[k], 0.25), "h_p75": wq(h[k], w[k], 0.75)}
    out["cratons"] = cr
    # The sea's fill on Earth's own surface, the check that the factors start at Earth's sea.
    wet = solid < 0
    W = float((w * np.where(wet, -solid, 0.0)).sum() / w.sum())
    out["water_m"] = W
    out["fill"] = {"f": float(w[c].sum() / w.sum()), "hc": hist(h[c], w[c]).tolist(), "ho": hist(h[~c], w[~c]).tolist(),
                   "W": W}
    out["fill_check_m"] = fill(W, out["fill"]["f"], np.array(out["fill"]["hc"]), np.array(out["fill"]["ho"]))
    # SRTM15+ hypsometry.
    sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "m5diag"))
    import m5earth  # noqa: E402
    zg, la, lo = m5earth.load(relief)
    LA = np.repeat(la[:, None], len(lo), 1)
    LO = np.repeat(lo[None, :], len(la), 0)
    wg = np.cos(np.radians(LA))
    sh = cc.ice_sheets(LA, LO)
    out["hypso_all"] = hypso(zg.ravel(), wg.ravel())
    out["hypso_no_sheets"] = hypso(zg.ravel(), wg.ravel(), sh.ravel())
    print_earth(out)
    print(json.dumps(out))


def print_earth(e):
    print("== Earth: CRUST1.0 by crust thickness (share of the planet, dry share, solid surface over the sea)")
    for r in e["coarse"]:
        print(f"  {r['band']:>7} share {r['share']:.4f} dry {r.get('dry', float('nan')):.3f} "
              f"mean {r.get('mean', float('nan')):8.0f} p50 {r.get('p50', float('nan')):8.0f}")
    print("-- crust >= 20 km:", json.dumps(e["cont20"]))
    print("-- crust >= 20 km, the ice sheets out:", json.dumps(e["cont20_no_sheets"]))
    print("-- p50 height by thickness:", " ".join(f"{k:g}:{v:.0f}" for k, v in e["p50_by_km"].items()))
    print("-- cratons (land):")
    for k, v in e["cratons"].items():
        print(f"  {k:>28} " + " ".join(f"{a} {b:.1f}" for a, b in v.items()))
    print(f"-- water over the planet {e['water_m']:.0f} m; Earth's fill on its own histograms {e['fill_check_m']:.0f} m")
    print("-- SRTM15+ hypsometry, all:", json.dumps(e["hypso_all"]))
    print("-- SRTM15+ hypsometry, without Antarctica and Greenland:", json.dumps(e["hypso_no_sheets"]))


# ------------------------------------------------------------------------------------------------ model

def model_run(run):
    import contextlib
    import io
    with contextlib.redirect_stdout(io.StringIO()):
        f, g, e = cc.load_dump(run / "fields")
    h_, w_, r_ = e["h"], e["w"], e["r"]
    lat_r, ok_r = cc.lat_of_rows(h_, r_, w_ * r_)
    ok = np.repeat(ok_r[:, None], w_, 1)
    sea = float(g["globals"]["climate/sea_level_m"])
    hs = np.nan_to_num(f["surface/elevation_m"][ok])
    T = np.nan_to_num(f["plates/thickness_m"][ok]) / 1000.0
    cont = np.nan_to_num(f["plates/crust_type"][ok]) > 0.5
    ice = np.nan_to_num(f["climate/ice_thickness_m"][ok])
    zc = (f["elevation"][ok] - cc.SEA_Y) * cc.M_PER_BLOCK_V  # the census's ground over the sea
    w = np.ones(hs.shape)  # equal-area rows
    out = {"run": run.name, "sea": sea, "land_census": float((zc > 0).mean())}
    out["coarse"] = coarse(T, hs, w, sea, label=cont.astype(float))
    s, d, m = profile(T, hs, w, sea)
    s0, d0, m0 = profile(T, hs, w, 0.0)
    out["profile"] = {"s": s.tolist(), "d": d.tolist(), "m": m.tolist(), "d0": d0.tolist(), "m0": m0.tolist()}
    c = T >= T_MIN
    out["cont20"] = {"share": float(c.mean()), "T_mean": float(T[c].mean()), "T_p10": float(np.quantile(T[c], 0.1)),
                     "T_p50": float(np.quantile(T[c], 0.5)), "T_p90": float(np.quantile(T[c], 0.9)),
                     "dry": float((hs[c] > sea).mean()), "h_mean": float((hs[c] - sea).mean()),
                     "h_p50": float(np.median(hs[c] - sea)), "vol_planet_km": float(T[c].sum() / T.size),
                     "label_share": float(cont[c].mean())}
    out["label"] = {"share": float(cont.mean()), "T_mean": float(T[cont].mean()),
                    "T_p10": float(np.quantile(T[cont], 0.1)), "T_p50": float(np.quantile(T[cont], 0.5)),
                    "T_p90": float(np.quantile(T[cont], 0.9)), "dry": float((hs[cont] > sea).mean()),
                    "h_mean": float((hs[cont] - sea).mean()), "h_p50": float(np.median(hs[cont] - sea)),
                    "vol_planet_km": float(T[cont].sum() / T.size), "under20": float((T[cont] < T_MIN).mean())}
    lat = np.repeat(lat_r[:, None], w_, 1)[ok]
    out["hypso"] = hypso(hs - sea, w)
    out["hypso_census"] = hypso(zc, w)
    p50 = {}
    for a in np.arange(20.0, 60.0, 1.0):
        k = (T >= a) & (T < a + 1)
        p50[float(a)] = float(np.median(hs[k] - sea)) if k.sum() > 20 else float("nan")
    out["p50_by_km"] = p50
    W = WATER_LAYER_M - float(ice.mean()) * RHO_ICE_OVER_SEA
    out["fill"] = {"f": float(c.mean()), "hc": hist(hs[c], w[c]).tolist(), "ho": hist(hs[~c], w[~c]).tolist(), "W": W}
    out["fill_self_m"] = fill(W, out["fill"]["f"], np.array(out["fill"]["hc"]), np.array(out["fill"]["ho"]))
    out["lat_abs_mean_cont"] = float(np.abs(lat[c]).mean())
    return out


def split(sM, xM, xM0, sE, xE):
    """Thickness, line and sea terms of sum s x between the model and Earth (NaN bins count as 0 weight)."""
    xE = np.nan_to_num(xE)
    xM, xM0 = np.nan_to_num(xM), np.nan_to_num(xM0)
    thick = float(((sM - sE) * xE).sum())
    line = float((sM * (xM0 - xE)).sum())
    sea = float((sM * (xM - xM0)).sum())
    return {"model": float((sM * xM).sum()), "earth": float((sE * xE).sum()), "thickness": thick, "line": line,
            "sea": sea}


def main_model(earth_json, args):
    e = json.loads(Path(earth_json).read_text().strip().splitlines()[-1])
    sE = np.array(e["profile"]["s"])
    dE, mE = np.array(e["profile"]["d"]), np.array(e["profile"]["m"])
    for arg in args:
        rs = runs_of(arg)
        res = [model_run(r) for r in rs]
        print(f"== model {arg} ({len(rs)} runs), final frame, medians")
        print(f"  sea {med([r['sea'] for r in res]):.0f} m; census land {med([r['land_census'] for r in res]):.3f}")
        print("-- by crust thickness (share of the planet, continental label share, dry share, surface over the sea)")
        for i, (a, b) in enumerate(COARSE):
            rows = [r["coarse"][i] for r in res]
            print(f"  {rows[0]['band']:>7} share {med([x['share'] for x in rows]):.4f} cont "
                  f"{med([x.get('cont') for x in rows]):.2f} dry {med([x.get('dry') for x in rows]):.3f} mean "
                  f"{med([x.get('mean') for x in rows]):8.0f} p50 {med([x.get('p50') for x in rows]):8.0f}")
        for key in ("cont20", "label"):
            ks = list(res[0][key])
            print(f"-- {key}: " + " ".join(f"{k} {med([r[key][k] for r in res]):.4g}" for k in ks))
        for key in ("hypso", "hypso_census"):
            ks = list(res[0][key])
            print(f"-- {key}: " + " ".join(f"{k} {med([r[key][k] for r in res]):.4g}" for k in ks))
        print("-- p50 height over the sea by thickness, model / Earth:",
              " ".join(f"{a:g}:{med([r['p50_by_km'][a] for r in res]):.0f}/{e['p50_by_km'][str(a)]:.0f}"
                       for a in np.arange(20.0, 60.0, 2.0)))
        # The split, per run, then medians.
        parts = {"dry": [], "height": []}
        for r in res:
            p = r["profile"]
            sM = np.array(p["s"])
            parts["dry"].append(split(sM, np.array(p["d"]), np.array(p["d0"]), sE, dE))
            parts["height"].append(split(sM, np.array(p["m"]), np.array(p["m0"]), sE, mE))
        for k, v in parts.items():
            print(f"-- split of the crust >= 20 km's {k} against Earth: " +
                  " ".join(f"{t} {med([x[t] for x in v]):.4g}" for t in ("model", "earth", "thickness", "line", "sea")))
        # Land: area x dry share.
        AE, DE = e["cont20"]["share"], e["cont20"]["dry"]
        la = []
        for r in res:
            AM, DM = r["cont20"]["share"], r["cont20"]["dry"]
            la.append({"area": (AM - AE) * DE, "dry": AM * (DM - DE), "land20_M": AM * DM, "land20_E": AE * DE})
        print("-- land on crust >= 20 km, A x D: " + " ".join(f"{t} {med([x[t] for x in la]):.4g}" for t in la[0]))
        # The sea against Earth's on four factors.
        fe = e["fill"]
        base = {"W": fe["W"], "f": fe["f"], "hc": np.array(fe["hc"]), "ho": np.array(fe["ho"])}
        sh = []
        for r in res:
            fm = r["fill"]
            arm = {"W": fm["W"], "f": fm["f"], "hc": np.array(fm["hc"]), "ho": np.array(fm["ho"])}
            part = shapley(base, arm)
            part["total"] = fill(arm["W"], arm["f"], arm["hc"], arm["ho"]) - fill(base["W"], base["f"], base["hc"],
                                                                                 base["ho"])
            part["run_sea"] = r["sea"]
            sh.append(part)
        print("-- the sea against Earth's (Shapley, m; total = the model's fill less Earth's): " +
              " ".join(f"{k} {med([x[k] for x in sh]):.0f}" for k in ("W", "f", "hc", "ho", "total", "run_sea")))
        print("-- per run: " + " ".join(f"{r['run']}:{r['sea']:.0f}/{r['fill_self_m']:.0f}" for r in res))


if __name__ == "__main__":
    mode = sys.argv[1]
    if mode == "series":
        main_series(sys.argv[2:])
    elif mode == "earth":
        main_earth(sys.argv[2], sys.argv[3])
    elif mode == "model":
        main_model(sys.argv[2], sys.argv[3:])
    else:
        sys.exit(__doc__)
