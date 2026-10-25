#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Earth's control for the M5 diagnosis: m5earth.py <earth_relief_15m_p.grd> <out.json>

Present-day SRTM15+ at 15' (earth_relief_15m_p.grd, Tozer et al. 2019; the grid the census's
control used, tools/review/earth-coast.py), on three point sets: the census's own L1 grid of the
16k planet (w 1024, h 328, r 16: Lambert cylindrical equal-area rows, each cell the nearest grid
node), Fibonacci points of the same count (332,800, the realism-checks control) and 100,000
Fibonacci points (the crust-budget maps' count). On each, with Antarctica and Greenland kept out
of the shares (cc's `sheets` box: the by-rock stand-in) and with everything: M4, M5, the geometry
factors (m5lib.factors), M5 at the levels that put given shares of the land above them, the
distance histogram, and on the grid the latitude bands, the 8-connected landmasses and the closed
sea bodies, as m5fields.py does for the model. Needs numpy, scipy, h5py."""
import json
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
import m5lib  # noqa: E402
from m5lib import cc  # noqa: E402

TARGETS = [0.047, 0.055, 0.0617, 0.07, 0.10, 0.128, 0.15]
LAT_BANDS = [0, 30, 45, 52, 60, 70, 90]


def load(path):
    import h5py
    with h5py.File(path, "r") as f:
        zg = f["z"][()].astype(np.float64)
        la = f["lat"][()] if "lat" in f else f["y"][()]
        lo = f["lon"][()] if "lon" in f else f["x"][()]
        at = {k: f["z"].attrs[k] for k in f["z"].attrs}
    zg = zg * float(np.ravel(at.get("scale_factor", 1.0))[0]) + float(np.ravel(at.get("add_offset", 0.0))[0])
    return zg, np.asarray(la, float), np.asarray(lo, float)


def sample(zg, la, lo, lat, lon):
    lon = ((np.asarray(lon) + 180.0) % 360.0) - 180.0
    iy = np.clip(np.rint((lat - la[0]) / (la[1] - la[0])), 0, len(la) - 1).astype(int)
    ix = np.clip(np.rint((lon - lo[0]) / (lo[1] - lo[0])), 0, len(lo) - 1).astype(int)
    return zg[iy, ix], lon


def sheets_of(lat, lon):
    return (lat < -60.0) | ((lat > 59.5) & (lon > -74.0) & (lon < -10.0) & ~((lat < 66.7) & (lon > -25.0)))


def metrics(p, z, sp, lat, sheets, grid_lab=None):
    out = {}
    for name, keep in (("no_sheets", ~sheets), ("all", np.ones(z.size, bool))):
        det = m5lib.coast_detail(p, z, sp, keep=keep)
        o = {"M5": m5lib.m5_of(det), "M4": float((z[(z > 0) & keep] > 2000).mean()),
             "factors": m5lib.factors(det)}
        dmin = det["D"].min(1)
        kc = keep[det["ci"]]
        o["dist_hist"] = m5lib.dist_hist(np.where(kc, dmin, np.inf))
        o["curve"] = m5lib.m5_curve(p, z, sp, det["comp"], det["d_sea"], TARGETS, keep=keep)
        o["curve_levels"] = {f"{lev:g}": m5lib.m5_of(m5lib.coast_detail(p, z, sp, level=lev, comp=det["comp"],
                                                                        d_sea=det["d_sea"], keep=keep))
                             for lev in (1500.0, 1000.0)}
        qual = np.isfinite(dmin)
        band = {}
        L = int((det["land"] & keep).sum())
        for a_, b_ in zip(LAT_BANDS[:-1], LAT_BANDS[1:]):
            sg = det["land"] & keep & (np.abs(lat) >= a_) & (np.abs(lat) < b_)
            cg = det["coast"] & sg
            qg = np.zeros(z.size, bool)
            qg[det["ci"][qual]] = True
            band[f"{a_}-{b_}"] = {"land_share": float(sg.sum() / max(L, 1)),
                                  "coast_per_land": float(cg.sum() / max(sg.sum(), 1)),
                                  "m5": float((qg & cg).sum() / max(cg.sum(), 1))}
        o["lat_bands"] = band
        if grid_lab is not None:
            o["grid8_M5"] = m5lib.m5_of(m5lib.coast_detail(p, z, sp, comp=grid_lab, d_sea=det["d_sea"], keep=keep))
            o["n_comp_census"] = int(np.unique(det["comp"][det["land"]]).size)
            o["n_comp_grid8"] = int(np.unique(grid_lab[det["land"]]).size)
        out[name] = o
    return out


def main():
    zg, la, lo = load(sys.argv[1])
    res = {}
    # The census's grid.
    h, w, r = 328, 1024, 16
    lat_r, ok_r = cc.lat_of_rows(h, r, w * r)
    lat2 = np.repeat(lat_r[:, None], w, 1)
    ok = np.repeat(ok_r[:, None], w, 1)
    lon2 = np.repeat((360.0 * np.arange(w) / w)[None, :], h, 0)
    n = int(ok.sum())
    z2 = np.zeros((h, w))
    z2[ok], lon_s = sample(zg, la, lo, lat2[ok], lon2[ok])
    p = cc.xyz(lat2[ok], lon2[ok])
    z = z2[ok]
    sh = sheets_of(lat2[ok], lon_s)
    lab = cc.label_wrap(ok & (z2 > 0))[ok]
    sp = float(np.sqrt(cc.EARTH_KM2 / n))
    res["grid"] = metrics(p, z, sp, lat2[ok], sh, grid_lab=lab)
    res["grid"]["cells"] = n
    # Closed sea bodies on the grid.
    slab = cc.label_wrap(ok & ~(z2 > 0))
    ssz = np.bincount(slab.ravel()) * cc.EARTH_KM2 / n / 1e6
    ssz[0] = 0
    res["grid"]["sea_bodies"] = {"n_ge1": int((ssz >= 1).sum()), "n_lt1": int(((ssz > 0) & (ssz < 1)).sum()),
                                 "area_lt1_share_of_sea": float(ssz[(ssz > 0) & (ssz < 1)].sum() / ssz.sum()),
                                 "largest_share_of_sea": float(ssz.max() / ssz.sum()),
                                 "bodies_ge0.1_Mkm2": sorted([round(float(x), 2) for x in ssz if 0.1 <= x < 50])}
    for npts in (n, 100_000):
        flat, flon = m5lib.fib(npts)
        zf, flon_s = sample(zg, la, lo, flat, flon)
        pf = cc.xyz(flat, flon)
        res[f"fib{npts}"] = metrics(pf, zf, float(np.sqrt(cc.EARTH_KM2 / npts)), flat, sheets_of(flat, flon_s))
    Path(sys.argv[2]).write_text(json.dumps(res, indent=1))
    for k, v in res.items():
        for s in ("no_sheets", "all"):
            o = v[s]
            fa = o["factors"]
            print(f"{k:>10} {s:>9}: M4 {o['M4']:.4f} M5 {o['M5']:.4f} ratio {fa['ratio']:.2f} f_near {fa['f_near']:.3f}"
                  f" kappa {fa['kappa']:.4f} s {fa['s']:.3f} land<=200 {fa['land_within_200']:.3f}"
                  f" grid8 {o.get('grid8_M5', float('nan')):.4f} curve "
                  + " ".join(f"{t:g}:{m:.3f}@{lv:.0f}" for t, lv, m in o["curve"]))
    print("M5EARTH_DONE")


if __name__ == "__main__":
    main()
