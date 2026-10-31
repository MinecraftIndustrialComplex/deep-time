#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
# (Copied 2026-10-31 from the advisor's /data/ban/deeptime-cache/advisor/adv_maps.py on the VM, unchanged
# but for these lines, so that advmaps.sh runs from the repo.)
# Advisor's read-only diagnosis over cached pool runs (crust-budget maps 50 Myr before each run's end)
# and Earth (SRTM15+ 15' at 100,000 Fibonacci points). Nothing here changes a run folder.
#
#   python3 -I adv_maps.py <libdir: tools/review/m5diag> <pool argkey folder> <out.jsonl> [<earth grd>]
#
# Per run and per sea level: land, M4, M5, the geometry factors, the land's hypsometry, where the land
# above 2 km stands from the sea, what the high coasts face (the nearest sea cell's crust, depth, plate,
# floor age), the landmass they stand on (area and half-width), and the orogeny, age and thickness of the
# high ground that qualifies them.
import json
import re
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

LIBDIR = sys.argv[1]
sys.path.insert(0, LIBDIR)
import m5lib  # noqa: E402
from m5lib import cc  # noqa: E402

NPTS = 100_000
FIELDS = 11
MODEL_OFFSETS = [0.0, -100.0, -300.0, -600.0, "datum", "earthland"]
EARTH_OFFSETS = [0.0, 100.0, 300.0, 600.0, "modelland"]
EARTH_LAND = 0.292
BANDS = [0, 500, 1000, 2000, 3000, 4000, 1e9]
AGE_BANDS = [(-1, 2), (2, 10), (10, 30), (30, 100), (100, 1e9)]


def q(a, qs):
    return [float(np.quantile(a, x)) for x in qs]


def analyse(tag, p, z_rel, sp, keep, plate=None, age=None, oro=None, oro_age=None, thick=None,
            kind=None, flags=None, arc_oc=None, arc_co=None):
    det = m5lib.coast_detail(p, z_rel, sp, keep=keep)
    fa = m5lib.factors(det)
    land, hi, d_sea, comp, ci, D = det["land"], det["hi"], det["d_sea"], det["comp"], det["ci"], det["D"]
    k = keep
    qual = np.isfinite(D[:, 0])
    out = {"tag": tag, "land": float((land & k).sum() / k.sum()), "M4": float(fa["M4"]), "M5": float(fa["M5"]),
           "f_near": float(fa["f_near"]), "kappa": float(fa["kappa"]), "s": float(fa["s"]),
           "land_within_200": float(fa["land_within_200"])}
    zl = z_rel[land & k]
    if zl.size:
        out["land_mean_m"] = float(zl.mean())
        out["hyps"] = [float(((zl > a) & (zl <= b)).mean()) for a, b in zip(BANDS[:-1], BANDS[1:])]
        out["z_p99_m"], out["z_max_m"] = float(np.quantile(zl, 0.99)), float(zl.max())
    dh = d_sea[hi & k]
    if dh.size:
        out["hi_dsea_pct"] = q(dh, (0.25, 0.5, 0.75, 0.9))
        out["hi_far"] = {str(km): float((dh > km).mean()) for km in (200, 400, 800)}
    hc = ci[qual & k[ci]]
    out["n_hc"] = int(hc.size)
    out["n_coast"] = int(k[ci].sum())
    if hc.size:
        ncomp = int(comp.max()) + 1
        half = np.zeros(ncomp)
        np.maximum.at(half, comp[land], d_sea[land])
        area = np.bincount(comp[land], minlength=ncomp) * cc.EARTH_KM2 / p.shape[0] / 1e6
        hw, ar = half[comp[hc]], area[comp[hc]]
        out["hc_halfwidth_lt200"] = float((hw < 200).mean())
        out["hc_halfwidth_lt400"] = float((hw < 400).mean())
        out["hc_area_lt1"] = float((ar < 1).mean())
        out["hc_area_1_10"] = float(((ar >= 1) & (ar < 10)).mean())
        out["hc_area_ge10"] = float((ar >= 10).mean())
        # all coast, for the same landmass classes (what share of the coast is on narrow blocks)
        ca = ci[k[ci]]
        out["coast_halfwidth_lt200"] = float((half[comp[ca]] < 200).mean())
        out["coast_area_lt1"] = float((area[comp[ca]] < 1).mean())
        out["m5_on_halfwidth_lt200"] = float(np.isfinite(D[:, 0])[k[ci] & (half[comp[ci]] < 200)].mean()) \
            if (k[ci] & (half[comp[ci]] < 200)).any() else None
        out["m5_on_halfwidth_ge200"] = float(np.isfinite(D[:, 0])[k[ci] & (half[comp[ci]] >= 200)].mean()) \
            if (k[ci] & (half[comp[ci]] >= 200)).any() else None
        sea_idx = np.nonzero(~land)[0]
        _, js = cKDTree(p[~land]).query(p[hc])
        ns = sea_idx[js]
        out["hc_sea_depth_med_m"] = float(np.median(-z_rel[ns]))
        if kind is not None:
            out["hc_sea_cont"] = float((kind[ns] > 0.5).mean())
        if plate is not None:
            out["hc_sea_same_plate"] = float((plate[ns] == plate[hc]).mean())
        if age is not None:
            a = age[ns]
            a = a[np.isfinite(a)]
            out["hc_sea_oceanic_share"] = float(a.size / hc.size)
            out["hc_sea_age_med_myr"] = float(np.median(a)) if a.size else None
            out["hc_sea_age_lt30"] = float((a < 30).mean()) if a.size else None
        if flags is not None:
            for name, bit in (("trench", 256), ("collfront", 512)):
                fp = np.nonzero((flags & bit) > 0)[0]
                if fp.size:
                    d, _ = cKDTree(p[fp]).query(p[hc], distance_upper_bound=cc.chord(200.0))
                    out[f"hc_{name}200"] = float(np.isfinite(d).mean())
                else:
                    out[f"hc_{name}200"] = 0.0
        if oro is not None:
            J = det["J"][:, 0][qual & k[ci]]
            out["hc_hg_him"] = float((oro[J] == 2).mean())
            out["hc_hg_and"] = float((oro[J] == 1).mean())
            out["hc_hg_age_pct"] = q(oro_age[J], (0.5, 0.75, 0.9))
            out["hc_hg_thick_pct_km"] = [x / 1000 for x in q(thick[J], (0.25, 0.5, 0.75))]
            out["hc_hg_z_med_m"] = float(np.median(z_rel[J]))
            out["hc_hg_dsea_med_km"] = float(np.median(d_sea[J]))
            if arc_oc is not None:
                out["hc_hg_arc_oc_med_m"] = float(np.median(arc_oc[J]))
                out["hc_hg_arc_oc_gt1km"] = float((arc_oc[J] > 1000).mean())
                out["hc_hg_arc_co_med_m"] = float(np.median(arc_co[J]))
            if flags is not None:
                fp = np.nonzero((flags & 512) > 0)[0]
                if fp.size:
                    d, _ = cKDTree(p[fp]).query(p[hc])
                    out["hc_collfront_dist_med_km"] = float(np.median(cc.arc_km(d)))
            ncomp_sizes = np.bincount(comp[land], minlength=ncomp)
            out["hc_on_largest_landmass"] = float((comp[hc] == int(np.argmax(ncomp_sizes))).mean())
    if oro is not None and (hi & k).any():
        H = hi & k
        for nm, m in (("hi_all", H), ("hi_coastal", H & (d_sea <= 200)), ("hi_inland", H & (d_sea > 200))):
            if m.any():
                out[nm] = {"n": int(m.sum()), "him": float((oro[m] == 2).mean()), "and": float((oro[m] == 1).mean()),
                           "none": float((oro[m] == 0).mean()),
                           "age_bands": [float(((oro_age[m] > a) & (oro_age[m] <= b)).mean()) for a, b in AGE_BANDS],
                           "thick_med_km": float(np.median(thick[m])) / 1000, "z_med_m": float(np.median(z_rel[m])),
                           "z_p90_m": float(np.quantile(z_rel[m], 0.9)),
                           "arc_oc_gt1km": float((arc_oc[m] > 1000).mean()) if arc_oc is not None else None}
    return out


def land_share(p, z, sp, keep, sea):
    land = (z - sea) > 0
    return float((land & keep).sum() / keep.sum())


def sea_for_land(p, z, sp, keep, target, lo=-4000.0, hi=6000.0):
    for _ in range(60):
        mid = 0.5 * (lo + hi)
        if land_share(p, z, sp, keep, mid) > target:
            lo = mid
        else:
            hi = mid
    return 0.5 * (lo + hi)


def model_run(run, pf, sp):
    a = np.fromfile(run / "maps" / "t-050.bin", "<f4").reshape(-1, FIELDS).astype(np.float64)
    n = a.shape[0]
    plate = np.fromfile(run / "maps" / "plate" / "t-050.bin", "<u4")
    agep = run / "maps" / "age" / "t-050.bin"
    age = np.fromfile(agep, "<f4").astype(np.float64) if agep.exists() else None
    m = re.search(r"cb_t t=-50 .*?sea_m=(-?\d+)", (run / "summary.txt").read_text())
    sea_m = float(m.group(1))
    _, nn = cKDTree(cc.xyz(a[:, 0], a[:, 1])).query(pf)
    b = a[nn]
    z = b[:, 2]
    thick, kind = b[:, 3], b[:, 4]
    oro = np.rint(np.nan_to_num(b[:, 5])).astype(int)
    oro_age = np.nan_to_num(b[:, 6], nan=1e9)
    flags = b[:, 10].astype(np.int64)
    pl = plate[nn] if plate.size == n else None
    ag = age[nn] if age is not None and age.size == n else None
    keep = np.ones(NPTS, bool)
    rows = []
    for off in MODEL_OFFSETS:
        if off == "datum":
            sea = 0.0
        elif off == "earthland":
            sea = sea_for_land(pf, z, sp, keep, EARTH_LAND)
        else:
            sea = sea_m + off
        extra = dict(plate=pl, age=ag, oro=oro, oro_age=oro_age, thick=thick, kind=kind, flags=flags,
                     arc_oc=b[:, 7], arc_co=b[:, 8]) \
            if off in (0.0, "datum", "earthland") else {}
        o = analyse(str(off), pf, z - sea, sp, keep, **extra)
        o.update({"run": run.name, "sea_m": sea_m, "sea_used_m": sea, "samples": int(n)})
        rows.append(o)
    return rows


def earth(path, pf, sp, flat, flon):
    import h5py
    with h5py.File(path, "r") as f:
        zg = f["z"][()].astype(np.float64)
        la = f["lat"][()] if "lat" in f else f["y"][()]
        lo = f["lon"][()] if "lon" in f else f["x"][()]
        at = {k: f["z"].attrs[k] for k in f["z"].attrs}
    zg = zg * float(np.ravel(at.get("scale_factor", 1.0))[0]) + float(np.ravel(at.get("add_offset", 0.0))[0])
    la, lo = np.asarray(la, float), np.asarray(lo, float)
    lon = ((flon + 180.0) % 360.0) - 180.0
    iy = np.clip(np.rint((flat - la[0]) / (la[1] - la[0])), 0, len(la) - 1).astype(int)
    ix = np.clip(np.rint((lon - lo[0]) / (lo[1] - lo[0])), 0, len(lo) - 1).astype(int)
    z = zg[iy, ix]
    sheets = (flat < -60.0) | ((flat > 59.5) & (lon > -74.0) & (lon < -10.0) & ~((flat < 66.7) & (lon > -25.0)))
    keep = ~sheets
    rows = []
    for off in EARTH_OFFSETS:
        if off == "modelland":
            sea = sea_for_land(pf, z, sp, keep, 0.309)
        else:
            sea = float(off)
        o = analyse(str(off), pf, z - sea, sp, keep)
        o.update({"run": "earth", "sea_m": 0.0, "sea_used_m": sea})
        rows.append(o)
    return rows


def med(rows, key, sub=None, idx=None):
    v = []
    for r in rows:
        x = r.get(key)
        if x is None:
            continue
        if sub is not None:
            x = x.get(sub) if isinstance(x, dict) else None
            if x is None:
                continue
        if idx is not None:
            x = x[idx]
        if isinstance(x, (int, float)) and np.isfinite(x):
            v.append(x)
    return (float(np.median(v)), len(v)) if v else (float("nan"), 0)


def main():
    folder, out_path = Path(sys.argv[2]), Path(sys.argv[3])
    flat, flon = m5lib.fib(NPTS)
    pf = cc.xyz(flat, flon)
    sp = float(np.sqrt(cc.EARTH_KM2 / NPTS))
    rows = []
    runs = sorted(d for d in folder.iterdir() if d.is_dir() and (d / "maps" / "t-050.bin").exists())
    for run in runs:
        rows.extend(model_run(run, pf, sp))
        print("done", run.name, file=sys.stderr, flush=True)
    if len(sys.argv) > 4:
        rows.extend(earth(sys.argv[4], pf, sp, flat, flon))
        print("done earth", file=sys.stderr, flush=True)
    with out_path.open("w") as f:
        for r in rows:
            f.write(json.dumps(r) + "\n")
    # Summary
    model = [r for r in rows if r["run"] != "earth"]
    er = [r for r in rows if r["run"] == "earth"]
    print(f"runs {len(runs)}; model rows {len(model)}; earth rows {len(er)}")
    print("\n== Sea level: medians over the runs (model) and Earth, by the sea offset")
    keys = ["land", "M4", "M5", "f_near", "kappa", "s", "land_within_200", "land_mean_m", "z_p99_m", "sea_used_m"]
    for off in MODEL_OFFSETS:
        sel = [r for r in model if r["tag"] == str(off)]
        print(f"model {off!s:>9}: " + "  ".join(f"{k} {med(sel, k)[0]:.4f}" for k in keys))
    for off in EARTH_OFFSETS:
        sel = [r for r in er if r["tag"] == str(off)]
        if sel:
            print(f"earth {off!s:>9}: " + "  ".join(f"{k} {sel[0].get(k, float('nan')):.4f}" for k in keys))
    print("\n== Hypsometry of the land (shares in 0-0.5, 0.5-1, 1-2, 2-3, 3-4, >4 km) and the high ground's distance to the sea")
    sel = [r for r in model if r["tag"] == "0.0"]
    print("model hyps   " + " ".join(f"{med(sel, 'hyps', idx=i)[0]:.4f}" for i in range(6)))
    if er:
        print("earth hyps   " + " ".join(f"{er[0]['hyps'][i]:.4f}" for i in range(6)))
    print("model hi d_sea p25/50/75/90 km " + " ".join(f"{med(sel, 'hi_dsea_pct', idx=i)[0]:.0f}" for i in range(4))
          + "  inland >200/400/800 " + " ".join(f"{med(sel, 'hi_far', sub=s)[0]:.3f}" for s in ("200", "400", "800")))
    if er:
        e = er[0]
        print("earth hi d_sea p25/50/75/90 km " + " ".join(f"{e['hi_dsea_pct'][i]:.0f}" for i in range(4))
              + "  inland >200/400/800 " + " ".join(f"{e['hi_far'][s]:.3f}" for s in ("200", "400", "800")))
    print("\n== The high coasts (the qualified coast): what they stand on and face (model at its own sea; Earth)")
    hk = ["hc_halfwidth_lt200", "hc_halfwidth_lt400", "hc_area_lt1", "hc_area_1_10", "hc_area_ge10",
          "coast_halfwidth_lt200", "coast_area_lt1", "m5_on_halfwidth_lt200", "m5_on_halfwidth_ge200",
          "hc_sea_depth_med_m", "hc_sea_cont", "hc_sea_same_plate", "hc_sea_oceanic_share", "hc_sea_age_med_myr",
          "hc_sea_age_lt30", "hc_trench200", "hc_collfront200", "hc_hg_him", "hc_hg_and", "hc_hg_z_med_m",
          "hc_hg_dsea_med_km", "hc_hg_arc_oc_med_m", "hc_hg_arc_oc_gt1km", "hc_hg_arc_co_med_m",
          "hc_collfront_dist_med_km", "hc_on_largest_landmass", "n_hc", "n_coast"]
    for k in hk:
        m, n = med(sel, k)
        ev = er[0].get(k) if er else None
        print(f"{k:28s} model {m:10.4f} (n {n})   earth {ev if ev is None else round(ev, 4)}")
    print("hc_hg_age p50/75/90 Myr " + " ".join(f"{med(sel, 'hc_hg_age_pct', idx=i)[0]:.1f}" for i in range(3)))
    print("hc_hg_thick p25/50/75 km " + " ".join(f"{med(sel, 'hc_hg_thick_pct_km', idx=i)[0]:.1f}" for i in range(3)))
    print("\n== The land above 2 km: make-up (model), all / coastal (within 200 km) / inland")
    for nm in ("hi_all", "hi_coastal", "hi_inland"):
        print(f"{nm:11s} n {med(sel, nm, sub='n')[0]:.0f} him {med(sel, nm, sub='him')[0]:.3f} and {med(sel, nm, sub='and')[0]:.3f} "
              f"none {med(sel, nm, sub='none')[0]:.3f} thick_med {med(sel, nm, sub='thick_med_km')[0]:.1f} km "
              f"z_med {med(sel, nm, sub='z_med_m')[0]:.0f} z_p90 {med(sel, nm, sub='z_p90_m')[0]:.0f} m arc_oc>1km {med(sel, nm, sub='arc_oc_gt1km')[0]:.3f}  age bands <2/2-10/10-30/30-100/>100: "
              + " ".join(f"{float(np.median([r[nm]['age_bands'][i] for r in sel if nm in r])):.3f}" for i in range(5)))
    print("\n== Spearman over the runs (model, own sea): M5 against sea_m, land, M4, f_near, kappa")
    from scipy.stats import spearmanr
    m5 = np.array([r["M5"] for r in sel])
    for k in ("sea_m", "land", "M4", "f_near", "kappa", "hc_sea_same_plate", "hc_halfwidth_lt200"):
        x = np.array([r.get(k, np.nan) for r in sel], float)
        ok = np.isfinite(x)
        if ok.sum() > 5:
            rho, pv = spearmanr(m5[ok], x[ok])
            print(f"  {k:22s} rho {rho:+.3f} p {pv:.4f}")
    print("\n== M5 per run at its own sea, with sea_m and land")
    for r in sorted(sel, key=lambda r: r["M5"]):
        print(f"  {r['run']:26s} M5 {r['M5']:.4f} M4 {r['M4']:.4f} land {r['land']:.3f} sea_m {r['sea_m']:.0f} f_near {r['f_near']:.3f}")


if __name__ == "__main__":
    main()
