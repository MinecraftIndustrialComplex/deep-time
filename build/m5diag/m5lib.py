#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Shared helpers of the M5 diagnosis (coast with high ground; build/m5diag/).

The coast and the high ground exactly as tools/review/continents_census.py's coast_metrics counts
them (its helpers, not a copy of its rule): land is z > 0, the coast is land within 1.6 point
spacings of the nearest sea point, the high ground land above `level` m, and a coast point counts
when high ground of the same landmass (cc.components: points joined within 1.6 spacings) lies
within 200 km of it on the sphere. What this module adds: the distance and the index of the
nearest qualifying high point per class of the high ground, so the coast can be split by the
origin of the high ground that qualifies it, and the geometry factors of M5 / M4.
"""
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools" / "review"))
import continents_census as cc  # noqa: E402

REACH_KM = 200.0
BAND = 1.6


def fib(n):
    """N Fibonacci points (lat, lon in degrees), earth-coast.py's way."""
    i = np.arange(n) + 0.5
    lat = np.degrees(np.arcsin(1 - 2 * i / n))
    lon = np.degrees((np.pi * (1 + 5 ** 0.5) * i) % (2 * np.pi) - np.pi)
    return lat, lon


def sea_distance(p, land):
    d = np.zeros(land.size)
    ds, _ = cKDTree(p[~land]).query(p[land])
    d[land] = cc.arc_km(ds)
    return d


def coast_detail(p, z, sp, level=2000.0, cls=None, ncls=1, comp=None, d_sea=None, keep=None):
    """The census's coast and high ground with, per coast point and class of high ground, the
    distance (km) to the nearest high point of that class on the same landmass within 200 km
    (inf if none) and its index. `keep` (as coast_metrics' ~exclude) limits the shares, not the
    geometry."""
    land = z > 0
    if d_sea is None:
        d_sea = sea_distance(p, land)
    coast = land & (d_sea <= BAND * sp)
    hi = land & (z > level)
    if comp is None:
        comp = cc.components(p, land, sp)
    if cls is None:
        cls = np.zeros(z.size, int)
        ncls = 1
    ci = np.nonzero(coast)[0]
    D = np.full((ci.size, ncls), np.inf)
    J = np.full((ci.size, ncls), -1, dtype=np.int64)
    comp_c = comp[ci]
    for k in range(ncls):
        hk = hi & (cls == k)
        for c in np.unique(comp[hk]):
            sel = np.nonzero(comp_c == c)[0]
            if sel.size == 0:
                continue
            hidx = np.nonzero(hk & (comp == c))[0]
            d, j = cKDTree(p[hidx]).query(p[ci[sel]], distance_upper_bound=cc.chord(REACH_KM))
            fin = np.isfinite(d)
            D[sel[fin], k] = cc.arc_km(d[fin])
            J[sel[fin], k] = hidx[j[fin]]
    keep = np.ones(z.size, bool) if keep is None else keep
    return {"land": land, "coast": coast, "hi": hi, "comp": comp, "d_sea": d_sea, "ci": ci, "D": D,
            "J": J, "keep": keep}


def m5_of(det):
    """M5 (the census's coast_with_hi) from coast_detail's result."""
    q = np.isfinite(det["D"]).any(1)
    kc = det["keep"][det["ci"]]
    return float((q & kc).sum() / max(kc.sum(), 1))


def factors(det):
    """M5 = s * f_near * M4 / kappa on equal-area points: M4 = H / L, f_near = the high ground within
    200 km of the sea over H, kappa = coast points over land points, s = qualified coast points
    over the high points within 200 km of the sea. Also the land within 50 / 100 / 200 km of the
    sea (a share of the land) and the coast length L_d = (land within d of the sea) / d per
    1,000 km2 of land, in points (multiply by the point area for km)."""
    keep = det["keep"]
    land, hi, d = det["land"] & keep, det["hi"] & keep, det["d_sea"]
    L, H = land.sum(), hi.sum()
    near = hi & (d <= REACH_KM)
    kc = keep[det["ci"]]
    C = int(kc.sum())
    Q = int((np.isfinite(det["D"]).any(1) & kc).sum())
    out = {"M4": H / max(L, 1), "M5": Q / max(C, 1), "f_near": near.sum() / max(H, 1),
           "kappa": C / max(L, 1), "s": Q / max(near.sum(), 1),
           "ratio": (Q / max(C, 1)) / max(H / max(L, 1), 1e-12)}
    for km in (50, 100, 200):
        out[f"land_within_{km}"] = (land & (d <= km)).sum() / max(L, 1)
    return out


def m5_curve(p, z, sp, comp, d_sea, targets, keep=None):
    """M5 when the high-ground level is set so that the land above it is each target share of the
    land (the same planet with its high ground at Earth's extent):
    [(target, level m, M5, f_near, s)]."""
    k = np.ones(z.size, bool) if keep is None else keep
    zl = z[(z > 0) & k]
    out = []
    for t in targets:
        lev = float(np.quantile(zl, 1.0 - t))
        det = coast_detail(p, z, sp, level=lev, comp=comp, d_sea=d_sea, keep=keep)
        fa = factors(det)
        out.append((t, lev, m5_of(det), float(fa["f_near"]), float(fa["s"])))
    return out


def crossing(p, land, ci, jn, qual):
    """The share of the qualified coast points whose straight line to their nearest qualifying
    high point passes over the sea (9 interior points, each the nearest point's land or sea)."""
    qi = np.nonzero(qual)[0]
    if qi.size == 0:
        return np.zeros(0, bool), qi
    a0, b0 = p[ci[qi]], p[jn[qi]]
    tree = cKDTree(p)
    crosses = np.zeros(qi.size, bool)
    for t in np.linspace(0.1, 0.9, 9):
        m = a0 * (1 - t) + b0 * t
        m /= np.linalg.norm(m, axis=1)[:, None]
        _, jm = tree.query(m)
        crosses |= ~land[jm]
    return crosses, qi


def fixed_measure(p, z, sea_body_km2, n, sp, keep=None):
    """M5 and M4 with the three measure fixes proposed by the diagnosis, one after another, on a
    grid (p, z; sea_body_km2 the area of each sea point's 8-connected sea body, km2; keep as
    coast_detail's, carried to the nearest cell): fib = the grid resampled to n Fibonacci points
    (nearest cell), bodies = then the closed sea bodies under 1 Mkm2 counted as land, inland =
    then a coast point whose straight line to its nearest qualifying high point crosses the sea
    not counted (an upper bound of an over-land reach)."""
    flat, flon = fib(n)
    pf = cc.xyz(flat, flon)
    _, nf = cKDTree(p).query(pf)
    kf = np.ones(n, bool) if keep is None else keep[nf]
    out = {}
    zf = z[nf]
    det = coast_detail(pf, zf, sp, keep=kf)
    out["fib"] = (m5_of(det), float((zf[(zf > 0) & kf] > 2000).mean()))
    zb = zf.copy()
    zb[(zb <= 0) & (sea_body_km2[nf] < 1e6)] = 1.0
    det = coast_detail(pf, zb, sp, keep=kf)
    out["fib_bodies"] = (m5_of(det), float((zb[(zb > 0) & kf] > 2000).mean()))
    q = np.isfinite(det["D"][:, 0])
    cr, qi = crossing(pf, det["land"], det["ci"], det["J"][:, 0], q)
    kc = kf[det["ci"]]
    out["fib_bodies_inland"] = (float((kc[qi] & ~cr).sum() / max(kc.sum(), 1)), out["fib_bodies"][1])
    return out


def dist_hist(dmin, edges=(0.0, 50.0, 100.0, 150.0, 200.0)):
    """Shares of the qualified coast points by the distance to their nearest qualifying high
    point: the coast point itself high (0), then (0, 50], (50, 100], (100, 150], (150, 200]."""
    q = np.isfinite(dmin)
    n = max(q.sum(), 1)
    out = {"self": float((q & (dmin <= 1e-6)).sum() / n)}
    for a, b in zip(edges[:-1], edges[1:]):
        out[f"{a:g}-{b:g}"] = float((q & (dmin > max(a, 1e-6)) & (dmin <= b)).sum() / n)
    return out
