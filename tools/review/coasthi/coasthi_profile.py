#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Profiles across the open collision fronts at the end of a run (coast-high lane, diagnosis only):
coasthi_profile.py <run folder> [...]. For each open collision (dynamics record, partings with
reason "open") the front's overriding-side map samples (with an incoming-plate sample within 1.6
lattice spacings) that stand within 150 km of collision-regime ground; from each, a transect along
the direction from the nearest incoming sample to it, -400 to +600 km in 50 km steps (negative:
the incoming side), reading the final fields' height over the sea, crust type, thickness and
regime at the nearest census point. Printed: per collision and over all, the median height, the
land share and the continental share per step, and the incoming plate's continental area."""
import json
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import continents_census as cc  # noqa: E402

STEPS = np.arange(-400, 601, 50)


def main():
    allz = []
    for run in map(Path, sys.argv[1:]):
        f, _g, e = cc.load_dump(run / "fields")
        h, w, r = e["h"], e["w"], e["r"]
        frow, fcol, fp = cc.fib_sample(h, w, r)
        z = ((f["elevation"] - cc.SEA_Y) * cc.M_PER_BLOCK_V)[frow, fcol].astype(float)
        reg = np.rint(np.nan_to_num(f["regime"][frow, fcol], nan=0.0)).astype(int)
        cont = f["plates/crust_type"][frow, fcol] > 0.5
        thick = f["plates/thickness_m"][frow, fcol] / 1000.0
        ftree = cKDTree(fp)
        coll_tree = cKDTree(fp[reg == 5]) if (reg == 5).any() else None
        m = np.fromfile(run / "maps" / "t0000.bin", "<f4").reshape(-1, cc.MAP_FIELDS).astype(np.float64)
        pid = np.fromfile(run / "maps" / "plate" / "t0000.bin", "<u4").astype(np.int64)
        mp = cc.xyz(m[:, 0], m[:, 1])
        mtree = cKDTree(mp)
        sp = float(np.sqrt(cc.EARTH_KM2 / mp.shape[0]))
        rec = json.loads((run / "rec.json").read_text())
        t_end = rec["steps"][-1]["t"]
        kind_m = m[:, 4] > 0.5
        for o in rec["partings"]:
            if o["reason"] != "open" or o["end_myr"] < t_end - 1e-6:
                continue
            a, b = o["over"], o["under"]
            ia, ib = np.nonzero(pid == a)[0], np.nonzero(pid == b)[0]
            if ia.size == 0 or ib.size == 0 or coll_tree is None:
                continue
            btree = cKDTree(mp[ib])
            d, nb = btree.query(mp[ia])
            fr = ia[cc.arc_km(d) <= 1.6 * sp]
            if fr.size == 0:
                continue
            dc = cc.arc_km(coll_tree.query(mp[fr])[0])
            fr = fr[dc <= 150.0]
            if fr.size == 0:
                continue
            _d, nbi = btree.query(mp[fr])
            pa, pb = mp[fr], mp[ib[nbi]]
            t = pa - pb * (pb * pa).sum(1)[:, None]
            t /= np.maximum(np.linalg.norm(t, axis=1), 1e-12)[:, None]
            prof = []
            for km in STEPS:
                ang = km / cc.EARTH_R_KM
                pt = pa * np.cos(ang) + t * np.sin(ang)
                k = ftree.query(pt)[1]
                prof.append((z[k], cont[k], thick[k], reg[k] == 5))
            zs = np.array([p[0] for p in prof])
            allz.append((zs, np.array([p[1] for p in prof])))
            inc_cont = kind_m[ib].sum() * cc.EARTH_KM2 / mp.shape[0] / 1e6
            print(f"{run.name} over {a} under {b}: {fr.size} front samples, front {o['front_km']:.0f} km, started"
                  f" {o['start_myr']:.0f} Myr, incoming continental {inc_cont:.1f} Mkm2")
            print("   km   " + " ".join(f"{s:6d}" for s in STEPS))
            print("   z50  " + " ".join(f"{np.median(zs[i]):6.0f}" for i in range(len(STEPS))))
            print("   land " + " ".join(f"{(zs[i] > 0).mean():6.2f}" for i in range(len(STEPS))))
            print("   cont " + " ".join(f"{prof[i][1].mean():6.2f}" for i in range(len(STEPS))))
            print("   T50  " + " ".join(f"{np.median(prof[i][2]):6.1f}" for i in range(len(STEPS))))
            print("   coll " + " ".join(f"{prof[i][3].mean():6.2f}" for i in range(len(STEPS))))
    if allz:
        Z = np.concatenate([a for a, _ in allz], axis=1)
        C = np.concatenate([c for _, c in allz], axis=1)
        print(f"ALL ({Z.shape[1]} front samples)")
        print("   km   " + " ".join(f"{s:6d}" for s in STEPS))
        print("   z50  " + " ".join(f"{np.median(Z[i]):6.0f}" for i in range(len(STEPS))))
        print("   land " + " ".join(f"{(Z[i] > 0).mean():6.2f}" for i in range(len(STEPS))))
        print("   hi   " + " ".join(f"{(Z[i] > 2000).mean():6.2f}" for i in range(len(STEPS))))
        print("   cont " + " ".join(f"{C[i].mean():6.2f}" for i in range(len(STEPS))))


if __name__ == "__main__":
    main()
