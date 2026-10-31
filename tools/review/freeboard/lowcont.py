#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Why base35's continents stand low, against Earth: lowcont.py <mode> ...

base35 (master 9d9e4c94) against base34: land 0.227 to 0.209, continental crust under the sea 0.540 to 0.602, the sea
at the end 174 to 319 m over the datum (the final frames' climate/sea_level_m, medians; sea-level.md, "Why base35's
continents stand low"). This splits a continental column's height over the sea into its thickness, the height line
it stands on at that thickness, and the sea, against CRUST1.0 and SRTM15+, and the area of the crust at least 20 km
thick into its sources through time, on the pool's cached runs only.

Modes:
  series <arm folder> ...
      summary.txt per run: the crust budget's cb_t lines (the continental label's share of the visible samples, its
      volume and mean own thickness, land, the record's sea, the dry share) at every 50 Myr, the label's area changes
      by process so far (cb_parts cont, shares of the visible samples), the surface's hypsometry line and its
      erosion_bands line (the land's net lowering and its parts, m/Myr, by height band, window relief wz* and cell
      relief rz*). Medians over the runs.
  area <arm folder>[@N] ...
      The crust budget's maps (maps/t*.bin, maps/pools/t*.bin, as thinning.py reads them) every 50 Myr: the crust at
      least 20 km thick, whatever its label, as a share of the planet (each sample 1/N of it, N the samples at the
      start, all visible; between resamplings the plates' samples keep their area and the gaps between them are
      unsampled new floor), by source: the start's continents and what grew on them (continental, no ocean basement),
      ocean floor made continental (an ocean basement over 1 km, Sample::basement: a matured island arc, a collision
      belt, the rest: the transition's margins and their resampled ring, the thickening's spread and the collapse's
      fill, PLT-254), and ocean-labelled crust; its mean thickness and volume; the planet's crust at 35 to 50 km; and
      at the end the interiors no orogeny raised (thinning.py's "never") over all their samples and their land only.
  earth <crust1.bnds> <earth_relief_15m_p.grd>
      Earth: CRUST1.0 (Laske et al. 2013, 1 x 1 degree; its bnds columns as its documentation counts them: 1 the
      water's top, 2 the ice's top (the solid surface, ice or rock), 3 the sediments' top (the rock), 9 the mantle's
      top (the Moho); cells by cos latitude) by crust thickness (column 3 less 9): the share of the planet, the share
      above the sea, the solid surface's mean and p50 over the sea, with the ice sheets in and out (cc.ice_sheets);
      the crust at least 20 km thick (the census's like-for-like continental crust, RLC-030); the cratons' boxes
      (m4cold.py's), the land only and every cell of crust at least 20 km thick; the fill on Earth's own histograms
      (a binning check) and at other continental areas; and SRTM15+ at 15' (Tozer et al. 2019) hypsometry: the land's
      mean and quantiles, its shares above 1 to 4 km, the shelf (0 to 200 m deep) and the slope (200 to 2,000 m) as
      shares of the planet, with and without the two ice sheets. Prints JSON on its last line (earth.json).
  model <earth.json> <arm folder>[@N] ...
      The same on each run's final frame (fields/ on the L1 grid, equal-area rows; the surface's elevation, the
      plate model's thickness and kind, the climate's sea; cells with no elevation or thickness dropped and
      counted), and the split against Earth:
        D = sum_b s_b d_b (the dry share of crust >= 20 km), H = sum_b s_b m_b (its mean height over the sea),
        s_b the band's share of that crust, d_b its dry share, m_b its mean height over the sea, 1-km bands;
        thickness term  sum (s_M - s_E) x_E,
        line term       sum s_M (x_M0 - x_E)   (x_M0 the model's band with the sea at the datum: a drowned column's
                                               Airy height S + (z - S) / 1.454 (ISO-078's water load, settle.rs),
                                               reloaded under a sea at 0),
        sea term        sum s_M (x_M - x_M0),
      which add up to D_M - D_E (or H_M - H_E) exactly per run, so their means over the runs add too (medians do
      not: printed with a residual); against Earth with its ice sheets in and out. And the sea's fill on four
      factors against Earth's (Shapley, m5sea.fill): the water, the share of crust >= 20 km, its height histogram
      and the thinner crust's, with the run's sea less its own fill as a fifth term (the residual: the steering's
      nudge and the open-sea rule, m5sea.py's), so the five add to the run's sea less Earth's fill.
Needs numpy (the VM's .pyenv-numpy, run with -I); area needs scipy (thinning.py), earth needs h5py."""
import itertools
import json
import re
import statistics
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "m5diag"))
sys.path.insert(0, str(Path(__file__).resolve().parent))  # thinning.py (-I leaves the script's folder off the path)
import continents_census as cc  # noqa: E402

T_MIN = 20.0  # km: the census's like-for-like continental crust (RLC-030, PLT-261's 20 km boundary)
BIN_EDGES = np.concatenate([np.arange(20.0, 70.0, 1.0), [70.0, 1e9]])
COARSE = [(0, 10), (10, 15), (15, 20), (20, 25), (25, 30), (30, 35), (35, 40), (40, 45), (45, 50), (50, 60),
          (60, 1e9)]
HIST_EDGES = np.arange(-12000.0, 9000.0 + 1.0, 25.0)
WATER_LAYER_M = 2664.0  # ledger: ISO-085
RHO_ICE_OVER_SEA = 917.0 / 1028.0  # ledger: CLM-210
WATER_LOADED = 3300.0 / (3300.0 - 1030.0)  # settle.rs WATER_LOADED (flexure.rs's mantle and water; ISO-078)
START_AREA = 0.4225  # cb_t t=-1000 cont on every run: the preset's start, (1 - 0.7) / (1 - 0.29) (Cogley 1984)
Z0_M, T_N_KM, AIRY_F = 525.0, 35.1, 3300.0 / 470.0  # PLT-281's line: z_0, T_n, pools::COLUMN_ISOSTATIC_FACTOR
AREA_TIMES = list(range(-1000, 1, 50))
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


def mn(xs):
    xs = [x for x in xs if x is not None and np.isfinite(x)]
    return statistics.fmean(xs) if xs else float("nan")


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


PART = re.compile(r"([a-z_]+):(-?[0-9.]+(?:e-?[0-9]+)?)")


def parse_summary(run):
    cb, hyp, bands, parts = {}, {}, {}, {}
    for line in (run / "summary.txt").read_text().splitlines():
        if line.startswith("cb_t "):
            d = {k: float(v) for k, v in NUM.findall(line)}
            cb[int(d["t"])] = d
        elif line.startswith("cb_parts "):
            t = int(float(line.split()[1].split("=")[1]))
            m = re.search(r" cont=\[([^]]*)\]", line)
            parts[t] = {k: float(v) for k, v in PART.findall(m.group(1))} if m else {}
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
    return cb, hyp, bands, parts


PART_TIMES = (-950, -900, -800, -500, 0)


def main_series(args):
    for arg in args:
        rs = runs_of(arg)
        parsed = [parse_summary(r) for r in rs]
        print(f"== series {arg} ({len(rs)} runs)")
        cols = ["vis", "cont", "vol_km3", "mean_t_km", "land", "sea_m", "dry", "dry_sl", "above_2km", "felsic_km3",
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
        # cb_parts divides by the visible samples at the snapshot; times vis / N (N: the start's, all visible) gives
        # shares of the planet. They are gross changes of the label at every thickness, not standing areas.
        print("-- the label's gross area changes by process so far (cb_parts cont x vis / N: shares of the planet, "
              "every thickness), medians:")
        names = sorted({k for p in parsed for t in PART_TIMES for k in p[3].get(t, {})})
        print(f"{'t':>6} " + " ".join(f"{n[:11]:>11}" for n in names) + f" {'sum':>8}")
        for t in PART_TIMES:
            rows = [(p[3][t], p[0][t]["vis"] / p[0][-1000]["vis"]) for p in parsed if t in p[3] and t in p[0]]
            if rows:
                print(f"{t:>6} " + " ".join(f"{med([r.get(n, 0.0) * f for r, f in rows]):>11.4f}" for n in names) +
                      f" {med([sum(r.values()) * f for r, f in rows]):>8.4f}")


# ------------------------------------------------------------------------------------------------ area

SOURCES = ("start", "arc", "belt", "other", "ocean")
SOURCE_NAMES = {
    "start": "the start's continents (no ocean basement)",
    "arc": "on ocean floor, island-arc crust (cb_origin's rule)",
    "belt": "on ocean floor, a collision belt",
    "other": "on ocean floor, the rest (margins, spread, fills)",
    "ocean": "ocean-labelled",
}
# The log factors of the end's area of crust >= 20 km over Earth's, per run (they add exactly per run, so their means
# add): A = V / T. Two groupings of the same five terms (see main_area).
FACTORS = ("start_area", "start_volume", "run_own", "taken_in", "start_thinning", "thick_to_start_class", "mix")


def area_run(run, earth):
    import thinning as th  # noqa: E402 (needs scipy)
    ser = th.sea_series(run)
    a, _ = th.read(run, -1000)
    n0 = len(a)  # every sample visible at the start (cb_t vis 100000)
    out = {}
    for t in AREA_TIMES:
        try:
            a, pp = th.read(run, t)
        except FileNotFoundError:
            continue
        hid = (a[:, 10].astype(np.int64) & 1) > 0
        assert not hid.any(), "the maps hold visible samples only"
        sea = th.sea_at(ser, t)
        T = a[:, 3] / 1000.0
        z = a[:, 2]
        cont = a[:, 4] > 0.5
        base = np.where(cont, np.clip(np.minimum(pp[:, 7], pp[:, 5] + pp[:, 6]), 0.0, None), 0.0) / 1000.0
        conv = cont & (base > 1.0)
        # Island-arc crust as the crust budget's cb_origin reads it (crust_budget.rs `origin`): at least 1,000 m of
        # island-arc magma and more than of continental-arc magma.
        island = (a[:, 7] >= 1000.0) & (a[:, 7] > a[:, 8])
        src = {"start": cont & ~conv, "arc": conv & island}
        src["belt"] = conv & ~island & (a[:, 5] > 1.5)
        src["other"] = conv & ~island & ~(a[:, 5] > 1.5)
        src["ocean"] = ~cont
        c = T >= T_MIN
        own = np.where(cont, T - base, 0.0)
        r = {"sea": sea, "vis": len(a) / n0, "c20": c.sum() / n0, "c20_of_vis": float(c.mean()),
             "label": cont.sum() / n0, "label_of_vis": float(cont.mean()), "T20": float(T[c].mean()),
             "vol20": float(T[c].sum() / n0), "own20": float(own[c].sum() / n0),
             "base20": float(base[c].sum() / n0), "ocean20": float(T[c & ~cont].sum() / n0),
             "own_all": float(own.sum() / n0), "b35_50": float(((T >= 35) & (T < 50)).sum() / n0),
             "b30_35": float(((T >= 30) & (T < 35)).sum() / n0), "start_under20": float((src["start"] & ~c).sum() / n0),
             # The start's crust by when it formed: the start's continental samples carry formed_myr 0 (state.rs).
             "c20_formed0": float((c & cont & (pp[:, 8] == 0.0)).sum() / n0)}
        for k, m in src.items():
            km = c & m
            r[f"c20_{k}"] = float(km.sum() / n0)
            r[f"T_{k}"] = float(T[km].mean()) if km.any() else float("nan")
            r[f"dry_{k}"] = float((z[km] > sea).mean()) if km.any() else float("nan")
            r[f"zs_{k}"] = float(np.median(z[km] - sea)) if km.any() else float("nan")
            r[f"arc_p50_{k}"] = float(np.median(a[km, 7])) if km.any() else float("nan")
        out[t] = r
    # The log factors at the end, against Earth's crust >= 20 km (A_E, V_E, T_E: CRUST1.0, earth.json).
    a_e, t_e = earth["cont20"]["share"], earth["cont20"]["T_mean"]
    v_e = a_e * t_e
    r0, r1 = out[-1000], out[0]
    v0, t0 = r0["vol20"], r0["T20"]
    v1, t1, own1, tsc = r1["vol20"], r1["T20"], r1["own20"], r1["T_start"]
    out["factors"] = {
        "total": float(np.log(r1["c20"] / a_e)),
        "start_area": float(np.log((v0 / t0) / a_e)),
        "start_volume": float(np.log(v0 / v_e)),
        "run_own": float(np.log(own1 / v0)),
        "taken_in": float(np.log(v1 / own1)),
        "start_thinning": float(np.log(t0 / tsc)),
        "thick_to_start_class": float(np.log(t_e / tsc)),
        "mix": float(np.log(tsc / t1)),
    }
    # The interiors no orogeny raised, at the end: over all their samples and their land only (thinning.py's classes).
    a, pp = th.read(run, 0)
    cls, _, _ = th.classify(a, pp)
    sea = th.sea_at(ser, 0)
    nv = cls["never"]
    zs = a[nv, 2] - sea
    T = a[nv, 3] / 1000.0
    land = zs > 0
    out["never"] = {"share": float(nv.sum() / n0), "T_p50": float(np.median(T)), "zs_p50": float(np.median(zs)),
                    "land": float(land.mean()), "T_p50_land": float(np.median(T[land])) if land.any() else float("nan"),
                    "zs_p50_land": float(np.median(zs[land])) if land.any() else float("nan"),
                    "z_p50": float(np.median(a[nv, 2]))}
    return out


def main_area(earth_json, args):
    e = json.loads(Path(earth_json).read_text().strip().splitlines()[-1])
    for arg in args:
        rs = runs_of(arg)
        res = [area_run(r, e) for r in rs]
        print(f"== area {arg} ({len(rs)} runs): the crust >= 20 km as a share of the planet, by source; medians")
        cols = ["sea", "vis", "label", "label_of_vis", "c20", "c20_of_vis", "T20", "vol20", "own20", "base20",
                "ocean20", "own_all", "b30_35", "b35_50", "start_under20", "c20_formed0"] + \
            [f"c20_{k}" for k in SOURCES] + [f"T_{k}" for k in SOURCES]
        print(f"{'t':>6} " + " ".join(f"{c[:11]:>11}" for c in cols))
        for t in AREA_TIMES:
            rows = [r[t] for r in res if t in r]
            if rows:
                print(f"{t:>6} " + " ".join(f"{med([x[c] for x in rows]):>11.4g}" for c in cols))
        print("-- by source at -500 and 0 Myr (medians): share of the planet, mean T km, dry share, z - S p50 m, "
              "island-arc magma p50 m")
        for t in (-500, 0):
            for k in SOURCES:
                rows = [r[t] for r in res if t in r]
                print(f"  t{t} {SOURCE_NAMES[k]:52s} " + " ".join(
                    f"{nm} {med([x[f'{nm}_{k}'] for x in rows]):.4g}" for nm in ("c20", "T", "dry", "zs", "arc_p50")))
        print("-- means: " + " ".join(f"t{t}:c20 {mn([r[t]['c20'] for r in res if t in r]):.4f}" for t in
                                     (-1000, -950, -900, -800, -700, -500, 0)))
        f = {k: mn([r["factors"][k] for r in res]) for k in ("total",) + FACTORS}
        tot = f["total"]
        print(f"-- log factors of the end's area over Earth's, per run then means (total {tot:.4f}, "
              f"x{np.exp(tot):.3f}); each term's share of the total in brackets")
        for name, keys in (("area first", ("start_area", "run_own", "taken_in", "start_thinning", "mix")),
                           ("volume and thickness", ("start_volume", "run_own", "taken_in", "thick_to_start_class",
                                                     "mix"))):
            s = sum(f[k] for k in keys)
            print(f"   {name}: " + " ".join(f"{k} {f[k]:+.4f} ({f[k] / tot:+.2f})" for k in keys) +
                  f"; sum {s:.4f}")
        print("-- the interiors no orogeny raised, the end (all samples; land only): " +
              " ".join(f"{k} {med([r['never'][k] for r in res]):.4g}" for k in res[0]["never"]))


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
    # The solid surface (ice or rock under the water) over the sea: what the sea fills. For height against
    # thickness the ice sheets are also taken out (the model's surface carries no ice sheets).
    h = solid
    out["coarse"] = coarse(T, h, w, 0.0)
    out["coarse_no_sheets"] = coarse(T[~sheets], h[~sheets], w[~sheets], 0.0)
    s, d, m = profile(T, h, w, 0.0)
    out["profile"] = {"s": s.tolist(), "d": d.tolist(), "m": m.tolist()}
    s, d, m = profile(T[~sheets], h[~sheets], w[~sheets], 0.0)
    out["profile_no_sheets"] = {"s": s.tolist(), "d": d.tolist(), "m": m.tolist()}
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
    for key, keep in (("p50_by_km", np.ones(T.shape, bool)), ("p50_by_km_no_sheets", ~sheets)):
        p50 = []
        for a in np.arange(20.0, 60.0, 1.0):
            k = keep & (T >= a) & (T < a + 1)
            p50.append(wq(h[k], w[k], 0.5) if k.any() else float("nan"))
        out[key] = dict(zip([float(a) for a in np.arange(20.0, 60.0, 1.0)], p50))
    # The cratons' boxes: the land only, and every cell of crust at least 20 km thick (wet ones too: Hudson Bay, the
    # Baltic, the White Sea), the solid surface over the sea.
    land = (top - solid <= 0.0) & (top > 0.0)
    for key, sel in (("cratons", land), ("cratons_all20", c)):
        cr = {}
        for name, (la0, la1, lo0, lo1) in CRATONS:
            k = sel & (lat >= la0) & (lat <= la1) & (lon >= lo0) & (lon <= lo1)
            cr[name] = {"T_p50": wq(T[k], w[k], 0.5), "T_mean": wmean(T[k], w[k]), "h_mean": wmean(h[k], w[k]),
                        "h_p50": wq(h[k], w[k], 0.5), "h_p25": wq(h[k], w[k], 0.25), "h_p75": wq(h[k], w[k], 0.75),
                        "dry": float((w[k] * (h[k] > 0)).sum() / w[k].sum())}
        out[key] = cr
    # The sea's fill on Earth's own surface, the check that the factors start at Earth's sea.
    wet = solid < 0
    W = float((w * np.where(wet, -solid, 0.0)).sum() / w.sum())
    out["water_m"] = W
    out["fill"] = {"f": float(w[c].sum() / w.sum()), "hc": hist(h[c], w[c]).tolist(), "ho": hist(h[~c], w[~c]).tolist(),
                   "W": W}
    out["fill_check_m"] = fill(W, out["fill"]["f"], np.array(out["fill"]["hc"]), np.array(out["fill"]["ho"]))
    # The same fill with Earth's histograms and water but another area of crust at least 20 km thick: the start's.
    out["fill_at_start_area_m"] = fill(W, START_AREA, np.array(out["fill"]["hc"]), np.array(out["fill"]["ho"]))
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
    print("-- the ice sheets out, by crust thickness:")
    for r in e["coarse_no_sheets"]:
        print(f"  {r['band']:>7} share {r['share']:.4f} dry {r.get('dry', float('nan')):.3f} "
              f"mean {r.get('mean', float('nan')):8.0f} p50 {r.get('p50', float('nan')):8.0f}")
    print("-- p50 height by thickness:", " ".join(f"{k:g}:{v:.0f}" for k, v in e["p50_by_km"].items()))
    print("-- the same, the ice sheets out:", " ".join(f"{k:g}:{v:.0f}" for k, v in e["p50_by_km_no_sheets"].items()))
    for key, what in (("cratons", "land only"), ("cratons_all20", "every cell of crust >= 20 km, wet ones too")):
        print(f"-- cratons ({what}):")
        for k, v in e[key].items():
            print(f"  {k:>28} " + " ".join(f"{a} {b:.3g}" for a, b in v.items()))
    print(f"-- water over the planet {e['water_m']:.0f} m; Earth's fill on its own histograms {e['fill_check_m']:.0f} m "
          f"(a binning check: W is read from the same surface); at the start's area {START_AREA} of crust >= 20 km "
          f"{e['fill_at_start_area_m']:.0f} m")
    print("-- SRTM15+ hypsometry, all:", json.dumps(e["hypso_all"]))
    print("-- SRTM15+ hypsometry, without Antarctica and Greenland:", json.dumps(e["hypso_no_sheets"]))


# ------------------------------------------------------------------------------------------------ model

def steering_offset_m(run):
    """The steering's sea offset at the end, m, emulated from the record (steering.rs `step`): over the tail, each
    plate step moves the sea by -w x rate x dt x clamp(e / LAND_FULL_RATE), w the shape over the tail, e the land
    share's distance outside its band, bounded by sea_max_m. The run's own offset is in its manifest's notes
    (notes.climate.steering.sea_offset_m), which the pool's folders do not keep; the land here is the record's (plate
    samples above the sea over the visible ones), not the climate's grid, so this is an estimate."""
    text = (run / "summary.txt").read_text()
    if "preset " not in text:
        return float("nan")
    line = text.split("preset ", 1)[1].splitlines()[0]
    st = json.JSONDecoder().raw_decode(line[line.index("{"):])[0].get("steering")
    if not st or not st.get("enabled"):
        return float("nan")
    rec = json.loads((run / "rec.json").read_text())
    steps = rec["steps"]
    t0, t1 = float(rec["t0_myr"]), float(steps[-1]["t"])
    tail = st["tail_fraction"] * (t1 - t0)
    lo, hi = st["land_min"], st["land_max"]
    shape = {"smoothstep": lambda x: x * x * (3.0 - 2.0 * x), "linear": lambda x: x}[st["shape"]]
    off, prev = 0.0, t0
    for s_ in steps:
        t = float(s_["t"])
        dt, prev = t - prev, t
        x = (t - (t1 - tail)) / tail
        if x <= 0.0 or dt <= 0.0:
            continue
        land = s_["land"] / max(s_["visible"], 1)
        e = lo - land if land < lo else (hi - land if land > hi else 0.0)
        off += -shape(min(x, 1.0)) * st["sea_rate_m_myr"] * dt * max(-1.0, min(1.0, e / 0.05))  # STE-010's 0.05
        off = max(-st["sea_max_m"], min(st["sea_max_m"], off))
    return off


def model_run(run):
    import contextlib
    import io
    with contextlib.redirect_stdout(io.StringIO()):
        f, g, e = cc.load_dump(run / "fields")
    h_, w_, r_ = e["h"], e["w"], e["r"]
    lat_r, ok_r = cc.lat_of_rows(h_, r_, w_ * r_)
    ok = np.repeat(ok_r[:, None], w_, 1)
    sea = float(g["globals"]["climate/sea_level_m"])
    hs = f["surface/elevation_m"][ok]
    T = f["plates/thickness_m"][ok] / 1000.0
    # A cell with no elevation or thickness is dropped and counted, not read as a 0 m, 0 km column.
    good = np.isfinite(hs) & np.isfinite(T)
    hs, T = hs[good], T[good]
    cont = np.nan_to_num(f["plates/crust_type"][ok][good]) > 0.5
    ice = np.nan_to_num(f["climate/ice_thickness_m"][ok][good])
    zc = (f["elevation"][ok][good] - cc.SEA_Y) * cc.M_PER_BLOCK_V  # the census's ground over the sea
    w = np.ones(hs.shape)  # equal-area rows
    out = {"run": run.name, "sea": sea, "land_census": float((zc > 0).mean()), "dropped": int((~good).sum())}
    out["coarse"] = coarse(T, hs, w, sea, label=cont.astype(float))
    # The surface with the sea at the datum: a drowned column's Airy height S + (z - S) / WATER_LOADED (ISO-078's
    # settling inverted), reloaded under a sea at 0 where it stays below it; a dry column keeps its height.
    za = np.where(hs < sea, sea + (hs - sea) / WATER_LOADED, hs)
    h0 = np.where(za < 0.0, za * WATER_LOADED, za)
    s, d, m = profile(T, hs, w, sea)
    _, d0, m0 = profile(T, h0, w, 0.0)
    _, d0l, m0l = profile(T, hs, w, 0.0)
    out["profile"] = {"s": s.tolist(), "d": d.tolist(), "m": m.tolist(), "d0": d0.tolist(), "m0": m0.tolist(),
                      "d0_loaded": d0l.tolist(), "m0_loaded": m0l.tolist()}
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
    # p50 by 1-km band [a, a + 1): over the sea, over the datum as it stands, and with the sea at the datum (h0).
    for key, x in (("p50_by_km", hs - sea), ("p50_datum_by_km", hs), ("p50_unloaded_by_km", h0)):
        p50 = {}
        for a in np.arange(20.0, 60.0, 1.0):
            k = (T >= a) & (T < a + 1)
            p50[float(a)] = float(np.median(x[k])) if k.sum() > 20 else float("nan")
        out[key] = p50
    W = WATER_LAYER_M - float(ice.mean()) * RHO_ICE_OVER_SEA
    out["fill"] = {"f": float(c.mean()), "hc": hist(hs[c], w[c]).tolist(), "ho": hist(hs[~c], w[~c]).tolist(), "W": W}
    out["fill_self_m"] = fill(W, out["fill"]["f"], np.array(out["fill"]["hc"]), np.array(out["fill"]["ho"]))
    out["steering_m"] = steering_offset_m(run)
    # The ocean floor and margins under 20 km: the mean height of the crust under 10 km, and of 10 to 20 km.
    for lo, hi in ((0.0, 10.0), (10.0, 20.0)):
        k = (T >= lo) & (T < hi)
        out[f"under20_{lo:g}_{hi:g}"] = {"share": float(k.mean()), "h_mean": float(hs[k].mean()) if k.any() else
                                         float("nan")}
    out["lat_abs_mean_cont"] = float(np.abs(lat[c]).mean())
    return out


def split(sM, xM, xM0, sE, xE):
    """Thickness, line and sea terms of sum s x between the model and Earth. A band with no crust on one side has no
    value there (NaN), counted as 0 since its share is 0; a band the model fills and Earth does not would put its whole
    value into the line term, so it is refused."""
    assert not ((sE <= 0.0) & (sM > 0.0)).any(), "a band the model fills and Earth does not"
    xE = np.where(sE > 0.0, xE, 0.0)
    xM, xM0 = np.where(sM > 0.0, xM, 0.0), np.where(sM > 0.0, xM0, 0.0)
    assert np.isfinite(xE).all() and np.isfinite(xM).all() and np.isfinite(xM0).all()
    thick = float(((sM - sE) * xE).sum())
    line = float((sM * (xM0 - xE)).sum())
    sea = float((sM * (xM - xM0)).sum())
    out = {"model": float((sM * xM).sum()), "earth": float((sE * xE).sum()), "thickness": thick, "line": line,
           "sea": sea}
    assert abs(out["model"] - out["earth"] - thick - line - sea) < 1e-6 * max(1.0, abs(out["model"]))
    return out


TERMS = ("model", "earth", "thickness", "line", "sea")


def print_terms(label, rows, keys, fmt="{:.4g}"):
    """Means (which add, since each run's terms add) and medians with their residual (which do not)."""
    means = {k: mn([x[k] for x in rows]) for k in keys}
    meds = {k: med([x[k] for x in rows]) for k in keys}
    print(f"-- {label}, means: " + " ".join(f"{k} {fmt.format(means[k])}" for k in keys))
    print("   medians: " + " ".join(f"{k} {fmt.format(meds[k])}" for k in keys), end="")
    if keys == TERMS:
        resid = meds["model"] - meds["earth"] - meds["thickness"] - meds["line"] - meds["sea"]
        print(f" (medians' residual {fmt.format(resid)})")
    else:
        print()


def main_model(earth_json, args):
    e = json.loads(Path(earth_json).read_text().strip().splitlines()[-1])
    earths = {"Earth": e["profile"], "Earth without its ice sheets": e["profile_no_sheets"]}
    for arg in args:
        rs = runs_of(arg)
        res = [model_run(r) for r in rs]
        print(f"== model {arg} ({len(rs)} runs), final frame, medians unless said")
        print(f"  sea {med([r['sea'] for r in res]):.0f} m (mean {mn([r['sea'] for r in res]):.0f}); census land "
              f"{med([r['land_census'] for r in res]):.3f}; cells dropped (no elevation or thickness) "
              f"{sum(r['dropped'] for r in res)}")
        print("-- by crust thickness (share of the planet, continental label share, dry share, surface over the sea)")
        for i, (a, b) in enumerate(COARSE):
            rows = [r["coarse"][i] for r in res]
            print(f"  {rows[0]['band']:>7} share {med([x['share'] for x in rows]):.4f} cont "
                  f"{med([x.get('cont') for x in rows]):.2f} dry {med([x.get('dry') for x in rows]):.3f} mean "
                  f"{med([x.get('mean') for x in rows]):8.0f} p50 {med([x.get('p50') for x in rows]):8.0f}")
        for key in ("cont20", "label", "under20_0_10", "under20_10_20"):
            ks = list(res[0][key])
            print(f"-- {key}: " + " ".join(f"{k} {med([r[key][k] for r in res]):.4g}" for k in ks))
        for key in ("hypso", "hypso_census"):
            ks = list(res[0][key])
            print(f"-- {key}: " + " ".join(f"{k} {med([r[key][k] for r in res]):.4g}" for k in ks))
        print("-- p50 height by 1-km band [T, T+1), m: the model over its sea / over the datum as it stands / with the "
              "sea at the datum (unloaded); PLT-281's line at the band's centre over the datum; Earth over its sea "
              "(ice sheets in / out):")
        for a in np.arange(20.0, 60.0, 2.0):
            line = Z0_M + (a + 0.5 - T_N_KM) * 1000.0 / AIRY_F
            print(f"  {a:g}: {med([r['p50_by_km'][a] for r in res]):.0f} / "
                  f"{med([r['p50_datum_by_km'][a] for r in res]):.0f} / "
                  f"{med([r['p50_unloaded_by_km'][a] for r in res]):.0f}; line {line:.0f}; Earth "
                  f"{e['p50_by_km'][str(a)]:.0f} / {e['p50_by_km_no_sheets'][str(a)]:.0f}")
        # The split, per run (the terms add exactly per run); means and medians over the runs.
        for ename, ep in earths.items():
            sE = np.array(ep["s"])
            dE, mE = np.array(ep["d"]), np.array(ep["m"])
            for read, (kd, km) in (("the line read with the sea at the datum (unloaded)", ("d0", "m0")),
                                   ("the line read on the surface as loaded", ("d0_loaded", "m0_loaded"))):
                dry, height = [], []
                for r in res:
                    p = r["profile"]
                    sM = np.array(p["s"])
                    dry.append(split(sM, np.array(p["d"]), np.array(p[kd]), sE, dE))
                    height.append(split(sM, np.array(p["m"]), np.array(p[km]), sE, mE))
                print(f"-- split against {ename}, {read}:")
                print_terms("the crust >= 20 km's dry share", dry, TERMS)
                print_terms("the crust >= 20 km's mean height over the sea, m", height, TERMS, "{:.0f}")
        # Land: area x dry share.
        AE, DE = e["cont20"]["share"], e["cont20"]["dry"]
        la = []
        for r in res:
            AM, DM = r["cont20"]["share"], r["cont20"]["dry"]
            la.append({"area": (AM - AE) * DE, "dry": AM * (DM - DE), "land20_M": AM * DM, "land20_E": AE * DE})
        print_terms("land on crust >= 20 km, A x D (area + dry = land20_M - land20_E)", la, tuple(la[0]))
        # The sea against Earth's fill on four factors, and the run's own residual.
        fe = e["fill"]
        base = {"W": fe["W"], "f": fe["f"], "hc": np.array(fe["hc"]), "ho": np.array(fe["ho"])}
        fill_e = fill(base["W"], base["f"], base["hc"], base["ho"])
        sh = []
        for r in res:
            fm = r["fill"]
            arm = {"W": fm["W"], "f": fm["f"], "hc": np.array(fm["hc"]), "ho": np.array(fm["ho"])}
            part = shapley(base, arm)
            part["fill"] = fill(arm["W"], arm["f"], arm["hc"], arm["ho"])
            part["total"] = part["fill"] - fill_e
            part["residual"] = r["sea"] - r["fill_self_m"]
            part["steering_est"] = r["steering_m"]
            part["residual_rest"] = part["residual"] - r["steering_m"]
            part["run_sea"] = r["sea"]
            part["run_less_earth_fill"] = r["sea"] - fill_e
            # Earth's histograms and water at the model's own area of crust >= 20 km, and at its volume spread at
            # Earth's mean thickness.
            part["earth_at_model_area"] = fill(base["W"], arm["f"], base["hc"], base["ho"]) - fill_e
            f_vol = r["cont20"]["vol_planet_km"] / e["cont20"]["T_mean"]
            part["f_at_earth_T"] = f_vol
            part["earth_at_model_volume"] = fill(base["W"], f_vol, base["hc"], base["ho"]) - fill_e
            sh.append(part)
        keys = ("W", "f", "hc", "ho", "total", "residual", "steering_est", "residual_rest", "run_less_earth_fill",
                "run_sea", "fill",
                "earth_at_model_area", "f_at_earth_T", "earth_at_model_volume")
        print(f"-- the sea against Earth's fill ({fill_e:.0f} m on its own histograms), Shapley, m: total = the "
              "model's fill less Earth's; residual = the run's sea less its own fill (the steering's nudge, the "
              "open-sea rule); W + f + hc + ho + residual = run_less_earth_fill per run")
        print_terms("the sea's terms", sh, keys, "{:.4g}")
        print("-- per run (sea / own fill / the steering's offset, emulated): " + " ".join(
            f"{r['run']}:{r['sea']:.0f}/{r['fill_self_m']:.0f}/{r['steering_m']:.0f}" for r in res))


if __name__ == "__main__":
    mode = sys.argv[1]
    if mode == "series":
        main_series(sys.argv[2:])
    elif mode == "area":
        main_area(sys.argv[2], sys.argv[3:])
    elif mode == "earth":
        main_earth(sys.argv[2], sys.argv[3])
    elif mode == "model":
        main_model(sys.argv[2], sys.argv[3:])
    else:
        sys.exit(__doc__)
