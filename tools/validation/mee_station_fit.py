#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The plateau's warmth (the mass-elevation effect) on Earth's relief against plateau stations
(docs/science/mass-elevation-effect.md, ledger CLM-225 to CLM-231).

    deeptime climate-dem --dem earth720.f32 --width 720 --dump out/ > out.log
    python3 tools/validation/mee_station_fit.py out/ [more dump folders ...]

From the first folder: the footprint sweep (the base height Z^ of `lapse::mass_elevation_base`
redone here on the dump's z.f32, each footprint refitted), each station's Z^ and relative sunlight,
and the least-squares fit of MEE_K_PER_M and MEE_Q0 (CLM-226) with the code's clamp (CLM-227) and
footprint (CLM-225). For every folder: the model's warmest and coldest month at each station's own
height, and the glacier share, warmest month and mean temperature of six regions.

The stations' warmth (the `ST` table, CLM-229) is each station's 1991-2020 monthly normal minus the
free air at the station's height over the nearby lowlands (NCEP/NCAR Reanalysis 1 long-term means,
1991-2020, interpolated in height between pressure levels), as the midpoint of the range over
several lowland columns. The normals were read in Wikipedia's climate boxes (copies of the CMA's,
SENAMHI's and NOAA's tables) on 2026-10-21; R1 was read from NOAA PSL's long-term-mean files.
The base height ignores the ice mask (CLM-228): on Earth's relief no station's surroundings are
under ice in the last feedback snapshot. Validation only; nothing here is shipped.
"""
import json
import math
import sys

import numpy as np

S0 = 1361.0  # W/m2; the ratio q/(S0/4) does not depend on it
OBL = math.radians(23.44)
YEAR_START = math.radians(280.0)  # month 1 starts at solar longitude 280 (insolation.rs)
FOOT_KM = 150.0  # lapse.rs MEE_FOOTPRINT_KM (CLM-225)
Q_MIN, Q_MAX = 0.60, 1.42  # lapse.rs MEE_Q_MIN, MEE_Q_MAX (CLM-227)
K_CODE, Q0_CODE = 2.43e-3, 0.792  # lapse.rs MEE_K_PER_M, MEE_Q0 (CLM-226)
GAMMA = 6.5e-3  # K/m, GLA-038

# name, lat, lon, station height m, obs warmest month (1-12), its normal degC, obs coldest month,
# its normal (None: not read), warmth over the free air in the warmest month, the year and the
# coldest month (K; None: not known), group, used in the fit (Lhasa is a valley station, Pikes
# Peak a lone summit with low-confidence normals).
ST = [
    ("Nagqu", 31.48, 92.05, 4507, 7, 10.0, 1, -11.3, 5.35, 1.8, -2.2, "tibet", True),
    ("Tingri", 28.63, 87.08, 4300, 7, 12.6, 1, -6.4, 6.9, 4.25, 1.45, "tibet", True),
    ("Shiquanhe", 32.50, 80.08, 4279, 7, 15.0, 1, -11.7, 9.15, 2.2, None, "tibet", True),
    ("Madoi", 34.92, 98.22, 4272, 7, 8.4, 1, -15.2, 4.2, 3.15, None, "tibet", True),
    ("Lhasa", 29.67, 91.13, 3649, 7, 17.5, 1, None, 8.9, 7.05, None, "valley", False),
    ("El Alto", -16.51, -68.19, 4058, 12, 9.9, 7, 5.1, 4.85, 3.4, 0.9, "andes", True),
    ("Puno", -15.84, -70.02, 3825, 12, 11.4, 7, 7.1, 5.2, 3.9, 1.5, "andes", True),
    ("Juliaca", -15.47, -70.16, 3838, 12, 11.1, 7, None, 4.9, 3.05, None, "andes", True),
    ("Oruro", -17.97, -67.11, 3735, 12, 12.5, 7, None, 5.8, 3.35, None, "andes", True),
    ("Leadville", 39.25, -106.29, 3095, 7, 12.8, 1, None, 1.5, -1.1, None, "rockies", True),
    ("Climax", 39.37, -106.19, 3457, 7, 11.9, 1, None, 3.0, -0.25, None, "rockies", True),
    ("Alamosa", 37.44, -105.87, 2299, 7, 18.5, 1, None, 2.2, -0.75, None, "rockies", True),
    ("Pikes Peak", 38.84, -105.04, 4302, 7, 4.8, 1, None, 1.65, -2.45, None, "rockies", False),
    ("Addis Ababa", 9.0, 38.75, 2355, 7, 18.8, 1, None, 2.25, 2.05, None, "ethiopia", True),
]
REGIONS = {
    "tibet": (27.0, 40.0, 75.0, 103.0),
    "andes": (-24.0, -13.0, -72.0, -64.0),
    "rockies": (35.0, 45.0, -113.0, -104.0),
    "ethiopia": (5.0, 15.0, 35.0, 42.0),
    "greenland": (59.0, 84.0, -74.0, -10.0),
    "antarctica": (-90.0, -60.0, -180.0, 180.0),
}


def daily(lat, lam):
    """Daily-mean insolation at the top of the atmosphere, circular orbit, W/m2."""
    dec = math.asin(math.sin(OBL) * math.sin(lam))
    x = -math.tan(lat) * math.tan(dec)
    h0 = math.pi if x <= -1 else (0.0 if x >= 1 else math.acos(x))
    return S0 / math.pi * (h0 * math.sin(lat) * math.sin(dec) + math.cos(lat) * math.cos(dec) * math.sin(h0))


def monthly(lat_deg, m, sub=4):
    lat = math.radians(lat_deg)
    return sum(daily(lat, (YEAR_START + 2 * math.pi * (m + (k + 0.5) / sub) / 12.0) % (2 * math.pi))
               for k in range(sub)) / sub


def blur(v, r, passes=2):
    """grid.rs `box_blur`: a box of half-width r, rows wrapping east-west, columns mirrored."""
    v = v.copy()
    h, w = v.shape
    for _ in range(passes):
        c = np.cumsum(np.concatenate([v[:, -r:], v, v[:, :r + 1]], axis=1), axis=1)
        cs = np.concatenate([np.zeros((h, 1)), c], axis=1)
        v = (cs[:, 2 * r + 1:2 * r + 1 + w] - cs[:, 0:w]) / (2 * r + 1)
        idx = np.arange(-r, h + r + 1)
        idx = np.where(idx < 0, -idx - 1, np.where(idx >= h, 2 * h - idx - 1, idx))
        p = v[np.clip(idx, 0, h - 1)]
        cs = np.concatenate([np.zeros((1, w)), np.cumsum(p, axis=0)], axis=0)
        v = (cs[2 * r + 1:2 * r + 1 + h] - cs[0:h]) / (2 * r + 1)
    return v


def load(d):
    meta = json.load(open(f"{d}/meta.json"))
    w, h = meta["w"], meta["h"]
    lat = np.array(meta["lat"])

    def rd(n):
        return np.fromfile(f"{d}/{n}.f32", dtype="<f4").astype(np.float64).reshape(h, w)

    f = {k: rd(k) for k in ("z", "glacier", "t_mean", "t_warm", "t_cold")}
    f["tsl"] = np.stack([rd(f"t_sl_{m + 1:02d}") for m in range(12)])
    return meta, lat, f


def cell(lat, w, la, lo):
    j = int(np.argmin(np.abs(lat - la)))
    i = int(math.floor((lo % 360.0) / 360.0 * w)) % w
    return j, i


def fit_rows(zhat, qrel):
    """MEE_m = K Z^ clamp(q_m) - K Q0 Z^, on the warmest month, the year and the coldest month."""
    rows, ys, tags = [], [], []
    for s in ST:
        if not s[12]:
            continue
        z = zhat[s[0]]
        q = np.clip(qrel[s[0]], Q_MIN, Q_MAX)
        for kind, m, y in (("warm", s[4] - 1, s[8]), ("cold", s[6] - 1, s[10])):
            if y is not None:
                rows.append([z * q[m], -z])
                ys.append(y)
                tags.append((s[0], kind, s[11]))
        if s[9] is not None:
            rows.append([z * q.mean(), -z])
            ys.append(s[9])
            tags.append((s[0], "annual", s[11]))
    return np.array(rows), np.array(ys), tags


def report(a, y, tags, k, q0, label):
    pred = a @ np.array([k, k * q0])
    print(f"{label}: K {k:.3e} K/m, Q0 {q0:.3f} ({q0 * S0 / 4:.0f} W/m2 on Earth), "
          f"rms {np.sqrt(np.mean((pred - y) ** 2)):.2f} K over {len(y)} station-months")
    for g in ("tibet", "andes", "rockies", "ethiopia"):
        sel = [n for n, t in enumerate(tags) if t[2] == g]
        print(f"  {g:<9} mean residual (model - observed) {np.mean(pred[sel] - y[sel]):+5.2f} K over {len(sel)}")
    print("  " + "  ".join(f"{t[0][:6]}/{t[1][0]} {p:+.1f}({o:+.1f})" for t, p, o in zip(tags, pred, y)))


def main():
    dirs = sys.argv[1:]
    meta, lat, f0 = load(dirs[0])
    w, h = meta["w"], meta["h"]
    dx_km = 40030.0 / w
    z = np.maximum(f0["z"], 0.0)
    qrel = {s[0]: np.array([monthly(s[1], m) for m in range(12)]) / (S0 / 4.0) for s in ST}
    print("footprint sweep (half-width km, cells): refitted K, Q0, rms")
    for fk in (56.0, 111.0, 150.0, 222.0, 333.0, 500.0):
        r = max(1, round(fk / dx_km))
        zb = np.maximum(np.minimum(blur(z, r), z), 0.0)
        a, y, _ = fit_rows({s[0]: zb[cell(lat, w, s[1], s[2])] for s in ST}, qrel)
        sol, *_ = np.linalg.lstsq(a, y, rcond=None)
        rms = np.sqrt(np.mean((a @ sol - y) ** 2))
        print(f"  {fk:5.0f} {r}: K {sol[0]:.3e} Q0 {sol[1] / sol[0]:.3f} rms {rms:.2f}")
    r = max(1, round(FOOT_KM / dx_km))
    zb = np.maximum(np.minimum(blur(z, r), z), 0.0)
    print(f"\ngrid {w}x{h}, cell {dx_km:.1f} km, footprint {FOOT_KM:.0f} km = {r} cells")
    print("station      lat    z_st  z_cell   Zhat  | q_warm q_cold q_year")
    zhat = {}
    for s in ST:
        j, i = cell(lat, w, s[1], s[2])
        zhat[s[0]] = zb[j, i]
        q = qrel[s[0]]
        print(f"{s[0]:<12}{s[1]:6.1f} {s[3]:6d} {z[j, i]:7.0f} {zb[j, i]:6.0f}  | "
              f"{q[s[4] - 1]:6.3f} {q[s[6] - 1]:6.3f} {q.mean():6.3f}")
    a, y, tags = fit_rows(zhat, qrel)
    sol, *_ = np.linalg.lstsq(a, y, rcond=None)
    print()
    report(a, y, tags, sol[0], sol[1] / sol[0], "least squares")
    report(a, y, tags, K_CODE, Q0_CODE, "as coded")
    for d in dirs:
        meta, lat, f = load(d)
        print(f"\n== {d}: t_global {meta['t_global_c']:.2f} C, glaciated land {meta['glaciated_land']:.4f}")
        print("station       obs warm  model warm   obs cold  model cold  (at the station's height)")
        for s in ST:
            j, i = cell(lat, w, s[1], s[2])
            t = f["tsl"][:, j, i] - GAMMA * s[3]
            oc = "" if s[7] is None else f"{s[7]:6.1f}"
            print(f"{s[0]:<12} {s[5]:7.1f} {t.max():9.1f} (m{int(t.argmax()) + 1:02d}) {oc:>8} {t.min():9.1f}")
        lon = (np.arange(w) + 0.5) / w * 360.0
        lon = np.where(lon > 180, lon - 360, lon)
        la, lo = np.meshgrid(lat, lon, indexing="ij")
        land = f["z"] > 0
        for g, (a0, a1, o0, o1) in REGIONS.items():
            box = (la >= a0) & (la <= a1) & (lo >= o0) & (lo <= o1) & land
            for zmin in (0.0, 3000.0):
                sel = box & (f["z"] > zmin)
                if sel.sum() == 0:
                    continue
                tw = f["t_warm"][sel]
                print(f"  {g:<10} z>{zmin:4.0f}: {sel.sum():5d} cells, glacier {f['glacier'][sel].mean():.3f}, "
                      f"warm p10/50/90 {np.percentile(tw, 10):5.1f}/{np.percentile(tw, 50):5.1f}/"
                      f"{np.percentile(tw, 90):5.1f}, mean T {f['t_mean'][sel].mean():5.1f}")


if __name__ == "__main__":
    main()
