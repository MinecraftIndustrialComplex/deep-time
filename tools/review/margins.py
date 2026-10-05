#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Continental margins in metres on a dumped L1 surface (Lambert rows), and where the continental crust
lies against the sea.

    margins.py planet <dump dir> <out.json>        one run (`deeptime dump-fields` of surface/elevation_m,
                                                  elevation, plates/crust_type; plates/thickness_m optional)
    margins.py pool <out.json> <run.json> ...      the transects of several runs pooled

MARGINS_FIELD=plates/elevation_m measures another dumped elevation against the surface's sea level. With
regime in the dump, transects are also summarised apart for passive and active margins (a trench, an
arc or a collision regime within 6 L1 cells of the coast cell is active; ACTIVE_REGIMES below).

Transects (the margins-and-spires study's definitions, build log 2026-10-03, so its ETOPO 2022 figures
compare): from every 3rd open-sea coast cell of land masses over 750 km2, |lat| <= 60 deg, seaward along
the downhill direction of the surface smoothed over 10 km, every 500 m out to 400 km: shelf break = the
first point deeper than 20 m whose 4 km gradient passes 1 deg; foot = the first point 1 km deeper than
the break whose gradient falls under 1 deg; slope 500-2,000 m = the angle between those isobaths; shore to
200 m. Earth (ETOPO 2022 at 2', 3,777 transects): slope 500-2,000 m median 2.1 deg (mean 3.1, p90 6.8),
break to foot 2.2 deg, shore to 200 m 25 km (mean 51), break depth 108 m, land 10 km inland 61 m.

Lengths are on the planet (block metric: a block is 300 m; on Lambert rows a cell is r*300*cos(lat) m
east-west and r*300/cos(lat) m north-south). The crust part: the share of continental cells below the
sea, by distance (km, on the planet, through the cells) from the nearest oceanic cell.
"""
import json
import math
import os
import sys

import numpy as np
from scipy import ndimage
from scipy.sparse import coo_matrix
from scipy.sparse.csgraph import dijkstra

STEP_M = 500.0
PROFILE_KM = [-50, -30, -20, -10, -5, -3, -1, 0, 1, 3, 5, 10, 15, 20, 30, 50, 75, 100, 150, 200]
SEAWARD_M = 400_000.0
BASE_M = 4_000.0
M_PER_BLOCK_H = 300.0
M_PER_BLOCK_V = 35.0
SEA_Y = 63
MAX_LAT = 60.0
STRIDE = 3
DIST_BINS_KM = [0, 10, 25, 50, 100, 200, 400, 1e9]
# An active margin: a trench, an arc or a collision orogen (plates/state.rs, mod regime) within 6 L1
# cells (about 30 km on the quick 16k planet, where the Earth-normalized 200 km from trench to arc is
# 25 km) of the coast cell. The back-arc (the overriding plate within the subduction distance, 800 km
# Earth-normalized) is not counted: its own-plate margins are rifted margins (back-arc basins, the far
# side of a narrow continent). Overrides: MARGINS_ACTIVE_REGIMES=2,3,4,5 MARGINS_ACTIVE_CELLS=3.
ACTIVE_REGIMES = [int(x) for x in os.environ.get("MARGINS_ACTIVE_REGIMES", "2,3,5").split(",")]
ACTIVE_CELLS = int(os.environ.get("MARGINS_ACTIVE_CELLS", "6"))


def angle(dh, dl):
    return math.degrees(math.atan2(dh, dl))


def analyse(prof_d, step):
    n = len(prof_d)
    k = max(1, int(round(BASE_M / step)))
    if n <= k + 2 or np.nanmax(prof_d) < 1000.0:
        return None
    g = np.full(n, np.nan)
    g[:-k] = (prof_d[k:] - prof_d[:-k]) / (k * step)
    gdeg = np.degrees(np.arctan(np.clip(g, -10, 10)))
    br = None
    for i in range(n - k):
        if prof_d[i] >= 20.0 and gdeg[i] > 1.0 and np.nanmax(prof_d[i:]) > max(1000.0, prof_d[i] + 500):
            br = i
            break
    if br is None:
        return None
    dbreak = float(prof_d[br])
    foot = None
    for i in range(br + 1, n - k):
        if prof_d[i] > dbreak + 1000.0 and gdeg[i] < 1.0:
            foot = i
            break
    if foot is None:
        foot = int(np.nanargmax(prof_d))
    if foot <= br:
        return None
    r = {
        "shelf_km": br * step / 1000.0,
        "break_m": dbreak,
        "slope_deg": angle(prof_d[foot] - dbreak, (foot - br) * step),
        "slope_km": (foot - br) * step / 1000.0,
        "steepest_deg": float(np.nanmax(gdeg[: max(foot, br + 1)])),
    }
    for iso in (200, 1000, 2000):
        ii = int(np.argmax(prof_d >= iso))
        if prof_d[ii] >= iso:
            r[f"to_{iso}m_km"] = ii * step / 1000.0
    i500 = int(np.argmax(prof_d >= 500.0))
    i2000 = int(np.argmax(prof_d >= 2000.0))
    if prof_d[i2000] >= 2000.0 and i2000 > i500:
        r["slope_500_2000_deg"] = angle(1500.0, (i2000 - i500) * step)
    return r


def summarise(rows):
    out = {"transects": len(rows)}
    keys = sorted({k for r in rows for k in r if isinstance(r[k], float)})
    for k in keys:
        v = np.array([r[k] for r in rows if k in r and np.isfinite(r[k])], float)
        if v.size:
            out[k] = {"n": int(v.size), "p10": float(np.percentile(v, 10)), "p25": float(np.percentile(v, 25)),
                      "p50": float(np.percentile(v, 50)), "p75": float(np.percentile(v, 75)),
                      "p90": float(np.percentile(v, 90)), "mean": float(v.mean())}
    P = np.array([r["profile"] for r in rows if "profile" in r], float)
    if P.size:
        out["profile_km"] = PROFILE_KM
        out["profile_p50_m"] = [float(v) for v in np.nanmedian(P, axis=0)]
    sl = np.array([r["slope_500_2000_deg"] for r in rows if "slope_500_2000_deg" in r])
    if sl.size:
        out["slope_500_2000_hist"] = {f"<{t}": float((sl < t).mean()) for t in (1, 2, 3, 4, 6, 8, 12)}
    return out


class Grid:
    def __init__(self, idx):
        self.w, self.h, self.r = idx["w"], idx["h"], idx["r"]
        c = self.w * self.r
        self.R = c / (2 * math.pi)
        self.zo = self.h * self.r / 2
        z = -self.zo + (np.arange(self.h) + 0.5) * self.r
        s = np.clip(-z / self.R, -1.0, 1.0)
        self.lat = np.degrees(np.arcsin(s))
        self.cos = np.sqrt(np.maximum(1.0 - s * s, 1e-6))
        self.inside = np.abs(z) < self.R
        self.cell = self.r * M_PER_BLOCK_H

    def metric(self, j):
        """(east-west, north-south) size of a cell of row j, m."""
        return self.cell * self.cos[j], self.cell / self.cos[j]


def load(d, name):
    idx = json.load(open(d + "/index.json"))
    if name not in idx:
        return None, None
    e = idx[name]
    return np.fromfile(d + "/" + e["file"], dtype="<f4").reshape(e["h"], e["w"]).astype(np.float64), e


def open_sea_mask(wet):
    lab, n = ndimage.label(wet)
    if n == 0:
        return wet
    sizes = ndimage.sum(wet, lab, range(1, n + 1))
    keep = np.zeros(n + 1, bool)
    keep[1:] = sizes > 0.02 * sizes.max()
    return keep[lab]


def transects(elev, sea, g, active=None):
    H, W = elev.shape
    land = elev > sea
    lab, n = ndimage.label(land)
    cell_km2 = (g.cell / 1000.0) ** 2  # equal-area rows
    sizes = ndimage.sum(np.ones_like(elev), lab, range(1, n + 1)) * cell_km2
    keep = np.zeros(n + 1, bool)
    keep[1:] = sizes >= 750.0
    bigland = keep[lab]
    osea = open_sea_mask(~land & g.inside[:, None])
    nb = np.zeros_like(land)
    for dz, dx in ((0, 1), (0, -1), (1, 0), (-1, 0)):
        nb |= np.roll(bigland, (dz, dx), axis=(0, 1))
    rows_ok = (np.abs(g.lat) <= MAX_LAT) & g.inside
    js, is_ = np.nonzero(osea & nb & rows_ok[:, None])
    sel = np.arange(0, js.size, STRIDE)
    out = []
    for j, i in zip(js[sel], is_[sel]):
        dxm, dzm = g.metric(j)
        r = 10_000.0
        si, sj = r / dxm, r / dzm
        v = ndimage.map_coordinates(elev, [[j, j, j + sj, j - sj], [i + si, i - si, i, i]], order=1, mode="wrap")
        gx, gz = (v[0] - v[1]) / (2 * r), (v[2] - v[3]) / (2 * r)
        norm = math.hypot(gx, gz)
        if norm < 1e-6:
            continue
        ux, uz = -gx / norm, -gz / norm
        s = np.arange(0.0, SEAWARD_M, STEP_M)
        ci, cj = i + ux * s / dxm, j + uz * s / dzm
        ok = (cj >= 0) & (cj <= H - 1)
        if not ok.all():
            first = int(np.argmin(ok))
            s, ci, cj = s[:first], ci[:first], cj[:first]
        if s.size < 20:
            continue
        d = sea - ndimage.map_coordinates(elev, [cj, ci], order=1, mode="wrap")
        back = np.nonzero((d < 0) & (s > 3000))[0]
        if back.size:
            d = d[: back[0]]
        a = analyse(d, STEP_M)
        if a is None:
            continue
        for km in (3, 10, 30):
            li, lj = i - ux * km * 1000 / dxm, j - uz * km * 1000 / dzm
            if 0 <= lj <= H - 1:
                a[f"land_{km}km_m"] = float(ndimage.map_coordinates(elev, [[lj], [li]], order=1, mode="wrap")[0] - sea)
        prof = []
        for km in PROFILE_KM:
            li, lj = i + ux * km * 1000 / dxm, j + uz * km * 1000 / dzm
            prof.append(float(ndimage.map_coordinates(elev, [[lj], [li]], order=1, mode="wrap")[0] - sea)
                        if 0 <= lj <= H - 1 else float("nan"))
        a["profile"] = prof
        if active is not None:
            a["active"] = bool(active[j, i])
        out.append(a)
    return out


def distance_from(mask_src, g):
    """Distance (km, on the planet) of every cell from the nearest True cell of mask_src, through the
    8-neighbour cell graph with each row's metric."""
    H, W = mask_src.shape
    idx = np.arange(H * W).reshape(H, W)
    rows, cols, wts = [], [], []
    for dj, di in ((0, 1), (1, 0), (1, 1), (1, -1)):
        a = idx[: H - dj, :] if dj else idx
        b = np.roll(idx, -di, axis=1)[dj:, :] if dj else np.roll(idx, -di, axis=1)
        jj = np.arange(H - dj)[:, None] if dj else np.arange(H)[:, None]
        ew = g.cell * 0.5 * (g.cos[jj] + g.cos[jj + dj]) * abs(di)
        ns = g.cell * 0.5 * (1 / g.cos[jj] + 1 / g.cos[jj + dj]) * dj
        w = np.broadcast_to(np.hypot(ew, ns) / 1000.0, a.shape)
        rows.append(a.ravel())
        cols.append(b.ravel())
        wts.append(w.ravel())
    r = np.concatenate(rows)
    c = np.concatenate(cols)
    w = np.concatenate(wts)
    m = coo_matrix((w, (r, c)), shape=(H * W, H * W)).tocsr()
    src = np.nonzero(mask_src.ravel())[0]
    if src.size == 0:
        return np.full((H, W), np.inf)
    d = dijkstra(m, directed=False, indices=src, min_only=True)
    return d.reshape(H, W)


def crust_part(elev, sea, kind, thick, g):
    cont = (kind > 0.5) & g.inside[:, None]
    ocean = (kind <= 0.5) & g.inside[:, None]
    dist = distance_from(ocean, g)
    wet = elev <= sea
    out = {"continental_cells": int(cont.sum()), "continental_share": float(cont.sum() / g.inside.sum() / g.w),
           "continental_wet_share": float((cont & wet).sum() / max(cont.sum(), 1)), "bins": []}
    for lo, hi in zip(DIST_BINS_KM[:-1], DIST_BINS_KM[1:]):
        m = cont & (dist >= lo) & (dist < hi)
        b = {"km": [lo, hi if hi < 1e8 else None], "cells": int(m.sum()),
             "share_of_continental": float(m.sum() / max(cont.sum(), 1)),
             "wet": float((m & wet).sum() / max(m.sum(), 1)),
             "elev_p50_m": float(np.median(elev[m] - sea)) if m.any() else None}
        if thick is not None and m.any():
            b["thickness_p50_km"] = float(np.median(thick[m]) / 1000.0)
        out["bins"].append(b)
    # Ocean-side cells within 100 km of continental crust: how shallow is the sea floor beside it.
    dcont = distance_from(cont, g)
    for lo, hi in ((0, 10), (10, 25), (25, 50), (50, 100), (100, 200)):
        m = ocean & (dcont > lo) & (dcont <= hi)
        out.setdefault("ocean_bins", []).append(
            {"km": [lo, hi], "cells": int(m.sum()), "elev_p50_m": float(np.median(elev[m] - sea)) if m.any() else None,
             "wet": float((m & wet).sum() / max(m.sum(), 1))})
    return out


def planet(d, out_path):
    h, e = load(d, "surface/elevation_m")
    y, _ = load(d, "elevation")
    field = os.environ.get("MARGINS_FIELD")
    if field:
        # Transects on another stage's elevation (e.g. plates/elevation_m, the plate model's own),
        # against the same sea level.
        h0 = h
        h, _ = load(d, field)
        y = y + (h - h0) / M_PER_BLOCK_V
    kind, _ = load(d, "plates/crust_type")
    thick, _ = load(d, "plates/thickness_m")
    g = Grid(e)
    inside = np.broadcast_to(g.inside[:, None], h.shape)
    sea = float(np.median((h - (y - SEA_Y) * M_PER_BLOCK_V)[inside][::97]))
    reg, _ = load(d, "regime")
    active = None
    if reg is not None:
        # Active margins (ACTIVE_REGIMES near the coast cell); the rest are passive.
        active = ndimage.binary_dilation(np.isin(np.rint(reg), ACTIVE_REGIMES), iterations=ACTIVE_CELLS)
    rows = transects(h, sea, g, active)
    res = {"sea_m": sea, "summary": summarise(rows)}
    if active is not None:
        res["summary_passive"] = summarise([r for r in rows if not r.get("active")])
        res["summary_active"] = summarise([r for r in rows if r.get("active")])
    if kind is not None:
        res["crust"] = crust_part(h, sea, kind, thick, g)
    res["rows"] = rows
    json.dump(res, open(out_path, "w"))
    show(res["summary"], res.get("crust"))
    for k in ("summary_passive", "summary_active"):
        if k in res:
            print(k)
            show(res[k])


def show(s, crust=None):
    ks = ["slope_500_2000_deg", "slope_deg", "steepest_deg", "shelf_km", "to_200m_km", "break_m", "land_3km_m",
          "land_10km_m", "land_30km_m"]
    print("transects", s.get("transects"))
    for k in ks:
        if k in s:
            v = s[k]
            print(f"  {k:20s} p10 {v['p10']:8.1f} p25 {v['p25']:8.1f} p50 {v['p50']:8.1f} p75 {v['p75']:8.1f} "
                  f"p90 {v['p90']:8.1f} mean {v['mean']:8.1f}")
    if "slope_500_2000_hist" in s:
        print("  slope 500-2000 cumulative", " ".join(f"{k}:{v:.2f}" for k, v in s["slope_500_2000_hist"].items()))
    if "profile_p50_m" in s:
        print("  profile km", " ".join(f"{k:>6}" for k in s["profile_km"]))
        print("  median m  ", " ".join(f"{v:6.0f}" for v in s["profile_p50_m"]))
    if crust:
        print(f"  continental share {crust['continental_share']:.3f}, wet {crust['continental_wet_share']:.3f}")
        for b in crust["bins"]:
            print(f"    from ocean {b['km']} km: {b['share_of_continental']:.3f} of continental, wet {b['wet']:.2f}, "
                  f"median {b['elev_p50_m']} m, crust {b.get('thickness_p50_km')}")
        for b in crust.get("ocean_bins", []):
            print(f"    ocean {b['km']} km from continent: cells {b['cells']}, median {b['elev_p50_m']} m")


def pool(out_path, files):
    rows, crusts = [], []
    for f in files:
        d = json.load(open(f))
        rows += d["rows"]
        if "crust" in d:
            crusts.append(d["crust"])
    s = summarise(rows)
    res = {"runs": len(files), "summary": s}
    if any("active" in r for r in rows):
        res["summary_passive"] = summarise([r for r in rows if r.get("active") is False])
        res["summary_active"] = summarise([r for r in rows if r.get("active") is True])
    if crusts:
        res["continental_wet_share_mean"] = float(np.mean([c["continental_wet_share"] for c in crusts]))
        bins = []
        for k in range(len(crusts[0]["bins"])):
            bs = [c["bins"][k] for c in crusts]
            bins.append({"km": bs[0]["km"], "share_of_continental": float(np.mean([b["share_of_continental"] for b in bs])),
                         "wet": float(np.mean([b["wet"] for b in bs]))})
        res["bins"] = bins
    json.dump(res, open(out_path, "w"))
    print(f"pooled {len(files)} runs")
    show(s)
    if crusts:
        print(f"  continental crust under the sea, mean over runs: {res['continental_wet_share_mean']:.3f}")
        for b in res["bins"]:
            print(f"    from ocean {b['km']} km: {b['share_of_continental']:.3f} of continental, wet {b['wet']:.2f}")
    for k in ("summary_passive", "summary_active"):
        if k in res:
            print(k)
            show(res[k])


if __name__ == "__main__":
    if sys.argv[1] == "planet":
        planet(sys.argv[2], sys.argv[3])
    else:
        pool(sys.argv[2], sys.argv[3:])
