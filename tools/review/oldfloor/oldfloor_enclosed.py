#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""oldfloor_enclosed.py <argkey folder>: how enclosed by continents the old floor is (the coordinator, 2026-10-27).

Seeds 1 to 4, every third crust-budget snapshot from -900 Myr, every seventh oceanic sample older than 250 Myr: of 16
directions, how many meet continental crust within 2,000 km (marched in 100 km steps on the great circle, the nearest
visible sample), and how many meet its own plate's. Numpy and scipy (the pool's numpy env).
"""
import sys, json, re
from pathlib import Path
import numpy as np
from scipy.spatial import cKDTree
FIELDS = 11
arm = Path(sys.argv[1]); R = 6371.0
def xyz(lat, lon):
    la, lo = np.radians(lat), np.radians(lon)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], -1)
hist = np.zeros(17, np.int64); same = np.zeros(17, np.int64); tot = 0
for run in sorted(p for p in arm.iterdir() if (p / "rec.json").is_file()):
    if int(run.name.rsplit("-", 1)[1]) > 4: continue
    maps = [p for p in sorted((run / "maps").glob("t*.bin"), key=lambda p: float(p.stem[1:])) if float(p.stem[1:]) >= -900][::3]
    for path in maps:
        a = np.fromfile(path, "<f4").reshape(-1, FIELDS); n = len(a)
        ap, pp = run / "maps" / "age" / path.name, run / "maps" / "plate" / path.name
        if not (ap.is_file() and pp.is_file()): continue
        age = np.fromfile(ap, "<f4"); plate = np.fromfile(pp, "<u4")
        if len(age) != n or len(plate) != n: continue
        cont = a[:, 4] > 0.5
        old = np.flatnonzero(~cont & (age > 250))
        if len(old) == 0: continue
        old = old[::7]
        p = xyz(a[:, 0], a[:, 1]); tree = cKDTree(p)
        for i in old:
            o = p[i]
            e1 = np.cross(o, [0, 0, 1.0]); 
            if np.linalg.norm(e1) < 1e-6: e1 = np.cross(o, [1.0, 0, 0])
            e1 /= np.linalg.norm(e1); e2 = np.cross(o, e1)
            hits = 0; hits_same = 0
            for k in range(16):
                th = 2 * np.pi * k / 16; d = np.cos(th) * e1 + np.sin(th) * e2
                for dist in np.arange(100, 2001, 100):
                    ang = dist / R; q = o * np.cos(ang) + d * np.sin(ang)
                    _, j = tree.query(q)
                    if cont[j]:
                        hits += 1; hits_same += plate[j] == plate[i]; break
            hist[hits] += 1; same[hits_same] += 1; tot += 1
print("old>250 samples read", tot)
print("directions (of 16) that meet continent within 2000 km:", {k: round(int(hist[k]) / tot, 3) for k in range(17) if hist[k]})
c = np.cumsum(hist[::-1])[::-1] / tot
print("share with >=12 of 16 enclosed:", round(float(c[12]), 3), " >=8:", round(float(c[8]), 3))
cs = np.cumsum(same[::-1])[::-1] / tot
print("same plate's continent: >=12:", round(float(cs[12]), 3), " >=8:", round(float(cs[8]), 3))
