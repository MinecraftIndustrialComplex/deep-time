#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
# The deepest sea floor of a bake's elevation layer (L3, blocks), and the deepest spot of each
# separate deep region. args: <deeptime folder>. From the coordinator's build/coord/trench/trench.py
# (2026-10-22), kept for the trench budget with the share of the floor deeper than 5 to 10 km added
# (Earth's: tools/review/earth-deep-floor.py; ledger HYP-050, HYP-051). Shares are of grid cells,
# not of area: Petroff-Guyou is conformal, not equal-area. Run on the VM (python3 with numpy, lz4).
import json, struct, sys
import numpy as np, lz4.block
d = sys.argv[1]
m = json.load(open(d + "/manifest.json"))
sc = m["scale"]; sea = sc["sea_level_y"]; mv = sc["m_per_block_v"]
f = open(d + "/rasters/elevation.dtr", "rb").read()
hl = struct.unpack_from("<I", f, 4)[0]
desc = json.loads(f[16:hl]); g = desc["grid"]; W, H, r, T = g["w"], g["h"], g["r"], desc["tile"]
io = (hl + 7) // 8 * 8
tx_n, tz_n = -(-W // T), -(-H // T)
E = np.empty((H, W), np.float32)
for tz in range(tz_n):
    for tx in range(tx_n):
        off, ln, _ = struct.unpack_from("<QII", f, io + 16 * (tz * tx_n + tx))
        tw, th = min(T, W - tx * T), min(T, H - tz * T)
        raw = lz4.block.decompress(f[off:off + ln], uncompressed_size=tw * th * 2)
        E[tz*T:tz*T+th, tx*T:tx*T+tw] = np.frombuffer(raw, "<i2").reshape(th, tw) * desc["scale"]
print("grid", W, H, "r", r, "sea", sea, "m/block v", mv, "min_y", sc["min_y"])
sea_cells = E < sea - 1
print("sea share of grid cells: %.3f" % sea_cells.mean())
depth_m = (sea - E) * mv
print("sea floor depth percentiles (m) p50 %.0f p90 %.0f p99 %.0f p99.9 %.0f max %.0f" % tuple(
    np.percentile(depth_m[sea_cells], [50, 90, 99, 99.9]).tolist() + [depth_m.max()]))
print("sea floor deeper than: " + " ".join(
    "%.0f km %.4f" % (t / 1000, np.mean(depth_m[sea_cells] > t)) for t in (5000, 6000, 7000, 8000, 10000)))
# separate deep regions: cells deeper than 6 km, 8-connected, deepest spot of each
from collections import deque
for thr in (6000.0,):
    mask = depth_m > thr
    lab = np.zeros(mask.shape, np.int32); n = 0; out = []
    ys, xs = np.nonzero(mask)
    for y0, x0 in zip(ys, xs):
        if lab[y0, x0]: continue
        n += 1; q = deque([(y0, x0)]); lab[y0, x0] = n; best = (depth_m[y0, x0], y0, x0); cnt = 0
        while q:
            y, x = q.popleft(); cnt += 1
            if depth_m[y, x] > best[0]: best = (depth_m[y, x], y, x)
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    yy, xx = y + dy, (x + dx) % W
                    if 0 <= yy < H and mask[yy, xx] and not lab[yy, xx]:
                        lab[yy, xx] = n; q.append((yy, xx))
        out.append((best[0], best[1], best[2], cnt))
    out.sort(reverse=True)
    print("regions deeper than %.0f m: %d (deeper than 10 km: %d)" % (
        thr, len(out), sum(1 for o in out if o[0] > 10000)))
    for dm, y, x, cnt in out[:8]:
        bx = g["origin_block_x"] + x * r + r // 2; bz = g["origin_block_z"] + y * r + r // 2
        area_km2 = cnt * (r * sc["m_per_block_h"] / 1000) ** 2
        print("  deepest %.0f m (floor y %.1f) at block x %d z %d; region %d cells, %.0f km2" % (
            dm, E[y, x], bx, bz, cnt, area_km2))
# shape of the deepest part of the top regions: cells within 2 km of the region's deepest,
# connected to it, their extent along the principal axes (blocks to km by m_per_block_h)
kmc = r * sc["m_per_block_h"] / 1000
for dm, y, x, cnt in out[:4]:
    mask2 = depth_m > dm - 2000
    seen = {(y, x)}; q = deque([(y, x)]); pts = []
    while q and len(pts) < 400000:
        yy, xx = q.popleft(); pts.append((yy, xx))
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                a, b = yy + dy, (xx + dx) % W
                if 0 <= a < H and mask2[a, b] and (a, b) not in seen:
                    seen.add((a, b)); q.append((a, b))
    P = np.array(pts, float)
    P[:, 1] = (P[:, 1] - x + W / 2) % W - W / 2 + x
    c = P - P.mean(0)
    ev, evec = np.linalg.eigh(np.cov(c.T))
    proj = c @ evec
    ln = (proj[:, 1].max() - proj[:, 1].min()) * kmc; wd = (proj[:, 0].max() - proj[:, 0].min()) * kmc
    print("  %.0f m spot: within 2 km of it %d cells, %.0f km long, %.0f km wide (full extents)" % (dm, len(pts), ln, wd))
