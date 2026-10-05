#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Placer and spawn report (tools/review/placer-spawn-compare.sh).

For each `seed=<bake folder>` it reads `<folder>-deposits.json` (`deeptime dump-deposits`) and the
bake's `spawn/candidates.json`, and prints per seed: the porphyries, the placers (shed by an eroded
porphyry: no parent; by a cut porphyry: parent set) with the gold they hold, and where the spawn
lands (the first candidate) and how far it is from (0, 0).

    placer-spawn-compare.py 1=<folder> 2=<folder> ... [--json out.json]
"""
import argparse
import json
import math
import os


def gold_kg(s):
    return sum(p["tonnage_t"] * p["grade"] for p in s["products"] if p["material"] == "c:gold") * 1000


def summarize(seed, folder):
    dump = json.load(open(folder + "-deposits.json"))
    cands = json.load(open(os.path.join(folder, "spawn", "candidates.json")))
    sites = dump["sites"]
    porphyries = [s for s in sites if s["deposit_type"].endswith("porphyry_cu_mo_au")]
    placers = [s for s in sites if s["deposit_type"].endswith("placer_au")]
    from_cut = [s for s in placers if s.get("parent") is not None]
    first = cands["candidates"][0]
    c = dump["circumference"]
    dx = abs(first["x"]) % c
    return {
        "seed": seed,
        "sites": len(sites),
        "porphyries": len(porphyries),
        "placers": len(placers),
        "placers_from_eroded": len(placers) - len(from_cut),
        "placers_from_cut": len(from_cut),
        "cut_porphyries_shedding": len({s["parent"] for s in from_cut}),
        "placer_gold_kg": round(sum(gold_kg(s) for s in placers)),
        "spawn": {
            "x": first["x"],
            "z": first["z"],
            "y": first["surface_y"],
            "from_origin": round(math.hypot(min(dx, c - dx), first["z"])),
            "ranked": "unmet" in first,
        },
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("runs", nargs="+", metavar="SEED=FOLDER")
    ap.add_argument("--json")
    a = ap.parse_args()
    rows = [summarize(*r.split("=", 1)) for r in a.runs]
    print("| seed | sites | porphyries | placers | from eroded | from cut | cut porphyries shedding | placer gold (kg) | spawn (x, y, z) | from (0, 0) | ranked |")
    print("|---|---|---|---|---|---|---|---|---|---|---|")
    for r in rows:
        sp = r["spawn"]
        print("| %s | %d | %d | %d | %d | %d | %d | %d | %d, %s, %d | %d | %s |" % (
            r["seed"], r["sites"], r["porphyries"], r["placers"], r["placers_from_eroded"], r["placers_from_cut"],
            r["cut_porphyries_shedding"], r["placer_gold_kg"], sp["x"], sp["y"], sp["z"], sp["from_origin"],
            "yes" if sp["ranked"] else "no"))
    if a.json:
        json.dump(rows, open(a.json, "w"), indent=1)


if __name__ == "__main__":
    main()
