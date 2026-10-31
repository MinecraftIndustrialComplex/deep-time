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
  536 to 1,000 m (1 to 1.62 km in place: accretionary by C&V's in-place line, erosive by the solid one), over 1 km.
MODE=floor: the sea floor's sediment by class (deep ocean, margins, young ocean, trenches near and far from continents)
  in place under ERO-215's law, against GlobSed's means.
MODE=trenchrun: per run, the trench sediment averaged over every snapshot, with Mann-Whitney p against the first arm.
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


def trench_runs(argv):
    """MODE=trenchrun: per run, the trench sediment averaged over every snapshot from -950 to -50 Myr (t0000 is the
    final resampling's lattice, no TRENCH flags): the trench samples' share of the visible samples, their pile's mean
    (m of rock), the sediment on them and on all the visible ocean floor (planet-mean m), and the share of the trench
    samples over 1 km; the medians of the runs, and the two-sided Mann-Whitney p of each arm against the first. A
    heavy-tailed pile read at three snapshots swings from run to run; this reads all of them."""
    from scipy.stats import mannwhitneyu

    ts = list(range(-950, 0, 50))
    keys = ("tr_share", "pile_mean", "sed_tr_m", "sed_ov_m", "thick_n")
    arms = []
    for arg in argv:
        per = []
        for r in runs_of(arg):
            acc = {k: [] for k in keys}
            for t in ts:
                try:
                    a, pp = read(r, t)
                except (FileNotFoundError, ValueError):
                    continue
                n = len(a)
                oc = a[:, 4] < 0.5
                tr = oc & ((a[:, 10].astype(np.int64) & TRENCH_FLAG) > 0)
                pile = pp[:, 0] + pp[:, 1]
                if not tr.any():
                    continue
                acc["tr_share"].append(tr.mean())
                acc["pile_mean"].append(pile[tr].mean())
                acc["sed_tr_m"].append(pile[tr].sum() / n)
                acc["sed_ov_m"].append(pile[oc].sum() / n)
                acc["thick_n"].append((pile[tr] >= 1000.0).mean())
            per.append((r.name, {k: float(np.mean(v)) if v else float("nan") for k, v in acc.items()}))
        arms.append((arg, per))
    for arg, per in arms:
        print(f"== trenchrun {arg} ({len(per)} runs; means over the snapshots -950 to -50)")
        print(f"{'run':26s} " + " ".join(f"{k:>10}" for k in keys))
        for name, v in per:
            print(f"{name:26s} " + " ".join(f"{v[k]:10.4f}" for k in keys))
    print("medians (Mann-Whitney p against the first arm)")
    print(f"{'arm':60s} " + " ".join(f"{k:>16}" for k in keys))
    base = arms[0][1]
    for j, (arg, per) in enumerate(arms):
        cells = []
        for k in keys:
            x = [v[k] for _, v in per]
            if j == 0:
                cells.append(f"{med(x):10.4f}      ")
            else:
                p = mannwhitneyu(x, [v[k] for _, v in base], alternative="two-sided").pvalue
                cells.append(f"{med(x):10.4f} ({p:.2f})")
        print(f"{arg[-60:]:60s} " + " ".join(f"{c:>16}" for c in cells))


def in_place_of_solid(sol):
    """The porous column holding `sol` m of rock under ERO-215's law (Newton on z - phi0 lam (1 - e^(-z/lam)))."""
    z = sol + PHI0 * LAM
    for _ in range(30):
        f = z - PHI0 * LAM * (1.0 - np.exp(-z / LAM)) - sol
        z = np.maximum(z - f / (1.0 - PHI0 * np.exp(-z / LAM)), 0.0)
    return z


def floor_runs(argv):
    """MODE=floor: the sea floor's sediment against GlobSed (Straume et al. 2019, Table 3 and section 3: the grid's
    mean 927 m, the continental margins 3,044 m, the deep ocean over 200 km from the continent-ocean boundary 404 m,
    the ocean younger than 82 Myr within 72 S to 72 N 267 m; seismic thicknesses, in place). Every model pile (rock,
    m) is turned into the porous column holding it under ERO-215's law, then averaged, so both stand in place. The
    classes: the sea (visible samples under the record's sea); the deep ocean (oceanic samples 200 km or more from
    any continental sample); the margins (the rest of the sea); the ocean younger than 82 Myr within 72 S to 72 N.
    And the trench samples' pile by the distance to continental crust (under 200 km: at a continent's or a matured
    arc's margin; else intra-oceanic). Means over the snapshots -950 to -50, medians of the runs."""
    import json

    from scipy.spatial import cKDTree

    ts = list(range(-950, 0, 50))
    rows = []
    for arg in argv:
        for r in runs_of(arg):
            steps = json.loads((r / "rec.json").read_text()).get("steps") or []
            ser = [(float(x["t"]), float(x["sea_m"])) for x in steps]
            acc = {}
            for t in ts:
                try:
                    a, pp = read(r, t)
                    age = np.fromfile(r / "maps" / "age" / name_of(t), "<f4").astype(np.float64)
                except (FileNotFoundError, ValueError):
                    continue
                sea = min(ser, key=lambda x: abs(x[0] - t))[1] if ser else 0.0
                la, lo = np.radians(a[:, 0]), np.radians(a[:, 1])
                xyz = np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], 1)
                cont = a[:, 4] > 0.5
                oc = ~cont
                d = np.full(len(a), np.inf)
                if cont.any():
                    dd, _ = cKDTree(xyz[cont]).query(xyz[oc])
                    d[oc] = 2.0 * np.arcsin(np.clip(dd / 2.0, 0, 1)) * 6371.0
                d[cont] = 0.0
                pile = pp[:, 0] + pp[:, 1]
                ip = in_place_of_solid(np.maximum(pile, 0.0))
                wet = a[:, 2] < sea
                deep = oc & (d >= 200.0)
                young = oc & (age < 82.0) & (np.abs(a[:, 0]) <= 72.0)
                tr = oc & ((a[:, 10].astype(np.int64) & TRENCH_FLAG) > 0)
                cls = {
                    "sea": wet,
                    "deep": deep,
                    "margins": wet & ~deep,
                    "young": young,
                    "tr_near": tr & (d < 200.0),
                    "tr_far": tr & (d >= 200.0),
                }
                for k, m in cls.items():
                    if not m.any():
                        continue
                    acc.setdefault(k + "_ip", []).append(ip[m].mean())
                    acc.setdefault(k + "_rock", []).append(pile[m].mean())
                    acc.setdefault(k + "_share", []).append(m.mean())
                    acc.setdefault(k + "_sed", []).append(pile[m].sum() / max(pile[wet].sum(), 1e-9))
            rows.append({k: float(np.mean(v)) for k, v in acc.items()})
    print(f"== floor ({len(rows)} runs; means over the snapshots -950 to -50, medians of the runs)")
    globsed = {"sea": 927, "deep": 404, "margins": 3044, "young": 267}
    print(f"{'class':10s} {'in place m':>11s} {'GlobSed m':>10s} {'rock m':>8s} {'share of planet':>16s}"
          f" {'share of the sea floor sediment':>32s}")
    for k in ("sea", "deep", "margins", "young", "tr_near", "tr_far"):
        g = globsed.get(k)
        print(f"{k:10s} {med([x.get(k + '_ip', float('nan')) for x in rows]):11.0f} {g if g else '':>10} "
              f"{med([x.get(k + '_rock', float('nan')) for x in rows]):8.0f} "
              f"{med([x.get(k + '_share', float('nan')) for x in rows]):16.4f} "
              f"{med([x.get(k + '_sed', float('nan')) for x in rows]):32.4f}")


def main(argv):
    mode = os.environ.get("MODE", "trench")
    if mode == "trenchrun":
        trench_runs(argv)
        return
    if mode == "floor":
        floor_runs(argv)
        return
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
