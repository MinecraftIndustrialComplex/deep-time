#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Earth's subduction zones classed as the model classes its fronts (no simulation):

    earth_fronts.py <PB2002_steps.dat> <crust1.bnds> [--age <Seton2020 .grd>] [threshold_km ...]

Bird 2003's subduction steps (PB2002_steps.dat, class SUB; the title's third byte "/" puts the right-hand plate under
the left-hand one, "\\" the opposite, the 2001GC000252 readme) are classed by the crust behind them on the overriding
side, as PLT-370 classes the model's fronts: ocean-continent where continental crust lies within the arc's distance
(PLT-042, 200 km) of the front, else ocean-ocean. Continental crust is a CRUST1.0 cell (Laske et al. 2013, 1 x 1
degree; the boundaries file, depths of each layer's top) whose crystalline crust (upper crust's top to the Moho) is at
least the threshold: 15 km, the model's own line (PLT-047: 7 km of ocean crust and 8 km of arc crust turn a sample
continental), and 20 and 25 km for the reading's sensitivity. The crust is looked up every 20 km from 20 to 200 km
along the great circle normal to each step, on the overriding side (the trench cell itself left out). Prints the SUB
length, the ocean-ocean share and length by threshold, and the ocean-ocean length by the steps' plate pairs; then, as
oofronts.py bands the model's ocean-ocean fronts, the SUB length by the distance along that normal to the first
continental crust (out to 2,400 km; an upper bound of the nearest distance, as the model's lattice distance is), and
the ocean-ocean share of the fronts it bands by that distance.

With --age (Seton et al. 2020's present-day ocean floor age, the NetCDF-3 6-minute grid, z in Myr, NaN off the dated
floor) it walks each ocean-ocean step's overriding-side normal from 50 km every 50 km to 1,500 km, as transect.py
walks the model's, and of the walks that meet continental crust prints the distance and the floor's age along the
strip between: the first and the last dated floor (the arcs and their forearcs carry no age), at the middle, and
where the youngest lies (young at the front, young behind, about even, by 10 Myr). Also the first dated floor's age
behind every ocean-ocean step, anywhere from 50 km and from 200 to 600 km (behind the arc, as transect.py reads the
model's). No plate bounds the walk (the steps file
names the plates but this reads no plate outlines), so it can cross a back-arc's spreading axis.
"""
import math
import sys
from collections import defaultdict

import numpy as np

R_KM = 6371.0
ARC_KM = 200.0  # ledger: PLT-042
FAR_KM = 2400
BANDS = [0.0, 200.0, 300.0, 600.0, 1200.0, 2400.0]
# plates/dynamics.rs NORMAL_BANDS_MM_YR: the model's subducting fronts by their convergence across the boundary.
NORMAL = [0.0, 5.0, 10.0, 20.0, 40.0, 80.0]


def read_steps(path):
    rows = []
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            if len(line) < 96 or not line.strip():
                continue
            rows.append({
                "bid": line[6:11],
                "lon0": float(line[12:20]), "lat0": float(line[21:28]),
                "lon1": float(line[29:37]), "lat1": float(line[38:45]),
                "length": float(line[46:51]), "cls": line[92:95],
                # The relative velocity's component across the step (Bird 2003, Table 2; negative converges), mm/yr.
                "div": float(line.split()[10]),
            })
    return rows


def read_crust(path):
    b = np.loadtxt(path).reshape(180, 360, 9)
    # Crystalline crust: the upper crust's top (layer 6, index 5) to the Moho (index 8), km.
    return b[:, :, 5] - b[:, :, 8]


def unit(lon, lat):
    lo, la = math.radians(lon), math.radians(lat)
    return np.array([math.cos(la) * math.cos(lo), math.cos(la) * math.sin(lo), math.sin(la)])


def read_age(path):
    from scipy.io import netcdf_file
    f = netcdf_file(path, "r", mmap=False)
    lon = np.array(f.variables["lon"][:], dtype=np.float64)
    lat = np.array(f.variables["lat"][:], dtype=np.float64)
    z = np.array(f.variables["z"][:], dtype=np.float64)
    f.close()
    return lon, lat, z


def age_at(grid, p):
    lon, lat, z = grid
    la = math.degrees(math.asin(max(-1.0, min(1.0, p[2]))))
    lo = math.degrees(math.atan2(p[1], p[0]))
    i = int(round((la - lat[0]) / (lat[1] - lat[0])))
    j = int(round((lo - lon[0]) / (lon[1] - lon[0])))
    i = min(len(lat) - 1, max(0, i))
    j = min(len(lon) - 1, max(0, j))
    return z[i, j]


def cell(p):
    lat = math.degrees(math.asin(max(-1.0, min(1.0, p[2]))))
    lon = math.degrees(math.atan2(p[1], p[0]))
    i = min(179, max(0, int(math.floor(89.5 - lat + 0.5))))
    j = min(359, max(0, int(math.floor(lon + 179.5 + 0.5))))
    return i, j


def walk_strip(st, grid, crust, m, over, th, km):
    """One ocean-ocean step's overriding side, every 50 km from 50 to 1,500 km (transect.py's walk)."""
    ages, hit = [], None
    for d in range(50, 1501, 50):
        ang = d / R_KM
        p = m * math.cos(ang) + over * math.sin(ang)
        i, j = cell(p)
        if crust[i, j] >= th:
            hit = d
            break
        a = age_at(grid, p)
        if np.isfinite(a):
            ages.append(a)
    behind = None
    for d in range(200, 601, 50):
        ang = d / R_KM
        p = m * math.cos(ang) + over * math.sin(ang)
        i, j = cell(p)
        if crust[i, j] >= th:
            break
        a = age_at(grid, p)
        if np.isfinite(a):
            behind = a
            break
    if behind is None:
        st["behind_none"] += km
    else:
        for lo, hi in ((0, 10), (10, 30), (30, 60), (60, 120), (120, 1e9)):
            if lo <= behind < hi:
                st[f"behind_age_{lo}"] += km
    first = ages[0] if ages else None
    if first is None:
        st["first_undated"] += km
    else:
        for lo, hi in ((0, 10), (10, 30), (30, 60), (60, 120), (120, 1e9)):
            if lo <= first < hi:
                st[f"first_age_{lo}"] += km
    if hit is None:
        st["no_continent_1500"] += km
        return
    band = next(f"{lo:.0f}" for lo, hi in zip(BANDS, BANDS[1:] + [1e9]) if lo < hit <= hi)
    st[f"hit_{band}"] += km
    if not ages:
        st["hit_undated"] += km
        return
    front, back, mid = ages[0], ages[-1], ages[len(ages) // 2]
    shape = "young_behind" if back < front - 10.0 else "young_at_front" if front < back - 10.0 else "even"
    st[f"shape_{shape}"] += km
    st["n_hit"] += km
    st["sum_front"] += front * km
    st["sum_mid"] += mid * km
    st["sum_back"] += back * km


def main(argv):
    grid = None
    if "--age" in argv:
        k = argv.index("--age")
        grid = read_age(argv[k + 1])
        argv = argv[:k] + argv[k + 2:]
    steps = [s for s in read_steps(argv[0]) if s["cls"] == "SUB"]
    crust = read_crust(argv[1])
    thresholds = [float(x) for x in argv[2:]] or [15.0, 20.0, 25.0]
    strip = {t: defaultdict(float) for t in thresholds}
    total = sum(s["length"] for s in steps)
    oo = {t: 0.0 for t in thresholds}
    conv = {t: [0.0, 0.0, 0.0, 0.0] for t in thresholds}
    conv_bands = {t: [[0.0] * len(NORMAL), [0.0] * len(NORMAL)] for t in thresholds}
    by_pair = {t: defaultdict(float) for t in thresholds}
    dist = {t: defaultdict(float) for t in thresholds}
    for s in steps:
        a, b = unit(s["lon0"], s["lat0"]), unit(s["lon1"], s["lat1"])
        m = a + b
        m /= np.linalg.norm(m)
        t = b - a
        t -= m * m.dot(t)
        t /= np.linalg.norm(t)
        left = np.cross(m, t)
        pol = s["bid"][2]
        over = left if pol == "/" else -left
        thick = []
        for d in range(20, FAR_KM + 1, 20):
            ang = d / R_KM
            p = m * math.cos(ang) + over * math.sin(ang)
            i, j = cell(p)
            thick.append((d, crust[i, j]))
        for th in thresholds:
            is_oo = max(c for d, c in thick if d <= ARC_KM) < th
            k = 0 if is_oo else 2
            conv[th][k] += -s["div"] * s["length"]
            conv[th][k + 1] += s["length"]
            b = max(i for i, lo in enumerate(NORMAL) if -s["div"] >= lo) if -s["div"] >= 0.0 else 0
            conv_bands[th][0 if is_oo else 1][b] += s["length"]
            if is_oo:
                oo[th] += s["length"]
                by_pair[th][s["bid"]] += s["length"]
            if grid is not None and max(c for d, c in thick if d <= ARC_KM) < th:
                walk_strip(strip[th], grid, crust, m, over, th, s["length"])
            first = next((d for d, c in thick if c >= th), None)
            # (lo, hi] bands, so a crust at 200 km is in the first, as the class above has it; "none" last.
            band = len(BANDS) - 1 if first is None else max(k for k, lo in enumerate(BANDS[:-1]) if first > lo)
            dist[th][band] += s["length"]
    print(f"# Bird 2003 SUB steps: {len(steps)}, {total:.0f} km")
    for th in thresholds:
        c = conv[th]
        print(f"threshold {th:4.0f} km: ocean-ocean {oo[th]:7.0f} km, share {oo[th] / total:.3f}; convergence across"
              f" the step, mm/yr: ocean-ocean {c[0] / max(c[1], 1e-9):.1f}, ocean-continent {c[2] / max(c[3], 1e-9):.1f},"
              f" all {(c[0] + c[2]) / max(c[1] + c[3], 1e-9):.1f}")
    print("# SUB km by the convergence across the step, the model's bands (0-5, 5-10, 10-20, 20-40, 40-80, >80 mm/yr;"
          " shares of the class)")
    for th in thresholds:
        for k, cls in enumerate(("ocean-ocean", "ocean-continent")):
            row = conv_bands[th][k]
            tot = max(sum(row), 1e-9)
            print(f"threshold {th:4.0f} km {cls:16s}: " + "  ".join(f"{x / tot:.3f}" for x in row))
    names = [f"{lo:.0f}-{hi:.0f}" for lo, hi in zip(BANDS, BANDS[1:])] + ["none"]
    print(f"# SUB km by the distance to the first continental crust along the overriding-side normal (none within"
          f" {FAR_KM} km); in brackets the share of the fronts past {ARC_KM:.0f} km")
    for th in thresholds:
        past = sum(v for k, v in dist[th].items() if k >= 1)
        cells = "  ".join(f"{n} {dist[th][k]:6.0f}" + (f" ({dist[th][k] / past:.3f})" if k >= 1 and past else "")
                          for k, n in enumerate(names))
        print(f"threshold {th:4.0f} km: {cells}")
    if grid is not None:
        for th in thresholds:
            st = strip[th]
            oo_km = oo[th]
            print(f"# threshold {th:.0f} km: the ocean-ocean steps' overriding side walked to 1,500 km (shares of the"
                  f" ocean-ocean length)")
            for k in sorted(k for k in st if not k.startswith(("sum_", "n_"))):
                print(f"  {k:30s} {st[k] / oo_km:.3f}")
            n = st.get("n_hit", 0.0)
            if n:
                print("  mean dated floor age on the strip before the continent, Myr: " + "  ".join(
                    f"{nm} {st[f'sum_{nm}'] / n:5.1f}" for nm in ("front", "mid", "back")))
    th = thresholds[0]
    print(f"# ocean-ocean km by Bird's segment title at {th:.0f} km (overriding plate on the '/' left or '\\' right)")
    for k, v in sorted(by_pair[th].items(), key=lambda kv: -kv[1]):
        print(f"  {k} {v:6.0f}")


if __name__ == "__main__":
    main(sys.argv[1:])
