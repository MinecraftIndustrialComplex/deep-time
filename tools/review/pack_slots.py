#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Slot map of a pack's overworld biome parameter list (the per-pack placement tables' design aid).

    tools/review/pack_slots.py <jar or parameter-list json> W3,W4,T0 [--weird -0.5,0.5]
    tools/review/pack_slots.py <jar> --biome terralith:steppe        # that biome's entries

For each named cell (temperature band letter F C T W H, humidity band digit 0-4, vanilla's
OverworldBiomeBuilder edges) prints the nearest biome at the middle of each erosion band E0-E6, for
coast, near-, mid- and far-inland continentalness and each weirdness value: what the climate search
answers there (vanilla's metric: the sum of squared distances outside each entry's ranges, plus the
offset; ties go to the first entry). Depth 0 (the surface).
"""
import argparse
import glob
import json
import zipfile

PL = "data/minecraft/worldgen/multi_noise_biome_source_parameter_list/overworld.json"
T = [-1.0, -0.45, -0.15, 0.2, 0.55, 1.0]
H = [-1.0, -0.35, -0.1, 0.1, 0.3, 1.0]
ERO = [-1.0, -0.78, -0.375, -0.2225, 0.05, 0.45, 0.55, 1.0]
CONT = {"coast": -0.15, "near": -0.04, "mid": 0.16, "far": 0.6}
KEYS = ("temperature", "humidity", "continentalness", "erosion", "weirdness", "depth")


def rng(v):
    if isinstance(v, (int, float)):
        return (float(v), float(v))
    if isinstance(v, list):
        return (float(v[0]), float(v[1]))
    return (float(v["min"]), float(v["max"]))


def load(path):
    if path.endswith(".jar"):
        pl = json.loads(zipfile.ZipFile(glob.glob(path)[0]).read(PL))
    else:
        pl = json.load(open(path))
    entries = pl.get("lithostitched:biomes") or pl.get("biomes")
    return [(e["biome"], [rng(e["parameters"][k]) for k in KEYS], float(e["parameters"].get("offset", 0)))
            for e in entries]


def dist(v, r):
    lo, hi = r
    return lo - v if v < lo else (v - hi if v > hi else 0.0)


def nearest(entries, pt):
    best, bd = None, 1e18
    for b, rs, off in entries:
        s = sum(dist(v, r) ** 2 for v, r in zip(pt, rs)) + off * off
        if s < bd:
            bd, best = s, b
    return best


def mid(edges, i):
    return 0.5 * (edges[i] + edges[i + 1])


def short(b):
    return b.replace("terralith:", "t:").replace("minecraft:", "")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("source")
    ap.add_argument("cells", nargs="?", default="")
    ap.add_argument("--weird", default="-0.5,0.5")
    ap.add_argument("--biome")
    a = ap.parse_args()
    entries = load(a.source)
    if a.biome:
        for b, rs, off in entries:
            if b == a.biome:
                print(b, " ".join(f"{k[:4]}={lo:+.3f}..{hi:+.3f}" for k, (lo, hi) in zip(KEYS, rs)), f"off={off}")
        return
    weird = [float(x) for x in a.weird.split(",")]
    for cell in a.cells.split(","):
        ti, hi = "FCTWH".index(cell[0]), int(cell[1])
        print(f"== {cell}")
        for cn, cv in CONT.items():
            parts = []
            for ei in range(7):
                for w in weird:
                    parts.append(f"E{ei}{'+' if w > 0 else '-'} {short(nearest(entries, (mid(T, ti), mid(H, hi), cv, mid(ERO, ei), w, 0.0)))}")
            print(f"  {cn:5s} " + " | ".join(parts))


if __name__ == "__main__":
    main()
