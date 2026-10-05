#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P1-15 spawn survey report (tools/review/p1-15-spawn-survey.sh).

For each `seed=<bake folder>` it reads the bake's `spawn/candidates.json` and the `<folder>-deposits.json`
that `deeptime dump-deposits` wrote, and prints, per seed: the sites by type, the chosen spawn (the first
candidate), and for the materials that matter how far the nearest site of each kind is from it: any
copper site, a copper-primary site, a porphyry, a porphyry holding at least `--min-mt` Mt, and a
porphyry's placer-gold child.

    p1-15-spawn-survey.py 1=<folder> 2=<folder> … [--reach 3072] [--min-mt 30] [--json out.json]
"""
import argparse
import collections
import json
import math
import os


def dist(c, a, b):
    dx = (a[0] - b[0]) % c
    dx = min(dx, c - dx)
    return math.hypot(dx, a[2] - b[2])


def load(folder):
    dump = json.load(open(folder + "-deposits.json"))
    cands = json.load(open(os.path.join(folder, "spawn", "candidates.json")))
    return dump["circumference"], dump["sites"], cands


def tonnage(s, material=None):
    return max([p["tonnage_t"] for p in s["products"] if material is None or p["material"] == material] or [0.0])


def summarize(seed, folder, reach, min_mt):
    c, sites, cands = load(folder)
    by_id = {s["id"]: s for s in sites}
    types = collections.Counter(s["deposit_type"].split(":")[-1] for s in sites)
    first = cands["candidates"][0]
    spawn = (first["x"], 0, first["z"])

    def nearest(pred):
        best = None
        for s in sites:
            if pred(s):
                d = dist(c, spawn, s["center"])
                if best is None or d < best[0]:
                    best = (d, s)
        return best

    def desc(b):
        if b is None:
            return None
        d, s = b
        return {"id": s["id"], "type": s["deposit_type"].split(":")[-1], "distance": round(d),
                "center": s["center"], "mt": round(tonnage(s) / 1e6, 2)}

    is_porphyry = lambda s: s["deposit_type"].endswith("porphyry_cu_mo_au")
    copper_primary = lambda s: any(p["material"] == "c:copper" and p["role"] == "primary" for s in [s] for p in s["products"])
    any_copper = lambda s: any(p["material"] == "c:copper" for p in s["products"])
    porphyries = [s for s in sites if is_porphyry(s)]
    kids = {s["id"]: [k for k in sites if k.get("parent") == s["id"]] for s in porphyries}
    row = {
        "seed": seed,
        "sites": len(sites),
        "types": dict(types),
        "spawn": {"x": first["x"], "z": first["z"], "y": first["surface_y"], "unmet": first["unmet"]},
        "essentials_at_spawn": [(e["material"], e["distance"]) for e in first["essentials"]],
        "nearest_any_copper": desc(nearest(any_copper)),
        "nearest_copper_primary": desc(nearest(copper_primary)),
        "nearest_porphyry": desc(nearest(is_porphyry)),
        "nearest_porphyry_big": desc(nearest(lambda s: is_porphyry(s) and tonnage(s) >= min_mt * 1e6)),
        "porphyries": len(porphyries),
        "porphyries_with_placer_child": sum(1 for v in kids.values() if any(k["deposit_type"].endswith("placer_au") for k in v)),
        "porphyry_within_reach": None,
        "reach": reach,
    }
    np_ = row["nearest_porphyry"]
    row["porphyry_within_reach"] = bool(np_ and np_["distance"] <= reach)
    return row


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("runs", nargs="+", metavar="SEED=FOLDER")
    ap.add_argument("--reach", type=float, default=3072)
    ap.add_argument("--min-mt", type=float, default=30)
    ap.add_argument("--json")
    a = ap.parse_args()
    rows = []
    for r in a.runs:
        seed, folder = r.split("=", 1)
        rows.append(summarize(seed, folder, a.reach, a.min_mt))
    print("| seed | sites | porphyries | spawn (x, z) | unmet | nearest any copper | nearest porphyry | nearest porphyry >= %g Mt | porphyry within %d |" % (a.min_mt, a.reach))
    print("|---|---|---|---|---|---|---|---|---|")
    fmt = lambda d: "none" if d is None else "%s #%d %d blocks (%s Mt)" % (d["type"], d["id"], d["distance"], d["mt"])
    for r in rows:
        print("| %s | %d | %d | %d, %d | %d | %s | %s | %s | %s |" % (
            r["seed"], r["sites"], r["porphyries"], r["spawn"]["x"], r["spawn"]["z"], r["spawn"]["unmet"],
            fmt(r["nearest_any_copper"]), fmt(r["nearest_porphyry"]), fmt(r["nearest_porphyry_big"]),
            "yes" if r["porphyry_within_reach"] else "NO"))
    print()
    for r in rows:
        print(r["seed"], dict(sorted(r["types"].items())), "porphyries with a placer child:", r["porphyries_with_placer_child"])
    if a.json:
        json.dump(rows, open(a.json, "w"), indent=1)


if __name__ == "__main__":
    main()
