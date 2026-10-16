#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The ground under a census's rock-named biomes (design aid for the geology gate, 2026-10-04).

    tools/review/geology_ground.py <name> [--runs <dir>]

Reads `<runs>/<name>-layers.npy` (`terralith-biomes.sh dump`), the world's `strata/classes.json` and province
notes, and `<runs>/<name>-mic-biomemap.json`; prints the surface rock classes of all land and of coastal land
(within 2 samples of the sea), and for each rock-named Terralith biome the rock, province and climate class
under its columns.
"""
import argparse
import base64
import json
import struct
from collections import Counter

import numpy as np

ROCK_NAMED = ["terralith:shield", "terralith:snowy_shield", "terralith:shield_clearing", "terralith:yosemite_cliffs",
              "terralith:yosemite_lowlands", "terralith:white_cliffs", "terralith:granite_cliffs", "terralith:basalt_cliffs",
              "terralith:bryce_canyon", "terralith:painted_mountains", "terralith:white_mesa", "terralith:sandstone_valley",
              "terralith:desert_canyon", "terralith:stony_spires"]
CODES = ["Af", "Am", "Aw", "As", "BWh", "BWk", "BSh", "BSk", "Csa", "Csb", "Csc", "Cwa", "Cwb", "Cwc", "Cfa",
         "Cfb", "Cfc", "Dsa", "Dsb", "Dsc", "Dsd", "Dwa", "Dwb", "Dwc", "Dwd", "Dfa", "Dfb", "Dfc", "Dfd", "ET", "EF", "sea"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("name")
    ap.add_argument("--runs", default="/Volumes/Graphide/ban/deeptime-runs/terralith-biomes")
    ap.add_argument("--sea-y", type=float, default=63.0)
    a = ap.parse_args()
    base = f"{a.runs}/{a.name}"
    arr = np.load(f"{base}-layers.npy")
    meta = json.load(open(f"{base}-layers.npy.json"))
    ch = {n: i for i, n in enumerate(meta["channels"])}
    rows, cols = arr.shape[:2]
    classes = json.load(open(f"{base}-world/strata/classes.json"))
    names = [c["name"] if isinstance(c, dict) else c for c in (classes.get("classes", classes) if isinstance(classes, dict) else classes)]
    man = json.load(open(f"{base}-world/manifest.json"))
    prov = man["notes"]["bake"]["meta"]["provinces"]
    pclass = {int(k): v["class"] for k, v in prov["province"].items()} if isinstance(prov["province"], dict) else \
        {i: p["class"] for i, p in enumerate(prov["province"])}
    H = arr[:, :, 0]
    K = np.nan_to_num(arr[:, :, ch["climate/koppen"]], nan=31).round().astype(int)
    top = np.nan_to_num(arr[:, :, ch["surface/class_top"]], nan=255).round().astype(int)
    pv = np.nan_to_num(arr[:, :, ch["meta/province"]], nan=-1).round().astype(int)
    land = (K != 31) & (H > a.sea_y)
    sea = ~land
    near = np.zeros_like(sea)
    for dj in (-2, -1, 0, 1, 2):
        for di in (-2, -1, 0, 1, 2):
            near |= np.roll(np.roll(sea, dj, 0), di, 1)
    coast = land & near
    rn = lambda i: names[i] if 0 <= i < len(names) else "nodata"
    def show(title, mask, top_n=14):
        c = Counter(rn(i) for i in top[mask])
        n = mask.sum()
        print(f"{title} ({n}): " + ", ".join(f"{k} {100 * v / n:.1f}" for k, v in c.most_common(top_n)))
    show("land rock", land)
    show("coastal land rock", coast)
    print("provinces on land:", Counter(pclass.get(p, "?") for p in pv[land]).most_common())
    m = json.load(open(f"{base}-mic-biomemap.json"))
    bio = np.array(struct.unpack("<%dH" % (m["w"] * m["h"]), base64.b64decode(m["biome_u16le"]))).reshape(rows, cols)
    pal = [p["id"] for p in m["palette"]]
    for b in ROCK_NAMED:
        if b not in pal:
            print(f"{b}: 0")
            continue
        mask = land & (bio == pal.index(b))
        n = mask.sum()
        if n == 0:
            print(f"{b}: 0")
            continue
        r = Counter(rn(i) for i in top[mask]).most_common(5)
        p = Counter(pclass.get(i, "?") for i in pv[mask]).most_common(3)
        k = Counter(CODES[i] for i in K[mask]).most_common(4)
        print(f"{b}: {n} cols ({100 * n / land.sum():.2f} % of land); rock {r}; province {p}; class {k}; coastal {coast[mask].mean():.2f}")


if __name__ == "__main__":
    main()
