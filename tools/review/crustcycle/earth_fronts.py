#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Earth's subduction zones classed as the model classes its fronts (no simulation):

    earth_fronts.py <PB2002_steps.dat> <crust1.bnds> [threshold_km ...]

Bird 2003's subduction steps (PB2002_steps.dat, class SUB; the title's third byte "/" puts the right-hand plate under
the left-hand one, "\\" the opposite, the 2001GC000252 readme) are classed by the crust behind them on the overriding
side, as PLT-370 classes the model's fronts: ocean-continent where continental crust lies within the arc's distance
(PLT-042, 200 km) of the front, else ocean-ocean. Continental crust is a CRUST1.0 cell (Laske et al. 2013, 1 x 1
degree; the boundaries file, depths of each layer's top) whose crystalline crust (upper crust's top to the Moho) is at
least the threshold: 15 km, the model's own line (PLT-047: 7 km of ocean crust and 8 km of arc crust turn a sample
continental), and 20 and 25 km for the reading's sensitivity. The crust is looked up every 20 km from 20 to 200 km
along the great circle normal to each step, on the overriding side (the trench cell itself left out). Prints the SUB
length, the ocean-ocean share and length by threshold, and the ocean-ocean length by the steps' plate pairs.
"""
import math
import sys
from collections import defaultdict

import numpy as np

R_KM = 6371.0
ARC_KM = 200.0  # ledger: PLT-042


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
            })
    return rows


def read_crust(path):
    b = np.loadtxt(path).reshape(180, 360, 9)
    # Crystalline crust: the upper crust's top (layer 6, index 5) to the Moho (index 8), km.
    return b[:, :, 5] - b[:, :, 8]


def unit(lon, lat):
    lo, la = math.radians(lon), math.radians(lat)
    return np.array([math.cos(la) * math.cos(lo), math.cos(la) * math.sin(lo), math.sin(la)])


def cell(p):
    lat = math.degrees(math.asin(max(-1.0, min(1.0, p[2]))))
    lon = math.degrees(math.atan2(p[1], p[0]))
    i = min(179, max(0, int(math.floor(89.5 - lat + 0.5))))
    j = min(359, max(0, int(math.floor(lon + 179.5 + 0.5))))
    return i, j


def main(argv):
    steps = [s for s in read_steps(argv[0]) if s["cls"] == "SUB"]
    crust = read_crust(argv[1])
    thresholds = [float(x) for x in argv[2:]] or [15.0, 20.0, 25.0]
    total = sum(s["length"] for s in steps)
    oo = {t: 0.0 for t in thresholds}
    by_pair = {t: defaultdict(float) for t in thresholds}
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
        for d in range(20, int(ARC_KM) + 1, 20):
            ang = d / R_KM
            p = m * math.cos(ang) + over * math.sin(ang)
            i, j = cell(p)
            thick.append(crust[i, j])
        for th in thresholds:
            if max(thick) < th:
                oo[th] += s["length"]
                by_pair[th][s["bid"]] += s["length"]
    print(f"# Bird 2003 SUB steps: {len(steps)}, {total:.0f} km")
    for th in thresholds:
        print(f"threshold {th:4.0f} km: ocean-ocean {oo[th]:7.0f} km, share {oo[th] / total:.3f}")
    th = thresholds[0]
    print(f"# ocean-ocean km by Bird's segment title at {th:.0f} km (overriding plate on the '/' left or '\\' right)")
    for k, v in sorted(by_pair[th].items(), key=lambda kv: -kv[1]):
        print(f"  {k} {v:6.0f}")


if __name__ == "__main__":
    main(sys.argv[1:])
