#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The crust-budget lane's probes over the crust-budget maps (no simulation): probe.py <arm folder>[@N] ...

Reads each run's `DEEPTIME_CRUST_BUDGET_MAPS` snapshots (plates/crust_budget.rs: maps/t*.bin, 11 f32 a visible sample;
maps/pools/t*.bin, the 7 pools, the basement, formed_myr) at CTIMES and prints medians over the runs.

MODE=trench (default): the incoming plate's sediment at its trenches (oceanic samples with the TRENCH flag, a visible
  sample beside its own group's hidden ones: the floor about to go under). The pile is the sediment and sedimentary
  pools, solid rock as every pool of the model (surface/pelagic.rs: no pore water). Clift & Vannucchi 2004's 1 km
  (PLT-442, PLT-660) is read against it as solid; the pile in place is the porous column holding that solid under
  Kominz et al. 2011's average ocean-floor sediment (phi0 0.66, lambda 1,333 m; ERO-215, surface::pelagic): 1 km in
  place holds 536 m of rock. Shares of the trench samples by pile: under 536 m solid (thinner than 1 km in place),
  536 to 1,000 m (1 to 1.78 km in place: accretionary by C&V's in-place line, erosive by the solid one), over 1 km.
MODE=arcs: where the arcs' magma went, as stocks on the visible samples (arc_oceanic_m, arc_continental_m: the magma
  each sample received from island and from continental arcs, PLT-045/PLT-285): on ocean floor, on continental crust
  an island arc made (dynamics::is_island_arc, PLT-220: island-arc magma more than continental), and on the rest of
  the continental crust; and of the island-arc crust made continental, the magma it then took at the continental rate.
  Planet-mean m (Earth-normalised km3 = m x 5.1e8 / 1000).
"""
import os
import statistics
import sys
from pathlib import Path

import numpy as np

MAP, POOLS = 11, 9
TRENCH_FLAG = 2  # state::TRENCH
PHI0, LAM = 0.66, 1333.0  # ledger: ERO-215 (Kominz et al. 2011's average ocean-floor sediment)
PLANET_KM2 = 5.1007e8


def solid_of_in_place(z):
    return z - PHI0 * LAM * (1.0 - np.exp(-z / LAM))


ONE_KM_SOLID = float(solid_of_in_place(np.array(1000.0)))


def runs_of(arg):
    folder, _, n = arg.partition("@")
    out = []
    for start in ("supercontinent", "scattered_continents"):
        for seed in range(1, (int(n) if n else 4) + 1):
            r = Path(folder) / f"{start}-{seed}"
            if (r / "maps").is_dir():
                out.append(r)
    return out


def name_of(t):
    return f"t{t:04d}.bin" if t >= 0 else f"t-{-t:03d}.bin"


def read(run, t):
    nm = name_of(t)
    a = np.fromfile(run / "maps" / nm, "<f4").reshape(-1, MAP).astype(np.float64)
    pp = np.fromfile(run / "maps" / "pools" / nm, "<f4").reshape(-1, POOLS).astype(np.float64)
    return a, pp


def trench(run, t):
    a, pp = read(run, t)
    tr = (a[:, 4] < 0.5) & ((a[:, 10].astype(np.int64) & TRENCH_FLAG) > 0)
    pile = pp[:, 0] + pp[:, 1]
    p = pile[tr]
    if not len(p):
        return None
    tot = p.sum()
    thin = p < ONE_KM_SOLID
    mid = (p >= ONE_KM_SOLID) & (p < 1000.0)
    thick = p >= 1000.0
    return {
        "n": float(len(p)),
        "pile_mean": float(p.mean()),
        "pile_p50": float(np.median(p)),
        "pile_p90": float(np.quantile(p, 0.9)),
        "thin_n": float(thin.mean()),
        "mid_n": float(mid.mean()),
        "thick_n": float(thick.mean()),
        "thin_vol": float(p[thin].sum() / tot),
        "mid_vol": float(p[mid].sum() / tot),
        "thick_vol": float(p[thick].sum() / tot),
    }


def arcs(run, t):
    a, pp = read(run, t)
    n = len(a)
    cont = a[:, 4] > 0.5
    ao, ac = a[:, 7], a[:, 8]
    island = (~cont) | (ao > ac)
    base = pp[:, 7]
    io_cont = cont & (ao > ac) & (ao > 0.0)
    rest = cont & ~io_cont
    m = lambda x: float(x.sum() / n)
    return {
        "ocean_ao": m(ao[~cont]),
        "iocont_share": float(io_cont.mean()),
        "iocont_ao": m(ao[io_cont]),
        "iocont_ac": m(ac[io_cont]),
        "iocont_base_km": float(base[io_cont].mean() / 1000.0) if io_cont.any() else float("nan"),
        "rest_ao": m(ao[rest]),
        "rest_ac": m(ac[rest]),
        "cont_ac_on_io_share": float(ac[io_cont].sum() / max(ac[cont].sum(), 1e-9)),
    }


def med(xs):
    xs = [x for x in xs if x == x]
    return statistics.median(xs) if xs else float("nan")


def main(argv):
    mode = os.environ.get("MODE", "trench")
    fn = {"trench": trench, "arcs": arcs}[mode]
    ts = [int(x) for x in os.environ.get("CTIMES", "-800,-500,-200,0").split(",")]
    if mode == "trench":
        print(f"# 1 km in place holds {ONE_KM_SOLID:.0f} m of rock (phi0 {PHI0}, lambda {LAM:.0f} m)")
    for arg in argv:
        rs = runs_of(arg)
        print(f"== {mode} {arg} ({len(rs)} runs)")
        cols = None
        for t in ts:
            res = [x for x in (fn(r, t) for r in rs) if x]
            if not res:
                continue
            if cols is None:
                cols = list(res[0])
                print(f"{'t':>6} " + " ".join(f"{c:>13}" for c in cols))
            print(f"{t:>6} " + " ".join(f"{med([x[c] for x in res]):>13.3f}" for c in cols))


if __name__ == "__main__":
    main(sys.argv[1:])
