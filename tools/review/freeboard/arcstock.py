#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Where the model's island-arc crust stands, against Earth's intra-oceanic arcs (measurement only, cached pool runs):
arcstock.py <arm folder>[@N] ...

The low-continents reads (sea-level.md, "Why base35's continents stand low"; the arm cb-on in
docs/design/continental-buoyancy.md) found most of the drowned crust is crust converted on ocean floor, island-arc crust
by cb_origin's rule the largest part (0.149 of the planet at 31.6 km on cb-on's final frames). Earth's intra-oceanic
arcs are about 14,500 to 17,000 km long (Stern & Scholl 2010; Gerya 2011, quoting Leat & Larter 2003) and juvenile
arcs "approximately 200-300 km across and up to 30 km thick" (Stern & Scholl 2010): about 0.006 to 0.010 of the
planet standing (length x width, our arithmetic). Arc crust a continent took in is continent on Earth. So this splits
the model's island-arc crust (crust_budget.rs `origin`: continental label, an ocean basement over 1 km, at least
1,000 m of island-arc magma and more than of continental-arc magma, as lowcont.py's `area`) by the continental
component it stands in: one that holds start-class crust (the start's continents, no ocean basement: "taken in") or
one that does not ("standing alone"). Components link continental samples within LINK (1.6 times the start's mean
sample spacing). Per snapshot (TIMES): the shares of the planet (the start's visible samples) of island-arc crust
taken in and standing alone, at any thickness and at 20 km and more, the alone part's mean thickness and dry share,
the number of alone components; and, at the end, the island-arc magma standing (the maps' arc_oceanic_m over the
visible samples, each the planet's area over the start's count) against what the run made (cb_globals arc_o,
km3 over the run), the share kept. Medians over runs, per arm.
Needs numpy and scipy (the VM's .pyenv-numpy, run with -I)."""
import os
import statistics
import sys
from pathlib import Path

import numpy as np
from scipy.sparse import coo_matrix
from scipy.sparse.csgraph import connected_components
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
import thinning as th  # noqa: E402

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "islfront"))
import cbread  # noqa: E402

R_KM = 6370.0
T_MIN = 20.0
TIMES = [int(x) for x in os.environ.get("TIMES", "-1000,-900,-700,-500,-300,-100,0").split(",")]
LINK = float(os.environ.get("LINK", "1.6"))


def classes(a, pp):
    cont = a[:, 4] > 0.5
    base = np.where(cont, np.clip(np.minimum(pp[:, 7], pp[:, 5] + pp[:, 6]), 0.0, None), 0.0) / 1000.0
    conv = cont & (base > 1.0)
    island = conv & (a[:, 7] >= 1000.0) & (a[:, 7] > a[:, 8])
    start = cont & ~conv
    return cont, start, island


def components(p, cont, link_km):
    idx = np.flatnonzero(cont)
    tree = cKDTree(p[idx])
    pairs = tree.query_pairs(th.chord(link_km), output_type="ndarray")
    n = len(idx)
    g = coo_matrix((np.ones(len(pairs)), (pairs[:, 0], pairs[:, 1])), shape=(n, n))
    _, lab = connected_components(g, directed=False)
    full = np.full(len(cont), -1)
    full[idx] = lab
    return full


def run_stats(run):
    a0, _ = th.read(run, -1000)
    n0 = len(a0)
    spacing = R_KM * np.sqrt(4.0 * np.pi / n0)
    ser = th.sea_series(run)
    out = {}
    for t in TIMES:
        try:
            a, pp = th.read(run, t)
        except FileNotFoundError:
            continue
        sea = th.sea_at(ser, t)
        cont, start, island = classes(a, pp)
        p = th.xyz(a[:, 0], a[:, 1])
        lab = components(p, cont, LINK * spacing)
        with_start = np.zeros(lab.max() + 1, bool)
        with_start[np.unique(lab[start])] = True
        taken = island & with_start[np.maximum(lab, 0)]
        alone = island & ~taken
        T = a[:, 3] / 1000.0
        c = T >= T_MIN
        r = {
            "arc": island.sum() / n0,
            "taken": taken.sum() / n0,
            "alone": alone.sum() / n0,
            "arc20": (island & c).sum() / n0,
            "taken20": (taken & c).sum() / n0,
            "alone20": (alone & c).sum() / n0,
            "alone_T": float(T[alone].mean()) if alone.any() else float("nan"),
            "alone_dry": float((a[alone, 2] > sea).mean()) if alone.any() else float("nan"),
            "alone_parts": int(len(np.unique(lab[alone]))),
            "start": start.sum() / n0,
            # The start's own samples by their age (a continental sample keeps its nearest corner's formed_myr
            # through a resampling, and the start's carry 0): how much of the start's crust still stands, and how
            # much of it the basement rule now reads as converted.
            "formed0": (cont & (pp[:, 8] == 0.0)).sum() / n0,
            "formed0_conv": (cont & ~start & (pp[:, 8] == 0.0)).sum() / n0,
            "formed0_island": (island & (pp[:, 8] == 0.0)).sum() / n0,
            # Their volume (km of crust over the planet's mean: thickness summed over the start's visible count) and
            # mean thickness: an area lost to shortening keeps its volume, one destroyed does not.
            "formed0_vol": float(T[cont & (pp[:, 8] == 0.0)].sum() / n0),
            "formed0_T": float(T[cont & (pp[:, 8] == 0.0)].mean()) if (cont & (pp[:, 8] == 0.0)).any() else float("nan"),
            "cont_vol": float(T[cont].sum() / n0),
            "arc_vol": float(T[island].sum() / n0),
        }
        if t == 0:
            # Each class's rock at the end: the shares of its column in cover (sediment and sedimentary rock),
            # felsic and metamorphic rock and mafic rock, its ocean basement (km) and its height over the sea (p50).
            # The start column (PLT-834, Rudnick & Gao's layers as Hacker et al. 2015 print them) is 0.05, 0.61, 0.34.
            # The lithosphere file (crust_budget.rs LITH_FIELDS: stored L, keel, the L the laws read, heat
            # flow), on runs made since the arc-stock lane added it.
            lf = run / "maps" / "lith" / th.name_of(t)
            lith = np.fromfile(lf, "<f4").reshape(-1, 4).astype(np.float64) if lf.exists() else None
            conv = cont & ~start
            belt = conv & ~island & (a[:, 5] > 1.5)
            col = pp[:, 0:7].sum(1)
            classes_end = {"start": start, "taken": taken, "alone": alone, "belt": belt,
                           "otherconv": conv & ~island & ~belt}
            # Hyper-extended continental crust (under 10 km of its own crystalline crust, PLT-661's margins and
            # the like: transition::hyper_extended, Brune et al. 2014's limit) and each class's area.
            own_cryst = pp[:, 2:7].sum(1) - np.clip(np.minimum(pp[:, 7], pp[:, 5] + pp[:, 6]), 0.0, None)
            classes_end["hyper"] = cont & (own_cryst < 10_000.0)
            classes_end["cont"] = cont
            # Where the hyper-extended crust lies: its distance to the nearest continental sample that is not
            # hyper-extended (km, p50 and p90), and the length of that crust's boundary with ocean floor or
            # hyper-extended crust (its samples within LINK spacings of one, times the spacing, km): the ring's
            # rows are the hyper area over that length times the spacing.
            hyper = classes_end["hyper"]
            core = cont & ~hyper
            if hyper.any() and core.any():
                tree_core = cKDTree(p[core])
                dist, _ = tree_core.query(p[hyper])
                dist_km = 2.0 * np.arcsin(np.clip(dist / 2.0, 0.0, 1.0)) * R_KM
                r["hyper_dist_p50_km"] = float(np.median(dist_km))
                r["hyper_dist_p90_km"] = float(np.quantile(dist_km, 0.9))
                outside = cKDTree(p[~core])
                near = outside.query_ball_point(p[core], th.chord(LINK * spacing), return_length=True)
                edge_km = float((near > 0).sum() * spacing)
                r["core_edge_km"] = edge_km
                r["hyper_rows"] = float(hyper.sum() * (4.0 * np.pi * R_KM**2 / n0) / (edge_km * spacing))
            # How broken the continental crust is: its components (the continental samples linked within LINK
            # spacings), the count over 10^5 km2, and the share of its area in the largest seven.
            cell_km2_all = 4.0 * np.pi * R_KM**2 / n0
            sizes = np.bincount(lab[cont]) * cell_km2_all
            sizes = np.sort(sizes[sizes > 0])[::-1]
            r["parts_over_1e5km2"] = int((sizes > 1.0e5).sum())
            r["largest7_share"] = float(sizes[:7].sum() / sizes.sum()) if sizes.size else float("nan")
            # The boundary's length, km: neighbour pairs (within 1.2 spacings, the lattice's six or so) with one
            # sample on each side, times a cell edge (spacing / sqrt 3, a hexagonal cell's): the continental label
            # against ocean floor, and the crust that is not hyper-extended against the rest.
            pairs = cKDTree(p).query_pairs(th.chord(1.2 * spacing), output_type="ndarray")
            edge = spacing / np.sqrt(3.0)
            r["coast_cont_km"] = float((cont[pairs[:, 0]] != cont[pairs[:, 1]]).sum() * edge)
            r["coast_core_km"] = float((core[pairs[:, 0]] != core[pairs[:, 1]]).sum() * edge) if hyper.any() else float("nan")
            for k, m in classes_end.items():
                r[f"{k}_area"] = float(m.sum() / n0)
                if not m.any():
                    continue
                c_sum = col[m].sum()
                r[f"{k}_cover"] = float(pp[m, 0:2].sum() / c_sum)
                r[f"{k}_felsic"] = float(pp[m, 2:5].sum() / c_sum)
                r[f"{k}_mafic"] = float(pp[m, 5:7].sum() / c_sum)
                r[f"{k}_base_km"] = float(np.clip(pp[m, 7], 0.0, None).mean() / 1000.0)
                r[f"{k}_T"] = float(T[m].mean())
                r[f"{k}_zs"] = float(np.median(a[m, 2] - sea))
                if lith is not None:
                    r[f"{k}_L"] = float(np.median(lith[m, 2]))
                    r[f"{k}_keel"] = float(np.median(lith[m, 1]))
                    r[f"{k}_q"] = float(np.median(lith[m, 3]))
            cell_km2 = 4.0 * np.pi * R_KM**2 / n0
            standing = float(a[:, 7].sum() / 1000.0 * cell_km2)
            g = cbread.last_globals(run / "summary.txt") or {}
            made = g.get("arc_o", float("nan"))
            r["arc_o_standing_km3"] = standing
            r["arc_o_made_km3"] = made
            r["arc_o_kept"] = standing / made if made else float("nan")
            r["alone_standing_km3"] = float(a[alone, 7].sum() / 1000.0 * cell_km2)
        # The boundary's length at every snapshot (as at the end, below): the label against ocean floor.
        if "coast_cont_km" not in r:
            pairs = cKDTree(p).query_pairs(th.chord(1.2 * spacing), output_type="ndarray")
            r["coast_cont_km"] = float((cont[pairs[:, 0]] != cont[pairs[:, 1]]).sum() * spacing / np.sqrt(3.0))
        out[t] = r
    return out


def main(args):
    for arg in args:
        runs = th.runs_of(arg)
        stats = [run_stats(r) for r in runs]
        print(f"== {arg} ({len(runs)} runs), medians, shares of the planet; LINK {LINK} x the start's spacing")
        for t in TIMES:
            rows = [s[t] for s in stats if t in s]
            if not rows:
                continue
            keys = [k for k in rows[0]]
            cells = []
            for k in keys:
                xs = [r[k] for r in rows if k in r and np.isfinite(r[k])]
                if xs:
                    v = statistics.median(xs)
                    cells.append(f"{k} {v:.4g}")
            print(f"  t{t}: " + "  ".join(cells))


if __name__ == "__main__":
    main(sys.argv[1:])
