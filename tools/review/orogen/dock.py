# SPDX-License-Identifier: LGPL-3.0-or-later
"""The orogen lane: collision fronts' sides (the dock tally) and the collision crust's split (globals), per arm.

Usage (VM): python3 -I dock.py <arm folder> [<arm folder> ...]
An arm folder is .../pool/<key>/<preset>-<length>/<argkey>. Prints, per run and as medians:
the dock tally's edge_dry and crust_dry (front-time weighted over states under 30 Myr), the share of
front-time in those states, and the kept collision crust's split: wedge (incoming plate), upper
(interior), stays (no room), toe (the taper belt past its width); the belt holds the rest.
"""
import json
import os
import statistics
import sys

G = "plates/collision_"


def dock(summary):
    ages = {}
    for line in open(summary, errors="replace"):
        line = line.strip()
        if line.startswith("dock_age="):
            parts = dict(p.split("=", 1) for p in line.split())
            km = [float(x) for x in parts["km_myr"].split(",")]
            means = [float(x) for x in parts["means"].split(",")]
            ages[parts["dock_age"]] = (sum(km), means)
    return ages


def flat_globals(path):
    gl = json.load(open(path))
    if isinstance(gl, dict) and "globals" in gl:
        gl = gl["globals"]
    out = {}
    if isinstance(gl, dict):
        for k, v in gl.items():
            if isinstance(v, list):
                v = v[-1] if v else None
            if isinstance(v, (int, float)):
                out[k] = float(v)
    return out


def run(folder):
    out = {"run": os.path.basename(folder)}
    s = os.path.join(folder, "summary.txt")
    if os.path.exists(s):
        a = dock(s)
        tot = sum(w for w, _ in a.values())
        young = [a[k] for k in ("0", "10") if k in a]
        w = sum(x for x, _ in young)
        # means order: under_cont, under_near, crust_dry, edge_dry, conv_mm_yr, crust_n
        for j, name in ((2, "crust_dry"), (3, "edge_dry"), (1, "under_near"), (0, "under_cont"),
                        (4, "conv_mm_yr"), (5, "crust_n")):
            v = [m[j] * x for x, m in young if x > 0 and m[j] == m[j]]
            out[name + "_lt30"] = sum(v) / w if w > 0 else float("nan")
        out["young_share"] = w / tot if tot > 0 else float("nan")
    g = os.path.join(folder, "fields", "globals.json")
    if os.path.exists(g):
        flat = flat_globals(g)
        kept = flat.get(G + "kept_km3")
        if kept:
            for k in ("wedge_km3", "upper_km3", "noroom_stays_km3", "taper_toe_km3"):
                v = flat.get(G + k)
                if v is not None:
                    out[k.replace("_km3", "") + "_of_kept"] = v / kept
            out["kept_km3"] = kept
            cr = flat.get(G + "crossed_km3")
            if cr:
                out["kept_of_crossed"] = kept / cr
    return out


def main():
    for arm in sys.argv[1:]:
        rows = []
        for name in sorted(os.listdir(arm)):
            p = os.path.join(arm, name)
            if os.path.isdir(p) and os.path.exists(os.path.join(p, "meta.json")):
                rows.append(run(p))
        print(f"== {arm} ({len(rows)} runs)")
        keys = sorted({k for r in rows for k in r if k != "run"})
        for r in rows:
            print("  " + r["run"].ljust(24) + " ".join(f"{k}={r[k]:.3g}" for k in keys if k in r))
        for k in keys:
            v = [r[k] for r in rows if k in r and r[k] == r[k]]
            if v:
                print(f"  median {k:24s} {statistics.median(v):.4g}  (n {len(v)}, min {min(v):.3g}, max {max(v):.3g})")


if __name__ == "__main__":
    main()
