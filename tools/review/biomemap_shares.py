#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Biome shares per Koppen class from the `biomemap.json` that `/deeptime probe biomes` writes.

    tools/review/biomemap_shares.py <biomemap.json> [--classes BWk,BSk,BWh] [--top 12]
    tools/review/biomemap_shares.py <before.json> <after.json> [--classes ...]   # side by side

Land columns only (sea class and columns below the sea are left out), within the pole margin the
census used. Prints, per class, the number of sampled columns and each biome's share of them.
"""
import argparse
import base64
import json
import struct
from collections import Counter


def load(path):
    m = json.load(open(path))
    w, h = m["w"], m["h"]
    biome = struct.unpack("<%dH" % (w * h), base64.b64decode(m["biome_u16le"]))
    koppen = base64.b64decode(m["koppen_u8"])
    height = struct.unpack("<%dh" % (w * h), base64.b64decode(m["height_i16le"]))
    pal = [p["id"] for p in m["palette"]]
    codes = m["koppen_codes"]
    sea = m["sea_level"]
    by = {}
    for i in range(w * h):
        k = koppen[i]
        if k == 31 or height[i] <= sea:
            continue
        by.setdefault(codes[k], Counter())[pal[biome[i]]] += 1
    return by


def show(by, classes, top):
    for c in classes:
        cnt = by.get(c)
        if not cnt:
            print(f"{c}: no columns")
            continue
        n = sum(cnt.values())
        print(f"{c}: {n} columns")
        for b, v in cnt.most_common(top):
            print(f"    {100 * v / n:5.1f} %  {b}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("maps", nargs="+")
    ap.add_argument("--classes", default="BWk,BSk,BWh,BSh")
    ap.add_argument("--top", type=int, default=12)
    a = ap.parse_args()
    classes = a.classes.split(",")
    for path in a.maps:
        print(f"== {path}")
        show(load(path), classes, a.top)


if __name__ == "__main__":
    main()
