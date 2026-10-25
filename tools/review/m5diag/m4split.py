#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""M4 (land above 2 km by rock) split by what holds it up: m4split.py <out.jsonl> [workers] [runs]

For each run of the arm in M5DIAG_RUNS (m5fields.RUNS). M4 is the census's (measure_v1census.py:
the L1 grid's `elevation`, z = (elevation - 63) x 35 m over the sea; the same in v1census versions
1 and 2). Two parts, one JSON object per run:

1. The final frame (fields/, the L1 grid). Each cell of the high ground (land above 2 km) gets a
   class by the fields' orogeny tag, its age, the regime and the crust's thickness T:
     coll_young     a collision belt raised in the last 10 Myr (collapse_quiet_myr, PLT-259)
     coll_10_50, coll_50_150, coll_150p   older collision belts by the age of their stamp, Myr
     andean_young, andean_old             an Andean margin's uplift, under and over 10 Myr
     arc_untagged   no orogeny tag, on an arc, back-arc or hotspot (regime 3, 4, 7)
     plateau_untagged   no orogeny tag, crust 49.5 km or thicker (the founder's and the lower
                    crust's fill to T_c = 50 km, PLT-256, PLT-581; an overlap stack)
     oceanic, other the rest
   The coordinator's five groups are coll_young; the older belts; arcs and Andean margins; the
   untagged plateau; oceanic and other. For each class: its part of M4, split by ice (ice >= 1 m)
   and by the crust's Airy height over the sea (the guard's line, measure_guard.py: z_0 525 m +
   (T - 35.1 km) / 7.02, PLT-281, no eclogite root), and the height's terms: the Airy height over
   the sea, the plate column's height over its Airy height (the settling's flexure F and what
   moved it after the settling) and the census's height over the plate's (the surface stage on
   the L1 grid), which add up to the census height. Also the thickness bands over the land (the
   crust budget's and HYP-040's, with Earth's rock heights by band), the rock height against T
   on the ice-free land (Earth's line HYP-040: -2,559 m + 81 m/km T), M4 counted three other
   ways (every column at its Airy height, at Earth's line, without its height over Airy) and the
   plate's height against its Airy height averaged over discs of 0 to 400 km (how wide a region
   the plate holds the high ground up over).
2. The crust-budget maps (maps/t*.bin, the visible plate samples every 50 Myr, the plate model's
   elevation over the step's sea from rec.json): continental samples by orogeny (Himalayan,
   Andean, untagged under and over 49.5 km) and the age of the stamp, pooled over the snapshots
   from -800 Myr on: the height over the sea, the share above 2 km, the thickness, the share
   whose Airy height is above 2 km and the height over Airy, by age (how fast a belt comes down
   once nothing raises it); and the high ground at each snapshot by the same groups."""
import contextlib
import io
import json
import sys
from multiprocessing import Pool
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
from m5fields import FIELDS, RUNS  # noqa: E402
from m5lib import cc  # noqa: E402

Z_0 = 525.0  # measure_guard.py: the normal column's Airy height over the datum, m
T_N = 35_100.0  # the normal continental column, m
FACTOR = 3300.0 / (3300.0 - 2830.0)  # COLUMN_ISOSTATIC_FACTOR, PLT-281
CLASSES = ["coll_young", "coll_10_50", "coll_50_150", "coll_150p", "andean_young", "andean_old",
           "arc_untagged", "plateau_untagged", "oceanic", "other"]
GROUPS = {"young_belts": ["coll_young"], "older_belts": ["coll_10_50", "coll_50_150", "coll_150p"],
          "arcs_andean": ["andean_young", "andean_old", "arc_untagged"], "plateau_untagged": ["plateau_untagged"],
          "other": ["oceanic", "other"]}
# HYP-040's bands and Earth's rock height p50 in them (CRUST1.0), and finer bands around T_c.
BANDS = [(0, 30), (30, 35), (35, 40), (40, 45), (45, 55), (55, 70), (70, 1e9)]
EARTH_BAND_SHARE = {"30-35": 0.197, "35-40": 0.367, "40-45": 0.274, "45-55": 0.097, "55-70": 0.019,
                    "70-1e+09": 0.006}
EARTH_BAND_P50 = {"35-45": 380.0, "45-55": 940.0, "55-70": 4083.0}
FINE = [(44, 46), (46, 48), (48, 49.5), (49.5, 50.5), (50.5, 52), (52, 55), (55, 60), (60, 70)]
DISCS_KM = [0, 50, 100, 200, 400]
AGE_EDGES = [0, 5, 10, 20, 30, 50, 75, 100, 150, 200, 300, 500, 1e9]
MAP_FROM = -800


def airy(t_m):
    """The guard's Airy height of a column of thickness t_m (m) over the datum, m."""
    return Z_0 + (t_m - T_N) / FACTOR


def classify(oro, age, th_km, cont, reg):
    c = np.full(oro.size, CLASSES.index("other"))
    c[~cont] = CLASSES.index("oceanic")
    u = cont & (oro == 0)
    c[u & (th_km >= 49.5)] = CLASSES.index("plateau_untagged")
    c[u & np.isin(reg, (3, 4, 7))] = CLASSES.index("arc_untagged")
    c[(oro == 1) & (age < 10)] = CLASSES.index("andean_young")
    c[(oro == 1) & (age >= 10)] = CLASSES.index("andean_old")
    c[(oro == 2) & (age >= 150)] = CLASSES.index("coll_150p")
    c[(oro == 2) & (age >= 50) & (age < 150)] = CLASSES.index("coll_50_150")
    c[(oro == 2) & (age >= 10) & (age < 50)] = CLASSES.index("coll_10_50")
    c[(oro == 2) & (age < 10)] = CLASSES.index("coll_young")
    return c


def med(x):
    return float(np.median(x)) if x.size else None


def q(x, p):
    return float(np.quantile(x, p)) if x.size else None


def final_frame(run):
    with contextlib.redirect_stdout(io.StringIO()):
        f, g, e = cc.load_dump(run / "fields")
    h, w, r = e["h"], e["w"], e["r"]
    lat_r, ok_r = cc.lat_of_rows(h, r, w * r)
    lat2 = np.repeat(lat_r[:, None], w, 1)
    ok = np.repeat(ok_r[:, None], w, 1)
    lon2 = np.repeat((360.0 * np.arange(w) / w)[None, :], h, 0)
    p = cc.xyz(lat2[ok], lon2[ok])
    sea = float(g["globals"]["climate/sea_level_m"])
    z = (f["elevation"][ok] - cc.SEA_Y) * cc.M_PER_BLOCK_V
    zp = np.nan_to_num(f["plates/elevation_m"][ok]) - sea
    T = np.nan_to_num(f["plates/thickness_m"][ok])
    cont = np.nan_to_num(f["plates/crust_type"][ok]) > 0.5
    oro = np.rint(np.nan_to_num(f["plates/orogeny_type"][ok])).astype(int)
    age = np.nan_to_num(f["plates/orogeny_age_myr"][ok], nan=1e9)
    reg = np.rint(np.nan_to_num(f["regime"][ok])).astype(int)
    ice = np.nan_to_num(f["climate/ice_thickness_m"][ok])
    land = z > 0
    hi = land & (z > 2000)
    L = max(int(land.sum()), 1)
    a_sea = airy(T) - sea
    exc = zp - a_sea
    srf = z - zp
    iced = ice >= 1.0
    cls = classify(oro, age, T / 1000.0, cont, reg)
    out = {"run": run.name, "start": run.name.rsplit("-", 1)[0], "seed": int(run.name.rsplit("-", 1)[1]),
           "sea_m": sea, "M4": float(hi.sum() / L), "cached_M4": None,
           "M4_plate": float(((zp > 2000) & (zp > 0)).sum() / max(int((zp > 0).sum()), 1)),
           "M4_ice_kept_out": float((hi & ~iced).sum() / max(int((land & ~iced).sum()), 1)),
           "hi_iced_share": float((hi & iced).sum() / max(int(hi.sum()), 1)),
           "land_iced_share": float((land & iced).sum() / L),
           "M4_airy": float((land & (a_sea > 2000)).sum() / L),
           "M4_earth_line": float((land & (-2559.0 + 81.0 * T / 1000.0 > 2000)).sum() / L),
           "M4_no_excess": float((land & (z - np.maximum(exc, 0.0) > 2000)).sum() / L),
           "M4_airy_hi_cells": float((hi & (a_sea > 2000)).sum() / L)}
    try:
        out["cached_M4"] = json.loads((run / "measures" / "v1census.json").read_text())["values"]["above_2km"]
    except (OSError, KeyError, ValueError):
        pass
    by = {}
    for i, k in enumerate(CLASSES):
        m = hi & (cls == i)
        n = int(m.sum())
        lm = land & (cls == i)
        o = {"m4": n / L, "land": float(lm.sum() / L),
             "m4_noice": float((m & ~iced).sum() / L), "m4_iced": float((m & iced).sum() / L),
             "m4_airy_ge2km": float((m & (a_sea >= 2000)).sum() / L),
             "m4_airy_lt2km": float((m & (a_sea < 2000)).sum() / L),
             "m4_airy_lt2km_noice": float((m & (a_sea < 2000) & ~iced).sum() / L)}
        if n:
            o.update({"T_km_p50": med(T[m]) / 1000.0, "T_km_p10": q(T[m], 0.1) / 1000.0,
                      "T_km_p90": q(T[m], 0.9) / 1000.0,
                      "z_mean": float(z[m].mean()), "airy_mean": float(a_sea[m].mean()),
                      "exc_mean": float(exc[m].mean()), "srf_mean": float(srf[m].mean()),
                      "airy_p50": med(a_sea[m]), "exc_p50": med(exc[m]), "exc_p90": q(exc[m], 0.9),
                      "srf_p50": med(srf[m]), "ice_p50_iced": med(ice[m & iced]),
                      "age_p50": med(age[m]) if k.startswith(("coll", "andean")) else None,
                      "lat_abs_p50": med(np.abs(lat2[ok][m]))})
        if lm.any():
            o.update({"land_z_p50": med(z[lm]), "land_T_km_p50": med(T[lm]) / 1000.0,
                      "land_exc_p50": med(exc[lm]), "land_airy_p50": med(a_sea[lm])})
        by[k] = o
    out["by_class"] = by
    # The thickness bands over the land.
    bands = {}
    for a_, b_ in BANDS + FINE + [(35, 45)]:
        m = land & (T >= a_ * 1000) & (T < b_ * 1000)
        mf = m & ~iced
        bands[f"{a_:g}-{b_:g}"] = {"land": float(m.sum() / L), "m4": float((m & hi).sum() / L),
                                   "z_p50": med(z[m]), "z_p50_icefree": med(z[mf]),
                                   "airy_p50": med(a_sea[m]), "exc_p50": med(exc[m]),
                                   "untagged": float((m & (oro == 0)).sum() / max(int(m.sum()), 1)),
                                   "coll": float((m & (oro == 2)).sum() / max(int(m.sum()), 1))}
    out["bands"] = bands
    fl = land & ~iced & cont
    if fl.sum() > 10:
        sl, ic = np.polyfit(T[fl] / 1000.0, z[fl], 1)
        out["icefree_line"] = {"slope_m_per_km": float(sl), "intercept_m": float(ic)}
    # The plate's height against the Airy height averaged over discs (continental cells within R
    # of each high cell, ocean cells at their own plate height): how wide a region holds it up.
    hidx = np.nonzero(hi & cont)[0]
    tree = cKDTree(p)
    disc = {}
    for R in DISCS_KM:
        if R == 0:
            reg_a = a_sea[hidx]
        else:
            nb = tree.query_ball_point(p[hidx], cc.chord(R))
            ref = np.where(cont, a_sea, zp)
            reg_a = np.array([ref[j].mean() for j in nb])
        d = zp[hidx] - reg_a
        disc[str(R)] = {"plate_minus_regional_airy_p50": med(d), "abs_p50": med(np.abs(d)),
                        "p90": q(d, 0.9), "share_regional_airy_ge2km": float((reg_a >= 2000).mean()) if hidx.size else None}
    out["discs"] = disc
    return out


def maps_part(run):
    rec = json.loads((run / "rec.json").read_text())
    ts = np.array([s["t"] for s in rec["steps"]])
    seas = np.array([s.get("sea_m", 0.0) or 0.0 for s in rec["steps"]])
    pooled = {k: [] for k in ("grp", "age", "zs", "T", "exc", "asea")}
    snaps = []
    for path in sorted((run / "maps").glob("t*.bin")):
        t = float(path.stem[1:])
        a = np.fromfile(path, "<f4").reshape(-1, FIELDS).astype(np.float64)
        sea_t = float(seas[int(np.argmin(np.abs(ts - t)))])
        zs = a[:, 2] - sea_t
        T = a[:, 3]
        cont = a[:, 4] > 0.5
        oro = np.rint(a[:, 5]).astype(int)
        age = a[:, 6]
        grp = np.where(oro == 2, 2, np.where(oro == 1, 1, np.where(T >= 49_500, 3, 0)))
        grp[~cont] = -1
        land = zs > 0
        hi = land & (zs > 2000)
        nl = max(int(land.sum()), 1)
        s = {"t": t, "M4": float(hi.sum() / nl), "sea_m": sea_t}
        for gi, gn in ((2, "coll"), (1, "andean"), (3, "untagged_thick"), (0, "untagged_thin"), (-1, "oceanic")):
            m = hi & (grp == gi)
            s[gn] = float(m.sum() / nl)
            if gi in (1, 2):
                for a_, b_ in zip(AGE_EDGES[:-1], AGE_EDGES[1:]):
                    s[f"{gn}_{a_:g}_{b_:g}"] = float((m & (age >= a_) & (age < b_)).sum() / nl)
        s["airy_ge2km"] = float((hi & cont & (airy(T) - sea_t >= 2000)).sum() / nl)
        snaps.append(s)
        if t >= MAP_FROM:
            c = cont
            pooled["grp"].append(grp[c])
            pooled["age"].append(age[c])
            pooled["zs"].append(zs[c])
            pooled["T"].append(T[c])
            pooled["exc"].append((zs - (airy(T) - sea_t))[c])
            pooled["asea"].append((airy(T) - sea_t)[c])
    P = {k: np.concatenate(v) for k, v in pooled.items()}
    decay = {}
    for gi, gn in ((2, "coll"), (1, "andean"), (3, "untagged_thick"), (0, "untagged_thin")):
        rows = {}
        edges = AGE_EDGES if gi in (1, 2) else [0, 1e9]
        for a_, b_ in zip(edges[:-1], edges[1:]):
            m = P["grp"] == gi
            if gi in (1, 2):
                m = m & (P["age"] >= a_) & (P["age"] < b_)
            n = int(m.sum())
            if n == 0:
                continue
            rows[f"{a_:g}-{b_:g}"] = {"n": n, "zs_p50": med(P["zs"][m]), "zs_p90": q(P["zs"][m], 0.9),
                                      "dry": float((P["zs"][m] > 0).mean()),
                                      "above_2km": float((P["zs"][m] > 2000).mean()),
                                      "T_km_p50": med(P["T"][m]) / 1000.0,
                                      "T_km_p90": q(P["T"][m], 0.9) / 1000.0,
                                      "airy_ge2km": float((P["asea"][m] >= 2000).mean()),
                                      "exc_p50": med(P["exc"][m])}
        decay[gn] = rows
    return {"snaps": snaps, "decay": decay}


def one(run):
    out = final_frame(run)
    if (run / "maps" / "t0000.bin").exists():
        out.update(maps_part(run))
    return out


def main():
    dest = Path(sys.argv[1])
    workers = int(sys.argv[2]) if len(sys.argv) > 2 else 3
    runs = sorted(p for p in RUNS.glob("*-*") if p.is_dir() and (p / "fields" / "index.json").exists())
    if len(sys.argv) > 3:
        runs = [p for p in runs if p.name in sys.argv[3].split(",")]
    print(f"{len(runs)} runs", flush=True)
    with Pool(workers) as pool, dest.open("w") as fh:
        for res in pool.imap_unordered(one, runs):
            fh.write(json.dumps(res) + "\n")
            fh.flush()
            print(f"{res['run']}: M4 {res['M4']:.4f} (cached {res['cached_M4']}) airy {res['M4_airy']:.4f}"
                  f" earth-line {res['M4_earth_line']:.4f} no-excess {res['M4_no_excess']:.4f}", flush=True)
    print("M4SPLIT_DONE")


if __name__ == "__main__":
    main()
