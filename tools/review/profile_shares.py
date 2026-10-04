#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Biome shares of one planet through several profiles' biome sources, side by side (decisions log
2026-10-04, Expanded Ecosphere beside Terralith): the `biomemap.json` grids that `/deeptime probe biomes 16`
wrote for each profile (tools/review/ee-biomes.sh census), as a Markdown table per Köppen class of interest
and an overall line per profile.

    tools/review/profile_shares.py ee=<grid> mic=<grid> standalone=<grid> [--classes ET,EF,...] [--top 3]

Land columns only (the sea class and columns at or below the sea are left out). Each cell lists the class's
commonest biomes with their share of its columns.
"""
import argparse
import base64
import json
import struct
from collections import Counter

DEFAULT = "Af,Am,Aw,As,BWh,BWk,BSh,BSk,Csa,Csb,Cfa,Cfb,Cfc,Dfb,Dfc,Dfd,ET,EF"


def load(path):
    m = json.load(open(path))
    w, h = m["w"], m["h"]
    biome = struct.unpack("<%dH" % (w * h), base64.b64decode(m["biome_u16le"]))
    koppen = base64.b64decode(m["koppen_u8"])
    height = struct.unpack("<%dh" % (w * h), base64.b64decode(m["height_i16le"]))
    pal = [p["id"] for p in m["palette"]]
    codes = m["koppen_codes"]
    sea = m["sea_level"]
    by, total = {}, Counter()
    for i in range(w * h):
        k = koppen[i]
        if k == 31 or height[i] <= sea:
            continue
        b = pal[biome[i]]
        by.setdefault(codes[k], Counter())[b] += 1
        total[b] += 1
    return by, total, m.get("census", {})


def short(b):
    return b.replace("minecraft:", "").replace("wythers:", "ee:").replace("terralith:", "tl:")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("grids", nargs="+", help="profile=path")
    ap.add_argument("--classes", default=DEFAULT)
    ap.add_argument("--top", type=int, default=3)
    a = ap.parse_args()
    profiles = []
    for g in a.grids:
        name, path = g.split("=", 1)
        profiles.append((name,) + load(path))
    print("| class | " + " | ".join(p[0] for p in profiles) + " |")
    print("|---|" + "---|" * len(profiles))
    for c in a.classes.split(","):
        cells = []
        n = 0
        for _, by, _, _ in profiles:
            cnt = by.get(c, Counter())
            s = sum(cnt.values())
            n = max(n, s)
            cells.append(", ".join("%s %d" % (short(b), round(100 * v / s)) for b, v in cnt.most_common(a.top)) if s else "-")
        print("| %s (%d) | %s |" % (c, n, " | ".join(cells)))
    print()
    print("| profile | land columns | biomes on land | share on the mod's own biomes | top biomes |")
    print("|---|---|---|---|---|")
    for name, by, total, census in profiles:
        s = sum(total.values())
        mod = sum(v for b, v in total.items() if not b.startswith("minecraft:"))
        print("| %s | %d | %d | %.2f | %s |" % (name, s, len(total), mod / s if s else 0,
                                               ", ".join("%s %.1f" % (short(b), 100 * v / s) for b, v in total.most_common(5))))


if __name__ == "__main__":
    main()
