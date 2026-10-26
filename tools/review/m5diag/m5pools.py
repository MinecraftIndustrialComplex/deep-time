#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""What the oceanic-labelled crust's standing stock is made of, and where it lies:
m5pools.py <arm folder>[@N] [map name, default t0000.bin]

The M5 stage (a)+(b) diagnostic (coordinator, 2026-10-25: "the one diagnostic run with a pools field on the oceanic
samples, to split the standing stock into sediment and spread rock"). From each run's crust-budget maps
(`DEEPTIME_CRUST_BUDGET_MAPS`; plates/crust_budget.rs): maps/t*.bin (MAP_FIELDS, 11 f32), maps/plate/t*.bin (u32)
and maps/pools/t*.bin (POOL_FIELDS, 9 f32: the seven rock pools, the basement, formed_myr), all in the same
visible-sample order.

The stock is an oceanic sample's crust beyond the ocean column's own (2 km of lavas, 5 km of gabbro): every
sediment, sedimentary, metamorphic and felsic metre, and mafic rock beyond the column's. It is split by pool, and
by where it lies:
  - ring: the distance to the nearest continental sample of the same plate, in sample spacings (ring 1 borders it);
  - margin: what stage (a) would turn continental, an oceanic sample whose same-plate continental neighbour is within
    W/2 = 100 km of the boundary half a spacing out (transition.rs's zone, PLT-261/262), not at a trench, in a
    subduction zone or on an arc; "margin200" the same within 200 km (Brune et al. 2014's widest);
  - pelagic: the sediment and sedimentary rock beyond the pelagic rain for the floor's age (5 m/Myr,
    `pelagic_m_per_myr`) is the continents' (terrigenous).
Volumes are planet-mean metres (sum over oceanic samples / all visible samples). Medians over runs.
Needs numpy and scipy."""
import statistics
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

MAP = 11
POOLS = 9
NAMES = ["sed", "sedr", "meta", "fplu", "fvol", "mvol", "mplu"]
OCEAN = np.array([0, 0, 0, 0, 0, 2000.0, 5000.0])
R_KM = 6370.0
W_KM = 200.0
PELAGIC_M_PER_MYR = 5.0
EXCLUDE = 2 | 4 | 64  # trench, arc, subduction zone


def xyz(lat, lon):
    la, lo = np.radians(lat), np.radians(lon)
    return np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], 1)


def chord_km(c):
    return 2 * R_KM * np.arcsin(np.minimum(c / 2, 1.0))


def one(run, name):
    t = float(name[1:-4])
    a = np.fromfile(run / "maps" / name, "<f4").reshape(-1, MAP).astype(np.float64)
    pl = np.fromfile(run / "maps" / "plate" / name, "<u4")
    pp = np.fromfile(run / "maps" / "pools" / name, "<f4").reshape(-1, POOLS).astype(np.float64)
    n = len(a)
    assert len(pl) == n and len(pp) == n, (run, len(a), len(pl), len(pp))
    p = xyz(a[:, 0], a[:, 1])
    cont = a[:, 4] > 0.5
    oc = ~cont
    flags = np.rint(a[:, 10]).astype(np.int64)
    tree = cKDTree(p)
    nn, _ = tree.query(p, k=2)
    spacing = float(np.median(chord_km(nn[:, 1])))
    # The nearest continental sample of the same plate, per oceanic sample.
    d_same = np.full(n, np.inf)
    for q in np.unique(pl[oc]):
        c = cont & (pl == q)
        o = oc & (pl == q)
        if c.any() and o.any():
            d, _ = cKDTree(p[c]).query(p[o])
            d_same[o] = chord_km(d)
    d_any = np.full(n, np.inf)
    d, _ = cKDTree(p[cont]).query(p[oc])
    d_any[oc] = chord_km(d)
    ring = np.where(np.isfinite(d_same), np.rint(d_same / spacing), 99)
    # The boundary lies half a spacing beyond the boundary sample (transition.rs).
    to_cob = d_same - 0.5 * spacing
    ok = (flags & EXCLUDE) == 0
    margin = oc & ok & (to_cob < 0.5 * W_KM)
    margin200 = oc & ok & (to_cob < W_KM)
    excess = np.maximum(pp[:, :7] - OCEAN, 0.0)
    excess[cont] = 0.0
    age = np.maximum(t - pp[:, 8], 0.0)
    sed = pp[:, 0] + pp[:, 1]
    terr = np.where(oc, np.maximum(sed - PELAGIC_M_PER_MYR * age, 0.0), 0.0)
    pel = np.where(oc, np.minimum(sed, PELAGIC_M_PER_MYR * age), 0.0)
    T = pp[:, :7].sum(1)
    out = {"spacing": spacing, "oc": oc.mean(), "stock": excess.sum() / n,
           "terr": terr.sum() / n, "pel": pel.sum() / n}
    for k, nm in enumerate(NAMES):
        out[nm] = excess[:, k].sum() / n
    sets = {"margin": margin, "margin200": margin200, "thick15": oc & (T >= 15000), "thin15": oc & (T < 15000),
            "ring1": oc & (ring == 1), "ring2": oc & (ring == 2), "ring3": oc & (ring == 3),
            "ring4+": oc & (ring >= 4) & (ring < 99), "otherplate": oc & (ring == 99),
            "excluded": oc & ~ok}
    for key, m in sets.items():
        out[f"{key}:n"] = m.sum() / n
        out[f"{key}:stock"] = excess[m].sum() / n
        out[f"{key}:terr"] = terr[m].sum() / n
        out[f"{key}:z"] = float(a[m, 2].mean()) if m.any() else float("nan")
        out[f"{key}:T"] = float(T[m].mean() / 1000.0) if m.any() else float("nan")
        for k, nm in enumerate(NAMES):
            out[f"{key}:{nm}"] = excess[m, k].sum() / n
    # The stock by distance to any continent, 50 km bands (km on the model sphere).
    for lo in range(0, 400, 50):
        m = oc & (d_any >= lo) & (d_any < lo + 50)
        out[f"dany{lo}"] = excess[m].sum() / n
    m = oc & (d_any >= 400)
    out["dany400"] = excess[m].sum() / n
    return out


def main():
    folder, _, top = sys.argv[1].partition("@")
    name = sys.argv[2] if len(sys.argv) > 2 else "t0000.bin"
    rows = []
    for run in sorted(q for q in Path(folder).iterdir() if (q / "maps" / "pools" / name).exists()):
        if top and int(run.name.rsplit("-", 1)[1]) > int(top):
            continue
        rows.append((run.name, one(run, name)))
    if not rows:
        print("no runs with maps/pools/" + name)
        return
    keys = rows[0][1].keys()
    m = {k: statistics.median(r[k] for _, r in rows) for k in keys}
    print(f"{len(rows)} runs ({', '.join(r for r, _ in rows)}), map {name}; planet-mean m unless marked")
    print(f"  spacing {m['spacing']:.0f} km, oceanic share {m['oc']:.3f}")
    print(f"  stock beyond the 7 km ocean column {m['stock']:.0f} m: " +
          ", ".join(f"{nm} {m[nm]:.0f}" for nm in NAMES))
    print(f"  sediment + sedimentary: pelagic (5 m/Myr x age) {m['pel']:.0f} m, beyond it (terrigenous) {m['terr']:.0f} m")
    print("  set: share of the planet, mean z m, mean T km, stock m (its share of the stock), terrigenous m, by pool")
    for key in ["margin", "margin200", "ring1", "ring2", "ring3", "ring4+", "otherplate", "excluded", "thick15",
                "thin15"]:
        s = m[f"{key}:stock"]
        print(f"    {key:>10s}: {m[key + ':n']:.3f}, z {m[key + ':z']:6.0f}, T {m[key + ':T']:5.1f}, stock {s:4.0f}"
              f" ({s / max(m['stock'], 1e-9):.2f}), terr {m[key + ':terr']:4.0f}; " +
              " ".join(f"{nm} {m[key + ':' + nm]:.0f}" for nm in NAMES))
    print("  stock by distance to any continental sample, km: " +
          ", ".join(f"{lo}-{lo + 50} {m[f'dany{lo}']:.0f}" for lo in range(0, 400, 50)) + f", 400+ {m['dany400']:.0f}")


if __name__ == "__main__":
    main()
