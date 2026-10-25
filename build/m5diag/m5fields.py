#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""M5 (coast with high ground) on the census's own grid, run by run: m5fields.py <out.jsonl> [workers]

For each base12 run (pool key 5cf16ecd76cd, earthlike_quick_16k-1000, argkey cd665cef): the final
fields as measure_v1census.py reads them (the L1 grid, z = (elevation - 63) x 35 m), checked
against the run's cached v1census values, then:

1. the qualified coast split by the origin of the high ground nearest to it within 200 km on the
   same landmass (the class of that high point), the coast that would stay qualified without each
   class, and the land above 2 km by the same classes. The tags are the final crust-budget map's
   (maps/t0000.bin, the nearest visible sample of each cell: orogeny, its age, the thickness, the
   arc magma, the flags) and the fields' regime;
2. the distance from each qualified coast point to its nearest qualifying high point (0 when the
   coast point is itself high), the high point's own distance to the sea and the landmass's area;
3. the geometry factors (m5lib.factors) and M5 at the levels that put Earth's share of the land
   above them;
4. the measure checks: M5 on Fibonacci points of the same count (nearest cell), M5 with the grid's
   8-connected landmasses for cc.components' 1.6-spacing ones, the coast band by latitude, closed
   depressions below the sea, the elevation field against the surface's and the ice, the x seam,
   the straight-line reach across the sea, and M5 on the plate model's elevation before the
   surface stage.

One JSON object per run to <out.jsonl>; m5table.py prints the medians."""
import contextlib
import io
import json
import sys
from multiprocessing import Pool
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
import m5lib  # noqa: E402
from m5lib import cc  # noqa: E402

POOL = Path("/data/ban/deeptime-runs/pool")
KEY = "5cf16ecd76cd"
ARM = "earthlike_quick_16k-1000/cd665cef"
FIELDS = 11
CLASSES = ["coll_young", "coll_old", "andean_young", "andean_old", "u_Tc", "u_ocean", "u_volc",
           "u_rift", "u_thick", "u_other"]
# Regime codes (plates/state.rs `regime`) and sample flags (state.rs).
REG_ARC, REG_BACKARC, REG_COLL, REG_RIFT, REG_HOT = 3, 4, 5, 6, 7
F_ARC, F_HOT, F_DIV, F_SZ, F_FORE = 4, 8, 16, 64, 128
TARGETS = [0.047, 0.055, 0.0617, 0.07, 0.10]
LAT_BANDS = [0, 30, 45, 52, 60, 70, 90]


def classify(oro, age, th, kind, arc_oc, arc_co, flags, reg):
    """The class index of each point's crust (CLASSES, in order of precedence)."""
    n = oro.size
    c = np.full(n, CLASSES.index("u_other"))
    u = oro == 0
    c[u & (th >= 50.5)] = CLASSES.index("u_thick")
    rift = (reg == REG_RIFT) | ((flags & F_DIV) != 0)
    c[u & rift] = CLASSES.index("u_rift")
    volc = (((arc_oc >= 1000.0) & (arc_oc > arc_co)) | np.isin(reg, (REG_ARC, REG_HOT))
            | ((flags & (F_ARC | F_HOT)) != 0))
    c[u & volc] = CLASSES.index("u_volc")
    c[u & ~kind] = CLASSES.index("u_ocean")
    c[u & kind & (th >= 49.5) & (th < 50.5)] = CLASSES.index("u_Tc")
    c[(oro == 1) & (age >= 10)] = CLASSES.index("andean_old")
    c[(oro == 1) & (age < 10)] = CLASSES.index("andean_young")
    c[(oro == 2) & (age >= 10)] = CLASSES.index("coll_old")
    c[(oro == 2) & (age < 10)] = CLASSES.index("coll_young")
    return c


def load_map(path):
    a = np.fromfile(path, "<f4").reshape(-1, FIELDS).astype(np.float64)
    return a


def align(p_cells, kind_cells, a):
    """The map's frame against the grid's: the transform of the samples' (lat, lon) whose nearest
    sample's crust kind agrees best with the cells' (the cells take the nearest sample's kind)."""
    rng = np.random.default_rng(0)
    pick = rng.choice(p_cells.shape[0], 20000, replace=False)
    best = None
    for ls in (1, -1):
        for lsign in (1, -1):
            for off in (0.0, 180.0, 90.0, -90.0):
                q = cc.xyz(ls * a[:, 0], lsign * a[:, 1] + off)
                _, j = cKDTree(q).query(p_cells[pick])
                agree = float(((a[j, 4] > 0.5) == kind_cells[pick]).mean())
                if best is None or agree > best[0]:
                    best = (agree, ls, lsign, off)
    return best


def one(run):
    with contextlib.redirect_stdout(io.StringIO()):
        f, g, e = cc.load_dump(run / "fields")
    h, w, r = e["h"], e["w"], e["r"]
    lat_r, ok_r = cc.lat_of_rows(h, r, w * r)
    lat2 = np.repeat(lat_r[:, None], w, 1)
    ok = np.repeat(ok_r[:, None], w, 1)
    lon2 = np.repeat((360.0 * np.arange(w) / w)[None, :], h, 0)
    ncell = int(ok.sum())
    cell_km2 = cc.EARTH_KM2 / ncell
    sp = float(np.sqrt(cell_km2))
    z2 = (f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V
    p = cc.xyz(lat2[ok], lon2[ok])
    z = z2[ok]
    lat = lat2[ok]
    out = {"run": run.name, "start": run.name.rsplit("-", 1)[0], "seed": int(run.name.rsplit("-", 1)[1]),
           "cells": ncell, "spacing_km": sp}
    cached = json.loads((run / "measures" / "v1census.json").read_text())["values"]
    det0 = m5lib.coast_detail(p, z, sp)
    out["M5"] = m5lib.m5_of(det0)
    out["M4"] = float((z[z > 0] > 2000).mean())
    out["cached_M5"] = cached["coast_hi"]
    out["cached_M4"] = cached["above_2km"]
    out["factors"] = m5lib.factors(det0)
    land, comp, d_sea = det0["land"], det0["comp"], det0["d_sea"]

    # Tags: the final map's nearest sample, in the grid's frame.
    a = load_map(run / "maps" / "t0000.bin")
    kind_f = (f["plates/crust_type"] > 0.5)[ok]
    agree, ls, lsign, off = align(p, kind_f, a)
    out["map_align"] = {"agree_kind": agree, "lat_sign": ls, "lon_sign": lsign, "lon_off": off}
    q = cc.xyz(ls * a[:, 0], lsign * a[:, 1] + off)
    _, nn = cKDTree(q).query(p)
    m_oro = np.rint(a[nn, 5]).astype(int)
    m_age = a[nn, 6]
    m_th = a[nn, 3] / 1000.0
    m_kind = a[nn, 4] > 0.5
    m_flags = a[nn, 10].astype(np.int64)
    f_oro = np.rint(np.nan_to_num(f["plates/orogeny_type"]))[ok].astype(int)
    reg = np.rint(np.nan_to_num(f["regime"]))[ok].astype(int)
    hi = det0["hi"]
    out["oro_agree_hi"] = float((m_oro[hi] == f_oro[hi]).mean()) if hi.any() else None
    cls = classify(m_oro, m_age, m_th, m_kind, a[nn, 7], a[nn, 8], m_flags, reg)
    det = m5lib.coast_detail(p, z, sp, cls=cls, ncls=len(CLASSES), comp=comp, d_sea=d_sea)
    D, J, ci = det["D"], det["J"], det["ci"]
    C = ci.size
    dmin = D.min(1)
    qual = np.isfinite(dmin)
    assert abs(qual.sum() / C - out["M5"]) < 1e-12
    near_cls = np.argmin(D, 1)
    L = int(land.sum())
    out["m5_by_nearest"] = {k: float((qual & (near_cls == i)).sum() / C) for i, k in enumerate(CLASSES)}
    out["m5_alone"] = {k: float(np.isfinite(D[:, i]).sum() / C) for i, k in enumerate(CLASSES)}
    out["m5_without"] = {k: float(np.isfinite(np.delete(D, i, 1)).any(1).sum() / C)
                         for i, k in enumerate(CLASSES)}
    out["m4_by"] = {k: float((hi & (cls == i)).sum() / L) for i, k in enumerate(CLASSES)}
    # The volcanic / arc tags of the parked columns (how much of u_Tc an arc test would also take).
    utc = cls == CLASSES.index("u_Tc")
    isl = (a[nn, 7] >= 1000.0) & (a[nn, 7] > a[nn, 8])
    out["u_Tc_hi_island_arc"] = float((utc & hi & isl).sum() / max((utc & hi).sum(), 1))
    out["u_Tc_hi_regime"] = {str(k): float((utc & hi & (reg == k)).sum() / max((utc & hi).sum(), 1))
                             for k in range(10)}
    out["u_Tc_hi_sz"] = float((utc & hi & ((m_flags & F_SZ) != 0)).sum() / max((utc & hi).sum(), 1))
    # The young collision class by the age of its stamp, and its high ground on a front.
    jn = J[np.arange(C), near_cls]
    cy = qual & (near_cls == CLASSES.index("coll_young"))
    out["coll_young_by_age"] = {f"{a_:g}-{b_:g}": float((cy & (m_age[np.maximum(jn, 0)] >= a_)
                                                         & (m_age[np.maximum(jn, 0)] < b_)).sum() / C)
                                for a_, b_ in ((-1e9, 2), (2, 5), (5, 10))}
    # The coast by the plate setting nearest to it, from the fields' regime (the final map
    # t0000.bin is a fresh lattice whose arc, trench and front flags are all zero): within 200 km
    # of a trench or arc cell, else of a collision cell, else neither.
    dfr = {}
    for nm, codes in (("sub", (2, 3)), ("coll", (REG_COLL,))):
        sel = np.isin(reg, codes)
        if sel.any():
            d_, _ = cKDTree(p[sel]).query(p[ci])
            dfr[nm] = cc.arc_km(d_)
        else:
            dfr[nm] = np.full(C, np.inf)
    ctype = np.where(dfr["sub"] <= 200, 0, np.where(dfr["coll"] <= 200, 1, 2))
    out["coast_by_front"] = {nm: {"coast_share": float((ctype == i).sum() / C),
                                  "m5_in": float((qual & (ctype == i)).sum() / max((ctype == i).sum(), 1)),
                                  "m5_part": float((qual & (ctype == i)).sum() / C)}
                             for i, nm in enumerate(("sub200", "coll200", "neither"))}
    # The qualifying high point's own setting, per class: its regime and whether it is on an
    # active collision (plates/collision_mm_yr > 0).
    coll_v = np.nan_to_num(f["plates/collision_mm_yr"])[ok]
    jq = jn[qual]
    nq = near_cls[qual]
    out["hi_setting_by_class"] = {}
    for i, k in enumerate(CLASSES):
        m = nq == i
        if not m.any():
            continue
        out["hi_setting_by_class"][k] = {
            "regime": {str(rc): float((reg[jq[m]] == rc).mean()) for rc in range(10)},
            "active_collision": float((coll_v[jq[m]] > 0).mean())}
    out["hi_regime_all"] = {str(rc): float((hi & (reg == rc)).sum() / max(hi.sum(), 1)) for rc in range(10)}
    out["hi_active_collision_all"] = float((hi & (coll_v > 0)).sum() / max(hi.sum(), 1))
    # Iced land kept out of the shares (the stand-in for Earth's by-rock figure without the
    # two ice sheets).
    ice_c = np.nan_to_num(f["climate/ice_thickness_m"][ok])
    for lim in (1.0, 100.0):
        kp = ~(land & (ice_c >= lim))
        dk = m5lib.coast_detail(p, z, sp, comp=comp, d_sea=d_sea, keep=kp)
        out[f"m5_ice_lt{lim:g}_kept_out"] = m5lib.m5_of(dk)
        out[f"m4_ice_lt{lim:g}_kept_out"] = float((z[land & kp] > 2000).mean())
    # Distances.
    out["dist_hist"] = m5lib.dist_hist(dmin)
    out["dist_hist_by"] = {k: m5lib.dist_hist(np.where(near_cls == i, dmin, np.inf))
                           for i, k in enumerate(CLASSES)}
    dh = d_sea[jn[qual]]
    out["hi_own_sea_km_hist"] = {f"{a_:g}-{b_:g}": float(((dh > a_) & (dh <= b_)).mean()) if dh.size else 0.0
                                 for a_, b_ in ((-1, 50), (50, 100), (100, 200), (200, 400), (400, 1e9))}
    lab = cc.label_wrap(ok & (z2 > 0))[ok]
    sizes = np.bincount(lab) * cell_km2 / 1e6
    lm = sizes[lab[ci]]
    out["coast_by_landmass_Mkm2"] = {}
    for a_, b_ in ((0, 1), (1, 10), (10, 30), (30, 1e9)):
        sel = (lm > a_) & (lm <= b_)
        out["coast_by_landmass_Mkm2"][f"{a_:g}-{b_:g}"] = {
            "coast_share": float(sel.sum() / C), "m5_in": float((qual & sel).sum() / max(sel.sum(), 1))}
    # M5 at Earth's share of land above the level.
    out["curve"] = m5lib.m5_curve(p, z, sp, comp, d_sea, TARGETS + [out["M4"]])
    out["curve_levels"] = {f"{lev:g}": m5lib.m5_of(m5lib.coast_detail(p, z, sp, level=lev, comp=comp, d_sea=d_sea))
                           for lev in (2500.0, 3000.0)}

    # Measure checks.
    chk = {}
    # (a) Fibonacci points of the same count, each the nearest cell's height.
    flat, flon = m5lib.fib(ncell)
    pf = cc.xyz(flat, flon)
    _, nf = cKDTree(p).query(pf)
    zf = z[nf]
    detf = m5lib.coast_detail(pf, zf, sp, cls=cls[nf], ncls=len(CLASSES))
    chk["fib_M5"] = m5lib.m5_of(detf)
    chk["fib_M4"] = float((zf[zf > 0] > 2000).mean())
    chk["fib_factors"] = m5lib.factors(detf)
    Df = detf["D"]
    qf = np.isfinite(Df.min(1))
    nfc = np.argmin(Df, 1)
    chk["fib_m5_by_nearest"] = {k: float((qf & (nfc == i)).sum() / Df.shape[0]) for i, k in enumerate(CLASSES)}
    chk["fib_curve"] = m5lib.m5_curve(pf, zf, sp, detf["comp"], detf["d_sea"], TARGETS)
    # (b) The grid's own 8-connected landmasses for the 1.6-spacing components.
    chk["n_comp_census"] = int(np.unique(comp[land]).size)
    chk["n_comp_grid8"] = int(np.unique(lab[land]).size)
    detl = m5lib.coast_detail(p, z, sp, comp=lab, d_sea=d_sea)
    chk["grid8_M5"] = m5lib.m5_of(detl)
    # (c) The coast band by latitude: coast points over land points on the grid and on the
    # Fibonacci points, and M5 in each band.
    band = {}
    for a_, b_ in zip(LAT_BANDS[:-1], LAT_BANDS[1:]):
        sg = land & (np.abs(lat) >= a_) & (np.abs(lat) < b_)
        sf = detf["land"] & (np.abs(flat) >= a_) & (np.abs(flat) < b_)
        cg = det0["coast"] & sg
        cf = detf["coast"] & sf
        qg = np.zeros(z.size, bool)
        qg[ci[qual]] = True
        qfm = np.zeros(zf.size, bool)
        qfm[detf["ci"][qf]] = True
        band[f"{a_}-{b_}"] = {"land_share": float(sg.sum() / max(L, 1)),
                              "coast_per_land_grid": float(cg.sum() / max(sg.sum(), 1)),
                              "coast_per_land_fib": float(cf.sum() / max(sf.sum(), 1)),
                              "m5_grid": float((qg & cg).sum() / max(cg.sum(), 1)),
                              "m5_fib": float((qfm & cf).sum() / max(cf.sum(), 1)),
                              "coast_share_grid": float(cg.sum() / C)}
    chk["lat_bands"] = band
    # (d) Closed depressions: the sea's 8-connected bodies; the coast whose nearest sea is a body
    # under 1 (0.1) Mkm2, and M5 with those bodies as land.
    sea2 = ok & ~(z2 > 0)
    slab = cc.label_wrap(sea2)
    ssz = np.bincount(slab.ravel()) * cell_km2 / 1e6
    ssz[0] = 0
    sea_ok = slab[ok]
    sea_idx = np.nonzero(~land)[0]
    _, js = cKDTree(p[sea_idx]).query(p[ci])
    body = ssz[sea_ok[sea_idx[js]]]
    chk["sea_bodies"] = {"n_ge1": int((ssz >= 1).sum()), "n_lt1": int(((ssz > 0) & (ssz < 1)).sum()),
                         "area_lt1_share_of_sea": float(ssz[(ssz > 0) & (ssz < 1)].sum() / max(ssz.sum(), 1e-9)),
                         "largest_share_of_sea": float(ssz.max() / max(ssz.sum(), 1e-9))}
    body_km2 = np.where(land, np.inf, ssz[sea_ok] * 1e6)
    chk["fixed_measure"] = m5lib.fixed_measure(p, z, body_km2, ncell, sp)
    for lim in (0.1, 1.0):
        chk[f"coast_on_body_lt{lim:g}"] = float((body < lim).sum() / C)
        chk[f"m5_coast_on_body_lt{lim:g}"] = float((qual & (body < lim)).sum() / max((body < lim).sum(), 1))
        zl = z.copy()
        fill = (~land) & (ssz[sea_ok] < lim)
        zl[fill] = 1.0
        dl = m5lib.coast_detail(p, zl, sp)
        chk[f"m5_bodies_lt{lim:g}_as_land"] = m5lib.m5_of(dl)
        chk[f"m4_bodies_lt{lim:g}_as_land"] = float((zl[zl > 0] > 2000).mean())
    # (e) The elevation field against the surface's and the ice.
    sea_m = g["globals"].get("climate/sea_level_m", float("nan"))
    se = f["surface/elevation_m"][ok]
    ice = np.nan_to_num(f["climate/ice_thickness_m"][ok])
    diff = z - (se - sea_m)
    iced = land & (ice >= 1.0)
    chk["elev_minus_surface_p50_land"] = float(np.median(diff[land]))
    chk["elev_minus_surface_p50_iced"] = float(np.median(diff[iced])) if iced.any() else None
    chk["iced_ice_p50"] = float(np.median(ice[iced])) if iced.any() else None
    chk["sea_m"] = sea_m
    if iced.sum() > 10:
        chk["diff_ice_slope"] = float(np.polyfit(ice[iced], diff[iced], 1)[0])
    chk["hi_iced_share"] = float((det0["hi"] & (ice >= 1.0)).sum() / max(det0["hi"].sum(), 1))
    chk["m4_by_surface_with_ice"] = float(((z + ice)[land] > 2000).mean())
    # (f) The x seam: a duplicated column, and the coast in the seam columns.
    chk["seam_equal_cols"] = float((f["elevation"][:, 0] == f["elevation"][:, -1])[ok_r].mean())
    colc = np.zeros((h, w), bool)
    cf2 = np.zeros(z.size, bool)
    cf2[ci] = True
    colc[ok] = cf2
    per_col = colc.sum(0)
    chk["seam_coast_col0_over_mean"] = float(per_col[0] / max(per_col.mean(), 1e-9))
    chk["seam_coast_colw_over_mean"] = float(per_col[-1] / max(per_col.mean(), 1e-9))
    landc = np.zeros((h, w), bool)
    landc[ok] = land
    per_land = landc.sum(0)
    cols = [0, 1, 2, 3, w // 2, w - 4, w - 3, w - 2, w - 1]
    chk["seam_cols"] = {str(c_): {"coast_over_mean": float(per_col[c_] / max(per_col.mean(), 1e-9)),
                                  "land_over_mean": float(per_land[c_] / max(per_land.mean(), 1e-9))} for c_ in cols}
    # Column-to-column change of the land mask across the seam against inside the grid.
    flips = (landc[:, 1:] != landc[:, :-1]).sum(0)
    chk["seam_land_flips_over_mean"] = float((landc[:, 0] != landc[:, -1]).sum() / max(flips.mean(), 1e-9))
    # (g) The straight line from a qualified coast point to its high point: across the sea?
    qi = np.nonzero(qual)[0]
    a0, b0 = p[ci[qi]], p[jn[qi]]
    tree = cKDTree(p)
    crosses = np.zeros(qi.size, bool)
    for t in np.linspace(0.1, 0.9, 9):
        m = a0 * (1 - t) + b0 * t
        m /= np.linalg.norm(m, axis=1)[:, None]
        _, jm = tree.query(m)
        crosses |= ~land[jm]
    chk["line_crosses_sea"] = float(crosses.mean()) if qi.size else 0.0
    chk["m5_if_crossing_dropped_upper"] = float((qi.size - crosses.sum()) / C)
    # (h) The plate model's own elevation (before the surface stage).
    zp = f["plates/elevation_m"][ok] - sea_m
    dp = m5lib.coast_detail(p, zp, sp)
    chk["plate_z_M5"] = m5lib.m5_of(dp)
    chk["plate_z_M4"] = float((zp[zp > 0] > 2000).mean())
    out["checks"] = chk
    return out


def main():
    dest = Path(sys.argv[1])
    workers = int(sys.argv[2]) if len(sys.argv) > 2 else 3
    runs = sorted(p for p in (POOL / KEY / ARM).glob("*-*")
                  if p.is_dir() and (p / "fields" / "index.json").exists())
    if len(sys.argv) > 3:
        runs = [p for p in runs if p.name in sys.argv[3].split(",")]
    print(f"{len(runs)} runs", flush=True)
    with Pool(workers) as pool, dest.open("w") as fh:
        for res in pool.imap_unordered(one, runs):
            fh.write(json.dumps(res) + "\n")
            fh.flush()
            print(f"{res['run']}: M5 {res['M5']:.4f} (cached {res['cached_M5']:.4f}) M4 {res['M4']:.4f}"
                  f" (cached {res['cached_M4']:.4f}) align {res['map_align']}", flush=True)
    print("M5FIELDS_DONE")


if __name__ == "__main__":
    main()
