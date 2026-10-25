#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""M5 through time from the crust-budget maps: m5maps.py <out.jsonl> [workers] [runs]

For each base12 run and each map time in TIMES: the visible plate samples (maps/t*.bin, the plate
model's elevation less the step's sea level from rec.json, before the surface stage) put on
100,000 Fibonacci points (each the nearest sample; equal-area and the same resolution at every
time), then the census's M4 and M5 rules (m5lib, cc's helpers), the geometry factors, the split of
the qualified coast by the class of its nearest qualifying high ground (m5fields.classify on the
map's tags; no regime field in the maps, so arc and rift come from the flags and the arc magma),
the coast left without each class, the distance histogram and M5 at Earth's shares of the land
above the level. The 0 Myr row is the bridge to the census grid (m5fields.py's plate_z_M5)."""
import json
import sys
from multiprocessing import Pool
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
import m5lib  # noqa: E402
from m5lib import cc  # noqa: E402
from m5fields import ARM, CLASSES, FIELDS, KEY, POOL, classify  # noqa: E402

TIMES = [-900, -700, -500, -400, -300, -200, -150, -100, -50, 0]
NPTS = 100_000
TARGETS = [0.055, 0.07]


def one(run):
    rec = json.loads((run / "rec.json").read_text())
    ts = np.array([s["t"] for s in rec["steps"]])
    sea = np.array([s.get("sea_m", 0.0) or 0.0 for s in rec["steps"]])
    flat, flon = m5lib.fib(NPTS)
    pf = cc.xyz(flat, flon)
    sp = float(np.sqrt(cc.EARTH_KM2 / NPTS))
    rows = []
    for t in TIMES:
        path = run / "maps" / f"t{t:04.0f}.bin"
        if not path.exists():
            continue
        a = np.fromfile(path, "<f4").reshape(-1, FIELDS).astype(np.float64)
        sea_t = float(sea[int(np.argmin(np.abs(ts - t)))])
        _, nn = cKDTree(cc.xyz(a[:, 0], a[:, 1])).query(pf)
        b = a[nn]
        z = b[:, 2] - sea_t
        flags = b[:, 10].astype(np.int64)
        cls = classify(np.rint(b[:, 5]).astype(int), b[:, 6], b[:, 3] / 1000.0, b[:, 4] > 0.5, b[:, 7], b[:, 8],
                       flags, np.full(NPTS, -1))
        det = m5lib.coast_detail(pf, z, sp, cls=cls, ncls=len(CLASSES))
        D = det["D"]
        C = D.shape[0]
        dmin = D.min(1)
        qual = np.isfinite(dmin)
        near = np.argmin(D, 1)
        land = det["land"]
        L = int(land.sum())
        lab_sizes = np.bincount(det["comp"][land])
        o = {"run": run.name, "start": run.name.rsplit("-", 1)[0], "seed": int(run.name.rsplit("-", 1)[1]),
             "t": t, "samples": int(a.shape[0]), "sea_m": sea_t, "land": L / NPTS,
             "largest_landmass_of_land": float(lab_sizes.max() / max(L, 1)) if L else 0.0,
             "n_landmass_ge1Mkm2": int((lab_sizes * cc.EARTH_KM2 / NPTS >= 1e6).sum()),
             "M5": float(qual.sum() / max(C, 1)), "M4": float((z[land] > 2000).mean()) if L else 0.0,
             "factors": m5lib.factors(det),
             "m5_by_nearest": {k: float((qual & (near == i)).sum() / max(C, 1)) for i, k in enumerate(CLASSES)},
             "m5_without": {k: float(np.isfinite(np.delete(D, i, 1)).any(1).sum() / max(C, 1))
                            for i, k in enumerate(CLASSES)},
             "m4_by": {k: float((det["hi"] & (cls == i)).sum() / max(L, 1)) for i, k in enumerate(CLASSES)},
             "dist_hist": m5lib.dist_hist(dmin),
             "curve": m5lib.m5_curve(pf, z, sp, det["comp"], det["d_sea"], TARGETS)}
        rows.append(o)
    return rows


def main():
    dest = Path(sys.argv[1])
    workers = int(sys.argv[2]) if len(sys.argv) > 2 else 3
    runs = sorted(p for p in (POOL / KEY / ARM).glob("*-*") if p.is_dir() and (p / "maps" / "t0000.bin").exists())
    if len(sys.argv) > 3:
        runs = [p for p in runs if p.name in sys.argv[3].split(",")]
    print(f"{len(runs)} runs", flush=True)
    with Pool(workers) as pool, dest.open("w") as fh:
        for rows in pool.imap_unordered(one, runs):
            for o in rows:
                fh.write(json.dumps(o) + "\n")
            fh.flush()
            if rows:
                print(rows[0]["run"], " ".join(f"{o['t']}:{o['M5']:.3f}/{o['M4']:.3f}" for o in rows), flush=True)
    print("M5MAPS_DONE")


if __name__ == "__main__":
    main()
