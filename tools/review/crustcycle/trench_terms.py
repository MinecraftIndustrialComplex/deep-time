#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The trenches' sediment input by its delivery terms (no simulation): trench_terms.py <length Myr> <summary.txt> ...

Reads each run's last `cb_globals` line (the crust budget, cumulative over the run) and splits the sediment that
reaches the trenches (`sed_trench`, rock: the model's pools hold no pore water) into the fronts under continental
crust (`sed_trench_c`), under oceanic crust (`sed_trench_o`) and the one-way slab's (the rest), each as km^3 a year,
per km of front (km^3/km/Myr, over the arcs' fronts `front_c` and `front_o`, km x Myr), the mean convergence normal
to the front there (`front_n_c`, `front_n_o` over them, mm/yr = km/Myr) and the pile that delivery implies (per km
over v_n, km of rock). Prints the mean of the runs beside Earth's, from Clift et al. 2009's Table 1 (library text,
32 margins, 40,225 km): its rows split by the agent into continental margins (26,675 km) and intra-oceanic ones
(13,550 km: Mariana, Izu-Bonin, South Luzon, Philippine-Mindanao, Tonga, Kermadec, Solomons, South Sandwich, Lesser
Antilles, Aleutians, Taiwan-North Luzon), each row's thickness its delivery rate over its convergence (Clift's
convergence is the full rate, Clift & Vannucchi's thickness in place); the solid figures put each row's thickness
through ERO-215's porosity law (the agent's derivation). Bird 2003's subduction boundaries: 51,310 km at 53.2 mm/yr
across the trench (Table 3's area flux over its length, the agent's arithmetic recorded in the bibliography).
"""
import re
import statistics
import sys

# Clift et al. 2009, Table 1: (length km, convergence km/Myr, delivery rate km^2/Myr/km = km^3/km/Myr), by class.
CLIFT = {
    "continental": [(2000, 90, 13), (2200, 82, 29), (1100, 70, 20), (450, 80, 17), (275, 78, 12), (500, 75, 12),
                    (1700, 70, 22), (1100, 85, 20), (1100, 80, 36), (1000, 100, 45), (1000, 70, 14), (2000, 20, 43),
                    (850, 38, 48), (550, 42, 61), (2050, 64, 96), (900, 40, 65), (1800, 60, 83), (2100, 77, 54),
                    (1800, 65, 99), (1000, 38, 179), (1200, 20, 131)],
    "oceanic": [(1600, 90, 19), (1300, 95, 19), (400, 90, 19), (1000, 95, 13), (1500, 110, 23), (1250, 70, 14),
                (2750, 110, 17), (700, 77, 16), (850, 40, 137), (1500, 75, 54), (700, 30, 95)],
}
PHI0, LAM = 0.66, 1.333  # ledger: ERO-215 (km)


def solid(z):
    import math
    return z - PHI0 * LAM * (1.0 - math.exp(-z / LAM))


def clift(cls):
    rows = CLIFT[cls] if cls != "all" else CLIFT["continental"] + CLIFT["oceanic"]
    length = sum(r[0] for r in rows)
    vol = sum(r[0] * r[2] for r in rows)
    vol_solid = sum(r[0] * r[2] * solid(r[2] / r[1]) / (r[2] / r[1]) for r in rows)
    conv = sum(r[0] * r[1] for r in rows) / length
    return length, vol / 1e6, vol / length, vol_solid / length, conv


def last_globals(path):
    line = None
    for ln in open(path):
        if ln.startswith("cb_globals "):
            line = ln
    return {k: float(v) for k, v in re.findall(r"(\w+)=([-+0-9.eE]+)", line)} if line else {}


def main(argv):
    length_myr = float(argv[0])
    runs = [last_globals(p) for p in argv[1:]]
    runs = [g for g in runs if g]
    yrs = length_myr * 1.0e6
    out = {"continental": [], "oceanic": [], "one-way": [], "all": []}
    for g in runs:
        c, o, tot = g.get("sed_trench_c", 0.0), g.get("sed_trench_o", 0.0), g.get("sed_trench", 0.0)
        fc, fo = g.get("front_c", 0.0), g.get("front_o", 0.0)
        vc = g.get("front_n_c", 0.0) / max(fc, 1e-9)
        vo = g.get("front_n_o", 0.0) / max(fo, 1e-9)
        out["continental"].append((c / yrs, fc / length_myr, c / max(fc, 1e-9), vc))
        out["oceanic"].append((o / yrs, fo / length_myr, o / max(fo, 1e-9), vo))
        out["one-way"].append(((tot - c - o) / yrs, float("nan"), float("nan"), float("nan")))
        v_all = (g.get("front_n_c", 0.0) + g.get("front_n_o", 0.0)) / max(fc + fo, 1e-9)
        out["all"].append((tot / yrs, (fc + fo) / length_myr, tot / max(fc + fo, 1e-9), v_all))
    mean = lambda xs: statistics.fmean([x for x in xs if x == x]) if any(x == x for x in xs) else float("nan")
    print(f"# {len(runs)} runs; the model's rock beside Clift et al. 2009's Table 1 (in place, and solid by ERO-215)")
    print(f"{'fronts':12s} {'km3/yr':>7s} {'km':>7s} {'per km':>7s} {'v_n':>6s} {'pile km':>8s} | {'Clift km':>8s}"
          f" {'km3/yr':>7s} {'per km':>7s} {'solid':>6s} {'conv':>5s} {'pile':>5s} {'solid pile':>10s}")
    for cls in ("continental", "oceanic", "one-way", "all"):
        rows = out[cls]
        q, L, per, v = (mean([r[k] for r in rows]) for k in range(4))
        pile = per / v if v == v and v > 0 else float("nan")
        if cls in CLIFT or cls == "all":
            cl, cq, cper, csol, cconv = clift(cls)
            earth = (f"{cl:8.0f} {cq:7.2f} {cper:7.1f} {csol:6.1f} {cconv:5.0f} {cper / cconv:5.2f}"
                     f" {csol / cconv:10.2f}")
        else:
            earth = ""
        print(f"{cls:12s} {q:7.2f} {L:7.0f} {per:7.1f} {v:6.1f} {pile:8.2f} | {earth}")
    print("  Bird 2003 (Table 3): subduction boundaries 51,310 km at 53.2 mm/yr across the trench (62.3 relative)")


if __name__ == "__main__":
    main(sys.argv[1:])
