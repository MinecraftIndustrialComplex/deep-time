#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The continental share and the crust under the sea with the converted margins and without them:
m5margins.py <arm folder>[@N] [map name, default t0000.bin]

The M5 stage (a)+(b) arm's census (coordinator, 2026-10-25: "the relabel moves samples into the continental count.
So give M3 (crust under the sea) and the continental share two ways, with the converted margins and without them";
Earth's reference for the "with" reading is Cogley 1984's 0.41 share with 30.6 % under the sea). From each run's
crust-budget maps (`DEEPTIME_CRUST_BUDGET_MAPS`; plates/crust_budget.rs): maps/t*.bin (MAP_FIELDS, 11 f32) and
maps/pools/t*.bin (POOL_FIELDS, 9 f32), in the same visible-sample order, and the record's last sea over the datum
(rec.json, `sea_m`), on the plate model's samples (not the census grid).

A converted margin is what PLT-661 and PLT-662 turn continental: continental crust over an ocean-crust basement
(`basement` > 0, PLT-331) whose own crystalline crust (metamorphic, felsic and mafic pools less the basement) is
thinner than 10 km (transition.rs HYPER_EXTENDED_M, Brune et al. 2014's hyper-extended crust). A matured island arc
(make_continental at 15 km, PLT-331) has a basement too; it is told apart by its island-arc magma (1 km or more,
MAP_FIELDS `iarc`) and counted on its own (`arc`). On a switch-off run the converted count is what the rule finds
with no relabel (the floor of the reading). Medians over runs. Needs numpy only."""
import json
import statistics
import sys
from pathlib import Path

import numpy as np

MAP = 11
POOLS = 9
HYPER_EXTENDED_M = 10_000.0  # ledger: PLT-661 (transition.rs HYPER_EXTENDED_M)
ARC_MAGMA_M = 1000.0


def sea_m(run):
    rec = json.loads((run / "rec.json").read_text())
    steps = rec.get("steps") or []
    return float(steps[-1]["sea_m"]) if steps and steps[-1].get("sea_m") is not None else 0.0


def one(run, name):
    a = np.fromfile(run / "maps" / name, "<f4").reshape(-1, MAP).astype(np.float64)
    pp = np.fromfile(run / "maps" / "pools" / name, "<f4").reshape(-1, POOLS).astype(np.float64)
    assert len(a) == len(pp), (run, len(a), len(pp))
    n = len(a)
    s = sea_m(run)
    z = a[:, 2]
    cont = a[:, 4] > 0.5
    iarc = a[:, 7]
    basement = pp[:, 7]
    own_cryst = pp[:, 2:7].sum(1) - basement
    hx = cont & (basement > 0.0) & (own_cryst < HYPER_EXTENDED_M)
    arc = hx & (iarc >= ARC_MAGMA_M)
    conv = hx & ~arc
    wet = z < s
    kept = cont & ~conv
    out = {
        "sea_m": s,
        "cont_with": cont.mean(),
        "cont_without": kept.mean(),
        "under_with": (cont & wet).sum() / max(int(cont.sum()), 1),
        "under_without": (kept & wet).sum() / max(int(kept.sum()), 1),
        "conv": conv.mean(),
        "conv_wet": (conv & wet).sum() / max(int(conv.sum()), 1),
        "conv_depth_m": float(np.median(s - z[conv])) if conv.any() else float("nan"),
        "conv_T_km": float(np.median(pp[conv, :7].sum(1)) / 1000.0) if conv.any() else float("nan"),
        "conv_sed_km": float(np.median(pp[conv, 0] + pp[conv, 1]) / 1000.0) if conv.any() else float("nan"),
        "arc": arc.mean(),
        "basement_any": (cont & (basement > 0.0)).mean(),
    }
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
    for r, v in rows:
        print(f"  {r}: " + ", ".join(f"{k} {x:.4f}" for k, x in v.items()))
    m = {k: statistics.median(v[k] for _, v in rows) for k in rows[0][1]}
    print(f"{len(rows)} runs, map {name}, medians (plate samples; Earth, Cogley 1984: share 0.41, 0.306 under the sea)")
    print(f"  continental share: with the converted margins {m['cont_with']:.3f}, without {m['cont_without']:.3f}")
    print(f"  under the sea:     with {m['under_with']:.3f}, without {m['under_without']:.3f} (sea {m['sea_m']:.0f} m)")
    print(f"  converted margins {m['conv']:.4f} of the planet, {m['conv_wet']:.2f} of them under the sea, depth p50 "
          f"{m['conv_depth_m']:.0f} m, crust p50 {m['conv_T_km']:.1f} km, sediment p50 {m['conv_sed_km']:.1f} km")
    print(f"  matured arcs over a basement {m['arc']:.4f}; any continental crust over a basement {m['basement_any']:.4f}")


if __name__ == "__main__":
    main()
