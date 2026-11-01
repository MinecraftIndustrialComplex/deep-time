#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Whether PLT-661's rings step outward (measurement only, cached pool runs): ratchet.py <arm folder>[@N] ...

PLT-661 turns ocean floor on the taper's ocean side continental while its own crust is hyper-extended (under 10 km
of crystalline crust of its own, the column less its ocean basement; transition.rs `hyper_extended`), and the taper
counts such crust as its ocean side, so the boundary stays where crust of 10 km and more of its own ends. A ring
whose own crystalline crust grows past 10 km (metamorphism of its buried sediment, PLT-661's sediment lithified and
heated, or crust spread onto it) becomes such crust, and the next ring forms beyond it. This reads, on the final
frame (TIMES), the converted crust that is neither island-arc crust nor a collision belt (arcstock.py's "the rest
converted" and hyper-extended classes: continental, an ocean basement over 1 km): the histogram of its own
crystalline crust (km, bins of 2 km), the pools of its own crust in the bins just under and over the line (shares:
cover, metamorphic, felsic igneous, mafic), and how many lattice rows of it lie between the start class (or
island-arc crust and belts) and the ocean floor: its samples' graph distance, in neighbour steps within 1.2
spacings, from the nearest continental sample of those classes. Medians over runs."""
import os
import statistics
import sys
from pathlib import Path

import numpy as np
from scipy.sparse import coo_matrix
from scipy.sparse.csgraph import dijkstra
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
import thinning as th  # noqa: E402
from arcstock import classes  # noqa: E402

R_KM = 6370.0
TIMES = [int(x) for x in os.environ.get("TIMES", "-500,0").split(",")]
BINS = np.arange(0.0, 22.0, 2.0)


def run_stats(run):
    a0, _ = th.read(run, -1000)
    n0 = len(a0)
    spacing = R_KM * np.sqrt(4.0 * np.pi / n0)
    out = {}
    for t in TIMES:
        try:
            a, pp = th.read(run, t)
        except FileNotFoundError:
            continue
        cont, start, island = classes(a, pp)
        basement = np.clip(np.minimum(pp[:, 7], pp[:, 5] + pp[:, 6]), 0.0, None)
        conv = cont & (basement > 1000.0)
        belt = conv & ~island & (a[:, 5] > 1.5)
        rest = conv & ~island & ~belt
        own = pp[:, 0:7].copy()
        # The basement comes off the mafic pools, plutonic first then volcanic (the reads' convention: own crust is
        # the column less the basement, which is oceanic crust's mafic rock).
        take_p = np.minimum(own[:, 6], basement)
        own[:, 6] -= take_p
        own[:, 5] -= np.minimum(own[:, 5], basement - take_p)
        own_cr = own[:, 2:7].sum(1) / 1000.0
        r = {"rest_area": float(rest.sum() / n0)}
        h, _ = np.histogram(np.clip(own_cr[rest], 0.0, BINS[-1] - 1e-6), bins=BINS)
        for lo, c in zip(BINS[:-1], h):
            r[f"own{int(lo):02d}"] = float(c / n0)
        for name, m in (("under", rest & (own_cr >= 8.0) & (own_cr < 10.0)),
                        ("over", rest & (own_cr >= 10.0) & (own_cr < 14.0)),
                        ("thick", rest & (own_cr >= 14.0))):
            if not m.any():
                continue
            tot = own[m].sum()
            r[f"{name}_cover"] = float(own[m, 0:2].sum() / tot)
            r[f"{name}_meta"] = float(own[m, 2].sum() / tot)
            r[f"{name}_felsic"] = float(own[m, 3:5].sum() / tot)
            r[f"{name}_mafic"] = float(own[m, 5:7].sum() / tot)
            r[f"{name}_formed"] = float(np.median(pp[m, 8]))
        # Rows of the rest converted crust from its source classes: graph steps through continental crust.
        p = th.xyz(a[:, 0], a[:, 1])
        pairs = cKDTree(p).query_pairs(th.chord(1.2 * spacing), output_type="ndarray")
        keep = cont[pairs[:, 0]] & cont[pairs[:, 1]]
        pr = pairs[keep]
        n = len(a)
        g = coo_matrix((np.ones(len(pr)), (pr[:, 0], pr[:, 1])), shape=(n, n)).tocsr()
        src = np.flatnonzero(start | island | belt)
        if src.size and rest.any():
            d = dijkstra(g, directed=False, indices=src, min_only=True, limit=40.0)
            dr = d[rest]
            fin = np.isfinite(dr)
            r["rows_p50"] = float(np.median(dr[fin])) if fin.any() else float("nan")
            r["rows_p90"] = float(np.quantile(dr[fin], 0.9)) if fin.any() else float("nan")
            r["rows_unreached"] = float((~fin).mean())
            for k in (1, 2, 3, 5):
                r[f"rows_le{k}"] = float((dr <= k).mean())
        # Where the hyper-extended crust lies (all continental crust under 10 km of its own crystalline crust): its
        # graph steps through continental crust from the nearest ocean floor (1 = on the coast), and the share of
        # it that is a sediment margin (on a basement, its own crust more cover than crystalline rock).
        hyp = cont & (own_cr < 10.0)
        sedm = hyp & (pp[:, 7] > 0.0) & (own_cr * 1000.0 < own[:, 0:2].sum(1))
        r["hyper_area"] = float(hyp.sum() / n0)
        r["hyper_sedmargin"] = float(sedm.sum() / max(hyp.sum(), 1))
        pr_o = pairs[cont[pairs[:, 0]] != cont[pairs[:, 1]]]
        coast = np.unique(np.where(cont[pr_o[:, 0]], pr_o[:, 0], pr_o[:, 1]))
        if hyp.any() and coast.size:
            do = dijkstra(g, directed=False, indices=coast, min_only=True, limit=40.0) + 1.0
            dh = do[hyp]
            for k in (1, 2, 3):
                r[f"hyper_coast_le{k}"] = float((dh <= k).mean())
            r["hyper_inland_sedmargin"] = float(sedm[hyp & (do > 2.0)].sum() / max((hyp & (do > 2.0)).sum(), 1))
            r["cont_coast_rows1"] = float(coast.size / n0)
        out[t] = r
    return out


def main(args):
    for arg in args:
        runs = th.runs_of(arg)
        stats = [run_stats(r) for r in runs]
        print(f"== {arg} ({len(runs)} runs), medians; areas are shares of the planet")
        for t in TIMES:
            rows = [s[t] for s in stats if t in s]
            if not rows:
                continue
            cells = []
            for k in rows[0]:
                xs = [r[k] for r in rows if k in r and np.isfinite(r[k])]
                if xs:
                    cells.append(f"{k} {statistics.median(xs):.4g}")
            print(f"  t{t}: " + "  ".join(cells))


if __name__ == "__main__":
    main(sys.argv[1:])
