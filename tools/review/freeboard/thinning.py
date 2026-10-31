#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Why the continents thin with the margin's sediment continental (PLT-661): thinning.py <arm folder>[@N] ...

The arm sedmargin (PLT-660 + PLT-661) against base33-s4: continental area 0.388 to 0.487, mean thickness 38.1 to
29.9 km, crust 35 to 40 km from 0.602 to 0.114 of the continental crust (sea-level.md, "Freeboard"). This reads each
run's crust-budget maps (plates/crust_budget.rs: maps/t*.bin, 11 f32 a visible sample; maps/pools/t*.bin, the 7 pools,
the basement, formed_myr) at every snapshot and the record's sea (rec.json), and splits the continental crust into

  margin   hyper-extended: under 10 km of its own crystalline crust (the crystalline pools less the basement,
           Sample::own_pools; transition::hyper_extended, PLT-661);
  edge     the rest within EDGE_KM of a margin or oceanic sample (the transition's and the resampling blend's reach);
  interior the rest, EDGE_KM or more from any margin or ocean sample, split into never raised by an orogeny
           (orogeny 0: the cratons and platforms the collapse never moves) and raised.

Per class and snapshot: the share of the planet's visible samples, the mean thickness T, its own crystalline crust,
the sediment and sedimentary rock on it, the height over the sea z - S (p50) and the land share; and the Airy reading
T - 7 (S - z0) ... printed as T_at_sea: the thickness the column would have with its surface at the sea, T - 7 (z - S)
(pools::COLUMN_ISOSTATIC_FACTOR, PLT-281). If the interiors thin because the sea fell (erosion to base level,
the collapse's room column T_n + 7 S, PLT-412), T_at_sea follows 35.1 + 7 S and z - S stays near the same on both arms.

Prints medians over runs, one table per arm. Needs numpy and scipy (the VM's .pyenv-numpy)."""
import json
import os
import statistics
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

MAP, POOLS = 11, 9
R_KM = 6370.0
HYPER_M = 10_000.0  # ledger: PLT-661 (Brune et al. 2014's 10 km)
AIRY = 7.0  # ledger: PLT-281 (pools::COLUMN_ISOSTATIC_FACTOR)
EDGE_KM = float(os.environ.get("EDGE_KM", "300"))
TIMES = [int(x) for x in os.environ.get("TIMES", "-1000,-900,-800,-700,-600,-500,-400,-300,-200,-100,0").split(",")]


def xyz(lat, lon):
    la, lo = np.radians(lat), np.radians(lon)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], 1)


def chord(km):
    return 2.0 * np.sin(km / R_KM / 2.0)


def runs_of(arg):
    folder, _, n = arg.partition("@")
    out = []
    for start in ("supercontinent", "scattered_continents"):
        for seed in range(1, (int(n) if n else 4) + 1):
            r = Path(folder) / f"{start}-{seed}"
            if (r / "maps").is_dir():
                out.append(r)
    return out


def sea_series(run):
    steps = json.loads((run / "rec.json").read_text()).get("steps") or []
    return [(float(s["t"]), float(s["sea_m"])) for s in steps]


def sea_at(series, t):
    return min(series, key=lambda s: abs(s[0] - t))[1]


def name_of(t):
    return f"t{t:04d}.bin" if t >= 0 else f"t-{-t:03d}.bin"


def read(run, t):
    nm = name_of(t)
    a = np.fromfile(run / "maps" / nm, "<f4").reshape(-1, MAP).astype(np.float64)
    pp = np.fromfile(run / "maps" / "pools" / nm, "<f4").reshape(-1, POOLS).astype(np.float64)
    return a, pp


def classify(a, pp):
    cont = a[:, 4] > 0.5
    base = np.where(cont, np.clip(np.minimum(pp[:, 7], pp[:, 5] + pp[:, 6]), 0.0, None), 0.0)
    own_cryst = pp[:, 2:7].sum(1) - base
    margin = cont & (own_cryst < HYPER_M)
    core = cont & ~margin
    p = xyz(a[:, 0], a[:, 1])
    other = ~core
    d = np.full(len(a), np.inf)
    if other.any() and core.any():
        dd, _ = cKDTree(p[other]).query(p[core])
        d[core] = 2.0 * np.arcsin(np.clip(dd / 2.0, 0, 1)) * R_KM
    edge = core & (d < EDGE_KM)
    interior = core & ~edge
    never = interior & (a[:, 5] < 0.5)
    raised = interior & (a[:, 5] >= 0.5)
    return {"cont": cont, "margin": margin, "edge": edge, "interior": interior, "never": never, "raised": raised,
            "core": core}, own_cryst, base


def stats(a, pp, sel, sea, own, n):
    if not sel.any():
        return {"share": 0.0}
    T = a[sel, 3] / 1000.0
    z = a[sel, 2]
    zs = z - sea
    sed = (pp[sel, 0] + pp[sel, 1]) / 1000.0
    return {"share": sel.sum() / n, "T": float(T.mean()), "T_p50": float(np.median(T)),
            "own": float(own[sel].mean() / 1000.0), "sed": float(sed.mean()), "zs_p50": float(np.median(zs)),
            "land": float((zs > 0).mean()), "T_at_sea": float(np.median(T - AIRY * zs / 1000.0)),
            "b35_40": float(((T >= 35) & (T < 40)).mean()), "b30_35": float(((T >= 30) & (T < 35)).mean())}


def one(run):
    ser = sea_series(run)
    out = {}
    for t in TIMES:
        try:
            a, pp = read(run, t)
        except FileNotFoundError:
            continue
        hid = (a[:, 10].astype(np.int64) & 1) > 0
        a, pp = a[~hid], pp[~hid]
        cls, own, _ = classify(a, pp)
        sea = sea_at(ser, t)
        n = len(a)
        out[t] = {"sea": sea, **{k: stats(a, pp, v, sea, own, n) for k, v in cls.items()}}
    return out


def med(xs):
    xs = [x for x in xs if x is not None and np.isfinite(x)]
    return statistics.median(xs) if xs else float("nan")


def main(argv):
    for arg in argv:
        rs = runs_of(arg)
        res = [one(r) for r in rs]
        print(f"== {arg} ({len(rs)} runs), EDGE_KM {EDGE_KM:g}")
        cols = ["share", "T", "own", "sed", "zs_p50", "land", "T_at_sea", "b35_40", "b30_35"]
        for cls in ("cont", "margin", "edge", "interior", "never", "raised"):
            print(f"-- {cls}")
            print(f"{'t':>6} {'sea':>6} " + " ".join(f"{c:>8}" for c in cols))
            for t in TIMES:
                rows = [r[t] for r in res if t in r]
                if not rows:
                    continue
                sea = med([r["sea"] for r in rows])
                vals = [med([r[cls].get(c) for r in rows]) for c in cols]
                print(f"{t:>6} {sea:>6.0f} " + " ".join(f"{v:>8.3f}" for v in vals))


FBANDS = [(0, 2), (2, 10), (10, 20), (20, 25), (25, 30), (30, 35), (35, 1e9)]


def census(run, t):
    """Label-free: every visible sample by its continental rock (felsic family: metamorphic, felsic plutonic and
    volcanic pools), whatever its label. Per band: share of the planet, share continental-labelled, mean thickness,
    z - S p50, land share, the band's felsic rock and sediment as planet-mean km."""
    ser = sea_series(run)
    a, pp = read(run, t)
    sea = sea_at(ser, t)
    n = len(a)
    F = (pp[:, 2] + pp[:, 3] + pp[:, 4]) / 1000.0
    S = (pp[:, 0] + pp[:, 1]) / 1000.0
    T = a[:, 3] / 1000.0
    zs = a[:, 2] - sea
    cont = a[:, 4] > 0.5
    out = {"sea": sea, "F_total": float(F.sum() / n), "S_total": float(S.sum() / n),
           "S_wet": float(S[zs < 0].sum() / n), "cont_share": float(cont.mean())}
    for lo, hi in FBANDS:
        m = (F >= lo) & (F < hi)
        if not m.any():
            out[(lo, hi)] = {"share": 0.0}
            continue
        out[(lo, hi)] = {"share": float(m.mean()), "cont": float(cont[m].mean()), "T": float(T[m].mean()),
                         "zs": float(np.median(zs[m])), "land": float((zs[m] > 0).mean()),
                         "F": float(F[m].sum() / n), "S": float(S[m].sum() / n)}
    return out


def origin(run, t):
    """The continental classes split by the crust's origin: on an ocean basement (basement over 1 km: ocean floor made
    continental, by PLT-661's margins, PLT-662's spread, a matured arc or a collision's buoyant floor) or not (the
    start's continents and what grew on them), and the edges by distance to the nearest margin or ocean sample."""
    ser = sea_series(run)
    a, pp = read(run, t)
    sea = sea_at(ser, t)
    n = len(a)
    cls, own, base = classify(a, pp)
    p = xyz(a[:, 0], a[:, 1])
    core = cls["core"]
    d = np.full(n, np.inf)
    other = ~core
    if other.any() and core.any():
        dd, _ = cKDTree(p[other]).query(p[core])
        d[core] = 2.0 * np.arcsin(np.clip(dd / 2.0, 0, 1)) * R_KM
    on_basement = base > 1000.0
    out = {"sea": sea}
    sel = {
        "margin": cls["margin"],
        "edge, basement": cls["edge"] & on_basement,
        "edge, none": cls["edge"] & ~on_basement,
        "edge <100 km": cls["edge"] & (d < 100),
        "edge 100-200 km": cls["edge"] & (d >= 100) & (d < 200),
        "edge 200-300 km": cls["edge"] & (d >= 200),
        "interior, basement": cls["interior"] & on_basement,
        "interior, none": cls["interior"] & ~on_basement,
    }
    for k, m in sel.items():
        st = stats(a, pp, m, sea, own, n)
        if m.any():
            st["base"] = float(base[m].mean() / 1000.0)
        out[k] = st
    return out


def main_origin(argv):
    ts = [int(x) for x in os.environ.get("CTIMES", "-500,0").split(",")]
    cols = ["share", "T", "own", "sed", "base", "zs_p50", "land"]
    for arg in argv:
        rs = runs_of(arg)
        print(f"== origin {arg} ({len(rs)} runs)")
        for t in ts:
            res = [origin(r, t) for r in rs]
            print(f"-- t {t}: sea {med([r['sea'] for r in res]):.0f} m")
            print(f"{'class':>20} " + " ".join(f"{c:>8}" for c in cols))
            for k in res[0]:
                if k == "sea":
                    continue
                v = [med([r[k].get(c) for r in res]) for c in cols]
                print(f"{k:>20} " + " ".join(f"{x:>8.3f}" for x in v))


def main_census(argv):
    ts = [int(x) for x in os.environ.get("CTIMES", "-1000,-500,0").split(",")]
    for arg in argv:
        rs = runs_of(arg)
        print(f"== census {arg} ({len(rs)} runs)")
        for t in ts:
            res = [census(r, t) for r in rs]
            print(f"-- t {t}: sea {med([r['sea'] for r in res]):.0f} m, continental label "
                  f"{med([r['cont_share'] for r in res]):.3f}, felsic rock {med([r['F_total'] for r in res]):.3f} km "
                  f"planet-mean, sediment {med([r['S_total'] for r in res]):.3f} (under the sea "
                  f"{med([r['S_wet'] for r in res]):.3f})")
            print(f"{'felsic km':>10} {'share':>7} {'cont':>6} {'T km':>6} {'z-S p50':>8} {'land':>6} {'felsic':>7} "
                  f"{'sed':>6}")
            for b in FBANDS:
                rows = [r[b] for r in res]
                v = [med([x.get(c) for x in rows]) for c in ("share", "cont", "T", "zs", "land", "F", "S")]
                print(f"{b[0]:>4g}-{b[1]:<5g} {v[0]:>7.3f} {v[1]:>6.2f} {v[2]:>6.1f} {v[3]:>8.0f} {v[4]:>6.2f} "
                      f"{v[5]:>7.3f} {v[6]:>6.3f}")


if __name__ == "__main__":
    if os.environ.get("MODE") == "census":
        main_census(sys.argv[1:])
    elif os.environ.get("MODE") == "origin":
        main_origin(sys.argv[1:])
    else:
        main(sys.argv[1:])
