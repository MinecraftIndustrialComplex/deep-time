#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Earth's plate boundaries with its small plates merged into their neighbours (no simulation):

    earthmerge.py <PB2002_steps.dat> [<plates> ...]

polenear.py found the model's boundary on pairs that both open and close (at least 400 km of each) at 0.59 to 0.65
against Earth's 0.09 to 0.135, with 14 to 17 plates on a boundary against Bird 2003's 52. This asks how much of the
gap is the plate count: Earth's steps keep their own motion and class (Bird's relative velocity at each step, classed
on its own azimuth and on the 446 km chord polenear.py uses), and only the labels coarsen. The plate with the shortest
outline merges into the neighbour it shares the most boundary with, one at a time, until <plates> remain (default 52,
40, 30, 20, 16, 14); a boundary between two merged plates drops out. At each count it prints, length-weighted, the
boundary on mixed pairs, the convergent boundary within 1,000 km of its own pair's divergent boundary, the boundary a
pair, the boundary on pairs over 4,000 km long and the mixed share of it, and the plates with half their outline on
one neighbour (and how many of those pairs are mixed), the measures polenear.py prints for the model.
"""
import math
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
from polenear import LEN_BANDS, MIXED_KM, NEAR_KM, R_KM, classes, earth_steps  # noqa: E402

LONG_KM = 4000.0
CHORD_KM = 446.0  # polenear.py's Earth chord: the model's 2.5 spacings at 16k


def merge_order(steps):
    """The merges, shortest outline first: [(plate, host)], the host the neighbour it shares most boundary with."""
    shared = {}
    for (a, b), ss in steps.items():
        ln = sum(s[1] for s in ss)
        shared.setdefault(a, {})
        shared.setdefault(b, {})
        shared[a][b] = shared[a].get(b, 0.0) + ln
        shared[b][a] = shared[b].get(a, 0.0) + ln
    order = []
    while len(shared) > 2:
        p = min(shared, key=lambda x: (sum(shared[x].values()), x))
        host = max(shared[p], key=lambda q: (shared[p][q], q))
        order.append((p, host))
        for q, ln in shared.pop(p).items():
            del shared[q][p]
            if q != host:
                shared[host][q] = shared[host].get(q, 0.0) + ln
                shared[q][host] = shared[q].get(host, 0.0) + ln
    return order


def measure(steps, label_of):
    pairs = {}
    for (a, b), ss in steps.items():
        la, lb = label_of(a), label_of(b)
        if la == lb:
            continue
        pairs.setdefault(tuple(sorted((la, lb))), []).extend(ss)
    tot = conv_tot = mixed_len = near = long_len = long_mixed = 0.0
    outline, by_pair, mixed_pairs = {}, {}, set()
    bands = [[0.0, 0.0, 0] for _ in LEN_BANDS]
    for key, ss in pairs.items():
        pts = np.array([s[0] for s in ss])
        ln = np.array([s[1] for s in ss])
        div, conv = classes(np.array([s[3] for s in ss]), np.array([s[2] for s in ss]))
        length, dl, cl = float(ln.sum()), float(ln[div].sum()), float(ln[conv].sum())
        mixed = dl >= MIXED_KM and cl >= MIXED_KM
        tot += length
        conv_tot += cl
        if mixed:
            mixed_len += length
            mixed_pairs.add(key)
        if div.any() and conv.any():
            d, _ = cKDTree(pts[div]).query(pts[conv])
            close = 2.0 * np.arcsin(np.clip(d / 2.0, 0.0, 1.0)) * R_KM < NEAR_KM
            near += float(ln[np.nonzero(conv)[0][close]].sum())
        k = max(i for i, e in enumerate(LEN_BANDS) if length >= e)
        bands[k][0] += length
        bands[k][1] += length if mixed else 0.0
        bands[k][2] += 1
        if length > LONG_KM:
            long_len += length
            long_mixed += length if mixed else 0.0
        for p in key:
            outline[p] = outline.get(p, 0.0) + length
        by_pair[key] = length
    half = [p for p in outline if max(by_pair[k] for k in by_pair if p in k) >= 0.5 * outline[p]]
    half_mixed = sum(1 for p in half if max((k for k in by_pair if p in k), key=by_pair.get) in mixed_pairs)
    return {
        "plates": len(outline), "pairs": len(pairs), "per_pair": tot / max(len(pairs), 1),
        "mixed": mixed_len / tot, "near": near / max(conv_tot, 1e-9), "long": long_len / tot,
        "long_mixed": long_mixed / max(long_len, 1e-9), "half": len(half), "half_mixed": half_mixed,
        "bands": [(b / tot, m / max(b, 1e-9), c) for b, m, c in bands],
    }


def main(argv):
    if not argv:
        print(__doc__)
        return 2
    counts = [int(x) for x in argv[1:]] or [52, 40, 30, 20, 16, 14]
    for half_km, name in ((0.0, "steps"), (CHORD_KM, f"{CHORD_KM:g} km chord")):
        steps, _ = earth_steps(argv[0], half_km)
        order = merge_order(steps)
        n_all = len({p for k in steps for p in k})
        print(f"# Earth, Bird 2003 PB2002, classed on the {name}: {n_all} plates")
        print("  plates  pairs  boundary a pair km  mixed  conv near own div  over 4000 km (mixed)  half outline on one (mixed)")
        for target in counts:
            host = {}
            for p, h in order[:max(0, n_all - target)]:
                host[p] = h

            def label_of(p):
                while p in host:
                    p = host[p]
                return p

            r = measure(steps, label_of)
            print(f"  {r['plates']:6d} {r['pairs']:6d} {r['per_pair']:19,.0f} {r['mixed']:6.3f} {r['near']:18.3f}"
                  f" {r['long']:10.3f} ({r['long_mixed']:.3f}) {r['half']:12d} ({r['half_mixed']})")
            names = [f"{a:g}-{b:g}" for a, b in zip(LEN_BANDS, LEN_BANDS[1:])] + [f">{LEN_BANDS[-1]:g}"]
            print("         by the pair's length, km (share of the boundary, of it mixed, pairs): " + "  ".join(
                f"{nm} {b:.3f} ({m:.2f}, {c})" for nm, (b, m, c) in zip(names, r["bands"])))
        print("  merged first: " + ", ".join(f"{p}>{h}" for p, h in order[:n_all - 14]))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
