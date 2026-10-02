#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""A vertical section (or a column census) of an area export (`deeptime-inspect export-area`).

    python3 tools/workbench/area_section.py <area.dtva.gz> --z Z|--x X [--y lo,hi] [--scale N] --out section.png
    python3 tools/workbench/area_section.py <area.dtva.gz> --census [--json out.json] [--around X,Z,R]

A section is the wall of blocks in the plane z = Z (or x = X): each block a scale x scale square in
the block's own colour (lava orange, magma blocks red, basalt dark, water blue, air white), y up;
the picture is 1 block = `scale` pixels wide and high, so a world 15 times taller than wide looks
as it is. `--census` counts blocks by id: the box, or the cylinder of radius R round (X, Z).
Standard library only (the PNG is written with zlib).
"""
import argparse
import array
import collections
import gzip
import json
import struct
import sys
import zlib

OVERRIDE = {
    "minecraft:lava": (255, 106, 0),
    "minecraft:magma_block": (200, 50, 30),
    "minecraft:basalt": (40, 40, 48),
    "minecraft:smooth_basalt": (72, 72, 80),
    "minecraft:air": (244, 244, 244),
    "minecraft:water": (58, 111, 216),
    "minecraft:bedrock": (20, 20, 20),
}


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


def column_runs(rc, rv, rl):
    """Yields, per column, its runs as a list of (entry, length)."""
    k = 0
    for c in range(len(rc)):
        runs = []
        for _ in range(rc[c]):
            runs.append((rv[k], rl[k]))
            k += 1
        yield runs


def png(path, width, height, rows):
    raw = b"".join(b"\x00" + bytes(r) for r in rows)

    def chunk(tag, data):
        c = struct.pack(">I", len(data)) + tag + data
        return c + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)

    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n")
        f.write(chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0)))
        f.write(chunk(b"IDAT", zlib.compress(raw, 6)))
        f.write(chunk(b"IEND", b""))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("file")
    ap.add_argument("--z", type=int)
    ap.add_argument("--x", type=int)
    ap.add_argument("--y", default=None)
    ap.add_argument("--scale", type=int, default=3)
    ap.add_argument("--span", default=None, help="lo,hi: crop the section to these x (or z) values")
    ap.add_argument("--out")
    ap.add_argument("--census", action="store_true")
    ap.add_argument("--json")
    ap.add_argument("--around")
    a = ap.parse_args()
    h, rc, rv, rl = load(a.file)
    w, dep, y0, y1 = h["width"], h["depth"], h["y0"], h["y1"]
    bx, bz = h["box"]["x0"], h["box"]["z0"]
    ids = [b["id"] for b in h["blocks"]]
    colors = []
    for e in h["entries"]:
        b = h["blocks"][e[0]]
        c = OVERRIDE.get(b["id"])
        if c is None:
            hx = b["color"].lstrip("#")
            c = tuple(int(hx[i:i + 2], 16) for i in (0, 2, 4))
        colors.append(c)
    entry_id = [ids[e[0]] for e in h["entries"]]

    if a.census:
        cnt = collections.Counter()
        cx = cz = r2 = None
        if a.around:
            cx, cz, r = (int(v) for v in a.around.split(","))
            r2 = r * r
        for c, runs in enumerate(column_runs(rc, rv, rl)):
            if r2 is not None:
                x, z = bx + c % w, bz + c // w
                if (x - cx) ** 2 + (z - cz) ** 2 > r2:
                    continue
            for v, n in runs:
                cnt[entry_id[v]] += n
        for k, n in cnt.most_common(40):
            print(f"{n:12,d} {k}")
        if a.json:
            json.dump(dict(cnt), open(a.json, "w"), indent=1)
        return

    lo, hi = (int(v) for v in a.y.split(",")) if a.y else (y0, y1)
    lo, hi = max(lo, y0), min(hi, y1)
    if a.z is not None:
        zz = a.z - bz
        cols = [zz * w + i for i in range(w)]
    elif a.x is not None:
        xx = a.x - bx
        cols = [z * w + xx for z in range(dep)]
    else:
        sys.exit("give --z or --x or --census")
    if a.span:
        base = bx if a.z is not None else bz
        lo_s, hi_s = (int(v) - base for v in a.span.split(","))
        cols = cols[max(0, lo_s):hi_s + 1]
    n = len(cols)
    s = a.scale
    grid = [[None] * (hi - lo + 1) for _ in range(n)]
    runs_all = list(column_runs(rc, rv, rl))
    for i, c in enumerate(cols):
        y = y0
        for v, ln in runs_all[c]:
            for yy in range(max(y, lo), min(y + ln - 1, hi) + 1):
                grid[i][yy - lo] = colors[v]
            y += ln
    rows = []
    for yy in range(hi, lo - 1, -1):
        line = bytearray()
        for i in range(n):
            col = grid[i][yy - lo] or (244, 244, 244)
            line += bytes(col) * s
        for _ in range(s):
            rows.append(line)
    png(a.out, n * s, (hi - lo + 1) * s, rows)
    print(f"{a.out}: {n * s} x {(hi - lo + 1) * s} px, y {lo}..{hi}")


if __name__ == "__main__":
    main()
