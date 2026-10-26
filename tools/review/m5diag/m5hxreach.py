#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""How far the hyper-extended continental crust lies from its continent: m5hxreach.py <arm folder>[@N] [map name ...]

The M5 lane's check on the zone's reach (2026-10-25). PLT-661 turns ocean floor on the taper's ocean side continental
(the taper's ocean side reaches W/2 = 100 km, transition_width_km 200, from the boundary half a spacing beyond the
continent's own crust, PLT-262), and such a column stays hyper-extended (own crystalline crust under 10 km,
PLT-661) and counts as the ocean side. If the converted margins stay where the rule made them, they lie within W/2 of
the continental side; if a resampling moves them out (a new sample takes the class of its major corner, so the zone's
edge moves with the lattice and the conversion is one-way), they lie beyond it. From each run's crust-budget maps
(maps/t*.bin MAP_FIELDS 11 f32, maps/plate/t*.bin u32, maps/pools/t*.bin POOL_FIELDS 9 f32): the distance from each
hyper-extended continental sample to the nearest continental sample of its plate with 10 km or more of its own
crystalline crust, less half a spacing, km. Matured island arcs (1 km or more of island-arc magma) are counted apart.
Medians over runs. Needs numpy and scipy."""
import statistics
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

MAP = 11
POOLS = 9
R_KM = 6370.0
HALF_W_KM = 100.0  # ledger: PLT-262 (transition_width_km 200 / 2)
HYPER_EXTENDED_M = 10_000.0  # ledger: PLT-661
ARC_MAGMA_M = 1000.0
BANDS = [0.0, 100.0, 200.0, 500.0, 1000.0, np.inf]


def xyz(lat, lon):
    la, lo = np.radians(lat), np.radians(lon)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], 1)


def chord_km(c):
    return 2 * R_KM * np.arcsin(np.minimum(c / 2, 1.0))


def one(run, name):
    a = np.fromfile(run / "maps" / name, "<f4").reshape(-1, MAP).astype(np.float64)
    pl = np.fromfile(run / "maps" / "plate" / name, "<u4")
    pp = np.fromfile(run / "maps" / "pools" / name, "<f4").reshape(-1, POOLS).astype(np.float64)
    n = len(a)
    assert len(pl) == n and len(pp) == n, (run, n, len(pl), len(pp))
    p = xyz(a[:, 0], a[:, 1])
    nn, _ = cKDTree(p).query(p, k=2)
    spacing = float(np.median(chord_km(nn[:, 1])))
    cont = a[:, 4] > 0.5
    own = pp[:, 2:7].sum(1) - pp[:, 7]
    hyper = cont & (own < HYPER_EXTENDED_M)
    arc = hyper & (a[:, 7] >= ARC_MAGMA_M)
    side = cont & ~hyper
    d = np.full(n, np.inf)
    for q in np.unique(pl[hyper]):
        c = side & (pl == q)
        h = hyper & (pl == q)
        if c.any():
            dd, _ = cKDTree(p[c]).query(p[h])
            d[h] = chord_km(dd) - 0.5 * spacing
    out = {"spacing_km": spacing, "hyper": hyper.mean(), "arc": arc.mean()}
    for lo, hi in zip(BANDS[:-1], BANDS[1:]):
        out[f"{lo:.0f}-{hi:.0f}"] = (hyper & ~arc & (d >= lo) & (d < hi)).mean()
    out["no side"] = (hyper & ~arc & ~np.isfinite(d)).mean()
    beyond = hyper & ~arc & (d >= HALF_W_KM)
    out["beyond W/2"] = beyond.mean()
    out["beyond W/2 under the sea depth p50 m"] = float(np.median(-a[beyond, 2])) if beyond.any() else float("nan")
    return out


def main():
    folder, _, top = sys.argv[1].partition("@")
    names = sys.argv[2:] or ["t0000.bin"]
    for name in names:
        rows = []
        for run in sorted(q for q in Path(folder).iterdir() if (q / "maps" / "pools" / name).exists()):
            if top and int(run.name.rsplit("-", 1)[1]) > int(top):
                continue
            rows.append(one(run, name))
        if not rows:
            print(f"{name}: no runs with maps/pools/{name}")
            continue
        m = {k: statistics.median(r[k] for r in rows) for k in rows[0]}
        print(f"{len(rows)} runs, map {name}: hyper-extended continental crust, share of the planet (medians); "
              f"spacing {m['spacing_km']:.0f} km")
        print(f"  all {m['hyper']:.4f}, of it matured island arcs {m['arc']:.4f}; the rest by distance to its "
              f"continent's own crust (km past the boundary; the taper's ocean side reaches {HALF_W_KM:.0f}):")
        print("   " + ", ".join(f"{k} {m[k]:.4f}" for k in m if "-" in k and "depth" not in k)
              + f", no side on the plate {m['no side']:.4f}")
        print(f"  beyond W/2 {m['beyond W/2']:.4f}, its depth p50 {m['beyond W/2 under the sea depth p50 m']:.0f} m "
              "below the datum")


if __name__ == "__main__":
    main()
