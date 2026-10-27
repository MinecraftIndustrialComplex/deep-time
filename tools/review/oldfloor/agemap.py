#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""agemap.py <argkey folder> <out.png>: the floor's age on the last snapshot of the run (seeds 1 to 4) with the most
floor older than 250 Myr, as an equirectangular PNG written with zlib alone (no matplotlib in the pool's env): the floor
blue (young) through green to yellow (300 Myr), white over 250 Myr, continents tan, TRENCH red, subduction fronts magenta,
collision fronts black (the coordinator, 2026-10-27).
"""
import sys, zlib, struct
from pathlib import Path
import numpy as np
FIELDS = 11
arm = Path(sys.argv[1]); out = sys.argv[2]
best = None
for run in sorted(p for p in arm.iterdir() if (p / "rec.json").is_file()):
    if int(run.name.rsplit("-", 1)[1]) > 4:
        continue
    maps = sorted((run / "maps").glob("t*.bin"), key=lambda p: float(p.stem[1:]))
    for path in reversed(maps):
        a = np.fromfile(path, "<f4").reshape(-1, FIELDS)
        ap = run / "maps" / "age" / path.name
        if not ap.is_file(): continue
        age = np.fromfile(ap, "<f4")
        if len(age) != len(a): continue
        old = int(np.nansum((a[:, 4] <= 0.5) & (age > 250)))
        if best is None or old > best[0]:
            best = (old, run, path)
        break
old, run, path = best
a = np.fromfile(path, "<f4").reshape(-1, FIELDS); age = np.fromfile(run / "maps" / "age" / path.name, "<f4")
W, H = 1440, 720
lat, lon = a[:, 0], a[:, 1]; cont = a[:, 4] > 0.5; fl = np.rint(a[:, 10]).astype(int)
x = ((lon + 180) / 360 * W).astype(int) % W; y = ((90 - lat) / 180 * H).clip(0, H - 1).astype(int)
img = np.zeros((H, W, 3), np.uint8); img[:] = (30, 30, 30)
def col(age):
    t = np.clip(age / 300.0, 0, 1)[:, None]
    young = np.array([60, 140, 230]); mid = np.array([40, 190, 120]); oldc = np.array([250, 220, 60])
    c = np.where(t < 0.5, young + (mid - young) * (t / 0.5), mid + (oldc - mid) * ((t - 0.5) / 0.5))
    return c.astype(np.uint8)
oc = ~cont
cols = np.zeros((len(a), 3), np.uint8)
cols[oc] = col(np.nan_to_num(age[oc]))
cols[cont] = (150, 120, 90)
cols[oc & (age > 250)] = (255, 255, 255)
cols[(fl & 2) != 0] = (230, 30, 30)
cols[(fl & 256) != 0] = (230, 30, 230)
cols[(fl & 512) != 0] = (0, 0, 0)
for dy in (-1, 0, 1):
    for dx in (-1, 0, 1):
        img[(y + dy).clip(0, H - 1), (x + dx) % W] = cols
raw = b"".join(b"\x00" + img[r].tobytes() for r in range(H))
def chunk(t, d): return struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d) & 0xffffffff)
png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", W, H, 8, 2, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw, 6)) + chunk(b"IEND", b"")
open(out, "wb").write(png)
print(run.name, path.stem, old, "samples older than 250 Myr of", int(oc.sum()), "ocean")
