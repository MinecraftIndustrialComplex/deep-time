#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Open-space census of an area export (`deeptime-inspect export-area`, .dtva or .dtva.gz).

    python3 tools/workbench/cave_stats.py <area.dtva.gz> [--json out.json] [--sea 63] [--slice-z Z --png out.ppm]

"Subsurface void" is every air or water block under a solid block of its own column (a cave, a
tunnel, or an arch cut into a cliff face: the ground above it is still there). Water over the sea
floor and air over the land are not counted. The voids are then split into connected bodies
(6-neighbour); for each body: size, y range, whether it opens to the open sea (touches water that
has no solid above it) or to the sky, and its "overhang" volume, the blocks that have solid above
them and open water or air beside them in a column that is itself open to the sea above (arches
under cliff overhangs).  Prints totals by medium and by depth band. Standard library only.
"""
import array
import collections
import gzip
import json
import struct
import sys

BANDS = [(-512, -300), (-300, -200), (-200, -100), (-100, 0), (0, 50), (50, 64), (64, 100), (100, 700)]


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


def main():
    args = sys.argv[1:]
    path = args[0]
    out_json = args[args.index("--json") + 1] if "--json" in args else None
    sea = int(args[args.index("--sea") + 1]) if "--sea" in args else 63
    h, rc, rv, rl = load(path)
    w, dep, y0, y1 = h["width"], h["depth"], h["y0"], h["y1"]
    ht = y1 - y0 + 1
    cat = [h["blocks"][e[0]]["cat"] for e in h["entries"]]
    # 0 solid, 1 air, 2 water, 3 lava
    grid = bytearray(w * dep * ht)
    top = [-10 ** 9] * (w * dep)   # y of the highest solid block of each column
    k = 0
    for c in range(w * dep):
        y = 0
        for _ in range(rc[c]):
            v, n = rv[k], rl[k]
            k += 1
            ct = cat[v]
            code = 1 if ct == "air" else 2 if ct == "water" else 3 if ct == "lava" else 0
            if code:
                base = c * ht + y
                grid[base:base + n] = bytes([code]) * n
            else:
                top[c] = y + n - 1
            y += n

    # Subsurface void: open, with a solid block above it in the same column.
    void = bytearray(len(grid))
    by_medium = collections.Counter()
    by_band = collections.defaultdict(collections.Counter)
    for c in range(w * dep):
        t = top[c]
        base = c * ht
        for y in range(max(0, t)):
            g = grid[base + y]
            if g:
                void[base + y] = g
                by_medium[g] += 1
                for a, b in BANDS:
                    if a <= y + y0 < b:
                        by_band[(a, b)][g] += 1
                        break
    names = {1: "air", 2: "water", 3: "lava"}
    total = sum(by_medium.values())
    print("box", h["box"], "y", y0, y1)
    print(f"subsurface void {total:,} blocks: " + ", ".join(f"{names[g]} {n:,}" for g, n in sorted(by_medium.items())))
    print("band          air      water       lava")
    for a, b in BANDS:
        cnt = by_band[(a, b)]
        print(f"{a:5d}..{b:<5d} {cnt[1]:9,d} {cnt[2]:10,d} {cnt[3]:10,d}")

    # Connected bodies of void (any open medium counts: a flooded arch is one with its air).
    seen = bytearray(len(grid))
    bodies = []
    for start in range(len(grid)):
        if not void[start] or void[start] == 3 or seen[start]:
            continue
        stack = [start]
        seen[start] = 1
        n = sea_open = sky_open = 0
        mouth = [0, 0, 0]
        lo = [10 ** 9] * 3
        hi = [-10 ** 9] * 3
        med = collections.Counter()
        while stack:
            q = stack.pop()
            n += 1
            med[void[q]] += 1
            c, y = divmod(q, ht)
            z, i = divmod(c, w)
            for a, val in enumerate((i, y, z)):
                lo[a] = min(lo[a], val)
                hi[a] = max(hi[a], val)
            for di, dz, dy in ((1, 0, 0), (-1, 0, 0), (0, 1, 0), (0, -1, 0), (0, 0, 1), (0, 0, -1)):
                ii, zz, yy = i + di, z + dz, y + dy
                if not (0 <= ii < w and 0 <= zz < dep and 0 <= yy < ht):
                    continue
                r = (zz * w + ii) * ht + yy
                if void[r] and void[r] != 3:
                    if not seen[r]:
                        seen[r] = 1
                        stack.append(r)
                elif grid[r] and grid[r] != 3:
                    # open space with no solid above in its own column: the sea or the sky
                    if top[zz * w + ii] < yy:
                        if grid[r] == 2:
                            sea_open += 1
                            mouth[0] += i
                            mouth[1] += y
                            mouth[2] += z
                        else:
                            sky_open += 1
        bodies.append({"blocks": n, "y": [lo[1] + y0, hi[1] + y0], "x": [lo[0] + h["box"]["x0"], hi[0] + h["box"]["x0"]],
                       "z": [lo[2] + h["box"]["z0"], hi[2] + h["box"]["z0"]], "air": med[1], "water": med[2],
                       "sea_faces": sea_open, "sky_faces": sky_open,
                       "mouth": [h["box"]["x0"] + mouth[0] // max(1, sea_open), mouth[1] // max(1, sea_open) + y0,
                                 h["box"]["z0"] + mouth[2] // max(1, sea_open)] if sea_open else None})
    bodies.sort(key=lambda b: -b["blocks"])
    to_sea = [b for b in bodies if b["sea_faces"]]
    to_sky = [b for b in bodies if b["sky_faces"]]
    print(f"void bodies {len(bodies)}; opening to the open sea {len(to_sea)} ({sum(b['blocks'] for b in to_sea):,} blocks); "
          f"opening to the sky {len(to_sky)} ({sum(b['blocks'] for b in to_sky):,} blocks)")
    print("   blocks      air     water   y lo..hi        x range          z range   seaFaces skyFaces")
    for b in bodies[:20]:
        print(f"{b['blocks']:9,d} {b['air']:8,d} {b['water']:9,d}  {b['y'][0]:5d}..{b['y'][1]:<5d} {b['x'][0]:7d}..{b['x'][1]:<7d} "
              f"{b['z'][0]:6d}..{b['z'][1]:<6d} {b['sea_faces']:7d} {b['sky_faces']:7d}")
    print("largest sea mouths (centre of the faces on the open sea):")
    for b in sorted(to_sea, key=lambda b: -b["sea_faces"])[:12]:
        print(f"  faces {b['sea_faces']:6d}  body {b['blocks']:7,d}  mouth at x {b['mouth'][0]} y {b['mouth'][1]} z {b['mouth'][2]}")
    # Submarine: void under the sea whose neighbouring open water column is the sea (an arch).
    arch = 0
    for c in range(w * dep):
        t = top[c]
        if t + y0 >= sea:
            continue
        base = c * ht
        for y in range(max(0, t)):
            if void[base + y] == 2:
                arch += 1
    print(f"water-filled void under the sea floor of columns that are themselves under the sea: {arch:,}")
    if out_json:
        with open(out_json, "w") as f:
            json.dump({"box": h["box"], "total": total, "by_medium": {names[g]: n for g, n in by_medium.items()},
                       "bodies_total": len(bodies), "to_sea": len(to_sea), "bodies": bodies[:200]}, f, indent=1)


if __name__ == "__main__":
    main()
