#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""oldfloor_plates.py <argkey folder> [--seeds=1,2,3,4] [--from=-900] [--every=4]: which plates hold the old floor.

The coordinator's diagnosis for the old floor's second closure law (2026-10-27). For every --every-th crust-budget
snapshot from --from on (maps/t*.bin with maps/plate/ and maps/age/; a resampling's snapshot is skipped as in
oldfloor.py), the visible oceanic samples older than 250 Myr are joined with their plate's row of the dynamics
record's nearest step (rec.json steps[].plates: id, visible, continental, speed mm/yr, front_km the slab fronts it
goes down at, boundary_km). Prints, pooled over runs and snapshots (each table over its own count), the old floor's share by its plate's speed, by its
plate's slab front (none, under 2,000 km, 2,000 to 5,000, more), by its plate's continental share and by its plate's
size, and the share on plates that both move under 20 mm/yr and have under 2,000 km of slab front. And whether the floor
trails its continent: on the great circle from the sample to its own plate's nearest TRENCH sample (where its plate
goes down), 12 points are read (the nearest visible sample to each); "trails" when one of them is its own plate's
continental crust, "leads" when none is, "no trench" when its plate has none. Numpy and scipy (the pool's numpy env).
"""
import json
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

FIELDS = 11
TRENCH = 2
FRONT_SUB = 256
FRONT_COL = 512
R_KM = 6371.0


def xyz(lat, lon):
    la, lo = np.radians(lat), np.radians(lon)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], axis=-1)


def band(x, edges, names):
    for e, n in zip(edges, names):
        if x < e:
            return n
    return names[-1]


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    seeds, t_from, every = None, -900.0, 4
    for a in sys.argv[1:]:
        if a.startswith("--seeds="):
            seeds = {int(s) for s in a.split("=", 1)[1].split(",")}
        if a.startswith("--from="):
            t_from = float(a.split("=", 1)[1])
        if a.startswith("--every="):
            every = int(a.split("=", 1)[1])
    tables = {k: {} for k in ("speed", "front", "continental", "size", "slow_short", "side", "trails_sub_km", "trails_col_km", "leads_sub_km", "leads_col_km")}
    total = 0
    for arm in args:
        for run in sorted(p for p in Path(arm).iterdir() if (p / "rec.json").is_file()):
            if seeds is not None and int(run.name.rsplit("-", 1)[1]) not in seeds:
                continue
            r = json.loads((run / "rec.json").read_text())
            N, steps = r["samples"], r["steps"]
            step_t = np.array([s["t"] for s in steps])
            # In time order (the names are signed times: t-1000, t-050, t0000), so the stride steps through time.
            paths = [p for p in sorted((run / "maps").glob("t*.bin"), key=lambda p: float(p.stem[1:]))
                     if float(p.stem[1:]) >= t_from][::every]
            for path in paths:
                a = np.fromfile(path, "<f4").reshape(-1, FIELDS)
                n = len(a)
                pp, ap = run / "maps" / "plate" / path.name, run / "maps" / "age" / path.name
                if n >= N or not (pp.is_file() and ap.is_file()):
                    continue
                plate = np.fromfile(pp, "<u4").astype(np.int64)
                age = np.fromfile(ap, "<f4").astype(np.float64)
                if len(plate) != n or len(age) != n:
                    continue
                old = (a[:, 4] <= 0.5) & (age > 250.0)
                step = steps[int(np.argmin(np.abs(step_t - float(path.stem[1:]))))]
                rows = {int(p[0]): p for p in step["plates"]}
                # Which side of its continent each old sample lies on, against its own plate's trench.
                p3 = xyz(a[:, 0].astype(np.float64), a[:, 1].astype(np.float64))
                tree = cKDTree(p3)
                cont = a[:, 4] > 0.5
                fl = np.rint(a[:, 10]).astype(np.int64)
                trench = (fl & TRENCH) != 0
                fronts = {}
                for name, bit in (("sub", FRONT_SUB), ("col", FRONT_COL)):
                    idx = np.flatnonzero((fl & bit) != 0)
                    fronts[name] = cKDTree(p3[idx]) if len(idx) else None
                for pid in np.unique(plate[old]):
                    mine = old & (plate == pid)
                    tr = np.flatnonzero(trench & (plate == pid))
                    if len(tr) == 0:
                        tables["side"]["no trench"] = tables["side"].get("no trench", 0) + int(mine.sum())
                        continue
                    src = p3[mine]
                    _, k = cKDTree(p3[tr]).query(src)
                    dst = p3[tr[k]]
                    trails = np.zeros(len(src), bool)
                    for f in np.linspace(0.08, 0.92, 12):
                        q = src * (1 - f) + dst * f
                        q /= np.linalg.norm(q, axis=1, keepdims=True)
                        _, j = tree.query(q)
                        trails |= cont[j] & (plate[j] == pid)
                    for side, sel in (("trails", trails), ("leads", ~trails)):
                        for name, tr in fronts.items():
                            if tr is None:
                                d = np.full(int(sel.sum()), np.inf)
                            else:
                                d, _ = tr.query(src[sel])
                                d = 2.0 * np.arcsin(np.clip(d / 2.0, 0, 1)) * R_KM
                            for km_ in d:
                                b = band(km_, [500, 1500, 3000, 1e12], ["<500", "500-1500", "1500-3000", "3000+"]) if np.isfinite(km_) else "none"
                                key = f"{side}_{name}_km"
                                tables[key][b] = tables[key].get(b, 0) + 1
                    tables["side"]["trails"] = tables["side"].get("trails", 0) + int(trails.sum())
                    tables["side"]["leads"] = tables["side"].get("leads", 0) + int((~trails).sum())
                ids, counts = np.unique(plate[old], return_counts=True)
                for pid, c in zip(ids, counts):
                    row = rows.get(int(pid))
                    if row is None:
                        continue
                    vis, cont, speed, front = row[1], row[2], row[3], row[4]
                    total += int(c)
                    keys = {
                        "speed": band(speed, [10, 20, 40, 1e9], ["<10", "10-20", "20-40", "40+"]),
                        "front": "none" if front <= 0 else band(front, [2000, 5000, 1e12], ["<2000", "2000-5000", "5000+"]),
                        "continental": band(cont / max(vis, 1), [0.05, 0.25, 0.5, 2], ["<0.05", "0.05-0.25", "0.25-0.5", "0.5+"]),
                        "size": band(vis / N, [0.02, 0.05, 0.1, 0.2, 2], ["<0.02", "0.02-0.05", "0.05-0.1", "0.1-0.2", "0.2+"]),
                        "slow_short": "slow and short" if speed < 20 and front < 2000 else "other",
                    }
                    for t, k in keys.items():
                        tables[t][k] = tables[t].get(k, 0) + int(c)
            print(f"{run.name}: {len(paths)} snapshots read", file=sys.stderr, flush=True)
    # Each table over its own count: the plate tables leave out plates with no row in the record's step, the side
    # tables do not, and the distance tables are within their side (trails_* over the trailing floor).
    print(json.dumps({"old250_samples": total,
                      "shares": {t: {k: round(v / max(sum(d.values()), 1), 3) for k, v in sorted(d.items())}
                                 for t, d in tables.items()}},
                     indent=1))


main()
