#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Earth's collision belts per km of collision front, the control of collfront.py (front lane, 2026-10-25):

    earthcoll.py <crust1.bnds> <PB2002_steps.dat.txt>

CRUST1.0 (Laske et al. 2013, 1 x 1 degree; read as m4earth.py reads it: T = the sediments' top minus the
Moho, land above 0 m with no water, the rock height the sediments' top) and PB2002's steps (Bird 2003;
the class in columns 93 to 95 as tools/review/pool/bird/bird.py reads it, the convergence the negative of
the divergent component, the polarity the boundary id's third byte: '/' the right plate goes under the
left, '\\' the left under the right). The collision fronts are the CCB steps (sampled every 10 km or
less). Each cell gets its nearest CCB point, its distance and its side (the overriding plate's or the
incoming plate's, from the step's left and right and the polarity; 'unknown' with no polarity, which is
every CCB step: PB2002 marks CCB '-', so the tables give both sides together).

Prints: the CCB steps by region (boxes on the step's midpoint) with their length and length-weighted
convergence; per region and over all CCB, per km of that region's CCB: the excess crust over a normal
column (sum max(0, T - T_ref) x area, km3 per km, T_ref 35.1 km as the model's and 40 km), by distance
band, the area of crust 45 km and 55 km or thicker and of land above 2 km by rock within 800 km
(km2 per km), and the mean thickness by band; and meridian cross-sections (excess crust per km of
strike, km2, and the width and mean thickness of crust 45 km or thicker) of the Himalaya-Tibet, the Tien
Shan and the Alps."""
import sys

import numpy as np
from scipy.spatial import cKDTree

R = 6371.0
DEG_KM = np.pi * R / 180.0
T_REFS = (35.1, 40.0)
D_EDGES = [0.0, 100.0, 200.0, 400.0, 800.0, 1500.0]
SIDES = ("over", "under", "unknown")
REGIONS = [
    ("Himalaya (India-Asia)", (20.0, 40.0, 65.0, 100.0)),
    ("Zagros, Bitlis, Caucasus (Arabia-Eurasia)", (24.0, 46.0, 35.0, 65.0)),
    ("Alps, Apennines, Dinarides, Aegean, Maghreb", (30.0, 50.0, -12.0, 35.0)),
]


def xyz(lat, lon):
    la, lo = np.radians(lat), np.radians(lon)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], -1)


def read_steps(path):
    rows = []
    for line in open(path, encoding="utf-8", errors="replace"):
        if not line.strip():
            continue
        v = line[11:92].split()
        rows.append({"bid": line[6:11], "lon0": float(v[0]), "lat0": float(v[1]), "lon1": float(v[2]),
                     "lat1": float(v[3]), "length": float(v[4]), "div": float(v[8]), "cls": line[92:95]})
    return rows


def region_of(lat, lon):
    for i, (_, (la0, la1, lo0, lo1)) in enumerate(REGIONS):
        if la0 <= lat < la1 and lo0 <= lon < lo1:
            return i
    return len(REGIONS)


def main():
    b = np.loadtxt(sys.argv[1])
    assert b.shape == (64800, 9), b.shape
    lat = np.repeat(np.arange(89.5, -90, -1.0), 360)
    lon = np.tile(np.arange(-179.5, 180, 1.0), 180)
    area = DEG_KM ** 2 * np.cos(np.radians(lat))
    top, ice_top, rock, moho = b[:, 0] * 1000, b[:, 1] * 1000, b[:, 2] * 1000, b[:, 8] * 1000
    T = (rock - moho) / 1000.0
    land = ((top - ice_top) <= 0.0) & (top > 0.0)
    hi = land & (rock > 2000.0)
    print(f"Earth's land {area[land].sum() / 1e6:.1f} million km2; land above 2 km by rock {area[hi].sum() / 1e6:.2f}")

    steps = [s for s in read_steps(sys.argv[2]) if s["cls"] == "CCB"]
    names = [REGIONS[i][0] for i in range(len(REGIONS))] + ["other CCB"]
    reg_len = np.zeros(len(names))
    reg_conv = np.zeros(len(names))
    by_bid = {}
    pts, owner, left_over = [], [], []
    for j, s in enumerate(steps):
        p0, p1 = xyz(s["lat0"], s["lon0"]), xyz(s["lat1"], s["lon1"])
        mid = (p0 + p1) / np.linalg.norm(p0 + p1)
        mlat, mlon = np.degrees(np.arcsin(mid[2])), np.degrees(np.arctan2(mid[1], mid[0]))
        r = region_of(mlat, mlon)
        s["region"] = r
        reg_len[r] += s["length"]
        reg_conv[r] += s["length"] * (-s["div"])
        key = (r, s["bid"])
        by_bid[key] = by_bid.get(key, 0.0) + s["length"]
        n = max(2, int(np.ceil(s["length"] / 10.0)) + 1)
        for t in np.linspace(0.0, 1.0, n):
            x = p0 * (1 - t) + p1 * t
            pts.append(x / np.linalg.norm(x))
            owner.append(j)
        pol = s["bid"][2]
        left_over.append(1 if pol == "/" else (0 if pol == "\\" else -1))
    pts, owner = np.array(pts), np.array(owner)
    left_over = np.array(left_over)
    normals = np.array([np.cross(xyz(s["lat0"], s["lon0"]), xyz(s["lat1"], s["lon1"])) for s in steps])
    print(f"CCB steps {len(steps)}, length {reg_len.sum():.0f} km, length-weighted convergence"
          f" {reg_conv.sum() / reg_len.sum():.1f} mm/yr")
    for r, nm in enumerate(names):
        bids = ", ".join(f"{k[1]} {v:.0f}" for k, v in sorted(by_bid.items(), key=lambda kv: -kv[1]) if k[0] == r)
        print(f"  {nm}: {reg_len[r]:.0f} km, convergence {reg_conv[r] / max(reg_len[r], 1):.1f} mm/yr ({bids})")

    X = xyz(lat, lon)
    dch, jp = cKDTree(pts).query(X)
    d = 2 * R * np.arcsin(np.clip(dch / 2, 0, 1))
    js = owner[jp]
    left = np.einsum("ij,ij->i", normals[js], X) > 0
    lo = left_over[js]
    side = np.where(lo < 0, 2, np.where((left & (lo == 1)) | (~left & (lo == 0)), 0, 1))
    reg = np.array([steps[j]["region"] for j in js])
    bands = np.searchsorted(np.array(D_EDGES[1:]), d, side="right")
    near = d <= 800.0
    # Earth's own normal column: the area-weighted median of the continental crust (T >= 25 km), all of
    # it and farther than 1500 km from a CCB (collfront.py takes the model's the same two ways).
    cont = T >= 25.0

    def wmed(m):
        o = np.argsort(T[m])
        c = np.cumsum(area[m][o]) / area[m].sum()
        return float(np.interp(0.5, c, T[m][o]))
    t_med, t_far = wmed(cont), wmed(cont & (d > 1500.0))
    print(f"Earth's continental crust (T >= 25 km) median {t_med:.1f} km, farther than 1500 km from a CCB {t_far:.1f} km;"
          f" land median {wmed(land):.1f} km")
    # Planet-wide totals over the CCB length, to set beside collfront.py's "planet" columns.
    L_all = reg_len.sum()
    for tr in (35.1, t_med):
        ex = np.maximum(T - tr, 0.0) * area
        print(f"planet: excess over {tr:.1f} km {ex.sum() / L_all:.0f} km3 per km of CCB; share within 800 km"
              f" {ex[near].sum() / ex.sum():.3f}, within 1500 km {ex[d <= 1500.0].sum() / ex.sum():.3f}")
    for tn, m in (("T >= 45 km", T >= 45), ("T >= 55 km", T >= 55), ("land above 2 km", hi)):
        print(f"planet: {tn:<16} {area[m].sum() / 1e6:5.2f} million km2 ({area[m].sum() / L_all:4.0f} km2 per km of CCB);"
              f" share within 800 km {area[m & near].sum() / area[m].sum():.3f}")
    for r, nm in list(enumerate(names)) + [(None, "all CCB")]:
        sel = np.ones(lat.size, bool) if r is None else (reg == r)
        L = reg_len.sum() if r is None else reg_len[r]
        if L <= 0:
            continue
        print(f"\n{nm}: CCB {L:.0f} km; per km of it:")
        for tr in T_REFS + (t_med, t_far):
            ex = np.maximum(T - tr, 0.0) * area
            print(f"  excess crust over {tr:.1f} km within 800 km {ex[sel & near].sum() / L:.0f} km3/km"
                  f" (within 1500 km {ex[sel & (d <= 1500)].sum() / L:.0f})")
        ex = np.maximum(T - T_REFS[0], 0.0) * area
        # PB2002 gives CCB steps no polarity ('-'), so the sides are not told apart: both sides together.
        print("  band (from_to km, both sides): excess over 35.1 km, km3/km | area km2/km | mean T km |"
              " mean T of crust over 30 km")
        for bb in range(len(D_EDGES) - 1):
            m = sel & (bands == bb)
            mc = m & (T > 30.0)
            if not m.any():
                continue
            tc = np.average(T[mc], weights=area[mc]) if mc.any() else float("nan")
            print(f"    {D_EDGES[bb]:.0f}_{D_EDGES[bb + 1]:.0f}: {ex[m].sum() / L:8.0f} | {area[m].sum() / L:7.0f}"
                  f" | {np.average(T[m], weights=area[m]):5.1f} | {tc:5.1f}")
        print(f"  sides known for {float((sel & near & (side < 2)).sum()) / max(int((sel & near).sum()), 1):.2f} of the cells")
        for tn, m in (("T >= 45 km", T >= 45), ("T >= 55 km", T >= 55), ("land above 2 km", hi)):
            print(f"  {tn:<16} within 800 km {area[sel & near & m].sum() / L:6.0f} km2/km")

    print("\nMeridian cross-sections (per km of strike, the strike taken east-west): excess over 35.1 km, km2;"
          " width of T >= 45 km, km; its mean T; peak T")
    for nm, la0, la1, lons in (("Himalaya-Tibet", 26, 40, (78, 98)), ("Tien Shan", 39, 46, (70, 95)),
                               ("Alps", 43, 49, (6, 16))):
        rows = []
        for lo0 in range(lons[0], lons[1], 1):
            m = (lon > lo0) & (lon < lo0 + 1) & (lat > la0) & (lat < la1)
            exs = (np.maximum(T[m] - 35.1, 0.0) * DEG_KM).sum()
            w45 = (T[m] >= 45).sum() * DEG_KM
            t45 = T[m][T[m] >= 45].mean() if (T[m] >= 45).any() else float("nan")
            rows.append((exs, w45, t45, T[m].max()))
        a = np.array(rows)
        print(f"  {nm} ({la0}-{la1} N, {lons[0]}-{lons[1]} E): excess p50 {np.median(a[:, 0]):.0f} (mean {a[:, 0].mean():.0f}) km2;"
              f" width p50 {np.median(a[:, 1]):.0f} km; mean T in it {np.nanmedian(a[:, 2]):.1f} km; peak T p50 {np.median(a[:, 3]):.1f} km")


if __name__ == "__main__":
    main()
