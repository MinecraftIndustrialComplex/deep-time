#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""What the continental crust is made of, by its own crystalline crust: m5cont.py <arm folder>[@N] [map, default t0000.bin]

The M5 stage (a)+(b) arm's continental share grew from base15's 0.386 to 0.634 while m5margins.py's converted
margins (own crystalline crust under 10 km) were 0.066 of it: this splits every continental plate sample by its own
crystalline crust (metamorphic, felsic and mafic pools less the ocean-crust basement, PLT-331) into bands, and gives
for each band its share of the planet, the share under the sea, the median thickness, height below the sea, own
crystalline crust and sediment (sediment and sedimentary rock), the mean own rock by pool (km), the share with an
orogeny tag (Andean, collision) and with 1 km or more of island-arc magma. Medians over runs. Needs numpy only."""
import json
import statistics
import sys
from pathlib import Path

import numpy as np

MAP = 11
POOLS = 9
BANDS = [(0, 10), (10, 15), (15, 20), (20, 25), (25, 30), (30, 1e9)]
NAMES = ["sed", "sedr", "meta", "fplu", "fvol", "mvol_own", "mplu_own", "basement"]


def sea_m(run):
    steps = json.loads((run / "rec.json").read_text()).get("steps") or []
    return float(steps[-1]["sea_m"]) if steps and steps[-1].get("sea_m") is not None else 0.0


def one(run, name):
    a = np.fromfile(run / "maps" / name, "<f4").reshape(-1, MAP).astype(np.float64)
    pp = np.fromfile(run / "maps" / "pools" / name, "<f4").reshape(-1, POOLS).astype(np.float64)
    s = sea_m(run)
    z, cont, orog, iarc = a[:, 2], a[:, 4] > 0.5, a[:, 5], a[:, 7]
    b = pp[:, 7]
    # The basement's share of each mafic pool, as Sample::own_pools takes it (gabbro first, 5 of 7 km).
    gab = np.minimum(b * 5.0 / 7.0, np.maximum(pp[:, 6], 0.0))
    lava = np.minimum(b - gab, np.maximum(pp[:, 5], 0.0))
    gab = np.minimum(b - lava, np.maximum(pp[:, 6], 0.0))
    own = np.stack([pp[:, 0], pp[:, 1], pp[:, 2], pp[:, 3], pp[:, 4], pp[:, 5] - lava, pp[:, 6] - gab, b], 1)
    oc = own[:, 2:7].sum(1)
    n = len(a)
    out = {}
    for lo, hi in BANDS:
        m = cont & (oc >= lo * 1e3) & (oc < hi * 1e3)
        k = int(m.sum())
        row = {"share": k / n, "wet": float((z[m] < s).mean()) if k else float("nan")}
        if k:
            row["T_km"] = float(np.median(pp[m, :7].sum(1))) / 1e3
            row["depth_m"] = float(np.median(s - z[m]))
            row["own_km"] = float(np.median(oc[m])) / 1e3
            row["sed_km"] = float(np.median(pp[m, 0] + pp[m, 1])) / 1e3
            row["orog"] = float((orog[m] > 0.5).mean())
            row["iarc"] = float((iarc[m] >= 1000.0).mean())
            for j, nm in enumerate(NAMES):
                row[nm] = float(own[m, j].mean()) / 1e3
        out[f"{lo}-{hi:g}"] = row
    # The sea's sediment (sediment and sedimentary rock under the sea, both kinds of crust; GlobSed's grid covers the
    # margins too), planet-mean m, and the continental crust's part of it.
    wet = z < s
    pile = pp[:, 0] + pp[:, 1]
    out["sea_sediment"] = {"share": float(wet.mean()), "planet_m": float((pile * wet).sum() / n),
                           "continental_m": float((pile * (wet & cont)).sum() / n)}
    return out


def main():
    folder, _, top = sys.argv[1].partition("@")
    name = sys.argv[2] if len(sys.argv) > 2 else "t0000.bin"
    rows = []
    for run in sorted(q for q in Path(folder).iterdir() if (q / "maps" / "pools" / name).exists()):
        if top and int(run.name.rsplit("-", 1)[1]) > int(top):
            continue
        rows.append(one(run, name))
    if not rows:
        print("no runs with maps/pools/" + name)
        return
    print(f"{len(rows)} runs, map {name}: continental plate samples by own crystalline crust, km (medians over runs)")
    sea = {k: statistics.median(r["sea_sediment"][k] for r in rows) for k in rows[0]["sea_sediment"]}
    for band in [b for b in rows[0] if b != "sea_sediment"]:
        keys = sorted({k for r in rows for k in r[band]}, key=list(rows[0][band]).index)
        med = {}
        for k in keys:
            v = [r[band][k] for r in rows if k in r[band] and r[band][k] == r[band][k]]
            med[k] = statistics.median(v) if v else float("nan")
        if med["share"] <= 0.0:
            print(f"  {band:>8}: share 0")
            continue
        print(f"  {band:>8}: share {med['share']:.3f} wet {med['wet']:.2f} T {med['T_km']:.1f} depth {med['depth_m']:.0f} m"
              f" own {med['own_km']:.1f} sed {med['sed_km']:.1f} orogeny {med['orog']:.2f} iarc {med['iarc']:.2f};"
              " mean km " + " ".join(f"{nm} {med[nm]:.1f}" for nm in NAMES))
    print(f"  the sea's sediment and sedimentary rock (both kinds, {sea['share']:.3f} of the planet under the sea):"
          f" {sea['planet_m']:.0f} m planet-mean ({sea['planet_m'] * 5.1007e5:.3g} km3), on continental crust"
          f" {sea['continental_m']:.0f} m")


if __name__ == "__main__":
    main()
