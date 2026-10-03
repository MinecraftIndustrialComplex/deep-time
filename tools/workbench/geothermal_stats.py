#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Geothermal census of an area export (`deeptime-inspect export-area`, .dtva or .dtva.gz), P2-14.

    python3 tools/workbench/geothermal_stats.py <area.dtva.gz> [--sites probes.jsonl] [--png out.png] [--json out.json] [--scale 4]

With `--sites` (the job's probes.jsonl, which holds `deeptime probe geothermal`) every count and check
is restricted to the columns within reach of a site, so lakes, rivers and natural terracotta elsewhere
in the box do not count. Counts the blocks the geothermal pass places (sinter/calcite, travertine/dripstone, the altered-ground
terracottas, sulfur, cinnabar, vent magma blocks), then checks the water-safety rules on the blocks the
generator really drew: no water block above the sea with air beside it (a leak), no water touching lava,
no magma block under water (a bubble column), and how many columns have one of the pass's blocks on
top. `--png` writes a top-down picture of each column's top ground block (plants, leaves and logs skipped). Standard library only; a
192 x 192 x 1152 box takes about half a minute. The reader is the one of `lava_stats.py`.
"""
import array
import gzip
import json
import struct
import sys
import zlib

OURS = {
    "minecraft:calcite": (236, 236, 228),
    "minecraft:dripstone_block": (150, 118, 86),
    "minecraft:white_terracotta": (209, 178, 161),
    "minecraft:yellow_terracotta": (186, 133, 35),
    "minecraft:orange_terracotta": (161, 83, 37),
    "minecraft:yellow_concrete": (240, 175, 21),
    "minecraft:red_terracotta": (143, 61, 46),
    "minecraft:magma_block": (190, 40, 10),
}
WATER = (48, 80, 200)
SEA = 63


def load(path):
    d = gzip.open(path).read() if path.endswith(".gz") else open(path, "rb").read()
    hl = struct.unpack_from("<I", d, 8)[0]
    h = json.loads(d[12:12 + hl])
    blob = 12 + hl
    blob += (-blob) % 4

    def sec(name):
        s = h["sections"][name]
        a = array.array("H")
        a.frombytes(d[blob + s["offset"]: blob + s["offset"] + s["count"] * 2])
        return a

    return h, sec("run_counts"), sec("run_values"), sec("run_lengths")


def color(bid):
    if bid in OURS:
        return OURS[bid]
    if "water" in bid:
        return WATER
    if "lava" in bid:
        return (255, 120, 0)
    if "grass" in bid:
        return (90, 130, 60)
    if "snow" in bid or "ice" in bid:
        return (240, 245, 250)
    if "sand" in bid:
        return (220, 205, 150)
    if "stone" in bid or "deepslate" in bid or "andesite" in bid or "granite" in bid or "diorite" in bid:
        return (120, 120, 120)
    if "dirt" in bid or "podzol" in bid or "mud" in bid:
        return (110, 80, 50)
    if "basalt" in bid:
        return (60, 60, 70)
    n = sum(bid.encode())
    return (60 + n % 120, 60 + (n * 7) % 120, 60 + (n * 13) % 120)


def write_png(path, w, h, rgb):
    raw = b"".join(b"\x00" + bytes(rgb[y * w * 3:(y + 1) * w * 3]) for y in range(h))

    def chunk(t, data):
        c = struct.pack(">I", len(data)) + t + data
        return c + struct.pack(">I", zlib.crc32(t + data) & 0xFFFFFFFF)

    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
                + chunk(b"IDAT", zlib.compress(raw, 6)) + chunk(b"IEND", b""))


def main():
    args = sys.argv[1:]
    path = args[0]
    opt = {args[i]: args[i + 1] for i in range(1, len(args) - 1, 2)}
    h, rc, rv, rl = load(path)
    w, dep, y0, y1 = h["width"], h["depth"], h["y0"], h["y1"]
    bx0, bz0 = h["box"]["x0"], h["box"]["z0"]
    near = None
    pool_mask = bytearray(w * dep)
    vent_mask = bytearray(w * dep)
    sites_in_box = 0
    if "--sites" in opt:
        near = bytearray(w * dep)
        for line in open(opt["--sites"]):
            rec = json.loads(line)
            if rec.get("command") != "deeptime probe geothermal" or not rec.get("probe"):
                continue
            for s in rec["probe"]["siteList"]:
                reach = int((s["apron"] + 2) * 1.34) + 2
                lx, lz = s["x"] - bx0, s["z"] - bz0
                if -reach <= lx < w + reach and -reach <= lz < dep + reach:
                    sites_in_box += 1
                hard = int((s["r"] + 2) * 1.34) + 1
                for ix in range(max(0, lx - reach), min(w, lx + reach + 1)):
                    for iz in range(max(0, lz - reach), min(dep, lz + reach + 1)):
                        near[iz * w + ix] = 1
                        if s["kind"] == "pool" and abs(ix - lx) <= hard and abs(iz - lz) <= hard:
                            pool_mask[iz * w + ix] = 1
                        if s["kind"] == "vent" and abs(ix - lx) <= hard and abs(iz - lz) <= hard:
                            vent_mask[iz * w + ix] = 1
    ht = y1 - y0 + 1
    ids = [h["blocks"][e[0]]["id"] for e in h["entries"]]
    cats = [h["blocks"][e[0]]["cat"] for e in h["entries"]]
    # kind per entry: 0 solid, 1 air, 2 water, 3 lava
    kind = [1 if c == "air" else 2 if c == "water" else 3 if c == "lava" else 0 for c in cats]
    magma_i = {i for i, b in enumerate(ids) if b == "minecraft:magma_block"}
    grid = bytearray(w * dep * ht)
    entry = array.array("H", [0]) * (w * dep * ht)
    counts = {}
    k = 0
    top = [None] * (w * dep)
    for c in range(w * dep):
        y = 0
        for _ in range(rc[c]):
            v, n = rv[k], rl[k]
            k += 1
            base = c * ht + y
            kd = kind[v]
            if kd:
                grid[base:base + n] = bytes([kd]) * n
            for i in range(base, base + n):
                entry[i] = v
            if near is None or near[c]:
                counts[ids[v]] = counts.get(ids[v], 0) + n
            if kd != 1 and cats[v] != "plant":   # the ground, not the trees and flowers over it
                top[c] = v
            y += n
    # Water-safety on the generator's own blocks.
    leaks = wet_lava = bubble = water_above_sea = 0
    first = None
    for c in range(w * dep):
        cx, cz = c % w, c // w
        if near is not None and not near[c]:
            continue
        for yy in range(ht):
            i = c * ht + yy
            if grid[i] == 2 and yy + y0 >= SEA + 3 and pool_mask[c]:
                water_above_sea += 1
                for dx, dz in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                    nx, nz = cx + dx, cz + dz
                    if 0 <= nx < w and 0 <= nz < dep and grid[(nz * w + nx) * ht + yy] == 1:
                        leaks += 1
                        if first is None:
                            first = (cx, cz, yy + y0)
            if grid[i] == 2:
                for ddy in (-1, 1):
                    if 0 <= yy + ddy < ht and grid[i + ddy] == 3:
                        wet_lava += 1
                if yy > 0 and entry[i - 1] in magma_i and vent_mask[c]:
                    bubble += 1
    mine = {b: n for b, n in counts.items() if b in OURS}
    on_top = {}
    for c in range(w * dep):
        if near is not None and not near[c]:
            continue
        if top[c] is not None and ids[top[c]] in OURS:
            on_top[ids[top[c]]] = on_top.get(ids[top[c]], 0) + 1
    res = {
        "box": h["box"], "sites_reaching_box": sites_in_box, "columns_near_sites": sum(near) if near is not None else None, "y": [y0, y1], "profile": h.get("meta", {}).get("profile"),
        "blocks": mine, "columns_topped": on_top, "pool_water_blocks": water_above_sea,
        "pool_water_leaks_sideways": leaks, "first_leak": first, "water_touching_lava_near_sites": wet_lava,
        "vent_magma_under_water": bubble, "pool_columns": sum(pool_mask), "vent_columns": sum(vent_mask),
    }
    print(json.dumps(res, indent=1))
    if "--json" in opt:
        json.dump(res, open(opt["--json"], "w"), indent=1)
    if "--png" in opt:
        sc = int(opt.get("--scale", 4))
        rgb = bytearray()
        for z in range(dep):
            row = bytearray()
            for x in range(w):
                t = top[z * w + x]
                row += bytes(color(ids[t]) if t is not None else (0, 0, 0)) * sc
            for _ in range(sc):
                rgb += row
        write_png(opt["--png"], w * sc, dep * sc, rgb)


if __name__ == "__main__":
    main()
