#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""How many slope-failure iterations relax sea-floor steps on L2 (the sea-floor spikes study, 2026-10-20)."""
import numpy as np

dx = 2400.0
S = np.tan(np.radians(15))
rate = 0.05
offs = [(1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0), (-1, 1), (0, 1), (1, 1)]


def run(h, iters):
    h = h.copy()
    for _ in range(iters):
        dz = np.zeros_like(h)
        for (oi, oj) in offs:
            d = dx * np.hypot(oi, oj)
            hn = np.roll(np.roll(h, -oj, 0), -oi, 1)
            out = np.maximum(h - hn - S * d, 0)
            inn = np.maximum(hn - h - S * d, 0)
            dz += rate * (inn - out)
        h += dz
    return h


def maxslope(h):
    gx = (np.roll(h, -1, 1) - np.roll(h, 1, 1)) / (2 * dx)
    gy = (np.roll(h, -1, 0) - np.roll(h, 1, 0)) / (2 * dx)
    return np.degrees(np.arctan(np.hypot(gx, gy)))[2:-2, 2:-2].max()


n = 64
step = np.full((n, n), -4000.0)
step[:, n // 2:] = -1000.0
trench = np.full((n, n), -5000.0)
trench[:, n // 2] = -10000.0
maze = np.full((n, n), -4000.0)
rng = np.random.default_rng(1)
blk = rng.random((n // 2, n // 2)) < 0.5
maze[np.kron(blk, np.ones((2, 2), bool))] = -1200.0
for name, h in [("step3km", step), ("trench", trench), ("maze", maze)]:
    res = [(it, round(float(maxslope(run(h, it))), 1)) for it in (40, 100, 200, 400, 800)]
    print(name, "start", round(float(maxslope(h)), 1), res, flush=True)
